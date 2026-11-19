use super::*;
#[cfg(unix)]
use crate::config::NetworkProxySpec;
#[cfg(unix)]
use crate::session::ThreadEnvironmentDefaults;
#[cfg(unix)]
use codex_network_proxy::CredentialProviderConfig;
#[cfg(unix)]
use codex_network_proxy::NetworkProxyConfig;
#[cfg(unix)]
use codex_protocol::config_types::EnvironmentVariablePattern;
#[cfg(unix)]
use codex_protocol::models::PermissionProfile;
#[cfg(unix)]
use codex_protocol::sandbox::SandboxOverride;
use core_test_support::PathBufExt;
use core_test_support::PathExt;
use pretty_assertions::assert_eq;
#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;
#[cfg(unix)]
use std::process::Command;
#[cfg(target_os = "linux")]
use std::process::Command as StdCommand;

use tempfile::tempdir;

#[cfg(unix)]
struct BlockingStdinPipe {
    original: i32,
    write_end: i32,
}

#[cfg(unix)]
impl BlockingStdinPipe {
    fn install() -> Result<Self> {
        let mut fds = [0i32; 2];
        if unsafe { libc::pipe(fds.as_mut_ptr()) } == -1 {
            return Err(std::io::Error::last_os_error()).context("create stdin pipe");
        }

        let original = unsafe { libc::dup(libc::STDIN_FILENO) };
        if original == -1 {
            let err = std::io::Error::last_os_error();
            unsafe {
                libc::close(fds[0]);
                libc::close(fds[1]);
            }
            return Err(err).context("dup stdin");
        }

        if unsafe { libc::dup2(fds[0], libc::STDIN_FILENO) } == -1 {
            let err = std::io::Error::last_os_error();
            unsafe {
                libc::close(fds[0]);
                libc::close(fds[1]);
                libc::close(original);
            }
            return Err(err).context("replace stdin");
        }

        unsafe {
            libc::close(fds[0]);
        }

        Ok(Self {
            original,
            write_end: fds[1],
        })
    }
}

