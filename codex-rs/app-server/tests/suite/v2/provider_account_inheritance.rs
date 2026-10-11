//! PF-84-S03 follow-up: a worker a client (the TUI) starts with
//! `thread/spawnAgent` inherits its parent's live named account.

use anyhow::Result;
use app_test_support::TestAppServer;
use app_test_support::to_response;
use codex_app_server_protocol::JSONRPCResponse;
use codex_app_server_protocol::RequestId;
use codex_app_server_protocol::ThreadSpawnAgentParams;
use codex_app_server_protocol::ThreadSpawnAgentResponse;
use codex_app_server_protocol::ThreadStartParams;
use codex_app_server_protocol::ThreadStartResponse;
use codex_app_server_protocol::TurnStartParams;
use codex_app_server_protocol::UserInput;
use core_test_support::responses;
use std::time::Duration;
use tempfile::TempDir;
use tokio::time::timeout;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);
const WORK_KEY: &str = "pf84-s03-work-account-key";

#[tokio::test]
async fn client_started_worker_inherits_the_parent_account() -> Result<()> {
    let server = responses::start_mock_server().await;
    let response_mock = responses::mount_sse_once(
        &server,
        responses::sse(vec![
            responses::ev_response_created("resp-1"),
            responses::ev_assistant_message("msg-1", "pong"),
            responses::ev_completed("resp-1"),
        ]),
    )
    .await;
    let codex_home = TempDir::new()?;
    std::fs::write(
        codex_home.path().join("config.toml"),
        format!(
            r#"
model = "mock-model"
approval_policy = "never"
sandbox_mode = "read-only"
model_provider = "mock_provider"

[features]
named_accounts = true

[model_providers.mock_provider]
name = "Mock provider for test"
base_url = "{}/v1"
wire_api = "responses"
env_key = "PF84_S03_UNSET_DEFAULT_KEY"
request_max_retries = 0
stream_max_retries = 0
"#,
            server.uri()
        ),
    )?;
    codex_vault::Vault::new(codex_home.path().to_path_buf()).write_provider_account(
        "mock_provider",
        &codex_vault::ProviderAccountName::parse("work")?,
        codex_vault::ProviderAccountKind::ApiKey,
        WORK_KEY,
    )?;

    let mut app = TestAppServer::builder()
        .with_codex_home(codex_home.path())
        .without_auto_env()
        .build()
        .await?;
    timeout(DEFAULT_TIMEOUT, app.initialize()).await??;

    let root_request = app
        .send_thread_start_request(ThreadStartParams {
            model: Some("mock-model".to_string()),
            provider_account: Some("work".to_string()),
            ..Default::default()
        })
        .await?;
    let root_response = timeout(
        DEFAULT_TIMEOUT,
        app.read_stream_until_response_message(RequestId::Integer(root_request)),
    )
    .await??;
    let ThreadStartResponse { thread: root, .. } =
        to_response::<ThreadStartResponse>(root_response)?;

    // The worker names no account: it must not fall back to `default`.
    let child_request = app
        .send_raw_request(
            "thread/spawnAgent",
            Some(serde_json::to_value(ThreadSpawnAgentParams {
                parent_thread_id: root.id.clone(),
                agent_role: "worker".to_string(),
                agent_nickname: Some("worker-a".to_string()),
                agent_class: None,
                thread: ThreadStartParams {
                    model: Some("mock-model".to_string()),
                    ..Default::default()
                },
            })?),
        )
        .await?;
    let child_response: JSONRPCResponse = timeout(
        DEFAULT_TIMEOUT,
        app.read_stream_until_response_message(RequestId::Integer(child_request)),
    )
    .await??;
    let ThreadSpawnAgentResponse { thread: child, .. } =
        to_response::<ThreadSpawnAgentResponse>(child_response)?;

    timeout(
        DEFAULT_TIMEOUT,
        app.start_turn_and_wait_for_completion(TurnStartParams {
            thread_id: child.id,
            input: vec![UserInput::Text {
                text: "say pong".to_string(),
                text_elements: Vec::new(),
            }],
            ..Default::default()
        }),
    )
    .await??;

    assert_eq!(
        response_mock.single_request().header("authorization"),
        Some(format!("Bearer {WORK_KEY}"))
    );
    Ok(())
}
