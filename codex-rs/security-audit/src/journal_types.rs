use codex_security_policy::BoundedText;
use codex_security_policy::PolicyPrincipal;
use serde::Deserialize;
use serde::Serialize;
use thiserror::Error;

use crate::ActionId;
use crate::AuthorityIdentity;
use crate::EventContext;
use crate::ReservationId;
use crate::SecurityEventError;
use crate::SecurityEventId;

pub(crate) const INTEGRITY_CHECKPOINT_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Eq)]
/// Identity of the controller-authorized journal owner.
///
/// Constructing this value does not authenticate a caller. The integration
/// adapter may construct it only after PF-20 has authenticated the producer,
/// owner generation and integrity-key binding.
pub struct JournalOwner {
    pub(crate) producer: PolicyPrincipal,
    pub(crate) owner_generation: u64,
    pub(crate) integrity_key_id: BoundedText,
}

impl JournalOwner {
    pub fn new(
        producer: PolicyPrincipal,
        owner_generation: u64,
        integrity_key_id: BoundedText,
    ) -> Result<Self, JournalError> {
        if owner_generation == 0 {
            return Err(JournalError::InvalidOwner);
        }
        Ok(Self {
            producer,
            owner_generation,
            integrity_key_id,
        })
    }

    pub fn producer(&self) -> &PolicyPrincipal {
        &self.producer
    }

    pub fn owner_generation(&self) -> u64 {
        self.owner_generation
    }

    pub fn integrity_key_id(&self) -> &BoundedText {
        &self.integrity_key_id
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct JournalConfig {
    pub(crate) max_records: usize,
    pub(crate) records_per_segment: usize,
}

impl JournalConfig {
    pub fn bounded(max_records: usize, records_per_segment: usize) -> Result<Self, JournalError> {
        if max_records == 0 || records_per_segment == 0 || records_per_segment > max_records {
            return Err(JournalError::InvalidConfig);
        }
        Ok(Self {
            max_records,
            records_per_segment,
        })
    }
}

impl Default for JournalConfig {
    fn default() -> Self {
        Self {
            max_records: 4096,
            records_per_segment: 256,
        }
    }
}

/// Externally protected high-water mark.
///
/// Implementations must bind the configured key to PF-20 controller ownership,
/// compare the exact previous checkpoint, and durably commit the replacement
/// before returning success. A local file beside the journal is insufficient.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IntegrityCheckpoint {
    pub schema_version: u32,
    pub sequence: u64,
    pub record_sha256: String,
    pub producer: PolicyPrincipal,
    pub owner_generation: u64,
    pub integrity_key_id: BoundedText,
    pub policy_generation: u64,
    pub run_generation: u64,
}

impl IntegrityCheckpoint {
    pub(crate) fn validate(&self) -> bool {
        self.schema_version == INTEGRITY_CHECKPOINT_SCHEMA_VERSION
            && self.sequence > 0
            && crate::storage::is_lower_hex_sha256(&self.record_sha256)
            && self.owner_generation > 0
            && self.run_generation > 0
    }
}

pub trait IntegrityRootStore: std::fmt::Debug + Send + Sync {
    /// Loads the protected root for this authenticated journal owner.
    ///
    /// `Ok(None)` is valid only for a controller-authorized first install.
    /// Deletion, rollback, a missing key, or inability to distinguish first
    /// install from loss must return an error and fail recovery closed.
    fn load(&self) -> Result<Option<IntegrityCheckpoint>, IntegrityRootError>;

