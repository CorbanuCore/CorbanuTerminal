use std::path::PathBuf;

use pretty_assertions::assert_eq;

use super::*;

const KINDS: [NestedKind; 4] = [
    NestedKind::Agent,
    NestedKind::Interactive,
    NestedKind::Host,
    NestedKind::Credentials,
];

#[test]
fn standalone_binaries_are_not_refused_without_an_origin() {
    for kind in KINDS {
        assert_eq!(standalone_refusal_for("codex-exec", kind, &[]), None);
    }
}

/// A standalone binary cannot hold a run to Aggressive, so every kind is
/// refused under both settings, naming the binary and the origin.
#[test]
fn standalone_binaries_refuse_every_kind_in_both_modes() {
    let origin = PathBuf::from("/origin-home");
    for nested in [NestedAgents::Refuse, NestedAgents::Pass] {
        for kind in KINDS {
            let message = standalone_refusal_for("codex-exec", kind, &[(origin.clone(), nested)])
                .unwrap_or_else(|| panic!("{kind:?} {nested:?} was not refused"));
            assert!(
                message.starts_with(
                    "`codex-exec` was started by an agent command while security level Aggressive is enforced"
                ) && message.contains("/origin-home"),
                "{kind:?} {nested:?}: {message}"
            );
        }
    }
    let pass = standalone_refusal_for(
        "codex-exec",
        NestedKind::Agent,
        &[(origin, NestedAgents::Pass)],
    )
    .unwrap();
    assert!(pass.contains("Only `corbanu exec` and `corbanu review` can run there"));
}

#[test]
fn corbanu_decisions_are_unchanged_by_the_move() {
    let origin = PathBuf::from("/h");
    assert_eq!(
        decide(
            "exec",
            NestedKind::Agent,
            &[(origin.clone(), NestedAgents::Pass)]
        ),
        NestedLaunch::EnforceAggressive(origin.clone())
    );
    match decide("", NestedKind::Interactive, &[(origin, NestedAgents::Pass)]) {
        NestedLaunch::Refuse(message) => assert!(
            message.starts_with("`corbanu` was started by an agent command"),
            "{message}"
        ),
        other => panic!("{other:?}"),
    }
}
