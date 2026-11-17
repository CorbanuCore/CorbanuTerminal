//! PF-83 steering authority and deferral (main #2073, 23915e5c6f, 44bbf0db17) and the PF-13
//! user-turn MCP check (3db1c68755): main's session tests ported onto upstream's turn-input API.
//!
//! Upstream returns direct client input that cannot join the running turn to the client
//! (`TurnInputSubmission::NotSubmitted`) instead of Core deferring it (the app server tells the
//! client to submit again after the turn). Held, follow-up, injected and mailbox input still waits
//! in Core. Each port keeps main's protection: input never joins a turn admitted under other
//! permissions, nothing is lost, and what runs later runs under the latest settings.

use super::*;
use crate::session::input_queue::InputQueueActivity;
use crate::session::step_settings::StepSettingsUpdate;
use codex_protocol::turn_input::NotSubmittedReason;
use codex_protocol::turn_input::TurnStartOptions;
use core_test_support::test_codex::local_requests;
use pretty_assertions::assert_eq;

fn text_input(text: &str) -> Vec<UserInput> {
    vec![UserInput::Text {
        text: text.to_string(),
        text_elements: Vec::new(),
    }]
}

async fn submit_start_or_steer(sess: &Arc<Session>, input: Vec<UserInput>) -> TurnInputSubmission {
    super::turn_input::handle(
        sess,
        TurnInputRequest::new(SubmittedTurnInput::UserInput {
            content: input,
            client_id: None,
        }),
        TurnInputMode::StartOrSteer,
        "test-submission".to_string(),
    )
    .await
    .expect("start-or-steer submission should be valid")
}

async fn set_approval_policy(sess: &Session, approval_policy: AskForApproval) {
    sess.update_settings(SessionSettingsUpdate {
        step_settings: StepSettingsUpdate {
            approval_policy: Some(approval_policy),
            ..Default::default()
        },
        ..Default::default()
    })
    .await
    .expect("update approval policy");
}

async fn spawn_never_ending(sess: &Arc<Session>, tc: &Arc<TurnContext>, kind: TaskKind) {
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

async fn running_turn_context(sess: &Session) -> Arc<TurnContext> {
    let active = sess.active_turn.lock().await;
    Arc::clone(
        &active
            .as_ref()
            .expect("active turn")
            .task
            .as_ref()
            .expect("running task")
            .turn_context,
    )
}

async fn next_turn_started(rx: &async_channel::Receiver<Event>, previous_turn_id: &str) {
    let deadline = tokio::time::Instant::now() + StdDuration::from_secs(2);
    loop {
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        let event = tokio::time::timeout(remaining, rx.recv())
            .await
            .expect("expected a fresh turn to start")
            .expect("channel open");
        match event.msg {
            EventMsg::Error(err) => panic!("no error expected: {err:?}"),
            EventMsg::TurnStarted(TurnStartedEvent { turn_id, .. }) => {
                assert_ne!(turn_id, previous_turn_id);
                return;
            }
            _ => {}
        }
    }
}

/// Main's `steer_input_accepts_runtime_user_layer_reload`: reloading the user config layer at
/// runtime does not change the running turn's authority, so input still steers into it.
#[tokio::test]
async fn steer_accepts_runtime_user_layer_reload() {
    let (sess, tc, _rx) = make_session_and_context_with_rx().await;
    spawn_never_ending(&sess, &tc, TaskKind::Regular).await;
    let home = sess.codex_home().await;
    std::fs::create_dir_all(&home).expect("create codex home");
    std::fs::write(
        home.join(CONFIG_TOML_FILE),
        "[apps.calendar]\nenabled = false\n",
    )
    .expect("write user config");
    let current_config = sess.get_config().await;
    let next_config = load_latest_config_for_session(&sess).await;
    assert_eq!(
        sess.refresh_runtime_config(current_config, next_config)
            .await,
        crate::ConfigRefreshOutcome::Published
    );

    let submission = submit_steer_only(&sess, text_input("after reload"), &tc.sub_id).await;
    assert!(
        matches!(&submission, TurnInputSubmission::Steered { turn_id, .. } if *turn_id == tc.sub_id),
        "{submission:?}"
    );
    sess.abort_all_tasks(TurnAbortReason::Replaced).await;
}

/// Main's `authorization_changed_user_input_runs_after_completion_with_latest_permissions`.
/// Upstream returns the input to the client instead of deferring it: it never joins the turn
/// admitted under the earlier permissions, nothing is queued, and submitted again after the turn
/// it runs with the latest permissions.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn authorization_changed_user_input_is_returned_and_runs_later_with_latest_permissions() {
    let (sess, tc, rx) = make_session_and_context_with_submission_loop_and_rx().await;
    spawn_never_ending(&sess, &tc, TaskKind::Regular).await;
    set_approval_policy(&sess, AskForApproval::UnlessTrusted).await;
    let items = text_input("keep my follow-up");

    assert_eq!(
        submit_start_or_steer(&sess, items.clone()).await,
        TurnInputSubmission::NotSubmitted {
            reason: NotSubmittedReason::AuthorizationChanged,
        }
    );
    assert!(!sess.input_queue.has_pending_input(&sess.active_turn).await);
    assert_eq!(
        sess.input_queue
            .get_pending_input_for_running_task(&sess, &tc.sub_id)
            .await
            .0,
        Vec::new()
    );
    assert!(Arc::ptr_eq(&running_turn_context(&sess).await, &tc));

    sess.on_task_finished(Arc::clone(&tc), Ok(None)).await;
    while rx.try_recv().is_ok() {}
    assert!(matches!(
        submit_start_or_steer(&sess, items.clone()).await,
        TurnInputSubmission::Started { .. }
    ));
    expect_user_message_item_started(&rx, &items).await;
    let next = running_turn_context(&sess).await;
    assert_ne!(next.sub_id, tc.sub_id);
    assert_eq!(next.approval_policy(), AskForApproval::UnlessTrusted);
    sess.abort_all_tasks(TurnAbortReason::Interrupted).await;
}

