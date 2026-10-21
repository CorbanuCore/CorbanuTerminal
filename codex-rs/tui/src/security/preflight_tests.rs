use pretty_assertions::assert_eq;

use super::*;
use crate::security::aggressive;
use crate::security::level::LevelContext;

fn context(boundary: Option<Boundary>) -> LevelContext {
    LevelContext {
        codex_home: PathBuf::from("/corbanu"),
        picker_enabled: true,
        active: ChosenLevel::Aggressive,
        preflight_enabled: true,
        boundary,
    }
}

fn clean_preflight(home: &Path) -> Preflight {
    Preflight::run(
        &file_sources(home, /*home*/ None, /*cwd*/ None),
        ReadinessFlags {
            secretless_launch: true,
            credential_broker: true,
            output_gate: true,
            untrusted_content: true,
        },
    )
}

#[test]
fn pf_29_s01_receipt_round_trip_and_corrupt_receipts_fail_closed() {
    let home = tempfile::tempdir().unwrap();
    assert_eq!(load_receipt(home.path()), Ok(None));
    save_receipt(home.path(), &clean_preflight(home.path())).unwrap();
    let receipt = load_receipt(home.path()).unwrap().unwrap();
    assert_eq!(receipt.activated_at, None);

    std::fs::write(receipt_path(home.path()), "version = 1\nsaved_at = \"x\"").unwrap();
    assert!(load_receipt(home.path()).is_err());
    // Version 1 stored seconds; it must not be read as milliseconds.
    for version in [1, 9] {
        std::fs::write(
            receipt_path(home.path()),
            format!("version = {version}\nsaved_at = 1\nactivated_at = 1791323285\nfindings = []"),
        )
        .unwrap();
        assert!(load_receipt(home.path()).is_err(), "version {version}");
    }

    remove_receipt(home.path()).unwrap();
    remove_receipt(home.path()).unwrap();
    assert_eq!(load_receipt(home.path()), Ok(None));
}

#[test]
fn pf_29_s01_isolation_only_after_a_preflight_and_lands_in_the_profile() {
    let root = tempfile::tempdir().unwrap();
    let corbanu = root.path().join("corbanu");
    let home = root.path().join("home");
    std::fs::create_dir_all(home.join(".ssh")).unwrap();
    std::fs::create_dir_all(&corbanu).unwrap();
    std::fs::write(home.join(".ssh/id_rsa"), "fake").unwrap();

    assert_eq!(
        isolation_paths(&corbanu, Some(&home), /*cwd*/ None),
        Vec::<PathBuf>::new()
    );
    // Even an unreadable receipt asks for isolation.
    std::fs::write(receipt_path(&corbanu), "garbage").unwrap();
    std::fs::write(corbanu.join("state_5.sqlite"), "db").unwrap();
    let paths = isolation_paths(&corbanu, Some(&home), /*cwd*/ None);
    for path in [
        home.join(".ssh"),
        home.join(".ssh/id_rsa"),
        // Corbanu's stores even before startup creates them, and every
        // database (with its -wal/-shm files) through one glob.
        corbanu.join("sessions"),
        corbanu.join("history.jsonl"),
        corbanu.join("*.sqlite*"),
    ] {
        assert!(
            paths.contains(&path),
            "{} missing from {paths:?}",
            path.display()
        );
    }
    assert!(!paths.contains(&corbanu.join("state_5.sqlite")));
    assert!(
        corbanu.join("sessions").is_dir(),
        "store folders exist from launch"
    );

    // #651: a home with glob syntax: its database glob is escaped (it
    // matches the databases there only), its stores stay as written (a deny
    // key with glob syntax is a pattern, which launch then refuses, failing
    // closed).
    let odd = root.path().join("corbanu[1]");
    std::fs::create_dir_all(&odd).unwrap();
    std::fs::write(receipt_path(&odd), "garbage").unwrap();
    std::fs::write(odd.join("state_5.sqlite"), "db").unwrap();
    let odd_paths = isolation_paths(&odd, Some(&home), /*cwd*/ None);
    assert!(
        odd_paths.contains(&root.path().join("corbanu[[]1]").join("*.sqlite*")),
        "{odd_paths:?}"
    );
    assert!(odd_paths.contains(&odd.join("sessions")));
    assert!(!odd_paths.contains(&odd.join("state_5.sqlite")));
    assert!(!odd_paths.contains(&odd.join("*.sqlite*")));

    // #651: a database folder set apart in the user config, relative to the
    // config file's folder as Core resolves it: its databases and what
    // Corbanu keeps there (when present).
    let databases = root.path().join("db");
    std::fs::create_dir_all(databases.join("accounting-pending")).unwrap();
    std::fs::write(corbanu.join("config.toml"), "sqlite_home = \"../db\"\n").unwrap();
    let with_folder = isolation_paths(&corbanu, Some(&home), /*cwd*/ None);
    assert!(
        with_folder.contains(&databases.join("*.sqlite*")),
        "{with_folder:?}"
    );
    assert!(with_folder.contains(&databases.join("accounting-pending")));
    assert!(!with_folder.contains(&databases.join("gpu-controller.lock")));
    std::fs::remove_file(corbanu.join("config.toml")).unwrap();
    // On the command line, against the working folder.
    let given = override_database_isolation(
        &corbanu,
        &[(
            "sqlite_home".to_string(),
            toml::Value::String("db".to_string()),
        )],
        Some(root.path()),
    );
    assert!(given.contains(&databases.join("*.sqlite*")), "{given:?}");
    // `~` is expanded as Core expands it.
    if let Some(user_home) = dirs::home_dir() {
        assert_eq!(resolve_folder("~/db", root.path()), user_home.join("db"));
    }
    assert_eq!(
        override_database_isolation(
            &corbanu,
            &[(
                "sqlite_home".to_string(),
                toml::Value::String(".".to_string())
            )],
            Some(&corbanu),
        ),
        Vec::<PathBuf>::new()
    );

    let mut overrides = aggressive::base_overrides(&corbanu, &corbanu);
    aggressive::deny_reads(&mut overrides, &paths);
    let profile = overrides
        .iter()
        .find(|(key, _)| key == &format!("permissions.{}", aggressive::PROFILE_ID))
        .map(|(_, value)| value.clone())
        .unwrap();
    // The keys are the paths as the inventory spells them, with this
    // platform's separators.
    for path in [home.join(".ssh").join("id_rsa"), corbanu.join("*.sqlite*")] {
        assert_eq!(
            profile["filesystem"][path.to_string_lossy().as_ref()],
            toml::Value::String("deny".to_string())
        );
    }
}

