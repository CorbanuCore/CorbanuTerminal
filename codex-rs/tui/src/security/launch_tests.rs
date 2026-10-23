use pretty_assertions::assert_eq;

use super::*;
use crate::security::level::NestedAgents;

/// #391: a stored Aggressive level turns `broker_model_auth` on, which makes
/// the whole test process brokered; these tests are about other rows.
fn broker_off() -> (String, toml::Value) {
    (
        "features.broker_model_auth".to_string(),
        toml::Value::Boolean(false),
    )
}

#[test]
fn absent_state_changes_no_launch_input_or_file() {
    let home = tempfile::tempdir().unwrap();
    let mut cli = vec![("model".to_string(), toml::Value::String("m".to_string()))];
    let mut plan = LaunchPlan::prepare(home.path(), &mut cli).unwrap();
    assert_eq!(plan.origin_registry_update(), None);
    plan.check_target(/*uses_remote_app_server*/ true).unwrap();
    plan.extend_env_overrides(&ShellEnvironmentPolicyToml::default(), &mut cli);
    let mut overrides = ConfigOverrides {
        approval_policy: Some(codex_protocol::protocol::AskForApproval::Never),
        ..Default::default()
    };
    plan.apply_launch_overrides(&mut overrides);
    assert_eq!(
        (
            cli,
            overrides.approval_policy,
            std::fs::read_dir(home.path()).unwrap().count()
        ),
        (
            vec![("model".to_string(), toml::Value::String("m".to_string()))],
            Some(codex_protocol::protocol::AskForApproval::Never),
            0
        )
    );
}

#[test]
fn stored_aggressive_adds_overrides_rules_and_refuses_remote_servers() {
    let home = tempfile::tempdir().unwrap();
    level::save(home.path(), ChosenLevel::Aggressive, NestedAgents::Refuse).unwrap();
    std::fs::remove_file(level::rules_path(home.path())).unwrap();
    let mut cli = Vec::new();
    let plan = LaunchPlan::prepare(home.path(), &mut cli).unwrap();
    assert_eq!(cli, aggressive::base_overrides(home.path(), home.path()));
    assert_eq!(
        std::fs::read_to_string(level::rules_path(home.path())).unwrap(),
        level::rules_contents()
    );
    assert!(home.path().join("secrets").is_dir());
    assert_eq!(plan.origin_registry_update(), Some(true));
    assert!(plan.check_target(/*uses_remote_app_server*/ false).is_ok());
    assert!(plan.check_target(/*uses_remote_app_server*/ true).is_err());
}

#[test]
fn stored_permissive_removes_a_leftover_rule_file() {
    let home = tempfile::tempdir().unwrap();
    level::save(home.path(), ChosenLevel::Aggressive, NestedAgents::Refuse).unwrap();
    std::fs::write(
        level::state_path(home.path()),
        "version = 1\nlevel = \"permissive\"\n",
    )
    .unwrap();
    let mut cli = Vec::new();
    LaunchPlan::prepare(home.path(), &mut cli).unwrap();
    assert_eq!(
        (cli, level::rules_path(home.path()).exists()),
        (Vec::new(), false)
    );
}

#[tokio::test]
async fn finish_refuses_a_config_that_misses_a_row() {
    let home = tempfile::tempdir().unwrap();
    level::save(home.path(), ChosenLevel::Aggressive, NestedAgents::Refuse).unwrap();
    let mut cli = Vec::new();
    let plan = LaunchPlan::prepare(home.path(), &mut cli).unwrap();
    // Built without the overrides, as a launch path that skipped them would.
    let mut config = crate::legacy_core::config::ConfigBuilder::default()
        .codex_home(home.path().to_path_buf())
        .cli_overrides(vec![broker_off()])
        .loader_overrides(codex_config::LoaderOverrides::without_managed_config_for_tests())
        .build()
        .await
        .unwrap();
    let error = plan.finish(&mut config).await.unwrap_err();
    assert!(
        error.contains("could not be fully applied") && error.contains("Approvals"),
        "{error}"
    );
    assert_eq!(level::context(), None);
}

