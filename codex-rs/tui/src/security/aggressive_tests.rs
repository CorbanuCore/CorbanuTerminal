use std::collections::BTreeMap;
use std::collections::HashMap;

use codex_protocol::config_types::ShellEnvironmentPolicyFilter;
use pretty_assertions::assert_eq;

use codex_protocol::config_types::SandboxMode;

use super::*;
use crate::legacy_core::config::ConfigBuilder;
use codex_config::LoaderOverrides;

async fn load(home: &Path, cwd: &Path, user_config: &str) -> Config {
    std::fs::write(home.join("config.toml"), user_config).unwrap();
    let mut cli = base_overrides(home);
    cli.extend(env_overrides(&ShellEnvironmentPolicyToml::default()));
    let mut harness = ConfigOverrides {
        cwd: Some(cwd.to_path_buf()),
        ..Default::default()
    };
    apply_launch_overrides(&mut harness);
    ConfigBuilder::default()
        .codex_home(home.to_path_buf())
        .cli_overrides(cli)
        .harness_overrides(harness)
        .loader_overrides(LoaderOverrides::without_managed_config_for_tests())
        .build()
        .await
        .unwrap()
}

fn names(overrides: &[(String, toml::Value)]) -> Vec<String> {
    overrides.iter().map(|(key, _)| key.clone()).collect()
}

#[test]
fn env_overrides_extend_legacy_excludes_and_blank_secret_sets() {
    let user = ShellEnvironmentPolicyToml {
        exclude: Some(vec!["AWS_*".to_string(), "*VAULT*".to_string()]),
        r#set: Some(HashMap::from([
            ("PATH_EXTRA".to_string(), "/opt/bin".to_string()),
            ("DEPLOY_TOKEN".to_string(), "value".to_string()),
        ])),
        ..Default::default()
    };
    let exclude = [
        "AWS_*",
        "*VAULT*",
        "*PASSWORD*",
        "*PASSPHRASE*",
        "*CREDENTIAL*",
    ]
    .map(|pattern| toml::Value::String(pattern.to_string()));
    assert_eq!(
        env_overrides(&user),
        vec![
            (
                "shell_environment_policy.exclude".to_string(),
                toml::Value::Array(exclude.to_vec())
            ),
            (
                "shell_environment_policy.set.DEPLOY_TOKEN".to_string(),
                toml::Value::String(String::new())
            ),
        ]
    );
}

#[test]
fn env_overrides_keep_canonical_filters() {
    let user = ShellEnvironmentPolicyToml {
        filters: Some(BTreeMap::from([(
            "AWS_*".to_string(),
            ShellEnvironmentPolicyFilter::Exclude,
        )])),
        ..Default::default()
    };
    assert_eq!(
        names(&env_overrides(&user)),
        SECRET_ENV_PATTERNS
            .map(|pattern| format!("shell_environment_policy.filters.{pattern}"))
            .to_vec()
    );
}

#[test]
fn launch_flags_cannot_weaken_aggressive() {
    let mut overrides = ConfigOverrides {
        approval_policy: Some(AskForApproval::Never),
        sandbox_mode: Some(SandboxMode::DangerFullAccess),
        additional_writable_roots: vec!["/elsewhere".into()],
        default_permissions: Some(":danger-full-access".to_string()),
        ..Default::default()
    };
    assert_eq!(
        apply_launch_overrides(&mut overrides),
        vec!["--ask-for-approval", "--sandbox", "--add-dir"]
    );
    assert_eq!(
        (
            overrides.approval_policy,
            overrides.approvals_reviewer,
            overrides.sandbox_mode,
            overrides.default_permissions.as_deref(),
            overrides.additional_writable_roots,
        ),
        (
            Some(AskForApproval::UnlessTrusted),
            Some(ApprovalsReviewer::User),
            None,
            Some(PROFILE_ID),
            Vec::new(),
        )
    );
}

/// A permissive user config plus the overlay loads as every mapping row;
/// without the overlay the same config fails verification.
#[tokio::test]
async fn overlay_on_permissive_user_config_verifies_every_row() {
    let home = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();
    std::fs::write(
        home.path().join("config.toml"),
        r#"approval_policy = "never"
approvals_reviewer = "auto_review"
sandbox_mode = "danger-full-access"
web_search = "live"

[sandbox_workspace_write]
writable_roots = ["/elsewhere"]
network_access = true

[shell_environment_policy]
set = { MY_API_KEY = "abc" }
"#,
    )
    .unwrap();
    let build = |cli: Vec<(String, toml::Value)>, harness: ConfigOverrides| {
        ConfigBuilder::default()
            .codex_home(home.path().to_path_buf())
            .loader_overrides(LoaderOverrides::without_managed_config_for_tests())
            .cli_overrides(cli)
            .harness_overrides(ConfigOverrides {
                cwd: Some(cwd.path().to_path_buf()),
                ..harness
            })
            .build()
    };
    let plain = build(Vec::new(), ConfigOverrides::default()).await.unwrap();
    let rows = verify(&plain, /*rules_present*/ false)
        .iter()
        .filter_map(|failure| failure.split(':').next().map(str::to_string))
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        rows,
        ["Approvals", "Network", "Sandbox", "Vault"]
            .map(str::to_string)
            .into()
    );

    let user_env = ShellEnvironmentPolicyToml {
        r#set: Some(HashMap::from([(
            "MY_API_KEY".to_string(),
            "abc".to_string(),
        )])),
        ..Default::default()
    };
    let mut cli = base_overrides(home.path());
    cli.extend(env_overrides(&user_env));
    let mut harness = ConfigOverrides::default();
    apply_launch_overrides(&mut harness);
    let config = build(cli, harness).await.unwrap();
    assert_eq!(
        verify(&config, /*rules_present*/ true),
        Vec::<String>::new()
    );
    assert_eq!(
        verify(&config, /*rules_present*/ false),
        vec!["Vault: the exec-policy rule file is missing".to_string()]
    );
}

