//! Fail-closed broker lifecycle and typed credential dispatch.
//!
//! This module owns no generic `resolve -> String` API. A trusted backend runs
//! beside the broker and receives only a typed operation plus an opaque
//! credential reference. The production backend is responsible for resolving
//! the raw value and completing the transport without returning that value over
//! IPC.

use crate::ipc::BrokerBinding;
use crate::ipc::BrokerChannelMac;
use crate::ipc::BrokerOperation;
use crate::ipc::CredentialReference;
use crate::ipc::ObservedPeer;
use crate::ipc::SignedBrokerFrame;
use crate::platform_contract::PlatformReport;
use crate::platform_contract::ProtectedModeAuthorization;
use crate::platform_contract::validate_protected_mode_report;
use crate::resolver_types::*;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Mutex;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

pub struct BrokerRuntime<B, A> {
    broker_instance: String,
    config: BrokerRuntimeConfig,
    backend: B,
    audit: A,
    state: Mutex<BrokerState>,
    _platform_authorization: ProtectedModeAuthorization,
}

struct BrokerState {
    next_session_slot: u64,
    next_operation_id: u64,
    sessions: HashMap<u64, SessionState>,
    runs: HashMap<RunKey, u64>,
    in_flight: HashMap<u64, InFlightOperation>,
}

