use super::*;
use pretty_assertions::assert_eq;

/// #1976: only Moderate and Aggressive on Windows with no Windows sandbox;
/// the non-admin and elevated sandboxes and Permissive are unchanged.
#[test]
fn issue_1976_refuses_only_protected_levels_on_windows_without_a_sandbox() {
    let mut refused = Vec::new();
    for windows in [false, true] {
        for level in [
            SecurityLevel::Permissive,
            SecurityLevel::Moderate,
            SecurityLevel::Aggressive,
        ] {
            for sandbox in [
                WindowsSandboxLevel::Disabled,
                WindowsSandboxLevel::RestrictedToken,
                WindowsSandboxLevel::Elevated,
            ] {
                if applies_on(windows, level, sandbox) {
                    refused.push((level, sandbox));
                }
            }
        }
    }
    assert_eq!(
        refused,
        vec![
            (SecurityLevel::Moderate, WindowsSandboxLevel::Disabled),
            (SecurityLevel::Aggressive, WindowsSandboxLevel::Disabled),
        ]
    );
    assert_eq!(
        applies(SecurityLevel::Moderate, WindowsSandboxLevel::Disabled),
        cfg!(windows)
    );
}

/// #1976: each refusal names the level and the setup steps.
#[test]
fn issue_1976_refusals_name_the_level_and_the_setup_steps() {
    for (level, name) in [
        (SecurityLevel::Moderate, "Moderate"),
        (SecurityLevel::Aggressive, "Aggressive"),
    ] {
        for (text, start) in [
            (command_refusal(level), "refused this command"),
            (file_refusal(level), "refused this file access"),
            (startup_notice(level), "needs the default Windows sandbox"),
        ] {
            assert!(
                text.starts_with(&format!("Security level {name} {start}")),
                "{text}"
            );
            for part in [
                "Set up default sandbox",
                "Administrator",
                "this session uses it from the next message",
                "`[windows]`",
                "switch to Permissive in /security",
            ] {
                assert!(text.contains(part), "{part}: {text}");
            }
        }
    }
}

/// #2000: once the session's Windows sandbox is set up, typing into a
/// command started before it (so without it) stays refused under Moderate
/// and Aggressive; commands started inside a sandbox, Permissive and other
/// platforms are unchanged.
#[test]
fn issue_2000_typing_into_a_command_started_without_the_sandbox_stays_refused() {
    use WindowsSandboxLevel::Disabled;
    use WindowsSandboxLevel::Elevated;
    use WindowsSandboxLevel::RestrictedToken;
    for level in [SecurityLevel::Moderate, SecurityLevel::Aggressive] {
        let name = name(level);
        // Still no sandbox: the #1976 refusal, whatever the process.
        let refusal = typing_refusal_on(/*windows*/ true, level, Disabled, Some(Disabled))
            .expect("refused without a sandbox");
        assert!(refusal.starts_with(&format!("Security level {name} refused this command")));
        for now in [RestrictedToken, Elevated] {
            let refusal = typing_refusal_on(/*windows*/ true, level, now, Some(Disabled))
                .expect("a command started without the sandbox stays refused");
            assert!(
                refusal.starts_with(&format!(
                    "Security level {name} refused typing into this command: it started \
                     before the Windows sandbox was set up"
                )),
                "{refusal}"
            );
            for started in [Some(RestrictedToken), Some(Elevated), None] {
                assert_eq!(
                    typing_refusal_on(/*windows*/ true, level, now, started),
                    None,
                    "{now:?} {started:?}"
                );
            }
            assert_eq!(
                typing_refusal_on(/*windows*/ false, level, now, Some(Disabled)),
                None
            );
        }
    }
    assert_eq!(
        typing_refusal_on(
            /*windows*/ true,
            SecurityLevel::Permissive,
            Elevated,
            Some(Disabled)
        ),
        None
    );
}

/// #2050: a fake executor answering `environment/info` with `info` (or
/// closing at once when `None`, like one that does not answer).
async fn fake_executor(info: Option<serde_json::Value>) -> String {
    use futures::SinkExt;
    use futures::StreamExt;
    use tokio_tungstenite::tungstenite::Message;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let url = format!("ws://{}", listener.local_addr().expect("addr"));
    tokio::spawn(async move {
        while let Ok((stream, _)) = listener.accept().await {
            let Ok(mut websocket) = tokio_tungstenite::accept_async(stream).await else {
                continue;
            };
            let Some(info) = info.clone() else {
                continue;
            };
            while let Some(Ok(Message::Text(text))) = websocket.next().await {
                let request: serde_json::Value = serde_json::from_str(text.as_ref()).expect("json");
                let Some(id) = request.get("id") else {
                    continue;
                };
                let result = match request["method"].as_str() {
                    Some("initialize") => serde_json::json!({ "sessionId": "fake" }),
                    _ => info.clone(),
                };
                let reply = serde_json::json!({ "id": id, "result": result }).to_string();
                if websocket.send(Message::Text(reply.into())).await.is_err() {
                    break;
                }
            }
        }
    });
    url
}