    /// Atomically compares and durably stores the next protected root.
    ///
    /// Success means the replacement survived according to the PF-20 storage
    /// contract; an ambiguous or merely queued write must return an error.
    fn compare_and_store(
        &self,
        expected: Option<&IntegrityCheckpoint>,
        next: &IntegrityCheckpoint,
    ) -> Result<(), IntegrityRootError>;
}

#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum IntegrityRootError {
    #[error("controller-owned integrity key is unavailable")]
    MissingKey,
    #[error("controller-owned integrity root is unavailable")]
    Unavailable,
    #[error("controller-owned integrity-root commit timed out")]
    Timeout,
    #[error("controller-owned integrity root changed concurrently")]
    Conflict,
    #[error("controller-owned integrity root is invalid")]
    Invalid,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppendAcknowledgement {
    /// Identity of the original durable event. For a duplicate append this is
    /// the already-recorded event, not the retry's newly-derived identity.
    pub event_id: SecurityEventId,
    /// Sequence of `event_id`. This may be lower than `checkpoint.sequence`
    /// for a duplicate because the checkpoint is the current protected
    /// high-water mark that proves the original sequence remains anchored.
    pub sequence: u64,
    /// Current protected high-water mark at acknowledgement time. It includes
    /// `sequence`; it is not necessarily the checkpoint first returned for a
    /// duplicate event.
    pub checkpoint: IntegrityCheckpoint,
    pub duplicate: bool,
}

#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum EventChainError {
    #[error("event producer does not match the journal owner")]
    ProducerMismatch,
    #[error("event ID already exists in the journal")]
    DuplicateEventId,
    #[error("event policy generation regressed")]
    PolicyGenerationRegression,
    #[error("event run generation regressed")]
    RunGenerationRegression,
    #[error("dispatch action and deduplication identity already exists")]
    DuplicateDispatchIdentity,
    #[error("dispatch reservation already exists")]
    DuplicateReservation,
    #[error("dispatch resolution references an unknown reservation")]
    UnknownReservation,
    #[error("dispatch reservation is already terminal")]
    AlreadyResolved,
    #[error("dispatch resolution action does not match its intent")]
    ActionMismatch,
    #[error("dispatch resolution causal parent does not match its intent")]
    CausalParentMismatch,
    #[error("dispatch resolution timestamp precedes its intent")]
    TimestampRegression,
    #[error("dispatch resolution is not valid for the recorded authority")]
    InvalidResolutionAuthority,
    #[error("restriction event already exists")]
    DuplicateRestriction,
    #[error("restriction event is invalid")]
    InvalidRestriction,
}

/// Non-serializable, single-resolution proof that dispatch intent reached the
/// protected root.
///
/// This is not authority. Consumers must still validate the live PF-16–20
/// request, grant or mandate, revocation state and dispatch fence immediately
/// before the effect. The value is deliberately non-`Clone`. Resolution
/// borrows it so a caller can preserve the exact permit across an error proven
/// not to have committed. The journal remains the exactly-once authority: a
/// durable terminal event prevents reuse from resolving the reservation again.
#[derive(Debug)]
pub struct DispatchPermit {
    pub(crate) context: EventContext,
    pub(crate) intent_event_id: SecurityEventId,
    pub(crate) action_id: ActionId,
    pub(crate) reservation_id: ReservationId,
    pub(crate) authority: AuthorityIdentity,
}

impl DispatchPermit {
    pub fn action_id(&self) -> &ActionId {
        &self.action_id
    }

    pub fn reservation_id(&self) -> &ReservationId {
        &self.reservation_id
    }
}

#[derive(Debug, Error)]
pub enum JournalError {
    #[error(transparent)]
    Event(#[from] SecurityEventError),
    #[error("security event producer does not own this journal")]
    ProducerMismatch,
    #[error("journal owner generation must be nonzero")]
    InvalidOwner,
    #[error("journal bounds must be nonzero and segment size cannot exceed total capacity")]
    InvalidConfig,
    #[error("operation requires a different security event kind")]
    WrongEventKind,
    #[error("dispatch resolution does not match its durable intent")]
    InvalidResolution,
    #[error(
        "dispatch reservation {reservation_id} for action {action_id} is already terminal at event {event_id} sequence {sequence}"
    )]
    AlreadyResolved {
        event_id: SecurityEventId,
        action_id: ActionId,
        reservation_id: ReservationId,
        sequence: u64,
    },
    #[error(
        "dispatch reservation {reservation_id} for action {action_id} is already durable at event {event_id} sequence {sequence}; reconcile instead of replaying"
    )]
    AlreadyReserved {
        event_id: SecurityEventId,
        action_id: ActionId,
        reservation_id: ReservationId,
        sequence: u64,
    },
    #[error("unresolved dispatches require explicit reconciliation")]
    ReconciliationRequired,
    #[error(transparent)]
    EventChain(#[from] EventChainError),
    #[error("journal must recover before protected dispatch")]
    RecoveryRequired,
    #[error("another journal writer owns the append lock")]
    ConcurrentWriter,
    #[error("bounded journal capacity is exhausted")]
    CapacityExceeded,
    #[error("durable journal storage is unavailable")]
    StorageUnavailable,
    #[error("controller-owned integrity root is unavailable")]
    IntegrityRootUnavailable,
    #[error("controller-owned integrity root changed concurrently")]
    IntegrityRootConflict,
    #[error("controller-owned integrity root is invalid")]
    IntegrityRootInvalid,
    #[error("event commit is ambiguous; do not dispatch or replay {event_id}")]
    CommitUnknown { event_id: SecurityEventId },
    #[error("published record does not match the operator-selected ambiguous commit")]
    AmbiguousCommitMismatch,
    #[error("journal serialization failed")]
    Serialization,
}