#[cfg(unix)]
impl Drop for BlockingStdinPipe {
    fn drop(&mut self) {
        unsafe {
            libc::dup2(self.original, libc::STDIN_FILENO);
            libc::close(self.original);
            libc::close(self.write_end);
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn assert_posix_snapshot_sections(snapshot: &str) {
    assert!(snapshot.contains("# Snapshot file"));
    assert!(snapshot.contains("aliases "));
    assert!(snapshot.contains("exports "));
    assert!(
        snapshot.contains("PATH"),
        "snapshot should capture a PATH export"
    );
    assert!(snapshot.contains("setopts "));
}

#[cfg(not(windows))]
#[test]
fn preserved_provider_context_respects_unix_environment_casing() {
    let mut env = HashMap::from([
        ("GH_HOST".to_string(), "startup.example.com".to_string()),
        ("gh_host".to_string(), "custom.example.com".to_string()),
    ]);

    replace_provider_context_with_trusted(
        &mut env,
        &HashMap::new(),
        ["GH_HOST".to_string(), "gh_host".to_string()],
        &["gh_host".to_string()],
    );

    assert_eq!(
        env,
        HashMap::from([("gh_host".to_string(), "custom.example.com".to_string())])
    );
}

#[cfg(windows)]
#[test]
fn inherited_provider_context_matches_windows_environment_casing() {
    let mut env = HashMap::new();
    let inherited = HashMap::from([("gh_host".to_string(), "github.example.com".to_string())]);

    replace_provider_context_with_trusted(
        &mut env,
        &inherited,
        std::iter::once("GH_HOST".to_string()),
        &[],
    );

    assert_eq!(
        env,
        HashMap::from([("GH_HOST".to_string(), "github.example.com".to_string())])
    );

    insert_env_value(&mut env, "gh_host", "github.alternate.example");
    assert_eq!(
        env_value(&env, "GH_HOST").map(String::as_str),
        Some("github.alternate.example")
    );
    assert_eq!(env.len(), 1);
}

async fn get_snapshot(shell_type: ShellType) -> Result<String> {
    let dir = tempdir()?;
    let path = dir.path().join("snapshot.sh");
    let shell = crate::shell::get_shell(shell_type)
        .with_context(|| format!("No available shell for {shell_type:?}"))?;
    write_shell_snapshot(
        &shell,
        &path.abs(),
        &dir.path().abs(),
        &ShellEnvironmentPolicy::default(),
        /*credential_broker*/ None,
        /*sandbox*/ None,
        /*removed_env_vars*/ &[],
    )
    .await?;
    let content = fs::read_to_string(&path).await?;
    Ok(content)
}

#[cfg(unix)]
#[tokio::test]
async fn snapshot_helpers_preserve_thread_without_caching_exec_metadata() -> Result<()> {
    let output = Command::new(std::env::current_exe()?)
        .arg("--exact")
        .arg("shell_snapshot::tests::snapshot_helper_exec_metadata_child")
        .arg("--nocapture")
        .env("CODEX_TEST_SNAPSHOT_HELPER_CHILD", "1")
        .env("CODEX_THREAD_ID", "helper-thread")
        .env("CODEX_TOOL_CALL_ID", "stale-call")
        .env("BASH_ENV", "/dev/null")
        .output()?;

    assert!(
        output.status.success(),
        "snapshot helper child failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    Ok(())
}

#[cfg(unix)]
#[tokio::test]
async fn snapshot_helper_exec_metadata_child() -> Result<()> {
    if std::env::var_os("CODEX_TEST_SNAPSHOT_HELPER_CHILD").is_none() {
        return Ok(());
    }
    assert_eq!(
        std::env::var("CODEX_THREAD_ID").as_deref(),
        Ok("helper-thread")
    );
    assert_eq!(
        std::env::var("CODEX_TOOL_CALL_ID").as_deref(),
        Ok("stale-call")
    );

    let dir = tempdir()?;
    let shell = crate::shell::get_shell(ShellType::Bash).context("No available Bash shell")?;
    let helper_env = run_script_with_timeout(
        &shell,
        "printf '%s|%s' \"${CODEX_THREAD_ID-unset}\" \"${CODEX_TOOL_CALL_ID-unset}\"",
        SNAPSHOT_TIMEOUT,
        SnapshotShellMode::NonLogin,
        &dir.path().abs(),
        /*credential_broker*/ None,
        /*sandbox*/ None,
        /*removed_env_vars*/ &[],
    )
    .await?;
    assert_eq!(helper_env, "helper-thread|unset");
    let snapshot = get_snapshot(ShellType::Bash).await?;
    assert!(!snapshot.contains("CODEX_THREAD_ID"));
    assert!(!snapshot.contains("CODEX_TOOL_CALL_ID"));
    assert!(!snapshot.contains("helper-thread"));
    assert!(!snapshot.contains("stale-call"));

    let validation_path = dir.path().join("validation.sh").abs();
    fs::write(
        &validation_path,
        "test \"${CODEX_THREAD_ID-missing}\" = helper-thread && \
         test \"${CODEX_TOOL_CALL_ID-missing}\" = missing\n",
    )
    .await?;
    validate_snapshot(
        &shell,
        &validation_path,
        &dir.path().abs(),
        /*credential_broker*/ None,
        /*sandbox*/ None,
        /*removed_env_vars*/ &[],
    )
    .await?;
    Ok(())
}

#[test]
fn snapshot_file_name_parser_supports_legacy_and_suffixed_names() {
    let session_id = "019cf82b-6a62-7700-bbbd-46909794ef89";

    assert_eq!(
        snapshot_session_id_from_file_name(&format!("{session_id}.sh")),
        Some(session_id)
    );
    assert_eq!(
        snapshot_session_id_from_file_name(&format!("{session_id}.123.sh")),
        Some(session_id)
    );
    assert_eq!(
        snapshot_session_id_from_file_name(&format!("{session_id}.tmp-123")),
        Some(session_id)
    );
    assert_eq!(
        snapshot_session_id_from_file_name("not-a-snapshot.txt"),
        None
    );
}

#[cfg(unix)]
#[tokio::test]
async fn inactive_profiles_keep_snapshots_but_active_brokers_require_sandbox() -> Result<()> {
    use crate::config::PermissionProfileSnapshot;
    use crate::environment_selection::ThreadEnvironments;
    use crate::environment_selection::TurnEnvironmentSnapshot;
    use crate::tools::sandboxing::SandboxAttempt;
    use codex_exec_server::EnvironmentManager;
    use codex_exec_server::LOCAL_ENVIRONMENT_ID;
    use codex_protocol::config_types::WindowsSandboxLevel;
    use codex_protocol::protocol::EnvironmentConfig;
    use codex_protocol::protocol::EnvironmentConfigState;
    use codex_protocol::protocol::TurnEnvironmentSelection;
    use codex_sandboxing::SandboxManager;
    use codex_sandboxing::SandboxType;
    use tokio_util::sync::CancellationToken;

    let dir = tempdir()?;
    std::fs::create_dir(dir.path().join(".codex"))?;
    std::fs::write(
        dir.path().join(".codex/startup.sh"),
        "printf started > startup-ran\n",
    )?;
    let session_id = ThreadId::new();
    let session_telemetry = SessionTelemetry::new(
        session_id,
        "test",
        "test",
        /*account_id*/ None,
        /*account_email*/ None,
        /*auth_mode*/ None,
        "test".to_string(),
        /*log_user_prompts*/ false,
        "test".to_string(),
        codex_protocol::protocol::SessionSource::Cli,
    );
    let shell = Shell {
        shell_type: ShellType::Bash,
        shell_path: PathBuf::from("/bin/bash"),
    };
    let permission_profile = PermissionProfile::workspace_write();
    let mut network_config = NetworkProxyConfig {
        enabled: true,
        proxy_url: "http://127.0.0.1:0".to_string(),
        socks_url: "socks5://127.0.0.1:0".to_string(),
        ..NetworkProxyConfig::default()
    };
    network_config.set_credential_broker_enabled(/*enabled*/ true);
    let network_spec = NetworkProxySpec::from_config_and_constraints(
        network_config,
        /*requirements*/ None,
        &permission_profile,
    )?;
    let started_proxy = network_spec
        .start_proxy(
            &permission_profile,
            codex_network_proxy::ManagedProxyRouting::SharedIngress,
            codex_network_proxy::LocalBindingPolicy::DefaultFalse,
            /*policy_decider*/ None,
            /*blocked_request_observer*/ None,
            /*enable_network_approval_flow*/ false,
            Default::default(),
        )
        .await?;
    for (state, prefer_executor_snapshots, expect_snapshot) in [
        (SnapshotCredentialBrokerState::Unavailable, false, false),
        (SnapshotCredentialBrokerState::Inactive, false, true),
        (SnapshotCredentialBrokerState::Inactive, true, false),
        (
            SnapshotCredentialBrokerState::Ready(started_proxy.proxy()),
            false,
            false,
        ),
    ] {
        let (_sender, receiver) = watch::channel(state);
        let config = Arc::new(ShellSnapshotConfig {
            removed_env_vars: Default::default(),
            codex_home: dir.path().abs(),
            session_id,
            session_telemetry: session_telemetry.clone(),
            state_db: None,
            credential_broker: Some(receiver),
            prefer_executor_snapshots,
        });
        let snapshot = tokio::time::timeout(
            SNAPSHOT_TIMEOUT,
            ShellSnapshot::build_for_cwd(
                config,
                dir.path().abs(),
                shell.clone(),
                /*allow_login_shell*/ false,
                ShellEnvironmentPolicy {
                    r#set: HashMap::from([(
                        "BASH_ENV".to_string(),
                        "./.codex/startup.sh".to_string(),
                    )]),
                    ..ShellEnvironmentPolicy::default()
                },
                /*sandbox*/ None,
            ),
        )
        .await?;

        assert_eq!(snapshot.is_some(), expect_snapshot);
    }

    let (sender, _) = watch::channel(SnapshotCredentialBrokerState::Starting);
    let snapshot_builder = ShellSnapshot::new(
        dir.path().abs(),
        session_id,
        session_telemetry,
        /*state_db*/ None,
        Some(sender),
        /*prefer_executor_snapshots*/ false,
        /*removed_env_vars*/ Vec::new(),
    );
    let mut config = EnvironmentConfig {
        environment_hooks: Default::default(),
        allow_login_shell: false,
        workspace_roots: Vec::new(),
        windows_sandbox_level: WindowsSandboxLevel::Disabled,
        windows_sandbox_type: SandboxType::None,
        use_legacy_landlock: false,
        permission_profile: PermissionProfileSnapshot::legacy(permission_profile),
        shell_environment_policy: ShellEnvironmentPolicy::default(),
        exec_policy: None,
        mcp_policy: None,
        network_policy: None,
        selected_capability_roots: Vec::new(),
    };
    let manager = Arc::new(EnvironmentManager::default_for_tests());
    let environments = ThreadEnvironments::new(
        Arc::clone(&manager),
        shell.clone(),
        ThreadEnvironmentDefaults::new(config.clone(), config.windows_sandbox_type),
        snapshot_builder.clone(),
        TurnEnvironmentSnapshot::default(),
        /*non_blocking_snapshots*/ false,
    );
    environments.update_selections(&[TurnEnvironmentSelection {
        selected_capability_roots: Default::default(),
        environment_id: LOCAL_ENVIRONMENT_ID.to_string(),
        cwd: PathUri::from_abs_path(&dir.path().abs()),
        workspace_roots: Vec::new(),
        config: EnvironmentConfigState::FromThread,
    }]);
    for (state, expect_snapshot) in [
        (SnapshotCredentialBrokerState::Starting, false),
        (SnapshotCredentialBrokerState::Inactive, true),
        (
            SnapshotCredentialBrokerState::Ready(started_proxy.proxy()),
            false,
        ),
        (SnapshotCredentialBrokerState::Inactive, true),
    ] {
        environments.set_snapshot_credential_broker(state.clone());
        let turn = environments.snapshot().await;
        let environment = turn.primary().expect("local environment");
        let snapshot = timeout(SNAPSHOT_TIMEOUT, environment.shell_snapshot.clone()).await?;
        assert_eq!(snapshot.is_some(), expect_snapshot);
        assert_eq!(environment.shell_snapshot_v2_supported, expect_snapshot);

        environments.set_snapshot_credential_broker(state.clone());
        let next = environments.snapshot().await;
        assert!(Arc::ptr_eq(
            &environment.shell_snapshot_cache,
            &next.primary().expect("next turn").shell_snapshot_cache,
        ));
        if matches!(state, SnapshotCredentialBrokerState::Ready(_)) {
            config
                .shell_environment_policy
                .r#set
                .insert("CORP_REGION".into(), "west".into());
            environments.set_active_thread_defaults(config.clone());
            let updated = environments.snapshot().await;
            assert!(!Arc::ptr_eq(
                &environment.shell_snapshot_cache,
                &updated
                    .primary()
                    .expect("updated turn")
                    .shell_snapshot_cache,
            ));
            let child = ThreadEnvironments::new(
                Arc::clone(&manager),
                shell.clone(),
                ThreadEnvironmentDefaults::new(config.clone(), config.windows_sandbox_type),
                snapshot_builder.clone(),
                updated.clone(),
                /*non_blocking_snapshots*/ false,
            );
            let child = child.snapshot().await;
            assert!(!Arc::ptr_eq(
                &updated.primary().expect("parent turn").shell_snapshot_cache,
                &child.primary().expect("child turn").shell_snapshot_cache,
            ));
        }
    }
    assert!(!dir.path().join("startup-ran").exists());

    // Identical concurrent captures must not share one command's cancellation.
    fs::write(
        dir.path().join(".codex/startup.sh"),
        "export OPENAI_API_KEY=sk-snapshot-cache-test\nprintf started > capture-started\nwhile [ ! -f finish-startup ]; do sleep 0.01; done\n",
    )
    .await?;
    config.shell_environment_policy.r#set.insert(
        "BASH_ENV".into(),
        dir.path().join(".codex/startup.sh").display().to_string(),
    );
    environments.set_snapshot_credential_broker(SnapshotCredentialBrokerState::Ready(
        started_proxy.proxy(),
    ));
    environments.set_active_thread_defaults(config.clone());
    let turn = environments.snapshot().await;
    let environment = turn.primary().expect("brokered environment");
    let mut tool_config = crate::config::ConfigBuilder::without_managed_config_for_tests()
        .codex_home(dir.path().to_path_buf())
        .build()
        .await?;
    tool_config
        .features
        .enable(codex_features::Feature::ShellSnapshot)?;
    tool_config.permissions.network = Some(network_spec);
    let cwd = dir.path().abs();
    let cwd_uri = PathUri::from_abs_path(&cwd);
    let manager = SandboxManager::new();
    let permissions = PermissionProfile::Disabled;
    let cancellation = CancellationToken::new();
    let mut attempt = SandboxAttempt {
        sandbox_override: SandboxOverride::NoOverride,
        sandbox: SandboxType::None,
        sandbox_requested: false,
        permissions: &permissions,
        exec_server_permissions: &permissions,
        enforce_managed_network: false,
        manager: &manager,
        sandbox_cwd: &cwd_uri,
        workspace_roots: std::slice::from_ref(&cwd_uri),
        sandbox_exe: None,
        use_legacy_landlock: false,
        windows_sandbox_type: SandboxType::None,
        windows_sandbox_level: WindowsSandboxLevel::Disabled,
        network_denial_cancellation_token: Some(cancellation.clone()),
        network_proxy: None,
    };
    let first_sandbox = ShellSnapshotSandbox::new(
        &attempt,
        /*network*/ None,
        LOCAL_ENVIRONMENT_ID,
        /*additional_permissions*/ None,
    );
    attempt.network_denial_cancellation_token = Some(CancellationToken::new());
    let second_sandbox = ShellSnapshotSandbox::new(
        &attempt,
        /*network*/ None,
        LOCAL_ENVIRONMENT_ID,
        /*additional_permissions*/ None,
    );
    let command = shell.derive_exec_args("true", /*use_login_shell*/ false);
    assert!(
        environment
            .shell_snapshot(&cwd, &command, &shell, &tool_config, /*sandbox*/ None)
            .await
            .is_none()
    );
    let first = async {
        let snapshot = environment
            .shell_snapshot(&cwd, &command, &shell, &tool_config, Some(first_sandbox))
            .await;
        fs::write(dir.path().join("finish-startup"), "").await?;
        Ok::<_, anyhow::Error>(snapshot)
    };
    let second = environment.shell_snapshot(
        &cwd,
        &command,
        &shell,
        &tool_config,
        Some(second_sandbox.clone()),
    );
    let cancel_after_startup = async {
        while !dir.path().join("capture-started").exists() {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        started_proxy
            .proxy()
            .add_allowed_domain("startup.example")
            .await
            .expect("startup network approval should not invalidate either capture");
        cancellation.cancel();
    };
    let (first, second, ()) = timeout(SNAPSHOT_TIMEOUT * 2, async {
        tokio::join!(biased; first, second, cancel_after_startup)
    })
    .await?;
    assert!(first?.is_none(), "the cancelled capture must fail");
    let second = second.context("the unrelated capture must succeed")?;
    let reused = environment
        .shell_snapshot(
            &cwd,
            &command,
            &shell,
            &tool_config,
            Some(second_sandbox.clone()),
        )
        .await
        .context("successful captures remain reusable")?;
    assert!(Arc::ptr_eq(&second, &reused));

    fs::remove_file(reused.path()).await?;
    let rebuilt = environment
        .shell_snapshot(
            &cwd,
            &command,
            &shell,
            &tool_config,
            Some(second_sandbox.clone()),
        )
        .await
        .context("a deleted cached snapshot must be rebuilt")?;
    assert!(!Arc::ptr_eq(&reused, &rebuilt));
    assert!(rebuilt.path().as_path().is_file());
    let proxy = started_proxy.proxy();
    let mut old_env = HashMap::new();
    let context = rebuilt.restore_credentials(&mut old_env, environment.shell_environment_policy());
    old_env = context
        .prepare_child_environment(&proxy, old_env, /*environment_id*/ None)?
        .env;

    let original_config = proxy.current_cfg().await?;
    let mut changed_config = original_config.clone();
    changed_config.set_credential_broker_openai_base_url(Some("https://gateway.example/v1"));
    for (config, remains_valid) in [
        (original_config.clone(), true),
        (changed_config, false),
        (original_config, false),
    ] {
        proxy
            .replace_config_state(codex_network_proxy::build_config_state(
                config,
                codex_network_proxy::NetworkProxyConstraints::default(),
                codex_utils_path_uri::Platform::native(),
            )?)
            .await?;
        assert_eq!(
            rebuilt.is_brokered_for(environment.shell_environment_policy()),
            remains_valid
        );
    }
    let refreshed = environment
        .shell_snapshot(
            &cwd,
            &command,
            &shell,
            &tool_config,
            Some(second_sandbox.clone()),
        )
        .await
        .context("a changed broker configuration must invalidate cached snapshots")?;
    assert!(!Arc::ptr_eq(&rebuilt, &refreshed));
    let mut new_env = HashMap::new();
    let context =
        refreshed.restore_credentials(&mut new_env, environment.shell_environment_policy());
    new_env = context
        .prepare_child_environment(&proxy, new_env, /*environment_id*/ None)?
        .env;
    // D353: the retained broker keeps a stable placeholder for this credential;
    // the snapshot itself must still be invalidated and rebuilt above.
    assert_eq!(new_env["OPENAI_API_KEY"], old_env["OPENAI_API_KEY"]);
    assert_ne!(new_env["OPENAI_API_KEY"], "sk-snapshot-cache-test");

    // A credential change during capture should recapture once under the new configuration.
    fs::remove_file(refreshed.path()).await?;
    fs::remove_file(dir.path().join("finish-startup")).await?;
    fs::remove_file(dir.path().join("capture-started")).await?;
    fs::write(
        dir.path().join(".codex/startup.sh"),
        "export OPENAI_API_KEY=sk-snapshot-cache-test\nprintf x >> capture-started\nwhile [ ! -f finish-startup ]; do sleep 0.01; done\n",
    )
    .await?;
    let capture =
        environment.shell_snapshot(&cwd, &command, &shell, &tool_config, Some(second_sandbox));
    let reconfigure = async {
        while !dir.path().join("capture-started").exists() {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        let mut config = proxy.current_cfg().await?;
        config.set_credential_broker_openai_base_url(Some("https://new-gateway.example/v1"));
        proxy
            .replace_config_state(codex_network_proxy::build_config_state(
                config,
                codex_network_proxy::NetworkProxyConstraints::default(),
                codex_utils_path_uri::Platform::native(),
            )?)
            .await?;
        fs::write(dir.path().join("finish-startup"), "").await?;
        Ok::<_, anyhow::Error>(())
    };
    let (recaptured, updated) = timeout(SNAPSHOT_TIMEOUT * 2, async {
        tokio::join!(capture, reconfigure)
    })
    .await?;
    updated?;
    let recaptured = recaptured.context("credential changes during startup must recapture")?;
    assert!(recaptured.is_brokered_for(environment.shell_environment_policy()));
    assert_eq!(
        fs::read_to_string(dir.path().join("capture-started")).await?,
        "xx"
    );
    Ok(())
}

#[cfg(unix)]
async fn credential_snapshot_proxy() -> Result<crate::config::StartedNetworkProxy> {
    let mut network_config = NetworkProxyConfig {
        enabled: true,
        ..NetworkProxyConfig::default()
    };
    network_config.set_credential_broker_enabled(/*enabled*/ true);
    network_config.set_allowed_domains(vec!["*".to_string()]);
    network_config.allow_local_binding = Some(true);
    network_config.credential_providers.insert(
        "local".to_string(),
        CredentialProviderConfig {
            env: vec!["LOCAL_TOKEN".to_string()],
            patterns: vec!["local_[a-z]{24}".to_string()],
            url_prefixes: vec!["https://public.local-provider.example/v1".to_string()],
            url_prefix_from_env: Some("LOCAL_URL".to_string()),
            ..CredentialProviderConfig::default()
        },
    );
    network_config.credential_providers.insert(
        "stripe".to_string(),
        CredentialProviderConfig {
            env: vec!["STRIPE_API_KEY".to_string()],
            patterns: vec!["stripe_live_[a-z]{24}".to_string()],
            url_prefix_from_env: Some("STRIPE_HOST".to_string()),
            ..CredentialProviderConfig::default()
        },
    );
    network_config.credential_providers.insert(
        "vendor".to_string(),
        CredentialProviderConfig {
            env: vec!["VENDOR_PASSWORD".to_string()],
            patterns: vec!["pin_[a-z]{8}".to_string()],
            url_prefix_from_env: Some("VENDOR_HOST".to_string()),
            ..CredentialProviderConfig::default()
        },
    );
    network_config.configure_credential_broker_environment(&HashMap::from([(
        "STRIPE_HOST".to_string(),
        "api.stripe.example".to_string(),
    )]));
    let permission_profile = PermissionProfile::workspace_write();
    let network_spec = NetworkProxySpec::from_config_and_constraints(
        network_config,
        /*requirements*/ None,
        &permission_profile,
    )?;
    Ok(network_spec
        .start_proxy(
            &permission_profile,
            codex_network_proxy::ManagedProxyRouting::SharedIngress,
            codex_network_proxy::LocalBindingPolicy::DefaultFalse,
            /*policy_decider*/ None,
            /*blocked_request_observer*/ None,
            /*enable_network_approval_flow*/ false,
            Default::default(),
        )
        .await?)
}

#[cfg(unix)]
#[tokio::test]
async fn snapshot_discovers_and_redacts_shell_initialized_credentials() -> Result<()> {
    let dir = tempdir()?;
    let startup = dir.path().join("startup.sh");
    let witness =
        shlex::try_quote(dir.path().join("d682-startup-witness").to_str().unwrap())?.into_owned();
    let entered =
        shlex::try_quote(dir.path().join("d682-startup-entered").to_str().unwrap())?.into_owned();
    std::fs::write(
        &startup,
        "printf entered > __D682_ENTERED__\n\
         unset GITHUB_ENTERPRISE_TOKEN UNSET_AUTH_HEADER\n\
         export GH_TOKEN='ghp_shell_only_secret'\n\
         export AUTH_HEADER=\"Bearer $GH_TOKEN\"\n\
         declare -rx GITHUB_TOKEN='ghp_readonly_secret'\n\
         declare -rx HOMEBREW_GITHUB_API_TOKEN=\"$GITHUB_TOKEN\"\n\
         export GH_ENTERPRISE_TOKEN='ghp_enterprise_secret'\n\
         export GH_HOST='attacker.example'\n\
         export OPENAI_API_KEY='sk-proj-snapshot-secret'\n\
         export STRIPE_API_KEY='stripe_live_abcdefghijklmnopqrstuvwx'\n\
         export STRIPE_HOST='https://startup.stripe.example/v1'\n\
         export STRIPE_AUTH_HEADER=\"Bearer $STRIPE_API_KEY\"\n\
         export VENDOR_PASSWORD='pin_abcdefgh'\n\
         export VENDOR_AUTH_HEADER=\"Bearer $VENDOR_PASSWORD\"\n\
         export VENDOR_HOST='https://attacker.vendor.example/v2'\n\
         export LOCAL_TOKEN='local_abcdefghijklmnopqrstuvwx'\n\
         export LOCAL_URL='http://127.0.0.1:1234/v1'\n\
         export LOCAL_AUTH_HEADER=\"Bearer $LOCAL_TOKEN\"\n\
         unset LOCAL_TOKEN\n\
         export AUTH_BUNDLE=\"GitHub $GH_TOKEN\n\
         OpenAI $OPENAI_API_KEY\"\n\
         export OPENAI_BASE_URL='https://api.snapshot.example/v1'\n\
         export IDENTITY_SEEN=\"${OPENAI_IDENTITY_TOKEN_FILE-missing}\"\n\
         export EXCLUDED_PARENT_HOME=\"${HOME-missing}\"\n\
         export STARTUP_PATH_OVERRIDE_SEEN=\"${PATH%%:*}\"\n\
         export STARTUP_CORP_REGION_SEEN=\"${CORP_REGION-missing}\"\n\
         export STARTUP_NPM_TOKEN_SEEN=\"${NPM_TOKEN-missing}\"\n\
         test \"$GH_TOKEN\" = ghp_shell_only_secret && \\\n\
         test \"$GITHUB_TOKEN\" = ghp_readonly_secret && \\\n\
         test \"$GH_ENTERPRISE_TOKEN\" = ghp_enterprise_secret && \\\n\
         test \"$OPENAI_API_KEY\" = sk-proj-snapshot-secret && \\\n\
         test \"$STRIPE_API_KEY\" = stripe_live_abcdefghijklmnopqrstuvwx && \\\n\
         test \"$VENDOR_PASSWORD\" = pin_abcdefgh && \\\n\
         test \"$LOCAL_AUTH_HEADER\" = 'Bearer local_abcdefghijklmnopqrstuvwx' && \\\n\
         printf initialized > __D682_WITNESS__\n"
            .replace("__D682_ENTERED__", &entered)
            .replace("__D682_WITNESS__", &witness),
    )?;

    let started_proxy = credential_snapshot_proxy().await?;
    let network_proxy = started_proxy.proxy();

    let shell = Shell {
        shell_type: ShellType::Bash,
        shell_path: PathBuf::from("/bin/bash"),
    };
    // Initialize synthetic credentials explicitly in the capture shell. This
    // fixture tests hardened capture/replay, independently of implicit BASH_ENV
    // startup behavior. Validation and replay below use the real shell directly.
    let wrapper = dir.path().join("capture-shell");
    let startup_arg = shlex::try_quote(startup.to_str().unwrap())?;
    std::fs::write(
        &wrapper,
        format!(
            "#!/bin/sh\nunset BASH_ENV\nexec /bin/bash --noprofile --norc -c '. \"$1\" || exit 1; eval \"$2\"' d682-capture {startup_arg} \"$2\"\n"
        ),
    )?;
    std::fs::set_permissions(
        &wrapper,
        std::os::unix::fs::PermissionsExt::from_mode(0o700),
    )?;
    let capture_shell = Shell {
        shell_type: ShellType::Bash,
        shell_path: wrapper,
    };
    let shell_environment_policy = ShellEnvironmentPolicy {
        exclude: vec![
            EnvironmentVariablePattern::new_case_insensitive("HOME"),
            EnvironmentVariablePattern::new_case_insensitive("VENDOR_HOST"),
            EnvironmentVariablePattern::new_case_insensitive("STRIPE_HOST"),
            EnvironmentVariablePattern::new_case_insensitive("*TOKEN*"),
            EnvironmentVariablePattern::new_case_insensitive("*KEY*"),
            EnvironmentVariablePattern::new_case_insensitive("*PASSWORD*"),
        ],
        r#set: HashMap::from([
            ("GH_HOST".to_string(), "github.example.com".to_string()),
            (
                "GITHUB_ENTERPRISE_TOKEN".to_string(),
                "ghp_removed_enterprise_secret".to_string(),
            ),
            (
                "UNSET_AUTH_HEADER".to_string(),
                "Bearer ghp_removed_enterprise_secret".to_string(),
            ),
            (
                "OPENAI_IDENTITY_TOKEN_FILE".to_string(),
                "identity-token-secret".to_string(),
            ),
            ("PATH".to_string(), "/enterprise/bin".to_string()),
            ("CORP_REGION".to_string(), "production".to_string()),
            ("NPM_TOKEN".to_string(), "npm_enterprise_token".to_string()),
            (
                "VENDOR_HOST".to_string(),
                "https://api.vendor.example/v2".to_string(),
            ),
        ]),
        ..ShellEnvironmentPolicy::default()
    };
    // Generic fail-open aliases use capture-time source authorization, separate
    // from the inert upstream credential registry. Deny all seven fixture
    // sources and the readonly equivalent explicitly; policy.set can otherwise
    // override exclusions. Startup still creates them after launch filtering.
    for key in [
        "GH_TOKEN",
        "GITHUB_TOKEN",
        "HOMEBREW_GITHUB_API_TOKEN",
        "GH_ENTERPRISE_TOKEN",
        "OPENAI_API_KEY",
        "STRIPE_API_KEY",
        "VENDOR_PASSWORD",
        "LOCAL_TOKEN",
    ] {
        let allowed = create_env_from_vars(
            std::iter::once((key.to_string(), "fixture-source".to_string())),
            &shell_environment_policy,
            None,
        );
        assert!(!allowed.contains_key(key), "capture policy admitted {key}");
    }
    let credential_broker = SnapshotCredentialBroker {
        network_proxy: network_proxy.clone(),
        shell_environment_policy: shell_environment_policy.clone(),
        allow_login_shell: false,
    };
    let path = dir.path().join("snapshot.sh").abs();
    let credentials = write_shell_snapshot(
        &capture_shell,
        &path,
        &dir.path().abs(),
        &credential_broker.shell_environment_policy,
        Some(&credential_broker),
        /*sandbox*/ None,
        /*removed_env_vars*/ &[],
    )
    .await?;
    let initialized = fs::read_to_string(dir.path().join("d682-startup-witness"))
        .await
        .unwrap_or_default()
        == "initialized";
    let startup_entered = fs::read_to_string(dir.path().join("d682-startup-entered"))
        .await
        .unwrap_or_default()
        == "entered";
    assert!(startup_entered, "capture must source the startup fixture");
    assert!(
        initialized,
        "capture must run the startup fixture and initialize its seven credentials"
    );
    let captured_credentials = credentials.as_ref().expect("brokered snapshot credentials");
    // D353/D354/D640: configured-provider destination/replay metadata is inert.
    assert!(captured_credentials.context_env.is_empty());
    assert!(captured_credentials.protected_startup_env.is_none());
    // This separate generic alias map can authorize sources permitted at capture.
    // Here the capture policy denies these sources, including LOCAL_TOKEN whose
    // value was copied before unsetting it. Any retained aliases must stay denied.
    for key in [
        "STRIPE_AUTH_HEADER",
        "VENDOR_AUTH_HEADER",
        "LOCAL_AUTH_HEADER",
    ] {
        assert!(
            captured_credentials
                .fail_open_aliases
                .get(key)
                .is_none_or(|alias| !alias.authorized),
            "unsupported configured alias {key} was authorized for replay"
        );
    }
    let snapshot = fs::read_to_string(&path).await?;

