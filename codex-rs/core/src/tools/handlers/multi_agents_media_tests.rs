//! #2222: model-selected attachments cannot bypass the file-read gate.
use super::*;
use codex_security_policy::SecurityLevel;
use pretty_assertions::assert_eq;
use test_case::test_case;

#[cfg(windows)]
fn stage_windows_sandbox_helpers() {
    let exe = std::env::current_exe().expect("test binary");
    let resources = exe.parent().expect("test dir").join("codex-resources");
    std::fs::create_dir_all(&resources).expect("resources");
    for helper in ["codex-windows-sandbox-setup", "codex-command-runner"] {
        let source = codex_utils_cargo_bin::cargo_bin(helper).expect("sandbox helper");
        let destination = resources.join(format!("{helper}.exe"));
        if let Err(error) = std::fs::copy(&source, &destination)
            && !destination.exists()
        {
            panic!("unable to stage {helper}: {error}");
        }
    }
}

#[test_case(SecurityLevel::Moderate, "local_image"; "moderate_image")]
#[test_case(SecurityLevel::Aggressive, "local_image"; "aggressive_image")]
#[test_case(SecurityLevel::Permissive, "local_image"; "permissive_image")]
#[test_case(SecurityLevel::Moderate, "local_audio"; "moderate_audio")]
#[test_case(SecurityLevel::Aggressive, "local_audio"; "aggressive_audio")]
#[test_case(SecurityLevel::Permissive, "local_audio"; "permissive_audio")]
#[tokio::test]
async fn issue_2222_agent_attachments(level: SecurityLevel, kind: &str) {
    let (mut session, turn) = make_session_and_context().await;
    let mut turn = with_spawn_eligible_runtime(&session, turn).await;
    turn.approval_policy
        .set(AskForApproval::Never)
        .expect("approvals off");
    let manager = install_live_root(&mut session, &turn).await;
    let child = start_own_child(&manager, (*turn.config).clone(), session.thread_id).await;
    let client = (*session.services.model_client())
        .clone()
        .with_ingress_level(level)
        .with_source_envelopes(/*enabled*/ true);
    client.note_recorded_for_taint(&[ResponseItem::FunctionCallOutput {
        id: None,
        call_id: "notes".into(),
        output: codex_protocol::models::FunctionCallOutputPayload::from_text(
            "untrusted notes".into(),
        ),
        internal_chat_message_metadata_passthrough: None,
    }]);
    session.services.replace_model_client(client);
    let session = Arc::new(session);
    let turn = Arc::new(turn);
    let fixture = tempfile::tempdir().expect("fixture");
    let path = fixture.path().join(".env");
    std::fs::write(&path, b"synthetic attachment").expect("fixture bytes");
    let items = json!([{"type": kind, "path": path}]);
    for name in ["spawn_agent", "send_input"] {
        let mut args = json!({"items": items});
        if name == "send_input" {
            args["target"] = json!(child.thread_id.to_string());
        }
        let call = invocation(session.clone(), turn.clone(), name, function_payload(args));
        let result = if name == "spawn_agent" {
            SpawnAgentHandler::default().handle(call).await
        } else {
            SendInputHandler.handle(call).await
        };
        if level == SecurityLevel::Permissive {
            result.expect("Permissive preserves attachment delivery");
        } else {
            let Err(error) = result else {
                panic!("{name} admitted {kind} at {level:?}");
            };
            let text = model_facing_error_message(&error).expect("model error");
            assert!(
                text.contains("credential access") && text.contains("approvals are off"),
                "{text}"
            );
        }
    }
    let delivered = manager
        .captured_ops()
        .into_iter()
        .filter_map(|(id, op)| {
            if id != child.thread_id {
                return None;
            }
            match op {
                Op::UserInput { items, .. } => Some(items),
                _ => None,
            }
        })
        .collect::<Vec<_>>();
    let expected = if level == SecurityLevel::Permissive {
        vec![serde_json::from_value::<Vec<UserInput>>(items).expect("items")]
    } else {
        Vec::new()
    };
    assert_eq!(delivered, expected);
}