fn turn_environment(url: Option<String>) -> crate::session::turn_context::TurnEnvironment {
    let environment = codex_exec_server::Environment::create_for_tests(url).expect("environment");
    crate::session::turn_context::TurnEnvironment::for_tests(
        "far".to_string(),
        std::sync::Arc::new(environment),
        codex_utils_path_uri::PathUri::parse("file:///C:/work").expect("cwd"),
        Vec::new(),
        /*shell*/ None,
    )
}

#[tokio::test]
async fn issue_2088_discovery_and_skill_reads_keep_the_refusal_with_full_disk_access() {
    use crate::environment_selection::TurnEnvironmentSnapshot;
    use crate::environment_selection::TurnEnvironmentState;
    use crate::session::tests::build_test_config;
    use crate::session::tests::make_session_and_context_for_config;
    use codex_features::Feature;
    use codex_protocol::models::PermissionProfile;

    for level in [
        SecurityLevel::Permissive,
        SecurityLevel::Moderate,
        SecurityLevel::Aggressive,
    ] {
        for discovery_enabled in [false, true] {
            let home = tempfile::tempdir().expect("home");
            let cwd = tempfile::tempdir().expect("cwd");
            let mut config = build_test_config(home.path(), cwd.path()).await;
            config.security_level = level;
            config
                .permissions
                .set_permission_profile(PermissionProfile::Disabled)
                .expect("profile");
            if discovery_enabled {
                config
                    .features
                    .enable(Feature::ExecutorCapabilityDiscovery)
                    .expect("feature");
            } else {
                config
                    .features
                    .disable(Feature::ExecutorCapabilityDiscovery)
                    .expect("feature");
            }
            let (session, turn) = make_session_and_context_for_config(config).await;
            session.services.replace_model_client(
                (*session.services.model_client())
                    .clone()
                    .with_ingress_level(level),
            );
            // No root is scanned: inspect the exact context forwarded to discovery and skills.read.
            let environments = TurnEnvironmentSnapshot {
                environments: vec![
                    TurnEnvironmentState::Ready(turn_environment(Some(
                        "ws://127.0.0.1:1".to_string(),
                    ))),
                    TurnEnvironmentState::Ready(
                        turn.initial_environments.primary().expect("local").clone(),
                    ),
                ],
            };
            let (snapshot, _) = session
                .executor_capability_discovery_for_step(
                    &turn.config,
                    /*resolved_roots*/ &[],
                    &environments,
                    /*v2_snapshots*/ None,
                )
                .await;
            if level == SecurityLevel::Permissive {
                assert_eq!(snapshot.is_some(), discovery_enabled);
                if let Some(snapshot) = snapshot {
                    assert!(snapshot.sandbox_contexts().is_empty());
                }
            } else {
                let snapshot =
                    snapshot.expect("protected levels must not use unsandboxed discovery");
                let context = snapshot
                    .sandbox_contexts()
                    .get("far")
                    .expect("skill resource context");
                assert_eq!(
                    context.unsandboxed_refusal,
                    Some(if cfg!(windows) {
                        file_refusal(level)
                    } else {
                        remote_file_refusal(level, "far")
                    })
                );
                // Upstream gives a remote Windows executor with no configured sandbox the
                // restricted token; with a refusal the context keeps main's `Disabled`, so
                // the executor's report is judged for the sandbox configured (#2729).
                assert_eq!(
                    context.windows_sandbox_selection,
                    codex_file_system::WindowsSandboxSelection::Disabled
                );
                let local = snapshot
                    .sandbox_contexts()
                    .get("local")
                    .expect("local context");
                assert_eq!(
                    local.unsandboxed_refusal,
                    cfg!(windows).then(|| file_refusal(level))
                );
            }
        }
    }
}

