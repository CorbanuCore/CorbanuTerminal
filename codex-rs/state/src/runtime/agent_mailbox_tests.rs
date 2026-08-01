use super::*;
use codex_protocol::AgentPath;
use codex_protocol::protocol::AgentMessageKind;
use pretty_assertions::assert_eq;

fn communication(message_id: &str) -> InterAgentCommunication {
    let mut communication = InterAgentCommunication::new(
        AgentPath::root(),
        AgentPath::try_from("/root/worker").expect("agent path"),
        Vec::new(),
        "inspect the repository".to_string(),
        /*trigger_turn*/ true,
    )
    .with_kind(AgentMessageKind::Assignment);
    communication.message_id = Some(message_id.to_string());
    communication.created_at_ms = Some(1_000);
    communication
}

#[tokio::test]
async fn mailbox_admission_is_idempotent_and_recoverable() {
    let runtime = StateRuntime::init_for_testing(
        test_support::unique_temp_dir(),
        "test-provider".to_string(),
    )
    .await
    .expect("state runtime");
    let recipient = ThreadId::new();
    let message = communication("message-1");

    assert_eq!(
        runtime
            .admit_agent_message(recipient, &message, 1_000)
            .await
            .expect("first admission"),
        AgentMailboxAdmission::Inserted
    );
    assert_eq!(
        runtime
            .admit_agent_message(recipient, &message, 1_001)
            .await
            .expect("duplicate admission"),
        AgentMailboxAdmission::Existing(AgentMailboxPhase::Admitted)
    );
    let mut timestamp_retry = message.clone();
    timestamp_retry.created_at_ms = Some(9_999);
    assert_eq!(
        runtime
            .admit_agent_message(recipient, &timestamp_retry, 1_001)
            .await
            .expect("server timestamp must not alter logical identity"),
        AgentMailboxAdmission::Existing(AgentMailboxPhase::Admitted)
    );
    assert!(
        runtime
            .transition_agent_message(
                "message-1",
                AgentMailboxPhase::Admitted,
                AgentMailboxPhase::Ready,
                1_002,
            )
            .await
            .expect("ready transition")
    );
    assert_eq!(
        runtime
            .list_recoverable_agent_messages(recipient)
            .await
            .expect("recoverable messages"),
        vec![AgentMailboxMessage {
            recipient_thread_id: recipient,
            communication: message,
            phase: AgentMailboxPhase::Ready,
            attempt_id: None,
            created_at_ms: 1_000,
            updated_at_ms: 1_002,
        }]
    );
}

#[tokio::test]
async fn mailbox_records_a_new_attempt_without_changing_logical_identity() {
    let runtime = StateRuntime::init_for_testing(
        test_support::unique_temp_dir(),
        "test-provider".to_string(),
    )
    .await
    .expect("state runtime");
    let recipient = ThreadId::new();
    let message = communication("message-1");
    runtime
        .admit_agent_message(recipient, &message, 1_000)
        .await
        .expect("admission");
    assert!(
        runtime
            .transition_agent_message(
                "message-1",
                AgentMailboxPhase::Admitted,
                AgentMailboxPhase::Ready,
                1_001,
            )
            .await
            .expect("ready")
    );
    assert!(
        runtime
            .begin_agent_message_submission(
                "message-1",
                AgentMailboxPhase::Ready,
                "attempt-1",
                1_002,
            )
            .await
            .expect("begin attempt")
    );

    let messages = runtime
        .list_recoverable_agent_messages(recipient)
        .await
        .expect("recoverable");
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].phase, AgentMailboxPhase::Submitting);
    assert_eq!(messages[0].attempt_id.as_deref(), Some("attempt-1"));
    assert_eq!(
        messages[0].communication.message_id.as_deref(),
        Some("message-1")
    );
}

#[tokio::test]
async fn completed_mailbox_message_is_terminal_but_remains_auditable() {
    let runtime = StateRuntime::init_for_testing(
        test_support::unique_temp_dir(),
        "test-provider".to_string(),
    )
    .await
    .expect("state runtime");
    let recipient = ThreadId::new();
    let message = communication("message-1");
    runtime
        .admit_agent_message(recipient, &message, 1_000)
        .await
        .expect("admission");
    assert!(
        runtime
            .mark_agent_message_completed("message-1", 1_001)
            .await
            .expect("completion")
    );
    assert_eq!(
        runtime
            .list_recoverable_agent_messages(recipient)
            .await
            .expect("recoverable"),
        Vec::new()
    );
    let stored = runtime
        .get_agent_message("message-1")
        .await
        .expect("lookup")
        .expect("stored message");
    assert_eq!(stored.phase, AgentMailboxPhase::Completed);
    assert_eq!(stored.communication, message);
}

#[tokio::test]
async fn mailbox_rejects_message_id_reuse_for_different_content() {
    let runtime = StateRuntime::init_for_testing(
        test_support::unique_temp_dir(),
        "test-provider".to_string(),
    )
    .await
    .expect("state runtime");
    let recipient = ThreadId::new();
    let first = communication("message-1");
    runtime
        .admit_agent_message(recipient, &first, 1_000)
        .await
        .expect("first admission");
    let mut conflicting = first;
    conflicting.content = "different work".to_string();

    let error = runtime
        .admit_agent_message(recipient, &conflicting, 1_001)
        .await
        .expect_err("message-id collision must fail");
    assert!(
        error
            .to_string()
            .contains("conflicts with a different message")
    );
}
