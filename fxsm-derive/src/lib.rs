extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;
extern crate fxsm;

use proc_macro::TokenStream;
use quote::{ Tokens, ToTokens };
use std::collections::BTreeMap;

const FSM_ATTRIBUTE: &'static str = "state_transitions";
const NEW_STATE_OBJ_NAME: &'static str = "new_state";

#[derive(Copy, Clone)]
enum EnumFieldData {
    Unit,
    Struct,
    Tuple,
}

enum Fragment {
    /// Tokens that can be used as an expression.
    Expr(Tokens),
}

macro_rules! quote_expr {
    ($($tt:tt)*) => {
        Fragment::Expr(quote!($($tt)*))
    }
}

macro_rules! quote_block {
    ($($tt:tt)*) => {
        Fragment::Block(quote!($($tt)*))
    }
}

/// Interpolate a fragment as the statements of a block.
struct Stmts(pub Fragment);
impl ToTokens for Stmts {
    fn to_tokens(&self, out: &mut Tokens) {
        match self.0 {
            Fragment::Expr(ref expr) => expr.to_tokens(out),
        }
    }
}

fn ident_with_data(enum_ident: &syn::Ident,
                   item_ident: &syn::Ident,
                   data: EnumFieldData) -> syn::Ident {
    match data {
        EnumFieldData::Unit => syn::Ident::new(format!("{}::{}", enum_ident, item_ident)),
        EnumFieldData::Tuple => syn::Ident::new(format!("{}::{}(_)", enum_ident, item_ident)),
        EnumFieldData::Struct => syn::Ident::new(format!("{}::{}{{..}}", enum_ident, item_ident)),
    }
}

fn serialize_variant(ident: &syn::Ident,
                     new_state_ident: &syn::Ident,
                     variant: &syn::Variant,
                     var_data_map: &BTreeMap<String, EnumFieldData>) -> Tokens {
    let variant_ident = ident_with_data(ident,
                                        &variant.ident,
                                        *var_data_map.get(&variant.ident.to_string())
                                                     .unwrap_or(&EnumFieldData::Unit));
    let transitions: Vec<_> = variant.attrs.iter()
        .filter(|a| a.value.name() == FSM_ATTRIBUTE)
        .map(|a| {
            if let syn::MetaItem::List(_, ref nested) = a.value {
                let transitions: Vec<_> = nested.iter()
                    .map(|n| {
                        if let syn::NestedMetaItem::MetaItem(ref mt) = *n {
                            if let syn::MetaItem::Word(ref id) = *mt {
                                let data = *var_data_map.get(&id.to_string())
                                                            .unwrap_or(&EnumFieldData::Unit);
                                let match_id = ident_with_data(ident,
                                                               id,
                                                               data);
                                    Stmts(quote_expr! {
                                        #match_id
                                    })
                            } else {
                                panic!("The syntax for fsm states attribute must be a list-syntax (the same as for #[derive(...)]");
                            }
                        } else {
                            panic!("The syntax for fsm states attribute must be a list-syntax (the same as for #[derive(...)]");
                        }
                    })
                    .collect();
                    if !transitions.is_empty() {
                        return quote! {
                            #variant_ident => match #new_state_ident {
                                #(
                                    #transitions => true,
                                )*
                                _ => false,
                            },
                        }
                    } else {
                        return quote! {
                            _ => false,
                        }
                    }
            } else {
                panic!("The syntax for fsm states attribute must be a list-syntax (the same as for #[derive(...)]");
            }
        })
        .collect();

    quote! {
        #(#transitions)*
    }
}

fn serialize_enum(ident: &syn::Ident,
                  variants: &[syn::Variant]) -> Fragment {
    let var_data_map: BTreeMap<String, EnumFieldData> = variants.iter()
        .map(|var| {
            let field_data = match var.data {
                syn::VariantData::Unit => EnumFieldData::Unit,
                syn::VariantData::Struct(_) => EnumFieldData::Struct,
                syn::VariantData::Tuple(_) => EnumFieldData::Tuple,
            };
            (var.ident.to_string(), field_data)
        })
        .collect();

    let arms: Vec<_> = variants.iter()
        .map(|variant| serialize_variant(ident,
                                         &syn::Ident::new(NEW_STATE_OBJ_NAME),
                                         variant,
                                         &var_data_map))
        .collect();

    quote_expr! {
        match *self {
            #(#arms)*
            _ => false,
        }
    }
}