#[test_case(SecurityLevel::Moderate, "local_image", false; "moderate_image_bytes")]
#[test_case(SecurityLevel::Moderate, "local_audio", false; "moderate_audio_bytes")]
#[test_case(SecurityLevel::Aggressive, "local_image", true; "aggressive_image_denied_before_taint")]
#[test_case(SecurityLevel::Aggressive, "local_audio", true; "aggressive_audio_denied_before_taint")]
#[tokio::test]
#[cfg_attr(windows, serial_test::serial(codex_home))]
async fn issue_2222_agent_attachments_are_read_before_delivery(
    level: SecurityLevel,
    kind: &str,
    denied: bool,
) {
    let home = tempfile::tempdir().expect("home");
    let cwd = tempfile::tempdir().expect("cwd");
    #[allow(unused_mut)]
    let mut config = crate::session::tests::build_test_config(home.path(), cwd.path()).await;
    #[cfg(windows)]
    {
        stage_windows_sandbox_helpers();
        config.permissions.windows_sandbox_mode =
            Some(codex_config::types::WindowsSandboxModeToml::Elevated);
    }
    let (mut session, mut turn) =
        crate::session::tests::make_session_and_context_for_config(config).await;
    #[cfg(target_os = "linux")]
    let linux_sandbox =
        Some(core_test_support::find_codex_linux_sandbox_exe().expect("Linux sandbox helper"));
    #[cfg(not(target_os = "linux"))]
    let linux_sandbox = None;
    let runtime = codex_exec_server::ExecServerRuntimePaths::new(
        std::env::current_exe().expect("test executable"),
        linux_sandbox,
    )
    .expect("runtime paths");
    let environments =
        codex_exec_server::EnvironmentManager::create_for_tests_with_local(None, runtime).await;
    let current = turn
        .environments
        .turn_environments()
        .next()
        .cloned()
        .expect("local environment");
    let environment_cwd = current.cwd().clone();
    turn.environments.environments[0] = crate::environment_selection::TurnEnvironmentState::Ready(
        crate::session::turn_context::TurnEnvironment::new(
            current.environment_id,
            environments
                .try_local_environment()
                .expect("runtime environment"),
            environment_cwd,
            Vec::new(),
            current.shell,
        ),
    );
    let manager = install_live_root(&mut session, &turn).await;
    let child = start_own_child(&manager, (*turn.config).clone(), session.thread_id).await;
    let client = (*session.services.model_client())
        .clone()
        .with_ingress_level(level)
        .with_source_envelopes(/*enabled*/ true);
    session.services.replace_model_client(client);
    let fixture = tempfile::tempdir().expect("fixture");
    let path = if denied {
        turn.config
            .codex_home
            .join("secrets")
            .join("media.wav")
            .to_path_buf()
    } else {
        fixture.path().join("media.wav")
    };
    std::fs::create_dir_all(path.parent().expect("parent")).expect("fixture parent");
    std::fs::write(&path, b"original bytes").expect("fixture");
    let result = SendInputHandler
        .handle(invocation(
            Arc::new(session),
            Arc::new(turn),
            "send_input",
            function_payload(json!({
                "target": child.thread_id.to_string(),
                "items": [{"type": kind, "path": path}],
            })),
        ))
        .await;
    if denied {
        let Err(error) = result else {
            panic!("Aggressive must deny protected attachment");
        };
        assert!(
            model_facing_error_message(&error)
                .expect("model error")
                .contains("Unable to read local media")
        );
    } else {
        result.expect("ordinary media can be read");
    }
    // Delivered items contain the bytes that were checked, not a path the
    // child will open after it could have changed.
    std::fs::write(&path, b"swapped bytes").expect("swap");
    let delivered = manager
        .captured_ops()
        .into_iter()
        .filter_map(|(id, op)| {
            if id != child.thread_id {
                return None;
            }
            match op {
                Op::UserInput { items, .. } => Some(items),
                _ => None,
            }
        })
        .collect::<Vec<_>>();
    let expected = if denied {
        Vec::new()
    } else {
        let item = if kind == "local_image" {
            UserInput::Image {
                image_url: codex_utils_image::data_url_from_bytes(
                    "application/octet-stream",
                    b"original bytes",
                ),
                detail: None,
            }
        } else {
            UserInput::Audio {
                audio_url: codex_utils_image::data_url_from_bytes("audio/wav", b"original bytes"),
            }
        };
        vec![vec![item]]
    };
    assert_eq!(delivered, expected);
}
