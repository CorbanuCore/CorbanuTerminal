use anyhow::Context;
use anyhow::Result;
use app_test_support::MockResponsesConfig;
use app_test_support::TestAppServer;
use app_test_support::create_final_assistant_message_sse_response;
use app_test_support::create_mock_responses_server_sequence_unchecked;
use app_test_support::write_models_cache;
use codex_app_server_protocol::JSONRPCError;
use codex_app_server_protocol::RequestId;
use codex_app_server_protocol::SandboxPolicy;
use codex_app_server_protocol::ThreadReadParams;
use codex_app_server_protocol::ThreadReadResponse;
use codex_app_server_protocol::ThreadSettingsUpdateParams;
use codex_app_server_protocol::ThreadSettingsUpdateResponse;
use codex_app_server_protocol::ThreadSettingsUpdatedNotification;
use codex_app_server_protocol::ThreadStartParams;
use codex_app_server_protocol::ThreadStartResponse;
use codex_app_server_protocol::TurnStartParams;
use codex_app_server_protocol::TurnStartResponse;
use codex_app_server_protocol::UserInput as V2UserInput;
use codex_core::test_support::all_model_presets;
use codex_protocol::config_types::SERVICE_TIER_DEFAULT_REQUEST_VALUE;
use core_test_support::responses;
use pretty_assertions::assert_eq;
use serde_json::Value;
use std::time::Duration;
use tempfile::TempDir;
use tokio::time::timeout;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);

// These fixtures exercise native JSON-RPC, Core turns and real commands. Only
// inference is scripted; every write is confined to a disposable fixture.
async fn confirm_permission(
    mcp: &mut TestAppServer,
    thread_id: &str,
    policy: codex_app_server_protocol::AskForApproval,
) -> Result<()> {
    let request = mcp
        .send_thread_settings_update_request(ThreadSettingsUpdateParams {
            thread_id: thread_id.into(),
            confirm: true,
            approval_policy: Some(policy),
            sandbox_policy: Some(
                if policy == codex_app_server_protocol::AskForApproval::Never {
                    SandboxPolicy::DangerFullAccess
                } else {
                    SandboxPolicy::ReadOnly {
                        network_access: false,
                    }
                },
            ),
            ..Default::default()
        })
        .await?;
    assert_eq!(
        timeout(
            DEFAULT_TIMEOUT,
            mcp.read_response::<ThreadSettingsUpdateResponse>(request)
        )
        .await??,
        ThreadSettingsUpdateResponse::Confirmed {
            outcome: codex_app_server_protocol::ThreadSettingsUpdateOutcome::Applied
        }
    );
    Ok(())
}

fn write_probe(path: &std::path::Path, call_id: &str) -> Result<String> {
    app_test_support::create_shell_command_sse_response(
        vec![
            "python3".into(),
            "-c".into(),
            "import pathlib,sys; pathlib.Path(sys.argv[1]).write_text('executed')".into(),
            path.to_string_lossy().into_owned(),
        ],
        /*workdir*/ None,
        Some(5000),
        call_id,
    )
}

async fn finish_probe(mcp: &mut TestAppServer, call_id: &str) -> Result<usize> {
    use codex_app_server_protocol::JSONRPCMessage;
    use codex_app_server_protocol::ServerRequest;
    timeout(DEFAULT_TIMEOUT, async {
        let mut approvals = 0;
        loop {
            match mcp.read_next_message().await? {
                JSONRPCMessage::Request(request) => {
                    let ServerRequest::CommandExecutionRequestApproval { request_id, params } =
                        serde_json::from_value(serde_json::to_value(request)?)?
                    else {
                        anyhow::bail!("unexpected server request")
                    };
                    assert_eq!(params.item_id, call_id);
                    approvals += 1;
                    mcp.send_response(request_id, serde_json::json!({"decision": "decline"}))
                        .await?;
                }
                JSONRPCMessage::Notification(notification)
                    if notification.method == "turn/completed" =>
                {
                    return Ok::<_, anyhow::Error>(approvals);
                }
                _ => {}
            }
        }
    })
    .await?
}

