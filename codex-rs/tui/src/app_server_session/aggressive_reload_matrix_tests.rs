//! D621 (#2310), owner decision (a): Aggressive keeps no `approval_policy` in
//! the session-flags layer and relies on the launch (harness) override alone.
//!
//! Upstream rejects `approval_policy = "untrusted"` in config, so
//! `security::aggressive::base_overrides` no longer sets it and only
//! `apply_launch_overrides` does. The embedded app server gets the
//! session-flags layer but never the harness overrides, so every thread
//! request either carries the policy or leaves it to the server, which then
//! resolves it from the session flags and the user's own config (or, on
//! resume, the saved thread). Each row below is one way this process or a
//! nested one starts or reattaches a thread, with the user's config asking for
//! `never`; the test reports every row before failing.

use std::path::Path;

use codex_config::LoaderOverrides;
use codex_config::types::ShellEnvironmentPolicyToml;
use codex_protocol::ThreadId;
use codex_protocol::protocol::AskForApproval as CoreAskForApproval;
use pretty_assertions::assert_eq;

use super::*;
use crate::legacy_core::config::ConfigBuilder;
use crate::legacy_core::config::ConfigOverrides;
use crate::resume_permissions::ResumePermissions;
use crate::security::aggressive;

/// The user's own config (and `-p loose`) asks for the loosest policy.
const PERMISSIVE_USER_CONFIG: &str = "approval_policy = \"never\"\n";

/// Aggressive's session flags, which hold no `approval_policy` key.
fn session_flags_without_approval(home: &Path) -> Vec<(String, toml::Value)> {
    let mut cli = aggressive::base_overrides_with_registry(home, home, /*registry*/ None);
    assert!(
        cli.iter().all(|(key, _)| key != "approval_policy"),
        "upstream rejects `untrusted` in any config layer"
    );
    cli.extend(aggressive::env_overrides(
        &ShellEnvironmentPolicyToml::default(),
    ));
    cli
}

fn launch_overrides(cwd: &Path) -> ConfigOverrides {
    let mut harness = ConfigOverrides {
        cwd: Some(cwd.to_path_buf()),
        ..Default::default()
    };
    aggressive::apply_launch_overrides(&mut harness);
    harness
}

async fn load(
    home: &Path,
    cli: &[(String, toml::Value)],
    harness: ConfigOverrides,
    loader: LoaderOverrides,
) -> Config {
    ConfigBuilder::default()
        .codex_home(home.to_path_buf())
        .cli_overrides(cli.to_vec())
        .harness_overrides(harness)
        .loader_overrides(loader)
        .build()
        .await
        .expect("Aggressive config without the session-flags approval key loads")
}

fn profile_loader() -> LoaderOverrides {
    let mut loader = LoaderOverrides::without_managed_config_for_tests();
    loader.user_config_profile = Some("loose".parse().expect("profile name"));
    loader
}

/// What a thread ends on: the policy the request carries, or else what the
/// embedded server resolves from its own load (session flags and user config,
/// no harness overrides).
fn effective(sent: Option<AskForApproval>, server: &Config) -> String {
    match sent {
        Some(policy) => format!("sent {policy:?}"),
        None => format!(
            "not sent; server resolves {:?}",
            AskForApproval::from(server.permissions.approval_policy.value())
        ),
    }
}

fn is_untrusted(sent: Option<AskForApproval>, server: &Config) -> bool {
    match sent {
        Some(policy) => policy == AskForApproval::UnlessTrusted,
        None => server.permissions.approval_policy.value() == CoreAskForApproval::UnlessTrusted,
    }
}

