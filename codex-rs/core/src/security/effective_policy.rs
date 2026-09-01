#![cfg_attr(not(test), allow(dead_code))]
// PF-22 defines the Core policy boundary before PF-23/PF-24 connect every
// protected-surface adapter and the trusted TUI controller.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;

use codex_protocol::SessionId;
use codex_protocol::ThreadId;
use codex_security_policy::ActorChain;
use codex_security_policy::BoundedText;
use codex_security_policy::PolicyPrincipal;
use codex_security_policy::PrincipalKind;
use codex_security_policy::RevocationState;
use codex_security_policy::SecurityLevel;
use codex_security_policy::SecuritySettings;
use thiserror::Error;
use uuid::Uuid;

#[path = "trusted_requests.rs"]
mod trusted_requests;

/// Human-owned persisted inputs from which Core may build an effective policy.
///
/// Model output, project files, hooks, plugins, connectors, and MCP results are
/// intentionally absent from this type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PersistedHumanSecurityState {
    pub(crate) settings: SecuritySettings,
    pub(crate) human_authority: PolicyPrincipal,
    pub(crate) revocations: RevocationState,
}

impl PersistedHumanSecurityState {
    pub(crate) fn new(
        settings: SecuritySettings,
        human_authority: PolicyPrincipal,
        revocations: RevocationState,
    ) -> Result<Self, SecurityPolicyError> {
        let state = Self {
            settings,
            human_authority,
            revocations,
        };
        state.validate()?;
        Ok(state)
    }