/// Main's `non_steerable_turn_defers_user_input_until_completion`: input sent while a compaction
/// runs is returned to the client without an error event and, sent again after the compaction,
/// starts its own turn.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn non_steerable_turn_returns_user_input_until_completion() {
    let (sess, tc, rx) = make_session_and_context_with_submission_loop_and_rx().await;
    spawn_never_ending(&sess, &tc, TaskKind::Compact).await;
    while rx.try_recv().is_ok() {}
    let items = text_input("run after compact");

    let submission = submit_start_or_steer(&sess, items.clone()).await;
    assert!(
        matches!(
            submission,
            TurnInputSubmission::NotSubmitted {
                reason: NotSubmittedReason::ActiveTurnNotSteerable { .. },
            }
        ),
        "{submission:?}"
    );
    assert!(
        rx.try_recv().is_err(),
        "non-steerable input should not emit an immediate error"
    );
    assert!(!sess.input_queue.has_pending_input(&sess.active_turn).await);

    sess.on_task_finished(Arc::clone(&tc), Ok(None)).await;
    let first = tokio::time::timeout(StdDuration::from_secs(2), rx.recv())
        .await
        .expect("expected compact turn complete event")
        .expect("channel open");
    assert!(matches!(
        first.msg,
        EventMsg::TurnComplete(TurnCompleteEvent { turn_id, .. }) if turn_id == tc.sub_id
    ));
    assert!(matches!(
        submit_start_or_steer(&sess, items.clone()).await,
        TurnInputSubmission::Started { .. }
    ));
    next_turn_started(&rx, &tc.sub_id).await;
    expect_user_message_item_started(&rx, &items).await;
    sess.abort_all_tasks(TurnAbortReason::Interrupted).await;
}

/// Main's `interrupt_clears_steered_input_without_follow_up_turn`.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn interrupt_clears_steered_input_without_follow_up_turn() {
    let (sess, tc, rx) = make_session_and_context_with_rx().await;
    spawn_never_ending(&sess, &tc, TaskKind::Regular).await;
    while rx.try_recv().is_ok() {}

    let submission = submit_steer_only(&sess, text_input("drop on interrupt"), &tc.sub_id).await;
    assert!(
        matches!(submission, TurnInputSubmission::Steered { .. }),
        "{submission:?}"
    );
    sess.abort_all_tasks(TurnAbortReason::Interrupted).await;

    let deadline = tokio::time::Instant::now() + StdDuration::from_secs(2);
    loop {
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        let event = tokio::time::timeout(remaining, rx.recv())
            .await
            .expect("expected turn aborted event")
            .expect("channel open");
        if matches!(event.msg, EventMsg::TurnAborted(_)) {
            break;
        }
    }
    assert!(sess.active_turn.lock().await.is_none());
    assert_eq!(
        sess.input_queue.drain_mailbox_input_items().await.0,
        Vec::<TurnInput>::new()
    );
    assert!(
        tokio::time::timeout(StdDuration::from_millis(200), rx.recv())
            .await
            .is_err(),
        "interrupt should not start a follow-up turn for cleared steer input"
    );
}

/// Main's `authorization_changed_mail_waits_for_turn_boundary`: mail that arrives after the
/// running turn's permissions changed waits for the next turn, even when the earlier selection
/// is restored or delivery to the turn is reopened.
#[tokio::test]
async fn authorization_changed_mail_waits_for_turn_boundary() {
    for trigger_turn in [false, true] {
        let (sess, tc, _rx) = make_session_and_context_with_rx().await;
        spawn_never_ending(&sess, &tc, TaskKind::Regular).await;
        let admitted_policy = tc.approval_policy();
        set_approval_policy(&sess, AskForApproval::UnlessTrusted).await;
        let mail = InterAgentCommunication::new(
            AgentPath::root(),
            AgentPath::root(),
            Vec::new(),
            "deferred peer work".into(),
            trigger_turn,
        );
        super::handlers::inter_agent_communication(
            &sess,
            "mail".into(),
            mail.clone().into(),
            TurnStartOptions {
                parent_turn_id: Some("parent".into()),
                ..Default::default()
            },
        )
        .await;
        // Reopening delivery or restoring the old selection must not release deferred mail.
        set_approval_policy(&sess, admitted_policy).await;
        sess.input_queue
            .accept_mailbox_delivery_for_current_turn(&sess.active_turn, &tc.sub_id)
            .await;
        assert!(!sess.input_queue.has_pending_input(&sess.active_turn).await);
        assert_eq!(
            sess.input_queue
                .get_pending_input_for_running_task(&sess, &tc.sub_id)
                .await
                .0,
            Vec::new()
        );
        sess.input_queue.mark_mailbox_ready_for_next_turn().await;
        assert_eq!(
            sess.input_queue
                .get_pending_input_for_running_task(&sess, &tc.sub_id)
                .await
                .0,
            Vec::new()
        );
        sess.abort_all_tasks(TurnAbortReason::Replaced).await;
        let (items, start_options) = sess.input_queue.drain_ready_mailbox_input_items().await;
        assert_eq!(
            (items, start_options.parent_turn_id),
            (
                vec![TurnInput::InterAgentCommunication(mail.into())],
                trigger_turn.then(|| "parent".into())
            )
        );
        sess.abort_all_tasks(TurnAbortReason::Replaced).await;
    }
}

