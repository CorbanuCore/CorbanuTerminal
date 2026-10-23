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

/// The empty file the Linux sandbox briefly creates for a missing denied home
/// reads as unreadable and refuses the probes; it is still not an origin, nor
/// is a home below such a file.
#[cfg(unix)]
#[test]
fn a_file_in_place_of_a_home_is_not_an_origin() {
    let account = tempfile::tempdir().unwrap();
    let placeholder = account.path().join(".pfterminal");
    std::fs::write(&placeholder, "").unwrap();
    assert_eq!(
        level::load_state(&placeholder).0.enforced(),
        level::ChosenLevel::Aggressive
    );
    assert!(sandboxed_away_from(&placeholder));
    let below = placeholder.join("home");
    assert_eq!(nested_origins(vec![placeholder, below]), Vec::new());
}

/// #766: under Moderate an agent launch runs with Moderate enforced; what
/// cannot be held to it is refused, naming Moderate.
#[test]
fn moderate_origin_holds_agents_and_refuses_the_rest() {
    let origin = PathBuf::from("/moderate-home");
    assert_eq!(
        decide_with("exec", NestedKind::Agent, &[], Some(&origin)),
        NestedLaunch::EnforceModerate(origin.clone())
    );
    for kind in [
        NestedKind::Interactive,
        NestedKind::Host,
        NestedKind::Credentials,
    ] {
        match decide_with("", kind, &[], Some(&origin)) {
            NestedLaunch::Refuse(message) => assert!(
                message.contains("while security level Moderate is enforced (/moderate-home)")
                    && !message.contains("Aggressive"),
                "{kind:?}: {message}"
            ),
            other => panic!("{kind:?}: {other:?}"),
        }
    }
    let standalone = standalone_refusal_with("codex-exec", NestedKind::Agent, &[], Some(&origin))
        .expect("a standalone binary cannot hold a run to Moderate");
    assert!(
        standalone.contains(
            "Only `corbanu exec` and `corbanu review` can run there, with Moderate enforced"
        ),
        "{standalone}"
    );
}

/// An Aggressive origin wins over a Moderate one, in both settings.
#[test]
fn aggressive_origin_wins_over_moderate() {
    let aggressive = PathBuf::from("/aggressive-home");
    let moderate = PathBuf::from("/moderate-home");
    assert_eq!(
        decide_with(
            "exec",
            NestedKind::Agent,
            &[(aggressive.clone(), NestedAgents::Pass)],
            Some(&moderate)
        ),
        NestedLaunch::EnforceAggressive(aggressive.clone())
    );
    match decide_with(
        "exec",
        NestedKind::Agent,
        &[(aggressive, NestedAgents::Refuse)],
        Some(&moderate),
    ) {
        NestedLaunch::Refuse(message) => assert!(message.contains("set to refuse"), "{message}"),
        other => panic!("{other:?}"),
    }
}

/// Only a marker this process may not read counts: a missing one, a link or
/// a folder in its place, or a readable one does not.
#[cfg(unix)]
#[test]
fn only_an_unreadable_marker_marks_moderate() {
    use std::os::unix::fs::PermissionsExt;

    let home = tempfile::tempdir().unwrap();
    assert!(!moderate_marker_denied(home.path()));
    let marker = prepare_moderate_marker(home.path()).unwrap();
    assert_eq!(marker, home.path().join(MODERATE_MARKER));
    assert!(marker.is_file());
    assert!(!moderate_marker_denied(home.path()));
    // As bubblewrap masks a denied file: counted on Linux only. On macOS
    // only Seatbelt's EPERM counts, which file permissions cannot produce.
    std::fs::set_permissions(&marker, std::fs::Permissions::from_mode(0o000)).unwrap();
    let readable_anyway = std::fs::File::open(&marker).is_ok();
    assert_eq!(
        moderate_marker_denied(home.path()),
        cfg!(target_os = "linux") && !readable_anyway
    );
    // Nor in a home denied whole, whose runtime folder cannot be listed.
    let folder = marker.parent().unwrap();
    std::fs::set_permissions(folder, std::fs::Permissions::from_mode(0o000)).unwrap();
    let listable_anyway = std::fs::read_dir(folder).is_ok();
    assert_eq!(
        moderate_marker_denied(home.path()),
        cfg!(target_os = "linux") && listable_anyway && !readable_anyway
    );
    std::fs::set_permissions(folder, std::fs::Permissions::from_mode(0o755)).unwrap();
    std::fs::set_permissions(&marker, std::fs::Permissions::from_mode(0o644)).unwrap();
    // Made again only when missing; something else in its place is refused.
    assert_eq!(prepare_moderate_marker(home.path()).unwrap(), marker);
    std::fs::remove_file(&marker).unwrap();
    let elsewhere = home.path().join("elsewhere");
    std::fs::write(&elsewhere, "").unwrap();
    std::fs::set_permissions(&elsewhere, std::fs::Permissions::from_mode(0o000)).unwrap();
    std::os::unix::fs::symlink(&elsewhere, &marker).unwrap();
    assert!(!moderate_marker_denied(home.path()));
    assert!(prepare_moderate_marker(home.path()).is_err());
    std::fs::remove_file(&marker).unwrap();
    std::fs::create_dir(&marker).unwrap();
    assert!(!moderate_marker_denied(home.path()));
    assert!(prepare_moderate_marker(home.path()).is_err());
}