    for secret in [
        "ghp_shell_only_secret",
        "ghp_readonly_secret",
        "ghp_enterprise_secret",
        "sk-proj-snapshot-secret",
        "stripe_live_abcdefghijklmnopqrstuvwx",
        "pin_abcdefgh",
        "local_abcdefghijklmnopqrstuvwx",
        "identity-token-secret",
    ] {
        assert!(!snapshot.contains(secret), "snapshot exposed {secret}");
    }
    assert!(!snapshot.contains("npm_enterprise_token"));
    assert!(!snapshot.contains("attacker.example"));
    assert!(!snapshot.contains("attacker.vendor.example"));
    assert!(!snapshot.contains("CODEX_NETWORK_PROXY_BROKERED_CREDENTIALS"));
    // Sensitive marker declarations may be omitted by the export filter. Replay
    // checks require identity/home omission or a safe sentinel and verify safe
    // startup settings semantically, without depending on export serialization.
    validate_snapshot(
        &shell,
        &path,
        &dir.path().abs(),
        Some(&credential_broker),
        /*sandbox*/ None,
        /*removed_env_vars*/ &[],
    )
    .await?;

    // D682 retires unsupported registered-provider restoration and destination
    // rebinding. This matrix also proves permissive replay filters cannot reopen
    // capture-time source denial through the separate generic alias helper.
    // Permitted generic fail-open behavior stays covered by the copied-credential
    // regression, not by a blanket claim that every alias is always denied.
    let snapshot_file = ShellSnapshotFile {
        path,
        credentials,
        shell_environment_policy: shell_environment_policy.clone(),
    };
    for (case, replay_policy) in [
        ("default", ShellEnvironmentPolicy::default()),
        (
            "include_credentials",
            ShellEnvironmentPolicy {
                ignore_default_excludes: true,
                include_only: vec![EnvironmentVariablePattern::new_case_insensitive("*")],
                ..ShellEnvironmentPolicy::default()
            },
        ),
        (
            "exclude_credentials",
            ShellEnvironmentPolicy {
                exclude: vec![
                    EnvironmentVariablePattern::new_case_insensitive("*TOKEN*"),
                    EnvironmentVariablePattern::new_case_insensitive("*KEY*"),
                    EnvironmentVariablePattern::new_case_insensitive("*PASSWORD*"),
                ],
                ..ShellEnvironmentPolicy::default()
            },
        ),
    ] {
        let mut env = HashMap::new();
        let context = snapshot_file.restore_credentials(&mut env, &replay_policy);
        assert_eq!(
            context.with_fallbacks(&env).as_ref(),
            &env,
            "{case} must not restore private destinations"
        );
        for brokered in [false, true] {
            let mut replay_env = if brokered {
                context
                    .clone()
                    .prepare_child_environment(
                        &network_proxy,
                        env.clone(),
                        Some("d682-snapshot-child"),
                    )?
                    .env
            } else {
                env.clone()
            };
            // Exercise both branches after preparation. Cleanup may remove entries,
            // and built-in credentials may remain masked; configured providers must
            // never gain a replay source, alias or destination.
            snapshot_file.restore_fail_open_aliases(&mut replay_env, Some("d682-snapshot-child"));
            for key in [
                "STRIPE_API_KEY",
                "VENDOR_PASSWORD",
                "LOCAL_TOKEN",
                "STRIPE_AUTH_HEADER",
                "VENDOR_AUTH_HEADER",
                "LOCAL_AUTH_HEADER",
                "STRIPE_HOST",
                "VENDOR_HOST",
            ] {
                assert!(
                    !replay_env.contains_key(key),
                    "{case}/{brokered} restored unsupported {key}"
                );
            }
            let replay = Command::new("/bin/bash")
                .arg("-c")
                .arg(
                    ". \"$1\" || exit 1\n\
                     test \"${IDENTITY_SEEN-missing}\" = missing || { printf '%s\\n' 'unsafe IDENTITY_SEEN' >&2; exit 1; }\n\
                     test \"${EXCLUDED_PARENT_HOME-missing}\" = missing || { printf '%s\\n' 'unsafe EXCLUDED_PARENT_HOME' >&2; exit 1; }\n\
                     test \"${STARTUP_PATH_OVERRIDE_SEEN-unset}\" = /enterprise/bin || { printf '%s\\n' 'missing startup PATH setting' >&2; exit 1; }\n\
                     test \"$PATH\" = /enterprise/bin || { printf '%s\\n' 'missing PATH setting' >&2; exit 1; }\n\
                     test \"${STARTUP_CORP_REGION_SEEN-unset}\" = production || { printf '%s\\n' 'missing startup region setting' >&2; exit 1; }\n\
                     test \"${CORP_REGION-unset}\" = production || { printf '%s\\n' 'missing region setting' >&2; exit 1; }\n\
                     export -p",
                )
                .arg("snapshot")
                .arg(snapshot_file.path().as_path())
                .env_clear()
                .envs(&replay_env)
                .output()?;
            assert!(
                replay.status.success(),
                "{case}/{brokered}: replay failed ({})",
                replay.status
            );
            let output = String::from_utf8(replay.stdout)?;
            for secret in [
                "ghp_shell_only_secret",
                "ghp_readonly_secret",
                "ghp_enterprise_secret",
                "sk-proj-snapshot-secret",
                "stripe_live_abcdefghijklmnopqrstuvwx",
                "pin_abcdefgh",
                "local_abcdefghijklmnopqrstuvwx",
                "identity-token-secret",
                "npm_enterprise_token",
                "ghp_removed_enterprise_secret",
            ] {
                assert!(
                    replay_env.values().all(|value| !value.contains(secret)),
                    "{case}/{brokered} restored a raw credential in the child environment"
                );
                assert!(
                    !output.contains(secret),
                    "{case}/{brokered} replay exposed {secret}"
                );
            }
            assert!(!output.contains("attacker.example"));
            assert!(!output.contains("attacker.vendor.example"));
            assert!(
                output.contains("CORP_REGION=\"production\""),
                "safe settings must survive"
            );
        }
    }
    Ok(())
}

