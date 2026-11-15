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
    crate::session::turn_context::TurnEnvironment::new(
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
                        turn.environments.primary().expect("local").clone(),
                    ),
                ],
            };
            let snapshot = session
                .executor_capability_discovery_for_step(
                    &turn.config,
                    &[],
                    &environments,
                    WindowsSandboxLevel::Disabled,
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
                assert_eq!(context.windows_sandbox_level, WindowsSandboxLevel::Disabled);
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
