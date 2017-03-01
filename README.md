# fxsm

[![](https://meritbadge.herokuapp.com/fxsm)](https://crates.io/crates/fxsm) [![](https://travis-ci.org/vityafx/fxsm.svg?branch=master)](https://travis-ci.org/vityafx/urlshortener-rs)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)


A very simple state machine procedural macro for enums.

## How does it work

It simply generates match conditions on your enums in appropriate `StateMachine` trait's methods.
 You may use the state machine through a [`StateMachine`](https://github.com/vityafx/fxsm/blob/master/fxsm/src/lib.rs) trait.
  

## Usage

1. Add `fxsm` as dependency in your `Cargo.toml`:

 ```toml
 [dependencies]
 fxsm-derive = "0.2"
 fxsm = "0.2"
 ```

2. Create a State Machine:

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
     use fxsm::{ StateMachine };
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
 
 More and updated examples are in [examples directory](https://github.com/vityafx/fxsm/blob/master/examples).

## License

This project is [licensed under the MIT license](https://github.com/vityafx/urlshortener-rs/blob/master/LICENSE).