#[cfg(unix)]
#[tokio::test]
async fn snapshot_protects_posix_startup_only_when_it_contains_credentials() -> Result<()> {
    let dir = tempdir()?;
    let started_proxy = credential_snapshot_proxy().await?;
    let network_proxy = started_proxy.proxy();
    let posix_startup = dir.path().join("posix-startup.sh");
    std::fs::write(
        &posix_startup,
        "export GH_TOKEN='ghp_posix_shell_secret'\nexport POSIX_STARTUP_LOADED=1\n",
    )?;
    let posix_shell = Shell {
        shell_type: ShellType::Sh,
        shell_path: PathBuf::from("/bin/sh"),
    };
    let posix_credential_broker = SnapshotCredentialBroker {
        network_proxy: network_proxy.clone(),
        shell_environment_policy: ShellEnvironmentPolicy {
            r#set: HashMap::from([("ENV".to_string(), "$PWD/posix-startup.sh".to_string())]),
            ..ShellEnvironmentPolicy::default()
        },
        allow_login_shell: true,
    };
    let posix_snapshot_path = dir.path().join("posix-snapshot.sh").abs();
    let posix_credentials = write_shell_snapshot(
        &posix_shell,
        &posix_snapshot_path,
        &dir.path().abs(),
        &posix_credential_broker.shell_environment_policy,
        Some(&posix_credential_broker),
        /*sandbox*/ None,
        /*removed_env_vars*/ &[],
    )
    .await?
    .expect("brokered POSIX snapshot has credentials");
    let posix_snapshot = fs::read_to_string(&posix_snapshot_path).await?;
    assert!(posix_snapshot.contains("POSIX_STARTUP_LOADED"));
    assert!(!posix_snapshot.contains("ghp_posix_shell_secret"));
    // D353/D354 keep upstream snapshot credential replay off. The retained
    // engine strips the secret from the snapshot without a protected replay path.
    assert!(posix_credentials.protected_startup_env.is_none());

    std::fs::write(&posix_startup, "export APPLICATION_SETTING=production\n")?;
    let application_snapshot_path = dir.path().join("posix-application-snapshot.sh").abs();
    let application_credentials = write_shell_snapshot(
        &posix_shell,
        &application_snapshot_path,
        &dir.path().abs(),
        &posix_credential_broker.shell_environment_policy,
        Some(&posix_credential_broker),
        /*sandbox*/ None,
        /*removed_env_vars*/ &[],
    )
    .await?
    .expect("brokered POSIX application snapshot has credentials");
    assert!(application_credentials.protected_startup_env.is_none());

    Ok(())
}