/// A `.rules` file that fails to parse would make the session drop every rule,
/// including the vault rule, so launch refuses Aggressive.
#[tokio::test]
async fn finish_refuses_a_broken_rules_file() {
    let home = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();
    level::save(home.path(), ChosenLevel::Aggressive, NestedAgents::Refuse).unwrap();
    let mut cli = Vec::new();
    let mut plan = LaunchPlan::prepare(home.path(), &mut cli).unwrap();
    plan.extend_env_overrides(&ShellEnvironmentPolicyToml::default(), &mut cli);
    cli.push(broker_off());
    let mut overrides = ConfigOverrides {
        cwd: Some(cwd.path().to_path_buf()),
        ..Default::default()
    };
    plan.apply_launch_overrides(&mut overrides);
    std::fs::write(
        level::rules_path(home.path()).with_file_name("mine.rules"),
        "prefix_rule(pattern = [\"git\"], decision = \n",
    )
    .unwrap();
    let mut config = crate::legacy_core::config::ConfigBuilder::default()
        .codex_home(home.path().to_path_buf())
        .cli_overrides(cli)
        .harness_overrides(overrides)
        .loader_overrides(codex_config::LoaderOverrides::without_managed_config_for_tests())
        .build()
        .await
        .unwrap();
    let error = plan.finish(&mut config).await.unwrap_err();
    let lines = error.lines().collect::<Vec<_>>();
    assert_eq!(
        lines.first().copied(),
        Some("Security level Aggressive could not be fully applied:"),
        "{error}"
    );
    assert_eq!(
        lines
            .iter()
            .filter(|line| line
                .trim_start()
                .starts_with("Vault: an exec-policy rules file does not parse"))
            .count(),
        1,
        "{error}"
    );
    assert_eq!(
        lines.len(),
        3,
        "only the rules failure and the fix hint: {error}"
    );
    assert_eq!(level::context(), None);
}

/// PF-24-S02: a Permissive launch of a home another process still enforces
/// Aggressive on leaves that process's rule file and registry entry alone.
#[test]
fn security_confirm_second_permissive_launch_keeps_the_running_aggressive_rules() {
    let home = tempfile::tempdir().unwrap();
    level::save(home.path(), ChosenLevel::Aggressive, NestedAgents::Refuse).unwrap();
    level::save(home.path(), ChosenLevel::Permissive, NestedAgents::Refuse).unwrap();
    let running = level::hold_aggressive_lock(home.path()).unwrap();
    let plan = LaunchPlan::prepare(home.path(), &mut Vec::new()).unwrap();
    assert_eq!(
        (
            level::rules_path(home.path()).exists(),
            plan.origin_registry_update()
        ),
        (true, None)
    );

    drop(running);
    let plan = LaunchPlan::prepare(home.path(), &mut Vec::new()).unwrap();
    assert_eq!(
        (
            level::rules_path(home.path()).exists(),
            plan.origin_registry_update()
        ),
        (false, Some(false))
    );
}

/// #428 (Travis 2026-10-11, option 1): in every state the TUI shows as
/// Aggressive while Core's level stays Permissive, the session config is the
/// one Core gates a model-chosen spawn `account` switch on (D3).
#[tokio::test]
async fn shown_aggressive_states_gate_spawn_account_switches_in_core() {
    for state in [
        "level file missing, Aggressive rules present",
        "Aggressive saved without a preflight, preflight feature on (boundary unverified)",
        "Permissive saved for the next start",
    ] {
        let home = tempfile::tempdir().unwrap();
        let cwd = tempfile::tempdir().unwrap();
        level::save(home.path(), ChosenLevel::Aggressive, NestedAgents::Refuse).unwrap();
        if state == "level file missing, Aggressive rules present" {
            std::fs::remove_file(level::state_path(home.path())).unwrap();
        }
        let mut cli = Vec::new();
        let mut plan = LaunchPlan::prepare(home.path(), &mut cli).unwrap();
        assert!(plan.aggressive(), "{state}");
        if state == "Permissive saved for the next start" {
            level::save(home.path(), ChosenLevel::Permissive, NestedAgents::Refuse).unwrap();
        }
        plan.extend_env_overrides(&ShellEnvironmentPolicyToml::default(), &mut cli);
        cli.push(broker_off());
        if state.ends_with("(boundary unverified)") {
            cli.push((
                "features.protected_mode_preflight".to_string(),
                toml::Value::Boolean(true),
            ));
        }
        let mut overrides = ConfigOverrides {
            cwd: Some(cwd.path().to_path_buf()),
            ..Default::default()
        };
        plan.apply_launch_overrides(&mut overrides);
        let config = crate::legacy_core::config::ConfigBuilder::default()
            .codex_home(home.path().to_path_buf())
            .cli_overrides(cli)
            .harness_overrides(overrides)
            .loader_overrides(codex_config::LoaderOverrides::without_managed_config_for_tests())
            .build()
            .await
            .unwrap();
        assert_eq!(
            (
                config.security_level,
                config.permissions.launch_enforces_aggressive()
            ),
            (
                crate::legacy_core::security_level_change::SecurityLevel::Permissive,
                true
            ),
            "{state}"
        );
    }
}

