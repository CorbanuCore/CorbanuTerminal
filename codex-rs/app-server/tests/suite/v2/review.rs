use anyhow::Result;
use app_test_support::MockResponsesConfig;
use app_test_support::TestAppServer;
use app_test_support::create_command_execution_sse_response;
use app_test_support::create_final_assistant_message_sse_response;
use app_test_support::create_mock_responses_server_repeating_assistant;
use app_test_support::create_mock_responses_server_sequence;
use codex_app_server_protocol::AskForApproval;
use codex_app_server_protocol::ClientRequest;
use codex_app_server_protocol::ItemCompletedNotification;
use codex_app_server_protocol::ItemStartedNotification;
use codex_app_server_protocol::JSONRPCError;
use codex_app_server_protocol::RequestId;
use codex_app_server_protocol::ReviewDelivery;
use codex_app_server_protocol::ReviewStartParams;
use codex_app_server_protocol::ReviewStartResponse;
use codex_app_server_protocol::ReviewTarget;
use codex_app_server_protocol::SandboxMode;
use codex_app_server_protocol::ServerRequest;
use codex_app_server_protocol::ThreadItem;
use codex_app_server_protocol::ThreadItemPayload;
use codex_app_server_protocol::ThreadStartParams;
use codex_app_server_protocol::ThreadStartResponse;
use codex_app_server_protocol::TurnItemsView;
use codex_app_server_protocol::TurnStatus;
use codex_app_server_protocol::UserInput as V2UserInput;
use codex_features::Feature;
use pretty_assertions::assert_eq;
use serde_json::json;
use tempfile::TempDir;
use test_case::test_case;
use tokio::time::timeout;

const DEFAULT_READ_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);
const INVALID_REQUEST_ERROR_CODE: i64 = -32600;

#[test_case(None; "omitted_delivery")]
#[test_case(Some(json!(null)); "null_delivery")]
#[test_case(Some(json!("inline")); "inline_delivery")]
#[tokio::test]
async fn review_start_runs_review_turn_and_emits_code_review_item(
    delivery: Option<serde_json::Value>,
) -> Result<()> {
    let review_payload = json!({
        "findings": [
            {
                "title": "Prefer Stylize helpers",
                "body": "Use .dim()/.bold() chaining instead of manual Style.",
                "confidence_score": 0.9,
                "priority": 1,
                "code_location": {
                    "absolute_file_path": "/tmp/file.rs",
                    "line_range": {"start": 10, "end": 20}
                }
            }
        ],
        "overall_correctness": "good",
        "overall_explanation": "Looks solid overall with minor polish suggested.",
        "overall_confidence_score": 0.75
    })
    .to_string();
    let server = create_mock_responses_server_repeating_assistant(&review_payload).await;

    let codex_home = TempDir::new()?;
    create_config_toml(codex_home.path(), &server.uri())?;

    let mut mcp = TestAppServer::builder()
        .with_codex_home(codex_home.path())
        .build_initialized()
        .await?;
    let thread_id = start_default_thread(&mut mcp).await?;
    let mut params = json!({
        "threadId": thread_id,
        "target": {
            "type": "commit",
            "sha": "1234567deadbeef",
            "title": "Tidy UI colors",
        },
    });
    if let Some(delivery) = delivery {
        params["delivery"] = delivery;
    }
    let request_id = mcp.send_raw_request("review/start", Some(params)).await?;
    let ReviewStartResponse {
        turn,
        review_thread_id,
    } = timeout(DEFAULT_READ_TIMEOUT, mcp.read_response(request_id)).await??;
    assert_eq!(review_thread_id, thread_id.clone());
    let turn_id = turn.id.clone();
    assert_eq!(turn.status, TurnStatus::InProgress);
    assert_eq!(turn.items_view, TurnItemsView::NotLoaded);
    assert_eq!(
        turn.items,
        [ThreadItem::untimed(ThreadItemPayload::UserMessage {
            id: turn_id.clone(),
            client_id: None,
            content: vec![V2UserInput::Text {
                text: "commit 1234567: Tidy UI colors".to_string(),
                text_elements: Vec::new(),
            }],
        })]
    );

    // Confirm we see the EnteredReviewMode marker on the main thread.
    let mut saw_entered_review_mode = false;
    for _ in 0..10 {
        let started: ItemStartedNotification =
            timeout(DEFAULT_READ_TIMEOUT, mcp.read_notification("item/started")).await??;
        match started.item.payload {
            ThreadItemPayload::EnteredReviewMode { review, .. } => {
                assert_eq!(started.turn_id, turn_id);
                assert_eq!(review, "commit 1234567: Tidy UI colors");
                saw_entered_review_mode = true;
                break;
            }
            _ => continue,
        }
    }
    assert!(
        saw_entered_review_mode,
        "did not observe enteredReviewMode item"
    );

    // Confirm we see the ExitedReviewMode marker (with review text)
    // on the same turn. Ignore any other items the stream surfaces.
    let mut review_body: Option<String> = None;
    for _ in 0..10 {
        let completed: ItemCompletedNotification = timeout(
            DEFAULT_READ_TIMEOUT,
            mcp.read_notification("item/completed"),
        )
        .await??;
        match completed.item.payload {
            ThreadItemPayload::ExitedReviewMode { review, .. } => {
                assert_eq!(completed.turn_id, turn_id);
                review_body = Some(review);
                break;
            }
            _ => continue,
        }
    }

    let review = review_body.expect("did not observe a code review item");
    assert!(review.contains("Prefer Stylize helpers"));
    assert!(review.contains("/tmp/file.rs:10-20"));
    assert!(
        !mcp.pending_notification_methods()
            .iter()
            .any(|method| method == "deprecationNotice"),
        "inline reviews should not emit a deprecation notice"
    );

    Ok(())
}