/// Registering keeps homes with a marker and drops those whose marker is
/// gone.
#[test]
fn moderate_registry_keeps_only_marked_homes() {
    let registry = tempfile::tempdir().unwrap();
    let kept = tempfile::tempdir().unwrap();
    let gone = tempfile::tempdir().unwrap();
    for home in [kept.path(), gone.path()] {
        prepare_moderate_marker(home).unwrap();
        register_moderate_home(registry.path(), home).unwrap();
    }
    std::fs::remove_file(gone.path().join(MODERATE_MARKER)).unwrap();
    register_moderate_home(registry.path(), kept.path()).unwrap();
    assert_eq!(
        registered_homes(registry.path()),
        vec![kept.path().to_path_buf()]
    );
}

/// #766 round 2: a home denied whole is explained by Moderate only when the
/// marked home is a default home, where no agent command can make a marker;
/// the marked home itself always is.
#[cfg(unix)]
#[test]
fn only_moderate_explains_an_unreadable_level() {
    use std::os::unix::fs::PermissionsExt;

    let root = tempfile::tempdir().unwrap();
    let marked = root.path().join("marked");
    let other = root.path().join("other");
    for home in [&marked, &other] {
        std::fs::create_dir_all(home.join("tmp")).unwrap();
    }
    assert!(explained_by_moderate(
        &marked, &marked, /*default_marked*/ false
    ));
    assert!(!explained_by_moderate(
        &other, &marked, /*default_marked*/ true
    ));
    // Denied whole, as the protected-file rules deny other default homes.
    std::fs::set_permissions(&other, std::fs::Permissions::from_mode(0o000)).unwrap();
    let denied = std::fs::read_dir(other.join("tmp")).is_err();
    assert_eq!(
        (
            explained_by_moderate(&other, &marked, /*default_marked*/ true),
            explained_by_moderate(&other, &marked, /*default_marked*/ false),
        ),
        (denied, false)
    );
    std::fs::set_permissions(&other, std::fs::Permissions::from_mode(0o755)).unwrap();
}

/// A marker its owner cannot read is given read permission again (Linux), but a
/// link, a hard link or a private file at its place is left alone.
#[cfg(target_os = "linux")]
#[test]
fn an_unreadable_marker_is_made_readable_again() {
    use std::os::unix::fs::PermissionsExt;

    let mode = |path: &std::path::Path| {
        std::fs::symlink_metadata(path)
            .unwrap()
            .permissions()
            .mode()
            & 0o777
    };
    let home = tempfile::tempdir().unwrap();
    keep_moderate_marker_readable(home.path()).unwrap();
    let marker = prepare_moderate_marker(home.path()).unwrap();
    // Whatever others may do, the owner cannot read these.
    for unreadable in [0o000, 0o044, 0o004, 0o040] {
        std::fs::set_permissions(&marker, std::fs::Permissions::from_mode(unreadable)).unwrap();
        keep_moderate_marker_readable(home.path()).unwrap();
        assert_eq!(mode(&marker), 0o644, "{unreadable:o}");
        assert!(!moderate_marker_denied(home.path()));
    }

    // A private file of the user's (0600) is not "unreadable".
    std::fs::set_permissions(&marker, std::fs::Permissions::from_mode(0o600)).unwrap();
    keep_moderate_marker_readable(home.path()).unwrap();
    assert_eq!(mode(&marker), 0o600);

    // A link or a hard link to another file: its target keeps its mode.
    let key = home.path().join("key");
    std::fs::write(&key, "").unwrap();
    std::fs::set_permissions(&key, std::fs::Permissions::from_mode(0o000)).unwrap();
    std::fs::remove_file(&marker).unwrap();
    std::os::unix::fs::symlink(&key, &marker).unwrap();
    keep_moderate_marker_readable(home.path()).unwrap();
    assert_eq!(mode(&key), 0o000);
    std::fs::remove_file(&marker).unwrap();
    std::fs::hard_link(&key, &marker).unwrap();
    keep_moderate_marker_readable(home.path()).unwrap();
    assert_eq!(mode(&key), 0o000);
}