#[cfg(unix)]
#[tokio::test]
async fn snapshot_discovers_and_restores_inherited_credential_aliases() -> Result<()> {
    let dir = tempdir()?;
    let startup = dir.path().join("startup.sh");
    let started_proxy = credential_snapshot_proxy().await?;
    let network_proxy = started_proxy.proxy();
    let shell = Shell {
        shell_type: ShellType::Bash,
        shell_path: PathBuf::from("/bin/bash"),
    };
    let trusted_startup = ("BASH_ENV".to_string(), startup.display().to_string());
    std::fs::write(
        &startup,
        "export GH_TOKEN='ghp_hidden_alias_secret'\n\
         export HIDDEN_HEADER=\"prefix\\\"$GH_TOKEN\\\"suffix\"\n\
         export GITHUB_TOKEN='ghp_0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefgh'\n\
         export UNKNOWN_HEADER=\"Bearer $GITHUB_TOKEN\"\n\
         export STRIPE_API_KEY='stripe_live_zyxwvutsrqponmlkjihgfedc'\n\
         export STRIPE_HIDDEN_HEADER=\"Bearer $STRIPE_API_KEY\"\n\
         export STRIPE_HOST='api.stripe.example'\n\
         unset GH_TOKEN GITHUB_TOKEN GITHUB_ENTERPRISE_TOKEN STRIPE_API_KEY\n",
    )?;
    let mut previously_brokered_env = HashMap::from([(
        "GH_TOKEN".to_string(),
        "ghp_hidden_alias_secret".to_string(),
    )]);
    network_proxy.apply_to_env(&mut previously_brokered_env);
    let inherited_credential_broker = SnapshotCredentialBroker {
        network_proxy: network_proxy.clone(),
        shell_environment_policy: ShellEnvironmentPolicy {
            r#set: HashMap::from([trusted_startup]),
            ..ShellEnvironmentPolicy::default()
        },
        allow_login_shell: false,
    };
    let inherited_path = dir.path().join("inherited-snapshot.sh").abs();
    let inherited_credentials = write_shell_snapshot(
        &shell,
        &inherited_path,
        &dir.path().abs(),
        &inherited_credential_broker.shell_environment_policy,
        Some(&inherited_credential_broker),
        /*sandbox*/ None,
        /*removed_env_vars*/ &[],
    )
    .await?;
    let inherited_snapshot = fs::read_to_string(&inherited_path).await?;
    assert!(!inherited_snapshot.contains("ghp_hidden_alias_secret"));
    assert!(!inherited_snapshot.contains("ghp_0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefgh"));
    assert!(!inherited_snapshot.contains("stripe_live_zyxwvutsrqponmlkjihgfedc"));
    let inherited_replay = Command::new("/bin/bash")
        .arg("-c")
        .arg(". \"$1\" && printf '%s\\n%s\\n%s' \"${HIDDEN_HEADER-unset}\" \"${UNKNOWN_HEADER-unset}\" \"${STRIPE_HIDDEN_HEADER-unset}\"")
        .arg("snapshot")
        .arg(inherited_path.as_path())
        .env_clear()
        .output()?;
    assert!(inherited_replay.status.success());
    assert_eq!(
        String::from_utf8(inherited_replay.stdout)?,
        "unset\nunset\nunset"
    );

    let snapshot_file = ShellSnapshotFile {
        shell_environment_policy: ShellEnvironmentPolicy::default(),
        path: inherited_path,
        credentials: inherited_credentials,
    };
    // D640 retires upstream's alias rewriting: copies of a credential are not
    // replaced by brokered placeholders, so managed-network children get none.
    let policy = &inherited_credential_broker.shell_environment_policy;
    let mut brokered_env = HashMap::new();
    let credential_context = snapshot_file.restore_credentials(&mut brokered_env, policy);
    brokered_env = credential_context
        .prepare_child_environment(
            &inherited_credential_broker.network_proxy,
            brokered_env,
            /*environment_id*/ None,
        )?
        .env;
    snapshot_file.restore_fail_open_aliases(&mut brokered_env, /*environment_id*/ None);
    for alias in ["HIDDEN_HEADER", "UNKNOWN_HEADER", "STRIPE_HIDDEN_HEADER"] {
        assert_eq!(brokered_env.get(alias), None, "{alias}");
    }
    for secret in [
        "ghp_hidden_alias_secret",
        "ghp_0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefgh",
        "stripe_live_zyxwvutsrqponmlkjihgfedc",
    ] {
        assert!(brokered_env.values().all(|value| !value.contains(secret)));
    }

    // Fail-open children leave managed networking and keep upstream's real
    // aliases, except a copy of a credential the hardened engine still brokers.
    let real_header = "Bearer ghp_0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefgh";
    let real_stripe_header = "Bearer stripe_live_zyxwvutsrqponmlkjihgfedc";
    let mut fail_open_env = HashMap::new();
    let _ = snapshot_file.restore_credentials(&mut fail_open_env, policy);
    snapshot_file.restore_fail_open_aliases(&mut fail_open_env, /*environment_id*/ None);
    assert_eq!(
        fail_open_env.get("UNKNOWN_HEADER").map(String::as_str),
        Some(real_header)
    );
    assert_eq!(fail_open_env.get("HIDDEN_HEADER"), None);
    assert_eq!(
        fail_open_env
            .get("STRIPE_HIDDEN_HEADER")
            .map(String::as_str),
        Some(real_stripe_header)
    );

    std::fs::write(
        &startup,
        "export GH_TOKEN='ghp_hidden_alias_secret'\n\
         credential_function() { printf '%s' 'ghp_hidden_alias_secret'; }\n\
         unset GH_TOKEN\n",
    )?;
    let residual_credential = capture_snapshot(
        &shell,
        &dir.path().abs(),
        &inherited_credential_broker.shell_environment_policy,
        Some(&inherited_credential_broker),
        /*sandbox*/ None,
        /*removed_env_vars*/ &[],
    )
    .await;
    assert!(
        residual_credential.is_err(),
        "snapshot with a credential-bearing shell function must be rejected"
    );

    std::fs::write(
        &startup,
        "unset PATH\nset -u\nexport GH_TOKEN='ghp_unset_path_secret'\n",
    )?;
    let (unset_path_snapshot, _) = capture_snapshot(
        &shell,
        &dir.path().abs(),
        &inherited_credential_broker.shell_environment_policy,
        Some(&inherited_credential_broker),
        /*sandbox*/ None,
        /*removed_env_vars*/ &[],
    )
    .await?;
    assert!(!unset_path_snapshot.contains("declare -x PATH="));
    assert!(!unset_path_snapshot.contains("ghp_unset_path_secret"));

    for real in [
        "ghp_abcdefghijklmnopqrstuvwxyz0123456789ABCDEFGH",
        "opaque_enterprise_credential",
        "0123456789abcdef0123456789abcdef01234567",
    ] {
        std::fs::write(
            &startup,
            format!(
                "unset GH_HOST\nexport GH_ENTERPRISE_TOKEN='{real}'\nexport AUTH_HEADER=\"Bearer $GH_ENTERPRISE_TOKEN\"\n"
            ),
        )?;
        let (snapshot, credentials) = capture_snapshot(
            &shell,
            &dir.path().abs(),
            &inherited_credential_broker.shell_environment_policy,
            Some(&inherited_credential_broker),
            /*sandbox*/ None,
            /*removed_env_vars*/ &[],
        )
        .await?;
        assert!(!snapshot.contains(real));
        let snapshot_file = ShellSnapshotFile {
            shell_environment_policy: ShellEnvironmentPolicy::default(),
            path: dir.path().join("hostless-snapshot.sh").abs(),
            credentials,
        };
        let policy = &inherited_credential_broker.shell_environment_policy;
        let mut restored = HashMap::new();
        let context = snapshot_file.restore_credentials(&mut restored, policy);
        restored = context
            .prepare_child_environment(&network_proxy, restored, /*environment_id*/ None)?
            .env;
        snapshot_file.restore_fail_open_aliases(&mut restored, /*environment_id*/ None);
        assert_eq!(
            restored.get("GH_ENTERPRISE_TOKEN").map(String::as_str),
            Some(real)
        );

        let mut excluded = policy.clone();
        excluded
            .exclude
            .push(EnvironmentVariablePattern::new_case_insensitive(
                "GH_ENTERPRISE_TOKEN",
            ));
        let mut filtered = HashMap::new();
        let _ = snapshot_file.restore_credentials(&mut filtered, &excluded);
        assert!(!filtered.contains_key("GH_ENTERPRISE_TOKEN"));

        let mut overridden = policy.clone();
        overridden
            .r#set
            .insert("GH_ENTERPRISE_TOKEN".to_string(), String::new());
        let mut empty = overridden.r#set.clone();
        let _ = snapshot_file.restore_credentials(&mut empty, &overridden);
        assert_eq!(
            empty.get("GH_ENTERPRISE_TOKEN").map(String::as_str),
            Some("")
        );

        let policies = if real.starts_with("ghp_") {
            vec![excluded, overridden]
        } else {
            vec![policy.clone(), excluded, overridden]
        };
        for policy in policies {
            let broker = SnapshotCredentialBroker {
                network_proxy: network_proxy.clone(),
                shell_environment_policy: policy.clone(),
                allow_login_shell: false,
            };
            let (snapshot, credentials) = capture_snapshot(
                &shell,
                &dir.path().abs(),
                &broker.shell_environment_policy,
                Some(&broker),
                /*sandbox*/ None,
                /*removed_env_vars*/ &[],
            )
            .await?;
            assert!(!snapshot.contains(real));
            let snapshot_file = ShellSnapshotFile {
                shell_environment_policy: ShellEnvironmentPolicy::default(),
                path: dir.path().join("hostless-alias-snapshot.sh").abs(),
                credentials,
            };
            std::fs::write(&snapshot_file.path, snapshot)?;
            let mut env = HashMap::from([("AUTH_HEADER".to_string(), format!("Bearer {real}"))]);
            let context = snapshot_file.restore_credentials(&mut env, &policy);
            env = context
                .prepare_child_environment(&network_proxy, env, /*environment_id*/ None)?
                .env;
            snapshot_file.restore_fail_open_aliases(&mut env, /*environment_id*/ None);
            let replayed = Command::new("/bin/bash")
                .args([
                    "-c",
                    ". \"$1\"; printf '%s' \"${AUTH_HEADER-}\"",
                    "snapshot",
                ])
                .arg(snapshot_file.path.as_path())
                .env_clear()
                .envs(env)
                .output()?;
            assert!(replayed.status.success());
            let expected =
                if policy.exclude.is_empty() && !policy.r#set.contains_key("GH_ENTERPRISE_TOKEN") {
                    format!("Bearer {real}")
                } else {
                    String::new()
                };
            assert_eq!(String::from_utf8(replayed.stdout)?, expected);
        }

        restored.insert("GH_HOST".to_string(), "github.later.example".to_string());
        network_proxy.apply_to_env(&mut restored);
        assert_ne!(
            restored.get("GH_ENTERPRISE_TOKEN").map(String::as_str),
            Some(real)
        );
    }

    Ok(())
}