#[tokio::test]
async fn thread_settings_confirmation_f04_restricts_work_steered_after_applied() -> Result<()> {
    use codex_app_server_protocol::AskForApproval;
    use codex_app_server_protocol::JSONRPCMessage;
    use std::sync::atomic::AtomicUsize;
    use std::sync::atomic::Ordering;

    // Desired post-Applied steering contract, pending a product decision.
    // Current "Permission selection confirmation — TO BUILD" explicitly keeps
    // active-turn snapshots unchanged and applies command permissions next turn.
    let fixture = TempDir::new()?;
    let workspace = fixture.path().join("workspace");
    std::fs::create_dir(&workspace)?;
    let ready = fixture.path().join("ready");
    let release = fixture.path().join("release");
    let marker = fixture.path().join("outside-workspace");
    let barrier_timeout = DEFAULT_TIMEOUT * 6;
    let barrier_deadline = barrier_timeout - DEFAULT_TIMEOUT;
    let barrier = app_test_support::create_shell_command_sse_response(
        vec![
            "python3".into(),
            "-c".into(),
            "import pathlib,sys,time; pathlib.Path(sys.argv[1]).touch(); deadline=time.monotonic()+float(sys.argv[3])\nwhile not pathlib.Path(sys.argv[2]).exists() and time.monotonic()<deadline: time.sleep(.01)".into(),
            ready.to_string_lossy().into_owned(),
            release.to_string_lossy().into_owned(),
            barrier_deadline.as_secs_f64().to_string(),
        ],
        Some(&workspace),
        Some(barrier_timeout.as_millis().try_into()?),
        "barrier",
    )?;
    let initial = [
        write_probe(&marker, "baseline")?,
        create_final_assistant_message_sse_response("baseline done")?,
        barrier,
    ];
    let probe = write_probe(&marker, "after-applied")?;
    let done = create_final_assistant_message_sse_response("steered done")?;
    let calls = AtomicUsize::new(0);
    let server = responses::start_mock_server().await;
    wiremock::Mock::given(wiremock::matchers::method("POST"))
        .and(wiremock::matchers::path_regex(".*/responses$"))
        .respond_with(move |request: &wiremock::Request| {
            let call = calls.fetch_add(1, Ordering::SeqCst);
            let body = if let Some(initial) = initial.get(call) {
                initial.clone()
            } else if call == initial.len()
                && request
                    .body_json::<Value>()
                    .expect("valid model request")
                    .to_string()
                    .contains("now run the outside-workspace probe")
            {
                // Refused/deferred steering must not make scripted inference
                // invent a write that the model was never asked to perform.
                probe.clone()
            } else {
                done.clone()
            };
            responses::sse_response(body)
        })
        .mount(&server)
        .await;
    let home = TempDir::new()?;
    create_config_toml(home.path(), &server.uri())?;
    let mut mcp = TestAppServer::builder()
        .with_codex_home(home.path())
        .build_initialized_with_timeout(Duration::from_secs(60))
        .await?;
    let request = mcp
        .send_thread_start_request_with_auto_env(ThreadStartParams {
            model: Some("mock-model".into()),
            cwd: Some(workspace.to_string_lossy().into_owned()),
            ..Default::default()
        })
        .await?;
    let ThreadStartResponse { thread, .. } = mcp.read_response(request).await?;
    confirm_permission(&mut mcp, &thread.id, AskForApproval::UnlessTrusted).await?;
    start_text_turn(&mut mcp, thread.id.clone()).await?;
    assert_eq!(finish_probe(&mut mcp, "baseline").await?, 1);
    assert!(!marker.exists());

    confirm_permission(&mut mcp, &thread.id, AskForApproval::Never).await?;
    let request = mcp
        .send_turn_start_request(TurnStartParams {
            thread_id: thread.id.clone(),
            input: vec![V2UserInput::Text {
                text: "wait at the barrier".into(),
                text_elements: vec![],
            }],
            ..Default::default()
        })
        .await?;
    let TurnStartResponse { turn } = mcp.read_response(request).await?;
    timeout(DEFAULT_TIMEOUT, async {
        while !ready.exists() {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .context("real barrier command did not start")?;
    confirm_permission(&mut mcp, &thread.id, AskForApproval::UnlessTrusted).await?;
    let request = mcp
        .send_turn_steer_request(codex_app_server_protocol::TurnSteerParams {
            thread_id: thread.id.clone(),
            expected_turn_id: turn.id.clone(),
            input: vec![V2UserInput::Text {
                text: "now run the outside-workspace probe".into(),
                text_elements: vec![],
            }],
            client_user_message_id: None,
            responsesapi_client_metadata: None,
            additional_context: None,
        })
        .await?;
    let acknowledged_same_turn = timeout(DEFAULT_TIMEOUT, async {
        loop {
            match mcp.read_next_message().await? {
                JSONRPCMessage::Response(response)
                    if response.id == RequestId::Integer(request) =>
                {
                    let steered: codex_app_server_protocol::TurnSteerResponse =
                        serde_json::from_value(response.result)?;
                    break Ok::<_, anyhow::Error>(steered.turn_id == turn.id);
                }
                JSONRPCMessage::Error(error) if error.id == RequestId::Integer(request) => {
                    break Ok(false);
                }
                JSONRPCMessage::Notification(_) => {}
                message => anyhow::bail!("unexpected message while awaiting steer: {message:?}"),
            }
        }
    })
    .await??;
    std::fs::write(&release, "continue")?;
    let approvals = finish_probe(&mut mcp, "after-applied").await?;
    let wrote = marker.exists();
    if wrote {
        std::fs::remove_file(&marker)?;
    }
    let model_received_steer = received_response_bodies(&server).await?.iter().any(|body| {
        body.to_string()
            .contains("now run the outside-workspace probe")
    });
    let admitted = acknowledged_same_turn && model_received_steer;

    // Reset the inference script so refusing/deferring the steer cannot shift
    // the independent fresh-turn control's response sequence.
    server.reset().await;
    responses::mount_sse_sequence(
        &server,
        vec![
            write_probe(&marker, "next-turn")?,
            create_final_assistant_message_sse_response("next done")?,
        ],
    )
    .await;
    start_text_turn(&mut mcp, thread.id.clone()).await?;
    assert_eq!(finish_probe(&mut mcp, "next-turn").await?, 1);
    assert!(!marker.exists(), "fresh turn must enforce the restriction");
    assert!(
        !wrote,
        concat!(
            "DESIRED post-Applied steering contract (pending product decision, not the current next-turn snapshot contract): ",
            "F04: work steered into the running turn after Applied must require approval and must not write after decline",
            "; refusing or deferring admission into this turn with no write is also permitted"
        )
    );
    assert!(
        (admitted && approvals > 0) || (!model_received_steer && approvals == 0),
        "DESIRED post-Applied steering contract: admitted work requires approval; otherwise it must not reach this turn's model or write"
    );
    Ok(())
}

#[tokio::test]
async fn thread_settings_confirmation_next_turn_tightening_and_loosening() -> Result<()> {
    use codex_app_server_protocol::AskForApproval;
    let home = TempDir::new()?;
    let outside = TempDir::new()?;
    let marker = outside.path().join("probe");
    let mut script = Vec::new();
    for _ in 0..3 {
        script.push(write_probe(&marker, "probe")?);
        script.push(create_final_assistant_message_sse_response("done")?);
    }
    let server = create_mock_responses_server_sequence_unchecked(script).await;
    create_config_toml(home.path(), &server.uri())?;
    let mut mcp = TestAppServer::builder()
        .with_codex_home(home.path())
        .build_initialized_with_timeout(Duration::from_secs(60))
        .await?;
    let thread = start_thread(&mut mcp).await?.thread;
    for (policy, expected_approvals, expected_write) in [
        (AskForApproval::Never, 0, true),
        (AskForApproval::UnlessTrusted, 1, false),
        (AskForApproval::Never, 0, true),
    ] {
        confirm_permission(&mut mcp, &thread.id, policy).await?;
        start_text_turn(&mut mcp, thread.id.clone()).await?;
        assert_eq!(
            (finish_probe(&mut mcp, "probe").await?, marker.exists()),
            (expected_approvals, expected_write)
        );
        if marker.exists() {
            std::fs::remove_file(&marker)?;
        }
    }
    Ok(())
}

#[tokio::test]
async fn thread_settings_confirmation_preserves_pending_approval() -> Result<()> {
    use codex_app_server_protocol::AskForApproval;
    use codex_app_server_protocol::ServerRequest;
    let home = TempDir::new()?;
    let outside = TempDir::new()?;
    let marker = outside.path().join("probe");
    let server = create_mock_responses_server_sequence_unchecked(vec![
        write_probe(&marker, "pending")?,
        create_final_assistant_message_sse_response("declined")?,
        write_probe(&marker, "next")?,
        create_final_assistant_message_sse_response("done")?,
    ])
    .await;
    create_config_toml(home.path(), &server.uri())?;
    let mut mcp = TestAppServer::builder()
        .with_codex_home(home.path())
        .build_initialized_with_timeout(Duration::from_secs(60))
        .await?;
    let thread = start_thread(&mut mcp).await?.thread;
    confirm_permission(&mut mcp, &thread.id, AskForApproval::UnlessTrusted).await?;
    start_text_turn(&mut mcp, thread.id.clone()).await?;
    let ServerRequest::CommandExecutionRequestApproval { request_id, params } =
        timeout(DEFAULT_TIMEOUT, mcp.read_stream_until_request_message()).await??
    else {
        anyhow::bail!("expected pending approval")
    };
    assert_eq!(params.item_id, "pending");
    // An outstanding approval must survive both a no-op confirmation and a
    // loosening confirmation; neither is consent to run the waiting command.
    for policy in [AskForApproval::UnlessTrusted, AskForApproval::Never] {
        confirm_permission(&mut mcp, &thread.id, policy).await?;
        assert!(
            !marker.exists(),
            "settings confirmation is not command approval"
        );
        let unexpected_resolution = timeout(
            Duration::from_millis(250),
            mcp.read_stream_until_matching_notification(
                "pending approval must not resolve after settings confirmation",
                |notification| {
                    notification.method == "serverRequest/resolved"
                        || notification.method == "turn/completed"
                },
            ),
        )
        .await;
        assert!(
            unexpected_resolution.is_err(),
            "expected the bounded stream drain to time out without resolution/completion: {unexpected_resolution:?}"
        );
    }
    mcp.send_response(request_id, serde_json::json!({"decision": "decline"}))
        .await?;
    assert_eq!(finish_probe(&mut mcp, "pending").await?, 0);
    assert!(
        !marker.exists(),
        "the original pending approval must remain declineable"
    );
    start_text_turn(&mut mcp, thread.id).await?;
    assert_eq!(finish_probe(&mut mcp, "next").await?, 0);
    assert_eq!(std::fs::read_to_string(marker)?, "executed");
    Ok(())
}

// An approved process crosses a settings update at a filesystem barrier. The
// next scripted command belongs to the original work, not to a steered message.
async fn preserve_running_authority_after_confirmation(
    final_policy: codex_app_server_protocol::AskForApproval,
) -> Result<()> {
    use codex_app_server_protocol::AskForApproval;
    use codex_app_server_protocol::ServerRequest;
    let fixture = TempDir::new()?;
    let ready = fixture.path().join("ready");
    let release = fixture.path().join("release");
    let completed = fixture.path().join("completed");
    let probe = fixture.path().join("probe");
    let barrier = app_test_support::create_shell_command_sse_response(
        vec![
            "python3".into(),
            "-c".into(),
            "import pathlib,sys,time; pathlib.Path(sys.argv[1]).touch(); deadline=time.monotonic()+50\nwhile not pathlib.Path(sys.argv[2]).exists() and time.monotonic()<deadline: time.sleep(.01)\nassert pathlib.Path(sys.argv[2]).exists(); pathlib.Path(sys.argv[3]).write_text('approved work completed')".into(),
            ready.to_string_lossy().into_owned(),
            release.to_string_lossy().into_owned(),
            completed.to_string_lossy().into_owned(),
        ],
        /*workdir*/ None,
        Some(60000),
        "approved-barrier",
    )?;
    let server = create_mock_responses_server_sequence_unchecked(vec![
        barrier,
        write_probe(&probe, "original-work-follow-up")?,
        create_final_assistant_message_sse_response("done")?,
    ])
    .await;
    let home = TempDir::new()?;
    create_config_toml(home.path(), &server.uri())?;
    let mut mcp = TestAppServer::builder()
        .with_codex_home(home.path())
        .build_initialized_with_timeout(Duration::from_secs(60))
        .await?;
    let thread = start_thread(&mut mcp).await?.thread;
    confirm_permission(&mut mcp, &thread.id, AskForApproval::UnlessTrusted).await?;
    start_text_turn(&mut mcp, thread.id.clone()).await?;
    let ServerRequest::CommandExecutionRequestApproval { request_id, params } =
        timeout(DEFAULT_TIMEOUT, mcp.read_stream_until_request_message()).await??
    else {
        anyhow::bail!("expected barrier approval");
    };
    assert_eq!(params.item_id, "approved-barrier");
    mcp.send_response(request_id, serde_json::json!({"decision": "accept"}))
        .await?;
    timeout(DEFAULT_TIMEOUT, async {
        while !ready.exists() {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .context("approved process did not reach barrier")?;
    confirm_permission(&mut mcp, &thread.id, AskForApproval::Never).await?;
    if final_policy != AskForApproval::Never {
        confirm_permission(&mut mcp, &thread.id, final_policy).await?;
    }
    assert!(!completed.exists(), "process must still be at the barrier");
    std::fs::write(&release, "continue")?;
    assert_eq!(finish_probe(&mut mcp, "original-work-follow-up").await?, 1);
    assert_eq!(
        std::fs::read_to_string(completed)?,
        "approved work completed"
    );
    assert!(
        !probe.exists(),
        "original work keeps its approval requirement"
    );
    Ok(())
}

#[tokio::test]
async fn thread_settings_confirmation_loosening_preserves_running_work_authority() -> Result<()> {
    preserve_running_authority_after_confirmation(codex_app_server_protocol::AskForApproval::Never)
        .await
}

#[tokio::test]
async fn thread_settings_confirmation_tightening_preserves_granted_approval() -> Result<()> {
    preserve_running_authority_after_confirmation(
        codex_app_server_protocol::AskForApproval::UnlessTrusted,
    )
    .await
}

#[tokio::test]
async fn thread_settings_confirmation_leaves_mcp_status_responses_unaffected() -> Result<()> {
    use codex_app_server_protocol::ListMcpServerStatusParams;
    use codex_app_server_protocol::ListMcpServerStatusResponse;
    use codex_app_server_protocol::McpServerStatusDetail;
    // This checks MCP status responses only. It does not prove equivalence of
    // live binding refresh, tool execution or authorization semantics.
    let mut observations = Vec::new();
    for confirm in [false, true] {
        let server = responses::start_mock_server().await;
        let home = TempDir::new()?;
        let config = |tool: &str| {
            MockResponsesConfig::new(&server.uri())
                .with_extra_config(&format!(
                    "[mcp_servers.fixture]\ncommand = {}\nenabled_tools = [\"{tool}\"]\n",
                    toml::Value::String(core_test_support::stdio_server_bin().unwrap())
                ))
                .write(home.path())
        };
        config("echo")?;
        let mut mcp = TestAppServer::builder()
            .with_codex_home(home.path())
            .build_initialized_with_timeout(Duration::from_secs(60))
            .await?;
        let thread = start_thread(&mut mcp).await?.thread;
        let mut lane = Vec::new();
        for refreshed in [false, true] {
            if refreshed {
                let request = mcp
                    .send_thread_settings_update_request(ThreadSettingsUpdateParams {
                        thread_id: thread.id.clone(),
                        confirm,
                        approval_policy: Some(codex_app_server_protocol::AskForApproval::Never),
                        sandbox_policy: Some(SandboxPolicy::DangerFullAccess),
                        ..Default::default()
                    })
                    .await?;
                let response: Value =
                    timeout(DEFAULT_TIMEOUT, mcp.read_response(request)).await??;
                assert_eq!(
                    response,
                    if confirm {
                        serde_json::json!({"outcome": "applied"})
                    } else {
                        serde_json::json!({})
                    }
                );
                config("sync")?;
                let request = mcp
                    .send_raw_request("config/mcpServer/reload", /*params*/ None)
                    .await?;
                let response: Value =
                    timeout(DEFAULT_TIMEOUT, mcp.read_response(request)).await??;
                assert_eq!(response, serde_json::json!({}));
            }
            let request = mcp
                .send_list_mcp_server_status_request(ListMcpServerStatusParams {
                    cursor: None,
                    limit: None,
                    detail: Some(McpServerStatusDetail::ToolsAndAuthOnly),
                    thread_id: Some(thread.id.clone()),
                })
                .await?;
            let response: ListMcpServerStatusResponse =
                timeout(DEFAULT_TIMEOUT, mcp.read_response(request)).await??;
            assert_eq!(response.data.len(), 1);
            let status = &response.data[0];
            let mut tools: Vec<_> = status.tools.keys().cloned().collect();
            tools.sort();
            assert_eq!(tools, vec![if refreshed { "sync" } else { "echo" }]);
            lane.push(response);
        }
        observations.push(lane);
    }
    assert_eq!(
        observations[0], observations[1],
        "MCP status responses are unaffected by confirmation; this does not prove refresh semantics are equivalent"
    );
    Ok(())
}

#[tokio::test]
async fn thread_settings_confirmation_noop_concurrent_and_legacy() -> Result<()> {
    let server = responses::start_mock_server().await;
    let codex_home = TempDir::new()?;
    create_config_toml(codex_home.path(), &server.uri())?;
    write_models_cache(codex_home.path())?;
    let mut mcp = TestAppServer::builder()
        .with_codex_home(codex_home.path())
        .build_initialized_with_timeout(DEFAULT_TIMEOUT)
        .await?;
    let thread = start_thread(&mut mcp).await?.thread;
    let legacy = mcp
        .send_thread_settings_update_request(ThreadSettingsUpdateParams {
            thread_id: thread.id.clone(),
            ..Default::default()
        })
        .await?;
    assert_eq!(
        timeout(DEFAULT_TIMEOUT, mcp.read_response::<Value>(legacy)).await??,
        serde_json::json!({})
    );
    for sandbox_policy in [
        SandboxPolicy::DangerFullAccess,
        SandboxPolicy::ReadOnly {
            network_access: false,
        },
    ] {
        // Identical submissions each need their own outcome, even when the
        // second operation produces no settings notification.
        let params = ThreadSettingsUpdateParams {
            thread_id: thread.id.clone(),
            confirm: true,
            sandbox_policy: Some(sandbox_policy),
            ..Default::default()
        };
        let first = mcp
            .send_thread_settings_update_request(params.clone())
            .await?;
        let second = mcp.send_thread_settings_update_request(params).await?;
        for request in [first, second] {
            assert_eq!(
                timeout(
                    DEFAULT_TIMEOUT,
                    mcp.read_response::<ThreadSettingsUpdateResponse>(request)
                )
                .await??,
                ThreadSettingsUpdateResponse::Confirmed {
                    outcome: codex_app_server_protocol::ThreadSettingsUpdateOutcome::Applied
                }
            );
        }
    }
    let empty = mcp
        .send_thread_settings_update_request(ThreadSettingsUpdateParams {
            thread_id: thread.id.clone(),
            confirm: true,
            ..Default::default()
        })
        .await?;
    assert_eq!(
        timeout(
            DEFAULT_TIMEOUT,
            mcp.read_response::<ThreadSettingsUpdateResponse>(empty)
        )
        .await??,
        ThreadSettingsUpdateResponse::Confirmed {
            outcome: codex_app_server_protocol::ThreadSettingsUpdateOutcome::Applied
        }
    );
    assert!(received_response_bodies(&server).await?.is_empty());
    Ok(())
}

#[tokio::test]
async fn thread_settings_update_emits_notification_and_updates_future_turns() -> Result<()> {
    let server = create_mock_responses_server_sequence_unchecked(vec![
        create_final_assistant_message_sse_response("done")?,
    ])
    .await;
    let codex_home = TempDir::new()?;
    create_config_toml(codex_home.path(), &server.uri())?;
    write_models_cache(codex_home.path())?;
    let (model_id, service_tier_id) = service_tier_model_and_tier_id()?;

    let mut mcp = TestAppServer::builder()
        .with_codex_home(codex_home.path())
        .build_initialized_with_timeout(DEFAULT_TIMEOUT)
        .await?;
    let thread = start_thread(&mut mcp).await?.thread;

    send_thread_settings_update(
        &mut mcp,
        ThreadSettingsUpdateParams {
            thread_id: thread.id.clone(),
            model: Some(model_id.clone()),
            service_tier: Some(Some(service_tier_id.clone())),
            ..Default::default()
        },
    )
    .await?;
    assert!(
        received_response_bodies(&server).await?.is_empty(),
        "settings-only update should not start a model request"
    );

    start_text_turn(&mut mcp, thread.id.clone()).await?;

    let updated = read_thread_settings_updated(&mut mcp).await?;
    assert_eq!(updated.thread_id, thread.id);
    assert_eq!(updated.thread_settings.model, model_id);
    assert_eq!(
        updated.thread_settings.service_tier.as_deref(),
        Some(service_tier_id.as_str())
    );

    timeout(
        DEFAULT_TIMEOUT,
        mcp.read_stream_until_notification_message("turn/completed"),
    )
    .await??;

    let read = read_thread_with_turns(&mut mcp, &thread.id).await?;
    assert_eq!(read.thread.turns.len(), 1);

    let request_bodies = received_response_bodies(&server).await?;
    assert!(
        request_bodies.iter().any(|body| {
            body.get("model").and_then(Value::as_str) == Some(model_id.as_str())
                && body.get("service_tier").and_then(Value::as_str)
                    == Some(service_tier_id.as_str())
        }),
        "future turn did not use updated model/service tier: {request_bodies:#?}"
    );
    Ok(())
}

#[tokio::test]
async fn thread_settings_update_cwd_retargets_default_environment() -> Result<()> {
    let server = responses::start_mock_server().await;
    let body = responses::sse(vec![
        responses::ev_response_created("resp-1"),
        responses::ev_assistant_message("msg-1", "done"),
        responses::ev_completed("resp-1"),
    ]);
    let response_mock = responses::mount_sse_once(&server, body).await;
    let codex_home = TempDir::new()?;
    let initial_workspace = TempDir::new()?;
    let workspace = TempDir::new()?;
    create_config_toml(codex_home.path(), &server.uri())?;

    let mut mcp = TestAppServer::builder()
        .with_codex_home(codex_home.path())
        .build_initialized_with_timeout(DEFAULT_TIMEOUT)
        .await?;
    let request_id = mcp
        .send_thread_start_request(ThreadStartParams {
            cwd: Some(initial_workspace.path().to_string_lossy().into_owned()),
            model: Some("mock-model".to_string()),
            ..Default::default()
        })
        .await?;
    let ThreadStartResponse { thread, .. } =
        timeout(DEFAULT_TIMEOUT, mcp.read_response(request_id)).await??;

    send_thread_settings_update(
        &mut mcp,
        ThreadSettingsUpdateParams {
            thread_id: thread.id.clone(),
            cwd: Some(workspace.path().to_path_buf()),
            ..Default::default()
        },
    )
    .await?;
    let updated = read_thread_settings_updated(&mut mcp).await?;
    assert_eq!(updated.thread_settings.cwd.as_path(), workspace.path());

    start_text_turn(&mut mcp, thread.id).await?;
    timeout(
        DEFAULT_TIMEOUT,
        mcp.read_stream_until_notification_message("turn/completed"),
    )
    .await??;

    let environment_context = response_mock
        .single_request()
        .message_input_texts("user")
        .into_iter()
        .find(|text| text.starts_with("<environment_context>"))
        .context("environment context should be model visible")?;
    assert!(
        environment_context.contains(&format!(
            "<cwd>{}</cwd>",
            workspace.path().to_string_lossy()
        )),
        "default environment should use the updated cwd: {environment_context}"
    );
    assert!(
        environment_context.contains(&format!(
            "<workspace_roots><root>{}</root></workspace_roots>",
            workspace.path().to_string_lossy()
        )),
        "default workspace root should use the updated cwd: {environment_context}"
    );

    Ok(())
}

#[tokio::test]
async fn thread_settings_update_while_turn_is_active_emits_notification() -> Result<()> {
    let server = responses::start_mock_server().await;
    let first_response =
        responses::sse_response(create_final_assistant_message_sse_response("first done")?)
            .set_delay(Duration::from_secs(2));
    let _requests = responses::mount_response_sequence(&server, vec![first_response]).await;
    let codex_home = TempDir::new()?;
    create_config_toml(codex_home.path(), &server.uri())?;

    let mut mcp = TestAppServer::builder()
        .with_codex_home(codex_home.path())
        .build_initialized_with_timeout(DEFAULT_TIMEOUT)
        .await?;
    let thread = start_thread(&mut mcp).await?.thread;
    start_text_turn(&mut mcp, thread.id.clone()).await?;
    timeout(
        DEFAULT_TIMEOUT,
        mcp.read_stream_until_notification_message("turn/started"),
    )
    .await??;

    send_thread_settings_update(
        &mut mcp,
        ThreadSettingsUpdateParams {
            thread_id: thread.id.clone(),
            model: Some("mock-model-4".to_string()),
            ..Default::default()
        },
    )
    .await?;

    let updated = read_thread_settings_updated(&mut mcp).await?;
    assert_eq!(updated.thread_id, thread.id);
    assert_eq!(updated.thread_settings.model, "mock-model-4");

    timeout(
        DEFAULT_TIMEOUT,
        mcp.read_stream_until_notification_message("turn/completed"),
    )
    .await??;
    Ok(())
}

#[tokio::test]
async fn thread_settings_update_null_service_tier_uses_default() -> Result<()> {
    let server = create_mock_responses_server_sequence_unchecked(vec![
        create_final_assistant_message_sse_response("done")?,
    ])
    .await;
    let codex_home = TempDir::new()?;
    create_config_toml(codex_home.path(), &server.uri())?;
    write_models_cache(codex_home.path())?;
    let (model_id, service_tier_id) = service_tier_model_and_tier_id()?;

    let mut mcp = TestAppServer::builder()
        .with_codex_home(codex_home.path())
        .build_initialized_with_timeout(DEFAULT_TIMEOUT)
        .await?;
    let thread = start_thread(&mut mcp).await?.thread;

    send_thread_settings_update(
        &mut mcp,
        ThreadSettingsUpdateParams {
            thread_id: thread.id.clone(),
            model: Some(model_id.clone()),
            service_tier: Some(Some(service_tier_id.clone())),
            ..Default::default()
        },
    )
    .await?;

    let set_updated = read_thread_settings_updated(&mut mcp).await?;
    assert_eq!(set_updated.thread_id, thread.id);
    assert_eq!(
        set_updated.thread_settings.service_tier.as_deref(),
        Some(service_tier_id.as_str())
    );

    send_thread_settings_update(
        &mut mcp,
        ThreadSettingsUpdateParams {
            thread_id: thread.id.clone(),
            service_tier: Some(None),
            ..Default::default()
        },
    )
    .await?;

    let clear_updated = read_thread_settings_updated(&mut mcp).await?;
    assert_eq!(clear_updated.thread_id, thread.id);
    assert_eq!(clear_updated.thread_settings.model, model_id);
    assert_eq!(
        clear_updated.thread_settings.service_tier.as_deref(),
        Some(SERVICE_TIER_DEFAULT_REQUEST_VALUE)
    );

    start_text_turn(&mut mcp, thread.id).await?;
    timeout(
        DEFAULT_TIMEOUT,
        mcp.read_stream_until_notification_message("turn/completed"),
    )
    .await??;

    let request_bodies = received_response_bodies(&server).await?;
    assert!(
        request_bodies.iter().any(|body| {
            body.get("model").and_then(Value::as_str) == Some(model_id.as_str())
                && body
                    .as_object()
                    .is_some_and(|object| !object.contains_key("service_tier"))
        }),
        "future turn did not clear service tier: {request_bodies:#?}"
    );
    Ok(())
}

#[tokio::test]
async fn thread_settings_update_rejects_sandbox_policy_with_permissions() -> Result<()> {
    let server = create_mock_responses_server_sequence_unchecked(Vec::new()).await;
    let codex_home = TempDir::new()?;
    create_config_toml(codex_home.path(), &server.uri())?;

    let mut mcp = TestAppServer::builder()
        .with_codex_home(codex_home.path())
        .build_initialized_with_timeout(DEFAULT_TIMEOUT)
        .await?;
    let thread = start_thread(&mut mcp).await?.thread;

    let request_id = mcp
        .send_thread_settings_update_request(ThreadSettingsUpdateParams {
            confirm: true,
            thread_id: thread.id,
            sandbox_policy: Some(SandboxPolicy::DangerFullAccess),
            permissions: Some(":workspace".to_string()),
            ..Default::default()
        })
        .await?;
    let error: JSONRPCError = timeout(
        DEFAULT_TIMEOUT,
        mcp.read_stream_until_error_message(RequestId::Integer(request_id)),
    )
    .await??;

    assert_eq!(
        error.error.message,
        "`permissions` cannot be combined with `sandboxPolicy`"
    );
    Ok(())
}

#[tokio::test]
async fn turn_start_settings_override_emits_thread_settings_updated() -> Result<()> {
    let server = create_mock_responses_server_sequence_unchecked(vec![
        create_final_assistant_message_sse_response("done")?,
    ])
    .await;
    let codex_home = TempDir::new()?;
    create_config_toml(codex_home.path(), &server.uri())?;

    let mut mcp = TestAppServer::builder()
        .with_codex_home(codex_home.path())
        .build_initialized_with_timeout(DEFAULT_TIMEOUT)
        .await?;
    let thread = start_thread(&mut mcp).await?.thread;
    timeout(
        DEFAULT_TIMEOUT,
        mcp.read_stream_until_notification_message("thread/started"),
    )
    .await??;

    let turn_request_id = mcp
        .send_turn_start_request(TurnStartParams {
            thread_id: thread.id.clone(),
            client_user_message_id: None,
            input: vec![V2UserInput::Text {
                text: "hello".to_string(),
                text_elements: Vec::new(),
            }],
            model: Some("mock-model-3".to_string()),
            ..Default::default()
        })
        .await?;
    let TurnStartResponse { turn } =
        timeout(DEFAULT_TIMEOUT, mcp.read_response(turn_request_id)).await??;
    assert!(!turn.id.is_empty());

    let updated = read_thread_settings_updated(&mut mcp).await?;
    assert_eq!(updated.thread_id, thread.id);
    assert_eq!(updated.thread_settings.model, "mock-model-3");

    timeout(
        DEFAULT_TIMEOUT,
        mcp.read_stream_until_notification_message("turn/completed"),
    )
    .await??;
    Ok(())
}

async fn send_thread_settings_update(
    mcp: &mut TestAppServer,
    params: ThreadSettingsUpdateParams,
) -> Result<()> {
    let request_id = mcp.send_thread_settings_update_request(params).await?;
    let _: ThreadSettingsUpdateResponse =
        timeout(DEFAULT_TIMEOUT, mcp.read_response(request_id)).await??;
    Ok(())
}

async fn start_text_turn(mcp: &mut TestAppServer, thread_id: String) -> Result<()> {
    let turn_request_id = mcp
        .send_turn_start_request(TurnStartParams {
            thread_id,
            input: vec![V2UserInput::Text {
                text: "hello".to_string(),
                text_elements: Vec::new(),
            }],
            ..Default::default()
        })
        .await?;
    let TurnStartResponse { turn } =
        timeout(DEFAULT_TIMEOUT, mcp.read_response(turn_request_id)).await??;
    assert!(!turn.id.is_empty());
    Ok(())
}

async fn start_thread(mcp: &mut TestAppServer) -> Result<ThreadStartResponse> {
    let request_id = mcp
        .send_thread_start_request_with_auto_env(ThreadStartParams {
            model: Some("mock-model".to_string()),
            ..Default::default()
        })
        .await?;
    timeout(DEFAULT_TIMEOUT, mcp.read_response(request_id)).await?
}

async fn read_thread_with_turns(
    mcp: &mut TestAppServer,
    thread_id: &str,
) -> Result<ThreadReadResponse> {
    let request_id = mcp
        .send_thread_read_request(ThreadReadParams {
            thread_id: thread_id.to_string(),
            include_turns: true,
        })
        .await?;
    timeout(DEFAULT_TIMEOUT, mcp.read_response(request_id)).await?
}

async fn read_thread_settings_updated(
    mcp: &mut TestAppServer,
) -> Result<ThreadSettingsUpdatedNotification> {
    timeout(
        DEFAULT_TIMEOUT,
        mcp.read_notification("thread/settings/updated"),
    )
    .await?
}

async fn received_response_bodies(server: &wiremock::MockServer) -> Result<Vec<Value>> {
    let requests = server
        .received_requests()
        .await
        .context("failed to fetch received requests")?;
    let mut bodies = Vec::new();
    for request in requests {
        if request.url.path().ends_with("/responses") {
            bodies.push(request.body_json::<Value>()?);
        }
    }
    Ok(bodies)
}

fn service_tier_model_and_tier_id() -> Result<(String, String)> {
    let model = all_model_presets()
        .iter()
        .find(|preset| preset.show_in_picker && !preset.service_tiers.is_empty())
        .context("bundled model catalog should include a picker model with service tiers")?;
    Ok((model.id.clone(), model.service_tiers[0].id.clone()))
}

fn create_config_toml(codex_home: &std::path::Path, server_uri: &str) -> std::io::Result<()> {
    MockResponsesConfig::new(server_uri)
        .with_root_config("compact_prompt = \"compact\"\nmodel_auto_compact_token_limit = 200000")
        .with_provider_config("supports_websockets = false")
        .write(codex_home)
}
