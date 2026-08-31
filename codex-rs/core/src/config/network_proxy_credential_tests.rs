use codex_security_policy::PolicyPrincipal;
use codex_security_policy::PrincipalKind;
use codex_security_policy::RevocationEvent;
use codex_security_policy::RevocationReason;
use codex_security_policy::RevocationTarget;
use pretty_assertions::assert_eq;

use super::*;

fn human() -> PolicyPrincipal {
    PolicyPrincipal::new(PrincipalKind::Human, "human:owner").expect("human")
}

#[test]
fn credential_authority_revoke_during_use_linearizes_after_the_active_resolution() {
    let revocations = RwLock::new(RevocationState::new());

    with_current_revocations(&revocations, |current| {
        assert_eq!(current.generation, 0);
        assert!(
            revocations.try_write().is_err(),
            "revocation must not mutate the state observed by an active resolution"
        );
    })
    .expect("active resolution");

    let event = RevocationEvent::new(
        human(),
        RevocationTarget::AllActiveAuthority,
        RevocationReason::HumanRequest,
        101,
    )
    .expect("revocation event");
    revocations
        .write()
        .expect("revocation write after resolution")
        .apply(&event)
        .expect("apply revocation");

    with_current_revocations(&revocations, |current| {
        assert_eq!(current.generation, 1);
    })
    .expect("next resolution observes revocation");
}

#[test]
fn pf_27_s04_broker_failures_map_without_raw_credential_fallback() {
    assert_eq!(
        map_broker_client_error(BrokerClientError::Denied),
        IsolatedCredentialDispatchError::Denied
    );
    assert_eq!(
        map_broker_client_error(BrokerClientError::Cancelled),
        IsolatedCredentialDispatchError::Cancelled
    );
    assert_eq!(
        map_broker_client_error(BrokerClientError::OutcomeUnknown),
        IsolatedCredentialDispatchError::OutcomeUnknown
    );
    assert_eq!(
        map_broker_client_error(BrokerClientError::Unavailable),
        IsolatedCredentialDispatchError::Unavailable
    );
}
