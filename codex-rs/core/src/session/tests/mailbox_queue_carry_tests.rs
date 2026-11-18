use super::*;
use pretty_assertions::assert_eq;

#[tokio::test]
async fn queue_only_mailbox_mail_waits_for_next_turn_after_answer_boundary() {
    let (sess, tc, _rx) = make_session_and_context_with_rx().await;
    let communication = InterAgentCommunication::new(
        AgentPath::try_from("/root/worker").expect("worker path should parse"),
        AgentPath::root(),
        Vec::new(),
        "late queue-only update".to_string(),
        /*trigger_turn*/ false,
    );
    sess.spawn_task(
        Arc::clone(&tc),
        Vec::new(),
        NeverEndingTask {
            kind: TaskKind::Regular,
            listen_to_cancellation_token: true,
        },
    )
    .await;

    sess.input_queue
        .defer_mailbox_delivery_to_next_turn(&sess.active_turn, &tc.sub_id)
        .await;
    sess.input_queue
        .enqueue_mailbox_communication(communication.clone().into(), Default::default())
        .await;

    assert!(
        !sess.input_queue.has_pending_input(&sess.active_turn).await,
        "queue-only mailbox mail should stay buffered once the current turn emitted its answer"
    );
    assert_eq!(
        sess.input_queue
            .get_pending_input_for_running_task(&sess, &tc.sub_id)
            .await
            .0,
        Vec::new()
    );

    sess.abort_all_tasks(TurnAbortReason::Replaced).await;

    assert_eq!(
        sess.input_queue.drain_mailbox_input_items().await.0,
        vec![TurnInput::InterAgentCommunication(communication.into())],
    );
}
