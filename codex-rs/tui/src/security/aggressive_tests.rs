use std::collections::BTreeMap;
use std::collections::HashMap;

use codex_protocol::config_types::ShellEnvironmentPolicyFilter;
use pretty_assertions::assert_eq;

use codex_protocol::config_types::SandboxMode;

use super::*;
use crate::legacy_core::config::ConfigBuilder;

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
