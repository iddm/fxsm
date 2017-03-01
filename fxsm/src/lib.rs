pub trait StateMachine<S> {
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
