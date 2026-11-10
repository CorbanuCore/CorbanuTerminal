use crate::TurnInputRequest;
use crate::TurnInputSubmission;
use crate::TurnStartOptions;
use crate::agent::AgentStatus;
use crate::agent::role::agent_nickname_candidates;
use crate::agent::status::is_final;
use crate::agent::types::AgentMetadata;
use crate::agent::types::LiveAgent;
use crate::agent_communication::AgentCommunicationContext;
use crate::agent_communication::AgentCommunicationKind;
use crate::agent_communication::MessageDelivery;
use crate::codex_thread::ThreadConfigSnapshot;
use crate::config::Config;
use crate::config::RolloutBudgetConfig;
use crate::context::ContextualUserFragment;
use crate::context::SubagentNotification;
use crate::environment_selection::TurnEnvironmentSnapshot;
use crate::security::EffectivePolicyInitialization;
use crate::security::EffectivePolicyView;
use crate::security::PersistedHumanSecurityState;
use crate::security::SecurityPolicyError;
use crate::security::TrustedSecurityController;
use crate::session_prefix::format_inter_agent_completion_message;
use crate::thread_manager::ResumeThreadWithHistoryOptions;
use crate::thread_manager::ThreadIdGenerator;
use crate::thread_manager::ThreadManagerState;
use crate::thread_manager::default_thread_id_generator;
use crate::turn_timing::now_unix_timestamp_ms;
use codex_history::InitialHistory;
use codex_history::ResumedHistory;
use codex_protocol::AgentPath;
use codex_protocol::ResponseItemId;
use codex_protocol::SessionId;
use codex_protocol::ThreadId;
use codex_protocol::crew::CREW_AUTO_DISPATCH_CHAIN_LIMIT;
use codex_protocol::error::CodexErr;
use codex_protocol::error::CodexErrorDetails;
use codex_protocol::error::Result as CodexResult;
use codex_protocol::items::SubAgentActivityItem;
use codex_protocol::items::TurnItem;
use codex_protocol::models::ContentItemMetadata;
use codex_protocol::models::ContentItemNamespace;
use codex_protocol::protocol::AgentMessageKind;
use codex_protocol::protocol::Event;
use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::HasLegacyEvent;
use codex_protocol::protocol::InterAgentCommunication;
use codex_protocol::protocol::ItemCompletedEvent;
use codex_protocol::protocol::ItemStartedEvent;
use codex_protocol::protocol::MultiAgentVersion;
use codex_protocol::protocol::Op;
use codex_protocol::protocol::SessionSource;
use codex_protocol::protocol::SubAgentSource;
use codex_protocol::protocol::ThreadHistoryMode;
use codex_protocol::protocol::ThreadSource;
use codex_protocol::user_input::UserInput;
use codex_security_policy::PolicyPrincipal;
use codex_security_policy::PrincipalKind;
#[cfg(test)]
use codex_security_policy::RevocationState;
#[cfg(test)]
use codex_security_policy::SecurityLevel;
use codex_security_policy::SecuritySettings;
use codex_thread_store::ReadThreadParams;
use futures::StreamExt;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::Weak;
use tracing::warn;
use uuid::Uuid;

pub(crate) use self::runtime::AgentControlInit;
pub(crate) use self::runtime::AgentTreeMembership;
pub(crate) use self::runtime::AgentTreeShutdownState;
pub(crate) use self::runtime::AgentTreeTeardownGuard;
pub(crate) use self::runtime::LocalAgentRuntime;
pub(crate) use self::watch::StatusSubscription;
pub(in crate::agent) use spawn::effective_spawn_config;
pub(in crate::agent) use spawn::fresh_child_history_and_tools;
pub(in crate::agent) use spawn::inherited_dynamic_tools;
pub(in crate::agent) use spawn_guard::PendingSpawn;
pub(in crate::agent) use spawn_guard::SpawnOwner;

const LAST_TASK_MESSAGE_MAX_CHARS: usize = 240;
const LAST_RESULT_MESSAGE_MAX_CHARS: usize = 500;
const ROOT_LAST_TASK_MESSAGE: &str = "Main thread";

mod api;
mod budget;
mod completion;
mod delivery;
mod durable_mailbox;
mod execution;
mod held_mailbox_warning;
mod inspection;
mod interrupt;
mod legacy;
mod mailbox;
mod residency;
pub use residency::ThreadEvictionOutcome;
mod resume;
mod root_handoff;
mod runtime;
mod runtime_context;
mod sender_context;
mod service_tier;
mod spawn;
mod spawn_guard;
mod spawn_telemetry;
mod target;
mod user_authorization;
mod watch;

/// Corbanu: the row `list_agents` and `wait_agent` report for one agent, with the
/// nickname, role and last task/result that `/spawn`'s agent graph shows.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct ListedAgent {
    pub(crate) agent_name: String,
    pub(crate) agent_nickname: Option<String>,
    pub(crate) agent_role: Option<String>,
    pub(crate) agent_status: AgentStatus,
    pub(crate) last_task_message: Option<String>,
    pub(crate) last_result_message: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct NativeAutoLoopState {
    operator_input_seen: bool,
    chain: u32,
    auto_turn_running: bool,
    auto_turn_dispatched: bool,
}

/// Turn-scoped guard that finalizes native loop-breaker state on every exit path, including
/// interruption and provider/tool failure.
pub(crate) struct NativeAutoTurnGuard {
    control: LocalAgentControl,
    agent_id: ThreadId,
}

impl Drop for NativeAutoTurnGuard {
    fn drop(&mut self) {
        self.control.complete_native_agent_turn(self.agent_id);
    }
}

/// Per-session controller handle for a local agent tree.
/// Handles retain a session identity and share their tree's `LocalAgentRuntime`.
/// Local startup preserves that state when creating or resuming children.
#[derive(Clone)]
pub(crate) struct LocalAgentControl {
    /// session_id is equal to the root thread's ID.
    session_id: SessionId,
    pub(crate) runtime: LocalAgentRuntime,
}

impl Default for LocalAgentControl {
    fn default() -> Self {
        Self::new(
            Weak::default(),
            default_thread_id_generator(),
            /*rollout_budget*/ None,
        )
    }
}