struct SessionState {
    binding: BrokerBinding,
    peer: ObservedPeer,
    channel_mac: BrokerChannelMac,
    credential_grants: HashMap<CredentialReference, i64>,
    next_sequence: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct RunKey {
    controller_instance: String,
    run_id: String,
}

struct InFlightOperation {
    session_slot: u64,
    credential: CredentialReference,
    fence: CancellationFence,
}

impl<B, A> BrokerRuntime<B, A>
where
    B: TypedCredentialBackend,
    A: DurableBrokerAudit,
{
    /// Constructs an eligible runtime only with PF-27-S03's unforgeable witness.
    pub fn new(
        broker_instance: impl Into<String>,
        config: BrokerRuntimeConfig,
        platform_authorization: ProtectedModeAuthorization,
        backend: B,
        audit: A,
    ) -> Result<Self, BrokerDispatchError> {
        let broker_instance = broker_instance.into();
        validate_broker_instance(&broker_instance)?;
        Ok(Self {
            broker_instance,
            config,
            backend,
            audit,
            state: Mutex::new(BrokerState {
                next_session_slot: 1,
                next_operation_id: 1,
                sessions: HashMap::new(),
                runs: HashMap::new(),
                in_flight: HashMap::new(),
            }),
            _platform_authorization: platform_authorization,
        })
    }

    /// Validates current platform evidence before constructing a runtime.
    /// Unsupported, stale, wrong-target, or malformed evidence remains a typed
    /// unavailable result and cannot fall back to a same-user broker.
    #[allow(clippy::too_many_arguments)]
    pub fn from_platform_report(
        broker_instance: impl Into<String>,
        config: BrokerRuntimeConfig,
        report: &PlatformReport<'_>,
        expected_target_id: &str,
        expected_probe_sha256: &str,
        now_unix_seconds: u64,
        backend: B,
        audit: A,
    ) -> Result<Self, BrokerDispatchError> {
        let authorization = validate_protected_mode_report(
            report,
            expected_target_id,
            expected_probe_sha256,
            now_unix_seconds,
        )
        .map_err(|_| BrokerDispatchError::PlatformUnavailable)?;
        Self::new(broker_instance, config, authorization, backend, audit)
    }

    pub fn register_session(
        &self,
        binding: BrokerBinding,
        peer: ObservedPeer,
        channel_mac: BrokerChannelMac,
        credential_grants: Vec<BrokerCredentialGrant>,
    ) -> Result<BrokerSessionHandle, BrokerDispatchError> {
        binding.validate()?;
        if credential_grants.is_empty()
            || credential_grants.len() > MAX_CREDENTIALS_PER_SESSION
            || credential_grants
                .iter()
                .map(|grant| &grant.reference)
                .collect::<HashSet<_>>()
                .len()
                != credential_grants.len()
        {
            return Err(BrokerDispatchError::InvalidCredentialGrant);
        }
        let credential_grants = credential_grants
            .into_iter()
            .map(|grant| (grant.reference, grant.expires_at_unix_seconds))
            .collect::<HashMap<_, _>>();
        let run_key = RunKey {
            controller_instance: binding.controller_instance.clone(),
            run_id: binding.run_id.clone(),
        };
        let mut state = self.lock_state()?;
        if let Some(current_generation) = state.runs.get(&run_key)
            && binding.run_generation <= *current_generation
        {
            return Err(BrokerDispatchError::StaleRunGeneration);
        }
        if !state.runs.contains_key(&run_key) && state.runs.len() >= self.config.max_tracked_runs {
            return Err(BrokerDispatchError::ResourceExhausted);
        }

        cancel_run_locked(&mut state, &run_key);
        if state.sessions.len() >= self.config.max_sessions {
            return Err(BrokerDispatchError::SessionCapacityReached);
        }
        let session_slot = state.next_session_slot;
        state.next_session_slot = state
            .next_session_slot
            .checked_add(1)
            .ok_or(BrokerDispatchError::ResourceExhausted)?;
        state.runs.insert(run_key, binding.run_generation);
        state.sessions.insert(
            session_slot,
            SessionState {
                binding,
                peer,
                channel_mac,
                credential_grants,
                next_sequence: 1,
            },
        );
        Ok(BrokerSessionHandle {
            broker_instance: self.broker_instance.clone(),
            session_slot,
        })
    }

    pub fn cancel_session(&self, handle: &BrokerSessionHandle) -> Result<(), BrokerDispatchError> {
        self.validate_handle(handle)?;
        let mut state = self.lock_state()?;
        remove_session_locked(&mut state, handle.session_slot);
        Ok(())
    }

    pub fn revoke_run(
        &self,
        controller_instance: &str,
        run_id: &str,
    ) -> Result<(), BrokerDispatchError> {
        let mut state = self.lock_state()?;
        let run_key = RunKey {
            controller_instance: controller_instance.to_string(),
            run_id: run_id.to_string(),
        };
        cancel_run_locked(&mut state, &run_key);
        Ok(())
    }

    pub fn revoke_credential(
        &self,
        handle: &BrokerSessionHandle,
        credential: &CredentialReference,
    ) -> Result<(), BrokerDispatchError> {
        self.validate_handle(handle)?;
        let mut state = self.lock_state()?;
        let session = state
            .sessions
            .get_mut(&handle.session_slot)
            .ok_or(BrokerDispatchError::SessionUnavailable)?;
        if session.credential_grants.remove(credential).is_none() {
            return Err(BrokerDispatchError::CredentialUnavailable);
        }
        for operation in state.in_flight.values() {
            if operation.session_slot == handle.session_slot && &operation.credential == credential
            {
                operation.fence.cancel();
            }
        }
        Ok(())
    }

    pub fn dispatch(
        &self,
        handle: &BrokerSessionHandle,
        peer: &ObservedPeer,
        frame: &SignedBrokerFrame,
    ) -> Result<TypedOperationReceipt, BrokerDispatchError> {
        self.validate_handle(handle)?;
        let (operation_id, request, fence) = {
            let mut state = self.lock_state()?;
            if state.in_flight.len() >= self.config.max_in_flight {
                return Err(BrokerDispatchError::OperationCapacityReached);
            }
            let session = state
                .sessions
                .get_mut(&handle.session_slot)
                .ok_or(BrokerDispatchError::SessionUnavailable)?;
            if &session.peer != peer {
                return Err(BrokerDispatchError::WrongPeer);
            }
            let request = session.channel_mac.verify(frame)?;
            if request.binding != session.binding {
                return Err(BrokerDispatchError::BindingMismatch);
            }
            if request.sequence != session.next_sequence {
                return Err(BrokerDispatchError::ReplayOrSequenceGap);
            }
            let credential = request.operation.credential();
            let expires_at = *session
                .credential_grants
                .get(credential)
                .ok_or(BrokerDispatchError::CredentialUnavailable)?;
            if now_unix_seconds()? >= expires_at {
                session.credential_grants.remove(credential);
                return Err(BrokerDispatchError::CredentialExpired);
            }
            session.next_sequence = session
                .next_sequence
                .checked_add(1)
                .ok_or(BrokerDispatchError::ResourceExhausted)?;
            let operation_id = state.next_operation_id;
            state.next_operation_id = state
                .next_operation_id
                .checked_add(1)
                .ok_or(BrokerDispatchError::ResourceExhausted)?;
            let fence = CancellationFence::active();
            state.in_flight.insert(
                operation_id,
                InFlightOperation {
                    session_slot: handle.session_slot,
                    credential: credential.clone(),
                    fence: fence.clone(),
                },
            );
            (operation_id, request, fence)
        };

        let intent = audit_intent(&request.binding, request.sequence, &request.operation);
        let permit = match self.audit.reserve(&intent) {
            Ok(permit) => permit,
            Err(_) => {
                self.finish_operation(operation_id);
                return Err(BrokerDispatchError::AuditUnavailable);
            }
        };

        if fence.ensure_active().is_err() || !self.operation_is_current(operation_id)? {
            self.finish_operation(operation_id);
            return self.finish_audit(
                handle.session_slot,
                permit,
                BrokerAuditResolution::Cancelled,
                None,
            );
        }

        let backend_result = match &request.operation {
            BrokerOperation::OpenAiResponses {
                credential,
                request,
            } => self
                .backend
                .execute_openai_responses(credential, request, &fence),
        };
        self.finish_operation(operation_id);
        match backend_result {
            Ok(receipt) => self.finish_audit(
                handle.session_slot,
                permit,
                BrokerAuditResolution::Completed,
                Some(receipt),
            ),
            Err(BackendDispatchError::Cancelled) => self.finish_audit(
                handle.session_slot,
                permit,
                BrokerAuditResolution::Cancelled,
                None,
            ),
            Err(BackendDispatchError::Failed) => self.finish_audit(
                handle.session_slot,
                permit,
                BrokerAuditResolution::Failed,
                None,
            ),
            Err(BackendDispatchError::OutcomeUnknown) => self.finish_audit(
                handle.session_slot,
                permit,
                BrokerAuditResolution::Unknown,
                None,
            ),
        }
    }

    fn finish_audit(
        &self,
        session_slot: u64,
        permit: A::Permit,
        resolution: BrokerAuditResolution,
        receipt: Option<TypedOperationReceipt>,
    ) -> Result<TypedOperationReceipt, BrokerDispatchError> {
        if self.audit.resolve(permit, resolution).is_err() {
            if let Ok(mut state) = self.state.lock() {
                remove_session_locked(&mut state, session_slot);
            }
            return Err(BrokerDispatchError::AuditCommitUnknown);
        }
        match (resolution, receipt) {
            (BrokerAuditResolution::Completed, Some(receipt)) => Ok(receipt),
            (BrokerAuditResolution::Failed, None) => Err(BrokerDispatchError::BackendFailed),
            (BrokerAuditResolution::Cancelled, None) => Err(BrokerDispatchError::Cancelled),
            (BrokerAuditResolution::Unknown, None) => Err(BrokerDispatchError::OutcomeUnknown),
            _ => Err(BrokerDispatchError::AuditCommitUnknown),
        }
    }

    fn operation_is_current(&self, operation_id: u64) -> Result<bool, BrokerDispatchError> {
        let state = self.lock_state()?;
        let Some(operation) = state.in_flight.get(&operation_id) else {
            return Ok(false);
        };
        Ok(state.sessions.contains_key(&operation.session_slot)
            && operation.fence.ensure_active().is_ok())
    }

    fn finish_operation(&self, operation_id: u64) {
        if let Ok(mut state) = self.state.lock() {
            state.in_flight.remove(&operation_id);
        }
    }

    fn validate_handle(&self, handle: &BrokerSessionHandle) -> Result<(), BrokerDispatchError> {
        if handle.broker_instance != self.broker_instance {
            return Err(BrokerDispatchError::BrokerRestarted);
        }
        Ok(())
    }

    fn lock_state(&self) -> Result<std::sync::MutexGuard<'_, BrokerState>, BrokerDispatchError> {
        self.state
            .lock()
            .map_err(|_| BrokerDispatchError::StateUnavailable)
    }
}

