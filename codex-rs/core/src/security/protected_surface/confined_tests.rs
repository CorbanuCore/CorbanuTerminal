use super::*;
use codex_security_policy::ActorChain;
use codex_security_policy::BoundedGrant;
use codex_security_policy::BoundedText;
use codex_security_policy::GrantContext;
use codex_security_policy::GrantScope;
use codex_security_policy::PolicyPrincipal;
use codex_security_policy::PrincipalKind;
use pretty_assertions::assert_eq;
use std::collections::BTreeMap;

fn human() -> PolicyPrincipal {
    PolicyPrincipal::new(PrincipalKind::Human, "human").unwrap()
}

fn lineage() -> ActorChain {
    ActorChain::new(vec![
        human(),
        PolicyPrincipal::new(PrincipalKind::Agent, "main").unwrap(),
    ])
    .unwrap()
}

fn state(level: SecurityLevel, taint_generation: u64, epoch: u64) -> PostTaintState {
    PostTaintState {
        taint_generation,
        policy: PolicyBinding::Bound {
            epoch,
            revocation_generation: 0,
            kill_switch_active: false,
            level,
            actor_chain: lineage(),
        },
        level,
    }
}

/// Moderate: a process started without the rules asks once, is then
/// allowed for this start and policy epoch only; a confined one never asks.
#[test]
fn pf_23_s02_moderate_asks_once_per_unconfined_start() {
    let thread = ThreadId::new();
    let moderate = state(SecurityLevel::Moderate, 1, 0);
    assert_eq!(
        typing(thread, 7, &state(SecurityLevel::Moderate, 0, 0)),
        Typing::Clear
    );
    // Unknown process: unconfined.
    assert_eq!(typing(thread, 7, &moderate), Typing::AskOnce);
    note_start(thread, 7, /*confined*/ true);
    assert_eq!(typing(thread, 7, &moderate), Typing::Clear);

    // The id is reused by a process started without the rules.
    note_start(thread, 7, /*confined*/ false);
    assert_eq!(typing(thread, 7, &moderate), Typing::AskOnce);
    note_allowed(thread, 7, &moderate);
    assert_eq!(typing(thread, 7, &moderate), Typing::Clear);
    // A policy change asks again; so does the next start with that id.
    assert_eq!(
        typing(thread, 7, &state(SecurityLevel::Moderate, 1, 1)),
        Typing::AskOnce
    );
    note_start(thread, 7, /*confined*/ false);
    assert_eq!(typing(thread, 7, &moderate), Typing::AskOnce);
    // Another session's marks do not count.
    assert_eq!(typing(ThreadId::new(), 7, &moderate), Typing::AskOnce);
}

/// Aggressive: typing into an unconfined process is refused unless a grant
/// names this exact start; the grant does not carry to a later start.
#[test]
fn pf_23_s02_aggressive_needs_a_grant_for_this_start() {
    let thread = ThreadId::new();
    let aggressive_state = state(SecurityLevel::Aggressive, 1, 0);
    note_start(thread, 3, /*confined*/ true);
    assert_eq!(typing(thread, 3, &aggressive_state), Typing::Clear);
    note_start(thread, 3, /*confined*/ false);
    assert!(matches!(
        typing(thread, 3, &aggressive_state),
        Typing::Refused(refusal) if refusal.contains("needs a grant")
    ));
    // The human's allowance (a Moderate answer) is not a grant.
    note_allowed(thread, 3, &aggressive_state);
    assert!(matches!(
        typing(thread, 3, &aggressive_state),
        Typing::Refused(_)
    ));

    let operation = grant_operation(thread, 3).unwrap();
    let text = |value: &str| BoundedText::new(value).unwrap();
    let surface = aggressive::Surface::UnconfinedProcess;
    let scope = GrantScope::new(
        surface.resource(),
        [surface.action()],
        GrantContext::new(
            text(&thread.to_string()),
            text(&thread.to_string()),
            text(aggressive::PURPOSE),
            text(&operation),
        ),
        None,
        BTreeMap::new(),
    )
    .unwrap();
    let now = aggressive::now_unix_seconds();
    let grant =
        BoundedGrant::issue(human(), lineage(), scope, now - 1, now + 600, text("n")).unwrap();
    aggressive::issue(thread, &aggressive_state, grant, now).unwrap();
    assert_eq!(typing(thread, 3, &aggressive_state), Typing::Clear);
    note_start(thread, 3, /*confined*/ false);
    assert!(matches!(
        typing(thread, 3, &aggressive_state),
        Typing::Refused(_)
    ));
}