fn get_finish_states(name: &syn::Ident,
                     variants: &[syn::Variant]) -> Fragment {
    let var_data_map: BTreeMap<String, EnumFieldData> = variants.iter()
        .map(|var| {
            let field_data = match var.data {
                syn::VariantData::Unit => EnumFieldData::Unit,
                syn::VariantData::Struct(_) => EnumFieldData::Struct,
                syn::VariantData::Tuple(_) => EnumFieldData::Tuple,
            };
            (var.ident.to_string(), field_data)
        })
        .collect();

    let arms: Vec<syn::Ident> = variants.iter()
        .filter(|variant| !variant.attrs.iter()
                          .find(|a| a.value.name() == FSM_ATTRIBUTE).is_some())
        .map(|variant| {
            ident_with_data(name,
                            &variant.ident,
                            *var_data_map.get(&variant.ident.to_string())
                                 .unwrap_or(&EnumFieldData::Unit))
        })
        .collect();
        
    let new_state_obj_name = syn::Ident::new(NEW_STATE_OBJ_NAME);
    quote_expr! {
        match #new_state_obj_name {
            #(#arms => true,)*
            _ => false,
        }
    }
}

fn gen_for_copyable(name: &syn::Ident,
                    variants: &[syn::Variant],
                    generics: &syn::Generics) -> Tokens {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let transitions = Stmts(serialize_enum(name, variants));
    let finish_states = Stmts(get_finish_states(name, variants));
    let new_state_obj_name = syn::Ident::new(NEW_STATE_OBJ_NAME);
    quote! {
        impl #impl_generics fxsm::FiniteStateMachine<#name #ty_generics> for #name #ty_generics #where_clause {
            fn change(&mut self, #new_state_obj_name: #name #ty_generics) -> bool {
                if self.can_change(#new_state_obj_name) {
                    *self = #new_state_obj_name;
                    return true
                }
                false
            }
            fn can_change(&self, #new_state_obj_name: #name #ty_generics) -> bool {
                #transitions
            }
            fn is_finish_state(#new_state_obj_name: #name #ty_generics) -> bool {
                #finish_states
            }
            fn at_finish_state(&self) -> bool {
                Self::is_finish_state(*self)
            }
        }
    }
}

fn gen_for_clonable(name: &syn::Ident,
                    variants: &[syn::Variant],
                    generics: &syn::Generics) -> Tokens {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let transitions = Stmts(serialize_enum(name, variants));
    let finish_states = Stmts(get_finish_states(name, variants));
    let new_state_obj_name = syn::Ident::new(NEW_STATE_OBJ_NAME);
    quote! {
        impl #impl_generics fxsm::FiniteStateMachine<#name #ty_generics> for #name #ty_generics #where_clause {
            fn change(&mut self, #new_state_obj_name: #name #ty_generics) -> bool {
                if self.can_change(#new_state_obj_name.clone()) {
                    *self = #new_state_obj_name.clone();
                    return true
                }
                false
            }
            fn can_change(&self, #new_state_obj_name: #name #ty_generics) -> bool {
                #transitions
            }
            fn is_finish_state(#new_state_obj_name: #name #ty_generics) -> bool {
                #finish_states
            }
            fn at_finish_state(&self) -> bool {
                Self::is_finish_state(self.clone())
            }
        }
    }
}

fn derives(nested: &[syn::NestedMetaItem], trait_name: &str) -> bool {
    nested.iter().find(|n| {
        if let syn::NestedMetaItem::MetaItem(ref mt) = **n {
            if let syn::MetaItem::Word(ref id) = *mt {
                return id == trait_name;
            }
            return false
        }
        false
    }).is_some()
}

#[proc_macro_derive(FiniteStateMachine, attributes(state_transitions))]
pub fn fxsm(input: TokenStream) -> TokenStream {
    // Construct a string representation of the type definition
    let s = input.to_string();

    // Parse the string representation
    let ast = syn::parse_derive_input(&s).unwrap();

    // Build the impl
    let gen = impl_fsm(&ast);

    // Return the generated impl
    gen.parse().unwrap()
}

fn impl_fsm(ast: &syn::DeriveInput) -> Tokens {
    if let syn::Body::Enum(ref variants) = ast.body {
        if let Some(ref a) = ast.attrs.iter().find(|a| a.name() == "derive") {
            if let syn::MetaItem::List(_, ref nested) = a.value {
                if derives(nested, "Copy") {
                    return gen_for_copyable(&ast.ident, &variants, &ast.generics);
                } else if derives(nested, "Clone") {
                    return gen_for_clonable(&ast.ident, &variants, &ast.generics);
                } else {
                    panic!("Unable to produce Finite State Machine code on a enum which does not drive Copy nor Clone traits.");
                }
            } else {
                panic!("Unable to produce Finite State Machine code on a enum which does not drive Copy nor Clone traits.");
            }
        } else {
            panic!("How were you able to call me without derive!?!?");
        }
    } else {
        panic!("Finite State Machine must be derived on a enum.");
    }
}
