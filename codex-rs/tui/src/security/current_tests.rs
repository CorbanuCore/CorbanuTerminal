use std::collections::HashMap;

use codex_config::LoaderOverrides;
use codex_config::types::ShellEnvironmentPolicyToml;
use codex_protocol::config_types::EnvironmentVariablePattern;
use pretty_assertions::assert_eq;

use super::*;
use crate::legacy_core::config::ConfigBuilder;
use crate::legacy_core::config::ConfigOverrides;
use crate::security::aggressive;

async fn build(home: &Path, cwd: &Path, user_config: &str, aggressive_level: bool) -> Config {
    std::fs::write(home.join("config.toml"), user_config).unwrap();
    let mut cli = Vec::new();
    let mut harness = ConfigOverrides {
        cwd: Some(cwd.to_path_buf()),
        ..Default::default()
    };
    if aggressive_level {
        cli = aggressive::base_overrides(home);
        cli.extend(aggressive::env_overrides(
            &ShellEnvironmentPolicyToml::default(),
        ));
        aggressive::apply_launch_overrides(&mut harness);
    }
    ConfigBuilder::default()
        .codex_home(home.to_path_buf())
        .cli_overrides(cli)
        .harness_overrides(harness)
        .loader_overrides(LoaderOverrides::without_managed_config_for_tests())
        .build()
        .await
        .unwrap()
}

/// A legacy workspace-write sandbox without temp folders, with an extra
/// writable root and the given network setting.
fn workspace_write(extra: &Path, network: bool) -> String {
    format!(
        r#"approval_policy = "on-request"
sandbox_mode = "workspace-write"
web_search = "live"

[sandbox_workspace_write]
writable_roots = [{extra:?}]
network_access = {network}
exclude_tmpdir_env_var = true
exclude_slash_tmp = true
"#,
        extra = extra.display().to_string()
    )
}

/// Code-blind case B-06: the review shows what the user would lose. The
/// legacy sandbox has no denied reads, so approved commands can leave it,
/// and its "off" network and vault rows say so.
#[tokio::test]
async fn workspace_write_rows_name_the_escape_route() {
    let home = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();
    let extra = tempfile::tempdir().unwrap();
    let extra_path = extra.path().display().to_string();

    let open = build(
        home.path(),
        cwd.path(),
        &workspace_write(extra.path(), true),
        false,
    )
    .await;
    let [sandbox, approvals, network, vault, _] = current_values(&open);
    assert_eq!(
        [
            sandbox,
            approvals,
            network,
            vault.split("; ").next().unwrap().to_string()
        ],
        [
            format!(
                "commands can write to the current folder, {extra_path}; approved or allow-listed commands can run outside the sandbox; permission-request tools are off"
            ),
            "on-request (reviewer: you)".to_string(),
            "on for agent commands; web search live".to_string(),
            "the vault store and sign-in file are readable to agent commands".to_string(),
        ]
    );

    let closed = build(
        home.path(),
        cwd.path(),
        &workspace_write(extra.path(), false),
        false,
    )
    .await;
    let [_, _, network, _, _] = current_values(&closed);
    assert_eq!(
        network,
        "off inside the sandbox, on for commands that run outside it; web search live"
    );
}

/// Under Aggressive every row matches the Aggressive value.
#[tokio::test]
async fn aggressive_rows_claim_only_what_is_enforced() {
    let home = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();
    let extra = tempfile::tempdir().unwrap();
    let config = build(
        home.path(),
        cwd.path(),
        &workspace_write(extra.path(), true),
        true,
    )
    .await;
    let [sandbox, approvals, network, vault, children] = current_values(&config);
    assert_eq!(
        [sandbox, approvals, network, vault, children],
        [
            format!(
                "profile {}: commands can write to the current folder; permission-request tools are off",
                aggressive::PROFILE_ID
            ),
            "untrusted (reviewer: you)".to_string(),
            "off for agent commands; web search disabled".to_string(),
            "the vault store and sign-in file are unreadable to agent commands; secret-like environment variables are removed; login profiles and shell snapshots are not used".to_string(),
            "spawned agents get this session's values".to_string(),
        ]
    );
}

/// Names the user lists explicitly are reported, not just the fixed probes.
#[test]
fn secret_kinds_include_named_variables_from_the_environment() {
    let env = || {
        [
            ("PATH", "/bin"),
            ("OPENAI_API_KEY", "fake"),
            ("STRIPE_SECRET", "fake"),
        ]
        .map(|(name, value)| (name.to_string(), value.to_string()))
    };
    let include_only = ShellEnvironmentPolicy {
        include_only: vec![EnvironmentVariablePattern::new_case_insensitive(
            "OPENAI_API_KEY",
        )],
        ..Default::default()
    };
    // Aggressive's excludes: the built-in KEY/SECRET/TOKEN ones plus these.
    let exclude = ["*VAULT*", "*PASSWORD*", "*PASSPHRASE*", "*CREDENTIAL*"]
        .map(EnvironmentVariablePattern::new_case_insensitive)
        .to_vec();
    let removed = ShellEnvironmentPolicy {
        ignore_default_excludes: false,
        exclude: exclude.clone(),
        ..Default::default()
    };
    let set = ShellEnvironmentPolicy {
        ignore_default_excludes: false,
        exclude,
        r#set: HashMap::from([("DB_PASSWORD".to_string(), "fake".to_string())]),
        ..Default::default()
    };
    assert_eq!(
        [
            kept_secret_kinds(&include_only, env()),
            kept_secret_kinds(&removed, env()),
            kept_secret_kinds(&set, env()),
        ],
        [vec!["KEY"], Vec::new(), vec!["PASSWORD"]]
    );
}

/// A role counts as changing its children's values unless it sets only
/// model and instruction keys.
#[test]
fn roles_that_set_permission_keys_change_children() {
    let dir = tempfile::tempdir().unwrap();
    let write = |name: &str, contents: &str| {
        let path = dir.path().join(name);
        std::fs::write(&path, contents).unwrap();
        path
    };
    assert_eq!(
        [
            role_changes_values(&write(
                "model.toml",
                "model = \"m\"\ndeveloper_instructions = \"x\"\n"
            )),
            role_changes_values(&write(
                "network.toml",
                "[sandbox_workspace_write]\nnetwork_access = true\n"
            )),
            role_changes_values(&write("broken.toml", "model = \n")),
            role_changes_values(&dir.path().join("missing.toml")),
        ],
        [false, true, true, true]
    );
}
