use std::collections::HashMap;
use std::sync::Arc;

use codex_app_server_client::DEFAULT_IN_PROCESS_CHANNEL_CAPACITY;
use codex_app_server_client::EnvironmentManager;
use codex_app_server_client::InProcessAppServerClient;
use codex_app_server_client::InProcessClientStartArgs;
use codex_app_server_protocol::ServerNotification;
use codex_app_server_protocol::Turn;
use codex_app_server_protocol::TurnCompletedNotification;
use codex_app_server_protocol::TurnStatus;
use codex_login::AuthManager;
use codex_protocol::protocol::SessionSource;
use pretty_assertions::assert_eq;
use teloxide::types::ChatId;

use super::BridgeRuntime;
use crate::bot::failure_tests::notice_bot;
use crate::bridge::RequestIdSequencer;
use crate::conversation::ConversationKey;
use crate::session::SessionStore;

struct TurnCompletedOutcome {
    notice_text: String,
    handled: bool,
    turn_id: Option<String>,
    status_line: String,
}

/// A chat with a running turn receives `turn/completed`; the one-line notice
/// goes to a loopback stand-in for the Bot API (no Telegram credentials).
async fn complete_turn(name: &str, status: TurnStatus, fail_notice: bool) -> TurnCompletedOutcome {
    let folder = std::env::temp_dir().join(format!(
        "telegram-turn-completed-{name}-{}",
        std::process::id()
    ));
    let (codex_home, cwd) = (folder.join("home"), folder.join("cwd"));
    for dir in [&codex_home, &cwd] {
        std::fs::create_dir_all(dir).unwrap();
    }
    let config = Arc::new(
        codex_core::config::ConfigBuilder::default()
            .loader_overrides(codex_config::LoaderOverrides::without_managed_config_for_tests())
            .codex_home(codex_home.clone())
            .harness_overrides(codex_core::config::ConfigOverrides {
                cwd: Some(cwd),
                ..Default::default()
            })
            .build()
            .await
            .unwrap(),
    );
    let state_db = codex_core::init_state_db(config.as_ref()).await;
    let client = InProcessAppServerClient::start(InProcessClientStartArgs {
        arg0_paths: codex_arg0::Arg0DispatchPaths::default(),
        config: Arc::clone(&config),
        cli_overrides: Vec::new(),
        loader_overrides: codex_config::LoaderOverrides::default(),
        strict_config: false,
        cloud_config_bundle: codex_config::CloudConfigBundleLoader::default(),
        feedback: codex_feedback::CodexFeedback::new(),
        log_db: None,
        state_db,
        environment_manager: Arc::new(EnvironmentManager::default_for_tests()),
        config_warnings: Vec::new(),
        session_source: SessionSource::Custom("telegram".to_string()),
        enable_codex_api_key_env: false,
        client_name: "codex_telegram_test".to_string(),
        client_version: "0.0.0-test".to_string(),
        experimental_api: true,
        mcp_server_openai_form_elicitation: false,
        opt_out_notification_methods: Vec::new(),
        channel_capacity: DEFAULT_IN_PROCESS_CHANNEL_CAPACITY,
    })
    .await
    .unwrap();
    let auth_manager =
        AuthManager::shared_from_config(config.as_ref(), /*enable_codex_api_key_env*/ false).await;
    let sessions = SessionStore::load(&codex_home).await.unwrap();
    let conversation = ConversationKey::from(ChatId(99));
    sessions
        .set_thread(conversation, "thread-1".to_string())
        .await
        .unwrap();
    sessions
        .set_turn(
            conversation,
            "turn-1".to_string(),
            /*actor_user_id*/ None,
        )
        .await;
    let (bot, notice) = notice_bot(fail_notice).await;
    let mut runtime = BridgeRuntime {
        bot,
        client,
        config,
        auth_manager,
        sessions: Arc::clone(&sessions),
        request_ids: RequestIdSequencer::new(),
        pending_inputs: HashMap::new(),
        last_successful_contact_at: HashMap::new(),
        last_errors: HashMap::new(),
        last_token_usage: HashMap::new(),
    };

    let handled = runtime
        .handle_notification(ServerNotification::TurnCompleted(
            TurnCompletedNotification {
                thread_id: "thread-1".to_string(),
                turn: Turn {
                    id: "turn-1".to_string(),
                    items: Vec::new(),
                    items_view: Default::default(),
                    status,
                    error: None,
                    started_at: None,
                    completed_at: None,
                    duration_ms: None,
                },
            },
        ))
        .await
        .is_ok();
    let payload = notice.await.unwrap();
    let outcome = TurnCompletedOutcome {
        notice_text: payload["text"].as_str().unwrap().to_string(),
        handled,
        turn_id: sessions.turn_id(conversation).await,
        status_line: runtime
            .runtime_status_text(conversation)
            .await
            .lines()
            .next()
            .unwrap_or_default()
            .to_string(),
    };
    runtime.client.shutdown().await.unwrap();
    std::fs::remove_dir_all(folder).unwrap();
    outcome
}

#[tokio::test(flavor = "multi_thread")]
async fn interrupted_turn_is_cleared_when_the_notice_is_sent() {
    let outcome = complete_turn(
        "interrupted-sent",
        TurnStatus::Interrupted,
        /*fail_notice*/ false,
    )
    .await;
    assert_eq!(outcome.notice_text, "Turn interrupted.");
    assert!(outcome.handled);
    assert_eq!(outcome.turn_id, None);
    assert_eq!(outcome.status_line, "State: Idle");
}

#[tokio::test(flavor = "multi_thread")]
async fn interrupted_turn_is_cleared_when_the_notice_is_refused() {
    let outcome = complete_turn(
        "interrupted-refused",
        TurnStatus::Interrupted,
        /*fail_notice*/ true,
    )
    .await;
    assert_eq!(outcome.notice_text, "Turn interrupted.");
    // The send error is still reported to the event loop, which logs it.
    assert!(!outcome.handled);
    assert_eq!(outcome.turn_id, None);
    assert_eq!(outcome.status_line, "State: Idle");
}

#[tokio::test(flavor = "multi_thread")]
async fn failed_turn_is_cleared_when_the_notice_is_refused() {
    let outcome = complete_turn(
        "failed-refused",
        TurnStatus::Failed,
        /*fail_notice*/ true,
    )
    .await;
    assert_eq!(outcome.notice_text, "Turn failed.");
    assert!(!outcome.handled);
    assert_eq!(outcome.turn_id, None);
    assert_eq!(outcome.status_line, "State: Idle");
}
