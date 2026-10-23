//! PF-84-S03 follow-up: a worker a client (the TUI) starts with
//! `thread/spawnAgent` inherits its parent's live named account, and a fork
//! runs on its source thread's account (#555), also when it is cut at an
//! earlier turn (#588).

use anyhow::Result;
use app_test_support::TestAppServer;
use app_test_support::to_response;
use codex_app_server_protocol::JSONRPCResponse;
use codex_app_server_protocol::RequestId;
use codex_app_server_protocol::ThreadForkParams;
use codex_app_server_protocol::ThreadForkResponse;
use codex_app_server_protocol::ThreadHistoryMode;
use codex_app_server_protocol::ThreadResumeParams;
use codex_app_server_protocol::ThreadResumeResponse;
use codex_app_server_protocol::ThreadSpawnAgentParams;
use codex_app_server_protocol::ThreadSpawnAgentResponse;
use codex_app_server_protocol::ThreadStartParams;
use codex_app_server_protocol::ThreadStartResponse;
use codex_app_server_protocol::TurnStartParams;
use codex_app_server_protocol::UserInput;
use core_test_support::responses;
use pretty_assertions::assert_eq;
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

// PF-84 (#555): `thread/fork` (TUI `/fork`, a prompt-edit fork, `/side`)
// runs on the account its source thread runs on, not on
// `[provider_accounts]` or `default`.

const PERSONAL_KEY: &str = "pf84-555-personal-account-key";
const OTHER_KEY: &str = "pf84-555-other-account-key";

fn pong(response_id: &str) -> String {
    responses::sse(vec![
        responses::ev_response_created(response_id),
        responses::ev_assistant_message(&format!("msg-{response_id}"), "pong"),
        responses::ev_completed(response_id),
    ])
}

/// A home whose `[provider_accounts]` selects `personal`, with `work`,
/// `personal` and `other` enrolled and no default key.
fn fork_home(server_uri: &str) -> Result<TempDir> {
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

[provider_accounts]
mock_provider = "personal"

[model_providers.mock_provider]
name = "Mock provider for test"
base_url = "{server_uri}/v1"
wire_api = "responses"
env_key = "PF84_S03_UNSET_DEFAULT_KEY"
request_max_retries = 0
stream_max_retries = 0
"#
        ),
    )?;
    let vault = codex_vault::Vault::new(codex_home.path().to_path_buf());
    for (name, key) in [
        ("work", WORK_KEY),
        ("personal", PERSONAL_KEY),
        ("other", OTHER_KEY),
    ] {
        vault.write_provider_account(
            "mock_provider",
            &codex_vault::ProviderAccountName::parse(name)?,
            codex_vault::ProviderAccountKind::ApiKey,
            key,
        )?;
    }
    Ok(codex_home)
}

async fn start_app(codex_home: &TempDir) -> Result<TestAppServer> {
    let mut app = TestAppServer::builder()
        .with_codex_home(codex_home.path())
        .without_auto_env()
        .build()
        .await?;
    timeout(DEFAULT_TIMEOUT, app.initialize()).await??;
    Ok(app)
}

async fn say_pong(app: &mut TestAppServer, thread_id: String) -> Result<()> {
    timeout(
        DEFAULT_TIMEOUT,
        app.start_turn_and_wait_for_completion(TurnStartParams {
            thread_id,
            input: vec![UserInput::Text {
                text: "say pong".to_string(),
                text_elements: Vec::new(),
            }],
            ..Default::default()
        }),
    )
    .await??;
    Ok(())
}

/// A thread started on `work` that has answered one turn.
async fn work_thread(app: &mut TestAppServer) -> Result<String> {
    let request = app
        .send_thread_start_request(ThreadStartParams {
            model: Some("mock-model".to_string()),
            provider_account: Some("work".to_string()),
            ..Default::default()
        })
        .await?;
    let response = timeout(
        DEFAULT_TIMEOUT,
        app.read_stream_until_response_message(RequestId::Integer(request)),
    )
    .await??;
    let ThreadStartResponse { thread, .. } = to_response::<ThreadStartResponse>(response)?;
    say_pong(app, thread.id.clone()).await?;
    Ok(thread.id)
}

async fn fork(
    app: &mut TestAppServer,
    thread_id: String,
    provider_account: Option<&str>,
) -> Result<ThreadForkResponse> {
    let request = app
        .send_thread_fork_request(ThreadForkParams {
            thread_id,
            provider_account: provider_account.map(str::to_string),
            ..Default::default()
        })
        .await?;
    let response = timeout(
        DEFAULT_TIMEOUT,
        app.read_stream_until_response_message(RequestId::Integer(request)),
    )
    .await??;
    to_response::<ThreadForkResponse>(response)
}