#[tokio::test]
#[ignore = "TODO(owenlin0): flaky"]
async fn review_start_exec_approval_item_id_matches_command_execution_item() -> Result<()> {
    let responses = vec![
        create_command_execution_sse_response(
            vec![
                "git".to_string(),
                "rev-parse".to_string(),
                "HEAD".to_string(),
            ],
            /*workdir*/ None,
            Some(5000),
            "review-call-1",
        )?,
        create_final_assistant_message_sse_response("done")?,
    ];
    let server = create_mock_responses_server_sequence(responses).await;

    let codex_home = TempDir::new()?;
    MockResponsesConfig::new(&server.uri())
        .with_provider_name("Mock provider")
        .with_approval_policy("on-request")
        .disable_feature(Feature::ShellSnapshot)
        .write(codex_home.path())?;

    let mut mcp = TestAppServer::builder()
        .with_codex_home(codex_home.path())
        .build_initialized()
        .await?;
    let thread_id = start_default_thread(&mut mcp).await?;
    let ReviewStartResponse { turn, .. } = mcp
        .request(|request_id| ClientRequest::ReviewStart {
            request_id,
            params: ReviewStartParams {
                thread_id,
                delivery: Some(ReviewDelivery::Inline),
                target: ReviewTarget::Commit {
                    sha: "1234567deadbeef".to_string(),
                    title: Some("Check review approvals".to_string()),
                },
            },
        })
        .await?;
    let turn_id = turn.id.clone();
    assert_eq!(turn.items_view, TurnItemsView::NotLoaded);
    assert_eq!(
        turn.items,
        [ThreadItem::untimed(ThreadItemPayload::UserMessage {
            id: turn_id.clone(),
            client_id: None,
            content: vec![V2UserInput::Text {
                text: "commit 1234567: Check review approvals".to_string(),
                text_elements: Vec::new(),
            }],
        })]
    );

    let server_req = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_request_message(),
    )
    .await??;
    let ServerRequest::CommandExecutionRequestApproval { request_id, params } = server_req else {
        panic!("expected CommandExecutionRequestApproval request");
    };
    assert_eq!(params.item_id, "review-call-1");
    assert_eq!(params.turn_id, turn_id);

    let mut command_item_id = None;
    for _ in 0..10 {
        let started: ItemStartedNotification =
            timeout(DEFAULT_READ_TIMEOUT, mcp.read_notification("item/started")).await??;
        if let ThreadItemPayload::CommandExecution { id, .. } = started.item.payload {
            command_item_id = Some(id);
            break;
        }
    }
    let command_item_id = command_item_id.expect("did not observe command execution item");
    assert_eq!(command_item_id, params.item_id);

    mcp.send_response(
        request_id,
        serde_json::json!({ "decision": codex_protocol::protocol::ReviewDecision::Approved }),
    )
    .await?;
    timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_notification_message("turn/completed"),
    )
    .await??;

    Ok(())
}

