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

use crate::approvals::PendingApproval;
use crate::approvals::PendingApprovalKind;
use crate::session::SessionStore;

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
    assert_eq!(owner_chat.chat_id, ChatId(10));
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