#[tokio::test]
async fn d621_every_reload_path_ends_on_untrusted() {
    let root = tempfile::tempdir().expect("tempdir");
    let root = dunce::canonicalize(root.path()).expect("canonical tempdir");
    let home = root.join("home");
    let workspace = root.join("workspace");
    std::fs::create_dir_all(&home).expect("home");
    std::fs::create_dir_all(&workspace).expect("workspace");
    std::fs::write(home.join("config.toml"), PERMISSIVE_USER_CONFIG).expect("user config");
    // `-p loose` selects `<home>/loose.config.toml` (upstream's profile files).
    std::fs::write(home.join("loose.config.toml"), PERMISSIVE_USER_CONFIG).expect("profile config");
    let cli = session_flags_without_approval(&home);
    let harness = launch_overrides(&workspace);
    let plain = LoaderOverrides::without_managed_config_for_tests;
    let config = load(&home, &cli, harness.clone(), plain()).await;
    let server = load(
        &home,
        &cli,
        ConfigOverrides {
            cwd: Some(workspace.clone()),
            ..Default::default()
        },
        plain(),
    )
    .await;
    let profile_config = load(&home, &cli, harness.clone(), profile_loader()).await;
    let profile_server = load(
        &home,
        &cli,
        ConfigOverrides {
            cwd: Some(workspace.clone()),
            ..Default::default()
        },
        profile_loader(),
    )
    .await;
    let thread_id = ThreadId::new();
    let mode = ThreadParamsMode::Embedded;
    // (path, what it ends on, whether that is `untrusted`)
    let mut rows: Vec<(&str, String, bool)> = Vec::new();
    let mut row = |path, sent: Option<AskForApproval>, server: &Config| {
        rows.push((path, effective(sent, server), is_untrusted(sent, server)));
    };

    // Launch: the config this process verifies before showing Aggressive.
    let launch = Some(config.permissions.approval_policy.value().into());
    row("launch config", launch, &server);
    let launch_p = Some(profile_config.permissions.approval_policy.value().into());
    row("launch config with -p", launch_p, &profile_server);

    // `/new` and the agents overview: thread/start from the loaded config.
    let start = thread_start_params_from_config(&config, mode, None, None);
    row(
        "thread/start from config (/new)",
        start.approval_policy,
        &server,
    );

    // The first thread at startup (`spawn_startup_thread_start`).
    let mut choices = StartupLaunchChoices::from_launch(&cli, &harness, &plain());
    let mut params = thread_start_params_with_reviewer(
        &config,
        mode,
        None,
        None,
        choices.approvals_reviewer,
        &choices.approval_policy,
    );
    choices.configure(&mut params);
    row("startup thread/start", params.approval_policy, &server);

    // The same with `-p <profile>`, which sends only the provider.
    choices = StartupLaunchChoices::from_launch(&cli, &harness, &profile_loader());
    let mut params = thread_start_params_with_reviewer(
        &profile_config,
        mode,
        None,
        None,
        choices.approvals_reviewer,
        &choices.approval_policy,
    );
    choices.configure(&mut params);
    row(
        "startup thread/start with -p",
        params.approval_policy,
        &profile_server,
    );

    // Resume by the paths that send the launch overrides (session picker,
    // `resume <id>`, the agents overview, spawn panes).
    let resume = thread_resume_params_from_config(
        config.clone(),
        thread_id,
        mode,
        None,
        ResumeModelSettings::RestoreFromThread,
        ResumePermissions::from_overrides(&config, &harness),
    );
    row(
        "thread/resume with launch overrides",
        resume.approval_policy,
        &server,
    );

    // Rejoining a loaded thread from the agents overview: the launch
    // overrides, but the rejoin otherwise keeps the thread's settings.
    let resume = thread_resume_params_from_config(
        config.clone(),
        thread_id,
        mode,
        None,
        ResumeModelSettings::PreserveExistingThread,
        ResumePermissions::from_overrides(&config, &harness),
    );
    row(
        "thread/resume rejoin from the agents overview",
        resume.approval_policy,
        &server,
    );

    // Rejoins that otherwise send no permission choice: thread routing's
    // snapshot refresh, attaching a live thread, and reconnect.
    let resume = thread_resume_params_from_config(
        config.clone(),
        thread_id,
        mode,
        None,
        ResumeModelSettings::PreserveExistingThread,
        ResumePermissions::launch_approval_only(&harness),
    );
    row(
        "thread/resume rejoin (thread routing, live attach, reconnect)",
        resume.approval_policy,
        &server,
    );

    // Restoring the operator's `/panes` sessions from the saved layout.
    let resume = thread_resume_params_from_config(
        config.clone(),
        thread_id,
        mode,
        None,
        ResumeModelSettings::RestoreFromThread,
        ResumePermissions::launch_approval_only(&harness),
    );
    row(
        "thread/resume restore (saved panes)",
        resume.approval_policy,
        &server,
    );

    // Fork (embedded): the policy comes from the loaded config.
    let fork = thread_fork_params_from_config(config.clone(), thread_id, mode, None);
    row("thread/fork", fork.approval_policy, &server);

    // A nested `corbanu exec` started by an agent command: the CLI's `-c`
    // overrides from `prepare_nested_exec`, minus the key, plus the launch
    // overrides it applies (`cli/src/nested_security.rs`).
    let nested_home = root.join("nested-home");
    std::fs::create_dir_all(&nested_home).expect("nested home");
    std::fs::write(nested_home.join("config.toml"), PERMISSIVE_USER_CONFIG)
        .expect("nested user config");
    let nested_cli = crate::security::nested::prepare_nested_exec(&nested_home, &home)
        .expect("nested overrides");
    let mut nested_harness = ConfigOverrides {
        cwd: Some(workspace.clone()),
        ..Default::default()
    };
    crate::security::nested::apply_aggressive_launch_overrides(&mut nested_harness);
    let nested = load(&nested_home, &nested_cli, nested_harness, plain()).await;
    let nested_policy = Some(nested.permissions.approval_policy.value().into());
    row("nested launch config", nested_policy, &server);

    // Child agents start from the parent's effective config and the turn's
    // runtime approval policy (`core/src/agent/child_config.rs`), and a role
    // cannot change `approval_policy` (`core/src/agent/role.rs`); they follow
    // the parent rows above.

    let report = rows
        .iter()
        .map(|(path, outcome, _)| format!("{path}: {outcome}"))
        .collect::<Vec<_>>()
        .join("\n");
    let failing = rows
        .iter()
        .filter(|(_, _, untrusted)| !untrusted)
        .map(|(path, _, _)| *path)
        .collect::<Vec<_>>();
    assert_eq!(failing, Vec::<&str>::new(), "matrix:\n{report}");
}