/// Main's `authorization_changed_injection_preserves_items_until_completion`: extension work
/// injected after the running turn's permissions changed is neither recorded nor handed to that
/// turn; it waits as the turn's deferred input.
#[tokio::test]
async fn authorization_changed_injection_preserves_items_until_completion() {
    let (sess, tc, _rx) = make_session_and_context_with_rx().await;
    spawn_never_ending(&sess, &tc, TaskKind::Regular).await;
    set_approval_policy(&sess, AskForApproval::UnlessTrusted).await;
    let item = ResponseItem::Message {
        encrypted_content: None,
        status: None,
        id: None,
        role: "user".into(),
        content: vec![ContentItem::InputText {
            text: "extension work".into(),
        }],
        phase: None,
        internal_chat_message_metadata_passthrough: None,
    };
    let history_before: Vec<ResponseItem> =
        sess.clone_history().await.raw_items().cloned().collect();
    sess.inject_extension_if_running(vec![item.clone()])
        .await
        .expect("a running turn takes the injection");
    assert_eq!(
        sess.clone_history()
            .await
            .raw_items()
            .cloned()
            .collect::<Vec<_>>(),
        history_before
    );
    assert!(!sess.input_queue.has_pending_input(&sess.active_turn).await);
    assert_eq!(
        sess.input_queue
            .get_pending_input_for_running_task(&sess, &tc.sub_id)
            .await
            .0,
        Vec::new()
    );
    let turn_state = Arc::clone(
        &sess
            .active_turn
            .lock()
            .await
            .as_ref()
            .expect("active turn")
            .turn_state,
    );
    let deferred = sess
        .input_queue
        .take_pending_input_for_turn_state(&turn_state)
        .await;
    let [(TurnInput::ResponseItem(deferred_item), None)] = deferred.as_slice() else {
        panic!("expected the injected item as deferred input, got {deferred:?}");
    };
    assert_eq!(deferred_item.clone().into_item(), item);
    sess.abort_all_tasks(TurnAbortReason::Replaced).await;
}

/// Main's `authorization_changed_deferred_input_survives_interrupt` (main deferred the user's
/// input; upstream defers held input the same way): an interrupt keeps deferred input as
/// interrupted input for the next admission instead of dropping it.
#[tokio::test]
async fn authorization_changed_deferred_input_survives_interrupt() {
    let (sess, tc, _rx) = make_session_and_context_with_rx().await;
    spawn_never_ending(&sess, &tc, TaskKind::Regular).await;
    let items = text_input("keep after interrupt");
    let deferred = TurnInput::UserInput {
        content: items,
        client_id: Some("client".into()),
        metadata: Default::default(),
    };
    sess.defer_held_input(deferred.clone(), /*final_output_json_schema*/ None)
        .await;
    sess.abort_all_tasks(TurnAbortReason::Interrupted).await;
    assert!(sess.input_queue.has_pending_input(&sess.active_turn).await);
    assert_eq!(
        sess.input_queue
            .subscribe_activity(/*turn_state*/ None)
            .await
            .1,
        Some(InputQueueActivity::Steer)
    );
    assert_eq!(
        sess.input_queue
            .get_pending_input_for_task_start(&sess.active_turn)
            .await
            .0,
        Vec::new()
    );
    assert_eq!(
        sess.input_queue.next_interrupted_deferred_input().await,
        (deferred, None)
    );
    assert!(!sess.input_queue.has_pending_input(&sess.active_turn).await);
}

/// PF-13 (main's `user_turn_updates_approvals_reviewer`; upstream deleted the test with
/// `Op::UserInput`): a user turn that changes the approvals reviewer starts with that reviewer,
/// and the MCP state its tool calls use is republished with it before the turn executes: the
/// turn's step capture (which runs before each sampling request) leaves no MCP refresh pending
/// and the published runtime config, which MCP tool approvals and elicitation reviews read,
/// carries the new reviewer.
#[tokio::test]
async fn user_turn_updates_approvals_reviewer_for_mcp() {
    let (session, _turn_context, _rx) = make_session_and_context_with_rx().await;
    let config = session.get_config().await;
    let request = TurnInputRequest::new(SubmittedTurnInput::UserInput {
        content: text_input("hello"),
        client_id: None,
    })
    .with_thread_settings(codex_protocol::protocol::ThreadSettingsOverrides {
        environments: Some(local_requests(config.cwd.clone())),
        approval_policy: Some(config.permissions.approval_policy.value()),
        approvals_reviewer: Some(codex_config::types::ApprovalsReviewer::AutoReview),
        ..Default::default()
    });
    let submission = super::turn_input::handle(
        &session,
        request,
        TurnInputMode::StartOrSteer,
        "sub-1".to_string(),
    )
    .await
    .expect("user turn submission should be valid");
    assert!(
        matches!(submission, TurnInputSubmission::Started { .. }),
        "{submission:?}"
    );

    let started = running_turn_context(&session).await;
    assert_eq!(
        started.config.approvals_reviewer,
        codex_config::types::ApprovalsReviewer::AutoReview
    );
    let step = session
        .capture_step_context(Arc::clone(&started), &CancellationToken::new())
        .await
        .expect("capture the started turn's step");
    assert!(
        !session.mcp_refresh.is_pending(),
        "the user-turn path must publish the refreshed MCP state before execution"
    );
    let published = session
        .services
        .mcp_runtime
        .current_config()
        .expect("the user-turn path should publish an MCP runtime config");
    let approvals_reviewer = {
        let state = session.state.lock().await;
        state.session_configuration.step_settings.approvals_reviewer
    };
    assert_eq!(
        (
            approvals_reviewer,
            published.approvals_reviewer,
            step.mcp.config().approvals_reviewer,
        ),
        (
            codex_config::types::ApprovalsReviewer::AutoReview,
            codex_config::types::ApprovalsReviewer::AutoReview,
            codex_config::types::ApprovalsReviewer::AutoReview,
        )
    );
    session.abort_all_tasks(TurnAbortReason::Interrupted).await;
}

