//! #2010 reproduction: every stop path removes the active-turn slot before it waits
//! for the stopped task. A task that reaches its next input poll in that window (as `run_turn`
//! does at the top of its loop) must not take mailbox mail it can no longer note; the mail must
//! still be queued once the stop has finished. #2046: nor may it take input or mail from a newer
//! turn that started while a background stop (guardian's) was still waiting for it.
use super::*;
use crate::session::tests::make_session_and_context_with_rx;
use crate::state::TaskKind;
use crate::tasks::SessionTask;
use crate::tasks::SessionTaskResult;
use codex_protocol::AgentPath;
use codex_protocol::protocol::InterAgentCommunication;
use codex_protocol::protocol::TurnAbortReason;
use codex_protocol::turn_input::TurnInput as SubmittedTurnInput;
use codex_protocol::turn_input::TurnInputMode;
use codex_protocol::turn_input::TurnInputRequest;
use codex_protocol::turn_input::TurnInputSubmission;
use codex_protocol::turn_input::TurnStartOptions;
use pretty_assertions::assert_eq;
use std::sync::Arc;
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;

type Polled = Arc<std::sync::Mutex<Option<Vec<TurnInput>>>>;

/// Waits for cancellation, then polls for input once, like `run_turn`'s next loop iteration.
struct StalePollTask {
    polled: Polled,
}

impl SessionTask for StalePollTask {
    fn kind(&self) -> TaskKind {
        TaskKind::Regular
    }

    fn span_name(&self) -> &'static str {
        "session_task.stale_poll"
    }

    async fn run(
        self: Arc<Self>,
        session: Arc<Session>,
        ctx: Arc<TurnContext>,
        _input: Vec<TurnInput>,
        cancellation_token: CancellationToken,
    ) -> SessionTaskResult {
        cancellation_token.cancelled().await;
        let (input, _) = session
            .input_queue
            .get_pending_input_for_running_task(&session, &ctx.sub_id)
            .await;
        *self.polled.lock().expect("polled lock") = Some(input);
        Ok(None)
    }
}

#[derive(Clone, Copy, Debug)]
enum StopPath {
    /// Interrupt, kill switch, and quit.
    Interrupt,
    /// A new turn replacing the running one.
    Replaced,
    /// Guardian's targeted stop path.
    Guardian,
}

async fn stale_poll_after(stop: StopPath) -> (Option<Vec<TurnInput>>, Vec<TurnInput>) {
    let (sess, turn, _rx) = make_session_and_context_with_rx().await;
    let polled = Polled::default();
    sess.spawn_task(
        Arc::clone(&turn),
        Vec::new(),
        StalePollTask {
            polled: Arc::clone(&polled),
        },
    )
    .await;

    // Queue-only, so the interrupt path does not start a model turn afterwards.
    let mail = InterAgentCommunication::new(
        AgentPath::root(),
        AgentPath::try_from("/root/worker").expect("agent path"),
        Vec::new(),
        "arrived while the turn was stopping".to_string(),
        /*trigger_turn*/ false,
    );
    sess.input_queue
        .enqueue_mailbox_communication(mail.into(), TurnStartOptions::default())
        .await;

    match stop {
        StopPath::Interrupt => sess.abort_all_tasks(TurnAbortReason::Interrupted).await,
        StopPath::Replaced => sess.abort_all_tasks(TurnAbortReason::Replaced).await,
        StopPath::Guardian => {
            assert!(
                sess.abort_turn_if_active(
                    &turn.sub_id,
                    TurnAbortReason::Interrupted,
                    /*error*/ None,
                )
                .await
            );
        }
    }

    let stale = polled.lock().expect("polled lock").take();
    let (still_queued, _) = sess.input_queue.drain_mailbox_input_items().await;
    (stale, still_queued)
}

#[tokio::test]
async fn stopped_turn_poll_leaves_mail_queued_on_every_stop_path() {
    for stop in [StopPath::Interrupt, StopPath::Replaced, StopPath::Guardian] {
        let (stale, still_queued) = stale_poll_after(stop).await;
        assert_eq!(
            stale.map(|input| input.len()),
            Some(0),
            "{stop:?}: the stopped task took mail"
        );
        assert_eq!(still_queued.len(), 1, "{stop:?}: the mail was lost");
    }
}

/// #2046: guardian stops a turn in a background task (guardian/review.rs), so the session can
/// start a new turn while the stop still waits for the stopped task. This task signals once it is
/// cancelled, then polls for input only when the test says so, after the new turn has started.
struct GatedStalePollTask {
    cancelled: Arc<Notify>,
    go: Arc<Notify>,
    polled: Polled,
}