fn windows_info(set_up: bool) -> serde_json::Value {
    serde_json::json!({
        "shell": { "name": "powershell", "path": "powershell.exe" },
        "cwd": "file:///C:/work",
        "hostSandbox": { "os": "windows", "windowsSandboxSetUp": set_up },
    })
}

/// #2050 (D510): a remote environment is judged by its executor's report,
/// whatever this host is: Windows without a sandbox for the session's
/// `[windows] sandbox`, and an executor that cannot say, are gaps; a sandbox
/// there, a Linux executor and a local environment are not.
#[tokio::test]
async fn issue_2050_a_remote_gap_follows_the_executor_report() {
    use WindowsSandboxLevel::Disabled;
    use WindowsSandboxLevel::Elevated;
    use WindowsSandboxLevel::RestrictedToken;
    let cases = [
        (
            Some(windows_info(/*set_up*/ false)),
            Disabled,
            Some(RemoteGap::WindowsSandboxNotSelected),
        ),
        (
            Some(windows_info(/*set_up*/ true)),
            Disabled,
            Some(RemoteGap::WindowsSandboxNotSelected),
        ),
        (
            Some(windows_info(/*set_up*/ false)),
            Elevated,
            Some(RemoteGap::WindowsSandboxNotSetUp),
        ),
        (Some(windows_info(/*set_up*/ true)), Elevated, None),
        (Some(windows_info(/*set_up*/ false)), RestrictedToken, None),
        (
            Some(serde_json::json!({
                "shell": { "name": "bash", "path": "/bin/bash" },
                "hostSandbox": { "os": "linux" },
            })),
            Disabled,
            None,
        ),
        // An older Windows executor, with no report.
        (
            Some(serde_json::json!({
                "shell": { "name": "powershell", "path": "powershell.exe" },
                "cwd": "file:///C:/work",
            })),
            Elevated,
            Some(RemoteGap::Unreported),
        ),
        // One that does not answer.
        (None, Elevated, Some(RemoteGap::Unreported)),
    ];
    for (info, level, gap) in cases {
        let environment = turn_environment(Some(fake_executor(info.clone()).await));
        assert_eq!(
            remote_gap(&environment.environment, level).await,
            gap,
            "{info:?} {level:?}"
        );
    }
    let local = turn_environment(/*url*/ None);
    assert_eq!(remote_gap(&local.environment, Disabled).await, None);
}

/// #2050: the remote refusals name the level, the environment and the steps
/// for the remote machine; file tools on a remote environment carry the
/// refusal under every protected level (checked against the report when
/// they run), and local environments keep #1976's rule.
#[test]
fn issue_2050_remote_refusals_name_the_level_environment_and_steps() {
    for (level, name) in [
        (SecurityLevel::Moderate, "Moderate"),
        (SecurityLevel::Aggressive, "Aggressive"),
    ] {
        for (text, start) in [
            (
                remote_command_refusal(level, "far", RemoteGap::WindowsSandboxNotSetUp),
                "refused this command",
            ),
            (
                remote_command_refusal(level, "far", RemoteGap::WindowsSandboxNotSelected),
                "refused this command",
            ),
            (
                remote_typing_refusal(level, "far", RemoteGap::WindowsSandboxNotSetUp),
                "refused typing into this command",
            ),
            (
                remote_typing_refusal(level, "far", RemoteGap::WindowsSandboxNotSelected),
                "refused typing into this command",
            ),
            (
                remote_file_refusal(level, "far"),
                "refused this file access",
            ),
        ] {
            assert!(
                text.starts_with(&format!("Security level {name} {start}")),
                "{text}"
            );
            for part in [
                "machine of environment `far`",
                "on that machine",
                "Set up default sandbox",
                "Administrator",
                "`[windows]` in config.toml on this computer",
                "/new",
                "switch to Permissive in /security",
            ] {
                assert!(text.contains(part), "{part}: {text}");
            }
        }
        let not_selected =
            remote_command_refusal(level, "far", RemoteGap::WindowsSandboxNotSelected);
        assert!(
            not_selected.contains("asks the remote Windows machine of environment `far` for none"),
            "{not_selected}"
        );
        let unreported = remote_command_refusal(level, "far", RemoteGap::Unreported);
        for part in [
            "environment `far` did not report one",
            "older than this version",
            "Update Corbanu Terminal on that machine",
            "switch to Permissive in /security",
        ] {
            assert!(unreported.contains(part), "{part}: {unreported}");
        }
    }
}

