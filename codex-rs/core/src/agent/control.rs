use crate::TurnInputRequest;
use crate::TurnInputSubmission;
use crate::TurnStartOptions;
use crate::agent::AgentStatus;
<<<<<<< HEAD
use crate::agent::registry::AgentMetadata;
use crate::agent::registry::AgentRegistry;
use crate::agent::role::agent_nickname_candidates;
||||||| 413492cd6c
use crate::agent::registry::AgentMetadata;
use crate::agent::registry::AgentRegistry;
use crate::agent::role::DEFAULT_ROLE_NAME;
use crate::agent::role::resolve_role_config;
=======
use crate::agent::role::DEFAULT_ROLE_NAME;
use crate::agent::role::resolve_role_config;
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
use crate::agent::status::is_final;
use crate::agent::types::AgentMetadata;
use crate::agent::types::LiveAgent;
use crate::agent_communication::AgentCommunicationContext;
use crate::agent_communication::AgentCommunicationKind;
use crate::agent_communication::MessageDelivery;
use crate::config::Config;
use crate::config::RolloutBudgetConfig;
use crate::context::ContextualUserFragment;
use crate::context::SubagentNotification;
use crate::environment_selection::TurnEnvironmentSnapshot;
<<<<<<< HEAD
use crate::rollout_budget::RolloutBudget;
use crate::security::EffectivePolicyInitialization;
use crate::security::EffectivePolicyView;
use crate::security::PersistedHumanSecurityState;
use crate::security::SecurityPolicyError;
use crate::security::TrustedSecurityController;
use crate::session::emit_subagent_session_started;
||||||| 413492cd6c
use crate::rollout_budget::RolloutBudget;
use crate::session::emit_subagent_session_started;
=======
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
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
<<<<<<< HEAD
use codex_protocol::models::ContentItem;
use codex_protocol::models::MessagePhase;
use codex_protocol::models::ResponseItem;
use codex_protocol::protocol::AgentMessageKind;
||||||| 413492cd6c
use codex_protocol::models::ContentItem;
use codex_protocol::models::MessagePhase;
use codex_protocol::models::ResponseItem;
=======
use codex_protocol::items::SubAgentActivityItem;
use codex_protocol::items::TurnItem;
use codex_protocol::models::ContentItemMetadata;
use codex_protocol::models::ContentItemNamespace;
use codex_protocol::protocol::Event;
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
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
<<<<<<< HEAD
use codex_security_policy::PolicyPrincipal;
use codex_security_policy::PrincipalKind;
#[cfg(test)]
use codex_security_policy::RevocationState;
#[cfg(test)]
use codex_security_policy::SecurityLevel;
use codex_security_policy::SecuritySettings;
use codex_thread_store::LoadThreadHistoryParams;
||||||| 413492cd6c
use codex_thread_store::LoadThreadHistoryParams;
=======
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
use codex_thread_store::ReadThreadParams;
<<<<<<< HEAD
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
||||||| 413492cd6c
use serde::Serialize;
use std::collections::HashMap;
=======
use futures::StreamExt;
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::Mutex;
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

<<<<<<< HEAD
const LAST_TASK_MESSAGE_MAX_CHARS: usize = 240;
const LAST_RESULT_MESSAGE_MAX_CHARS: usize = 500;
const ROOT_LAST_TASK_MESSAGE: &str = "Main thread";
||||||| 413492cd6c
=======
mod api;
mod budget;
mod completion;
mod delivery;
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
mod execution;
<<<<<<< HEAD
mod held_mailbox_warning;
||||||| 413492cd6c
=======
mod inspection;
mod interrupt;
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
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

<<<<<<< HEAD
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum SpawnAgentForkMode {
    FullHistory,
    LastNTurns(usize),
}

#[derive(Clone, Debug, Default)]
pub(crate) struct SpawnAgentOptions {
    pub(crate) fork_parent_spawn_call_id: Option<String>,
    pub(crate) fork_mode: Option<SpawnAgentForkMode>,
    pub(crate) parent_thread_id: Option<ThreadId>,
    pub(crate) parent_turn_id: Option<String>,
    pub(crate) environments: Option<Vec<TurnEnvironmentSelection>>,
}

#[derive(Clone, Debug)]
pub(crate) struct LiveAgent {
    pub(crate) thread_id: ThreadId,
    pub(crate) metadata: AgentMetadata,
    pub(crate) status: AgentStatus,
}

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
    control: AgentControl,
    agent_id: ThreadId,
}

impl Drop for NativeAutoTurnGuard {
    fn drop(&mut self) {
        self.control.complete_native_agent_turn(self.agent_id);
    }
}

