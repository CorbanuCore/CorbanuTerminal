use super::*;
use crate::security::effective_policy::EffectivePolicyInitialization;
use crate::security::effective_policy::PersistedHumanSecurityState;
use crate::security::effective_policy::TrustedSecurityController;
use codex_protocol::SessionId;
use codex_security_policy::PolicyPrincipal;
use codex_security_policy::PrincipalKind;
use codex_security_policy::RevocationState;
use codex_security_policy::SecuritySettings;
use pretty_assertions::assert_eq;

fn authority() -> (BrowserAuthority, TrustedSecurityController) {
    let view = EffectivePolicyView::default();
    let agent = ThreadId::new();
    let state = PersistedHumanSecurityState::new(
        SecuritySettings::new(SecurityLevel::Moderate),
        PolicyPrincipal::new(PrincipalKind::Human, "human:tester").unwrap(),
        RevocationState::new(),
    )
    .unwrap();
    let controller = TrustedSecurityController::initialize(
        &view,
        state,
        agent,
        SessionId::from(agent),
        EffectivePolicyInitialization::Root,
    )
    .unwrap();
    (BrowserAuthority::new(view, agent), controller)
}

#[test]
fn native_authority_follows_policy_changes_and_new_runtime_incarnations() {
    let (authority, controller) = authority();
    let original = authority.current().unwrap();
    let change = controller
        .confirm_level_change(SecurityLevel::Aggressive, RevocationState::new())
        .unwrap();
    controller.apply_confirmed_change(change).unwrap();
    let current = authority.current().unwrap();
    assert_eq!(current.0, SecurityLevel::Aggressive);
    assert_ne!(current.1, original.1);
    let (resumed, _) = self::authority();
    assert_ne!(resumed.current().unwrap().1, original.1);
}

#[test]
fn unknown_or_uninitialized_agent_is_denied() {
    let authority = BrowserAuthority::new(EffectivePolicyView::default(), ThreadId::new());
    assert_eq!(authority.current(), Err(BrowserError::StaleAuthority));
}

#[test]
fn backend_observation_does_not_claim_other_controls() {
    let (authority, _) = authority();
    let (_, epoch) = authority.current().unwrap();
    let event = inspector(
        SecurityLevel::Moderate,
        SecurityLevel::Moderate,
        epoch,
        SecurityControlHealth::Enforcing {},
    )
    .unwrap();
    assert_eq!(
        *event.snapshot.controls(),
        SecurityControlHealthSnapshot {
            browser_isolation: SecurityControlHealth::Enforcing {},
            content_firewall: SecurityControlHealth::Unavailable {},
            confidentiality: SecurityControlHealth::Unavailable {},
            protected_actions: SecurityControlHealth::Unavailable {},
        }
    );
    assert_eq!(event.epoch, epoch);
}
