use super::*;
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
    let explanation = explained(Some("Moderate"), Some(1), REFUSED).unwrap();
    assert!(
        explanation.starts_with("Security level Moderate blocks creating user namespaces"),
        "{explanation}"
    );
    assert_eq!(explained(Some("Aggressive"), Some(0), REFUSED), None);
    assert_eq!(
        explained(Some("Aggressive"), /*exit_code*/ None, REFUSED),
        None
    );
    assert_eq!(
        explained(
            Some("Aggressive"),
            Some(1),
            "cat: /home/someone/.ssh/id_ed25519: Operation not permitted"
        ),
        None
    );
    assert_eq!(explained(/*level*/ None, Some(1), REFUSED), None);
}
