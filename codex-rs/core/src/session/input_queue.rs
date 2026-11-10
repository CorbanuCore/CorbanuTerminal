use super::Session;
use crate::agent::api::AgentControl;
use crate::agent_communication::PENDING_MAILBOX_MESSAGES;
use crate::state::ActiveTurn;
use crate::state::MailboxDeliveryPhase;
use crate::state::TurnState;
use codex_diagnostics::GaugeGuard;
use codex_history::ResponseItemEnvelope;
use codex_protocol::ThreadId;
use codex_protocol::models::ResponseItem;
use codex_protocol::protocol::InterAgentCommunication;
use codex_protocol::turn_input::TurnStartOptions;
use codex_protocol::user_input::UserInput;
use serde::Deserialize;
use serde::Serialize;
use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::Weak;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use tokio::sync::Mutex;
use tokio::sync::Notify;
use tokio::sync::watch;
use tokio_util::sync::CancellationToken;
use tokio_util::task::AbortOnDropHandle;

/// Host capture metadata belonging to one input, including steers within another turn.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserInputMetadata {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub acceptance_order: Option<u64>,
    /// Optional host-assigned UUID for this particular Responses user item.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_item_uuid: Option<uuid::Uuid>,
    #[serde(
        default,
        skip_serializing_if = "codex_history::UserInputOrigin::is_user"
    )]
    pub origin: codex_history::UserInputOrigin,
}

/// Input consumed by a regular turn.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum TurnInput {
    UserInput {
        content: Vec<UserInput>,
        client_id: Option<String>,
        #[serde(flatten)]
        metadata: UserInputMetadata,
    },
    FunctionCallOutput(#[serde(with = "turn_input_response_item")] ResponseItemEnvelope),
    // Preserve the existing serialized format while carrying injection API metadata
    // through the in-memory queue.
    ResponseItem(#[serde(with = "turn_input_response_item")] ResponseItemEnvelope),
    InterAgentCommunication(crate::PendingAgentMessage),
}

mod turn_input_response_item {
    use super::ResponseItem;
    use super::ResponseItemEnvelope;
    use serde::Deserialize;
    use serde::Deserializer;
    use serde::Serialize;
    use serde::Serializer;
    use serde::ser::Error as _;

    pub(super) fn serialize<S>(
        item: &ResponseItemEnvelope,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if item.metadata.is_some() {
            return Err(S::Error::custom(
                "annotated response items cannot cross the turn-input serialization boundary",
            ));
        }
        item.item.serialize(serializer)
    }

    pub(super) fn deserialize<'de, D>(deserializer: D) -> Result<ResponseItemEnvelope, D::Error>
    where
        D: Deserializer<'de>,
    {
        ResponseItem::deserialize(deserializer).map(ResponseItemEnvelope::new)
    }
}

/// Steering takes precedence when several kinds of input are pending together.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum InputQueueActivity {
    Mailbox,
    Steer,
}

impl TurnInput {
    fn wait_activity(&self) -> InputQueueActivity {
        match self {
            Self::InterAgentCommunication(_)
            | Self::ResponseItem(ResponseItemEnvelope {
                item: ResponseItem::AgentMessage { .. },
                ..
            }) => InputQueueActivity::Mailbox,
            Self::UserInput { .. } | Self::FunctionCallOutput(_) | Self::ResponseItem(_) => {
                InputQueueActivity::Steer
            }
        }
    }
}

/// Turn-local pending input storage owned by the input queue flow.
#[derive(Default)]
pub(crate) struct TurnInputQueue {
    items: Vec<TurnInput>,
    /// Activity from notifying writes. Passive response items do not wake a wait.
    notified_activity: Option<InputQueueActivity>,
    /// Input that arrived under other execution authority than the running turn was admitted
    /// with. The turn never takes it: it waits for re-admission, and an interrupt hands it back
    /// to submission admission (`InputQueue::clear_pending`).
    deferred: Vec<(TurnInput, Option<serde_json::Value>)>,
}

/// Session-scoped pending input storage and active-turn mailbox delivery coordination.
pub(crate) struct InputQueue {
    activity_tx: watch::Sender<InputQueueActivity>,
    controller: Option<(ThreadId, Arc<dyn AgentControl>, watch::Receiver<bool>)>,
    mailbox_pending_mails: Mutex<VecDeque<PendingMailboxCommunication>>,
    capacity_wait_scheduled: AtomicBool,
    interrupted_deferred_input: Mutex<VecDeque<(TurnInput, Option<serde_json::Value>)>>,
    interrupted_input_ready: Notify,
}

pub(crate) struct PendingMailboxCommunication {
    pub(crate) communication: crate::PendingAgentMessage,
    start_options: TurnStartOptions,
    /// Whether the mail may join a newly admitted human turn: trigger-turn mail, or mail that
    /// arrived while no task ran. Other mail waits for one completed turn boundary.
    ready_for_admitted_turn: bool,
    /// The running turn that may not take the mail: it arrived under other execution
    /// authority than that turn was admitted with.
    deferred_for_turn: Option<String>,
    deferred_turn_state: Option<Weak<Mutex<TurnState>>>,
    _diagnostics_guard: GaugeGuard,
}

/// The task boundary mail arrived at, which decides when a turn may take it.
#[derive(Default)]
struct MailAdmission {
    arrived_idle: bool,
    deferred_for_turn: Option<String>,
    deferred_turn_state: Option<Weak<Mutex<TurnState>>>,
}

impl PendingMailboxCommunication {
    fn new(
        communication: crate::PendingAgentMessage,
        start_options: TurnStartOptions,
        admission: &MailAdmission,
    ) -> Self {
        Self {
            ready_for_admitted_turn: communication.trigger_turn || admission.arrived_idle,
            deferred_for_turn: admission.deferred_for_turn.clone(),
            deferred_turn_state: admission.deferred_turn_state.clone(),
            communication,
            start_options,
            _diagnostics_guard: PENDING_MAILBOX_MESSAGES.track(),
        }
    }

    fn deferred_for(&self, turn_state: &Mutex<TurnState>) -> bool {
        self.deferred_turn_state
            .as_ref()
            .is_some_and(|deferred| std::ptr::eq(deferred.as_ptr(), turn_state))
    }
}

