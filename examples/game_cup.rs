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
/* Here is a short diagram without `Rescheduled` part.
 * (Waiting) -> (Checkins) -> (InProgress)
 *    |              |         |    |
 *     \             |        /     |
 *      \            |       /      |
 *       ------->(Aborted)<--    (Finished)
 */

#[derive(Copy, Clone, Debug, FiniteStateMachine)]
enum CupStateNew<W: Copy, C: Copy, I: Copy, F: Copy> {
    #[state_transitions(Finished, Checkins)]
    Waiting(W),
    Checkins(C),
    InProgress(I),
    Finished{ f: F },
}

fn main() {
    use fxsm::{ FiniteStateMachine };
    let mut fsm = CupState::Waiting;
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

    let fsm_new: CupStateNew<u64, u64, u64, u64> = CupStateNew::Waiting(0);
    // must not be able to change to itself
    assert!(!fsm_new.can_change(CupStateNew::Waiting(0u64)));
    assert!(fsm_new.can_change(CupStateNew::Checkins(0u64)));
    assert!(fsm_new.can_change(CupStateNew::Finished{ f: 0u64 }));
    assert!(!fsm_new.can_change(CupStateNew::InProgress(0u64)));
}