#[tokio::test]
async fn issue_2050_file_refusal_follows_where_the_tool_runs() {
    let remote = turn_environment(Some(fake_executor(/*info*/ None).await));
    let local = turn_environment(/*url*/ None);
    for level in [SecurityLevel::Moderate, SecurityLevel::Aggressive] {
        for sandbox in [
            WindowsSandboxLevel::Disabled,
            WindowsSandboxLevel::RestrictedToken,
            WindowsSandboxLevel::Elevated,
        ] {
            // A Windows client without its own sandbox keeps #1976's
            // refusal for every environment (#2049).
            assert_eq!(
                environment_file_refusal(level, sandbox, &remote),
                Some(if applies(level, sandbox) {
                    file_refusal(level)
                } else {
                    remote_file_refusal(level, "far")
                })
            );
            assert_eq!(
                environment_file_refusal(level, sandbox, &local),
                applies(level, sandbox).then(|| file_refusal(level))
            );
        }
    }
    for environment in [&remote, &local] {
        assert_eq!(
            environment_file_refusal(
                SecurityLevel::Permissive,
                WindowsSandboxLevel::Disabled,
                environment
            ),
            None
        );
    }
}

/// #2562 (D491, D522 (a)): commands and capability discovery are judged on
/// the weaker of the environment's sandbox and the session's: an
/// environment configured with the elevated sandbox in a session without one,
/// or one launching with MXC, counts as unsandboxed, so protected levels refuse.
#[test]
fn issue_2562_environment_sandbox_never_outranks_the_session() {
    use crate::config::PermissionProfileSnapshot;
    use crate::tools::orchestrator::unsandboxed_check_level;
    use WindowsSandboxLevel::Disabled;
    use WindowsSandboxLevel::Elevated;
    use WindowsSandboxLevel::RestrictedToken;
    use codex_protocol::models::PermissionProfile;
    use codex_protocol::protocol::EnvironmentConfig;
    use codex_sandboxing::SandboxType;

    let environment = |level, sandbox_type| EnvironmentConfig {
        environment_hooks: Default::default(),
        allow_login_shell: true,
        workspace_roots: Vec::new(),
        windows_sandbox_level: level,
        windows_sandbox_type: sandbox_type,
        use_legacy_landlock: false,
        permission_profile: PermissionProfileSnapshot::legacy(PermissionProfile::read_only()),
        shell_environment_policy: Default::default(),
        exec_policy: None,
        mcp_policy: None,
        network_policy: None,
        selected_capability_roots: Vec::new(),
    };
    for (environment_level, sandbox_type, session_level, expected) in [
        // The environment's elevated sandbox does not stand in for the session's.
        (
            Elevated,
            SandboxType::WindowsRestrictedToken,
            Disabled,
            Disabled,
        ),
        (
            RestrictedToken,
            SandboxType::WindowsRestrictedToken,
            Disabled,
            Disabled,
        ),
        // MXC counts as no sandbox however the session is set up.
        (Elevated, SandboxType::WindowsMxc, Elevated, Disabled),
        (Disabled, SandboxType::WindowsMxc, RestrictedToken, Disabled),
        // Otherwise the environment's own sandbox is the one judged.
        (
            Elevated,
            SandboxType::WindowsRestrictedToken,
            Elevated,
            Elevated,
        ),
        (Disabled, SandboxType::None, Elevated, Disabled),
    ] {
        let check_level =
            unsandboxed_check_level(&environment(environment_level, sandbox_type), session_level);
        assert_eq!(
            check_level, expected,
            "{environment_level:?} {sandbox_type:?} in a {session_level:?} session"
        );
        // On a Windows client the protected levels then refuse (#1976).
        assert_eq!(
            applies_on(/*windows*/ true, SecurityLevel::Moderate, check_level),
            expected == Disabled
        );
    }
}