/// Main's per-shell snapshot scripts filtered secret exports inside the shell.
/// Upstream captures with `snapshot_capture_script` and `capture_snapshot`
/// filters the parsed capture in Rust; these helpers run that capture script
/// and apply the same filter and rendering as the no-broker path.
#[cfg(unix)]
fn capture_script(shell_type: ShellType) -> String {
    snapshot_capture_script(
        shell_type,
        SnapshotCaptureOptions {
            startup: SnapshotStartup::Interactive,
            declarations: true,
            environment: false,
        },
    )
    .expect("shell supports snapshots")
}

#[cfg(unix)]
fn filtered_snapshot(
    shell_type: ShellType,
    output: &std::process::Output,
    removed_env_vars: &[String],
) -> String {
    let mut capture =
        CapturedSnapshot::parse(shell_type, &output.stdout).expect("valid snapshot capture");
    let secret_exports = secret_export_patterns(removed_env_vars);
    capture.retain_exports(|key| !secret_exports.iter().any(|pattern| pattern.matches(key)));
    capture.render_script(&ShellEnvironmentPolicy::default())
}

#[cfg(unix)]
#[test]
fn bash_snapshot_filters_secret_and_invalid_exports() -> Result<()> {
    let output = Command::new("/bin/bash")
        .arg("-c")
        .arg(capture_script(ShellType::Bash))
        .env("BASH_ENV", "/dev/null")
        .env("VALID_NAME", "ok")
        .env("KIMI_API_KEY", "must-not-be-persisted")
        .env("lowercase_api_key", "must-not-be-persisted")
        .env("SERVICE_TOKEN", "must-not-be-persisted")
        .env("AWS_SECRET_ACCESS_KEY", "must-not-be-persisted")
        .env("APP_PASSWORD", "must-not-be-persisted")
        .env("PWD", "/tmp/stale")
        .env("NEXTEST_BIN_EXE_codex-write-config-schema", "/path/to/bin")
        .env("BAD-NAME", "broken")
        .output()?;

    assert!(output.status.success());

    let stdout = filtered_snapshot(ShellType::Bash, &output, /*removed_env_vars*/ &[]);
    assert!(stdout.contains("VALID_NAME"));
    assert!(!stdout.contains("KIMI_API_KEY"));
    assert!(!stdout.contains("lowercase_api_key"));
    assert!(!stdout.contains("SERVICE_TOKEN"));
    assert!(!stdout.contains("AWS_SECRET_ACCESS_KEY"));
    assert!(!stdout.contains("APP_PASSWORD"));
    assert!(!stdout.contains("must-not-be-persisted"));
    assert!(!stdout.contains("PWD=/tmp/stale"));
    assert!(!stdout.contains("NEXTEST_BIN_EXE_codex-write-config-schema"));
    assert!(!stdout.contains("BAD-NAME"));

    Ok(())
}

#[cfg(unix)]
#[test]
fn bash_snapshot_filters_secrets_when_bashrc_shadows_filter_helpers() -> Result<()> {
    let home = tempdir()?;
    std::fs::write(
        home.path().join(".bashrc"),
        "tr() { printf 'BROKEN'; }\nshopt() { return 0; }\n",
    )?;
    let output = Command::new("/bin/bash")
        .arg("-c")
        .arg(capture_script(ShellType::Bash))
        .env_remove("BASH_ENV")
        .env("HOME", home.path())
        .env("PATH", "/usr/bin:/bin")
        .env("lowercase_api_key", "must-not-be-persisted")
        .output()?;

    assert!(output.status.success());

    let stdout = filtered_snapshot(ShellType::Bash, &output, /*removed_env_vars*/ &[]);
    assert!(!stdout.contains("lowercase_api_key"));
    assert!(!stdout.contains("must-not-be-persisted"));

    Ok(())
}

/// Issue #310: a provider credential exported by a startup file under a name
/// the secret patterns miss must not reach the snapshot.
#[cfg(unix)]
#[test]
fn snapshots_drop_provider_credentials_exported_by_startup_files() -> Result<()> {
    let removed = vec!["CORP_LLM_CRED".to_string(), "bad name|*".to_string()];
    let home = tempdir()?;
    let rc = "export CORP_LLM_CRED=must-not-be-persisted\nexport corp_llm_cred=must-not-be-persisted\nexport VALID_NAME=ok\n";
    std::fs::write(home.path().join(".bashrc"), rc)?;
    std::fs::write(home.path().join(".zshrc"), rc)?;

    let mut outputs = vec![
        (
            ShellType::Bash,
            Command::new("/bin/bash")
                .arg("-c")
                .arg(capture_script(ShellType::Bash))
                .env_remove("BASH_ENV")
                .env("HOME", home.path())
                .env("PATH", "/usr/bin:/bin")
                .output()?,
        ),
        // Non-interactive dash ignores `$ENV`, so pass the exports directly.
        (
            ShellType::Sh,
            Command::new("/bin/sh")
                .arg("-c")
                .arg(capture_script(ShellType::Sh))
                .env_remove("ENV")
                .env("PATH", "/usr/bin:/bin")
                .env("CORP_LLM_CRED", "must-not-be-persisted")
                .env("corp_llm_cred", "must-not-be-persisted")
                .env("VALID_NAME", "ok")
                .output()?,
        ),
    ];
    if std::path::Path::new("/bin/zsh").exists() {
        outputs.push((
            ShellType::Zsh,
            Command::new("/bin/zsh")
                .arg("-f")
                .arg("-c")
                .arg(capture_script(ShellType::Zsh))
                .env_clear()
                .env("PATH", "/usr/bin:/bin")
                .env("ZDOTDIR", home.path())
                .output()?,
        ));
    }

    for (shell_type, output) in outputs {
        assert!(output.status.success(), "{output:?}");
        let stdout = filtered_snapshot(shell_type, &output, &removed);
        assert!(stdout.contains("VALID_NAME"), "{stdout}");
        assert!(!stdout.contains("must-not-be-persisted"), "{stdout}");
    }

    Ok(())
}

#[cfg(unix)]
#[tokio::test]
async fn try_create_creates_and_deletes_snapshot_file() -> Result<()> {
    let dir = tempdir()?;
    let shell = Shell {
        shell_type: ShellType::Bash,
        shell_path: PathBuf::from("/bin/bash"),
    };

    let snapshot = ShellSnapshot::try_create(
        &dir.path().abs(),
        ThreadId::new(),
        &dir.path().abs(),
        &ShellEnvironmentPolicy::default(),
        &shell,
        /*state_db*/ None,
        /*credential_broker*/ None,
        /*sandbox*/ None,
        /*removed_env_vars*/ &[],
    )
    .await
    .expect("snapshot should be created");
    let path = snapshot.path.clone();
    assert!(path.exists());

    drop(snapshot);

    assert!(!path.exists());

    Ok(())
}

#[cfg(unix)]
#[tokio::test]
async fn try_create_uses_distinct_generation_paths() -> Result<()> {
    let dir = tempdir()?;
    let session_id = ThreadId::new();
    let shell = Shell {
        shell_type: ShellType::Bash,
        shell_path: PathBuf::from("/bin/bash"),
    };

    let initial_snapshot = ShellSnapshot::try_create(
        &dir.path().abs(),
        session_id,
        &dir.path().abs(),
        &ShellEnvironmentPolicy::default(),
        &shell,
        /*state_db*/ None,
        /*credential_broker*/ None,
        /*sandbox*/ None,
        /*removed_env_vars*/ &[],
    )
    .await
    .expect("initial snapshot should be created");
    let refreshed_snapshot = ShellSnapshot::try_create(
        &dir.path().abs(),
        session_id,
        &dir.path().abs(),
        &ShellEnvironmentPolicy::default(),
        &shell,
        /*state_db*/ None,
        /*credential_broker*/ None,
        /*sandbox*/ None,
        /*removed_env_vars*/ &[],
    )
    .await
    .expect("refreshed snapshot should be created");
    let initial_path = initial_snapshot.path.clone();
    let refreshed_path = refreshed_snapshot.path.clone();
    assert_ne!(initial_path, refreshed_path);
    assert_eq!(initial_path.exists(), true);
    assert_eq!(refreshed_path.exists(), true);

    drop(initial_snapshot);

    assert_eq!(initial_path.exists(), false);
    assert_eq!(refreshed_path.exists(), true);

    drop(refreshed_snapshot);

    assert_eq!(refreshed_path.exists(), false);

    Ok(())
}