fn audit_intent(
    binding: &BrokerBinding,
    sequence: u64,
    operation: &BrokerOperation,
) -> BrokerAuditIntent {
    match operation {
        BrokerOperation::OpenAiResponses {
            credential,
            request,
        } => BrokerAuditIntent {
            controller_instance: binding.controller_instance.clone(),
            session_id: binding.session_id.clone(),
            task_id: binding.task_id.clone(),
            run_id: binding.run_id.clone(),
            run_generation: binding.run_generation,
            sequence,
            credential_reference: credential.clone(),
            operation: "openai.responses.create",
            destination: "https://api.openai.com:443",
            path: request.path().to_string(),
        },
    }
}

fn remove_session_locked(state: &mut BrokerState, session_slot: u64) {
    state.sessions.remove(&session_slot);
    for operation in state.in_flight.values() {
        if operation.session_slot == session_slot {
            operation.fence.cancel();
        }
    }
}

fn cancel_run_locked(state: &mut BrokerState, run_key: &RunKey) {
    let slots = state
        .sessions
        .iter()
        .filter_map(|(slot, session)| {
            (session.binding.controller_instance == run_key.controller_instance
                && session.binding.run_id == run_key.run_id)
                .then_some(*slot)
        })
        .collect::<Vec<_>>();
    for slot in slots {
        remove_session_locked(state, slot);
    }
}

fn validate_broker_instance(value: &str) -> Result<(), BrokerDispatchError> {
    if value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        Ok(())
    } else {
        Err(BrokerDispatchError::InvalidBrokerInstance)
    }
}

fn now_unix_seconds() -> Result<i64, BrokerDispatchError> {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| BrokerDispatchError::ClockUnavailable)?
        .as_secs();
    i64::try_from(seconds).map_err(|_| BrokerDispatchError::ClockUnavailable)
}