#[tokio::test]
async fn review_start_rejects_empty_base_branch() -> Result<()> {
    let server = create_mock_responses_server_repeating_assistant("Done").await;
    let codex_home = TempDir::new()?;
    create_config_toml(codex_home.path(), &server.uri())?;

    let mut mcp = TestAppServer::builder()
        .with_codex_home(codex_home.path())
        .build_initialized()
        .await?;
    let thread_id = start_default_thread(&mut mcp).await?;

    let request_id = mcp
        .send_review_start_request(ReviewStartParams {
            thread_id,
            delivery: Some(ReviewDelivery::Inline),
            target: ReviewTarget::BaseBranch {
                branch: "   ".to_string(),
            },
        })
        .await?;
    let error: JSONRPCError = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_error_message(RequestId::Integer(request_id)),
    )
    .await??;
    assert_eq!(error.error.code, INVALID_REQUEST_ERROR_CODE);
    assert!(
        error.error.message.contains("branch must not be empty"),
        "unexpected message: {}",
        error.error.message
    );

    Ok(())
}

#[tokio::test]
async fn review_start_rejects_empty_commit_sha() -> Result<()> {
    let server = create_mock_responses_server_repeating_assistant("Done").await;
    let codex_home = TempDir::new()?;
    create_config_toml(codex_home.path(), &server.uri())?;

    let mut mcp = TestAppServer::builder()
        .with_codex_home(codex_home.path())
        .build_initialized()
        .await?;
    let thread_id = start_default_thread(&mut mcp).await?;

    let request_id = mcp
        .send_review_start_request(ReviewStartParams {
            thread_id,
            delivery: Some(ReviewDelivery::Inline),
            target: ReviewTarget::Commit {
                sha: "\t".to_string(),
                title: None,
            },
        })
        .await?;
    let error: JSONRPCError = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_error_message(RequestId::Integer(request_id)),
    )
    .await??;
    assert_eq!(error.error.code, INVALID_REQUEST_ERROR_CODE);
    assert!(
        error.error.message.contains("sha must not be empty"),
        "unexpected message: {}",
        error.error.message
    );

    Ok(())
}

#[tokio::test]
async fn review_start_rejects_empty_custom_instructions() -> Result<()> {
    let server = create_mock_responses_server_repeating_assistant("Done").await;
    let codex_home = TempDir::new()?;
    create_config_toml(codex_home.path(), &server.uri())?;

    let mut mcp = TestAppServer::builder()
        .with_codex_home(codex_home.path())
        .build_initialized()
        .await?;
    let thread_id = start_default_thread(&mut mcp).await?;

    let request_id = mcp
        .send_review_start_request(ReviewStartParams {
            thread_id,
            delivery: Some(ReviewDelivery::Inline),
            target: ReviewTarget::Custom {
                instructions: "\n\n".to_string(),
            },
        })
        .await?;
    let error: JSONRPCError = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_error_message(RequestId::Integer(request_id)),
    )
    .await??;
    assert_eq!(error.error.code, INVALID_REQUEST_ERROR_CODE);
    assert!(
        error
            .error
            .message
            .contains("instructions must not be empty"),
        "unexpected message: {}",
        error.error.message
    );

    Ok(())
}

async fn start_default_thread(mcp: &mut TestAppServer) -> Result<String> {
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
    Ok(thread.id)
}

fn create_config_toml(codex_home: &std::path::Path, server_uri: &str) -> std::io::Result<()> {
    MockResponsesConfig::new(server_uri)
        .with_provider_name("Mock provider")
        .disable_feature(Feature::ShellSnapshot)
        .write(codex_home)
}