impl InputQueue {
    pub(crate) fn new() -> Self {
        let (activity_tx, _) = watch::channel(InputQueueActivity::Mailbox);
        Self {
            activity_tx,
            controller: None,
            mailbox_pending_mails: Mutex::new(VecDeque::new()),
            capacity_wait_scheduled: AtomicBool::new(false),
            interrupted_deferred_input: Mutex::new(VecDeque::new()),
            interrupted_input_ready: Notify::new(),
        }
    }

    pub(crate) fn with_controller(thread_id: ThreadId, control: Arc<dyn AgentControl>) -> Self {
        let updates = control.watch_mailbox(thread_id);
        Self {
            controller: Some((thread_id, control, updates)),
            ..Self::new()
        }
    }

    pub(crate) fn try_schedule_capacity_wait(&self) -> bool {
        self.capacity_wait_scheduled
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
    }

    pub(crate) fn finish_capacity_wait(&self) {
        self.capacity_wait_scheduled.store(false, Ordering::Release);
    }

    pub(crate) fn mailbox_updates(&self) -> Option<watch::Receiver<bool>> {
        self.controller
            .as_ref()
            .map(|(_, _, updates)| updates.clone())
    }

    /// Moves mail the agent controller holds for this thread into the session queue; whether
    /// any was read. Mail read without the session's task boundary at hand gets the default
    /// admission: queue-only mail then waits for one completed turn boundary.
    fn read_mailbox(
        &self,
        pending: &mut VecDeque<PendingMailboxCommunication>,
        admission: &MailAdmission,
    ) -> bool {
        let Some((thread_id, control, _)) = &self.controller else {
            return false;
        };
        let mail = control.take_mailbox(*thread_id);
        let read = !mail.is_empty();
        pending.extend(mail.into_iter().map(|communication| {
            PendingMailboxCommunication::new(communication, TurnStartOptions::default(), admission)
        }));
        read
    }

    pub(crate) async fn subscribe_activity(
        &self,
        turn_state: Option<&Mutex<TurnState>>,
    ) -> (
        watch::Receiver<InputQueueActivity>,
        Option<InputQueueActivity>,
    ) {
        let activity_rx = self.activity_tx.subscribe();
        let turn_activity = match turn_state {
            Some(turn_state) => turn_state.lock().await.pending_input.pending_activity(),
            // Turn-local waiters cannot consume input still waiting for re-admission.
            None => (!self.interrupted_deferred_input.lock().await.is_empty())
                .then_some(InputQueueActivity::Steer),
        };
        let pending_activity = if let Some(activity) = turn_activity {
            Some(activity)
        } else if self
            .mailbox_pending_mails
            .lock()
            .await
            .iter()
            .any(|mail| !turn_state.is_some_and(|turn_state| mail.deferred_for(turn_state)))
            || self.has_unread_controller_mail()
        {
            Some(InputQueueActivity::Mailbox)
        } else {
            None
        };
        (activity_rx, pending_activity)
    }

