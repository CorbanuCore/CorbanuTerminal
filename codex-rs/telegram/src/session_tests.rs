use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use std::time::Instant;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use codex_app_server_protocol::CommandExecutionRequestApprovalParams;
use codex_app_server_protocol::RequestId;
use pretty_assertions::assert_eq;
use teloxide::types::ChatId;
use teloxide::types::MessageId;
use teloxide::types::ThreadId;
use teloxide::types::UserId;

use crate::approvals::PendingApproval;
use crate::approvals::PendingApprovalKind;
use crate::conversation::ConversationKey;
use crate::session::SessionStore;
use crate::session::approval_expired;

#[tokio::test]
async fn old_format_state_file_without_chat_settings_still_loads() {
    let codex_home = unique_temp_dir("codex-telegram-session-old-format");
    let telegram_dir = codex_home.join("telegram");
    fs::create_dir_all(&telegram_dir).expect("create telegram state dir");
    fs::write(
        telegram_dir.join("state.json"),
        r#"{
  "chats": {
    "10": {
      "thread_id": "thread-old",
      "last_delivered_item_id": "item-old"
    }
  }
}
"#,
    )
    .expect("write old state");

    let sessions = SessionStore::load(&codex_home)
        .await
        .expect("load old-format state");

    assert_eq!(
        sessions.thread_id(ChatId(10)).await,
        Some("thread-old".to_string())
    );
    assert_eq!(
        sessions.last_delivered_item_id("thread-old").await,
        Some("item-old".to_string())
    );
    assert_eq!(sessions.model(ChatId(10)).await, None);
    assert_eq!(sessions.model_provider(ChatId(10)).await, None);
    assert_eq!(sessions.approval_policy(ChatId(10)).await, None);

    fs::remove_dir_all(codex_home).expect("remove codex home");
}

#[tokio::test]
async fn pending_approval_can_only_be_taken_by_owner_chat() {
    let codex_home = unique_temp_dir("codex-telegram-session");
    fs::create_dir_all(&codex_home).expect("create codex home");
    let sessions = SessionStore::load(&codex_home)
        .await
        .expect("load session store");
    let request_id = RequestId::Integer(3);
    sessions
        .insert_pending_approval(
            ChatId(10),
            PendingApproval {
                request_id: request_id.clone(),
                kind: PendingApprovalKind::Command(command_params("thread")),
            },
        )
        .await;

    let wrong_chat = sessions
        .take_pending_approval(ChatId(20), &request_id)
        .await;
    let owner_chat = sessions
        .take_pending_approval(ChatId(10), &request_id)
        .await
        .expect("owner can take approval");

    assert_eq!(wrong_chat, None);
    assert_eq!(owner_chat.conversation.chat_id, ChatId(10));
    assert_eq!(owner_chat.approval.request_id, request_id);

    fs::remove_dir_all(codex_home).expect("remove codex home");
}

#[tokio::test]
async fn clear_turn_for_thread_removes_pending_approvals() {
    let codex_home = unique_temp_dir("codex-telegram-session-clear");
    fs::create_dir_all(&codex_home).expect("create codex home");
    let sessions = SessionStore::load(&codex_home)
        .await
        .expect("load session store");
    let request_id = RequestId::Integer(4);
    sessions
        .set_thread(ChatId(10), "thread".to_string())
        .await
        .expect("set thread");
    sessions
        .insert_pending_approval(
            ChatId(10),
            PendingApproval {
                request_id: request_id.clone(),
                kind: PendingApprovalKind::Command(command_params("thread")),
            },
        )
        .await;

    sessions.clear_turn_for_thread("thread").await;

    assert_eq!(
        sessions
            .take_pending_approval(ChatId(10), &request_id)
            .await,
        None
    );

    fs::remove_dir_all(codex_home).expect("remove codex home");
}

