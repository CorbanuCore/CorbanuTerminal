use pretty_assertions::assert_eq;

use super::*;

#[test]
fn absent_state_changes_no_launch_input_or_file() {
    let home = tempfile::tempdir().unwrap();
    let mut cli = vec![("model".to_string(), toml::Value::String("m".to_string()))];
    let mut plan = LaunchPlan::prepare(home.path(), &mut cli).unwrap();
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
    level::save(home.path(), ChosenLevel::Aggressive).unwrap();
    std::fs::remove_file(level::rules_path(home.path())).unwrap();
    let mut cli = Vec::new();
    let plan = LaunchPlan::prepare(home.path(), &mut cli).unwrap();
    assert_eq!(cli, aggressive::base_overrides(home.path()));
    assert_eq!(
        std::fs::read_to_string(level::rules_path(home.path())).unwrap(),
        level::rules_contents()
    );
    assert!(plan.check_target(/*uses_remote_app_server*/ false).is_ok());
    assert!(plan.check_target(/*uses_remote_app_server*/ true).is_err());
}

#[test]
fn stored_permissive_removes_a_leftover_rule_file() {
    let home = tempfile::tempdir().unwrap();
    level::save(home.path(), ChosenLevel::Aggressive).unwrap();
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
    level::save(home.path(), ChosenLevel::Aggressive).unwrap();
    let mut cli = Vec::new();
    let plan = LaunchPlan::prepare(home.path(), &mut cli).unwrap();
    // Built without the overrides, as a launch path that skipped them would.
    let mut config = crate::legacy_core::config::ConfigBuilder::default()
        .codex_home(home.path().to_path_buf())
        .loader_overrides(codex_config::LoaderOverrides::without_managed_config_for_tests())
        .build()
        .await
        .unwrap();
    let error = plan.finish(&mut config).unwrap_err();
    assert!(
        error.contains("could not be fully applied") && error.contains("Approvals"),
        "{error}"
    );
    assert_eq!(level::context(), None);
}
