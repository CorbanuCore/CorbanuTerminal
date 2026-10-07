use pretty_assertions::assert_eq;

use super::*;
use crate::legacy_core::protected_preflight::ReadinessFlags;
use crate::legacy_core::protected_preflight::file_sources;

fn request(home: &Path, target: ChosenLevel, raise_core: bool) -> TransitionRequest {
    TransitionRequest {
        codex_home: home.to_path_buf(),
        target,
        nested: NestedAgents::Refuse,
        configured: SecurityLevel::Permissive,
        reviewed: LevelBasis::read(home, SecurityLevel::Permissive),
        passed_preflight: raise_core.then(|| {
            Preflight::run(
                &file_sources(home, /*home*/ None, /*cwd*/ None),
                ReadinessFlags {
                    secretless_launch: true,
                    credential_broker: true,
                    output_gate: true,
                    untrusted_content: true,
                },
            )
        }),
        raise_core,
    }
}

fn state(home: &Path) -> (level::StoredLevel, StoredSecurityState, bool) {
    (
        level::load(home),
        StoredSecurityState::load(home),
        preflight::receipt_path(home).exists(),
    )
}

#[test]
fn security_confirm_aggressive_saves_the_level_and_raises_core() {
    let home = tempfile::tempdir().unwrap();
    let outcome = run(request(home.path(), ChosenLevel::Aggressive, true)).unwrap();
    assert_eq!(
        outcome.core.map(|report| report.next_start),
        Some(SecurityLevel::Aggressive)
    );
    assert_eq!(
        state(home.path()),
        (
            level::StoredLevel::Chosen(ChosenLevel::Aggressive),
            StoredSecurityState::Level(SecurityLevel::Aggressive),
            true
        )
    );
}

/// Without the PF-29 preflight Core's level is not raised; the level file
/// alone is saved, as before PF-24-S02.
#[test]
fn security_confirm_aggressive_without_preflight_leaves_core_alone() {
    let home = tempfile::tempdir().unwrap();
    let outcome = run(request(home.path(), ChosenLevel::Aggressive, false)).unwrap();
    assert_eq!(outcome.core, None);
    assert_eq!(
        state(home.path()),
        (
            level::StoredLevel::Chosen(ChosenLevel::Aggressive),
            StoredSecurityState::Absent,
            false
        )
    );
}

#[test]
fn security_confirm_downgrade_saves_both_records() {
    let home = tempfile::tempdir().unwrap();
    run(request(home.path(), ChosenLevel::Aggressive, true)).unwrap();
    let outcome = run(request(home.path(), ChosenLevel::Permissive, false)).unwrap();
    assert_eq!(
        outcome.core.map(|report| report.next_start),
        Some(SecurityLevel::Permissive)
    );
    assert_eq!(
        state(home.path()),
        (
            level::StoredLevel::Chosen(ChosenLevel::Permissive),
            StoredSecurityState::Level(SecurityLevel::Permissive),
            false
        )
    );
}

/// Core's record cannot be written: the level file and receipt are put back
/// exactly as they were, and the view can retry.
#[test]
fn security_confirm_write_failure_changes_nothing() {
    let home = tempfile::tempdir().unwrap();
    run(request(home.path(), ChosenLevel::Aggressive, true)).unwrap();
    let before = std::fs::read(level::state_path(home.path())).unwrap();
    // The state file's lock cannot be created when it is a folder.
    std::fs::create_dir(home.path().join("security_state.lock")).unwrap();
    let failure = run(request(home.path(), ChosenLevel::Permissive, false)).unwrap_err();
    assert!(failure.message.starts_with("Not saved:"), "{failure:?}");
    assert!(failure.message.ends_with("Nothing changed"), "{failure:?}");
    assert_eq!(std::fs::read(level::state_path(home.path())).unwrap(), before);
    assert_eq!(
        state(home.path()),
        (
            level::StoredLevel::Chosen(ChosenLevel::Aggressive),
            StoredSecurityState::Level(SecurityLevel::Aggressive),
            true
        )
    );
}

#[test]
fn security_confirm_state_changed_since_review_asks_to_review_again() {
    let home = tempfile::tempdir().unwrap();
    let stale = request(home.path(), ChosenLevel::Permissive, false);
    // Another session confirms Aggressive while the review is open.
    run(request(home.path(), ChosenLevel::Aggressive, true)).unwrap();
    let failure = run(TransitionRequest {
        reviewed: stale.reviewed,
        ..request(home.path(), ChosenLevel::Permissive, false)
    })
    .unwrap_err();
    assert!(failure.review_again, "{failure:?}");
    assert_eq!(
        state(home.path()),
        (
            level::StoredLevel::Chosen(ChosenLevel::Aggressive),
            StoredSecurityState::Level(SecurityLevel::Aggressive),
            true
        )
    );
}

#[test]
fn security_confirm_runs_off_the_calling_thread() {
    let home = tempfile::tempdir().unwrap();
    let mut pending = start(request(home.path(), ChosenLevel::Aggressive, true), false);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    let result = loop {
        if let Some(result) = pending.poll() {
            break result;
        }
        assert!(std::time::Instant::now() < deadline, "the commit did not finish");
        std::thread::sleep(std::time::Duration::from_millis(10));
    };
    assert_eq!(result.unwrap().level, ChosenLevel::Aggressive);
}