/// Main's `running_turn_notes_the_mail_it_takes`.
#[tokio::test]
async fn running_turn_notes_the_mail_it_takes() {
    let (sess, tc, _rx) = make_session_and_context_with_rx().await;
    spawn_never_ending(&sess, &tc, TaskKind::Regular).await;
    let mut mail = InterAgentCommunication::new(
        AgentPath::root(),
        AgentPath::try_from("/root/worker").expect("agent path"),
        Vec::new(),
        "taken mid-turn".to_string(),
        /*trigger_turn*/ true,
    );
    let message_id = mail.ensure_message_identity().to_string();
    sess.input_queue
        .enqueue_mailbox_communication(mail.clone().into(), TurnStartOptions::default())
        .await;

    let (input, _) = sess
        .input_queue
        .get_pending_input_for_running_task(&sess, &tc.sub_id)
        .await;
    assert_eq!(input, vec![TurnInput::InterAgentCommunication(mail.into())]);
    let turn_state = Arc::clone(
        &sess
            .active_turn
            .lock()
            .await
            .as_ref()
            .expect("active turn")
            .turn_state,
    );
    assert_eq!(
        turn_state
            .lock()
            .await
            .take_unrecorded_mailbox_message_ids(&[]),
        vec![message_id]
    );
    sess.abort_all_tasks(TurnAbortReason::Replaced).await;
}

/// Main's `interrupted_input_cannot_be_spliced_into_review_task`: input an interrupted turn held
/// is re-admitted through submission admission (#1927); a review turn cannot take it, so it waits
/// as that turn's deferred input instead of joining the review.
#[tokio::test]
async fn interrupted_input_cannot_be_spliced_into_review_task() {
    let (sess, tc, _rx) = make_session_and_context_with_rx().await;
    spawn_never_ending(&sess, &tc, TaskKind::Regular).await;
    let held = TurnInput::UserInput {
        content: text_input("deferred user work"),
        client_id: None,
        metadata: Default::default(),
    };
    sess.defer_held_input(held.clone(), /*final_output_json_schema*/ None)
        .await;
    sess.abort_all_tasks(TurnAbortReason::Replaced).await;
    let review = sess.new_default_turn().await;
    spawn_never_ending(&sess, &review, TaskKind::Review).await;
    assert_eq!(
        sess.input_queue
            .get_pending_input_for_running_task(&sess, &review.sub_id)
            .await
            .0,
        Vec::new()
    );
    let (interrupted, final_output_json_schema) =
        sess.input_queue.next_interrupted_deferred_input().await;
    assert!(
        matches!(&interrupted, TurnInput::UserInput { content, .. } if *content == text_input("deferred user work")),
        "{interrupted:?}"
    );
    super::handlers::readmit_interrupted_input(&sess, interrupted, final_output_json_schema).await;
    assert_eq!(
        sess.input_queue
            .get_pending_input_for_running_task(&sess, &review.sub_id)
            .await
            .0,
        Vec::new()
    );
    let turn_state = Arc::clone(
        &sess
            .active_turn
            .lock()
            .await
            .as_ref()
            .expect("active turn")
            .turn_state,
    );
    assert_eq!(
        sess.input_queue
            .subscribe_activity(Some(&turn_state))
            .await
            .1,
        None
    );
    let deferred = sess
        .input_queue
        .take_pending_input_for_turn_state(&turn_state)
        .await;
    let [(TurnInput::UserInput { content, .. }, None)] = deferred.as_slice() else {
        panic!("expected the held input as the review turn's deferred input, got {deferred:?}");
    };
    assert_eq!(*content, text_input("deferred user work"));
    sess.abort_all_tasks(TurnAbortReason::Interrupted).await;
}

fn sleep_invocation(sess: &Arc<Session>, tc: &Arc<TurnContext>, call_id: String) -> ToolInvocation {
    ToolInvocation {
        session: Arc::clone(sess),
        turn: Arc::clone(tc),
        step_context: StepContext::for_test(Arc::clone(tc)),
        cancellation_token: CancellationToken::new(),
        tracker: Arc::new(Mutex::new(TurnDiffTracker::new())),
        call_id,
        tool_name: codex_tools::ToolName::namespaced("clock", "sleep"),
        source: crate::tools::context::ToolCallSource::Direct,
        payload: ToolPayload::Function {
            arguments: json!({"duration_ms": 10}).to_string(),
        },
        native_call: None,
        environments: Default::default(),
    }
}

