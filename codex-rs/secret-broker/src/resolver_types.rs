use crate::ipc::BrokerFrameError;
use crate::ipc::CredentialReference;
use crate::ipc::OpenAiResponsesOperation;
use std::fmt;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use thiserror::Error;

pub(crate) const MAX_CREDENTIALS_PER_SESSION: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BrokerRuntimeConfig {
    pub max_sessions: usize,
    pub max_in_flight: usize,
}

impl BrokerRuntimeConfig {
    pub fn bounded(max_sessions: usize, max_in_flight: usize) -> Result<Self, BrokerDispatchError> {
        if max_sessions == 0 || max_sessions > 1_024 || max_in_flight == 0 || max_in_flight > 4_096
        {
            return Err(BrokerDispatchError::InvalidConfig);
        }
        Ok(Self {
            max_sessions,
            max_in_flight,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BrokerCredentialGrant {
    pub(crate) reference: CredentialReference,
    pub(crate) expires_at_unix_seconds: i64,
}

impl BrokerCredentialGrant {
    pub fn expiring(
        reference: CredentialReference,
        expires_at_unix_seconds: i64,
    ) -> Result<Self, BrokerDispatchError> {
        if expires_at_unix_seconds <= 0 {
            return Err(BrokerDispatchError::InvalidCredentialGrant);
        }
        Ok(Self {
            reference,
            expires_at_unix_seconds,
        })
    }
}

/// Opaque process-local session handle. It is bound to one broker boot.
pub struct BrokerSessionHandle {
    pub(crate) broker_instance: String,
    pub(crate) session_slot: u64,
}

impl fmt::Debug for BrokerSessionHandle {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("BrokerSessionHandle(<redacted>)")
    }
}

#[derive(Clone)]
pub struct CancellationFence(Arc<AtomicBool>);

impl CancellationFence {
    pub(crate) fn active() -> Self {
        Self(Arc::new(AtomicBool::new(false)))
    }

    pub(crate) fn cancel(&self) {
        self.0.store(true, Ordering::Release);
    }

    /// Trusted transports check this before connect and between upload chunks.
    pub fn ensure_active(&self) -> Result<(), BackendDispatchError> {
        if self.0.load(Ordering::Acquire) {
            Err(BackendDispatchError::Cancelled)
        } else {
            Ok(())
        }
    }
}

impl fmt::Debug for CancellationFence {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("CancellationFence(<state>)")
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedOperationReceipt {
    pub response_status: u16,
    pub uploaded_bytes: u64,
    pub downloaded_bytes: u64,
}

/// Trusted in-broker transport. Implementations must never return secret bytes.
pub trait TypedCredentialBackend: Send + Sync + 'static {
    fn execute_openai_responses(
        &self,
        credential: &CredentialReference,
        operation: &OpenAiResponsesOperation,
        cancellation: &CancellationFence,
    ) -> Result<TypedOperationReceipt, BackendDispatchError>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BrokerAuditIntent {
    pub controller_instance: String,
    pub session_id: String,
    pub task_id: String,
    pub run_id: String,
    pub run_generation: u64,
    pub sequence: u64,
    pub credential_reference: CredentialReference,
    pub operation: &'static str,
    pub destination: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BrokerAuditResolution {
    Completed,
    Failed,
    Cancelled,
    Unknown,
}

/// Adapter boundary for PF-41's durable intent/terminal-resolution journal.
/// `reserve` must durably commit before returning a non-cloneable permit.
pub trait DurableBrokerAudit: Send + Sync + 'static {
    type Permit: Send;

    fn reserve(&self, intent: &BrokerAuditIntent) -> Result<Self::Permit, BrokerAuditError>;

    fn resolve(
        &self,
        permit: Self::Permit,
        resolution: BrokerAuditResolution,
    ) -> Result<(), BrokerAuditError>;
}

#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum BackendDispatchError {
    #[error("broker operation was cancelled")]
    Cancelled,
    #[error("broker operation failed before a known external effect")]
    Failed,
    #[error("broker operation outcome is unknown")]
    OutcomeUnknown,
}

#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum BrokerAuditError {
    #[error("durable broker audit is unavailable")]
    Unavailable,
    #[error("durable broker audit commit outcome is unknown")]
    CommitUnknown,
}

#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum BrokerDispatchError {
    #[error("broker runtime bounds are invalid")]
    InvalidConfig,
    #[error("broker instance identity is invalid")]
    InvalidBrokerInstance,
    #[error("protected broker platform evidence is unavailable")]
    PlatformUnavailable,
    #[error("broker credential grant is invalid")]
    InvalidCredentialGrant,
    #[error("broker session capacity is exhausted")]
    SessionCapacityReached,
    #[error("broker operation capacity is exhausted")]
    OperationCapacityReached,
    #[error("broker resource counter is exhausted")]
    ResourceExhausted,
    #[error("broker run generation is stale")]
    StaleRunGeneration,
    #[error("broker session is unavailable")]
    SessionUnavailable,
    #[error("broker session belongs to a previous broker instance")]
    BrokerRestarted,
    #[error("broker request came from the wrong OS peer")]
    WrongPeer,
    #[error("broker request binding does not match its channel")]
    BindingMismatch,
    #[error("broker request was replayed or skipped a sequence")]
    ReplayOrSequenceGap,
    #[error("broker credential is unavailable for this session")]
    CredentialUnavailable,
    #[error("broker credential authority is expired")]
    CredentialExpired,
    #[error("broker clock is unavailable")]
    ClockUnavailable,
    #[error("broker state is unavailable")]
    StateUnavailable,
    #[error("durable dispatch intent could not be committed")]
    AuditUnavailable,
    #[error("durable terminal receipt commit is ambiguous")]
    AuditCommitUnknown,
    #[error("broker operation was cancelled")]
    Cancelled,
    #[error("broker operation failed before a known external effect")]
    BackendFailed,
    #[error("broker operation outcome is unknown")]
    OutcomeUnknown,
    #[error(transparent)]
    Frame(#[from] BrokerFrameError),
}