#[tokio::test]
async fn replacement_thread_overwrites_stale_persisted_mapping() {
    let codex_home = unique_temp_dir("codex-telegram-session-replacement");
    fs::create_dir_all(&codex_home).expect("create codex home");
    let conversation = ConversationKey::new(ChatId(42), None);
    let sessions = SessionStore::load(&codex_home).await.unwrap();
    sessions
        .set_thread(conversation, "stale-thread".to_string())
        .await
        .unwrap();

    let restarted = SessionStore::load(&codex_home).await.unwrap();
    assert_eq!(
        restarted.thread_id(conversation).await.as_deref(),
        Some("stale-thread")
    );
    assert!(!restarted.thread_loaded(conversation).await);

    restarted
        .set_thread(conversation, "replacement-thread".to_string())
        .await
        .unwrap();
    assert_eq!(
        restarted.thread_id(conversation).await.as_deref(),
        Some("replacement-thread")
    );
    assert!(restarted.thread_loaded(conversation).await);

    let reloaded = SessionStore::load(&codex_home).await.unwrap();
    assert_eq!(
        reloaded.thread_id(conversation).await.as_deref(),
        Some("replacement-thread")
    );
    fs::remove_dir_all(codex_home).expect("remove codex home");
}

#[tokio::test]
async fn delivered_item_marker_survives_reload() {
    let codex_home = unique_temp_dir("codex-telegram-session-delivered");
    fs::create_dir_all(&codex_home).expect("create codex home");
    let sessions = SessionStore::load(&codex_home)
        .await
        .expect("load session store");
    sessions
        .set_thread(ChatId(10), "thread".to_string())
        .await
        .expect("set thread");
    sessions.mark_item_delivered("thread", "item-2").await;

    let reloaded = SessionStore::load(&codex_home)
        .await
        .expect("reload session store");

    assert_eq!(
        reloaded.last_delivered_item_id("thread").await,
        Some("item-2".to_string())
    );
    assert_eq!(reloaded.item_delivered("thread", "item-2").await, true);

    fs::remove_dir_all(codex_home).expect("remove codex home");
}

#[tokio::test]
async fn stream_edit_suppression_expires() {
    let codex_home = unique_temp_dir("codex-telegram-session-suppress");
    fs::create_dir_all(&codex_home).expect("create codex home");
    let sessions = SessionStore::load(&codex_home)
        .await
        .expect("load session store");
    let now = Instant::now();

    sessions
        .suppress_stream_edits_until(ChatId(10), now + Duration::from_secs(5))
        .await;

    assert_eq!(
        sessions.stream_edits_suppressed(ChatId(10), now).await,
        true
    );
    assert_eq!(
        sessions
            .stream_edits_suppressed(ChatId(10), now + Duration::from_secs(5))
            .await,
        false
    );
    assert_eq!(
        sessions.stream_edits_suppressed(ChatId(10), now).await,
        false
    );

    fs::remove_dir_all(codex_home).expect("remove codex home");
}

#[tokio::test]
async fn topics_in_one_group_have_isolated_threads_and_approvals() {
    let codex_home = unique_temp_dir("codex-telegram-session-topics");
    fs::create_dir_all(&codex_home).unwrap();
    let sessions = SessionStore::load(&codex_home).await.unwrap();
    let topic_a = ConversationKey::new(ChatId(-1_001), Some(ThreadId(MessageId(11))));
    let topic_b = ConversationKey::new(ChatId(-1_001), Some(ThreadId(MessageId(12))));
    sessions
        .set_thread(topic_a, "thread-a".into())
        .await
        .unwrap();
    sessions
        .set_thread(topic_b, "thread-b".into())
        .await
        .unwrap();
    let request_id = RequestId::Integer(9);
    sessions
        .insert_pending_approval(
            topic_a,
            PendingApproval {
                request_id: request_id.clone(),
                kind: PendingApprovalKind::Command(command_params("thread-a")),
            },
        )
        .await;

    assert_eq!(
        sessions.thread_id(topic_a).await.as_deref(),
        Some("thread-a")
    );
    assert_eq!(
        sessions.thread_id(topic_b).await.as_deref(),
        Some("thread-b")
    );
    assert_eq!(
        sessions.take_pending_approval(topic_b, &request_id).await,
        None
    );
    assert!(
        sessions
            .take_pending_approval(topic_a, &request_id)
            .await
            .is_some()
    );

    let reloaded = SessionStore::load(&codex_home).await.unwrap();
    assert_eq!(
        reloaded.thread_id(topic_a).await.as_deref(),
        Some("thread-a")
    );
    assert_eq!(
        reloaded.thread_id(topic_b).await.as_deref(),
        Some("thread-b")
    );
    fs::remove_dir_all(codex_home).unwrap();
}

