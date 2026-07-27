use anyhow::Result;
use app_test_support::TestAppServer;
use app_test_support::to_response;
use codex_app_server_protocol::JSONRPCResponse;
use codex_app_server_protocol::RequestId;
use codex_app_server_protocol::ThreadAgentMessageParams;
use codex_app_server_protocol::ThreadAgentMessageResponse;
use codex_app_server_protocol::ThreadSpawnAgentParams;
use codex_app_server_protocol::ThreadSpawnAgentResponse;
use codex_app_server_protocol::ThreadStartParams;
use codex_app_server_protocol::ThreadStartResponse;
use codex_protocol::crew::AgentClass;
use codex_protocol::protocol::AgentMessageKind;
use core_test_support::responses;
use std::path::Path;
use std::time::Duration;
use tempfile::TempDir;
use tokio::time::timeout;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);

#[tokio::test]
async fn thread_agent_message_uses_native_mailbox_and_deduplicates_stable_id() -> Result<()> {
    let server = responses::start_mock_server().await;
    let body = responses::sse(vec![
        responses::ev_response_created("resp-1"),
        responses::ev_assistant_message("msg-1", "accepted"),
        responses::ev_completed("resp-1"),
    ]);
    let response_mock = responses::mount_sse_once(&server, body).await;
    let codex_home = TempDir::new()?;
    create_config_toml(codex_home.path(), &server.uri())?;

    let mut app = TestAppServer::new(codex_home.path()).await?;
    timeout(DEFAULT_TIMEOUT, app.initialize()).await??;

    let root_request = app
        .send_thread_start_request(ThreadStartParams {
            model: Some("mock-model".to_string()),
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

    let child_request = app
        .send_raw_request(
            "thread/spawnAgent",
            Some(serde_json::to_value(ThreadSpawnAgentParams {
                parent_thread_id: root.id.clone(),
                agent_role: "worker".to_string(),
                agent_nickname: Some("worker-a".to_string()),
                agent_class: Some(AgentClass::CrewMember {
                    crew_id: "native-app-server-crew".to_string(),
                    logical_member_id: "worker-a".to_string(),
                    human_addressable: true,
                }),
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

    let params = ThreadAgentMessageParams {
        source_thread_id: root.id.clone(),
        target_thread_id: child.id.clone(),
        message_id: Some("message-stable-1".to_string()),
        assignment_id: Some("assignment-stable-1".to_string()),
        kind: AgentMessageKind::Assignment,
        content: "Inspect the provider-neutral boundary.".to_string(),
        trigger_turn: true,
    };
    let message_request = app
        .send_raw_request(
            "thread/sendAgentMessage",
            Some(serde_json::to_value(&params)?),
        )
        .await?;
    let message_response: JSONRPCResponse = timeout(
        DEFAULT_TIMEOUT,
        app.read_stream_until_response_message(RequestId::Integer(message_request)),
    )
    .await??;
    let receipt = to_response::<ThreadAgentMessageResponse>(message_response)?;
    assert_eq!(receipt.message_id, "message-stable-1");
    assert_eq!(receipt.target_thread_id, child.id);
    assert!(receipt.trigger_turn);

    timeout(
        DEFAULT_TIMEOUT,
        app.read_stream_until_notification_message("turn/completed"),
    )
    .await??;
    let requests = response_mock.requests();
    assert_eq!(requests.len(), 1);
    let request_json = serde_json::to_string(&requests[0].input())?;
    assert!(request_json.contains("Inspect the provider-neutral boundary."));
    assert!(
        !request_json.contains("message-stable-1") && !request_json.contains("assignment-stable-1"),
        "transport identities must not be copied into model-visible task text"
    );

    let duplicate_request = app
        .send_raw_request(
            "thread/sendAgentMessage",
            Some(serde_json::to_value(params)?),
        )
        .await?;
    let duplicate_response: JSONRPCResponse = timeout(
        DEFAULT_TIMEOUT,
        app.read_stream_until_response_message(RequestId::Integer(duplicate_request)),
    )
    .await??;
    let duplicate_receipt = to_response::<ThreadAgentMessageResponse>(duplicate_response)?;
    assert_eq!(duplicate_receipt.message_id, "message-stable-1");
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert_eq!(
        response_mock.requests().len(),
        1,
        "a duplicate stable message id must not start another provider turn"
    );

    let reverse_response_mock = responses::mount_sse_once(
        &server,
        responses::sse(vec![
            responses::ev_response_created("resp-2"),
            responses::ev_assistant_message("msg-2", "parent accepted"),
            responses::ev_completed("resp-2"),
        ]),
    )
    .await;
    let reverse_request = app
        .send_raw_request(
            "thread/sendAgentMessage",
            Some(serde_json::to_value(ThreadAgentMessageParams {
                source_thread_id: child.id,
                target_thread_id: root.id.clone(),
                message_id: Some("message-child-to-root-1".to_string()),
                assignment_id: Some("assignment-child-to-root-1".to_string()),
                kind: AgentMessageKind::Assignment,
                content: "Apply the child review to the root task.".to_string(),
                trigger_turn: true,
            })?),
        )
        .await?;
    let reverse_response: JSONRPCResponse = timeout(
        DEFAULT_TIMEOUT,
        app.read_stream_until_response_message(RequestId::Integer(reverse_request)),
    )
    .await??;
    let reverse_receipt = to_response::<ThreadAgentMessageResponse>(reverse_response)?;
    assert_eq!(reverse_receipt.target_thread_id, root.id);
    timeout(
        DEFAULT_TIMEOUT,
        app.read_stream_until_notification_message("turn/completed"),
    )
    .await??;
    assert_eq!(reverse_response_mock.requests().len(), 1);

    Ok(())
}

fn create_config_toml(codex_home: &Path, server_uri: &str) -> std::io::Result<()> {
    std::fs::write(
        codex_home.join("config.toml"),
        format!(
            r#"
model = "mock-model"
approval_policy = "never"
sandbox_mode = "read-only"
model_provider = "mock_provider"

[model_providers.mock_provider]
name = "Mock provider for test"
base_url = "{server_uri}/v1"
wire_api = "responses"
request_max_retries = 0
stream_max_retries = 0
"#
        ),
    )
}