    /// Delivers a notice into the running turn when it can take it now. A turn running under
    /// other execution authority than the session's current one takes no new input.
    #[expect(
        clippy::await_holding_invalid_type,
        reason = "active turn checks and turn state updates must remain atomic"
    )]
    pub(crate) async fn deliver_mailbox_communication_to_current_turn(
        &self,
        session: &Session,
        communication: InterAgentCommunication,
        expected_turn_id: Option<&str>,
    ) -> bool {
        let active = session.active_turn.lock().await;
        let Some((active_turn, task)) = active.as_ref().and_then(|turn| {
            turn.task
                .as_ref()
                .map(|task| (turn, task))
                .filter(|(_, task)| {
                    expected_turn_id.is_none_or(|id| {
                        task.turn_context.sub_id == id && !task.cancellation_token.is_cancelled()
                    })
                })
        }) else {
            return false;
        };
        if !Session::authorization_matches(
            &session.state.lock().await.session_configuration,
            &task.turn_context,
        ) {
            return false;
        }
        let mut turn_state = active_turn.turn_state.lock().await;
        if !turn_state.accepts_mailbox_delivery_for_current_turn() {
            return false;
        }
        let input = TurnInput::InterAgentCommunication(communication.into());
        // #1952 (D485): noted in the same step as the take.
        turn_state.track_taken_mailbox_messages([&input]);
        turn_state.pending_input.items.push(input);
        self.activity_tx.send_replace(InputQueueActivity::Mailbox);
        true
    }

    #[cfg(test)]
    pub(crate) async fn enqueue_mailbox_communication(
        &self,
        communication: crate::PendingAgentMessage,
        start_options: TurnStartOptions,
    ) {
        let admission = MailAdmission::default();
        let mut pending = self.mailbox_pending_mails.lock().await;
        self.read_mailbox(&mut pending, &admission);
        pending.push_back(PendingMailboxCommunication::new(
            communication,
            start_options,
            &admission,
        ));
        self.activity_tx.send_replace(InputQueueActivity::Mailbox);
    }

    /// Enqueues mailbox delivery relative to the session's actual task boundary.
    pub(crate) async fn enqueue_mailbox_communication_for_session(
        &self,
        session: &Session,
        communication: crate::PendingAgentMessage,
        start_options: TurnStartOptions,
    ) {
        self.admit_mailbox_for_session(session, Some((communication, start_options)))
            .await;
    }

    /// Moves mail the agent controller holds into the session queue, admitted at the session's
    /// current task boundary. Call it when the controller signals new mail.
    pub(crate) async fn admit_controller_mailbox(&self, session: &Session) {
        self.admit_mailbox_for_session(session, /*communication*/ None)
            .await;
    }

    /// Holding `active_turn` while the queue entries are created closes the race with task
    /// completion: mail received after the task has been detached is immediately eligible for
    /// the next admitted human turn, while mail received during the task is advanced by that
    /// task's terminal boundary. Mail that arrives under other execution authority than the
    /// running turn was admitted with waits for a turn admitted under the current one.
    #[allow(
        clippy::await_holding_invalid_type,
        reason = "active-turn and mailbox admission must remain atomic across both locks"
    )]
    async fn admit_mailbox_for_session(
        &self,
        session: &Session,
        communication: Option<(crate::PendingAgentMessage, TurnStartOptions)>,
    ) {
        let active_turn = session.active_turn.lock().await;
        let state = session.state.lock().await;
        let task = active_turn.as_ref().and_then(|turn| turn.task.as_ref());
        let deferred_for_turn = task
            .filter(|task| {
                !Session::authorization_matches(&state.session_configuration, &task.turn_context)
            })
            .map(|task| task.turn_context.sub_id.clone());
        let admission = MailAdmission {
            arrived_idle: task.is_none(),
            deferred_turn_state: active_turn
                .as_ref()
                .filter(|_| deferred_for_turn.is_some())
                .map(|turn| Arc::downgrade(&turn.turn_state)),
            deferred_for_turn,
        };
        let mut pending = self.mailbox_pending_mails.lock().await;
        // Mail retained while unloaded precedes new submissions to the loaded session.
        let mut added = self.read_mailbox(&mut pending, &admission);
        if let Some((communication, start_options)) = communication {
            pending.push_back(PendingMailboxCommunication::new(
                communication,
                start_options,
                &admission,
            ));
            added = true;
        }
        drop(pending);
        drop(state);
        drop(active_turn);
        // Deferred mail cannot interrupt a waiter in the turn that cannot consume it.
        if added && admission.deferred_for_turn.is_none() {
            self.activity_tx.send_replace(InputQueueActivity::Mailbox);
        }
    }

    fn has_unread_controller_mail(&self) -> bool {
        self.controller
            .as_ref()
            .is_some_and(|(_, _, updates)| *updates.borrow())
    }

    pub(crate) async fn has_pending_mailbox_items(&self) -> bool {
        !self.mailbox_pending_mails.lock().await.is_empty() || self.has_unread_controller_mail()
    }

    pub(crate) async fn has_trigger_turn_mailbox_items(&self) -> bool {
        self.mailbox_pending_mails
            .lock()
            .await
            .iter()
            .any(|mail| mail.communication.trigger_turn)
    }

    pub(crate) async fn drain_mailbox(&self) -> Vec<PendingMailboxCommunication> {
        let mut pending = self.mailbox_pending_mails.lock().await;
        self.read_mailbox(&mut pending, &MailAdmission::default());
        pending.drain(..).collect()
    }

    #[cfg(test)]
    pub(crate) async fn drain_mailbox_input_items(&self) -> (Vec<TurnInput>, TurnStartOptions) {
        self.drain_mailbox_input_for_turn(/*turn_id*/ None).await
    }

    async fn drain_mailbox_input_for_turn(
        &self,
        turn_id: Option<&str>,
    ) -> (Vec<TurnInput>, TurnStartOptions) {
        // Mail the controller still holds is not read here: a running turn takes it only after
        // `admit_controller_mailbox` checked it against the turn's authority.
        let mut pending_mails = self.mailbox_pending_mails.lock().await;
        let (ready, deferred): (VecDeque<_>, VecDeque<_>) =
            pending_mails.drain(..).partition(|mail| {
                mail.deferred_for_turn.is_none() || mail.deferred_for_turn.as_deref() != turn_id
            });
        *pending_mails = deferred;
        Self::mailbox_input_items(ready.into())
    }

    /// Drains only mail that is eligible to join a newly admitted human turn.
    ///
    /// Trigger-turn mail and mail that arrived while no task ran are immediately eligible.
    /// Other queue-only mail becomes eligible after one active-turn boundary, so mail that
    /// arrived during a turn does not start a mail-only model turn of its own.
    pub(crate) async fn drain_ready_mailbox_input_items(
        &self,
    ) -> (Vec<TurnInput>, TurnStartOptions) {
        let ready_mails = {
            let mut pending_mails = self.mailbox_pending_mails.lock().await;
            self.read_mailbox(&mut pending_mails, &MailAdmission::default());
            let (ready_mails, waiting_mails): (VecDeque<_>, VecDeque<_>) = pending_mails
                .drain(..)
                .partition(|mail| mail.ready_for_admitted_turn);
            *pending_mails = waiting_mails;
            ready_mails
        };
        Self::mailbox_input_items(ready_mails.into())
    }

    /// Advances queue-only mail across a completed turn boundary without starting a turn.
    pub(crate) async fn mark_mailbox_ready_for_next_turn(&self) {
        for mail in self.mailbox_pending_mails.lock().await.iter_mut() {
            mail.ready_for_admitted_turn = true;
        }
    }

    fn mailbox_input_items(
        pending_mails: Vec<PendingMailboxCommunication>,
    ) -> (Vec<TurnInput>, TurnStartOptions) {
        // A later follow-up supersedes the earlier choice, including an omitted choice.
        let mut start_options = pending_mails
            .iter()
            .rev()
            .find(|mail| mail.communication.trigger_turn)
            .map(|mail| mail.start_options.clone())
            .unwrap_or_default();
        start_options.parent_turn_id = pending_mails
            .iter()
            .filter(|mail| mail.communication.trigger_turn)
            .map(|mail| mail.start_options.parent_turn_id.as_deref())
            .reduce(|expected, candidate| expected.filter(|id| candidate == Some(*id)))
            .and_then(|id| id.filter(|id| !id.trim().is_empty()).map(str::to_string));
        start_options.root_turn_id = pending_mails
            .iter()
            .find(|mail| mail.communication.trigger_turn)
            .and_then(|mail| {
                mail.start_options
                    .parent_turn_id
                    .as_deref()
                    .filter(|id| !id.trim().is_empty())
                    .and(mail.start_options.root_turn_id.as_deref())
                    .filter(|id| !id.trim().is_empty())
            })
            .map(str::to_string);
        let items = pending_mails
            .into_iter()
            .map(|mail| TurnInput::InterAgentCommunication(mail.communication))
            .collect();
        (items, start_options)
    }

    pub(crate) async fn turn_state_for_sub_id(
        &self,
        active_turn: &Mutex<Option<ActiveTurn>>,
        sub_id: &str,
    ) -> Option<Arc<Mutex<TurnState>>> {
        let active = active_turn.lock().await;
        active.as_ref().and_then(|active_turn| {
            active_turn
                .task
                .as_ref()
                .is_some_and(|task| task.turn_context.sub_id == sub_id)
                .then(|| Arc::clone(&active_turn.turn_state))
        })
    }

    /// Signal once a user message is queued for this sampling request.
    pub(crate) async fn watch_user_input(
        &self,
        active_turn: &Mutex<Option<ActiveTurn>>,
        sub_id: &str,
        interrupt: CancellationToken,
    ) -> Option<AbortOnDropHandle<()>> {
        let turn_state = self.turn_state_for_sub_id(active_turn, sub_id).await?;
        // Subscribe before inspecting the queue so an arrival cannot be missed.
        let mut activity = self.activity_tx.subscribe();
        Some(AbortOnDropHandle::new(tokio::spawn(async move {
            loop {
                if turn_state.lock().await.pending_input.has_user_input() {
                    interrupt.cancel();
                    return;
                }
                if activity.changed().await.is_err() {
                    return;
                }
            }
        })))
    }

    /// Clear any pending waiters and input buffered for the current turn. Input deferred to
    /// re-admission goes back to submission admission.
    pub(crate) async fn clear_pending(&self, active_turn: &ActiveTurn) {
        let mut turn_state = active_turn.turn_state.lock().await;
        turn_state.clear_pending_waiters();
        // Pending mail can own extension attachments whose destructors need the lock released.
        let mut pending_input = std::mem::take(&mut turn_state.pending_input);
        drop(turn_state);
        let deferred = std::mem::take(&mut pending_input.deferred);
        drop(pending_input);
        if !deferred.is_empty() {
            self.interrupted_deferred_input
                .lock()
                .await
                .extend(deferred);
            self.interrupted_input_ready.notify_one();
        }
    }

    /// Interrupted input returns to submission admission, never directly to a task's queue.
    pub(crate) async fn next_interrupted_deferred_input(
        &self,
    ) -> (TurnInput, Option<serde_json::Value>) {
        loop {
            let ready = self.interrupted_input_ready.notified();
            if let Some(input) = self.interrupted_deferred_input.lock().await.pop_front() {
                return input;
            }
            ready.await;
        }
    }

    pub(crate) async fn defer_mailbox_delivery_to_next_turn(
        &self,
        active_turn: &Mutex<Option<ActiveTurn>>,
        sub_id: &str,
    ) {
        let turn_state = self.turn_state_for_sub_id(active_turn, sub_id).await;
        let Some(turn_state) = turn_state else {
            return;
        };
        let mut turn_state = turn_state.lock().await;
        // Explicit same-turn work still needs a follow-up. Queue-only child mail does not: keep
        // it pending so task completion records it for the next turn without sampling again.
        if turn_state.pending_input.items.iter().any(|input| {
            !matches!(
                input,
                TurnInput::InterAgentCommunication(communication) if !communication.trigger_turn
            )
        }) {
            return;
        }
        turn_state.set_mailbox_delivery_phase(MailboxDeliveryPhase::NextTurn);
    }

    pub(crate) async fn accept_mailbox_delivery_for_current_turn(
        &self,
        active_turn: &Mutex<Option<ActiveTurn>>,
        sub_id: &str,
    ) {
        let turn_state = self.turn_state_for_sub_id(active_turn, sub_id).await;
        let Some(turn_state) = turn_state else {
            return;
        };
        self.accept_mailbox_delivery_for_turn_state(turn_state.as_ref())
            .await;
    }

    pub(super) async fn accept_mailbox_delivery_for_turn_state(
        &self,
        turn_state: &Mutex<TurnState>,
    ) {
        turn_state
            .lock()
            .await
            .accept_mailbox_delivery_for_current_turn();
    }

    pub(super) async fn extend_pending_input_and_accept_mailbox_delivery_for_turn_state(
        &self,
        turn_state: &Mutex<TurnState>,
        input: Vec<TurnInput>,
    ) {
        let activity = input.iter().map(TurnInput::wait_activity).max();
        {
            let mut turn_state = turn_state.lock().await;
            turn_state.pending_input.notified_activity =
                turn_state.pending_input.notified_activity.max(activity);
            turn_state.pending_input.items.extend(input);
            turn_state.accept_mailbox_delivery_for_current_turn();
        }
        self.activity_tx
            .send_replace(activity.unwrap_or(InputQueueActivity::Steer));
    }

    pub(crate) async fn extend_pending_input_for_turn_state(
        &self,
        turn_state: &Mutex<TurnState>,
        input: Vec<TurnInput>,
    ) {
        turn_state.lock().await.pending_input.items.extend(input);
    }
    pub(crate) async fn defer_input_for_turn_state(
        &self,
        turn_state: &Mutex<TurnState>,
        input: Vec<TurnInput>,
        final_output_json_schema: Option<serde_json::Value>,
    ) {
        turn_state.lock().await.pending_input.deferred.extend(
            input
                .into_iter()
                .map(|input| (input, final_output_json_schema.clone())),
        );
    }

    /// The turn's leftover input, deferred input last, each with the output schema it was
    /// submitted with.
    pub(crate) async fn take_pending_input_for_turn_state(
        &self,
        turn_state: &Mutex<TurnState>,
    ) -> Vec<(TurnInput, Option<serde_json::Value>)> {
        let pending = std::mem::take(&mut turn_state.lock().await.pending_input);
        let mut input: Vec<_> = pending
            .items
            .into_iter()
            .map(|input| (input, None))
            .collect();
        input.extend(pending.deferred);
        input
    }

    #[cfg(test)]
    pub(crate) async fn get_pending_input(
        &self,
        active_turn: &Mutex<Option<ActiveTurn>>,
    ) -> (Vec<TurnInput>, TurnStartOptions) {
        self.get_pending_input_inner(active_turn, /*allow_untracked_mailbox*/ true)
            .await
    }

    /// Takes input for an already-running task. Mail the agent controller holds is admitted at
    /// the session's task boundary first. A task detached by any stop path must not drain
    /// mailbox mail: it no longer has a turn state that can record its outcome (#2010).
    pub(crate) async fn get_pending_input_for_running_task(
        &self,
        session: &Session,
    ) -> (Vec<TurnInput>, TurnStartOptions) {
        self.admit_controller_mailbox(session).await;
        self.get_pending_input_inner(&session.active_turn, /*allow_untracked_mailbox*/ false)
            .await
    }

    #[allow(
        clippy::await_holding_invalid_type,
        reason = "taking mail off the queue and noting it in the turn state must remain atomic"
    )]
    async fn get_pending_input_inner(
        &self,
        active_turn: &Mutex<Option<ActiveTurn>>,
        allow_untracked_mailbox: bool,
    ) -> (Vec<TurnInput>, TurnStartOptions) {
        let (pending_input, accepts_mailbox_delivery, turn_id, turn_state) =
            Self::take_turn_pending_input(active_turn).await;
        if !accepts_mailbox_delivery {
            return (pending_input, TurnStartOptions::default());
        }
        let (mailbox_items, start_options) = match turn_state {
            // #1952 (D485): the turn notes the mail it takes in the same step, with no await
            // between, so an interrupt that stops the turn knows what it took.
            Some(turn_state) => {
                let mut turn_state = turn_state.lock().await;
                let (mailbox_items, start_options) =
                    self.drain_mailbox_input_for_turn(turn_id.as_deref()).await;
                turn_state.track_taken_mailbox_messages(&mailbox_items);
                (mailbox_items, start_options)
            }
            // Test-only: older tests poll an empty slot to inspect queued mail. Executing tasks
            // use the fail-closed path above.
            None if allow_untracked_mailbox => {
                self.drain_mailbox_input_for_turn(turn_id.as_deref()).await
            }
            None => (Vec::new(), TurnStartOptions::default()),
        };
        if pending_input.is_empty() {
            (mailbox_items, start_options)
        } else {
            let mut pending_input = pending_input;
            pending_input.extend(mailbox_items);
            (pending_input, start_options)
        }
    }

    /// Takes input while admitting a task without bypassing mailbox turn-boundary eligibility.
    ///
    /// A task start may inherit user steering from a reserved active-turn slot, but queue-only
    /// collaboration mail must remain session-pending until it has crossed a completed turn
    /// boundary. Draining the entire mailbox here converts that mail into a same-turn follow-up
    /// and can create a mail-only model turn before the next human prompt.
    pub(crate) async fn get_pending_input_for_task_start(
        &self,
        active_turn: &Mutex<Option<ActiveTurn>>,
    ) -> (Vec<TurnInput>, TurnStartOptions) {
        let (pending_input, accepts_mailbox_delivery, turn_id, _turn_state) =
            Self::take_turn_pending_input(active_turn).await;
        if !accepts_mailbox_delivery {
            return (pending_input, TurnStartOptions::default());
        }
        debug_assert!(
            turn_id.is_none(),
            "task start must precede task installation"
        );
        let (mailbox_items, start_options) = self.drain_ready_mailbox_input_items().await;
        if pending_input.is_empty() {
            (mailbox_items, start_options)
        } else {
            let mut pending_input = pending_input;
            pending_input.extend(mailbox_items);
            (pending_input, start_options)
        }
    }

    #[allow(
        clippy::await_holding_invalid_type,
        reason = "active-turn and turn-state inspection must remain atomic"
    )]
    async fn take_turn_pending_input(
        active_turn: &Mutex<Option<ActiveTurn>>,
    ) -> (
        Vec<TurnInput>,
        bool,
        Option<String>,
        Option<Arc<Mutex<TurnState>>>,
    ) {
        let mut active = active_turn.lock().await;
        match active.as_mut() {
            Some(active_turn) => {
                let mut turn_state = active_turn.turn_state.lock().await;
                let accepts_mailbox_delivery =
                    turn_state.accepts_mailbox_delivery_for_current_turn();
                let pending_input = if accepts_mailbox_delivery {
                    turn_state.pending_input.take_items()
                } else {
                    Vec::new()
                };
                (
                    pending_input,
                    accepts_mailbox_delivery,
                    active_turn
                        .task
                        .as_ref()
                        .map(|task| task.turn_context.sub_id.clone()),
                    Some(Arc::clone(&active_turn.turn_state)),
                )
            }
            // `get_pending_input_for_task_start` also uses this helper. It may legitimately
            // start from an idle slot and must still inspect ready mailbox work; the running
            // task path separately refuses to drain without a TurnState (#2010).
            None => (Vec::new(), true, None, None),
        }
    }

    #[expect(
        clippy::await_holding_invalid_type,
        reason = "active turn checks and turn state reads must remain atomic"
    )]
    pub(crate) async fn has_pending_input(&self, active_turn: &Mutex<Option<ActiveTurn>>) -> bool {
        if !self.interrupted_deferred_input.lock().await.is_empty() {
            return true;
        }
        let (has_turn_pending_input, accepts_mailbox_delivery, turn_id) = {
            let active = active_turn.lock().await;
            match active.as_ref() {
                Some(active_turn) => {
                    let turn_state = active_turn.turn_state.lock().await;
                    (
                        !turn_state.pending_input.is_empty(),
                        turn_state.accepts_mailbox_delivery_for_current_turn(),
                        active_turn
                            .task
                            .as_ref()
                            .map(|task| task.turn_context.sub_id.clone()),
                    )
                }
                None => (false, true, None),
            }
        };
        if !accepts_mailbox_delivery {
            return false;
        }
        if has_turn_pending_input {
            return true;
        }
        self.has_unread_controller_mail()
            || self
                .mailbox_pending_mails
                .lock()
                .await
                .iter()
                .any(|mail| mail.deferred_for_turn.is_none() || mail.deferred_for_turn != turn_id)
    }
}