    fn validate(&self) -> Result<(), SecurityPolicyError> {
        self.settings
            .validate()
            .map_err(|error| SecurityPolicyError::CorruptPersistedState(error.to_string()))?;
        if self.human_authority.kind != PrincipalKind::Human {
            return Err(SecurityPolicyError::HumanAuthorityRequired);
        }
        self.revocations
            .validate()
            .map_err(|error| SecurityPolicyError::CorruptPersistedState(error.to_string()))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct AgentSecurityBinding {
    actor_chain: ActorChain,
    session_id: BoundedText,
    task_id: BoundedText,
    minimum_level: SecurityLevel,
    force_deny: bool,
}

#[derive(Clone, Debug)]
struct EffectivePolicyState {
    persisted: PersistedHumanSecurityState,
    runtime_nonce: [u8; 16],
    epoch: u64,
    root_agent_id: ThreadId,
    agents: HashMap<ThreadId, AgentSecurityBinding>,
}

#[derive(Default)]
struct SharedEffectivePolicy {
    state: RwLock<Option<EffectivePolicyState>>,
}

/// Read/inheritance capability shared with agent runtimes.
///
/// It can evaluate current policy and derive child identity, but exposes no
/// security-level mutation operation.
#[derive(Clone, Default)]
pub(crate) struct EffectivePolicyView {
    shared: Arc<SharedEffectivePolicy>,
}

/// Trusted control capability retained outside model/tool routing.
///
/// A human-facing controller first creates a bound confirmation and then applies
/// it. Agent runtimes receive only [`EffectivePolicyView`].
#[derive(Clone)]
pub(crate) struct TrustedSecurityController {
    shared: Arc<SharedEffectivePolicy>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EffectivePolicyInitialization {
    Root,
    DetachedSpawnedAgent,
}

#[derive(Clone, Debug)]
pub(crate) struct ConfirmedSecurityLevelChange {
    expected_epoch: u64,
    expected_runtime_nonce: [u8; 16],
    authority_id: BoundedText,
    next: PersistedHumanSecurityState,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct EffectivePolicySnapshot {
    pub(crate) runtime_nonce: [u8; 16],
    pub(crate) epoch: u64,
    pub(crate) requested_level: SecurityLevel,
    pub(crate) creator_required_level: SecurityLevel,
    pub(crate) level: SecurityLevel,
    pub(crate) actor_chain: ActorChain,
    pub(crate) session_id: BoundedText,
    pub(crate) task_id: BoundedText,
    pub(crate) revocation_generation: u64,
    pub(crate) authority_kill_switch_active: bool,
    pub(crate) kill_switch_active: bool,
}

impl EffectivePolicySnapshot {
    /// Compose a profile restriction with the existing product policy.
    ///
    /// Permissive is allow-neutral. Moderate and Aggressive can only narrow an
    /// existing allow, and the kill switch always denies.
    pub(crate) fn compose_existing_decision(
        &self,
        existing_allow: bool,
        profile_allows: bool,
    ) -> bool {
        if self.kill_switch_active {
            return false;
        }
        match self.level {
            SecurityLevel::Permissive => existing_allow,
            SecurityLevel::Moderate | SecurityLevel::Aggressive => existing_allow && profile_allows,
        }
    }
}

impl EffectivePolicyView {
    pub(crate) fn is_initialized(&self) -> Result<bool, SecurityPolicyError> {
        Ok(self.read_state()?.is_some())
    }

    pub(crate) fn snapshot_for_agent(
        &self,
        agent_id: ThreadId,
    ) -> Result<EffectivePolicySnapshot, SecurityPolicyError> {
        let state = self.read_state()?;
        let state = state
            .as_ref()
            .ok_or(SecurityPolicyError::RuntimeNotInitialized)?;
        let binding = state
            .agents
            .get(&agent_id)
            .ok_or(SecurityPolicyError::UnknownAgent(agent_id))?;
        Ok(snapshot(state, binding))
    }

    pub(crate) fn snapshot_for_turn(
        &self,
        agent_id: ThreadId,
        task_id: impl Into<String>,
    ) -> Result<EffectivePolicySnapshot, SecurityPolicyError> {
        let mut snapshot = self.snapshot_for_agent(agent_id)?;
        snapshot.task_id = BoundedText::new(task_id.into())?;
        Ok(snapshot)
    }

    pub(crate) fn inherit_auxiliary_agent(
        &self,
        agent_id: ThreadId,
        task_id: impl Into<String>,
        configured_level: SecurityLevel,
    ) -> Result<EffectivePolicySnapshot, SecurityPolicyError> {
        let root_agent_id = {
            let state = self.read_state()?;
            state
                .as_ref()
                .ok_or(SecurityPolicyError::RuntimeNotInitialized)?
                .root_agent_id
        };
        self.inherit_child(root_agent_id, agent_id, task_id, configured_level)
    }

    pub(crate) fn inherit_child(
        &self,
        parent_id: ThreadId,
        child_id: ThreadId,
        task_id: impl Into<String>,
        configured_child_level: SecurityLevel,
    ) -> Result<EffectivePolicySnapshot, SecurityPolicyError> {
        let mut state_guard = self.write_state()?;
        let state = state_guard
            .as_mut()
            .ok_or(SecurityPolicyError::RuntimeNotInitialized)?;
        let parent = state
            .agents
            .get(&parent_id)
            .cloned()
            .ok_or(SecurityPolicyError::UnknownAgent(parent_id))?;
        let child_actor = PolicyPrincipal::new(PrincipalKind::Agent, format!("agent:{child_id}"))?;
        let mut actors = parent.actor_chain.as_slice().to_vec();
        actors.push(child_actor);
        let configured_minimum = if configured_child_level > state.persisted.settings.level {
            configured_child_level
        } else {
            SecurityLevel::Permissive
        };
        let binding = AgentSecurityBinding {
            actor_chain: ActorChain::new(actors)?,
            session_id: parent.session_id,
            task_id: BoundedText::new(task_id.into())?,
            minimum_level: parent.minimum_level.max(configured_minimum),
            force_deny: parent.force_deny,
        };
        if let Some(existing) = state.agents.get(&child_id) {
            if existing != &binding {
                return Err(SecurityPolicyError::ConflictingAgentBinding(child_id));
            }
        } else {
            state.agents.insert(child_id, binding.clone());
        }
        Ok(snapshot(state, &binding))
    }

    /// Untrusted text is data, never a security-policy command. The text is not
    /// parsed, stored, logged, or echoed from this boundary.
    pub(crate) fn reject_untrusted_policy_mutation(
        &self,
        origin: UntrustedPolicyOrigin,
        _untrusted_text: &str,
    ) -> Result<(), SecurityPolicyError> {
        Err(SecurityPolicyError::UntrustedMutationOrigin(origin))
    }

    fn read_state(
        &self,
    ) -> Result<std::sync::RwLockReadGuard<'_, Option<EffectivePolicyState>>, SecurityPolicyError>
    {
        self.shared
            .state
            .read()
            .map_err(|_| SecurityPolicyError::RuntimePoisoned)
    }

    fn write_state(
        &self,
    ) -> Result<std::sync::RwLockWriteGuard<'_, Option<EffectivePolicyState>>, SecurityPolicyError>
    {
        self.shared
            .state
            .write()
            .map_err(|_| SecurityPolicyError::RuntimePoisoned)
    }
}

impl TrustedSecurityController {
    pub(crate) fn initialize(
        view: &EffectivePolicyView,
        persisted: PersistedHumanSecurityState,
        root_agent_id: ThreadId,
        session_id: SessionId,
        initialization: EffectivePolicyInitialization,
    ) -> Result<Self, SecurityPolicyError> {
        persisted.validate()?;
        let root_agent =
            PolicyPrincipal::new(PrincipalKind::Agent, format!("agent:{root_agent_id}"))?;
        let root_binding = AgentSecurityBinding {
            actor_chain: ActorChain::new(vec![persisted.human_authority.clone(), root_agent])?,
            session_id: BoundedText::new(format!("session:{session_id}"))?,
            task_id: BoundedText::new(format!("task:{root_agent_id}"))?,
            minimum_level: match initialization {
                EffectivePolicyInitialization::Root => SecurityLevel::Permissive,
                EffectivePolicyInitialization::DetachedSpawnedAgent => SecurityLevel::Aggressive,
            },
            force_deny: initialization == EffectivePolicyInitialization::DetachedSpawnedAgent,
        };
        let proposed = EffectivePolicyState {
            persisted,
            runtime_nonce: *Uuid::new_v4().as_bytes(),
            epoch: 0,
            root_agent_id,
            agents: HashMap::from([(root_agent_id, root_binding)]),
        };
        let mut guard = view
            .shared
            .state
            .write()
            .map_err(|_| SecurityPolicyError::RuntimePoisoned)?;
        match guard.as_ref() {
            Some(existing) if !same_initial_state(existing, &proposed) => {
                return Err(SecurityPolicyError::RuntimeAlreadyInitialized);
            }
            Some(_) => {}
            None => *guard = Some(proposed),
        }
        Ok(Self {
            shared: Arc::clone(&view.shared),
        })
    }

    pub(crate) fn confirm_level_change(
        &self,
        next_level: SecurityLevel,
        next_revocations: RevocationState,
    ) -> Result<ConfirmedSecurityLevelChange, SecurityPolicyError> {
        next_revocations
            .validate()
            .map_err(|error| SecurityPolicyError::CorruptPersistedState(error.to_string()))?;
        let state_guard = self.read_state()?;
        let state = state_guard
            .as_ref()
            .ok_or(SecurityPolicyError::RuntimeNotInitialized)?;
        let next = PersistedHumanSecurityState::new(
            SecuritySettings::new(next_level),
            state.persisted.human_authority.clone(),
            next_revocations,
        )?;
        Ok(ConfirmedSecurityLevelChange {
            expected_epoch: state.epoch,
            expected_runtime_nonce: state.runtime_nonce,
            authority_id: state.persisted.human_authority.id.clone(),
            next,
        })
    }

    /// Validate the complete replacement before taking the write lock, then
    /// replace the state in one critical section. Readers see either snapshot.
    pub(crate) fn apply_confirmed_change(
        &self,
        confirmation: ConfirmedSecurityLevelChange,
    ) -> Result<u64, SecurityPolicyError> {
        confirmation.next.validate()?;
        let mut state_guard = self.write_state()?;
        let state = state_guard
            .as_mut()
            .ok_or(SecurityPolicyError::RuntimeNotInitialized)?;
        if state.epoch != confirmation.expected_epoch {
            return Err(SecurityPolicyError::StaleConfirmation {
                expected: confirmation.expected_epoch,
                actual: state.epoch,
            });
        }
        if state.runtime_nonce != confirmation.expected_runtime_nonce
            || state.persisted.human_authority.id != confirmation.authority_id
            || state.persisted.human_authority != confirmation.next.human_authority
        {
            return Err(SecurityPolicyError::AuthorityMismatch);
        }
        if state.persisted == confirmation.next {
            return Ok(state.epoch);
        }
        let next_epoch = state
            .epoch
            .checked_add(1)
            .ok_or(SecurityPolicyError::EpochOverflow)?;
        state.persisted = confirmation.next;
        state.epoch = next_epoch;
        Ok(next_epoch)
    }

    fn read_state(
        &self,
    ) -> Result<std::sync::RwLockReadGuard<'_, Option<EffectivePolicyState>>, SecurityPolicyError>
    {
        self.shared
            .state
            .read()
            .map_err(|_| SecurityPolicyError::RuntimePoisoned)
    }

    fn write_state(
        &self,
    ) -> Result<std::sync::RwLockWriteGuard<'_, Option<EffectivePolicyState>>, SecurityPolicyError>
    {
        self.shared
            .state
            .write()
            .map_err(|_| SecurityPolicyError::RuntimePoisoned)
    }
}

fn snapshot(
    state: &EffectivePolicyState,
    binding: &AgentSecurityBinding,
) -> EffectivePolicySnapshot {
    EffectivePolicySnapshot {
        runtime_nonce: state.runtime_nonce,
        epoch: state.epoch,
        requested_level: state.persisted.settings.level,
        creator_required_level: binding.minimum_level,
        level: state.persisted.settings.level.max(binding.minimum_level),
        actor_chain: binding.actor_chain.clone(),
        session_id: binding.session_id.clone(),
        task_id: binding.task_id.clone(),
        revocation_generation: state.persisted.revocations.generation,
        authority_kill_switch_active: state.persisted.revocations.kill_switch_active,
        kill_switch_active: binding.force_deny || state.persisted.revocations.kill_switch_active,
    }
}

fn same_initial_state(left: &EffectivePolicyState, right: &EffectivePolicyState) -> bool {
    left.epoch == 0 && left.persisted == right.persisted && left.agents == right.agents
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum UntrustedPolicyOrigin {
    ModelOutput,
    ProjectContent,
    ToolOutput,
    Hook,
    Plugin,
    Connector,
    McpServer,
}

#[derive(Debug, Error)]
pub(crate) enum SecurityPolicyError {
    #[error("effective security policy has not been initialized")]
    RuntimeNotInitialized,
    #[error("effective security policy was already initialized with different trusted state")]
    RuntimeAlreadyInitialized,
    #[error("effective security policy state lock is poisoned")]
    RuntimePoisoned,
    #[error("persisted security state is corrupt: {0}")]
    CorruptPersistedState(String),
    #[error("security authority must be a human principal")]
    HumanAuthorityRequired,
    #[error("security policy has no binding for agent {0}")]
    UnknownAgent(ThreadId),
    #[error("agent {0} already has a different security binding")]
    ConflictingAgentBinding(ThreadId),
    #[error("security confirmation was issued by a different human authority")]
    AuthorityMismatch,
    #[error("security confirmation is stale: expected epoch {expected}, current epoch {actual}")]
    StaleConfirmation { expected: u64, actual: u64 },
    #[error("security policy epoch overflowed")]
    EpochOverflow,
    #[error("untrusted {0:?} content cannot mutate the security policy")]
    UntrustedMutationOrigin(UntrustedPolicyOrigin),
    #[error(transparent)]
    BoundedText(#[from] codex_security_policy::BoundedTextError),
    #[error(transparent)]
    Authorization(#[from] codex_security_policy::AuthorizationError),
}