#[tokio::test]
async fn approval_captures_the_user_who_started_the_turn() {
    let codex_home = unique_temp_dir("codex-telegram-session-principal");
    fs::create_dir_all(&codex_home).unwrap();
    let sessions = SessionStore::load(&codex_home).await.unwrap();
    let conversation = ConversationKey::new(ChatId(-1_001), Some(ThreadId(MessageId(3))));
    let request_id = RequestId::Integer(10);
    sessions
        .set_turn(conversation, "turn".into(), Some(UserId(77)))
        .await;
    sessions
        .insert_pending_approval(
            conversation,
            PendingApproval {
                request_id: request_id.clone(),
                kind: PendingApprovalKind::Command(command_params("thread")),
            },
        )
        .await;

    let pending = sessions
        .pending_approval(conversation, &request_id)
        .await
        .unwrap();
    assert_eq!(pending.actor_user_id, Some(UserId(77)));
    fs::remove_dir_all(codex_home).unwrap();
}

#[test]
fn approval_expires_after_the_bounded_callback_window() {
    let created_at = Instant::now();
    assert!(!approval_expired(
        created_at,
        created_at + Duration::from_secs(15 * 60)
    ));
    assert!(approval_expired(
        created_at,
        created_at + Duration::from_secs(15 * 60 + 1)
    ));
}

#[tokio::test]
async fn racing_callbacks_can_consume_an_approval_only_once() {
    let codex_home = unique_temp_dir("codex-telegram-session-race");
    fs::create_dir_all(&codex_home).unwrap();
    let sessions = SessionStore::load(&codex_home).await.unwrap();
    let request_id = RequestId::Integer(11);
    sessions
        .insert_pending_approval(
            ChatId(10),
            PendingApproval {
                request_id: request_id.clone(),
                kind: PendingApprovalKind::Command(command_params("thread")),
            },
        )
        .await;

    let first = {
        let sessions = sessions.clone();
        let request_id = request_id.clone();
        tokio::spawn(async move {
            sessions
                .take_pending_approval(ChatId(10), &request_id)
                .await
        })
    };
    let second = {
        let sessions = sessions.clone();
        let request_id = request_id.clone();
        tokio::spawn(async move {
            sessions
                .take_pending_approval(ChatId(10), &request_id)
                .await
        })
    };
    let consumed =
        usize::from(first.await.unwrap().is_some()) + usize::from(second.await.unwrap().is_some());

    assert_eq!(consumed, 1);
    fs::remove_dir_all(codex_home).unwrap();
}

fn command_params(thread_id: &str) -> CommandExecutionRequestApprovalParams {
    CommandExecutionRequestApprovalParams {
        thread_id: thread_id.to_string(),
        turn_id: "turn".to_string(),
        item_id: "item".to_string(),
        started_at_ms: 1,
        approval_id: None,
        environment_id: None,
        reason: None,
        network_approval_context: None,
        command: Some("true".to_string()),
        cwd: None,
        command_actions: None,
        additional_permissions: None,
        proposed_execpolicy_amendment: None,
        proposed_network_policy_amendments: None,
        available_decisions: None,
    }
}

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    std::env::temp_dir().join(format!("{prefix}-{}-{nanos}", std::process::id()))
}
