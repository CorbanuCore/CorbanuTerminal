use pretty_assertions::assert_eq;

use super::*;

fn write_state(home: &Path, contents: &str) {
    std::fs::write(state_path(home), contents).unwrap();
}

#[test]
fn absent_state_is_permissive_and_touches_nothing() {
    let home = tempfile::tempdir().unwrap();
    assert_eq!(load(home.path()), StoredLevel::Absent);
    assert_eq!(load(home.path()).enforced(), ChosenLevel::Permissive);
    assert_eq!(std::fs::read_dir(home.path()).unwrap().count(), 0);
}

#[test]
fn unknown_or_corrupt_state_fails_visibly_and_enforces_aggressive() {
    let home = tempfile::tempdir().unwrap();
    for contents in [
        "version = 1\nlevel = \"moderate\"\n",
        "version = 1\nlevel = \"Permissive\"\n",
        "version = 2\nlevel = \"permissive\"\n",
        "version = 1\nlevel = \"permissive\"\nextra = true\n",
        "level = \"permissive\"\n",
        "not toml",
    ] {
        write_state(home.path(), contents);
        let stored = load(home.path());
        assert!(
            matches!(&stored, StoredLevel::Invalid(reason) if reason.contains(STATE_FILE)),
            "{contents}: {stored:?}"
        );
        assert_eq!(stored.enforced(), ChosenLevel::Aggressive, "{contents}");
    }
}

#[test]
fn aggressive_round_trip_restores_permissive_files_exactly() {
    let home = tempfile::tempdir().unwrap();
    let config = "approval_policy = \"never\"\nsandbox_mode = \"danger-full-access\"\n";
    std::fs::write(home.path().join("config.toml"), config).unwrap();
    std::fs::create_dir_all(home.path().join(RULES_DIR)).unwrap();
    let user_rules = "prefix_rule(pattern=[\"git\", \"push\"], decision=\"allow\")\n";
    std::fs::write(
        home.path().join(RULES_DIR).join("default.rules"),
        user_rules,
    )
    .unwrap();

    save(home.path(), ChosenLevel::Aggressive).unwrap();
    assert_eq!(
        load(home.path()),
        StoredLevel::Chosen(ChosenLevel::Aggressive)
    );
    assert_eq!(
        std::fs::read_to_string(rules_path(home.path())).unwrap(),
        rules_contents()
    );

    save(home.path(), ChosenLevel::Permissive).unwrap();
    assert_eq!(
        load(home.path()),
        StoredLevel::Chosen(ChosenLevel::Permissive)
    );
    let mut rules = std::fs::read_dir(home.path().join(RULES_DIR))
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect::<Vec<_>>();
    rules.sort();
    assert_eq!(
        (
            std::fs::read_to_string(home.path().join("config.toml")).unwrap(),
            rules,
            std::fs::read_to_string(home.path().join(RULES_DIR).join("default.rules")).unwrap(),
        ),
        (
            config.to_string(),
            vec!["default.rules".to_string()],
            user_rules.to_string()
        )
    );
}

#[test]
fn rules_forbid_every_vault_entry_point() {
    let rules = rules_contents();
    for program in VAULT_PROGRAMS {
        assert!(
            rules.contains(&format!(
                "pattern = [\"{program}\", \"vault\"], decision = \"forbidden\""
            )),
            "{program}"
        );
    }
}

#[test]
fn status_line_reports_active_and_pending_levels() {
    assert_eq!(
        [
            status_line(ChosenLevel::Permissive, &StoredLevel::Absent),
            status_line(
                ChosenLevel::Aggressive,
                &StoredLevel::Chosen(ChosenLevel::Aggressive)
            ),
            status_line(
                ChosenLevel::Aggressive,
                &StoredLevel::Chosen(ChosenLevel::Permissive)
            ),
            status_line(ChosenLevel::Aggressive, &StoredLevel::Invalid("x".into())),
        ],
        [
            "Permissive active (/security)".to_string(),
            "Aggressive active (/security)".to_string(),
            "Aggressive active; Permissive saved for next start (/security)".to_string(),
            "Aggressive active; stored level unreadable, Aggressive enforced (/security)"
                .to_string(),
        ]
    );
}
