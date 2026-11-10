//! #2047: tests poll pending input through the same path `run_turn` uses. With no active turn
//! there is no turn state to record taken mail, so the poll leaves mailbox mail queued for the
//! next task start; with an active turn the poll takes it.
use super::*;
use crate::session::tests::NeverEndingTask;
use crate::session::tests::make_session_and_context_with_rx;
use crate::state::TaskKind;
use codex_protocol::AgentPath;
use codex_protocol::protocol::InterAgentCommunication;
use codex_protocol::protocol::TurnAbortReason;
use pretty_assertions::assert_eq;
use std::sync::Arc;

/// Trigger-turn mail, so the next task start may admit it without first crossing a turn boundary.
fn worker_mail() -> InterAgentCommunication {
    InterAgentCommunication::new(
        AgentPath::try_from("/root/worker").expect("agent path"),
        AgentPath::root(),
        Vec::new(),
        "worker update".to_string(),
        /*trigger_turn*/ true,
    )
}

#[tokio::test]
async fn idle_session_poll_leaves_mail_for_the_next_task_start() {
    let (sess, _tc, _rx) = make_session_and_context_with_rx().await;
    let mail = worker_mail();
    sess.input_queue
        .enqueue_mailbox_communication(mail.clone(), /*parent_turn_id*/ None)
        .await;

    assert_eq!(
        sess.input_queue
            .get_pending_input_for_running_task(&sess.active_turn)
            .await,
        (Vec::new(), None)
    );
    assert!(sess.input_queue.has_pending_mailbox_items().await);

    assert_eq!(
        sess.input_queue
            .get_pending_input_for_task_start(&sess.active_turn)
            .await
            .0,
        vec![TurnInput::InterAgentCommunication(mail)]
    );
}

#[tokio::test]
async fn active_turn_poll_takes_queued_mail() {
    let (sess, tc, _rx) = make_session_and_context_with_rx().await;
    sess.spawn_task(
        Arc::clone(&tc),
        Vec::new(),
        NeverEndingTask {
            kind: TaskKind::Regular,
            listen_to_cancellation_token: true,
        },
    )
    .await;
    let mail = worker_mail();
    sess.input_queue
        .enqueue_mailbox_communication(mail.clone(), /*parent_turn_id*/ None)
        .await;

    assert_eq!(
        sess.input_queue
            .get_pending_input_for_running_task(&sess.active_turn)
            .await
            .0,
        vec![TurnInput::InterAgentCommunication(mail)]
    );
    assert!(!sess.input_queue.has_pending_mailbox_items().await);

    sess.abort_all_tasks(TurnAbortReason::Replaced).await;
}