/// Main's `deferred_input_does_not_interrupt_turn_local_sleep` (main deferred the user's input;
/// upstream defers held input the same way): deferred input, in the turn or in the interrupted
/// FIFO, does not wake the running turn's sleep; consumable input still does.
#[tokio::test]
async fn deferred_input_does_not_interrupt_turn_local_sleep() {
    let (sess, tc, _rx) = make_session_and_context_with_rx().await;
    spawn_never_ending(&sess, &tc, TaskKind::Regular).await;
    let turn_state = Arc::clone(
        &sess
            .active_turn
            .lock()
            .await
            .as_ref()
            .expect("active turn")
            .turn_state,
    );
    let (mut activity_rx, activity) = sess.input_queue.subscribe_activity(Some(&turn_state)).await;
    assert_eq!(activity, None);
    sess.defer_held_input(
        TurnInput::UserInput {
            content: text_input("later turn"),
            client_id: None,
            metadata: Default::default(),
        },
        /*final_output_json_schema*/ None,
    )
    .await;
    // Both turn-local deferral and the interrupted session FIFO.
    for interrupted in [false, true] {
        if interrupted {
            #[expect(
                clippy::await_holding_invalid_type,
                reason = "Clearing pending input must stay atomic with the active turn"
            )]
            let active = sess.active_turn.lock().await;
            sess.input_queue
                .clear_pending(active.as_ref().expect("active turn"))
                .await;
            drop(active);
            assert!(sess.input_queue.has_pending_input(&sess.active_turn).await);
        }
        assert!(!activity_rx.has_changed().expect("activity channel open"));
        assert_eq!(
            sess.input_queue
                .subscribe_activity(Some(&turn_state))
                .await
                .1,
            None
        );
        let output = crate::tools::handlers::SleepHandler
            .handle(sleep_invocation(
                &sess,
                &tc,
                format!("deferred-sleep-{interrupted}"),
            ))
            .await
            .expect("sleep runs");
        assert!(
            output.log_output().contains("Sleep completed."),
            "{}",
            output.log_output()
        );
    }
    // Consumable input still wakes the same subscription.
    sess.input_queue
        .extend_pending_input_and_accept_mailbox_delivery_for_turn_state(
            &turn_state,
            vec![TurnInput::UserInput {
                content: text_input("this turn"),
                client_id: None,
                metadata: Default::default(),
            }],
        )
        .await;
    activity_rx.changed().await.expect("activity channel open");
    assert_eq!(*activity_rx.borrow_and_update(), InputQueueActivity::Steer);
    sess.abort_all_tasks(TurnAbortReason::Interrupted).await;
}

/// Main's `deferred_mail_does_not_interrupt_turn_local_sleep`: mail deferred to the next turn
/// (the running turn's permissions changed) does not wake the running turn's sleep, even after
/// the earlier permissions are restored; mail the turn may take still does.
#[tokio::test]
async fn deferred_mail_does_not_interrupt_turn_local_sleep() {
    for trigger_turn in [false, true] {
        let (sess, tc, _rx) = make_session_and_context_with_rx().await;
        spawn_never_ending(&sess, &tc, TaskKind::Regular).await;
        let turn_state = Arc::clone(
            &sess
                .active_turn
                .lock()
                .await
                .as_ref()
                .expect("active turn")
                .turn_state,
        );
        let (mut activity_rx, activity) =
            sess.input_queue.subscribe_activity(Some(&turn_state)).await;
        assert_eq!(activity, None);
        let admitted_policy = tc.approval_policy();
        set_approval_policy(&sess, AskForApproval::UnlessTrusted).await;
        let mail = InterAgentCommunication::new(
            AgentPath::root(),
            AgentPath::root(),
            Vec::new(),
            "deferred peer work".into(),
            trigger_turn,
        );
        sess.input_queue
            .enqueue_mailbox_communication_for_session(
                &sess,
                mail.clone().into(),
                TurnStartOptions::default(),
            )
            .await;
        // Restoring authority must not release mail already bound to the next turn.
        set_approval_policy(&sess, admitted_policy).await;
        assert!(!activity_rx.has_changed().expect("activity channel open"));
        assert_eq!(
            sess.input_queue
                .subscribe_activity(Some(&turn_state))
                .await
                .1,
            None
        );
        assert_eq!(
            sess.input_queue
                .subscribe_activity(/*turn_state*/ None)
                .await
                .1,
            Some(InputQueueActivity::Mailbox)
        );
        let next_turn_state = Mutex::new(crate::state::TurnState::default());
        assert_eq!(
            sess.input_queue
                .subscribe_activity(Some(&next_turn_state))
                .await
                .1,
            Some(InputQueueActivity::Mailbox)
        );
        for consumable in [false, true] {
            if consumable {
                sess.input_queue
                    .enqueue_mailbox_communication_for_session(
                        &sess,
                        mail.clone().into(),
                        TurnStartOptions::default(),
                    )
                    .await;
                activity_rx.changed().await.expect("activity channel open");
                assert_eq!(
                    *activity_rx.borrow_and_update(),
                    InputQueueActivity::Mailbox
                );
            }
            let output = crate::tools::handlers::SleepHandler
                .handle(sleep_invocation(
                    &sess,
                    &tc,
                    format!("mail-sleep-{trigger_turn}-{consumable}"),
                ))
                .await
                .expect("sleep runs");
            let expected = if consumable {
                "Sleep interrupted by new input."
            } else {
                "Sleep completed."
            };
            assert!(
                output.log_output().contains(expected),
                "{}",
                output.log_output()
            );
        }
        assert_eq!(
            sess.input_queue
                .get_pending_input_for_running_task(&sess, &tc.sub_id)
                .await
                .0,
            vec![TurnInput::InterAgentCommunication(mail.clone().into())]
        );
        sess.abort_all_tasks(TurnAbortReason::Replaced).await;
        assert_eq!(
            sess.input_queue.drain_mailbox_input_items().await.0,
            vec![TurnInput::InterAgentCommunication(mail.into())]
        );
    }
}