/// #651: every database folder config may name is denied at launch, and
/// verification counts each one's database glob as denied, also one that is
/// not the folder Core loads (the user config's, overridden with `-c`).
#[tokio::test]
async fn issue_651_every_candidate_database_folder_counts_as_denied() {
    let home = tempfile::tempdir().unwrap();
    let root = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();
    level::save(home.path(), ChosenLevel::Aggressive, NestedAgents::Refuse).unwrap();
    std::fs::write(home.path().join(preflight::RECEIPT_FILE), "garbage").unwrap();
    let configured = root.path().join("configured");
    let given = root.path().join("given");
    std::fs::create_dir_all(&configured).unwrap();
    std::fs::create_dir_all(&given).unwrap();
    std::fs::write(
        home.path().join("config.toml"),
        format!("sqlite_home = {:?}\n", configured.display().to_string()),
    )
    .unwrap();
    let mut cli = vec![(
        "sqlite_home".to_string(),
        toml::Value::String(given.display().to_string()),
    )];
    let mut plan = LaunchPlan::prepare(home.path(), &mut cli).unwrap();
    for folder in [&configured, &given] {
        assert!(
            plan.isolated.contains(&folder.join("*.sqlite*")),
            "{:?}",
            plan.isolated
        );
    }
    plan.extend_env_overrides(&ShellEnvironmentPolicyToml::default(), &mut cli);
    cli.push(broker_off());
    let mut overrides = ConfigOverrides {
        cwd: Some(cwd.path().to_path_buf()),
        ..Default::default()
    };
    plan.apply_launch_overrides(&mut overrides);
    let config = crate::legacy_core::config::ConfigBuilder::default()
        .codex_home(home.path().to_path_buf())
        .cli_overrides(cli)
        .harness_overrides(overrides)
        .loader_overrides(codex_config::LoaderOverrides::without_managed_config_for_tests())
        .build()
        .await
        .unwrap();
    assert_eq!(config.sqlite.home(), given.as_path());
    let databases = preflight::verify_isolation(&config, &plan.isolated)
        .into_iter()
        .filter(|line| line.contains(".sqlite"))
        .collect::<Vec<_>>();
    assert_eq!(databases, Vec::<String>::new());
    assert_eq!(
        preflight::verify_database_folder(&config),
        Vec::<String>::new()
    );
}

/// #750/#749: Moderate is Core's level. It is active only where the loaded
/// config enforces it (saved with Core's record, or `[security] level`), adds
/// no launch overrides and keeps no rule file; a saved Moderate without
/// Core's record is reported and not shown as active. A remote app server is
/// refused, as for Aggressive.
#[tokio::test]
async fn issue_750_moderate_is_active_only_where_core_enforces_it() {
    async fn active(
        home: &std::path::Path,
        cli: Vec<(String, toml::Value)>,
    ) -> (ChosenLevel, Vec<String>) {
        let mut prepared = Vec::new();
        let plan = LaunchPlan::prepare(home, &mut prepared).unwrap();
        assert_eq!(prepared, Vec::new());
        let mut cli = cli;
        cli.push(broker_off());
        let mut config = crate::legacy_core::config::ConfigBuilder::default()
            .codex_home(home.to_path_buf())
            .cli_overrides(cli)
            .loader_overrides(codex_config::LoaderOverrides::without_managed_config_for_tests())
            .build()
            .await
            .unwrap();
        let before = config.startup_warnings.len();
        let level = plan.active_level(&mut config);
        (level, config.startup_warnings[before..].to_vec())
    }
    let home = tempfile::tempdir().unwrap();
    level::save(home.path(), ChosenLevel::Aggressive, NestedAgents::Refuse).unwrap();
    level::save(home.path(), ChosenLevel::Moderate, NestedAgents::Refuse).unwrap();
    let (level, warnings) = active(home.path(), Vec::new()).await;
    assert_eq!(level, ChosenLevel::Permissive);
    assert!(
        warnings
            .iter()
            .any(|warning| warning.contains("Core's level was not confirmed")),
        "{warnings:?}"
    );
    // Aggressive's rule file is removed at the Moderate launch.
    assert!(!level::rules_path(home.path()).exists());
    let plan = LaunchPlan::prepare(home.path(), &mut Vec::new()).unwrap();
    assert_eq!(plan.origin_registry_update(), Some(false));
    assert!(plan.check_target(/*uses_remote_app_server*/ true).is_err());

    std::fs::write(
        home.path().join("security_state.json"),
        r#"{"version":1,"level":"moderate","revocations":{"schema_version":1,"generation":0,"kill_switch_active":false}}"#,
    )
    .unwrap();
    let (level, warnings) = active(home.path(), Vec::new()).await;
    assert_eq!(level, ChosenLevel::Moderate);
    assert!(
        warnings
            .iter()
            .any(|warning| warning.contains("broker_model_auth is off")),
        "{warnings:?}"
    );

    // `[security] level = "moderate"` in config, nothing saved (#749).
    let config_only = tempfile::tempdir().unwrap();
    let (level, _) = active(
        config_only.path(),
        vec![(
            "security".to_string(),
            toml::toml! { version = 1
            level = "moderate" }
            .into(),
        )],
    )
    .await;
    assert_eq!(level, ChosenLevel::Moderate);
    let (level, _) = active(config_only.path(), Vec::new()).await;
    assert_eq!(level, ChosenLevel::Permissive);
}