/// #2562 (D491/D510, D522 (a)): V2 capability discovery, like V1, runs
/// nothing under a protected level on an executor that cannot confine it
/// (Windows without the sandbox for the session's level, or no report), and
/// still runs on one that can, and under Permissive.
#[tokio::test]
async fn issue_2562_v2_discovery_skips_unconfined_executors_under_protected_levels() {
    use crate::environment_selection::TurnEnvironmentSnapshot;
    use crate::environment_selection::TurnEnvironmentState;
    use crate::session::tests::build_test_config;
    use crate::session::tests::make_session_and_context_for_config;
    use codex_features::Feature;

    let v2 = |mut info: serde_json::Value| {
        info["executorVersion"] = serde_json::json!("0.167.0");
        info["capabilities"] = serde_json::json!({ "capabilityDiscoveryV2": true });
        info
    };
    let unreported = serde_json::json!({
        "shell": { "name": "powershell", "path": "powershell.exe" },
        "cwd": "file:///C:/work",
    });
    let linux = serde_json::json!({
        "shell": { "name": "bash", "path": "/bin/bash" },
        "hostSandbox": { "os": "linux" },
    });
    use WindowsSandboxLevel::Disabled;
    use WindowsSandboxLevel::Elevated;
    use codex_sandboxing::SandboxType;
    // (executor report, the environment's own sandbox, level, discovered)
    for (info, environment_level, level, discovered) in [
        // The environment's elevated sandbox, set up on the executor, does not
        // stand in for the session's missing one.
        (
            v2(windows_info(/*set_up*/ true)),
            Elevated,
            SecurityLevel::Moderate,
            false,
        ),
        (
            v2(windows_info(/*set_up*/ false)),
            Disabled,
            SecurityLevel::Moderate,
            false,
        ),
        (
            v2(windows_info(/*set_up*/ true)),
            Disabled,
            SecurityLevel::Aggressive,
            false,
        ),
        (
            v2(unreported.clone()),
            Disabled,
            SecurityLevel::Moderate,
            false,
        ),
        (v2(linux), Disabled, SecurityLevel::Aggressive, true),
        (
            v2(windows_info(/*set_up*/ false)),
            Disabled,
            SecurityLevel::Permissive,
            true,
        ),
        (v2(unreported), Disabled, SecurityLevel::Permissive, true),
    ] {
        let home = tempfile::tempdir().expect("home");
        let cwd = tempfile::tempdir().expect("cwd");
        let mut config = build_test_config(home.path(), cwd.path()).await;
        config.security_level = level;
        config
            .features
            .enable(Feature::ExecutorCapabilityDiscoveryV2)
            .expect("feature");
        let (session, turn) = make_session_and_context_for_config(config).await;
        session.services.replace_model_client(
            (*session.services.model_client())
                .clone()
                .with_ingress_level(level),
        );
        let environments = TurnEnvironmentSnapshot {
            environments: vec![TurnEnvironmentState::Ready(configured_environment(
                Some(fake_executor(Some(info.clone())).await),
                environment_level,
                if environment_level == Disabled {
                    SandboxType::None
                } else {
                    SandboxType::WindowsRestrictedToken
                },
            ))],
        };
        let (snapshots, _) = session
            .executor_capability_discovery_for_step_v2(&turn.config, &environments)
            .await;
        let attempted = snapshots
            .unwrap_or_default()
            .iter()
            .any(|snapshot| snapshot.environment_id == "far");
        assert_eq!(
            attempted, discovered,
            "{environment_level:?} environment at {level:?}: {info}"
        );
    }
}

/// An environment whose own config carries `level` and `sandbox_type`
/// (`turn_environment` hardcodes `Disabled`); remote when `url` is set.
fn configured_environment(
    url: Option<String>,
    level: WindowsSandboxLevel,
    sandbox_type: codex_sandboxing::SandboxType,
) -> crate::session::turn_context::TurnEnvironment {
    use crate::config::PermissionProfileSnapshot;
    use crate::environment_selection::EnvironmentConfigOrigin;
    use codex_protocol::models::PermissionProfile;
    use codex_protocol::protocol::EnvironmentConfig;
    use codex_protocol::protocol::EnvironmentConfigState;
    use codex_protocol::protocol::TurnEnvironmentSelection;

    crate::session::turn_context::TurnEnvironment::new(
        TurnEnvironmentSelection {
            selected_capability_roots: Default::default(),
            environment_id: "far".to_string(),
            cwd: codex_utils_path_uri::PathUri::parse("file:///C:/work").expect("cwd"),
            workspace_roots: Vec::new(),
            config: EnvironmentConfigState::Ready(EnvironmentConfig {
                environment_hooks: Default::default(),
                allow_login_shell: true,
                workspace_roots: Vec::new(),
                windows_sandbox_level: level,
                windows_sandbox_type: sandbox_type,
                use_legacy_landlock: false,
                permission_profile: PermissionProfileSnapshot::legacy(
                    PermissionProfile::read_only(),
                ),
                shell_environment_policy: Default::default(),
                exec_policy: None,
                mcp_policy: None,
                network_policy: None,
                selected_capability_roots: Vec::new(),
            }),
        },
        EnvironmentConfigOrigin::Thread,
        std::sync::Arc::new(
            codex_exec_server::Environment::create_for_tests(url).expect("environment"),
        ),
        /*shell*/ None,
    )
}