/// Main's `authorization_changed_public_injection_waits_for_later_model_turn`.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn authorization_changed_public_injection_waits_for_later_model_turn() -> anyhow::Result<()> {
    assert_authorization_changed_model_admission(NewWork::Injection).await
}

/// End-to-end form of the returned-input refusal: user input sent after the running turn's
/// permissions changed is refused with `AuthorizationChanged`, never reaches that turn's model
/// requests and, sent again, reaches the next turn.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn authorization_changed_returned_input_never_reaches_superseded_turn() -> anyhow::Result<()>
{
    assert_authorization_changed_model_admission(NewWork::UserInput { schema: None }).await
}

/// Main's `authorization_changed_natural_completion_preserves_model_output_schema`. Upstream
/// returns the user's input (with its schema) to the client; submitted again after the turn it
/// keeps its output schema.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn authorization_changed_returned_input_keeps_model_output_schema() -> anyhow::Result<()> {
    assert_authorization_changed_model_admission(NewWork::UserInput {
        schema: Some(json!({
            "type": "object",
            "properties": {"answer": {"type": "string"}},
            "required": ["answer"],
            "additionalProperties": false
        })),
    })
    .await
}

/// New work sent while the running turn's permissions changed.
enum NewWork {
    /// Public response-item injection (deferred by Core, recorded at the turn's end).
    Injection,
    /// User input (returned to the client, sent again after the turn).
    UserInput { schema: Option<serde_json::Value> },
}

/// Real model requests: new work sent after the running turn's permissions changed never
/// enters that turn's requests and is not lost.
async fn assert_authorization_changed_model_admission(new_work: NewWork) -> anyhow::Result<()> {
    use core_test_support::responses::ev_function_call_with_namespace;
    use core_test_support::streaming_sse::start_streaming_sse_server;

    let (release, gate) = tokio::sync::oneshot::channel();
    let (server, _completions) = start_streaming_sse_server(vec![
        vec![StreamingSseChunk {
            gate: Some(gate),
            body: sse(vec![
                ev_response_created("old-tool"),
                ev_function_call_with_namespace(
                    "boundary-sleep",
                    "clock",
                    "sleep",
                    r#"{"duration_ms": 1}"#,
                ),
                ev_completed("old-tool"),
            ]),
        }],
        vec![StreamingSseChunk {
            gate: None,
            body: sse(vec![
                ev_response_created("old-end"),
                ev_completed("old-end"),
            ]),
        }],
        vec![StreamingSseChunk {
            gate: None,
            body: sse(vec![
                ev_response_created("new-end"),
                ev_completed("new-end"),
            ]),
        }],
    ])
    .await;
    let test = test_codex()
        .with_model("gpt-5.4")
        .with_config(|config| {
            config
                .features
                .enable(Feature::CurrentTimeReminder)
                .expect("enable the current-time reminder");
            config
                .current_time_reminder
                .get_or_insert_with(Default::default)
                .sleep_tool = true;
        })
        .build_with_streaming_server(&server)
        .await?;
    test.codex
        .start_or_steer_turn(
            TurnInputRequest::user_input(text_input("initial turn")).with_thread_settings(
                codex_protocol::protocol::ThreadSettingsOverrides {
                    approval_policy: Some(AskForApproval::Never),
                    ..Default::default()
                },
            ),
        )
        .await?;
    tokio::time::timeout(
        StdDuration::from_secs(10),
        server.wait_for_request_count(/*count*/ 1),
    )
    .await?;
    test.codex
        .submit(Op::ThreadSettings {
            thread_settings: codex_protocol::protocol::ThreadSettingsOverrides {
                approval_policy: Some(AskForApproval::UnlessTrusted),
                ..Default::default()
            },
            reply: None,
        })
        .await?;
    wait_for_event(&test.codex, |event| {
        matches!(event, EventMsg::ThreadSettingsApplied(applied)
            if applied.thread_settings.approval_policy == AskForApproval::UnlessTrusted)
    })
    .await;
    let injected = "pf83 new work must wait for current authorization";
    let schema = match &new_work {
        NewWork::UserInput { schema } => schema.clone(),
        NewWork::Injection => None,
    };
    if let NewWork::UserInput { schema } = &new_work {
        let returned = test
            .codex
            .start_or_steer_turn(TurnInputRequest::user_input(text_input(injected)).on_start(
                TurnStartOptions {
                    final_output_json_schema: schema.clone(),
                    ..Default::default()
                },
            ))
            .await?;
        // A schema that differs from the running turn's is reported before the permissions.
        let expected = if schema.is_some() {
            NotSubmittedReason::ActiveTurnOutputSchemaMismatch
        } else {
            NotSubmittedReason::AuthorizationChanged
        };
        assert_eq!(
            returned,
            TurnInputSubmission::NotSubmitted { reason: expected }
        );
    } else {
        test.codex
            .inject_if_running(vec![ResponseItem::Message {
                encrypted_content: None,
                status: None,
                id: None,
                role: "user".into(),
                content: vec![ContentItem::InputText {
                    text: injected.into(),
                }],
                phase: None,
                internal_chat_message_metadata_passthrough: None,
            }])
            .await
            .expect("public injection accepted for later delivery");
    }
    release.send(()).expect("release old model stream");
    wait_for_event(&test.codex, |event| {
        matches!(event, EventMsg::TurnComplete(_))
    })
    .await;
    // Injected items are recorded at completion and start no turn; returned input is submitted
    // again by the client, with its schema.
    let next_turn = match &new_work {
        NewWork::UserInput { schema } => TurnInputRequest::user_input(text_input(injected))
            .on_start(TurnStartOptions {
                final_output_json_schema: schema.clone(),
                ..Default::default()
            }),
        NewWork::Injection => TurnInputRequest::user_input(text_input("read deferred items")),
    };
    assert!(matches!(
        test.codex.start_or_steer_turn(next_turn).await?,
        TurnInputSubmission::Started { .. }
    ));
    wait_for_event(&test.codex, |event| {
        matches!(event, EventMsg::TurnComplete(_))
    })
    .await;
    let bodies: Vec<serde_json::Value> = server
        .requests()
        .await
        .iter()
        .map(|body| serde_json::from_slice(body).expect("model request JSON"))
        .collect();
    assert_eq!(bodies.len(), 3);
    assert!(
        bodies[..2]
            .iter()
            .all(|body| !body["input"].to_string().contains(injected)),
        "new work entered the superseded turn"
    );
    assert!(
        bodies[2]["input"].to_string().contains(injected),
        "deferred work was lost"
    );
    if let Some(schema) = schema {
        assert_eq!(bodies[2]["text"]["format"]["schema"], schema);
    }
    test.codex.submit(Op::Shutdown).await?;
    server.shutdown().await;
    Ok(())
}