/// A workspace that contains the Corbanu home still cannot rewrite the level.
#[tokio::test]
async fn codex_home_inside_the_workspace_stays_read_only() {
    let cwd = tempfile::tempdir().unwrap();
    let home = cwd.path().join(".corbanu");
    std::fs::create_dir_all(&home).unwrap();
    let config = load(&home, cwd.path(), "").await;
    assert_eq!(
        verify(&config, /*rules_present*/ true),
        Vec::<String>::new()
    );
}

/// User or project layers cannot widen the profile through a table merge.
#[tokio::test]
async fn another_layer_defining_the_profile_fails_verification() {
    let home = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();
    let config = load(
        home.path(),
        cwd.path(),
        "[permissions.corbanu-aggressive.network]\nenabled = true\n",
    )
    .await;
    let failures = verify(&config, /*rules_present*/ true);
    assert!(
        failures
            .iter()
            .any(|failure| failure.contains("also defines permissions.corbanu-aggressive")),
        "{failures:?}"
    );
}

/// A custom role that would hand children different values is refused.
#[tokio::test]
async fn role_that_changes_child_values_fails_verification() {
    let home = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(home.path().join("agents")).unwrap();
    std::fs::write(
        home.path().join("agents/loose.toml"),
        "web_search = \"live\"\n[features]\nshell_snapshot = true\n",
    )
    .unwrap();
    std::fs::write(
        home.path().join("agents/plain.toml"),
        "developer_instructions = \"Be brief\"\n",
    )
    .unwrap();
    let config = load(
        home.path(),
        cwd.path(),
        "[agents.loose]\ndescription = \"x\"\nconfig_file = \"./agents/loose.toml\"\n[agents.plain]\ndescription = \"y\"\nconfig_file = \"./agents/plain.toml\"\n",
    )
    .await;
    assert_eq!(
        verify(&config, /*rules_present*/ true),
        vec!["Child agents: role `loose` sets web_search, features.shell_snapshot".to_string()]
    );
}

/// The launch check loads the whole exec policy the way a session does.
#[tokio::test]
async fn exec_policy_must_load_and_forbid_vault_commands() {
    let home = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();
    super::super::level::sync_rules(home.path(), super::super::level::ChosenLevel::Aggressive)
        .unwrap();
    let rules = home.path().join("rules");
    let config = load(home.path(), cwd.path(), "").await;
    assert_eq!(verify_exec_policy(&config).await, Vec::<String>::new());

    // A broken file next to the vault rule: a session would drop every rule.
    let broken = rules.join("mine.rules");
    std::fs::write(&broken, "prefix_rule(pattern = [\"git\"], decision = \n").unwrap();
    let failures = verify_exec_policy(&config).await;
    assert_eq!(failures.len(), 1, "{failures:?}");
    assert!(
        failures[0].starts_with(&format!(
            "Vault: an exec-policy rules file does not parse, so sessions would drop every rule, including the vault rule: {}:",
            broken.display()
        )),
        "{failures:?}"
    );
    std::fs::remove_file(&broken).unwrap();

    // `host_executable` would make the rule skip `corbanu` at other paths.
    std::fs::write(
        rules.join("paths.rules"),
        "host_executable(name = \"corbanu\", paths = [\"/opt/elsewhere/corbanu\"])\n",
    )
    .unwrap();
    assert_eq!(
        verify_exec_policy(&config).await,
        vec![
            "Vault: host_executable rules limit `corbanu` to listed paths, so the vault rule would not apply at other paths"
                .to_string()
        ]
    );
    std::fs::remove_file(rules.join("paths.rules")).unwrap();

    std::fs::remove_file(super::super::level::rules_path(home.path())).unwrap();
    assert_eq!(
        verify_exec_policy(&config).await,
        vec![
            "Vault: the loaded exec policy does not forbid `corbanu vault`, `codex vault`, `pfterminal vault`, `corbanu-debug vault`, `pfterminal-debug vault`"
                .to_string()
        ]
    );
}

/// A broken `.rules` file in the project's `.codex/rules` counts only when the
/// project is trusted, exactly as a session loads it.
#[tokio::test]
async fn project_rules_count_only_when_the_project_is_trusted() {
    let home = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();
    super::super::level::sync_rules(home.path(), super::super::level::ChosenLevel::Aggressive)
        .unwrap();
    let project_rules = cwd.path().join(".codex").join("rules");
    std::fs::create_dir_all(&project_rules).unwrap();
    std::fs::write(
        project_rules.join("broken.rules"),
        "prefix_rule(pattern = [\"git\"], decision = \n",
    )
    .unwrap();

    let untrusted = load(home.path(), cwd.path(), "").await;
    assert_eq!(verify_exec_policy(&untrusted).await, Vec::<String>::new());

    let trusted = load(
        home.path(),
        cwd.path(),
        &format!(
            "[projects.{:?}]\ntrust_level = \"trusted\"\n",
            cwd.path().display().to_string()
        ),
    )
    .await;
    let failures = verify_exec_policy(&trusted).await;
    assert_eq!(failures.len(), 1, "{failures:?}");
    assert!(
        failures[0].contains("does not parse") && failures[0].contains("broken.rules"),
        "{failures:?}"
    );
}