fn bearers(response_mock: &responses::ResponseMock) -> Vec<Option<String>> {
    response_mock
        .requests()
        .iter()
        .map(|request| request.header("authorization"))
        .collect()
}

#[tokio::test]
async fn fork_of_a_loaded_thread_runs_on_its_account() -> Result<()> {
    let server = responses::start_mock_server().await;
    let response_mock =
        responses::mount_sse_sequence(&server, vec![pong("resp-1"), pong("resp-2")]).await;
    let codex_home = fork_home(&server.uri())?;
    let mut app = start_app(&codex_home).await?;

    let source = work_thread(&mut app).await?;
    let forked = fork(&mut app, source, /*provider_account*/ None).await?;
    assert_eq!(
        forked.provider_account.as_deref(),
        Some("mock_provider:work")
    );
    say_pong(&mut app, forked.thread.id).await?;

    assert_eq!(
        bearers(&response_mock),
        vec![
            Some(format!("Bearer {WORK_KEY}")),
            Some(format!("Bearer {WORK_KEY}")),
        ]
    );
    Ok(())
}

#[tokio::test]
async fn fork_of_a_stored_thread_runs_on_its_recorded_account() -> Result<()> {
    let server = responses::start_mock_server().await;
    let response_mock =
        responses::mount_sse_sequence(&server, vec![pong("resp-1"), pong("resp-2")]).await;
    let codex_home = fork_home(&server.uri())?;
    let mut app = start_app(&codex_home).await?;
    let source = work_thread(&mut app).await?;
    app.shutdown_gracefully().await?;

    // A new app server has not loaded the thread: the fork takes the account
    // its newest turn recorded.
    let mut app = start_app(&codex_home).await?;
    let forked = fork(&mut app, source, /*provider_account*/ None).await?;
    assert_eq!(
        forked.provider_account.as_deref(),
        Some("mock_provider:work")
    );
    say_pong(&mut app, forked.thread.id).await?;

    assert_eq!(
        bearers(&response_mock),
        vec![
            Some(format!("Bearer {WORK_KEY}")),
            Some(format!("Bearer {WORK_KEY}")),
        ]
    );
    Ok(())
}

#[tokio::test]
async fn an_explicit_fork_account_beats_the_source_account() -> Result<()> {
    let server = responses::start_mock_server().await;
    let response_mock =
        responses::mount_sse_sequence(&server, vec![pong("resp-1"), pong("resp-2")]).await;
    let codex_home = fork_home(&server.uri())?;
    let mut app = start_app(&codex_home).await?;

    // `other` is neither the source's account nor `[provider_accounts]`.
    let source = work_thread(&mut app).await?;
    let forked = fork(&mut app, source, Some("other")).await?;
    assert_eq!(
        forked.provider_account.as_deref(),
        Some("mock_provider:other")
    );
    say_pong(&mut app, forked.thread.id).await?;

    assert_eq!(
        bearers(&response_mock),
        vec![
            Some(format!("Bearer {WORK_KEY}")),
            Some(format!("Bearer {OTHER_KEY}")),
        ]
    );
    Ok(())
}

