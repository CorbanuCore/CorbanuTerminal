use super::RuntimeState;
use super::runtime_state;

#[test]
fn status_priority_preserves_actionable_state() {
    assert_eq!(
        runtime_state(/*approvals*/ 1, /*active_turn*/ true, /*queued*/ 1, /*has_error*/ true),
        RuntimeState::AwaitingApproval
    );
    assert_eq!(runtime_state(/*approvals*/ 0, /*active_turn*/ true, /*queued*/ 1, /*has_error*/ true), RuntimeState::WorkingQueued);
    assert_eq!(runtime_state(/*approvals*/ 0, /*active_turn*/ true, /*queued*/ 0, /*has_error*/ true), RuntimeState::Working);
    assert_eq!(runtime_state(/*approvals*/ 0, /*active_turn*/ false, /*queued*/ 1, /*has_error*/ true), RuntimeState::Blocked);
    assert_eq!(runtime_state(/*approvals*/ 0, /*active_turn*/ false, /*queued*/ 1, /*has_error*/ false), RuntimeState::Recovering);
    assert_eq!(runtime_state(/*approvals*/ 0, /*active_turn*/ false, /*queued*/ 0, /*has_error*/ true), RuntimeState::Blocked);
    assert_eq!(runtime_state(/*approvals*/ 0, /*active_turn*/ false, /*queued*/ 0, /*has_error*/ false), RuntimeState::Idle);
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