/// Main's `authorization_changed_interrupted_input_wakes_fresh_admission` (main deferred the
/// user's input; upstream holds input the same way): after an interrupt, held input goes back
/// through submission admission (#1927) and starts a fresh turn under the latest permissions,
/// keeping its output schema.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn authorization_changed_interrupted_input_wakes_fresh_admission() {
    let (sess, tc, rx) = make_session_and_context_with_submission_loop_and_rx().await;
    spawn_never_ending(&sess, &tc, TaskKind::Regular).await;
    set_approval_policy(&sess, AskForApproval::UnlessTrusted).await;
    let items = text_input("resume my deferred message");
    let schema = json!({"type": "object", "properties": {"answer": {"type": "string"}}, "required": ["answer"], "additionalProperties": false});
    sess.defer_held_input(
        TurnInput::UserInput {
            content: items.clone(),
            client_id: Some("interrupted-client".into()),
            metadata: Default::default(),
        },
        Some(schema.clone()),
    )
    .await;
    sess.abort_all_tasks(TurnAbortReason::Interrupted).await;
    expect_user_message_item_started(&rx, &items).await;
    let next = running_turn_context(&sess).await;
    assert_ne!(next.sub_id, tc.sub_id);
    assert_eq!(next.approval_policy(), AskForApproval::UnlessTrusted);
    assert_eq!(next.final_output_json_schema, Some(schema));
    sess.abort_all_tasks(TurnAbortReason::Interrupted).await;
}

/// Main's `authorization_changed_turn_receives_code_mode_notify_and_user_shell_output`, user
/// shell half: output of the turn's own work (a user shell command run beside the turn) still
/// reaches the turn after its permissions changed; it is not new work.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn authorization_changed_turn_receives_user_shell_output() {
    let (sess, tc, rx) = make_session_and_context_with_rx().await;
    spawn_never_ending(&sess, &tc, TaskKind::Regular).await;
    set_approval_policy(&sess, AskForApproval::UnlessTrusted).await;
    crate::tasks::execute_user_shell_command(
        Arc::clone(&sess),
        Arc::clone(&tc),
        "echo pf83-shell-output".into(),
        /*timeout_ms*/ None,
        CancellationToken::new(),
        crate::tasks::UserShellCommandMode::ActiveTurnAuxiliary,
    )
    .await;
    loop {
        let event = tokio::time::timeout(StdDuration::from_secs(10), rx.recv())
            .await
            .expect("expected the shell command to end")
            .expect("channel open");
        if let EventMsg::ExecCommandEnd(event) = event.msg {
            assert_eq!(event.exit_code, 0);
            assert_eq!(event.aggregated_output.trim(), "pf83-shell-output");
            break;
        }
    }
    let (pending, _) = sess
        .input_queue
        .get_pending_input_for_running_task(&sess, &tc.sub_id)
        .await;
    let [TurnInput::ResponseItem(shell_output)] = pending.as_slice() else {
        panic!("expected the shell output as the turn's input, got {pending:?}");
    };
    assert!(
        serde_json::to_string(&shell_output.clone().into_item())
            .expect("serialize shell output")
            .contains("pf83-shell-output")
    );
    let turn_state = Arc::clone(
        &sess
            .active_turn
            .lock()
            .await
            .as_ref()
            .expect("active turn")
            .turn_state,
    );
    assert!(
        sess.input_queue
            .take_pending_input_for_turn_state(&turn_state)
            .await
            .is_empty()
    );
    assert_eq!(
        running_turn_context(&sess).await.approval_policy(),
        tc.approval_policy()
    );
    sess.abort_all_tasks(TurnAbortReason::Interrupted).await;
}

