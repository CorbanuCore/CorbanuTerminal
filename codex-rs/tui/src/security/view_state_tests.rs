use codex_protocol::security::AuthorityEpoch;
use codex_protocol::security::SecurityControlAction;
use codex_protocol::security::SecurityInspectorSnapshot;
use codex_protocol::security::SecurityLevel;
use pretty_assertions::assert_eq;

use super::*;

fn event(runtime: u8) -> SecurityInspectorEvent {
    SecurityInspectorEvent {
        snapshot: SecurityInspectorSnapshot::new(
            SecurityLevel::Moderate,
            SecurityLevel::Aggressive,
            Default::default(),
        )
        .unwrap(),
        epoch: AuthorityEpoch::new(
            [runtime; 16],
            /*policy_revision*/ 0,
            /*revocation_generation*/ 0,
        )
        .unwrap(),
    }
}

#[test]
fn unavailable_inspector_never_manufactures_a_request_or_effective_protection() {
    let state = SecurityViewState::default();
    assert_eq!(state.observation(), None);
    assert_eq!(
        state
            .prepare_request(SecurityControlAction::SetLevel {
                level: SecurityLevel::Permissive
            })
            .unwrap(),
        None
    );
}

#[test]
fn prepare_and_cancel_are_observational_and_reconnect_discards_stale_state() {
    let mut state = SecurityViewState::default();
    let initial = event(/*runtime*/ 1);
    state.observe(initial.clone());
    let action = SecurityControlAction::SetLevel {
        level: SecurityLevel::Permissive,
    };
    let pending = state.prepare_request(action.clone()).unwrap().unwrap();
    assert_eq!(pending.expected_epoch(), initial.epoch);
    assert_eq!(state.observation(), Some(&initial));
    // Cancel drops a proposal; it never changes a persisted or observed level.
    drop(pending);
    assert_eq!(state.observation(), Some(&initial));
    state.invalidate();
    assert_eq!(state.observation(), None);
    assert_eq!(state.prepare_request(action.clone()).unwrap(), None);
    let resumed = event(/*runtime*/ 2);
    state.observe(resumed.clone());
    assert_eq!(
        state
            .prepare_request(action)
            .unwrap()
            .unwrap()
            .expected_epoch(),
        resumed.epoch
    );
    assert_eq!(state.observation(), Some(&resumed));
}
