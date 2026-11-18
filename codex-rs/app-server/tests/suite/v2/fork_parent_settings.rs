//! #2015 (D519, D711 (a)): upstream replaced the detached review with
//! `thread/fork` + `turn/start`. A thread forked from a loaded thread runs
//! with that thread's current settings (approval policy, sandbox, Windows
//! sandbox, security level), including changes made since it started, never
//! with the weaker ones the server started with, and a fork request cannot
//! put it below the parent's security level.

use std::path::Path;
use std::time::Duration;

use anyhow::Result;
use app_test_support::MockResponsesConfig;
use app_test_support::TestAppServer;
use codex_app_server_protocol::AskForApproval;
use codex_app_server_protocol::ClientRequest;
use codex_app_server_protocol::SandboxMode;
use codex_app_server_protocol::SandboxPolicy;
use codex_app_server_protocol::ThreadForkParams;
use codex_app_server_protocol::ThreadForkResponse;
use codex_app_server_protocol::ThreadSettingsUpdateOutcome;
use codex_app_server_protocol::ThreadSettingsUpdateParams;
use codex_app_server_protocol::ThreadSettingsUpdateResponse;
use codex_app_server_protocol::ThreadStartParams;
use codex_app_server_protocol::ThreadStartResponse;
use codex_app_server_protocol::TurnStartParams;
use codex_app_server_protocol::TurnStartResponse;
use codex_app_server_protocol::UserInput as V2UserInput;
use codex_features::Feature;
use codex_rollout::RolloutItem;
use codex_rollout::RolloutRecorder;
use core_test_support::responses;
use pretty_assertions::assert_eq;
use serde_json::json;
use tempfile::TempDir;
use tokio::time::timeout;

const DEFAULT_READ_TIMEOUT: Duration = Duration::from_secs(10);
const WEAK_SANDBOX: &str = "`sandbox_mode` is `danger-full-access`";
const WEAK_APPROVALS: &str = "Approval policy is currently never";

fn two_turn_sse() -> Vec<String> {
    ["parent", "fork"]
        .into_iter()
        .map(|name| {
            responses::sse(vec![
                responses::ev_response_created(&format!("{name}-response")),
                responses::ev_assistant_message(&format!("{name}-message"), "done"),
                responses::ev_completed(&format!("{name}-response")),
            ])
        })
        .collect()
}

/// The server starts with the weakest settings, so inheriting them would show.
async fn weak_server(codex_home: &Path, server_uri: &str) -> Result<TestAppServer> {
    MockResponsesConfig::new(server_uri)
        .with_provider_name("Mock provider")
        .with_approval_policy("never")
        .with_sandbox_mode("danger-full-access")
        .disable_feature(Feature::ShellSnapshot)
        // Moderate and Aggressive send nothing without source envelopes.
        .enable_feature(Feature::SourceEnvelopes)
        .write(codex_home)?;
    TestAppServer::builder()
        .with_codex_home(codex_home)
        .build_initialized()
        .await
}

fn security_config(level: &str) -> Option<std::collections::HashMap<String, serde_json::Value>> {
    Some(
        [(
            "security".to_string(),
            json!({ "version": 1, "level": level }),
        )]
        .into_iter()
        .collect(),
    )
}

/// An Aggressive, read-only parent thread with one completed turn.
async fn start_aggressive_parent(mcp: &mut TestAppServer) -> Result<String> {
    let ThreadStartResponse { thread, .. } = mcp
        .start_thread(ThreadStartParams {
            model: Some("mock-model".to_string()),
            sandbox: Some(SandboxMode::ReadOnly),
            config: security_config("aggressive"),
            ..Default::default()
        })
        .await?;
    timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_notification_message("thread/started"),
    )
    .await??;
    run_turn(mcp, &thread.id).await?;
    Ok(thread.id)
}

async fn run_turn(mcp: &mut TestAppServer, thread_id: &str) -> Result<()> {
    let _: TurnStartResponse = mcp
        .request(|request_id| ClientRequest::TurnStart {
            request_id,
            params: TurnStartParams {
                thread_id: thread_id.to_string(),
                client_user_message_id: None,
                input: vec![V2UserInput::Text {
                    text: "hello".to_string(),
                    text_elements: Vec::new(),
                }],
                ..Default::default()
            },
        })
        .await?;
    timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_notification_message("turn/completed"),
    )
    .await??;
    Ok(())
}

async fn fork(mcp: &mut TestAppServer, params: ThreadForkParams) -> Result<ThreadForkResponse> {
    let request = mcp.send_thread_fork_request(params).await?;
    timeout(DEFAULT_READ_TIMEOUT, mcp.read_response(request)).await?
}