/// Input racing the end of a turn that cannot take it: either it is returned to the client or it
/// starts a fresh turn; it never errors, never joins the ending turn and, when started, runs.
async fn assert_boundary_race_starts_fresh_or_returns(kind: TaskKind, authorization_changed: bool) {
    let (sess, tc, rx) = make_session_and_context_with_submission_loop_and_rx().await;
    spawn_never_ending(&sess, &tc, kind).await;
    if authorization_changed {
        set_approval_policy(&sess, AskForApproval::UnlessTrusted).await;
    }
    while rx.try_recv().is_ok() {}
    let items = text_input("racing the turn boundary");
    let submit = tokio::spawn({
        let sess = Arc::clone(&sess);
        let items = items.clone();
        async move { submit_start_or_steer(&sess, items).await }
    });
    sess.on_task_finished(Arc::clone(&tc), Ok(None)).await;
    let submission = submit.await.expect("submission task should not panic");
    match submission {
        TurnInputSubmission::Started { turn_id, .. } => {
            assert_ne!(turn_id, tc.sub_id);
            next_turn_started(&rx, &tc.sub_id).await;
            expect_user_message_item_started(&rx, &items).await;
            if authorization_changed {
                assert_eq!(
                    running_turn_context(&sess).await.approval_policy(),
                    AskForApproval::UnlessTrusted
                );
            }
        }
        TurnInputSubmission::NotSubmitted {
            reason:
                NotSubmittedReason::AuthorizationChanged
                | NotSubmittedReason::ActiveTurnNotSteerable { .. },
        } => {
            assert!(!sess.input_queue.has_pending_input(&sess.active_turn).await);
            while let Ok(event) = rx.try_recv() {
                assert!(
                    !matches!(event.msg, EventMsg::Error(_)),
                    "a returned input should not emit an error: {:?}",
                    event.msg
                );
            }
        }
        other => panic!("unexpected submission at the turn boundary: {other:?}"),
    }
    sess.abort_all_tasks(TurnAbortReason::Interrupted).await;
}

/// Main's `authorization_changed_defer_race_starts_fresh_turn_without_error`. Main paused its
/// deferral at a test hook; upstream returns the input instead of deferring it, so the race is
/// between the refusal and the turn's end.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn authorization_changed_boundary_race_starts_fresh_turn_or_returns_input() {
    assert_boundary_race_starts_fresh_or_returns(
        TaskKind::Regular,
        /*authorization_changed*/ true,
    )
    .await;
}

/// Main's `non_steerable_defer_race_starts_fresh_turn_without_error`, as above for a compaction.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn non_steerable_boundary_race_starts_fresh_turn_or_returns_input() {
    assert_boundary_race_starts_fresh_or_returns(
        TaskKind::Compact,
        /*authorization_changed*/ false,
    )
    .await;
}

/// Replaces the session's active-turn slot while the finishing turn's terminal hooks run.
struct ReplaceActiveTurnOnStop {
    session: Arc<std::sync::OnceLock<std::sync::Weak<Session>>>,
    replaced: Arc<std::sync::atomic::AtomicBool>,
}

impl codex_extension_api::TurnLifecycleContributor for ReplaceActiveTurnOnStop {
    fn on_turn_stop<'a>(
        &'a self,
        _input: codex_extension_api::TurnStopInput<'a>,
    ) -> codex_extension_api::ExtensionFuture<'a, ()> {
        Box::pin(async move {
            if let Some(session) = self.session.get().and_then(std::sync::Weak::upgrade) {
                *session.active_turn.lock().await = Some(crate::state::ActiveTurn::default());
                self.replaced
                    .store(true, std::sync::atomic::Ordering::SeqCst);
            }
        })
    }
}

/// Main's `task_finish_submits_follow_up_when_active_turn_was_replaced`: main paused the finish
/// on its guardian circuit-breaker lock to replace the active turn; upstream's finish has no such
/// point, so a turn-stop lifecycle hook (which runs during the finish, before the terminal event)
/// replaces the slot. The steered input the finished turn detached still starts its own turn.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn task_finish_submits_follow_up_when_active_turn_was_replaced() {
    let session_slot = Arc::new(std::sync::OnceLock::new());
    let replaced = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let mut builder = codex_extension_api::ExtensionRegistryBuilder::<crate::config::Config>::new();
    builder.turn_lifecycle_contributor(Arc::new(ReplaceActiveTurnOnStop {
        session: Arc::clone(&session_slot),
        replaced: Arc::clone(&replaced),
    }));
    let codex_home = tempfile::tempdir().expect("create temp dir");
    let test_cwd = kept_test_cwd();
    let (sess, tc, rx) = make_session_and_context_with_extensions_and_rx(
        CodexAuth::from_api_key("Test API Key"),
        Vec::new(),
        codex_home.path(),
        &test_cwd,
        |_config| {},
        /*start_submission_loop*/ true,
        Arc::new(builder.build()),
    )
    .await;
    session_slot
        .set(Arc::downgrade(&sess))
        .expect("session slot set once");
    sess.spawn_task(
        Arc::clone(&tc),
        Vec::new(),
        NeverEndingTask {
            kind: TaskKind::Regular,
            listen_to_cancellation_token: false,
        },
    )
    .await;
    while rx.try_recv().is_ok() {}
    let pending_user_input = text_input("survives finish replacement");
    assert!(matches!(
        submit_start_or_steer(&sess, pending_user_input.clone()).await,
        TurnInputSubmission::Steered { .. }
    ));

    sess.on_task_finished(Arc::clone(&tc), /*task_result*/ Ok(None))
        .await;
    assert!(
        replaced.load(std::sync::atomic::Ordering::SeqCst),
        "the active-turn slot was replaced during the finish"
    );

    // The finished turn's terminal event, then the follow-up turn the detached input starts.
    let first = recv_skipping_mcp_startup(&rx, "expected original turn complete event").await;
    assert!(
        matches!(
            &first.msg,
            EventMsg::TurnComplete(TurnCompleteEvent { turn_id, .. }) if *turn_id == tc.sub_id
        ),
        "expected the original turn's completion, got {:?}",
        first.msg
    );
    next_turn_started(&rx, &tc.sub_id).await;
    expect_user_message_item_started(&rx, &pending_user_input).await;
    sess.abort_all_tasks(TurnAbortReason::Interrupted).await;
}
