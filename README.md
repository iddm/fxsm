# fxsm

[![](https://meritbadge.herokuapp.com/fxsm)](https://crates.io/crates/fxsm) [![](https://travis-ci.org/vityafx/fxsm.svg?branch=master)](https://travis-ci.org/vityafx/urlshortener-rs)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)


A FSM procedural macro for enums.

This library aims to implement a very simple FSM for your rust enums.

## How does it work

When you derive a `FiniteStateMachine` macro it performs a check what implementation to provide based on what your enum derives.
 If your enum implements `Copy` trait then the library will provide a code with copying, otherwise with cloning impls.
 You may use the FSM through a `FiniteStateMachine` trait:
 
 ```rust
 pub trait FiniteStateMachine<S> {
    /// Returns true if it changed state successfully; false otherwise.
    fn change(&mut self, new_state: S) -> bool;
    /// Returns true if it is possible to change state to `new_state`; false otherwise.
    fn can_change(&self, new_state: S) -> bool;
    /// Returns true if it is a one of finish states.
    fn is_finish_state(state: S) -> bool;
    /// Returns true if the FSM is in a finish state.
    fn at_finish_state(&self) -> bool;
    /// Returns a number of total finish states.
    fn finish_states() -> usize;
}
```
 

## Usage

1. Add `fxsm` as dependency in your `Cargo.toml`:

 ```toml
 [dependencies]
 fxsm-derive = "0.2"
 fxsm = "0.2"
 ```

2. Create a Finite-State Machine:

 ```rust
 #[macro_use]
 extern crate fxsm_derive;
 extern crate fxsm;
 
 
 #[derive(Clone, Debug, FiniteStateMachine)]
 enum CupState {
     #[state_transitions(Checkins, Aborted, Rescheduled)]
     Waiting,
     #[state_transitions(InProgress, Aborted, Rescheduled)]
     Checkins,
     #[state_transitions(Finished, Aborted, Rescheduled)]
     InProgress(String),
 
     // Finish-states
     Aborted(u64),
     Rescheduled { info: String },
     Finished,
 }
 ```

3. Use it:

 ```rust
 fn main() {
     use fxsm::{ FiniteStateMachine };
     let mut fsm = CupState::Waiting;
     assert_eq!(CupState::finish_states(), 3);
     // must not be able to change to itself
     assert!(!fsm.can_change(CupState::Waiting));
     assert!(fsm.can_change(CupState::Checkins));
     assert!(!fsm.can_change(CupState::InProgress(String::default())));
     assert!(fsm.can_change(CupState::Aborted(0u64)));
     assert!(fsm.can_change(CupState::Rescheduled{info: String::default()}));
     assert!(!fsm.can_change(CupState::Finished));
     assert!(!fsm.at_finish_state());
 
     assert!(fsm.change(CupState::Checkins));
     assert!(!fsm.can_change(CupState::Waiting));
     assert!(fsm.can_change(CupState::Aborted(0u64)));
     assert!(fsm.can_change(CupState::Rescheduled{info: String::default()}));
     assert!(fsm.can_change(CupState::InProgress(String::default())));
     assert!(!fsm.can_change(CupState::Finished));
     assert!(!fsm.at_finish_state());
 
     // You still always able to change it without FSM rules:
     fsm = CupState::Finished;
     assert!(fsm.at_finish_state());
     assert!(CupState::is_finish_state(CupState::Finished));
     assert!(CupState::is_finish_state(CupState::Aborted(0u64)));
     assert!(CupState::is_finish_state(CupState::Rescheduled{ info: String::default()}));
     assert!(!CupState::is_finish_state(CupState::Waiting));
     assert!(!CupState::is_finish_state(CupState::Checkins));
     assert!(!CupState::is_finish_state(CupState::InProgress(String::default())));
 }
 ```

## License

This project is [licensed under the MIT license](https://github.com/vityafx/urlshortener-rs/blob/master/LICENSE).
