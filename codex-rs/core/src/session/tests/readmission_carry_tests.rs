//! Regression coverage for held-input admission and suspension cleanup carried from main.

use super::*;
use codex_protocol::turn_input::SuspendTurnOutcome;
use pretty_assertions::assert_eq;
use std::future::Future;
use std::task::Context;
use std::task::Wake;
use std::task::Waker;

async fn start_held_task(sess: &Arc<Session>, tc: &Arc<TurnContext>, kind: TaskKind) {
    sess.spawn_task(
        Arc::clone(tc),
        Vec::new(),
        NeverEndingTask {
            kind,
            listen_to_cancellation_token: true,
        },
    )
    .await;
}

#[test_case::test_case(TaskKind::Compact; "compaction")]
#[test_case::test_case(TaskKind::Regular; "changed authority")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rejected_held_user_input_starts_after_successful_completion(kind: TaskKind) {
    let (sess, tc, rx) = make_session_and_context_with_submission_loop_and_rx().await;
    start_held_task(&sess, &tc, kind).await;
    let expected_policy = if kind == TaskKind::Regular {
        sess.update_settings(SessionSettingsUpdate {
            step_settings: StepSettingsUpdate {
                approval_policy: Some(AskForApproval::UnlessTrusted),
                ..Default::default()
            },
            ..Default::default()
        })
        .await
        .expect("change authority");
        AskForApproval::UnlessTrusted
    } else {
        tc.approval_policy()
    };
    let content = vec![UserInput::Text {
        text: "held work must survive admission refusal".into(),
        text_elements: Vec::new(),
    }];
    while rx.try_recv().is_ok() {}
    super::handlers::readmit_interrupted_input(
        &sess,
        TurnInput::UserInput {
            content: content.clone(),
            client_id: Some("held-client-input".into()),
            metadata: Default::default(),
        },
        /*final_output_json_schema*/ None,
    )
    .await;

    assert!(rx.try_recv().is_err(), "held input should stay deferred");
    assert_eq!(
        sess.input_queue
            .get_pending_input_for_running_task(&sess, &tc.sub_id)
            .await
            .0,
        Vec::new()
    );
    assert_eq!(
        sess.active_turn
            .lock()
            .await
            .as_ref()
            .unwrap()
            .task
            .as_ref()
            .unwrap()
            .turn_context
            .sub_id,
        tc.sub_id
    );

    // Complete naturally, without draining the deferred queue in the test.
    sess.on_task_finished(Arc::clone(&tc), Ok(None)).await;
    let completed = recv_skipping_mcp_startup(&rx, "original turn completes").await;
    assert!(
        matches!(completed.msg, EventMsg::TurnComplete(TurnCompleteEvent { turn_id, .. }) if turn_id == tc.sub_id)
    );
    let started = recv_skipping_mcp_startup(&rx, "held input starts its own turn").await;
    let EventMsg::TurnStarted(TurnStartedEvent { turn_id, .. }) = started.msg else {
        panic!("expected admitted follow-up turn, got {:?}", started.msg);
    };
    assert_ne!(turn_id, tc.sub_id);
    {
        let active = sess.active_turn.lock().await;
        let next = &active.as_ref().unwrap().task.as_ref().unwrap().turn_context;
        assert_eq!(
            (next.sub_id.as_str(), next.approval_policy()),
            (turn_id.as_str(), expected_policy)
        );
    }
    expect_user_message_item_started(&rx, &content).await;
    sess.abort_all_tasks(TurnAbortReason::Interrupted).await;
}

