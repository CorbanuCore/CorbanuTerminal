use pretty_assertions::assert_eq;
use tempfile::TempDir;

use super::*;

fn confirm(home: &Path, level: &str) {
    std::fs::write(
        confirmed_state_path(home),
        format!(
            r#"{{"version":1,"level":"{level}","revocations":{{"generation":0,"events":[]}}}}"#
        ),
    )
    .unwrap();
}

fn invalid(stored: &StoredLevel) -> bool {
    matches!(stored, StoredLevel::Invalid(_))
}

#[test]
fn security_confirm_level_file_weaker_than_the_confirmed_record_is_invalid() {
    let home = TempDir::new().unwrap();
    save(home.path(), ChosenLevel::Aggressive, NestedAgents::Pass).unwrap();
    confirm(home.path(), "aggressive");
    assert_eq!(
        load_state(home.path()),
        (
            StoredLevel::Chosen(ChosenLevel::Aggressive),
            NestedAgents::Pass
        )
    );

    // Edited to Permissive outside /security.
    std::fs::write(
        state_path(home.path()),
        "version = 1\nlevel = \"permissive\"\n",
    )
    .unwrap();
    let (stored, nested) = load_state(home.path());
    assert!(invalid(&stored), "{stored:?}");
    assert_eq!(
        (stored.enforced(), nested),
        (ChosenLevel::Aggressive, NestedAgents::Refuse)
    );

    // Deleted together with the rule file.
    std::fs::remove_file(state_path(home.path())).unwrap();
    std::fs::remove_file(rules_path(home.path())).unwrap();
    assert!(invalid(&load(home.path())));
}

#[test]
fn security_confirm_record_no_stricter_than_the_file_changes_nothing() {
    let home = TempDir::new().unwrap();
    confirm(home.path(), "permissive");
    assert_eq!(load(home.path()), StoredLevel::Absent);
    save(home.path(), ChosenLevel::Permissive, NestedAgents::Refuse).unwrap();
    assert_eq!(
        load(home.path()),
        StoredLevel::Chosen(ChosenLevel::Permissive)
    );
    // A file stricter than the record (Aggressive saved without the Core
    // record, or before it is committed) stays as saved.
    save(home.path(), ChosenLevel::Aggressive, NestedAgents::Refuse).unwrap();
    assert_eq!(
        load(home.path()),
        StoredLevel::Chosen(ChosenLevel::Aggressive)
    );
}

#[test]
fn security_confirm_unreadable_record_enforces_aggressive() {
    let home = TempDir::new().unwrap();
    std::fs::write(confirmed_state_path(home.path()), "{").unwrap();
    assert!(invalid(&load(home.path())));
    std::fs::write(
        confirmed_state_path(home.path()),
        r#"{"version":1,"level":"relaxed"}"#,
    )
    .unwrap();
    assert!(invalid(&load(home.path())));
}

/// A downgrade saves the level file before Core's record moves: the save
/// verifies the file itself.
#[test]
fn security_confirm_save_verifies_the_file_while_the_record_is_stricter() {
    let home = TempDir::new().unwrap();
    confirm(home.path(), "aggressive");
    save(home.path(), ChosenLevel::Permissive, NestedAgents::Refuse).unwrap();
    assert!(invalid(&load(home.path())));
    confirm(home.path(), "permissive");
    assert_eq!(
        load(home.path()),
        StoredLevel::Chosen(ChosenLevel::Permissive)
    );
}

#[test]
fn security_confirm_aggressive_lock_is_seen_by_another_launch() {
    let home = TempDir::new().unwrap();
    assert!(!aggressive_running(home.path()));
    let held = hold_aggressive_lock(home.path()).unwrap();
    assert!(aggressive_running(home.path()));
    // A second Aggressive process shares it.
    let second = hold_aggressive_lock(home.path()).unwrap();
    drop(held);
    assert!(aggressive_running(home.path()));
    drop(second);
    assert!(!aggressive_running(home.path()));
}

/// #750: Moderate is a level a person can save. It writes no rule file
/// (and removes Aggressive's), keeps no nested-launch setting, and a file
/// weaker than an Aggressive record still reads as changed outside
/// `/security`.
#[test]
fn issue_750_moderate_is_saved_and_read_back() {
    let home = TempDir::new().unwrap();
    save(home.path(), ChosenLevel::Aggressive, NestedAgents::Pass).unwrap();
    assert!(rules_path(home.path()).exists());
    save(home.path(), ChosenLevel::Moderate, NestedAgents::Pass).unwrap();
    assert_eq!(
        load_state(home.path()),
        (
            StoredLevel::Chosen(ChosenLevel::Moderate),
            NestedAgents::Refuse
        )
    );
    assert_eq!(
        std::fs::read_to_string(state_path(home.path())).unwrap(),
        "version = 1\nlevel = \"moderate\"\n"
    );
    assert_eq!(load(home.path()).enforced(), ChosenLevel::Moderate);
    sync_rules(home.path(), ChosenLevel::Moderate).unwrap();
    assert!(!rules_path(home.path()).exists());
    confirm(home.path(), "moderate");
    assert_eq!(
        load(home.path()),
        StoredLevel::Chosen(ChosenLevel::Moderate)
    );
    confirm(home.path(), "aggressive");
    let stored = load(home.path());
    assert!(invalid(&stored), "{stored:?}");
    assert_eq!(stored.enforced(), ChosenLevel::Aggressive);
    assert!(ChosenLevel::Permissive < ChosenLevel::Moderate);
    assert!(ChosenLevel::Moderate < ChosenLevel::Aggressive);
}