#[cfg(unix)]
#[tokio::test]
async fn snapshot_shell_does_not_inherit_stdin() -> Result<()> {
    let _stdin_guard = BlockingStdinPipe::install()?;

    let dir = tempdir()?;
    let home = dir.path().abs();
    let read_status_path = home.join("stdin-read-status");
    let read_status_display = read_status_path.display();
    // Persist the startup `read` exit status so the test can assert whether
    // bash saw EOF on stdin after the snapshot process exits.
    let bashrc = format!("read -t 1 -r ignored\nprintf '%s' \"$?\" > \"{read_status_display}\"\n");
    fs::write(home.join(".bashrc"), bashrc).await?;

    let shell = Shell {
        shell_type: ShellType::Bash,
        shell_path: PathBuf::from("/bin/bash"),
    };

    let home_display = home.display();
    let script = format!(
        "HOME=\"{home_display}\"; export HOME; {}",
        snapshot_capture_script(
            ShellType::Bash,
            SnapshotCaptureOptions {
                startup: SnapshotStartup::Interactive,
                declarations: true,
                environment: false,
            }
        )
        .expect("bash supports snapshots")
    );
    let output = run_script_with_timeout(
        &shell,
        &script,
        Duration::from_secs(2),
        SnapshotShellMode::Login,
        &home,
        /*credential_broker*/ None,
        /*sandbox*/ None,
        /*removed_env_vars*/ &[],
    )
    .await
    .context("run snapshot command")?;
    let read_status = fs::read_to_string(&read_status_path)
        .await
        .context("read stdin probe status")?;

    assert_eq!(
        read_status, "1",
        "expected shell startup read to see EOF on stdin; status={read_status:?}"
    );

    assert!(
        output.contains("# Snapshot file"),
        "expected snapshot marker in output; output={output:?}"
    );

    Ok(())
}

/// D544 (a) / #2562 (main ledger row 13): an armed launch contract refuses only the sandboxed
/// snapshot. (Arming is one-way and process-wide, so the guard's inputs are pinned here.)
#[test]
fn armed_launch_contract_skips_only_sandboxed_snapshots() {
    assert!(!sandboxed_snapshot_allowed(
        /*sandboxed*/ true, /*launch_contract_armed*/ true
    ));
    assert!(sandboxed_snapshot_allowed(
        /*sandboxed*/ true, /*launch_contract_armed*/ false
    ));
    assert!(sandboxed_snapshot_allowed(
        /*sandboxed*/ false, /*launch_contract_armed*/ true
    ));
    assert!(sandboxed_snapshot_allowed(
        /*sandboxed*/ false, /*launch_contract_armed*/ false
    ));
}

/// #310 / #2562 (main ledger row 13): the capture shell never inherits a provider credential
/// variable (`removed_env_vars`), matched case-insensitively, on the plain path and on the
/// sandboxed path every brokered capture takes. The probe is a variable the test runner sets
/// and a shell never sets itself (`CARGO_MANIFEST_DIR` under Cargo, `TEST_TMPDIR` under Bazel).
#[cfg(unix)]
#[tokio::test]
async fn snapshot_shell_does_not_inherit_removed_env_vars() -> Result<()> {
    use crate::tools::sandboxing::SandboxAttempt;
    use codex_exec_server::LOCAL_ENVIRONMENT_ID;
    use codex_protocol::config_types::WindowsSandboxLevel;
    use codex_sandboxing::SandboxManager;
    use codex_sandboxing::SandboxType;

    let (probe, expected) = ["CARGO_MANIFEST_DIR", "TEST_TMPDIR"]
        .into_iter()
        .find_map(|name| std::env::var(name).ok().map(|value| (name, value)))
        .context("the test runner sets CARGO_MANIFEST_DIR or TEST_TMPDIR")?;
    let dir = tempdir()?;
    let cwd = dir.path().abs();
    let shell = Shell {
        shell_type: ShellType::Bash,
        shell_path: PathBuf::from("/bin/bash"),
    };
    let script = format!("printf '%s\\n' \"${{{probe}-removed}}\"");
    let cwd_uri = PathUri::from_abs_path(&cwd);
    let manager = SandboxManager::new();
    let permissions = PermissionProfile::Disabled;
    let attempt = SandboxAttempt {
        sandbox_override: SandboxOverride::NoOverride,
        sandbox: SandboxType::None,
        sandbox_requested: false,
        permissions: &permissions,
        exec_server_permissions: &permissions,
        enforce_managed_network: false,
        manager: &manager,
        sandbox_cwd: &cwd_uri,
        workspace_roots: std::slice::from_ref(&cwd_uri),
        sandbox_exe: None,
        use_legacy_landlock: false,
        windows_sandbox_type: SandboxType::None,
        windows_sandbox_level: WindowsSandboxLevel::Disabled,
        network_denial_cancellation_token: None,
        network_proxy: None,
    };
    let sandbox = ShellSnapshotSandbox::new(
        &attempt,
        /*network*/ None,
        LOCAL_ENVIRONMENT_ID,
        /*additional_permissions*/ None,
    );
    for sandbox in [None, Some(&sandbox)] {
        let run = |removed: Vec<String>| {
            let shell = shell.clone();
            let script = script.clone();
            let cwd = cwd.clone();
            async move {
                run_script_with_timeout(
                    &shell,
                    &script,
                    Duration::from_secs(10),
                    SnapshotShellMode::NonLogin,
                    &cwd,
                    /*credential_broker*/ None,
                    sandbox,
                    &removed,
                )
                .await
            }
        };
        let path = if sandbox.is_some() {
            "sandboxed"
        } else {
            "plain"
        };
        assert_eq!(
            run(Vec::new()).await?.trim_end(),
            expected,
            "{path} control"
        );
        assert_eq!(
            run(vec![probe.to_ascii_lowercase()]).await?.trim_end(),
            "removed",
            "{path}"
        );
    }
    Ok(())
}

/// #310 / #2562 (main ledger row 13): on the brokered path the snapshot file drops a
/// secret-named export and a provider credential name, both matched case-insensitively, even
/// when their values look nothing like credentials (so the value scan can't be what drops them).
#[cfg(unix)]
#[tokio::test]
async fn brokered_snapshot_drops_secret_and_provider_names_case_insensitively() -> Result<()> {
    let dir = tempdir()?;
    let startup = dir.path().join("startup.sh");
    std::fs::write(
        &startup,
        "export lowercase_api_key=plainvalueone\n\
         export corp_llm_cred=plainvaluetwo\n\
         export KEEP_ME_VISIBLE=plainvaluethree\n",
    )?;
    let started_proxy = credential_snapshot_proxy().await?;
    let shell = Shell {
        shell_type: ShellType::Bash,
        shell_path: PathBuf::from("/bin/bash"),
    };
    let shell_environment_policy = ShellEnvironmentPolicy {
        r#set: HashMap::from([("BASH_ENV".to_string(), startup.display().to_string())]),
        ..ShellEnvironmentPolicy::default()
    };
    let credential_broker = SnapshotCredentialBroker {
        network_proxy: started_proxy.proxy(),
        shell_environment_policy: shell_environment_policy.clone(),
        allow_login_shell: false,
    };
    let path = dir.path().join("snapshot.sh").abs();
    write_shell_snapshot(
        &shell,
        &path,
        &dir.path().abs(),
        &shell_environment_policy,
        Some(&credential_broker),
        /*sandbox*/ None,
        &["CORP_LLM_CRED".to_string()],
    )
    .await?;
    let snapshot = fs::read_to_string(&path).await?;
    assert!(
        snapshot.contains("KEEP_ME_VISIBLE"),
        "control: the startup export is captured: {snapshot}"
    );
    for hidden in [
        "lowercase_api_key",
        "corp_llm_cred",
        "plainvalueone",
        "plainvaluetwo",
    ] {
        assert!(!snapshot.contains(hidden), "{hidden} reached the file");
    }
    Ok(())
}

