use super::RuntimeState;
use super::runtime_state;

#[test]
fn status_priority_preserves_actionable_state() {
    assert_eq!(
        runtime_state(1, true, 1, true),
        RuntimeState::AwaitingApproval
    );
    assert_eq!(runtime_state(0, true, 1, true), RuntimeState::WorkingQueued);
    assert_eq!(runtime_state(0, true, 0, true), RuntimeState::Working);
    assert_eq!(runtime_state(0, false, 1, true), RuntimeState::Blocked);
    assert_eq!(runtime_state(0, false, 1, false), RuntimeState::Recovering);
    assert_eq!(runtime_state(0, false, 0, true), RuntimeState::Blocked);
    assert_eq!(runtime_state(0, false, 0, false), RuntimeState::Idle);
}

#[test]
fn every_status_has_a_concrete_next_action() {
    for state in [
        RuntimeState::Idle,
        RuntimeState::Working,
        RuntimeState::WorkingQueued,
        RuntimeState::AwaitingApproval,
        RuntimeState::Recovering,
        RuntimeState::Blocked,
    ] {
        assert!(!state.next_action().is_empty());
    }
}