/// #2562 (main-merges-2 ledger row 614): V1 discovery in a session without a
/// Windows sandbox refuses an environment configured with the elevated one
/// under a protected level (on Windows; elsewhere a local environment has no
/// refusal). Unlike `turn_environment`, this environment carries its level.
#[tokio::test]
async fn issue_2562_discovery_judges_an_elevated_environment_in_an_unsandboxed_session() {
    use crate::environment_selection::TurnEnvironmentSnapshot;
    use crate::environment_selection::TurnEnvironmentState;
    use crate::session::tests::build_test_config;
    use crate::session::tests::make_session_and_context_for_config;
    use codex_sandboxing::SandboxType;

    for level in [SecurityLevel::Moderate, SecurityLevel::Aggressive] {
        let home = tempfile::tempdir().expect("home");
        let cwd = tempfile::tempdir().expect("cwd");
        let mut config = build_test_config(home.path(), cwd.path()).await;
        config.security_level = level;
        // The test config selects no Windows sandbox for the session.
        let (session, turn) = make_session_and_context_for_config(config).await;
        session.services.replace_model_client(
            (*session.services.model_client())
                .clone()
                .with_ingress_level(level),
        );
        let elevated = configured_environment(
            /*url*/ None,
            WindowsSandboxLevel::Elevated,
            SandboxType::WindowsRestrictedToken,
        );
        let environments = TurnEnvironmentSnapshot {
            environments: vec![TurnEnvironmentState::Ready(elevated)],
        };
        let (snapshot, _) = session
            .executor_capability_discovery_for_step(
                &turn.config,
                /*resolved_roots*/ &[],
                &environments,
                /*v2_snapshots*/ None,
            )
            .await;
        let snapshot = snapshot.expect("protected levels keep a discovery context");
        let context = snapshot
            .sandbox_contexts()
            .get("far")
            .expect("environment context");
        assert_eq!(
            context.unsandboxed_refusal,
            cfg!(windows).then(|| file_refusal(level)),
            "{level:?}"
        );
    }
}

/// #2729: a context carrying a refusal keeps main's `Disabled` selection
/// where upstream raised it to the restricted token; nothing else changes.
#[tokio::test]
async fn issue_2729_a_refusal_keeps_the_configured_windows_selection() {
    use codex_file_system::FileSystemSandboxContext;
    use codex_file_system::WindowsSandboxSelection;
    use codex_protocol::models::PermissionProfile;

    for (level, sandbox_type, refusal, raised, kept) in [
        (
            WindowsSandboxLevel::Disabled,
            codex_sandboxing::SandboxType::WindowsRestrictedToken,
            true,
            WindowsSandboxSelection::RestrictedToken,
            WindowsSandboxSelection::Disabled,
        ),
        (
            WindowsSandboxLevel::Disabled,
            codex_sandboxing::SandboxType::WindowsRestrictedToken,
            false,
            WindowsSandboxSelection::RestrictedToken,
            WindowsSandboxSelection::RestrictedToken,
        ),
        (
            WindowsSandboxLevel::RestrictedToken,
            codex_sandboxing::SandboxType::WindowsRestrictedToken,
            true,
            WindowsSandboxSelection::RestrictedToken,
            WindowsSandboxSelection::RestrictedToken,
        ),
        (
            WindowsSandboxLevel::Elevated,
            codex_sandboxing::SandboxType::WindowsRestrictedToken,
            true,
            WindowsSandboxSelection::Elevated,
            WindowsSandboxSelection::Elevated,
        ),
        (
            WindowsSandboxLevel::Disabled,
            codex_sandboxing::SandboxType::WindowsMxc,
            true,
            WindowsSandboxSelection::Mxc,
            WindowsSandboxSelection::Mxc,
        ),
    ] {
        let environment = configured_environment(/*url*/ None, level, sandbox_type);
        let mut context = FileSystemSandboxContext::from_permission_profile(
            PermissionProfile::read_only(),
            codex_utils_path_uri::PathUri::parse("file:///C:/work").expect("cwd"),
        );
        context.windows_sandbox_selection = raised;
        context.unsandboxed_refusal = refusal.then(|| "refused".to_string());
        keep_configured_selection(&mut context, &environment);
        assert_eq!(
            context.windows_sandbox_selection, kept,
            "{level:?} {sandbox_type:?} refusal={refusal}"
        );
    }
}