impl LocalAgentControl {
    /// Construct a new `LocalAgentControl` that can spawn/message agents via the given manager state.
    pub(crate) fn new(
        manager: Weak<ThreadManagerState>,
        thread_id_generator: ThreadIdGenerator,
        rollout_budget: Option<RolloutBudgetConfig>,
    ) -> Self {
        Self {
            session_id: SessionId::default(),
            runtime: LocalAgentRuntime::new(manager, thread_id_generator, rollout_budget),
        }
    }

    pub(crate) fn with_session_id(mut self, session_id: SessionId, max_threads: usize) -> Self {
        self.session_id = session_id;
        self.runtime.agent_execution_limiter.initialize(max_threads);
        self
    }

    pub(crate) fn session_id(&self) -> SessionId {
        self.session_id
    }

    #[cfg(test)]
    pub(crate) fn with_effective_security_policy(
        self,
        level: SecurityLevel,
        root_thread_id: ThreadId,
        inherits_from_spawn_parent: bool,
    ) -> Result<Self, SecurityPolicyError> {
        self.with_recovered_security_policy(
            crate::security::recovery::Recovery {
                level,
                revocations: RevocationState::new(),
                unreadable: None,
                home: None,
            },
            root_thread_id,
            inherits_from_spawn_parent,
        )
    }

