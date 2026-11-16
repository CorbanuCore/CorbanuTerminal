use super::*;
use pretty_assertions::assert_eq;

#[derive(Clone, Copy)]
struct PanickingTask;

impl SessionTask for PanickingTask {
    fn kind(&self) -> TaskKind {
        TaskKind::Regular
    }

    fn span_name(&self) -> &'static str {
        "session_task.panicking"
    }

    async fn run(
        self: Arc<Self>,
        _session: Arc<Session>,
        _ctx: Arc<TurnContext>,
        _input: Vec<TurnInput>,
        _cancellation_token: CancellationToken,
    ) -> SessionTaskResult {
        panic!("#616 simulated panic in a turn task");
    }
}

/// #616: a task that panics still ends its turn, with an error, so clients
/// waiting for the turn to end (exec, the TUI) don't wait forever.
#[tokio::test]
async fn a_panicking_task_fails_its_turn_instead_of_leaving_it_running() {
    let (sess, tc, rx) = make_session_and_context_with_rx().await;
    sess.spawn_task(
        Arc::clone(&tc),
        vec![TurnInput::UserInput {
            metadata: Default::default(),
            content: vec![UserInput::Text {
                text: "hello".to_string(),
                text_elements: Vec::new(),
            }],
            client_id: None,
        }],
        PanickingTask,
    )
    .await;

    let error = timeout(Duration::from_secs(2), async {
        loop {
            let event = rx.recv().await.expect("event");
            match event.msg {
                EventMsg::Error(error) => return error,
                EventMsg::TurnComplete(_) | EventMsg::TurnAborted(_) => {
                    panic!("turn ended without an error: {:?}", event.msg)
                }
                _ => {}
            }
        }
    })
    .await
    .expect("the panic should be reported as a turn error");
    assert_eq!(
        error.message,
        format!("Fatal error: {}", crate::tasks::TURN_PANICKED_MESSAGE)
    );
    let _ = recv_terminal_event(&rx, TerminalEventKind::TurnComplete).await;
    // The turn is cleared just after TurnComplete is sent.
    timeout(Duration::from_secs(2), async {
        while sess.active_turn.lock().await.is_some() {
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("the finished turn is cleared");
}

#[test_case::test_case(false; "natural_completion")]
#[test_case::test_case(true; "terminal_error")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn taken_unanswered_mail_is_saved_without_follow_up(terminal_error: bool) {
    let (sess, tc, rx) = make_session_and_context_with_submission_loop_and_rx().await;
    sess.spawn_task(
        Arc::clone(&tc),
        Vec::new(),
        NeverEndingTask {
            kind: TaskKind::Regular,
            listen_to_cancellation_token: false,
        },
    )
    .await;

    let mut mail = InterAgentCommunication::new(
        AgentPath::root().join("worker").expect("worker path"),
        AgentPath::root(),
        Vec::new(),
        "save this unanswered mail".to_string(),
        /*trigger_turn*/ true,
    );
    mail.set_turn_id_if_missing(&tc.sub_id);
    let expected = mail.to_model_input_item();
    let turn_state = Arc::clone(
        &sess
            .active_turn
            .lock()
            .await
            .as_ref()
            .expect("active turn")
            .turn_state,
    );
    sess.input_queue
        .extend_pending_input_for_turn_state(
            &turn_state,
            vec![TurnInput::InterAgentCommunication(mail.into())],
        )
        .await;
    if terminal_error {
        sess.send_event(
            &tc,
            EventMsg::Error(ErrorEvent {
                message: "quota exhausted".to_string(),
                codex_error_info: Some(CodexErrorInfo::UsageLimitExceeded),
                misalignment: None,
            }),
        )
        .await;
    }
    while rx.try_recv().is_ok() {}

    sess.on_task_finished(Arc::clone(&tc), /*task_result*/ Ok(None))
        .await;

    assert_eq!(
        strip_response_item_ids(&strip_metadata_from_items(&raw_history_items(
            &sess.clone_history().await,
        ))),
        strip_response_item_ids(&strip_metadata_from_items(&[expected]))
    );
    assert!(sess.active_turn.lock().await.is_none());
    // A mistaken follow-up is dispatched through the submission loop, so allow that loop
    // to run and fail on a second turn rather than checking only the synchronous slot.
    let unexpected_start = tokio::time::timeout(Duration::from_millis(100), async {
        while let Ok(event) = rx.recv().await {
            if matches!(event.msg, EventMsg::TurnStarted(_)) {
                return;
            }
        }
        panic!("event channel closed");
    })
    .await;
    assert!(
        unexpected_start.is_err(),
        "unanswered mail started another turn"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn refused_turn_retains_pending_user_follow_up_after_refusal_is_cleared() {
    let (sess, tc, rx) = make_session_and_context_with_submission_loop_and_rx().await;
    sess.spawn_task(
        Arc::clone(&tc),
        Vec::new(),
        NeverEndingTask {
            kind: TaskKind::Regular,
            listen_to_cancellation_token: false,
        },
    )
    .await;
    let pending = vec![UserInput::Text {
        text: "pending after refusal".to_string(),
        text_elements: Vec::new(),
    }];
    let submission = submit_steer_only(&sess, pending.clone(), &tc.sub_id).await;
    assert!(matches!(submission, TurnInputSubmission::Steered { .. }));
    // The refusal decision is already captured. The live provider is now usable; querying
    // its current state at completion would misclassify the original refusal as a failure.
    tc.turn_refused
        .store(true, std::sync::atomic::Ordering::Release);
    sess.send_event(
        &tc,
        EventMsg::Error(ErrorEvent {
            message: "earlier pre-turn refusal".to_string(),
            codex_error_info: Some(CodexErrorInfo::BadRequest),
            misalignment: None,
        }),
    )
    .await;
    while rx.try_recv().is_ok() {}

    sess.on_task_finished(Arc::clone(&tc), /*task_result*/ Ok(None))
        .await;

    let _ = recv_terminal_event(&rx, TerminalEventKind::TurnComplete).await;
    let follow_up = recv_skipping_mcp_startup(&rx, "expected refused-input follow-up turn").await;
    assert!(
        matches!(follow_up.msg, EventMsg::TurnStarted(started) if started.turn_id != tc.sub_id),
        "the captured refusal must retain its existing follow-up path"
    );
    expect_user_message_item_started(&rx, &pending).await;
    sess.abort_all_tasks(TurnAbortReason::Interrupted).await;
}
