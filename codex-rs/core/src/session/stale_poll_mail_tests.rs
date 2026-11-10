//! #2010 reproduction (rev-2033): every stop path removes the active-turn slot before it waits
//! for the stopped task. A task that reaches its next input poll in that window (as `run_turn`
//! does at the top of its loop) must not take mailbox mail it can no longer note; the mail must
//! still be queued once the stop has finished.
use super::*;
use crate::session::tests::make_session_and_context_with_rx;
use crate::state::TaskKind;
use crate::tasks::SessionTask;
use crate::tasks::SessionTaskContext;
use crate::tasks::SessionTaskResult;
use codex_protocol::AgentPath;
use codex_protocol::protocol::InterAgentCommunication;
use codex_protocol::protocol::TurnAbortReason;
use pretty_assertions::assert_eq;
use std::sync::Arc;
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
        session: Arc<SessionTaskContext>,
        _ctx: Arc<TurnContext>,
        _input: Vec<TurnInput>,
        cancellation_token: CancellationToken,
    ) -> SessionTaskResult {
        cancellation_token.cancelled().await;
        let session = session.clone_session();
        let (input, _) = session
            .input_queue
            .get_pending_input_for_running_task(&session.active_turn)
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
        .enqueue_mailbox_communication(mail, /*parent_turn_id*/ None)
        .await;

    match stop {
        StopPath::Interrupt => sess.abort_all_tasks(TurnAbortReason::Interrupted).await,
        StopPath::Replaced => sess.abort_all_tasks(TurnAbortReason::Replaced).await,
        StopPath::Guardian => {
            assert!(
                sess.abort_turn_if_active(&turn.sub_id, TurnAbortReason::Interrupted)
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