#[tokio::test]
async fn held_response_waits_for_finishing_then_joins_replacement_turn() {
    let (sess, tc, _rx) = make_session_and_context_with_rx().await;
    let finishing = CancellationToken::new();
    *sess.active_turn.lock().await = Some(ActiveTurn {
        finishing: Some(finishing.clone()),
        ..Default::default()
    });
    let item = ResponseItemEnvelope::new(user_message("held extension result"));
    let history_before = raw_history_items(&sess.clone_history().await);
    let readmission = super::handlers::readmit_interrupted_input(
        &sess,
        TurnInput::ResponseItem(item.clone()),
        /*final_output_json_schema*/ None,
    );
    let mut readmission = Box::pin(tokio::task::unconstrained(readmission));
    assert!(futures::poll!(readmission.as_mut()).is_pending());
    assert_eq!(
        raw_history_items(&sess.clone_history().await),
        history_before,
        "finishing history must not be interleaved"
    );

    // Install the replacement before releasing the old turn's finishing fence.
    *sess.active_turn.lock().await = None;
    start_held_task(&sess, &tc, TaskKind::Regular).await;
    finishing.cancel();
    tokio::time::timeout(StdDuration::from_secs(2), readmission)
        .await
        .expect("held response re-admitted");
    assert_eq!(
        sess.input_queue
            .get_pending_input_for_running_task(&sess, &tc.sub_id)
            .await
            .0,
        vec![TurnInput::ResponseItem(item)]
    );
    assert_eq!(
        raw_history_items(&sess.clone_history().await),
        history_before,
        "replacement turn consumes the item rather than a standalone history append"
    );
    sess.abort_all_tasks(TurnAbortReason::Interrupted).await;
}

// Snapshot bookkeeping synchronously when oneshot::Sender::send wakes the
// receiver. Checking only after awaiting the reply can miss cleanup ordered
// after send, because the submission loop may run before its caller is polled.
struct SuspensionReplyWake {
    thread_id: ThreadId,
    unsaved_at_reply: std::sync::Mutex<Option<bool>>,
}

impl Wake for SuspensionReplyWake {
    fn wake(self: Arc<Self>) {
        *self.unsaved_at_reply.lock().unwrap() =
            Some(crate::home_access::unsaved_state(self.thread_id).0);
    }
}

#[tokio::test]
async fn successful_suspension_clears_unsaved_thread_before_reply() {
    let (mut session, tc) = make_session_and_context().await;
    attach_in_memory_thread_store(&mut session).await;
    let (tx_event, _rx_event) = async_channel::unbounded();
    session.tx_event = tx_event;
    let (tx_sub, rx_sub) = async_channel::bounded(1);
    session.tx_sub = tx_sub.clone();
    let sess = Arc::new(session);
    let tc = Arc::new(tc);
    start_held_task(&sess, &tc, TaskKind::Regular).await;
    crate::home_access::note_rollout_unwritten(
        sess.thread_id(),
        sess.codex_home().await.to_path_buf(),
    );
    assert!(crate::home_access::unsaved_state(sess.thread_id()).0);

    let (reply, mut accepted) = tokio::sync::oneshot::channel();
    let observer = Arc::new(SuspensionReplyWake {
        thread_id: sess.thread_id(),
        unsaved_at_reply: std::sync::Mutex::new(None),
    });
    let waker = Waker::from(Arc::clone(&observer));
    assert!(
        std::pin::Pin::new(&mut accepted)
            .poll(&mut Context::from_waker(&waker))
            .is_pending()
    );
    tx_sub
        .send(Submission {
            id: "suspend".into(),
            op: Op::SuspendTurnAndShutdown { reply },
            turn_extension_init: None,
            trace: None,
            parent_turn_id: None,
            root_turn_id: None,
            residency_guard: None,
            message_delivery: None,
        })
        .await
        .unwrap();
    tokio::time::timeout(
        StdDuration::from_secs(2),
        super::handlers::submission_loop(Arc::clone(&sess), Arc::clone(&tc.config), rx_sub),
    )
    .await
    .expect("suspension exits the submission loop");
    assert_eq!(*observer.unsaved_at_reply.lock().unwrap(), Some(false));
    assert_eq!(
        accepted.await.unwrap().unwrap(),
        SuspendTurnOutcome::Suspended {
            turn_id: tc.sub_id.clone()
        }
    );
    assert!(!crate::home_access::unsaved_state(sess.thread_id()).0);
    assert!(sess.active_turn.lock().await.is_none());
}