/// Outside Aggressive, the rejoin and restore paths still leave the policy to
/// the thread's saved settings, even after `--ask-for-approval`.
#[test]
fn launch_approval_only_is_aggressive_only() {
    let permissive = ConfigOverrides {
        approval_policy: Some(CoreAskForApproval::Never),
        ..Default::default()
    };
    assert_eq!(
        ResumePermissions::launch_approval_only(&permissive),
        ResumePermissions::default()
    );
    let mut aggressive_launch = permissive;
    aggressive::apply_launch_overrides(&mut aggressive_launch);
    assert_eq!(
        ResumePermissions::launch_approval_only(&aggressive_launch),
        ResumePermissions {
            aggressive_approval: true,
            ..Default::default()
        }
    );
}

/// A remote server never takes the client's approval policy (the Aggressive
/// launch refuses one), so the override is not sent there.
#[tokio::test]
async fn aggressive_approval_is_not_sent_to_a_remote_server() {
    let home = tempfile::tempdir().expect("tempdir");
    let config = ConfigBuilder::default()
        .codex_home(home.path().to_path_buf())
        .loader_overrides(LoaderOverrides::without_managed_config_for_tests())
        .build()
        .await
        .expect("config");
    let permissions = ResumePermissions {
        aggressive_approval: true,
        ..Default::default()
    };
    for model_settings in [
        ResumeModelSettings::PreserveExistingThread,
        ResumeModelSettings::RestoreFromThread,
    ] {
        let resume = thread_resume_params_from_config(
            config.clone(),
            ThreadId::new(),
            ThreadParamsMode::Remote,
            None,
            model_settings,
            permissions,
        );
        assert_eq!(resume.approval_policy, None, "{model_settings:?}");
    }
}
