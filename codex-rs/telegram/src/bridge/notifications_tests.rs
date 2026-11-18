use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use codex_app_server_protocol::AuthRecoveryNotification;
use codex_app_server_protocol::FileChangeRequestApprovalParams;
use codex_app_server_protocol::RequestId;
use codex_app_server_protocol::ServerNotification;
use codex_app_server_protocol::StrictReviewRequiredNotification;
use pretty_assertions::assert_eq;
use teloxide::types::ChatId;
use teloxide::types::MessageId;
use teloxide::types::ThreadId;
use teloxide::types::UserId;

use super::thread_status_notice;
use crate::approvals::PendingApproval;
use crate::approvals::PendingApprovalKind;
use crate::conversation::ConversationKey;
use crate::render::render_html_chunks;
use crate::session::SessionStore;

struct TestSessions {
    home: PathBuf,
    sessions: Arc<SessionStore>,
}

impl TestSessions {
    async fn new() -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let home = std::env::temp_dir().join(format!(
            "codex-telegram-notifications-{}-{nanos}",
            std::process::id()
        ));
        std::fs::create_dir_all(&home).unwrap();
        let sessions = SessionStore::load(&home).await.unwrap();
        Self { home, sessions }
    }
}

impl Drop for TestSessions {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.home);
    }
}

fn notices(thread_id: &str) -> [(ServerNotification, &'static str); 3] {
    let recovery = AuthRecoveryNotification {
        thread_id: thread_id.into(),
        turn_id: "turn".into(),
        provider: "provider".into(),
        message: "Reconnecting <provider> & keeping the turn.".into(),
    };
    [
        (
            ServerNotification::StrictReviewRequired(StrictReviewRequiredNotification {
                thread_id: thread_id.into(),
                turn_id: "turn".into(),
                started_at_ms: 1,
            }),
            "Warning: This request requires additional safety checks, some tool calls might take extra time",
        ),
        (
            ServerNotification::AuthRecoveryStarted(recovery.clone()),
            "Reconnecting <provider> & keeping the turn.",
        ),
        (
            ServerNotification::AuthRecoveryCompleted(AuthRecoveryNotification {
                message: "Recovered <provider> & continuing.".into(),
                ..recovery
            }),
            "✓ Recovered <provider> & continuing.",
        ),
    ]
}

#[tokio::test]
async fn status_notices_route_to_exact_chat_and_topic_with_server_messages() {
    let fixture = TestSessions::new().await;
    let conversations = [
        ("private", ConversationKey::from(ChatId(10))),
        (
            "topic-a",
            ConversationKey::new(ChatId(-1001), Some(ThreadId(MessageId(11)))),
        ),
        (
            "topic-b",
            ConversationKey::new(ChatId(-1001), Some(ThreadId(MessageId(12)))),
        ),
    ];
    for (thread_id, conversation) in conversations {
        fixture
            .sessions
            .set_thread(conversation, thread_id.into())
            .await
            .unwrap();
    }

    // Routing also works before a persisted conversation has been resumed.
    let sessions = SessionStore::load(&fixture.home).await.unwrap();
    for (thread_id, conversation) in conversations {
        for (notification, expected_message) in notices(thread_id) {
            assert_eq!(
                thread_status_notice(&sessions, notification).await,
                Some((conversation, expected_message.into()))
            );
        }
    }

    // The notice stays plain text until the existing send_text renderer escapes
    // it; server-supplied angle brackets cannot become Telegram HTML.
    for ((notification, _), expected_html) in notices("topic-a").into_iter().skip(1).zip([
        "Reconnecting &lt;provider&gt; &amp; keeping the turn.",
        "✓ Recovered &lt;provider&gt; &amp; continuing.",
    ]) {
        let (_, message) = thread_status_notice(&sessions, notification).await.unwrap();
        let html: Vec<_> = render_html_chunks(&message)
            .into_iter()
            .map(|chunk| chunk.html)
            .collect();
        assert_eq!(html, vec![expected_html.to_string()]);
    }
}

#[tokio::test]
async fn unknown_and_replaced_threads_do_not_receive_status_notices() {
    let fixture = TestSessions::new().await;
    let conversation = ConversationKey::from(ChatId(10));
    fixture
        .sessions
        .set_thread(conversation, "old-thread".into())
        .await
        .unwrap();
    fixture
        .sessions
        .set_thread(conversation, "new-thread".into())
        .await
        .unwrap();
    for thread_id in ["unknown-thread", "old-thread"] {
        for (notification, _) in notices(thread_id) {
            assert_eq!(
                thread_status_notice(&fixture.sessions, notification).await,
                None
            );
        }
    }
    for (notification, expected_message) in notices("new-thread") {
        assert_eq!(
            thread_status_notice(&fixture.sessions, notification).await,
            Some((conversation, expected_message.into()))
        );
    }
}

#[tokio::test]
async fn status_notices_preserve_the_active_turn_and_pending_approval() {
    let fixture = TestSessions::new().await;
    let conversation = ConversationKey::new(ChatId(-1001), Some(ThreadId(MessageId(11))));
    fixture
        .sessions
        .set_thread(conversation, "thread".into())
        .await
        .unwrap();
    fixture
        .sessions
        .set_turn(conversation, "turn".into(), Some(UserId(77)))
        .await;
    let request_id = RequestId::Integer(7);
    fixture
        .sessions
        .insert_pending_approval(
            conversation,
            PendingApproval {
                request_id: request_id.clone(),
                kind: PendingApprovalKind::FileChange(FileChangeRequestApprovalParams {
                    thread_id: "thread".into(),
                    turn_id: "turn".into(),
                    item_id: "item".into(),
                    started_at_ms: 1,
                    reason: None,
                    grant_root: None,
                }),
            },
        )
        .await;
    let approval = fixture
        .sessions
        .pending_approval(conversation, &request_id)
        .await
        .unwrap();

    for (notification, _) in notices("thread") {
        assert!(
            thread_status_notice(&fixture.sessions, notification)
                .await
                .is_some()
        );
        assert_eq!(
            (
                fixture.sessions.turn_id(conversation).await,
                fixture.sessions.pending_approval_count(conversation).await,
                fixture
                    .sessions
                    .pending_approval(conversation, &request_id)
                    .await,
            ),
            (Some("turn".into()), 1, Some(approval.clone()))
        );
    }
}