/// #588 (Travis, 2026-10-13, option A): a fork cut at an earlier turn of a
/// stored paginated thread runs on the source's current account, as a resume
/// does, not on the account that turn ran on.
#[tokio::test]
async fn fork_of_an_earlier_turn_runs_on_the_source_current_account() -> Result<()> {
    let server = responses::start_mock_server().await;
    let response_mock = responses::mount_sse_sequence(
        &server,
        vec![
            pong("resp-1"),
            pong("resp-2"),
            pong("resp-3"),
            pong("resp-4"),
        ],
    )
    .await;
    let codex_home = fork_home(&server.uri())?;

    // Turn 1 on `work`.
    let mut app = start_app(&codex_home).await?;
    let request = app
        .send_thread_start_request(ThreadStartParams {
            model: Some("mock-model".to_string()),
            provider_account: Some("work".to_string()),
            history_mode: Some(ThreadHistoryMode::Paginated),
            ..Default::default()
        })
        .await?;
    let response = timeout(
        DEFAULT_TIMEOUT,
        app.read_stream_until_response_message(RequestId::Integer(request)),
    )
    .await??;
    let ThreadStartResponse { thread, .. } = to_response::<ThreadStartResponse>(response)?;
    let source = thread.id;
    let first_turn = timeout(
        DEFAULT_TIMEOUT,
        app.start_turn_and_wait_for_completion(TurnStartParams {
            thread_id: source.clone(),
            input: vec![UserInput::Text {
                text: "say pong".to_string(),
                text_elements: Vec::new(),
            }],
            ..Default::default()
        }),
    )
    .await??
    .turn
    .id;
    app.shutdown_gracefully().await?;

    // The conversation moves to `other`; turn 2 runs there.
    let mut app = start_app(&codex_home).await?;
    let request = app
        .send_thread_resume_request(ThreadResumeParams {
            thread_id: source.clone(),
            provider_account: Some("other".to_string()),
            ..Default::default()
        })
        .await?;
    let response = timeout(
        DEFAULT_TIMEOUT,
        app.read_stream_until_response_message(RequestId::Integer(request)),
    )
    .await??;
    let resumed = to_response::<ThreadResumeResponse>(response)?;
    assert_eq!(
        resumed.provider_account.as_deref(),
        Some("mock_provider:other")
    );
    let second_turn = timeout(
        DEFAULT_TIMEOUT,
        app.start_turn_and_wait_for_completion(TurnStartParams {
            thread_id: source.clone(),
            input: vec![UserInput::Text {
                text: "say pong".to_string(),
                text_elements: Vec::new(),
            }],
            ..Default::default()
        }),
    )
    .await??
    .turn
    .id;
    app.shutdown_gracefully().await?;

    // A new app server has not loaded the thread. A fork through turn 1 and a
    // fork before turn 2 (both keep only turn 1, which ran on `work`) run on
    // `other`, the account the conversation runs on now.
    let mut app = start_app(&codex_home).await?;
    for (last_turn_id, before_turn_id) in [(Some(first_turn), None), (None, Some(second_turn))] {
        let request = app
            .send_thread_fork_request(ThreadForkParams {
                thread_id: source.clone(),
                last_turn_id,
                before_turn_id,
                ..Default::default()
            })
            .await?;
        let response = timeout(
            DEFAULT_TIMEOUT,
            app.read_stream_until_response_message(RequestId::Integer(request)),
        )
        .await??;
        let forked = to_response::<ThreadForkResponse>(response)?;
        assert_eq!(
            forked.provider_account.as_deref(),
            Some("mock_provider:other")
        );
        say_pong(&mut app, forked.thread.id).await?;
    }

    assert_eq!(
        bearers(&response_mock),
        vec![
            Some(format!("Bearer {WORK_KEY}")),
            Some(format!("Bearer {OTHER_KEY}")),
            Some(format!("Bearer {OTHER_KEY}")),
            Some(format!("Bearer {OTHER_KEY}")),
        ]
    );
    Ok(())
}

/// #588: the fork's own config decides whether named accounts apply, not the
/// app server's. The home turns them off after `work`'s turn; a fork that
/// turns them on for itself (request `config`) runs on the recorded `work`,
/// not on `[provider_accounts]`'s `personal`. With them off for the fork too,
/// the fork is refused and names the recorded account, as a resume is.
#[tokio::test]
async fn the_forks_own_config_decides_whether_its_recorded_account_applies() -> Result<()> {
    let server = responses::start_mock_server().await;
    let response_mock =
        responses::mount_sse_sequence(&server, vec![pong("resp-1"), pong("resp-2")]).await;
    let codex_home = fork_home(&server.uri())?;
    let mut app = start_app(&codex_home).await?;
    let source = work_thread(&mut app).await?;
    app.shutdown_gracefully().await?;
    let config_path = codex_home.path().join("config.toml");
    let config = std::fs::read_to_string(&config_path)?;
    std::fs::write(
        &config_path,
        config.replace("named_accounts = true", "named_accounts = false"),
    )?;

    let mut app = start_app(&codex_home).await?;
    let request = app
        .send_thread_fork_request(ThreadForkParams {
            thread_id: source.clone(),
            config: Some(std::collections::HashMap::from([(
                "features.named_accounts".to_string(),
                serde_json::Value::Bool(true),
            )])),
            ..Default::default()
        })
        .await?;
    let response = timeout(
        DEFAULT_TIMEOUT,
        app.read_stream_until_response_message(RequestId::Integer(request)),
    )
    .await??;
    let forked = to_response::<ThreadForkResponse>(response)?;
    assert_eq!(
        forked.provider_account.as_deref(),
        Some("mock_provider:work")
    );
    say_pong(&mut app, forked.thread.id).await?;
    assert_eq!(
        bearers(&response_mock),
        vec![
            Some(format!("Bearer {WORK_KEY}")),
            Some(format!("Bearer {WORK_KEY}")),
        ]
    );

    let request = app
        .send_thread_fork_request(ThreadForkParams {
            thread_id: source,
            ..Default::default()
        })
        .await?;
    let error = timeout(
        DEFAULT_TIMEOUT,
        app.read_stream_until_error_message(RequestId::Integer(request)),
    )
    .await??;
    assert!(
        error
            .error
            .message
            .contains("the thread's recorded account mock_provider:work: named accounts need"),
        "{}",
        error.error.message
    );
    assert_eq!(response_mock.requests().len(), 2);
    Ok(())
}