    /// PF-23-S03: start from the recovered durable state (the stricter level
    /// and the stored revocations; Aggressive and the kill switch when it was
    /// unreadable).
    pub(crate) fn with_recovered_security_policy(
        self,
        recovery: crate::security::recovery::Recovery,
        root_thread_id: ThreadId,
        inherits_from_spawn_parent: bool,
    ) -> Result<Self, SecurityPolicyError> {
        let level = recovery.level;
        let home = recovery.home.clone();
        if self.runtime.security_policy.is_initialized()? {
            // Resuming an already-bound root on the same control plane must preserve its
            // binding. Treating it as a new auxiliary agent appends the same principal to its
            // own actor chain, which correctly fails cycle validation but makes root resume
            // impossible.
            if self
                .runtime
                .security_policy
                .snapshot_for_agent(root_thread_id)
                .is_ok()
            {
                return Ok(self);
            }
            if !inherits_from_spawn_parent {
                self.runtime.security_policy.inherit_auxiliary_agent(
                    root_thread_id,
                    format!("task:auxiliary:{root_thread_id}"),
                    level,
                )?;
            }
            return Ok(self);
        }
        let human_authority = PolicyPrincipal::new(
            PrincipalKind::Human,
            format!("human:session:{}", self.session_id),
        )?;
        let persisted = PersistedHumanSecurityState::new(
            SecuritySettings::new(level),
            human_authority,
            recovery.revocations,
        )?;
        let controller = TrustedSecurityController::initialize(
            &self.runtime.security_policy,
            persisted,
            root_thread_id,
            self.session_id,
            if inherits_from_spawn_parent {
                EffectivePolicyInitialization::DetachedSpawnedAgent
            } else if recovery.unreadable.is_some() {
                EffectivePolicyInitialization::UnreadableState
            } else {
                EffectivePolicyInitialization::Root
            },
        )?;
        if let Some(home) = home {
            // Register, then read again: a commit saved between the first
            // read and the registration is not missed.
            self.runtime.security_policy.register_home(&home);
            controller.catch_up(&crate::security::recovery::recover(&home, level));
        }
        // The tree's first initialization wins, as the early returns above already ensure.
        let _ = self.runtime.trusted_security_controller.set(controller);
        Ok(self)
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn effective_security_policy(&self) -> EffectivePolicyView {
        self.runtime.security_policy.clone()
    }

    #[cfg(test)]
    pub(crate) fn trusted_security_controller(&self) -> Option<TrustedSecurityController> {
        self.runtime.trusted_security_controller.get().cloned()
    }

    fn native_auto_loop_states(
        &self,
    ) -> std::sync::MutexGuard<'_, HashMap<ThreadId, NativeAutoLoopState>> {
        self.runtime
            .native_auto_loop_state_by_agent
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// Starts lifecycle accounting for one native agent turn.
    ///
    /// A turn containing only terminal-result mailbox input is automatic. Any user assignment,
    /// follow-up, or other non-terminal task is fresh work and resets a previously paused chain.
    pub(crate) fn begin_native_agent_turn(
        &self,
        agent_id: ThreadId,
        terminal_result_only: bool,
    ) -> NativeAutoTurnGuard {
        let mut states = self.native_auto_loop_states();
        let state = states.entry(agent_id).or_default();
        if terminal_result_only {
            state.auto_turn_running = state.operator_input_seen;
            state.auto_turn_dispatched = false;
        } else {
            state.operator_input_seen = true;
            state.chain = 0;
            state.auto_turn_running = false;
            state.auto_turn_dispatched = false;
        }
        drop(states);
        NativeAutoTurnGuard {
            control: self.clone(),
            agent_id,
        }
    }

    /// Explicit human steering resets a native auto-processing chain even when it joins an
    /// already-running terminal-result turn.
    pub(crate) fn note_native_operator_input(&self, agent_id: ThreadId) {
        let mut states = self.native_auto_loop_states();
        let state = states.entry(agent_id).or_default();
        state.operator_input_seen = true;
        state.chain = 0;
        state.auto_turn_running = false;
        state.auto_turn_dispatched = false;
    }

    /// Records the first successful outbound collaboration action from an automatic manager turn.
    /// Multiple sends in the same turn count as one link in the chain.
    pub(crate) fn note_native_agent_dispatch(&self, agent_id: ThreadId) {
        let mut states = self.native_auto_loop_states();
        let state = states.entry(agent_id).or_default();
        if state.auto_turn_running && !state.auto_turn_dispatched {
            state.auto_turn_dispatched = true;
            state.chain = state.chain.saturating_add(1);
        }
    }

    fn complete_native_agent_turn(&self, agent_id: ThreadId) {
        let mut states = self.native_auto_loop_states();
        let state = states.entry(agent_id).or_default();
        if state.auto_turn_running && !state.auto_turn_dispatched {
            // Acknowledging a child result without creating more work terminates the chain.
            state.chain = 0;
        }
        state.auto_turn_running = false;
        state.auto_turn_dispatched = false;
    }

    fn native_terminal_result_auto_trigger_allowed(&self, agent_id: ThreadId) -> bool {
        let states = self.native_auto_loop_states();
        states.get(&agent_id).is_some_and(|state| {
            state.operator_input_seen && state.chain < CREW_AUTO_DISPATCH_CHAIN_LIMIT
        })
    }

    #[cfg(test)]
    pub(crate) fn native_auto_loop_state_for_test(
        &self,
        agent_id: ThreadId,
    ) -> Option<(bool, u32, bool, bool)> {
        self.native_auto_loop_states().get(&agent_id).map(|state| {
            (
                state.operator_input_seen,
                state.chain,
                state.auto_turn_running,
                state.auto_turn_dispatched,
            )
        })
    }

    /// Send rich user input items from the agent `sender` to an existing agent thread.
    pub(crate) async fn send_input(
        &self,
        agent_id: ThreadId,
        sender: Option<ThreadId>,
        input: Vec<UserInput>,
        start_options: TurnStartOptions,
    ) -> CodexResult<String> {
        let state = self.runtime.upgrade()?;
        let thread = state.get_thread(agent_id).await?;
        // Corbanu: agent-to-agent input starts worker turns, so it is capacity-gated like a
        // turn-starting message (a person's input is never refused for capacity).
        thread
            .ensure_execution_capacity_for_turn_start(self)
            .await?;
        self.send_input_after_capacity_check(agent_id, sender, &state, input, start_options)
            .await
    }

    /// `send_input` once capacity is settled: a newly spawned agent's first task was already
    /// admitted by its spawn.
    pub(super) async fn send_input_after_capacity_check(
        &self,
        agent_id: ThreadId,
        sender: Option<ThreadId>,
        state: &Arc<ThreadManagerState>,
        input: Vec<UserInput>,
        start_options: TurnStartOptions,
    ) -> CodexResult<String> {
        let thread = state.get_thread(agent_id).await?;
        // PF-30-S02: record the sender's standing on this exact input before submitting it.
        Box::pin(register_agent_input_origin(state, agent_id, sender, &input)).await;
        let result = match thread
            .start_or_steer_turn(TurnInputRequest::user_input(input).on_start(start_options))
            .await
        {
            Ok(TurnInputSubmission::Started { turn_id, .. }) => Ok(turn_id),
            Ok(TurnInputSubmission::Steered { .. }) => {
                // MAv1 exposes an opaque `submission_id` to the model. The legacy
                // `Op::UserInput` path returned a fresh ID for every steer, while the
                // turn-input API returns the active turn ID. Keep the tool-visible ID
                // unique without adding a submission receipt back to Core.
                Ok(Uuid::now_v7().to_string())
            }
            Ok(TurnInputSubmission::NotSubmitted { reason }) => Err(CodexErr::InvalidRequest(
                format!("turn input was not submitted: {reason:?}"),
            )),
            Err(err) => Err(err),
        };
        self.handle_thread_request_result(agent_id, state, result)
            .await
    }

    pub(crate) async fn send_inter_agent_communication(
        &self,
        agent_id: ThreadId,
        communication: InterAgentCommunication,
        agent_communication_context: AgentCommunicationContext,
        start_options: TurnStartOptions,
    ) -> CodexResult<String> {
        let state = self.runtime.upgrade()?;
        if communication.trigger_turn {
            let thread = state.get_thread(agent_id).await?;
            thread
                .ensure_execution_capacity_for_turn_start(self)
                .await?;
        }
        self.send_inter_agent_communication_after_capacity_check(
            agent_id,
            &state,
            communication,
            agent_communication_context,
            start_options,
        )
        .await
    }

    pub(crate) async fn emit_sub_agent_activity(
        &self,
        thread_id: ThreadId,
        turn_id: String,
        item: SubAgentActivityItem,
    ) -> CodexResult<()> {
        let state = self.runtime.upgrade()?;
        let thread = state.get_thread(thread_id).await?;
        let started_at_ms = now_unix_timestamp_ms();
        let item = TurnItem::SubAgentActivity(item);
        thread
            .session
            .send_event_raw(Event {
                id: turn_id.clone(),
                msg: EventMsg::ItemStarted(ItemStartedEvent {
                    thread_id,
                    turn_id: turn_id.clone(),
                    item: item.clone(),
                    started_at_ms,
                }),
            })
            .await;
        let completed_at_ms = now_unix_timestamp_ms();
        let completed = ItemCompletedEvent {
            thread_id,
            turn_id: turn_id.clone(),
            item,
            started_at_ms: Some(started_at_ms),
            completed_at_ms,
        };
        thread
            .session
            .send_event_raw(Event {
                id: turn_id.clone(),
                msg: EventMsg::ItemCompleted(completed.clone()),
            })
            .await;
        for legacy in completed.as_legacy_events(/*show_raw_agent_reasoning*/ false) {
            thread
                .session
                .send_event_raw(Event {
                    id: turn_id.clone(),
                    msg: legacy,
                })
                .await;
        }
        Ok(())
    }

    async fn send_inter_agent_communication_after_capacity_check(
        &self,
        agent_id: ThreadId,
        state: &Arc<ThreadManagerState>,
        communication: InterAgentCommunication,
        context: AgentCommunicationContext,
        start_options: TurnStartOptions,
    ) -> CodexResult<String> {
        self.submit_inter_agent_communication(
            agent_id,
            state,
            communication,
            context,
            start_options,
        )
        .await
    }

    async fn submit_inter_agent_communication(
        &self,
        agent_id: ThreadId,
        state: &Arc<ThreadManagerState>,
        mut communication: InterAgentCommunication,
        context: AgentCommunicationContext,
        start_options: TurnStartOptions,
    ) -> CodexResult<String> {
        let observers = state.extensions.message_lifecycle_contributors();
        let delivery = (!observers.is_empty()).then(|| {
            let message_id = communication
                .id
                .take()
                .filter(|id| !id.is_empty())
                .unwrap_or_else(|| ResponseItemId::new("amsg"));
            communication.id = Some(message_id.clone());
            MessageDelivery::publish(context.sender_thread_id, message_id, observers)
        });
        let log_content = crate::agent_communication::send_log_content(&communication);
        // Corbanu (#1058/#1435/#1452/#1545, #1999/#2033): every message goes through the
        // durable mailbox, which admits it before the hand-off and recovers it after a reload,
        // so a message is never lost or delivered twice. Upstream's in-memory queue for
        // unloaded recipients is not durable and takes no new mail.
        {
            // A known recipient that is not loaded (an evicted V2 manager whose child just
            // finished, say) has the message admitted to the durable mailbox; its reload
            // recovers it. Holding the thread map keeps this atomic with publication of a
            // reloaded session, whose recovery then runs after the admission.
            let threads = state.threads.read().await;
            if !threads.contains_key(&agent_id) {
                // As upstream's unloaded path: a tree that is shutting down takes no new mail.
                let _membership = self.runtime.admit_start()?;
                self.runtime.ensure_agent_known(agent_id)?;
                // Read the sender's database handle from the held map: taking the thread map
                // again here could deadlock behind a queued writer.
                let state_db = threads
                    .get(&context.sender_thread_id)
                    .and_then(|sender| sender.state_db());
                let submission_id = self
                    .admit_for_unloaded_recipient(agent_id, state_db, communication)
                    .await?;
                drop(threads);
                crate::agent_communication::emit_agent_communication_send(
                    &submission_id,
                    &context,
                    log_content.as_deref(),
                    agent_id,
                );
                return Ok(submission_id);
            }
        }
        let submission_id = self
            .send_persisted_inter_agent_communication(
                agent_id,
                communication,
                start_options,
                delivery,
            )
            .await?;
        crate::agent_communication::emit_agent_communication_send(
            &submission_id,
            &context,
            log_content.as_deref(),
            agent_id,
        );
        Ok(submission_id)
    }

    /// Whether `agent_id`'s resident session is the one `spawn_agent` created
    /// with a first turn queued (D435). With `PendingInit`, that turn has not
    /// started yet; a session reloaded later reads `PendingInit` with no turn.
    pub(crate) async fn spawned_with_first_turn(&self, agent_id: ThreadId) -> bool {
        let Ok(state) = self.runtime.upgrade() else {
            return false;
        };
        state.get_thread(agent_id).await.is_ok_and(|thread| {
            thread
                .session
                .spawned_with_first_turn
                .load(std::sync::atomic::Ordering::Acquire)
        })
    }

    /// Interrupt the current task for an existing agent thread.
    pub(crate) async fn interrupt_agent(&self, agent_id: ThreadId) -> CodexResult<String> {
        let state = self.runtime.upgrade()?;
        self.handle_thread_request_result(
            agent_id,
            &state,
            state
                .send_op(
                    agent_id,
                    Op::Interrupt,
                    /*parent_turn_id*/ None,
                    /*root_turn_id*/ None,
                    /*message_delivery*/ None,
                )
                .await,
        )
        .await
    }

    async fn handle_thread_request_result(
        &self,
        agent_id: ThreadId,
        state: &Arc<ThreadManagerState>,
        result: CodexResult<String>,
    ) -> CodexResult<String> {
        if result
            .as_ref()
            .is_err_and(|err| matches!(err.details(), CodexErrorDetails::InternalAgentDied))
        {
            let _ = state.remove_thread(&agent_id).await;
            self.forget_v2_residency(agent_id);
            self.runtime.registry.release_spawned_thread(agent_id);
        }
        result
    }

    /// Fetch the last known status for `agent_id`, returning `NotFound` when unavailable.
    pub(crate) async fn get_status(&self, agent_id: ThreadId) -> AgentStatus {
        let Ok(state) = self.runtime.upgrade() else {
            // No agent available if upgrade fails.
            return AgentStatus::NotFound;
        };
        let Ok(thread) = state.get_thread(agent_id).await else {
            return if self
                .runtime
                .registry
                .agent_metadata_for_thread(agent_id)
                .is_some()
            {
                AgentStatus::Unloaded
            } else {
                AgentStatus::NotFound
            };
        };
        thread.agent_status().await
    }

    pub(crate) fn get_agent_metadata(&self, agent_id: ThreadId) -> Option<AgentMetadata> {
        self.runtime.registry.agent_metadata_for_thread(agent_id)
    }

    pub(crate) fn get_agent_metadata_for_path(
        &self,
        agent_path: &AgentPath,
    ) -> Option<AgentMetadata> {
        self.runtime.registry.agent_metadata_for_path(agent_path)
    }

    pub(crate) fn is_root_thread(&self, agent_id: ThreadId) -> bool {
        self.runtime.registry.agent_id_for_path(&AgentPath::root()) == Some(agent_id)
    }

    /// Whether `thread_id` is loaded and is the root of its own tree: a thread a client
    /// started (such as a pane the person opened), not a spawned or internal agent.
    pub(crate) async fn is_loaded_root_thread(&self, thread_id: ThreadId) -> bool {
        let Ok(state) = self.runtime.upgrade() else {
            return false;
        };
        state
            .get_thread(thread_id)
            .await
            .is_ok_and(|thread| !thread.session_source.is_non_root_agent())
    }

    /// Whether native terminal mailbox items should wake this thread automatically.
    ///
    /// Persistent crew managers process child results without a TUI-side report queue.
    /// Human-facing roots and ephemeral task agents retain ordinary Codex behavior.
    pub(crate) async fn auto_processes_terminal_results(&self, agent_id: ThreadId) -> bool {
        let Ok(state) = self.runtime.upgrade() else {
            return false;
        };
        let Ok(thread) = state.get_thread(agent_id).await else {
            return false;
        };
        let is_persistent_crew_member = matches!(
            thread.session_source.get_agent_class(),
            Some(codex_protocol::crew::AgentClass::CrewMember { .. })
        );
        is_persistent_crew_member && self.native_terminal_result_auto_trigger_allowed(agent_id)
    }

    pub(crate) fn record_agent_result_status(
        &self,
        agent_id: ThreadId,
        status: &AgentStatus,
    ) -> Option<String> {
        let result_message = result_message_from_status(status);
        match result_message.as_ref() {
            Some(message) => self
                .runtime
                .registry
                .update_last_result_message(agent_id, message.clone()),
            None => self.runtime.registry.clear_last_result_message(agent_id),
        }
        result_message
    }

    pub(crate) fn register_thread_spawn_metadata(
        &self,
        thread_id: ThreadId,
        session_source: &SessionSource,
    ) {
        let SessionSource::SubAgent(SubAgentSource::ThreadSpawn {
            agent_path,
            agent_nickname,
            agent_role,
            ..
        }) = session_source
        else {
            return;
        };
        self.runtime
            .registry
            .register_spawned_thread(AgentMetadata {
                agent_id: Some(thread_id),
                agent_path: agent_path.clone(),
                agent_nickname: agent_nickname.clone(),
                agent_role: agent_role.clone(),
                last_task_message: None,
                last_result_message: None,
            });
    }

    pub(crate) fn restore_thread_spawn_metadata(
        &self,
        thread_id: ThreadId,
        session_source: &SessionSource,
    ) {
        let SessionSource::SubAgent(SubAgentSource::ThreadSpawn {
            agent_path,
            agent_nickname,
            agent_role,
            ..
        }) = session_source
        else {
            return;
        };
        self.runtime.registry.restore_spawned_thread(AgentMetadata {
            agent_id: Some(thread_id),
            agent_path: agent_path.clone(),
            agent_nickname: agent_nickname.clone(),
            agent_role: agent_role.clone(),
            last_task_message: None,
            last_result_message: None,
        });
        self.ensure_thread_security_inheritance(thread_id, session_source);
    }

    fn ensure_thread_security_inheritance(
        &self,
        thread_id: ThreadId,
        session_source: &SessionSource,
    ) {
        if self
            .runtime
            .security_policy
            .snapshot_for_agent(thread_id)
            .is_ok()
        {
            return;
        }
        let SessionSource::SubAgent(SubAgentSource::ThreadSpawn {
            parent_thread_id, ..
        }) = session_source
        else {
            return;
        };
        let Ok(parent) = self
            .runtime
            .security_policy
            .snapshot_for_agent(*parent_thread_id)
        else {
            warn!(
                %thread_id,
                %parent_thread_id,
                "cannot restore child security binding without its parent binding"
            );
            return;
        };
        if let Err(error) = self.runtime.security_policy.inherit_child(
            *parent_thread_id,
            thread_id,
            format!("task:restore:{thread_id}"),
            parent.level,
        ) {
            warn!(%thread_id, %error, "failed to restore child security binding");
        }
    }

    pub(crate) async fn restore_persisted_agent_subtree(
        &self,
        root_thread_id: ThreadId,
    ) -> CodexResult<()> {
        let state = self.runtime.upgrade()?;
        let Some(agent_graph_store) = state.agent_graph_store() else {
            return Ok(());
        };
        let descendant_ids = match agent_graph_store
            .list_thread_spawn_descendants(
                root_thread_id,
                Some(codex_agent_graph_store::ThreadSpawnEdgeStatus::Open),
            )
            .await
        {
            Ok(descendant_ids) => descendant_ids,
            Err(err) => {
                warn!(
                    %root_thread_id,
                    %err,
                    "failed to enumerate persisted agent subtree during resume"
                );
                return Ok(());
            }
        };
        for thread_id in descendant_ids {
            let stored_thread = match state
                .read_stored_thread(ReadThreadParams {
                    thread_id,
                    include_archived: true,
                    include_history: false,
                })
                .await
            {
                Ok(stored_thread) => stored_thread,
                Err(err) => {
                    warn!(
                        %root_thread_id,
                        %thread_id,
                        %err,
                        "failed to restore one persisted agent during resume"
                    );
                    continue;
                }
            };
            self.restore_thread_spawn_metadata(thread_id, &stored_thread.source);
        }
        Ok(())
    }

    /// #1701: whether `agent_id` is a loaded thread that `caller` did not
    /// spawn, directly or through its own agents. `wait_agent` reads only
    /// the caller's own agents; an id that is not loaded is not foreign
    /// (waiting reports it as not found).
    pub(crate) async fn is_foreign_loaded_thread(
        &self,
        caller: ThreadId,
        agent_id: ThreadId,
    ) -> CodexResult<bool> {
        let state = self.runtime.upgrade()?;
        if state.get_thread(agent_id).await.is_err() {
            return Ok(false);
        }
        Ok(!self
            .runtime
            .live_thread_spawn_descendants(caller)
            .await?
            .contains(&agent_id))
    }

    /// #1701: the names `list_agents` gives the loaded agents `caller`
    /// spawned, directly or through its own agents.
    pub(crate) async fn spawn_descendant_agent_names(
        &self,
        caller: ThreadId,
    ) -> CodexResult<Vec<String>> {
        Ok(self
            .runtime
            .live_thread_spawn_descendants(caller)
            .await?
            .into_iter()
            .map(|thread_id| {
                self.get_agent_metadata(thread_id)
                    .and_then(|metadata| metadata.agent_path)
                    .map(|path| path.to_string())
                    .unwrap_or_else(|| thread_id.to_string())
            })
            .collect())
    }

    /// #1756: the names `list_agents` gives the agents `caller` spawned,
    /// directly or through its own agents: the loaded ones, and those not
    /// loaded now (open edges in the agent graph, listed as unloaded).
    pub(crate) async fn own_agent_names(&self, caller: ThreadId) -> CodexResult<Vec<String>> {
        let mut thread_ids = self.runtime.live_thread_spawn_descendants(caller).await?;
        let state = self.runtime.upgrade()?;
        if let Some(agent_graph_store) = state.agent_graph_store()
            && let Ok(persisted) = agent_graph_store
                .list_thread_spawn_descendants(
                    caller,
                    Some(codex_agent_graph_store::ThreadSpawnEdgeStatus::Open),
                )
                .await
        {
            thread_ids.extend(persisted);
        }
        Ok(thread_ids
            .into_iter()
            .map(|thread_id| {
                self.get_agent_metadata(thread_id)
                    .and_then(|metadata| metadata.agent_path)
                    .map(|path| path.to_string())
                    .unwrap_or_else(|| thread_id.to_string())
            })
            .collect())
    }

    pub(crate) async fn get_agent_config_snapshot(
        &self,
        agent_id: ThreadId,
    ) -> Option<ThreadConfigSnapshot> {
        let Ok(state) = self.runtime.upgrade() else {
            return None;
        };
        let Ok(thread) = state.get_thread(agent_id).await else {
            return None;
        };
        Some(thread.config_snapshot().await)
    }

    /// The live session of `thread_id`, if this thread manager holds it. Used
    /// to attribute paid work done for a conversation that is not persisted
    /// itself (PF-60-S06).
    pub(crate) async fn live_session(
        &self,
        thread_id: ThreadId,
    ) -> Option<Arc<crate::session::session::Session>> {
        let state = self.runtime.upgrade().ok()?;
        let thread = state.get_thread(thread_id).await.ok()?;
        Some(Arc::clone(&thread.session))
    }

    pub(crate) async fn list_agents(
        &self,
        current_session_source: &SessionSource,
        path_prefix: Option<&str>,
    ) -> CodexResult<Vec<LiveAgent>> {
        let state = self.runtime.upgrade()?;
        let resolved_prefix = path_prefix
            .map(|prefix| {
                current_session_source
                    .get_agent_path()
                    .unwrap_or_else(AgentPath::root)
                    .resolve(prefix)
                    .map_err(CodexErr::UnsupportedOperation)
            })
            .transpose()?;

        let mut live_agents = self.runtime.registry.live_agents();
        live_agents.sort_by(|left, right| {
            left.agent_path
                .as_deref()
                .unwrap_or_default()
                .cmp(right.agent_path.as_deref().unwrap_or_default())
                .then_with(|| {
                    left.agent_id
                        .map(|id| id.to_string())
                        .unwrap_or_default()
                        .cmp(&right.agent_id.map(|id| id.to_string()).unwrap_or_default())
                })
        });

        let root_path = AgentPath::root();
        let mut agents = Vec::with_capacity(live_agents.len().saturating_add(1));
        if resolved_prefix
            .as_ref()
            .is_none_or(|prefix| agent_matches_prefix(Some(&root_path), prefix))
            && let Some(root_thread_id) = self.runtime.registry.agent_id_for_path(&root_path)
            && let Ok(root_thread) = state.get_thread(root_thread_id).await
        {
            agents.push(LiveAgent {
                thread_id: root_thread_id,
                metadata: AgentMetadata {
                    agent_id: Some(root_thread_id),
                    agent_path: Some(root_path),
                    last_task_message: Some(ROOT_LAST_TASK_MESSAGE.to_string()),
                    ..Default::default()
                },
                status: root_thread.agent_status().await,
            });
        }

        for metadata in live_agents {
            let Some(thread_id) = metadata.agent_id else {
                continue;
            };
            if resolved_prefix
                .as_ref()
                .is_some_and(|prefix| !agent_matches_prefix(metadata.agent_path.as_ref(), prefix))
            {
                continue;
            }

            // Corbanu (#1756): a registered agent that is not loaded now is listed as
            // `Unloaded`, with its last result when it had one.
            let status = match state.get_thread(thread_id).await {
                Ok(thread) => thread.agent_status().await,
                Err(_) => AgentStatus::Unloaded,
            };
            let mut metadata = metadata;
            if metadata.last_result_message.is_none() {
                metadata.last_result_message = result_message_from_status(&status);
            }
            agents.push(LiveAgent {
                thread_id,
                metadata,
                status,
            });
        }

        Ok(agents)
    }

    /// Starts a detached watcher for sub-agents spawned from another thread.
    ///
    /// This is only enabled for `SubAgentSource::ThreadSpawn`, where a parent thread exists and
    /// can receive completion notifications.
    fn maybe_start_completion_watcher(
        &self,
        child_thread_id: ThreadId,
        session_source: Option<SessionSource>,
        child_reference: String,
        child_agent_path: Option<AgentPath>,
    ) {
        let Some(SessionSource::SubAgent(SubAgentSource::ThreadSpawn {
            parent_thread_id, ..
        })) = session_source
        else {
            return;
        };
        let Ok(membership) = self.runtime.admit_start() else {
            return;
        };
        let teardown = membership.into_teardown_guard("completion_watcher", Some(child_thread_id));
        let control = self.clone();
        let watcher = async move {
            let status = match control.subscribe_status(child_thread_id).await {
                Ok(mut updates) => {
                    let mut final_status = None;
                    while let Some(Ok(snapshot)) = tokio::select! {
                        biased;
                        _ = control.runtime.shutdown.cancelled() => return,
                        update = updates.next() => update,
                    } {
                        if let Some(status) = snapshot.status()
                            && is_final(status)
                        {
                            final_status = Some(status.clone());
                            break;
                        }
                    }
                    match final_status {
                        Some(status) => status,
                        None => control.get_status(child_thread_id).await,
                    }
                }
                Err(_) => control.get_status(child_thread_id).await,
            };
            // Tree shutdown is a lifecycle handoff, not a child result for the parent.
            if !is_final(&status)
                || (matches!(&status, AgentStatus::Shutdown)
                    && control.runtime.shutdown.is_cancelled())
            {
                return;
            }
            control.record_agent_result_status(child_thread_id, &status);

            let Ok(state) = control.runtime.upgrade() else {
                return;
            };
            let child_thread = state.get_thread(child_thread_id).await.ok();
            let child_uses_multi_agent_v2 = match child_thread.as_ref() {
                Some(child_thread) => {
                    child_thread.multi_agent_version() == Some(MultiAgentVersion::V2)
                }
                None => true,
            };
            if child_agent_path.is_some() && child_uses_multi_agent_v2 {
                let Some(child_agent_path) = child_agent_path.clone() else {
                    return;
                };
                let Some(parent_agent_path) = child_agent_path
                    .as_str()
                    .rsplit_once('/')
                    .and_then(|(parent, _)| AgentPath::try_from(parent).ok())
                else {
                    return;
                };
                let Some(message) = format_inter_agent_completion_message(
                    parent_agent_path.clone(),
                    child_agent_path.clone(),
                    &status,
                ) else {
                    return;
                };
                let communication = InterAgentCommunication::new(
                    child_agent_path,
                    parent_agent_path,
                    Vec::new(),
                    message,
                    /*trigger_turn*/ false,
                )
                .with_kind(AgentMessageKind::TerminalResult);
                let context =
                    AgentCommunicationContext::new(AgentCommunicationKind::Result, child_thread_id);
                let _ = control
                    .send_inter_agent_communication(
                        parent_thread_id,
                        communication,
                        context,
                        TurnStartOptions::default(),
                    )
                    .await;
                return;
            }
            let Ok(parent_thread) = state.get_thread(parent_thread_id).await else {
                return;
            };
            let config = parent_thread.config().await;
            let namespace = match parent_thread
                .multi_agent_version()
                .unwrap_or_else(|| config.multi_agent_version_from_features())
            {
                MultiAgentVersion::Disabled => None,
                MultiAgentVersion::V1 => Some(ContentItemNamespace::MultiAgentV1),
                MultiAgentVersion::V2 => Some(
                    config
                        .multi_agent_v2
                        .tool_namespace
                        .clone()
                        .map(ContentItemNamespace::from)
                        .unwrap_or(ContentItemNamespace::Functions),
                ),
            };
            parent_thread
                .inject_agent_result_fragment_without_turn(
                    SubagentNotification::new(child_reference.as_str(), status)
                        .with_metadata(ContentItemMetadata::tool(namespace)),
                )
                .await;
        };
        tokio::spawn(async move {
            watcher.await;
            teardown.complete();
        });
    }

    fn prepare_agent_metadata(
        &self,
        reservation: &mut crate::agent::registry::SpawnReservation,
        config: &Config,
        agent_path: Option<AgentPath>,
        agent_role: Option<String>,
        preferred_agent_nickname: Option<String>,
    ) -> CodexResult<AgentMetadata> {
        if let Some(agent_path) = agent_path.as_ref() {
            reservation.reserve_agent_path(agent_path)?;
        }
        let candidate_names = agent_nickname_candidates(config, agent_role.as_deref());
        let candidate_name_refs: Vec<&str> = candidate_names.iter().map(String::as_str).collect();
        let agent_nickname = Some(reservation.reserve_agent_nickname_with_preference(
            &candidate_name_refs,
            preferred_agent_nickname.as_deref(),
        )?);
        Ok(AgentMetadata {
            agent_id: None,
            agent_path,
            agent_nickname,
            agent_role,
            ..Default::default()
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn prepare_thread_spawn(
        &self,
        reservation: &mut crate::agent::registry::SpawnReservation,
        config: &Config,
        parent_thread_id: ThreadId,
        depth: i32,
        agent_path: Option<AgentPath>,
        agent_role: Option<String>,
        preferred_agent_nickname: Option<String>,
        agent_class: Option<codex_protocol::crew::AgentClass>,
    ) -> CodexResult<(SessionSource, AgentMetadata)> {
        if depth == 1 {
            self.runtime.registry.register_root_thread(parent_thread_id);
        }
        let agent_metadata = self.prepare_agent_metadata(
            reservation,
            config,
            agent_path,
            agent_role,
            preferred_agent_nickname,
        )?;
        let session_source = SessionSource::SubAgent(SubAgentSource::ThreadSpawn {
            parent_thread_id,
            depth,
            agent_path: agent_metadata.agent_path.clone(),
            agent_nickname: agent_metadata.agent_nickname.clone(),
            agent_role: agent_metadata.agent_role.clone(),
            agent_class,
        });
        Ok((session_source, agent_metadata))
    }

    async fn inherited_environments_for_source(
        &self,
        state: &Arc<ThreadManagerState>,
        session_source: Option<&SessionSource>,
    ) -> Option<TurnEnvironmentSnapshot> {
        let parent_thread_id = session_source?.parent_thread_id()?;
        let parent_thread = state.get_thread(parent_thread_id).await.ok()?;
        Some(parent_thread.session.inherited_environment_snapshot().await)
    }

    /// Establishes the durable parent side of a native spawn before a child can be created.
    ///
    /// A lazily started human root has no resumable rollout until its first material write. A
    /// durable child must never be committed below that transient root: the graph edge would
    /// survive while the parent needed to reconstruct and address it would not.
    pub(crate) async fn prepare_durable_thread_spawn_parent(
        &self,
        session_source: &SessionSource,
        child_is_ephemeral: bool,
    ) -> CodexResult<()> {
        if child_is_ephemeral {
            return Ok(());
        }
        let Some(parent_thread_id) = session_source.parent_thread_id() else {
            return Ok(());
        };
        let state = self.runtime.upgrade()?;
        let parent_thread = state.get_thread(parent_thread_id).await?;
        parent_thread
            .session
            .try_ensure_rollout_materialized()
            .await
            .map_err(|err| {
                CodexErr::Fatal(format!(
                    "failed to materialize native agent parent {parent_thread_id}: {err}"
                ))
            })?;
        parent_thread.flush_rollout().await.map_err(|err| {
            CodexErr::Fatal(format!(
                "failed to flush native agent parent {parent_thread_id}: {err}"
            ))
        })
    }

    /// Persists the child before publishing its durable Core graph edge.
    pub(crate) async fn persist_durable_thread_spawn(
        &self,
        child_thread: &crate::CodexThread,
        child_thread_id: ThreadId,
        session_source: Option<&SessionSource>,
    ) -> CodexResult<()> {
        if child_thread.config_snapshot().await.ephemeral {
            return Ok(());
        }
        if session_source
            .and_then(SessionSource::parent_thread_id)
            .is_none()
        {
            return Ok(());
        }
        child_thread
            .session
            .try_ensure_rollout_materialized()
            .await
            .map_err(|err| {
                CodexErr::Fatal(format!(
                    "failed to materialize native agent child {child_thread_id}: {err}"
                ))
            })?;
        child_thread.flush_rollout().await.map_err(|err| {
            CodexErr::Fatal(format!(
                "failed to flush native agent child {child_thread_id}: {err}"
            ))
        })?;
        self.persist_thread_spawn_edge_for_source(
            /*ephemeral*/ false,
            child_thread_id,
            session_source,
        )
        .await
        .map(|_| ())
    }

    /// Returns true after publication, or false when the source or configuration skips it.
    async fn persist_thread_spawn_edge_for_source(
        &self,
        ephemeral: bool,
        child_thread_id: ThreadId,
        session_source: Option<&SessionSource>,
    ) -> CodexResult<bool> {
        let Some(parent_thread_id) = session_source.and_then(SessionSource::parent_thread_id)
        else {
            return Ok(false);
        };
        if ephemeral {
            return Ok(false);
        }
        let state = self.runtime.upgrade()?;
        let Some(agent_graph_store) = state.agent_graph_store() else {
            return Ok(false);
        };
        agent_graph_store
            .upsert_thread_spawn_edge(
                parent_thread_id,
                child_thread_id,
                codex_agent_graph_store::ThreadSpawnEdgeStatus::Open,
            )
            .await
            .map_err(|err| {
                CodexErr::Fatal(format!(
                    "failed to persist native agent edge {parent_thread_id} -> {child_thread_id}: {err}"
                ))
            })?;
        Ok(true)
    }
}

async fn inherited_exec_policy_for_source(
    state: &ThreadManagerState,
    session_source: Option<&SessionSource>,
    child_config: &Config,
) -> Option<Arc<crate::exec_policy::ExecPolicyManager>> {
    let parent_thread_id = session_source?.parent_thread_id()?;
    let parent_thread = state.get_thread(parent_thread_id).await.ok()?;
    inherited_exec_policy_for_parent(&parent_thread, child_config).await
}

pub(in crate::agent) async fn inherited_exec_policy_for_parent(
    parent_thread: &crate::CodexThread,
    child_config: &Config,
) -> Option<Arc<crate::exec_policy::ExecPolicyManager>> {
    let parent_config = parent_thread.session.get_config().await;
    if !crate::exec_policy::child_uses_parent_exec_policy(&parent_config, child_config) {
        return None;
    }

    Some(Arc::clone(&parent_thread.session.services.exec_policy))
}

fn agent_matches_prefix(agent_path: Option<&AgentPath>, prefix: &AgentPath) -> bool {
    if prefix.is_root() {
        return true;
    }

    agent_path.is_some_and(|agent_path| {
        agent_path == prefix
            || agent_path
                .as_str()
                .strip_prefix(prefix.as_str())
                .is_some_and(|suffix| suffix.starts_with('/'))
    })
}

pub(crate) fn render_input_preview(input: &[UserInput]) -> String {
    input
        .iter()
        .map(|item| match item {
            UserInput::Text { text, .. } => text.clone(),
            UserInput::Image { .. } => "[image]".to_string(),
            UserInput::LocalImage { path, .. } => {
                format!("[local_image:{}]", path.display())
            }
            UserInput::Audio { .. } => "[audio]".to_string(),
            UserInput::LocalAudio { path } => {
                format!("[local_audio:{}]", path.display())
            }
            UserInput::Skill { name, path, .. } => {
                format!("[skill:${name}]({})", path.display())
            }
            UserInput::Mention { name, path, .. } => format!("[mention:${name}]({path})"),
            _ => "[input]".to_string(),
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn thread_spawn_depth(session_source: &SessionSource) -> Option<i32> {
    match session_source {
        SessionSource::SubAgent(SubAgentSource::ThreadSpawn { depth, .. }) => Some(*depth),
        _ => None,
    }
}

fn last_task_message_from_communication(communication: &InterAgentCommunication) -> Option<String> {
    if communication.encrypted_content.is_some() {
        return None;
    }
    non_empty_bounded_message(communication.content.clone(), LAST_TASK_MESSAGE_MAX_CHARS)
}

fn result_message_from_status(status: &AgentStatus) -> Option<String> {
    match status {
        AgentStatus::Completed(Some(message)) => {
            non_empty_bounded_message(message.clone(), LAST_RESULT_MESSAGE_MAX_CHARS)
        }
        AgentStatus::Completed(None) => None,
        AgentStatus::Errored(error) => non_empty_bounded_message(
            format!("Agent errored: {error}"),
            LAST_RESULT_MESSAGE_MAX_CHARS,
        ),
        AgentStatus::Shutdown => Some("Agent shut down.".to_string()),
        AgentStatus::NotFound => Some("Agent was not found.".to_string()),
        AgentStatus::PendingInit
        | AgentStatus::Unloaded
        | AgentStatus::Running
        | AgentStatus::Interrupted => None,
    }
}

fn nickname_from_picker_label(agent_reference: &str) -> Option<&str> {
    let agent_reference = agent_reference.trim();
    let (nickname, role_suffix) = agent_reference.rsplit_once(" [")?;
    if nickname.trim().is_empty() || !role_suffix.ends_with(']') {
        return None;
    }
    Some(nickname.trim())
}

fn non_empty_bounded_message(message: String, max_chars: usize) -> Option<String> {
    let message = message.trim();
    if message.is_empty() {
        return None;
    }
    let mut chars = message.chars();
    let preview = chars.by_ref().take(max_chars).collect::<String>();
    if chars.next().is_some() {
        Some(format!("{preview}..."))
    } else {
        Some(preview)
    }
}

/// PF-30-S02 lineage. Text one agent hands another arrives as `Op::UserInput`,
/// which the receiver would otherwise record with human standing. Mark the
/// exact input on the receiver before submitting it; its prompt seam then
/// records it with the sender's standing: host only while every item in the
/// sender's history had standing, otherwise agent data. An unknown sender
/// counts as agent data. Only the receiver's `source_envelopes` mode records
/// anything; otherwise a no-op.
async fn register_agent_input_origin(
    state: &Arc<ThreadManagerState>,
    agent_id: ThreadId,
    sender: Option<ThreadId>,
    input: &[UserInput],
) {
    use crate::security::ingress::MessageOrigin;
    let Ok(receiver) = state.get_thread(agent_id).await else {
        return;
    };
    let client = receiver.session.services.model_client();
    if !client.source_envelopes_enabled() {
        return;
    }
    let origin = match sender {
        Some(sender) => match state.get_thread(sender).await {
            Ok(sender) => sender.session.agent_handoff_origin().await,
            Err(_) => MessageOrigin::External(codex_protocol::provenance::SourceKind::ChildAgent),
        },
        None => MessageOrigin::External(codex_protocol::provenance::SourceKind::ChildAgent),
    };
    client.mark_agent_input(input, origin);
}

#[cfg(test)]
#[path = "control_tests.rs"]
mod tests;