/// The security level the thread's last recorded turn ran under, as JSON.
async fn last_turn_security_level(rollout: &Path) -> Result<serde_json::Value> {
    let deadline = tokio::time::Instant::now() + DEFAULT_READ_TIMEOUT;
    loop {
        let (items, _, _) = RolloutRecorder::load_rollout_items(rollout).await?;
        let level = items.iter().rev().find_map(|item| match item {
            RolloutItem::TurnContext(context) => Some(json!(context.security_level)),
            _ => None,
        });
        if let Some(level) = level {
            return Ok(level);
        }
        if tokio::time::Instant::now() >= deadline {
            anyhow::bail!("no turn context in {}", rollout.display());
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

fn sandbox_tag(request: &core_test_support::responses::ResponsesRequest) -> String {
    let metadata: serde_json::Value = serde_json::from_str(
        &request
            .header("x-codex-turn-metadata")
            .expect("x-codex-turn-metadata header"),
    )
    .expect("turn metadata JSON");
    metadata["sandbox"]
        .as_str()
        .expect("sandbox tag")
        .to_string()
}

/// #2015: the fork runs with the parent's approval policy as tightened
/// mid-session (a settings update after its last turn), its sandbox and its
/// security level, not with the server's weaker start-up settings.
#[tokio::test]
async fn issue_2015_fork_runs_with_the_parents_current_settings() -> Result<()> {
    let server = responses::start_mock_server().await;
    let response_mock = responses::mount_sse_sequence(&server, two_turn_sse()).await;
    let codex_home = TempDir::new()?;
    let mut mcp = weak_server(codex_home.path(), &server.uri()).await?;
    let parent_id = start_aggressive_parent(&mut mcp).await?;

    // Tightened after the parent's last turn: only the live thread has it.
    let request = mcp
        .send_thread_settings_update_request(ThreadSettingsUpdateParams {
            thread_id: parent_id.clone(),
            confirm: true,
            approval_policy: Some(AskForApproval::UnlessTrusted),
            sandbox_policy: Some(SandboxPolicy::ReadOnly {
                network_access: false,
            }),
            ..Default::default()
        })
        .await?;
    assert_eq!(
        timeout(
            DEFAULT_READ_TIMEOUT,
            mcp.read_response::<ThreadSettingsUpdateResponse>(request)
        )
        .await??,
        ThreadSettingsUpdateResponse::Confirmed {
            outcome: ThreadSettingsUpdateOutcome::Applied
        }
    );

    let forked = fork(
        &mut mcp,
        ThreadForkParams {
            thread_id: parent_id,
            ..Default::default()
        },
    )
    .await?;
    assert_eq!(forked.approval_policy, AskForApproval::UnlessTrusted);
    assert_eq!(
        forked.sandbox,
        SandboxPolicy::ReadOnly {
            network_access: false
        }
    );
    run_turn(&mut mcp, &forked.thread.id).await?;

    let requests = response_mock.requests();
    assert_eq!(requests.len(), 2);
    let (parent, fork_request) = (&requests[0], &requests[1]);
    assert!(fork_request.body_contains_text("`sandbox_mode` is `read-only`"));
    assert!(!fork_request.body_contains_text(WEAK_SANDBOX));
    assert!(!fork_request.body_contains_text(WEAK_APPROVALS));
    // The same Windows sandbox (off Windows, none for both).
    assert_eq!(sandbox_tag(fork_request), sandbox_tag(parent));
    let rollout = forked.thread.path.expect("fork rollout path");
    assert_eq!(
        last_turn_security_level(&rollout).await?,
        json!("aggressive")
    );
    Ok(())
}

/// D711 (a): a fork request that asks for a lower security level than the
/// parent's runs at the parent's level.
#[tokio::test]
async fn issue_2015_fork_request_cannot_lower_the_parents_security_level() -> Result<()> {
    let server = responses::start_mock_server().await;
    let _response_mock = responses::mount_sse_sequence(&server, two_turn_sse()).await;
    let codex_home = TempDir::new()?;
    let mut mcp = weak_server(codex_home.path(), &server.uri()).await?;
    let parent_id = start_aggressive_parent(&mut mcp).await?;

    let forked = fork(
        &mut mcp,
        ThreadForkParams {
            thread_id: parent_id,
            config: security_config("permissive"),
            ..Default::default()
        },
    )
    .await?;
    run_turn(&mut mcp, &forked.thread.id).await?;
    let rollout = forked.thread.path.expect("fork rollout path");
    assert_eq!(
        last_turn_security_level(&rollout).await?,
        json!("aggressive")
    );
    Ok(())
}

/// #2015: a Windows sandbox set up after the server started (setup saves
/// `[windows] sandbox`; a new thread loads it) reaches a fork of a thread
/// that runs with it.
#[tokio::test]
async fn issue_2015_fork_keeps_a_windows_sandbox_set_up_after_start() -> Result<()> {
    let server = responses::start_mock_server().await;
    let response_mock = responses::mount_sse_sequence(&server, two_turn_sse()).await;
    let codex_home = TempDir::new()?;
    MockResponsesConfig::new(&server.uri())
        .with_provider_name("Mock provider")
        .with_approval_policy("never")
        .with_sandbox_mode("workspace-write")
        .disable_feature(Feature::ShellSnapshot)
        .write(codex_home.path())?;
    let mut mcp = TestAppServer::builder()
        .with_codex_home(codex_home.path())
        .build_initialized()
        .await?;
    let config = codex_home.path().join("config.toml");
    let mut configured = std::fs::read_to_string(&config)?;
    configured.push_str("\n[windows]\nsandbox = \"unelevated\"\n");
    std::fs::write(&config, configured)?;

    let ThreadStartResponse { thread, .. } = mcp
        .start_thread(ThreadStartParams {
            model: Some("mock-model".to_string()),
            ..Default::default()
        })
        .await?;
    timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_notification_message("thread/started"),
    )
    .await??;
    run_turn(&mut mcp, &thread.id).await?;
    let forked = fork(
        &mut mcp,
        ThreadForkParams {
            thread_id: thread.id,
            ..Default::default()
        },
    )
    .await?;
    run_turn(&mut mcp, &forked.thread.id).await?;

    let requests = response_mock.requests();
    assert_eq!(requests.len(), 2);
    if cfg!(windows) {
        assert_ne!(
            sandbox_tag(&requests[0]),
            "none",
            "the thread has the sandbox"
        );
    }
    assert_eq!(sandbox_tag(&requests[1]), sandbox_tag(&requests[0]));
    Ok(())
}