/// #2015: runs a detached review of a thread and returns the parent's and the
/// review's model requests.
async fn run_detached_review(
    mcp: &mut TestAppServer,
    response_mock: &core_test_support::responses::ResponseMock,
    thread_id: &str,
) -> Result<(
    core_test_support::responses::ResponsesRequest,
    core_test_support::responses::ResponsesRequest,
)> {
    let ReviewStartResponse { .. } = mcp
        .request(|request_id| ClientRequest::ReviewStart {
            request_id,
            params: ReviewStartParams {
                thread_id: thread_id.to_string(),
                delivery: Some(ReviewDelivery::Detached),
                target: ReviewTarget::Custom {
                    instructions: "detached review".to_string(),
                },
            },
        })
        .await?;
    timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_notification_message("turn/completed"),
    )
    .await??;
    let mut requests = response_mock.requests();
    assert_eq!(requests.len(), 2);
    let review = requests.pop().expect("review request");
    let parent = requests.pop().expect("parent request");
    Ok((parent, review))
}

fn two_turn_sse() -> Vec<String> {
    vec![
        responses::sse(vec![
            responses::ev_response_created("materialize-response"),
            responses::ev_assistant_message("materialize-message", "materialized"),
            responses::ev_completed("materialize-response"),
        ]),
        responses::sse(vec![
            responses::ev_response_created("review-response"),
            responses::ev_assistant_message("review-message", "No findings."),
            responses::ev_completed("review-response"),
        ]),
    ]
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

/// #2015: a detached review runs with the parent thread's current sandbox
/// and approval policy (set at thread start and changed by a later turn),
/// not with the weaker ones the server started with.
#[tokio::test]
async fn issue_2015_detached_review_inherits_the_parent_threads_current_settings() -> Result<()> {
    const WEAK_SANDBOX: &str = "`sandbox_mode` is `danger-full-access`";
    const WEAK_APPROVALS: &str = "Approval policy is currently never";
    let server = responses::start_mock_server().await;
    let response_mock = responses::mount_sse_sequence(&server, two_turn_sse()).await;
    let codex_home = TempDir::new()?;
    MockResponsesConfig::new(&server.uri())
        .with_provider_name("Mock provider")
        .with_approval_policy("never")
        .with_sandbox_mode("danger-full-access")
        .disable_feature(Feature::ShellSnapshot)
        .write(codex_home.path())?;
    let mut mcp = TestAppServer::builder()
        .with_codex_home(codex_home.path())
        .build_initialized()
        .await?;

    let ThreadStartResponse { thread, .. } = mcp
        .start_thread(ThreadStartParams {
            model: Some("mock-model".to_string()),
            sandbox: Some(SandboxMode::ReadOnly),
            ..Default::default()
        })
        .await?;
    timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_notification_message("thread/started"),
    )
    .await??;
    let _: TurnStartResponse = mcp
        .request(|request_id| ClientRequest::TurnStart {
            request_id,
            params: TurnStartParams {
                thread_id: thread.id.clone(),
                client_user_message_id: None,
                input: vec![V2UserInput::Text {
                    text: "materialize rollout".to_string(),
                    text_elements: Vec::new(),
                }],
                approval_policy: Some(AskForApproval::UnlessTrusted),
                ..Default::default()
            },
        })
        .await?;
    timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_notification_message("turn/completed"),
    )
    .await??;

    let (parent, review) = run_detached_review(&mut mcp, &response_mock, &thread.id).await?;
    for (name, request) in [("parent", &parent), ("review", &review)] {
        assert!(
            request.body_contains_text("`sandbox_mode` is `read-only`"),
            "{name}"
        );
        assert!(!request.body_contains_text(WEAK_SANDBOX), "{name}");
        assert!(!request.body_contains_text(WEAK_APPROVALS), "{name}");
    }
    assert_eq!(sandbox_tag(&review), sandbox_tag(&parent));
    Ok(())
}

/// #2015: a Windows sandbox set up after the server started (setup saves
/// `[windows] sandbox`; a new thread loads it) reaches a detached review of
/// a thread that runs with it.
#[tokio::test]
async fn issue_2015_detached_review_keeps_a_windows_sandbox_set_up_after_start() -> Result<()> {
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

    let thread_id = start_default_thread(&mut mcp).await?;
    materialize_thread_rollout(&mut mcp, &thread_id).await?;
    let (parent, review) = run_detached_review(&mut mcp, &response_mock, &thread_id).await?;
    if cfg!(windows) {
        assert_ne!(sandbox_tag(&parent), "none", "the thread has the sandbox");
    }
    assert_eq!(sandbox_tag(&review), sandbox_tag(&parent));
    Ok(())
}