/// Control-plane handle for multi-agent operations.
/// `AgentControl` is held by each session (via `SessionServices`). It provides capability to
/// spawn new agents and the inter-agent communication layer.
/// An `AgentControl` instance is intended to be created at most once per root thread/session
/// tree. That same `AgentControl` is then shared with every sub-agent spawned from that root,
/// which keeps the registry scoped to that root thread rather than the entire `ThreadManager`.
#[derive(Clone, Default)]
pub(crate) struct AgentControl {
    /// ID shared by the whole agent control session. This means every sub-agents from a common
    /// root share the same session ID.
||||||| 413492cd6c
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum SpawnAgentForkMode {
    FullHistory,
    LastNTurns(usize),
}

#[derive(Clone, Debug, Default)]
pub(crate) struct SpawnAgentOptions {
    pub(crate) fork_parent_spawn_call_id: Option<String>,
    pub(crate) fork_mode: Option<SpawnAgentForkMode>,
    pub(crate) parent_thread_id: Option<ThreadId>,
    pub(crate) parent_turn_id: Option<String>,
    pub(crate) environments: Option<Vec<TurnEnvironmentSelection>>,
}

#[derive(Clone, Debug)]
pub(crate) struct LiveAgent {
    pub(crate) thread_id: ThreadId,
    pub(crate) metadata: AgentMetadata,
    pub(crate) status: AgentStatus,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub(crate) struct ListedAgent {
    pub(crate) agent_name: String,
    pub(crate) agent_status: AgentStatus,
}

/// Control-plane handle for multi-agent operations.
/// `AgentControl` is held by each session (via `SessionServices`). It provides capability to
/// spawn new agents and the inter-agent communication layer.
/// An `AgentControl` instance is intended to be created at most once per root thread/session
/// tree. That same `AgentControl` is then shared with every sub-agent spawned from that root,
/// which keeps the registry scoped to that root thread rather than the entire `ThreadManager`.
#[derive(Clone, Default)]
pub(crate) struct AgentControl {
    /// ID shared by the whole agent control session. This means every sub-agents from a common
    /// root share the same session ID.
=======
/// Per-session controller handle for a local agent tree.
/// Handles retain a session identity and share their tree's `LocalAgentRuntime`.
/// Local startup preserves that state when creating or resuming children.
#[derive(Clone)]
pub(crate) struct LocalAgentControl {
    /// session_id is equal to the root thread's ID.
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
    session_id: SessionId,
<<<<<<< HEAD
    /// Weak handle back to the global thread registry/state.
    /// This is `Weak` to avoid reference cycles and shadow persistence of the form
    /// `ThreadManagerState -> CodexThread -> Session -> SessionServices -> ThreadManagerState`.
    manager: Weak<ThreadManagerState>,
    state: Arc<AgentRegistry>,
    v2_residency: Arc<V2Residency>,
    agent_execution_limiter: Arc<AgentExecutionLimiter>,
    /// Session-scoped state shared by the root thread and every cloned sub-agent control handle.
    rollout_budget: Arc<RolloutBudget>,
    /// Native terminal-result auto-processing policy, shared across the whole agent tree. This is
    /// lifecycle policy over Core turns, not a second mailbox or task queue.
    native_auto_loop_state_by_agent: Arc<Mutex<HashMap<ThreadId, NativeAutoLoopState>>>,
    /// Read-only policy and child-inheritance capability shared by the agent tree.
    security_policy: EffectivePolicyView,
    /// Separate trusted mutation capability retained for the future human TUI
    /// path. No model/tool-facing method exposes it.
    trusted_security_controller: Option<TrustedSecurityController>,
||||||| 413492cd6c
    /// Weak handle back to the global thread registry/state.
    /// This is `Weak` to avoid reference cycles and shadow persistence of the form
    /// `ThreadManagerState -> CodexThread -> Session -> SessionServices -> ThreadManagerState`.
    manager: Weak<ThreadManagerState>,
    state: Arc<AgentRegistry>,
    v2_residency: Arc<V2Residency>,
    agent_execution_limiter: Arc<AgentExecutionLimiter>,
    /// Session-scoped state shared by the root thread and every cloned sub-agent control handle.
    rollout_budget: Arc<RolloutBudget>,
=======
    pub(crate) runtime: LocalAgentRuntime,
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
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
<<<<<<< HEAD
        // Every control handle created by one ThreadManager must observe one lifecycle policy.
        // App-server `/spawn` can legitimately materialize a new control handle while loading a
        // persisted parent/child edge; sourcing the state from the manager prevents those handles
        // from becoming independent loop-breaker islands.
        let native_auto_loop_state_by_agent = manager
            .upgrade()
            .map(|state| Arc::clone(&state.native_auto_loop_state_by_agent))
            .unwrap_or_default();
        let control = Self {
            manager,
            native_auto_loop_state_by_agent,
            ..Default::default()
        };
        if let Some(rollout_budget) = rollout_budget {
            control.rollout_budget.configure(rollout_budget);
||||||| 413492cd6c
        let control = Self {
            manager,
            ..Default::default()
        };
        if let Some(rollout_budget) = rollout_budget {
            control.rollout_budget.configure(rollout_budget);
=======
        Self {
            session_id: SessionId::default(),
            runtime: LocalAgentRuntime::new(manager, thread_id_generator, rollout_budget),
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
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

<<<<<<< HEAD
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
        mut self,
        recovery: crate::security::recovery::Recovery,
        root_thread_id: ThreadId,
        inherits_from_spawn_parent: bool,
    ) -> Result<Self, SecurityPolicyError> {
        let level = recovery.level;
        let home = recovery.home.clone();
        if self.security_policy.is_initialized()? {
            // Resuming an already-bound root on the same control plane must preserve its
            // binding. Treating it as a new auxiliary agent appends the same principal to its
            // own actor chain, which correctly fails cycle validation but makes root resume
            // impossible.
            if self
                .security_policy
                .snapshot_for_agent(root_thread_id)
                .is_ok()
            {
                return Ok(self);
            }
            if !inherits_from_spawn_parent {
                self.security_policy.inherit_auxiliary_agent(
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
            &self.security_policy,
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
            self.security_policy.register_home(&home);
            controller.catch_up(&crate::security::recovery::recover(&home, level));
        }
        self.trusted_security_controller = Some(controller);
        Ok(self)
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn effective_security_policy(&self) -> EffectivePolicyView {
        self.security_policy.clone()
    }

    #[cfg(test)]
    pub(crate) fn trusted_security_controller(&self) -> Option<TrustedSecurityController> {
        self.trusted_security_controller.clone()
    }

    pub(crate) fn rollout_budget(&self) -> &RolloutBudget {
        self.rollout_budget.as_ref()
    }

    fn native_auto_loop_states(
        &self,
    ) -> std::sync::MutexGuard<'_, HashMap<ThreadId, NativeAutoLoopState>> {
        self.native_auto_loop_state_by_agent
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
||||||| 413492cd6c
    pub(crate) fn rollout_budget(&self) -> &RolloutBudget {
        self.rollout_budget.as_ref()
    }

    /// Send rich user input items to an existing agent thread.
=======
    /// Send rich user input items to an existing agent thread.
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
    pub(crate) async fn send_input(
        &self,
        agent_id: ThreadId,
        sender: Option<ThreadId>,
        input: Vec<UserInput>,
        start_options: TurnStartOptions,
    ) -> CodexResult<String> {
<<<<<<< HEAD
        let state = self.upgrade()?;
        self.ensure_execution_capacity_for_turn_start(agent_id, /*starts_turn*/ true)
            .await?;
        self.send_input_after_capacity_check(agent_id, sender, &state, input, parent_turn_id)
||||||| 413492cd6c
        let state = self.upgrade()?;
        self.ensure_execution_capacity_for_turn_start(agent_id, /*starts_turn*/ true)
            .await?;
        self.send_input_after_capacity_check(agent_id, &state, input, parent_turn_id)
=======
        let state = self.runtime.upgrade()?;
        let thread = state.get_thread(agent_id).await?;
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
        self.handle_thread_request_result(agent_id, &state, result)
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
            .await
<<<<<<< HEAD
    }

    async fn send_input_after_capacity_check(
        &self,
        agent_id: ThreadId,
        sender: Option<ThreadId>,
        state: &Arc<ThreadManagerState>,
        input: Vec<UserInput>,
        parent_turn_id: Option<String>,
    ) -> CodexResult<String> {
        Box::pin(register_agent_input_origin(state, agent_id, sender, &input)).await;
        self.handle_thread_request_result(
            agent_id,
            state,
            state.send_op(agent_id, input.into(), parent_turn_id).await,
        )
        .await
||||||| 413492cd6c
    }

    async fn send_input_after_capacity_check(
        &self,
        agent_id: ThreadId,
        state: &Arc<ThreadManagerState>,
        input: Vec<UserInput>,
        parent_turn_id: Option<String>,
    ) -> CodexResult<String> {
        self.handle_thread_request_result(
            agent_id,
            state,
            state.send_op(agent_id, input.into(), parent_turn_id).await,
        )
        .await
=======
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
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
<<<<<<< HEAD
        _state: &Arc<ThreadManagerState>,
        communication: InterAgentCommunication,
||||||| 413492cd6c
        state: &Arc<ThreadManagerState>,
        communication: InterAgentCommunication,
        context: AgentCommunicationContext,
        parent_turn_id: Option<String>,
    ) -> CodexResult<String> {
        self.submit_inter_agent_communication(
            agent_id,
            state,
            communication,
            context,
            parent_turn_id,
        )
        .await
    }

    async fn submit_inter_agent_communication(
        &self,
        agent_id: ThreadId,
        state: &Arc<ThreadManagerState>,
        communication: InterAgentCommunication,
=======
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
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        context: AgentCommunicationContext,
        start_options: TurnStartOptions,
    ) -> CodexResult<String> {
<<<<<<< HEAD
        let communication_for_log =
            crate::agent_communication::logging_enabled().then(|| communication.clone());
        let result = self
            .send_persisted_inter_agent_communication(agent_id, communication, parent_turn_id)
            .await;
        if let (Some(communication), Ok(communication_id)) =
            (communication_for_log, result.as_ref())
||||||| 413492cd6c
        let communication_for_log =
            crate::agent_communication::logging_enabled().then(|| communication.clone());
        let parent_turn_id = parent_turn_id.filter(|_| communication.trigger_turn);
        let result = self
            .handle_thread_request_result(
                agent_id,
                state,
                state
                    .send_op(
                        agent_id,
                        Op::InterAgentCommunication { communication },
                        parent_turn_id,
                    )
                    .await,
            )
            .await;
        if let (Some(communication), Ok(communication_id)) =
            (communication_for_log, result.as_ref())
=======
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
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        {
            // Keep unloaded delivery atomic with publication of a reloaded session.
            let threads = state.threads.read().await;
            if !communication.trigger_turn && !threads.contains_key(&agent_id) {
                let _membership = self.runtime.admit_start()?;
                self.runtime.ensure_agent_known(agent_id)?;
                let submission_id = uuid::Uuid::now_v7().to_string();
                self.runtime.mailboxes.enqueue(
                    agent_id,
                    Some(submission_id.clone()),
                    vec![crate::PendingAgentMessage::new(communication, delivery)],
                )?;
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
        // Loaded recipients retain submission ordering with follow-ups and interrupts.
        let result = submit_loaded_agent_communication(
            agent_id,
            communication,
            context,
            log_content.as_deref(),
            start_options,
            |op, parent_turn_id, root_turn_id| {
                state.send_op(agent_id, op, parent_turn_id, root_turn_id, delivery)
            },
        )
        .await;
        self.handle_thread_request_result(agent_id, state, result)
            .await
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
            return if self.state.agent_metadata_for_thread(agent_id).is_some() {
                AgentStatus::Unloaded
            } else {
                AgentStatus::NotFound
            };
        };
        thread.agent_status().await
    }

    pub(crate) fn get_agent_metadata(&self, agent_id: ThreadId) -> Option<AgentMetadata> {
<<<<<<< HEAD
        self.state.agent_metadata_for_thread(agent_id)
    }

    pub(crate) fn get_agent_metadata_for_path(
        &self,
        agent_path: &AgentPath,
    ) -> Option<AgentMetadata> {
        self.state.agent_metadata_for_path(agent_path)
    }

    pub(crate) fn is_root_thread(&self, agent_id: ThreadId) -> bool {
        self.state.agent_id_for_path(&AgentPath::root()) == Some(agent_id)
    }

    /// Whether `thread_id` is loaded and is the root of its own tree: a thread a client
    /// started (such as a pane the person opened), not a spawned or internal agent.
    pub(crate) async fn is_loaded_root_thread(&self, thread_id: ThreadId) -> bool {
        let Ok(state) = self.upgrade() else {
            return false;
        };
        state
            .get_thread(thread_id)
            .await
            .is_ok_and(|thread| !thread.session_source.is_non_root_agent())
    }

    pub(crate) fn ensure_agent_known(&self, agent_id: ThreadId) -> CodexResult<AgentMetadata> {
        self.state
            .agent_metadata_for_thread(agent_id)
            .ok_or_else(|| CodexErr::ThreadNotFound(agent_id))
    }

    /// Whether native terminal mailbox items should wake this thread automatically.
    ///
    /// Persistent crew managers process child results without a TUI-side report queue.
    /// Human-facing roots and ephemeral task agents retain ordinary Codex behavior.
    pub(crate) async fn auto_processes_terminal_results(&self, agent_id: ThreadId) -> bool {
        let Ok(state) = self.upgrade() else {
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
                .state
                .update_last_result_message(agent_id, message.clone()),
            None => self.state.clear_last_result_message(agent_id),
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
        self.state.register_spawned_thread(AgentMetadata {
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
        self.state.restore_spawned_thread(AgentMetadata {
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
        if self.security_policy.snapshot_for_agent(thread_id).is_ok() {
            return;
        }
        let SessionSource::SubAgent(SubAgentSource::ThreadSpawn {
            parent_thread_id, ..
        }) = session_source
        else {
            return;
        };
        let Ok(parent) = self.security_policy.snapshot_for_agent(*parent_thread_id) else {
            warn!(
                %thread_id,
                %parent_thread_id,
                "cannot restore child security binding without its parent binding"
            );
            return;
        };
        if let Err(error) = self.security_policy.inherit_child(
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
        let state = self.upgrade()?;
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

    pub(crate) async fn list_live_agent_subtree_thread_ids(
        &self,
        agent_id: ThreadId,
    ) -> CodexResult<Vec<ThreadId>> {
        let mut thread_ids = vec![agent_id];
        thread_ids.extend(self.live_thread_spawn_descendants(agent_id).await?);
        Ok(thread_ids)
    }

    pub(crate) async fn get_agent_config_snapshot(
        &self,
        agent_id: ThreadId,
    ) -> Option<ThreadConfigSnapshot> {
        let Ok(state) = self.upgrade() else {
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
        let state = self.upgrade().ok()?;
        let thread = state.get_thread(thread_id).await.ok()?;
        Some(Arc::clone(&thread.session))
    }

    pub(crate) async fn resolve_agent_reference(
        &self,
        _current_thread_id: ThreadId,
        current_session_source: &SessionSource,
        agent_reference: &str,
    ) -> CodexResult<ThreadId> {
        let agent_reference = agent_reference.trim();
        if let Ok(thread_id) = ThreadId::from_string(agent_reference)
            && self.state.agent_metadata_for_thread(thread_id).is_some()
        {
            return Ok(thread_id);
        }
        if let Some(thread_id) = self.state.agent_id_for_nickname(agent_reference) {
            return Ok(thread_id);
        }
        if let Some(agent_nickname) = nickname_from_picker_label(agent_reference)
            && let Some(thread_id) = self.state.agent_id_for_nickname(agent_nickname)
        {
            return Ok(thread_id);
        }
        let current_agent_path = current_session_source
            .get_agent_path()
            .unwrap_or_else(AgentPath::root);
        let agent_path = current_agent_path
            .resolve(agent_reference)
            .map_err(CodexErr::UnsupportedOperation)?;
        if let Some(thread_id) = self.state.agent_id_for_path(&agent_path) {
            return Ok(thread_id);
        }
        Err(CodexErr::UnsupportedOperation(format!(
            "live agent path `{}` not found",
            agent_path.as_str()
        )))
    }

    /// Subscribe to status updates for `agent_id`, yielding the latest value and changes.
    pub(crate) async fn subscribe_status(
        &self,
        agent_id: ThreadId,
    ) -> CodexResult<watch::Receiver<AgentStatus>> {
        let state = self.upgrade()?;
        let thread = state.get_thread(agent_id).await?;
        Ok(thread.subscribe_status())
    }

    pub(crate) async fn format_environment_context_subagents(
        &self,
        parent_thread_id: ThreadId,
    ) -> String {
        let Ok(agents) = self.open_thread_spawn_children(parent_thread_id).await else {
            return String::new();
        };

        let state = self.upgrade().ok();
        let mut lines = Vec::new();
        for (thread_id, metadata) in agents {
            let status = match state.as_ref() {
                Some(state) => match state.get_thread(thread_id).await {
                    Ok(thread) => Some(thread.agent_status().await),
                    Err(_) => None,
                },
                None => None,
            };
            let last_result_message = metadata
                .last_result_message
                .clone()
                .or_else(|| status.as_ref().and_then(result_message_from_status));
            let reference = metadata
                .agent_path
                .as_ref()
                .map(|agent_path| agent_path.name().to_string())
                .unwrap_or_else(|| thread_id.to_string());
            lines.push(format_subagent_context_line(
                reference.as_str(),
                metadata.agent_nickname.as_deref(),
                metadata.agent_role.as_deref(),
                status.as_ref(),
                metadata.last_task_message.as_deref(),
                last_result_message.as_deref(),
            ));
        }

        lines
            .into_iter()
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
||||||| 413492cd6c
        self.state.agent_metadata_for_thread(agent_id)
    }

    pub(crate) fn ensure_agent_known(&self, agent_id: ThreadId) -> CodexResult<AgentMetadata> {
        self.state
            .agent_metadata_for_thread(agent_id)
            .ok_or_else(|| CodexErr::ThreadNotFound(agent_id))
    }

    pub(crate) async fn list_live_agent_subtree_thread_ids(
        &self,
        agent_id: ThreadId,
    ) -> CodexResult<Vec<ThreadId>> {
        let mut thread_ids = vec![agent_id];
        thread_ids.extend(self.live_thread_spawn_descendants(agent_id).await?);
        Ok(thread_ids)
    }

    pub(crate) async fn get_agent_config_snapshot(
        &self,
        agent_id: ThreadId,
    ) -> Option<ThreadConfigSnapshot> {
        let Ok(state) = self.upgrade() else {
            return None;
        };
        let Ok(thread) = state.get_thread(agent_id).await else {
            return None;
        };
        Some(thread.config_snapshot().await)
    }

    pub(crate) async fn resolve_agent_reference(
        &self,
        _current_thread_id: ThreadId,
        current_session_source: &SessionSource,
        agent_reference: &str,
    ) -> CodexResult<ThreadId> {
        let current_agent_path = current_session_source
            .get_agent_path()
            .unwrap_or_else(AgentPath::root);
        let agent_path = current_agent_path
            .resolve(agent_reference)
            .map_err(CodexErr::UnsupportedOperation)?;
        if let Some(thread_id) = self.state.agent_id_for_path(&agent_path) {
            return Ok(thread_id);
        }
        Err(CodexErr::UnsupportedOperation(format!(
            "live agent path `{}` not found",
            agent_path.as_str()
        )))
    }

    /// Subscribe to status updates for `agent_id`, yielding the latest value and changes.
    pub(crate) async fn subscribe_status(
        &self,
        agent_id: ThreadId,
    ) -> CodexResult<watch::Receiver<AgentStatus>> {
        let state = self.upgrade()?;
        let thread = state.get_thread(agent_id).await?;
        Ok(thread.subscribe_status())
    }

    pub(crate) async fn format_environment_context_subagents(
        &self,
        parent_thread_id: ThreadId,
    ) -> String {
        let Ok(agents) = self.open_thread_spawn_children(parent_thread_id).await else {
            return String::new();
        };

        agents
            .into_iter()
            .map(|(thread_id, metadata)| {
                let reference = metadata
                    .agent_path
                    .as_ref()
                    .map(|agent_path| agent_path.name().to_string())
                    .unwrap_or_else(|| thread_id.to_string());
                format_subagent_context_line(reference.as_str(), metadata.agent_nickname.as_deref())
            })
            .collect::<Vec<_>>()
            .join("\n")
=======
        self.runtime.registry.agent_metadata_for_thread(agent_id)
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
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
<<<<<<< HEAD
            agents.push(ListedAgent {
                agent_name: root_path.to_string(),
                agent_nickname: None,
                agent_role: None,
                agent_status: root_thread.agent_status().await,
                last_task_message: Some(ROOT_LAST_TASK_MESSAGE.to_string()),
                last_result_message: None,
||||||| 413492cd6c
            agents.push(ListedAgent {
                agent_name: root_path.to_string(),
                agent_status: root_thread.agent_status().await,
=======
            agents.push(LiveAgent {
                thread_id: root_thread_id,
                metadata: AgentMetadata {
                    agent_id: Some(root_thread_id),
                    agent_path: Some(root_path),
                    ..Default::default()
                },
                status: root_thread.agent_status().await,
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
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

<<<<<<< HEAD
            let agent_name = metadata
                .agent_path
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_else(|| thread_id.to_string());
            let agent_status = match state.get_thread(thread_id).await {
                Ok(thread) => thread.agent_status().await,
                Err(_) => AgentStatus::Unloaded,
            };
            let last_result_message = metadata
                .last_result_message
                .clone()
                .or_else(|| result_message_from_status(&agent_status));
            agents.push(ListedAgent {
                agent_name,
                agent_nickname: metadata.agent_nickname.clone(),
                agent_role: metadata.agent_role.clone(),
                agent_status,
                last_task_message: metadata.last_task_message.clone(),
                last_result_message,
||||||| 413492cd6c
            let Ok(thread) = state.get_thread(thread_id).await else {
                continue;
            };
            let agent_name = metadata
                .agent_path
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_else(|| thread_id.to_string());
            agents.push(ListedAgent {
                agent_name,
                agent_status: thread.agent_status().await,
=======
            let Ok(thread) = state.get_thread(thread_id).await else {
                continue;
            };
            agents.push(LiveAgent {
                thread_id,
                metadata,
                status: thread.agent_status().await,
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
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
<<<<<<< HEAD
                .inject_agent_result_without_turn(message)
||||||| 413492cd6c
                .inject_user_message_without_turn(message)
=======
                .inject_fragment_without_turn(
                    SubagentNotification::new(child_reference.as_str(), status)
                        .with_metadata(ContentItemMetadata::tool(namespace)),
                )
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
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
<<<<<<< HEAD
        let Some(SessionSource::SubAgent(SubAgentSource::ThreadSpawn {
            parent_thread_id, ..
        })) = session_source
        else {
            return None;
        };

        let parent_thread = state.get_thread(*parent_thread_id).await.ok()?;
        Some(
            parent_thread
                .session
                .services
                .turn_environments
                .snapshot()
                .await,
        )
    }

    async fn inherited_exec_policy_for_source(
        &self,
        state: &Arc<ThreadManagerState>,
        session_source: Option<&SessionSource>,
        child_config: &Config,
    ) -> Option<Arc<crate::exec_policy::ExecPolicyManager>> {
        let Some(SessionSource::SubAgent(SubAgentSource::ThreadSpawn {
            parent_thread_id, ..
        })) = session_source
        else {
            return None;
        };

        let parent_thread = state.get_thread(*parent_thread_id).await.ok()?;
        let parent_config = parent_thread.session.get_config().await;
        if !crate::exec_policy::child_uses_parent_exec_policy(&parent_config, child_config) {
            return None;
        }

        Some(Arc::clone(&parent_thread.session.services.exec_policy))
    }

    async fn open_thread_spawn_children(
        &self,
        parent_thread_id: ThreadId,
    ) -> CodexResult<Vec<(ThreadId, AgentMetadata)>> {
        let mut children_by_parent = self.live_thread_spawn_children().await?;
        Ok(children_by_parent
            .remove(&parent_thread_id)
            .unwrap_or_default())
    }

    async fn live_thread_spawn_children(
        &self,
    ) -> CodexResult<HashMap<ThreadId, Vec<(ThreadId, AgentMetadata)>>> {
        let state = self.upgrade()?;
        let mut children_by_parent = HashMap::<ThreadId, Vec<(ThreadId, AgentMetadata)>>::new();

        for (parent_thread_id, child_thread_id) in state.list_live_thread_spawn_edges().await {
            let metadata = match self.state.agent_metadata_for_thread(child_thread_id) {
                Some(metadata) => metadata,
                None => match state.get_thread(child_thread_id).await {
                    Ok(child_thread) => AgentMetadata {
                        agent_id: Some(child_thread_id),
                        agent_path: child_thread.session_source.get_agent_path(),
                        agent_nickname: child_thread.session_source.get_nickname(),
                        agent_role: child_thread.session_source.get_agent_role(),
                        last_task_message: None,
                        last_result_message: None,
                    },
                    Err(_) => AgentMetadata {
                        agent_id: Some(child_thread_id),
                        ..Default::default()
                    },
                },
            };
            children_by_parent
                .entry(parent_thread_id)
                .or_default()
                .push((child_thread_id, metadata));
        }

        for children in children_by_parent.values_mut() {
            children.sort_by(|left, right| {
                left.1
                    .agent_path
                    .as_deref()
                    .unwrap_or_default()
                    .cmp(right.1.agent_path.as_deref().unwrap_or_default())
                    .then_with(|| left.0.to_string().cmp(&right.0.to_string()))
            });
        }

        Ok(children_by_parent)
||||||| 413492cd6c
        let Some(SessionSource::SubAgent(SubAgentSource::ThreadSpawn {
            parent_thread_id, ..
        })) = session_source
        else {
            return None;
        };

        let parent_thread = state.get_thread(*parent_thread_id).await.ok()?;
        Some(
            parent_thread
                .session
                .services
                .turn_environments
                .snapshot()
                .await,
        )
    }

    async fn inherited_exec_policy_for_source(
        &self,
        state: &Arc<ThreadManagerState>,
        session_source: Option<&SessionSource>,
        child_config: &Config,
    ) -> Option<Arc<crate::exec_policy::ExecPolicyManager>> {
        let Some(SessionSource::SubAgent(SubAgentSource::ThreadSpawn {
            parent_thread_id, ..
        })) = session_source
        else {
            return None;
        };

        let parent_thread = state.get_thread(*parent_thread_id).await.ok()?;
        let parent_config = parent_thread.session.get_config().await;
        if !crate::exec_policy::child_uses_parent_exec_policy(&parent_config, child_config) {
            return None;
        }

        Some(Arc::clone(&parent_thread.session.services.exec_policy))
    }

    async fn open_thread_spawn_children(
        &self,
        parent_thread_id: ThreadId,
    ) -> CodexResult<Vec<(ThreadId, AgentMetadata)>> {
        let mut children_by_parent = self.live_thread_spawn_children().await?;
        Ok(children_by_parent
            .remove(&parent_thread_id)
            .unwrap_or_default())
    }

    async fn live_thread_spawn_children(
        &self,
    ) -> CodexResult<HashMap<ThreadId, Vec<(ThreadId, AgentMetadata)>>> {
        let state = self.upgrade()?;
        let mut children_by_parent = HashMap::<ThreadId, Vec<(ThreadId, AgentMetadata)>>::new();

        for (parent_thread_id, child_thread_id) in state.list_live_thread_spawn_edges().await {
            children_by_parent
                .entry(parent_thread_id)
                .or_default()
                .push((
                    child_thread_id,
                    self.state
                        .agent_metadata_for_thread(child_thread_id)
                        .unwrap_or(AgentMetadata {
                            agent_id: Some(child_thread_id),
                            ..Default::default()
                        }),
                ));
        }

        for children in children_by_parent.values_mut() {
            children.sort_by(|left, right| {
                left.1
                    .agent_path
                    .as_deref()
                    .unwrap_or_default()
                    .cmp(right.1.agent_path.as_deref().unwrap_or_default())
                    .then_with(|| left.0.to_string().cmp(&right.0.to_string()))
            });
        }

        Ok(children_by_parent)
=======
        let parent_thread_id = session_source?.parent_thread_id()?;
        let parent_thread = state.get_thread(parent_thread_id).await.ok()?;
        Some(parent_thread.session.inherited_environment_snapshot().await)
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
    }

<<<<<<< HEAD
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
        let state = self.upgrade()?;
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
        self.persist_thread_spawn_edge_for_source(child_thread, child_thread_id, session_source)
            .await
    }

||||||| 413492cd6c
=======
    /// Returns true after publication, or false when the source or configuration skips it.
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
    async fn persist_thread_spawn_edge_for_source(
        &self,
        ephemeral: bool,
        child_thread_id: ThreadId,
        session_source: Option<&SessionSource>,
<<<<<<< HEAD
    ) -> CodexResult<()> {
||||||| 413492cd6c
    ) {
=======
    ) -> CodexResult<bool> {
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        let Some(parent_thread_id) = session_source.and_then(SessionSource::parent_thread_id)
        else {
<<<<<<< HEAD
            return Ok(());
||||||| 413492cd6c
            return;
=======
            return Ok(false);
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        };
<<<<<<< HEAD
        if child_thread.config_snapshot().await.ephemeral {
            return Ok(());
||||||| 413492cd6c
        if child_thread.config_snapshot().await.ephemeral {
            return;
=======
        if ephemeral {
            return Ok(false);
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        }
<<<<<<< HEAD
        let state = self.upgrade()?;
||||||| 413492cd6c
        let Ok(state) = self.upgrade() else {
            return;
        };
=======
        let state = self.runtime.upgrade()?;
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        let Some(agent_graph_store) = state.agent_graph_store() else {
<<<<<<< HEAD
            return Ok(());
||||||| 413492cd6c
            return;
=======
            return Ok(false);
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        };
        agent_graph_store
            .upsert_thread_spawn_edge(
                parent_thread_id,
                child_thread_id,
                codex_agent_graph_store::ThreadSpawnEdgeStatus::Open,
            )
            .await
<<<<<<< HEAD
            .map_err(|err| {
                CodexErr::Fatal(format!(
                    "failed to persist native agent edge {parent_thread_id} -> {child_thread_id}: {err}"
                ))
            })
||||||| 413492cd6c
        {
            warn!("failed to persist thread-spawn edge: {err}");
        }
=======
        {
            warn!("failed to persist thread-spawn edge: {err}");
            return Err(CodexErr::Fatal(
                "failed to persist spawned agent relationship".to_string(),
            ));
        }
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
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
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

<<<<<<< HEAD
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

||||||| 413492cd6c
=======
/// Share loaded-recipient submission while callers retain residency and failure handling.
async fn submit_loaded_agent_communication<F>(
    agent_id: ThreadId,
    communication: InterAgentCommunication,
    context: AgentCommunicationContext,
    log_content: Option<&str>,
    start_options: TurnStartOptions,
    submit: impl FnOnce(Op, Option<String>, Option<String>) -> F,
) -> CodexResult<String>
where
    F: std::future::Future<Output = CodexResult<String>>,
{
    let (parent_turn_id, root_turn_id) = if communication.trigger_turn {
        (
            start_options.parent_turn_id.clone(),
            start_options.root_turn_id.clone(),
        )
    } else {
        (None, None)
    };
    let submission_id = submit(
        Op::InterAgentCommunication {
            communication,
            start_options,
        },
        parent_turn_id,
        root_turn_id,
    )
    .await?;
    crate::agent_communication::emit_agent_communication_send(
        &submission_id,
        &context,
        log_content,
        agent_id,
    );
    Ok(submission_id)
}
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
#[cfg(test)]
#[path = "control_tests.rs"]
mod tests;