impl TurnInputQueue {
    fn has_user_input(&self) -> bool {
        self.items
            .iter()
            .any(|input| matches!(input, TurnInput::UserInput { .. }))
    }

    /// Whether the turn has input it may take; input deferred to re-admission is not counted.
    pub(crate) fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// D532 (a): moves the input the turn may take to input deferred to re-admission.
    pub(crate) fn defer_items(&mut self, final_output_json_schema: Option<serde_json::Value>) {
        let items = self.take_items();
        self.deferred.extend(
            items
                .into_iter()
                .map(|input| (input, final_output_json_schema.clone())),
        );
    }

    /// Takes the input the turn may take, leaving input deferred to re-admission.
    fn take_items(&mut self) -> Vec<TurnInput> {
        self.notified_activity = None;
        std::mem::take(&mut self.items)
    }

    /// #1952: whether `message_id` waits in input deferred to re-admission, which an interrupt
    /// keeps (`InputQueue::clear_pending`).
    pub(crate) fn defers_mailbox_message(&self, message_id: &str) -> bool {
        self.deferred.iter().any(|(input, _)| {
            matches!(
                input,
                TurnInput::InterAgentCommunication(communication)
                    if communication.message_id.as_deref() == Some(message_id)
            )
        })
    }

    fn pending_activity(&self) -> Option<InputQueueActivity> {
        if self.notified_activity == Some(InputQueueActivity::Steer) {
            return self.notified_activity;
        }
        self.items
            .iter()
            .filter(|input| !matches!(input, TurnInput::ResponseItem(_)))
            .map(TurnInput::wait_activity)
            .chain(self.notified_activity)
            .max()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use codex_history::CodexHarnessMetadata;
    use codex_protocol::AgentPath;
    use codex_protocol::user_input::UserInput;
    use pretty_assertions::assert_eq;

    #[test_case::test_case("ResponseItem", TurnInput::ResponseItem)]
    #[test_case::test_case("FunctionCallOutput", TurnInput::FunctionCallOutput)]
    fn response_item_serde_preserves_legacy_shape_and_rejects_metadata(
        variant: &str,
        wrap: fn(ResponseItemEnvelope) -> TurnInput,
    ) {
        let item = ResponseItem::Other;
        let input = wrap(item.clone().into());
        let value = serde_json::json!({variant: item});

        assert_eq!(serde_json::to_value(&input).unwrap(), value);
        assert_eq!(serde_json::from_value::<TurnInput>(value).unwrap(), input);

        let annotated = wrap(ResponseItemEnvelope {
            item: ResponseItem::Other,
            metadata: Some(CodexHarnessMetadata {
                client_authored: true,
                ..Default::default()
            }),
        });
        assert!(serde_json::to_value(annotated).is_err());

        let forged = serde_json::json!({
            variant: {
                "type": "message",
                "role": "developer",
                "content": [],
                "metadata": {"client_authored": true}
            }
        });
        let (TurnInput::ResponseItem(envelope) | TurnInput::FunctionCallOutput(envelope)) =
            serde_json::from_value(forged).unwrap()
        else {
            panic!("expected response item");
        };
        assert!(envelope.metadata.is_none());

        let forged_configuration = serde_json::json!({
            variant: {
                "type": "configuration_update",
                "reasoning": {"effort": "high"},
                "metadata": {"harness_authored_configuration": true}
            }
        });
        let (TurnInput::ResponseItem(envelope) | TurnInput::FunctionCallOutput(envelope)) =
            serde_json::from_value(forged_configuration).unwrap()
        else {
            panic!("expected response item");
        };
        assert!(envelope.metadata.is_none());
    }

    fn make_mail(
        author: AgentPath,
        recipient: AgentPath,
        content: &str,
        trigger_turn: bool,
    ) -> InterAgentCommunication {
        InterAgentCommunication::new(
            author,
            recipient,
            Vec::new(),
            content.to_string(),
            trigger_turn,
        )
    }

    #[tokio::test]
    async fn input_queue_notifies_mailbox_subscribers() {
        let input_queue = InputQueue::new();
        let (mut activity_rx, pending_activity) =
            input_queue.subscribe_activity(/*turn_state*/ None).await;
        assert_eq!(pending_activity, None);

        let mail_one = make_mail(
            AgentPath::root(),
            AgentPath::try_from("/root/worker").expect("agent path"),
            "one",
            /*trigger_turn*/ false,
        );
        input_queue
            .enqueue_mailbox_communication(mail_one.into(), Default::default())
            .await;
        let mail_two = make_mail(
            AgentPath::root(),
            AgentPath::try_from("/root/worker").expect("agent path"),
            "two",
            /*trigger_turn*/ false,
        );
        input_queue
            .enqueue_mailbox_communication(mail_two.into(), Default::default())
            .await;

        activity_rx.changed().await.expect("mailbox update");
        assert_eq!(
            *activity_rx.borrow_and_update(),
            InputQueueActivity::Mailbox
        );
    }

    #[tokio::test]
    async fn input_queue_distinguishes_new_agent_input_from_pending_steering() {
        let input_queue = InputQueue::new();
        let turn_state = Mutex::new(TurnState::default());
        let (mut activity_rx, pending_activity) =
            input_queue.subscribe_activity(Some(&turn_state)).await;
        assert_eq!(pending_activity, None);

        input_queue
            .extend_pending_input_and_accept_mailbox_delivery_for_turn_state(
                &turn_state,
                vec![TurnInput::UserInput {
                    metadata: Default::default(),
                    content: vec![UserInput::Text {
                        text: "steer".to_string(),
                        text_elements: Vec::new(),
                    }],
                    client_id: None,
                }],
            )
            .await;

        activity_rx.changed().await.expect("steer update");
        assert_eq!(*activity_rx.borrow_and_update(), InputQueueActivity::Steer);

        let mail = make_mail(
            AgentPath::root(),
            AgentPath::try_from("/root/worker").expect("agent path"),
            "follow-up",
            /*trigger_turn*/ true,
        );
        input_queue
            .extend_pending_input_and_accept_mailbox_delivery_for_turn_state(
                &turn_state,
                vec![TurnInput::InterAgentCommunication(mail.into())],
            )
            .await;
        activity_rx.changed().await.expect("mailbox update");
        assert_eq!(
            *activity_rx.borrow_and_update(),
            InputQueueActivity::Mailbox
        );
        assert_eq!(
            input_queue.subscribe_activity(Some(&turn_state)).await.1,
            Some(InputQueueActivity::Steer)
        );
    }

    #[tokio::test]
    async fn input_queue_reports_already_pending_steer() {
        let input_queue = InputQueue::new();
        let turn_state = Mutex::new(TurnState::default());
        let passive_output: TurnInput = serde_json::from_value(serde_json::json!({
            "ResponseItem": {"type": "function_call_output", "name": "notify", "output": "passive"}
        }))
        .unwrap();
        input_queue
            .extend_pending_input_for_turn_state(&turn_state, vec![passive_output.clone()])
            .await;
        assert_eq!(
            input_queue.subscribe_activity(Some(&turn_state)).await.1,
            None
        );
        input_queue
            .extend_pending_input_and_accept_mailbox_delivery_for_turn_state(
                &turn_state,
                vec![passive_output.clone()],
            )
            .await;
        assert_eq!(
            input_queue.subscribe_activity(Some(&turn_state)).await.1,
            Some(InputQueueActivity::Steer)
        );
        // Consuming the injection must not make later passive context wake a wait.
        input_queue
            .take_pending_input_for_turn_state(&turn_state)
            .await;
        input_queue
            .extend_pending_input_for_turn_state(&turn_state, vec![passive_output])
            .await;
        assert_eq!(
            input_queue.subscribe_activity(Some(&turn_state)).await.1,
            None
        );
        let communication = make_mail(
            AgentPath::root(),
            AgentPath::try_from("/root/worker").expect("agent path"),
            "already pending mail",
            /*trigger_turn*/ false,
        );
        input_queue
            .extend_pending_input_for_turn_state(
                &turn_state,
                vec![TurnInput::InterAgentCommunication(communication.into())],
            )
            .await;
        assert_eq!(
            input_queue.subscribe_activity(Some(&turn_state)).await.1,
            Some(InputQueueActivity::Mailbox)
        );
        input_queue
            .extend_pending_input_and_accept_mailbox_delivery_for_turn_state(
                &turn_state,
                vec![TurnInput::UserInput {
                    metadata: Default::default(),
                    content: vec![UserInput::Text {
                        text: "already pending".to_string(),
                        text_elements: Vec::new(),
                    }],
                    client_id: None,
                }],
            )
            .await;

        let (_activity_rx, pending_activity) =
            input_queue.subscribe_activity(Some(&turn_state)).await;

        assert_eq!(pending_activity, Some(InputQueueActivity::Steer));
    }

    #[tokio::test]
    async fn input_queue_drains_mailbox_in_delivery_order() {
        let input_queue = InputQueue::new();
        let mail_one = make_mail(
            AgentPath::root(),
            AgentPath::try_from("/root/worker").expect("agent path"),
            "one",
            /*trigger_turn*/ false,
        );
        let mail_two = make_mail(
            AgentPath::try_from("/root/worker").expect("agent path"),
            AgentPath::root(),
            "two",
            /*trigger_turn*/ true,
        );

        input_queue
            .enqueue_mailbox_communication(mail_one.clone().into(), Default::default())
            .await;
        input_queue
            .enqueue_mailbox_communication(mail_two.clone().into(), Default::default())
            .await;

        assert_eq!(
            input_queue.drain_mailbox_input_items().await.0,
            vec![
                TurnInput::InterAgentCommunication(mail_one.into()),
                TurnInput::InterAgentCommunication(mail_two.into())
            ]
        );
        assert!(!input_queue.has_pending_mailbox_items().await);
    }

    #[tokio::test]
    async fn input_queue_uses_unambiguous_trigger_parent_and_first_root() {
        let (parent, peer, root, root2) = (Some("a"), Some("b"), Some("r"), Some("s"));
        for (pending_mails, expected_parent_turn_id, expected_root_turn_id) in [
            (Vec::new(), None, None),
            (vec![(false, Some("q"), root)], None, None),
            (vec![(true, Some(""), root)], None, None),
            (vec![(true, Some("   "), root)], None, None),
            (vec![(true, None, root)], None, None),
            (vec![(true, parent, None)], parent, None),
            (vec![(true, parent, Some(""))], parent, None),
            (vec![(true, parent, root), (true, peer, root)], None, root),
            (vec![(true, parent, root), (true, peer, root2)], None, root),
            (vec![(true, parent, root), (true, None, root)], None, root),
            (
                vec![(true, parent, root), (true, parent, root)],
                parent,
                root,
            ),
            (
                vec![(false, Some("q"), root2), (true, parent, root)],
                parent,
                root,
            ),
        ] {
            let input_queue = InputQueue::new();
            for (trigger_turn, parent_turn_id, root_turn_id) in pending_mails {
                input_queue
                    .enqueue_mailbox_communication(
                        make_mail(AgentPath::root(), AgentPath::root(), "task", trigger_turn)
                            .into(),
                        TurnStartOptions {
                            parent_turn_id: parent_turn_id.map(str::to_string),
                            root_turn_id: root_turn_id.map(str::to_string),
                            ..Default::default()
                        },
                    )
                    .await;
            }
            let (_, start_options) = input_queue.drain_mailbox_input_items().await;
            assert_eq!(
                start_options.parent_turn_id.as_deref(),
                expected_parent_turn_id
            );
            assert_eq!(start_options.root_turn_id.as_deref(), expected_root_turn_id);
        }
    }

    #[tokio::test]
    async fn input_queue_uses_latest_followup_choice_and_ignores_queue_only_mail() {
        use codex_protocol::turn_input::CyberAccessProgram;

        for latest in [Some(CyberAccessProgram::Standard), None] {
            let input_queue = InputQueue::new();
            for (trigger_turn, program) in [
                (true, Some(CyberAccessProgram::DaybreakBlue)),
                (true, latest),
                (false, Some(CyberAccessProgram::DaybreakRed)),
            ] {
                input_queue
                    .enqueue_mailbox_communication(
                        make_mail(AgentPath::root(), AgentPath::root(), "task", trigger_turn)
                            .into(),
                        TurnStartOptions {
                            cyber_access_program: program,
                            ..Default::default()
                        },
                    )
                    .await;
            }
            let (_, start_options) = input_queue.drain_mailbox_input_items().await;
            assert_eq!(start_options.cyber_access_program, latest);
        }
    }

    #[tokio::test]
    async fn input_queue_tracks_pending_trigger_turn_mail() {
        let input_queue = InputQueue::new();

        let queued_mail = make_mail(
            AgentPath::root(),
            AgentPath::try_from("/root/worker").expect("agent path"),
            "queued",
            /*trigger_turn*/ false,
        );
        input_queue
            .enqueue_mailbox_communication(queued_mail.into(), Default::default())
            .await;
        assert!(!input_queue.has_trigger_turn_mailbox_items().await);

        let trigger_mail = make_mail(
            AgentPath::root(),
            AgentPath::try_from("/root/worker").expect("agent path"),
            "wake",
            /*trigger_turn*/ true,
        );
        input_queue
            .enqueue_mailbox_communication(trigger_mail.into(), Default::default())
            .await;
        assert!(input_queue.has_trigger_turn_mailbox_items().await);
    }

    #[tokio::test]
    async fn admitted_turn_drain_waits_one_boundary_for_queue_only_mail() {
        let input_queue = InputQueue::new();
        let queued_mail = make_mail(
            AgentPath::try_from("/root/worker").expect("agent path"),
            AgentPath::root(),
            "queued",
            /*trigger_turn*/ false,
        );
        input_queue
            .enqueue_mailbox_communication(queued_mail.clone().into(), Default::default())
            .await;

        assert_eq!(
            input_queue.drain_ready_mailbox_input_items().await.0,
            Vec::new()
        );
        input_queue.mark_mailbox_ready_for_next_turn().await;
        assert_eq!(
            input_queue.drain_ready_mailbox_input_items().await.0,
            vec![TurnInput::InterAgentCommunication(queued_mail.into())]
        );
    }

    #[tokio::test]
    async fn idle_session_makes_queue_only_mail_ready_for_next_turn() {
        let (session, _context) = super::super::tests::make_session_and_context().await;
        let input_queue = &session.input_queue;
        let queued_mail = make_mail(
            AgentPath::try_from("/root/worker").expect("agent path"),
            AgentPath::root(),
            "queued while idle",
            /*trigger_turn*/ false,
        );

        input_queue
            .enqueue_mailbox_communication_for_session(
                &session,
                queued_mail.clone().into(),
                TurnStartOptions::default(),
            )
            .await;

        assert_eq!(
            input_queue.drain_ready_mailbox_input_items().await.0,
            vec![TurnInput::InterAgentCommunication(queued_mail.into())]
        );
    }

    #[tokio::test]
    async fn admitted_turn_drain_accepts_trigger_mail_immediately() {
        let input_queue = InputQueue::new();
        let trigger_mail = make_mail(
            AgentPath::try_from("/root/worker").expect("agent path"),
            AgentPath::root(),
            "trigger",
            /*trigger_turn*/ true,
        );
        input_queue
            .enqueue_mailbox_communication(
                trigger_mail.clone().into(),
                TurnStartOptions {
                    parent_turn_id: Some("parent-turn".to_string()),
                    ..Default::default()
                },
            )
            .await;

        let (items, start_options) = input_queue.drain_ready_mailbox_input_items().await;
        assert_eq!(
            (items, start_options.parent_turn_id),
            (
                vec![TurnInput::InterAgentCommunication(trigger_mail.into())],
                Some("parent-turn".to_string())
            )
        );
    }

    fn mail_with_id(content: &str) -> (InterAgentCommunication, String) {
        let mut mail = make_mail(
            AgentPath::root(),
            AgentPath::try_from("/root/worker").expect("agent path"),
            content,
            /*trigger_turn*/ true,
        );
        let message_id = mail.ensure_message_identity().to_string();
        (mail, message_id)
    }

    #[tokio::test]
    async fn running_turn_notes_the_mail_it_takes() {
        let input_queue = InputQueue::new();
        let active_turn = Mutex::new(Some(ActiveTurn::default()));
        let (mail, message_id) = mail_with_id("taken mid-turn");
        input_queue
            .enqueue_mailbox_communication(mail.clone().into(), Default::default())
            .await;

        let (input, _) = input_queue.get_pending_input(&active_turn).await;

        assert_eq!(input, vec![TurnInput::InterAgentCommunication(mail.into())]);
        let turn_state = Arc::clone(
            &active_turn
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
    }

    #[tokio::test]
    async fn stopped_turn_keeps_mailbox_items_for_the_next_turn() {
        let input_queue = InputQueue::new();
        let stopped_turn = Mutex::new(None);
        let (mail, _) = mail_with_id("arrived while stopping");
        input_queue
            .enqueue_mailbox_communication(mail.clone().into(), Default::default())
            .await;

        // The active slot is removed before shutdown waits for the running task. A stale task
        // reaching its next input poll must leave mail queued rather than silently taking it.
        assert_eq!(
            input_queue
                .get_pending_input_inner(&stopped_turn, /*allow_untracked_mailbox*/ false)
                .await
                .0,
            Vec::new()
        );
        assert!(input_queue.has_pending_mailbox_items().await);

        // A subsequently admitted turn receives the preserved item normally.
        let next_turn = Mutex::new(Some(ActiveTurn::default()));
        assert_eq!(
            input_queue.get_pending_input(&next_turn).await.0,
            vec![TurnInput::InterAgentCommunication(mail.into())]
        );
        assert!(!input_queue.has_pending_mailbox_items().await);
    }

    #[tokio::test]
    async fn task_start_still_admits_ready_mail_from_an_idle_slot() {
        let input_queue = InputQueue::new();
        let idle_slot = Mutex::new(None);
        let (mail, _) = mail_with_id("normal task start");
        input_queue
            .enqueue_mailbox_communication(mail.clone().into(), Default::default())
            .await;

        // Control: the task-admission path may begin with no installed active turn and must
        // retain its existing ability to take ready mail.
        assert_eq!(
            input_queue
                .get_pending_input_for_task_start(&idle_slot)
                .await
                .0,
            vec![TurnInput::InterAgentCommunication(mail.into())]
        );
    }

    #[tokio::test]
    async fn unrecorded_mail_leaves_out_recorded_and_deferred_mail() {
        let input_queue = InputQueue::new();
        let turn_state = Mutex::new(TurnState::default());
        let (recorded, recorded_id) = mail_with_id("recorded");
        let (deferred, _) = mail_with_id("deferred");
        let (lost, lost_id) = mail_with_id("lost");
        let taken = [recorded, deferred.clone(), lost]
            .into_iter()
            .map(|mail| TurnInput::InterAgentCommunication(mail.into()))
            .collect::<Vec<_>>();
        turn_state.lock().await.track_taken_mailbox_messages(&taken);
        input_queue
            .defer_input_for_turn_state(
                &turn_state,
                vec![TurnInput::InterAgentCommunication(deferred.into())],
                /*final_output_json_schema*/ None,
            )
            .await;

        let mut turn_state = turn_state.lock().await;
        assert_eq!(
            turn_state.take_unrecorded_mailbox_message_ids(&[recorded_id]),
            vec![lost_id]
        );
        assert_eq!(
            turn_state.take_unrecorded_mailbox_message_ids(&[]),
            Vec::<String>::new()
        );
    }
}