#[test]
fn pf_29_s01_resume_refused_for_conversations_before_activation() {
    let now = now_ms();
    let fresh = ThreadId::new();
    let created = thread_created_at(&fresh).unwrap();
    assert!((created - now).abs() <= 2_000, "{created} vs {now}");

    // Millisecond precision: a thread one millisecond older is refused.
    let at_creation = context(Some(Boundary::Clean {
        activated_at: created + 1,
    }));
    assert!(refusal_for(&at_creation, &fresh).is_some());
    let clean_before = context(Some(Boundary::Clean {
        activated_at: created,
    }));
    assert_eq!(refusal_for(&clean_before, &fresh), None);
    let clean_after = context(Some(Boundary::Clean {
        activated_at: now + 60_000,
    }));
    assert!(
        refusal_for(&clean_after, &fresh)
            .unwrap()
            .contains("recorded before protected mode was activated")
    );
    for boundary in [
        Boundary::NotClean {
            blockers: vec!["x".to_string()],
        },
        Boundary::Unverified("corrupt".to_string()),
    ] {
        assert!(refusal_for(&context(Some(boundary)), &fresh).is_some());
    }
    // Flag off (no boundary) or Permissive: today's behaviour.
    assert_eq!(refusal_for(&context(/*boundary*/ None), &fresh), None);
    let mut permissive = context(Some(Boundary::Unverified("x".to_string())));
    permissive.active = ChosenLevel::Permissive;
    assert_eq!(refusal_for(&permissive, &fresh), None);
}

/// #444 (AGG-32): a config layer present at start was reported as "not
/// found when Corbanu Terminal started".
#[test]
fn sec_444_undenied_config_layer_is_not_called_missing_at_start() {
    let line = "Isolation: agent commands can still read /etc/codex/config.toml";
    let layer = super::not_denied_at_launch(line, /*config_layer*/ true);
    assert!(!layer.contains("not found"), "{layer}");
    assert!(!layer.contains("not there"), "{layer}");
    assert!(layer.contains("config files are read only after the sandbox is set up"));
    let file = super::not_denied_at_launch(line, /*config_layer*/ false);
    assert!(file.contains("it was not there when Corbanu Terminal started"));
}
