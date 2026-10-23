use super::*;
use codex_sandboxing::user_namespaces::Rule;
use pretty_assertions::assert_eq;

const REFUSED: &str = "unshare: unshare failed: Operation not permitted";

/// The rule's levels: Moderate and Aggressive, also where only the launch
/// enforces Aggressive; never Permissive alone. Only Linux enforces it.
#[test]
fn issue_826_the_rule_follows_the_protected_levels() {
    let cases = [
        (None, false, None),
        (Some(SecurityLevel::Permissive), false, None),
        (Some(SecurityLevel::Moderate), false, Some("Moderate")),
        (Some(SecurityLevel::Aggressive), false, Some("Aggressive")),
        (None, true, Some("Aggressive")),
        (Some(SecurityLevel::Permissive), true, Some("Aggressive")),
        (Some(SecurityLevel::Moderate), true, Some("Aggressive")),
    ];
    for (level, launch, named) in cases {
        assert_eq!(denying_level(level, launch), named, "{level:?} {launch}");
        assert_eq!(
            rule_applies(level, launch),
            cfg!(target_os = "linux") && named.is_some(),
            "{level:?} {launch}"
        );
    }
}

/// A failed command whose output reports the refusal is explained, naming
/// the level; a command that succeeded, one whose failure is not about a
/// namespace, and any command where the rule does not apply are not.
#[test]
fn issue_826_only_a_refused_namespace_under_the_rule_is_explained() {
    let explanation = explained(Some(Rule::ModerateOneCommand), Some(1), REFUSED).unwrap();
    assert!(
        explanation.starts_with("Security level Moderate blocks creating user namespaces"),
        "{explanation}"
    );
    assert_eq!(explained(Some(Rule::Aggressive), Some(0), REFUSED), None);
    assert_eq!(
        explained(Some(Rule::Aggressive), /*exit_code*/ None, REFUSED),
        None
    );
    assert_eq!(
        explained(
            Some(Rule::Aggressive),
            Some(1),
            "cat: /home/someone/.ssh/id_ed25519: Operation not permitted"
        ),
        None
    );
    assert_eq!(explained(/*rule*/ None, Some(1), REFUSED), None);
}

/// D27 (a): a run a person let create user namespaces is known by thread,
/// turn and call, once: so a refusal its host reports isn't put down to the
/// rule, and a reused call ID later is explained again.
#[test]
fn d27_allowed_runs_are_known_by_thread_turn_and_call_once() {
    let thread = ThreadId::new();
    let other = ThreadId::new();
    note_allowed_run(thread, "turn-d27", "call-d27");
    assert!(!take_allowed_run(thread, "turn-d27", "call-d27-other"));
    assert!(!take_allowed_run(thread, "turn-d27-other", "call-d27"));
    assert!(!take_allowed_run(other, "turn-d27", "call-d27"));
    assert!(take_allowed_run(thread, "turn-d27", "call-d27"));
    assert!(!take_allowed_run(thread, "turn-d27", "call-d27"));
    // Bounded: older runs are forgotten once enough newer ones are noted.
    note_allowed_run(thread, "turn-d27", "call-d27-old");
    for n in 0..ALLOWED_RUNS_KEPT {
        note_allowed_run(other, "turn-d27", &format!("call-d27-{n}"));
    }
    assert!(!take_allowed_run(thread, "turn-d27", "call-d27-old"));
}

/// D27 (a), D48 (c): a person warned at taint generation `g` may still
/// allow it only while Moderate is configured and live, the kill switch is
/// off and no more untrusted content has arrived (the generation is still
/// `g`): untrusted content arriving between the prompt and the run takes it
/// away.
#[test]
fn d48_allowed_only_while_nothing_changed_since_the_warning() {
    use crate::security::tainted_action::PolicyBinding;
    use codex_security_policy::ActorChain;
    use codex_security_policy::PolicyPrincipal;
    use codex_security_policy::PrincipalKind;
    let actor_chain = ActorChain::new(vec![
        PolicyPrincipal::new(PrincipalKind::Human, "human").expect("principal"),
    ])
    .expect("chain");
    let state = |generation: u64, configured: SecurityLevel, live: SecurityLevel, kill: bool| {
        PostTaintState {
            taint_generation: generation,
            policy: PolicyBinding::Bound {
                epoch: 0,
                revocation_generation: 0,
                kill_switch_active: kill,
                level: live,
                actor_chain: actor_chain.clone(),
            },
            level: configured,
        }
    };
    let moderate = SecurityLevel::Moderate;
    for warned_at in [0, 3] {
        assert!(still_offered(
            &state(warned_at, moderate, moderate, false),
            warned_at
        ));
        // More untrusted content since the warning.
        assert!(!still_offered(
            &state(warned_at + 1, moderate, moderate, false),
            warned_at
        ));
        // The kill switch turned on.
        assert!(!still_offered(
            &state(warned_at, moderate, moderate, true),
            warned_at
        ));
        // Moderate no longer live, or no longer configured.
        for (configured, live) in [
            (moderate, SecurityLevel::Aggressive),
            (moderate, SecurityLevel::Permissive),
            (SecurityLevel::Aggressive, SecurityLevel::Aggressive),
        ] {
            assert!(!still_offered(
                &state(warned_at, configured, live, false),
                warned_at
            ));
        }
    }
    // Not bound to a live policy.
    assert!(!still_offered(
        &PostTaintState {
            taint_generation: 0,
            policy: PolicyBinding::Unbound,
            level: moderate,
        },
        /*warned_at*/ 0
    ));
}