/// #2147 review r2 / #310: a zsh startup file (`.zshenv`, which the brokered
/// non-login capture reads) exports a brokered GitHub token and ties the
/// secret-named array `deploy_token` to `DEPLOY_TOKEN`, a copy of it. Core's
/// secret-name filter drops the export (its value check would too), and the
/// capture still records `DEPLOY_TOKEN` as an alias of the brokered token
/// instead of bailing on it (`invalid_export`): the snapshot is written, the
/// alias reaches neither the file nor a replay of it, and a brokered child that
/// inherited the token and the alias gets neither the alias nor any copy of the
/// token (D640: no placeholders for aliases). A non-secret export in the same
/// file is kept. Runs where zsh is native (macOS).
#[cfg(target_os = "macos")]
#[tokio::test]
async fn zsh_secret_named_tied_credential_alias_is_dropped_without_bailing() -> Result<()> {
    const SECRET: &str = "ghp_0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefgh";
    let dir = tempdir()?;
    std::fs::write(
        dir.path().join(".zshenv"),
        format!(
            "export GITHUB_TOKEN='{SECRET}'\n\
             typeset -T DEPLOY_TOKEN deploy_token\n\
             deploy_token=(prefix \"Bearer $GITHUB_TOKEN\")\n\
             export DEPLOY_TOKEN\n\
             export KEEP_ME_VISIBLE=plainvaluethree\n"
        ),
    )?;
    let started_proxy = credential_snapshot_proxy().await?;
    let shell = Shell {
        shell_type: ShellType::Zsh,
        shell_path: PathBuf::from("/bin/zsh"),
    };
    let shell_environment_policy = ShellEnvironmentPolicy {
        r#set: HashMap::from([("ZDOTDIR".to_string(), dir.path().display().to_string())]),
        ..ShellEnvironmentPolicy::default()
    };
    let credential_broker = SnapshotCredentialBroker {
        network_proxy: started_proxy.proxy(),
        shell_environment_policy: shell_environment_policy.clone(),
        allow_login_shell: false,
    };
    let path = dir.path().join("snapshot.sh").abs();
    let credentials = write_shell_snapshot(
        &shell,
        &path,
        &dir.path().abs(),
        &shell_environment_policy,
        Some(&credential_broker),
        /*sandbox*/ None,
        /*removed_env_vars*/ &[],
    )
    .await?;
    let snapshot = fs::read_to_string(&path).await?;
    assert!(
        snapshot.contains("KEEP_ME_VISIBLE"),
        "control: the startup export is captured: {snapshot}"
    );
    for hidden in ["DEPLOY_TOKEN", "deploy_token", SECRET, "Bearer"] {
        assert!(!snapshot.contains(hidden), "{hidden} reached the file");
    }
    assert!(
        credentials
            .as_ref()
            .is_some_and(|credentials| credentials.fail_open_aliases.contains_key("DEPLOY_TOKEN")),
        "control: the capture saw DEPLOY_TOKEN as an alias of the brokered token"
    );
    let replay = Command::new("/bin/zsh")
        .arg("-f")
        .arg("-c")
        .arg(". \"$1\" && printf '%s|%s' \"${DEPLOY_TOKEN-unset}\" \"${KEEP_ME_VISIBLE-unset}\"")
        .arg("snapshot")
        .arg(path.as_path())
        .env_clear()
        .output()?;
    assert!(replay.status.success(), "{replay:?}");
    assert_eq!(String::from_utf8(replay.stdout)?, "unset|plainvaluethree");

    let snapshot_file = ShellSnapshotFile {
        shell_environment_policy: ShellEnvironmentPolicy::default(),
        path,
        credentials,
    };
    // As if the command inherited the token and its alias.
    let mut brokered_env = HashMap::from([
        ("GITHUB_TOKEN".to_string(), SECRET.to_string()),
        (
            "DEPLOY_TOKEN".to_string(),
            format!("prefix:Bearer {SECRET}"),
        ),
    ]);
    let credential_context =
        snapshot_file.restore_credentials(&mut brokered_env, &shell_environment_policy);
    brokered_env = credential_context
        .prepare_child_environment(
            &credential_broker.network_proxy,
            brokered_env,
            /*environment_id*/ None,
        )?
        .env;
    snapshot_file.restore_fail_open_aliases(&mut brokered_env, /*environment_id*/ None);
    assert_eq!(brokered_env.get("DEPLOY_TOKEN"), None);
    assert!(
        brokered_env.values().all(|value| !value.contains(SECRET)),
        "a brokered child got a copy of the credential: {brokered_env:?}"
    );
    Ok(())
}

#[cfg(target_os = "linux")]
#[tokio::test]
async fn timed_out_snapshot_shell_is_terminated() -> Result<()> {
    use std::process::Stdio;
    use tokio::time::Duration as TokioDuration;
    use tokio::time::Instant;
    use tokio::time::sleep;

    let dir = tempdir()?;
    let pid_path = dir.path().join("pid");
    let script = format!("echo $$ > \"{}\"; sleep 30", pid_path.display());

    let shell = Shell {
        shell_type: ShellType::Sh,
        shell_path: PathBuf::from("/bin/sh"),
    };

    let err = run_script_with_timeout(
        &shell,
        &script,
        Duration::from_secs(1),
        SnapshotShellMode::Login,
        &dir.path().abs(),
        /*credential_broker*/ None,
        /*sandbox*/ None,
        /*removed_env_vars*/ &[],
    )
    .await
    .expect_err("snapshot shell should time out");
    assert!(
        err.to_string().contains("timed out"),
        "expected timeout error, got {err:?}"
    );

    let pid = fs::read_to_string(&pid_path)
        .await
        .expect("snapshot shell writes its pid before timing out")
        .trim()
        .parse::<i32>()?;

    let deadline = Instant::now() + TokioDuration::from_secs(1);
    loop {
        let kill_status = StdCommand::new("kill")
            .arg("-0")
            .arg(pid.to_string())
            .stderr(Stdio::null())
            .stdout(Stdio::null())
            .status()?;
        if !kill_status.success() {
            break;
        }
        if Instant::now() >= deadline {
            panic!("timed out snapshot shell is still alive after grace period");
        }
        sleep(TokioDuration::from_millis(50)).await;
    }

    Ok(())
}

#[cfg(target_os = "macos")]
#[tokio::test]
async fn macos_zsh_snapshot_includes_sections() -> Result<()> {
    let snapshot = get_snapshot(ShellType::Zsh).await?;
    assert_posix_snapshot_sections(&snapshot);
    Ok(())
}

#[cfg(target_os = "linux")]
#[tokio::test]
async fn linux_bash_snapshot_includes_sections() -> Result<()> {
    let snapshot = get_snapshot(ShellType::Bash).await?;
    assert_posix_snapshot_sections(&snapshot);
    Ok(())
}

#[cfg(target_os = "linux")]
#[tokio::test]
async fn linux_sh_snapshot_includes_sections() -> Result<()> {
    let snapshot = get_snapshot(ShellType::Sh).await?;
    assert_posix_snapshot_sections(&snapshot);
    Ok(())
}

#[cfg(target_os = "windows")]
#[ignore]
#[tokio::test]
async fn windows_powershell_snapshot_includes_sections() -> Result<()> {
    let snapshot = get_snapshot(ShellType::PowerShell).await?;
    assert!(snapshot.contains("# Snapshot file"));
    assert!(snapshot.contains("aliases "));
    assert!(snapshot.contains("exports "));
    Ok(())
}

async fn write_rollout_stub(codex_home: &Path, session_id: ThreadId) -> Result<PathBuf> {
    let dir = codex_home
        .join("sessions")
        .join("2025")
        .join("01")
        .join("01");
    fs::create_dir_all(&dir).await?;
    let path = dir.join(format!("rollout-2025-01-01T00-00-00-{session_id}.jsonl"));
    fs::write(&path, "").await?;
    Ok(path)
}

#[tokio::test]
async fn cleanup_stale_snapshots_removes_orphans_and_keeps_live() -> Result<()> {
    let dir = tempdir()?;
    let codex_home = dir.path().abs();
    let snapshot_dir = codex_home.join(SNAPSHOT_DIR);
    fs::create_dir_all(&snapshot_dir).await?;

    let live_session = ThreadId::new();
    let orphan_session = ThreadId::new();
    let live_snapshot = snapshot_dir.join(format!("{live_session}.123.sh"));
    let orphan_snapshot = snapshot_dir.join(format!("{orphan_session}.456.sh"));
    let invalid_snapshot = snapshot_dir.join("not-a-snapshot.txt");

    write_rollout_stub(&codex_home, live_session).await?;
    fs::write(&live_snapshot, "live").await?;
    fs::write(&orphan_snapshot, "orphan").await?;
    fs::write(&invalid_snapshot, "invalid").await?;

    cleanup_stale_snapshots(&codex_home, ThreadId::new(), /*state_db*/ None).await?;

    assert_eq!(live_snapshot.exists(), true);
    assert_eq!(orphan_snapshot.exists(), false);
    assert_eq!(invalid_snapshot.exists(), false);
    Ok(())
}

#[cfg(unix)]
#[tokio::test]
async fn cleanup_stale_snapshots_removes_stale_rollouts() -> Result<()> {
    let dir = tempdir()?;
    let codex_home = dir.path().abs();
    let snapshot_dir = codex_home.join(SNAPSHOT_DIR);
    fs::create_dir_all(&snapshot_dir).await?;

    let stale_session = ThreadId::new();
    let stale_snapshot = snapshot_dir.join(format!("{stale_session}.123.sh"));
    let rollout_path = write_rollout_stub(&codex_home, stale_session).await?;
    fs::write(&stale_snapshot, "stale").await?;

    set_file_mtime(&rollout_path, SNAPSHOT_RETENTION + Duration::from_secs(60))?;

    cleanup_stale_snapshots(&codex_home, ThreadId::new(), /*state_db*/ None).await?;

    assert_eq!(stale_snapshot.exists(), false);
    Ok(())
}

#[cfg(unix)]
#[tokio::test]
async fn cleanup_stale_snapshots_skips_active_session() -> Result<()> {
    let dir = tempdir()?;
    let codex_home = dir.path().abs();
    let snapshot_dir = codex_home.join(SNAPSHOT_DIR);
    fs::create_dir_all(&snapshot_dir).await?;

    let active_session = ThreadId::new();
    let active_snapshot = snapshot_dir.join(format!("{active_session}.123.sh"));
    let rollout_path = write_rollout_stub(&codex_home, active_session).await?;
    fs::write(&active_snapshot, "active").await?;

    set_file_mtime(&rollout_path, SNAPSHOT_RETENTION + Duration::from_secs(60))?;

    cleanup_stale_snapshots(&codex_home, active_session, /*state_db*/ None).await?;

    assert_eq!(active_snapshot.exists(), true);
    Ok(())
}

#[cfg(unix)]
fn set_file_mtime(path: &Path, age: Duration) -> Result<()> {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_secs()
        .saturating_sub(age.as_secs());
    let tv_sec = now
        .try_into()
        .map_err(|_| anyhow!("Snapshot mtime is out of range for libc::timespec"))?;
    let ts = libc::timespec { tv_sec, tv_nsec: 0 };
    let times = [ts, ts];
    let c_path = std::ffi::CString::new(path.as_os_str().as_bytes())?;
    let result = unsafe { libc::utimensat(libc::AT_FDCWD, c_path.as_ptr(), times.as_ptr(), 0) };
    if result != 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    Ok(())
}
#[cfg(unix)]
#[tokio::test]
async fn snapshot_rejects_nul_script_before_execution() -> Result<()> {
    let root = tempdir()?;
    let shell = Shell {
        shell_type: ShellType::Sh,
        shell_path: PathBuf::from("/bin/sh"),
    };
    let error = run_script_with_timeout(
        &shell,
        "printf ran > ran\0",
        Duration::from_secs(5),
        SnapshotShellMode::NonLogin,
        &root.path().abs(),
        /*credential_broker*/ None,
        /*sandbox*/ None,
        /*removed_env_vars*/ &[],
    )
    .await
    .expect_err("invalid snapshot script must fail to spawn");
    assert_eq!(
        error
            .downcast_ref::<std::io::Error>()
            .map(std::io::Error::kind),
        Some(std::io::ErrorKind::InvalidInput)
    );
    assert!(!root.path().join("ran").exists());
    Ok(())
}