impl SessionTask for GatedStalePollTask {
    fn kind(&self) -> TaskKind {
        TaskKind::Regular
    }

    fn span_name(&self) -> &'static str {
        "session_task.gated_stale_poll"
    }

    async fn run(
        self: Arc<Self>,
        session: Arc<Session>,
        ctx: Arc<TurnContext>,
        _input: Vec<TurnInput>,
        cancellation_token: CancellationToken,
    ) -> SessionTaskResult {
        cancellation_token.cancelled().await;
        self.cancelled.notify_one();
        self.go.notified().await;
        let (input, _) = session
            .input_queue
            .get_pending_input_for_running_task(&session, &ctx.sub_id)
            .await;
        *self.polled.lock().expect("polled lock") = Some(input);
        Ok(None)
    }
}

/// The user's new turn: runs until it is stopped.
struct IdleTask;

impl SessionTask for IdleTask {
    fn kind(&self) -> TaskKind {
        TaskKind::Regular
    }

    fn span_name(&self) -> &'static str {
        "session_task.idle"
    }

    async fn run(
        self: Arc<Self>,
        _session: Arc<Session>,
        _ctx: Arc<TurnContext>,
        _input: Vec<TurnInput>,
        cancellation_token: CancellationToken,
    ) -> SessionTaskResult {
        cancellation_token.cancelled().await;
        Ok(None)
    }
}

#[tokio::test]
async fn guardian_stopped_turn_leaves_a_new_turns_input_and_mail_alone() {
    let (sess, old_turn, _rx) = make_session_and_context_with_rx().await;
    let cancelled = Arc::new(Notify::new());
    let go = Arc::new(Notify::new());
    let polled = Polled::default();
    sess.spawn_task(
        Arc::clone(&old_turn),
        Vec::new(),
        GatedStalePollTask {
            cancelled: Arc::clone(&cancelled),
            go: Arc::clone(&go),
            polled: Arc::clone(&polled),
        },
    )
    .await;

    // Guardian's stop, detached as in guardian/review.rs.
    let stop = tokio::spawn({
        let sess = Arc::clone(&sess);
        let turn_id = old_turn.sub_id.clone();
        async move {
            sess.abort_turn_if_active(&turn_id, TurnAbortReason::Interrupted, /*error*/ None)
                .await
        }
    });
    cancelled.notified().await;
    let window_opened = std::time::Instant::now();

    // The active slot is empty now, so the user's next message starts a new turn, then steers it;
    // an agent message arrives for it too.
    let new_turn = sess.new_default_turn().await;
    sess.spawn_task(Arc::clone(&new_turn), Vec::new(), IdleTask)
        .await;
    // Main called the private `steer_input`; upstream steers through
    // `turn_input::handle` in steer-only mode.
    let steered = super::turn_input::handle(
        &sess,
        TurnInputRequest::new(SubmittedTurnInput::UserInput {
            content: vec![UserInput::Text {
                text: "steer meant for the new turn".to_string(),
                text_elements: Vec::new(),
            }],
            client_id: None,
        }),
        TurnInputMode::Steer {
            expected_turn_id: new_turn.sub_id.clone(),
        },
        "steer-new-turn".to_string(),
    )
    .await
    .expect("steer the new turn");
    assert!(
        matches!(steered, TurnInputSubmission::Steered { .. }),
        "{steered:?}"
    );
    sess.input_queue
        .enqueue_mailbox_communication(
            InterAgentCommunication::new(
                AgentPath::root(),
                AgentPath::try_from("/root/worker").expect("agent path"),
                Vec::new(),
                "mail meant for the new turn".to_string(),
                /*trigger_turn*/ false,
            )
            .into(),
            TurnStartOptions::default(),
        )
        .await;
    // Guardian's stop waits 100 ms for the stopped task before aborting it; the stale poll must
    // happen inside that window or this test proves nothing.
    assert!(
        window_opened.elapsed() < std::time::Duration::from_millis(80),
        "new turn took {:?} to start; the stale poll would miss guardian's wait",
        window_opened.elapsed()
    );
    go.notify_one();
    assert!(stop.await.expect("guardian stop task"));

    let stale = polled.lock().expect("polled lock").take();
    assert_eq!(
        stale.map(|input| input.len()),
        Some(0),
        "the guardian-stopped turn took the new turn's input or mail"
    );
    let (new_turn_input, _) = sess
        .input_queue
        .get_pending_input_for_running_task(&sess, &new_turn.sub_id)
        .await;
    assert_eq!(new_turn_input.len(), 2, "the new turn lost input or mail");
    sess.abort_all_tasks(TurnAbortReason::Interrupted).await;
}
