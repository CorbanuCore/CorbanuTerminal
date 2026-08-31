use codex_security_policy::RestrictionApplication;
use codex_security_policy::RestrictionAuditStatus;
use codex_security_policy::RevocationError;
use codex_security_policy::RevocationEvent;
use codex_security_policy::RevocationState;
use serde::Deserialize;
use serde::Serialize;

use crate::ActionId;
use crate::EventContext;
use crate::IntegrityCheckpoint;
use crate::JournalError;
use crate::ReferenceJournal;
use crate::ReservationId;
use crate::SecurityEventId;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryBlocker {
    ConcurrentWriter,
    StorageUnavailable,
    InvalidRecord,
    InterruptedWrite,
    IntegrityRootUnavailable,
    MissingIntegrityKey,
    MissingIntegrityRoot,
    TruncatedJournal,
    RecordsAheadOfIntegrityRoot,
    IntegrityRootMismatch,
    OwnerMismatch,
    PolicyGenerationMismatch,
    RunGenerationMismatch,
    RestrictionAuditGap,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RecoveryState {
    Empty,
    Ready,
    ReconciliationRequired,
    Blocked(RecoveryBlocker),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecoveryReport {
    pub state: RecoveryState,
    pub event_count: usize,
    pub checkpoint: Option<IntegrityCheckpoint>,
    /// Durable intents without a terminal completed/unknown receipt.
    ///
    /// Consumers must reconcile every entry as explicitly unknown before
    /// starting another protected dispatch after recovery.
    pub pending_dispatches: Vec<PendingDispatch>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PendingDispatch {
    pub intent_event_id: SecurityEventId,
    pub action_id: ActionId,
    pub reservation_id: ReservationId,
    /// Timestamp of the durable intent. Reconciliation timestamps are clamped
    /// to at least this value so a backwards wall-clock step cannot deadlock
    /// operator resolution.
    pub occurred_at_unix_seconds: i64,
}

impl RecoveryReport {
    pub(crate) fn blocked(blocker: RecoveryBlocker) -> Self {
        Self {
            state: RecoveryState::Blocked(blocker),
            event_count: 0,
            checkpoint: None,
            pending_dispatches: Vec::new(),
        }
    }

    pub fn permits_protected_dispatch(&self) -> bool {
        matches!(self.state, RecoveryState::Empty | RecoveryState::Ready)
            && self.pending_dispatches.is_empty()
    }

    pub(crate) fn permits_journal_writes(&self) -> bool {
        matches!(
            self.state,
            RecoveryState::Empty | RecoveryState::Ready | RecoveryState::ReconciliationRequired
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditGapReason {
    InvalidEvent,
    StorageUnavailable,
    IntegrityRootConflict,
    IntegrityRootInvalid,
    CommitUnknown,
    RecoveryRequired,
    ConcurrentWriter,
    CapacityExceeded,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EmergencyRestrictionResult {
    pub application: RestrictionApplication,
    pub audit_event_id: Option<SecurityEventId>,
    pub gap: Option<AuditGapReason>,
}

/// Fence authority first, then attempt the durable audit write.
///
/// The PF-19 revocation state changes before the closure touches storage. Audit
/// failure is returned as an explicit gap and never rolls back the restriction.
/// On restart, [`ReferenceJournal::recover`] compares the reconstructed
/// restriction ledger with the PF-20-owned state and remains blocked while a gap
/// exists.
pub fn apply_emergency_restriction(
    state: &mut RevocationState,
    event: &RevocationEvent,
    journal: &mut ReferenceJournal,
    context: EventContext,
    causal_parent: Option<SecurityEventId>,
) -> Result<EmergencyRestrictionResult, RevocationError> {
    let mut audit_event_id = None;
    let mut gap = None;
    let application = state.apply_restriction(event, || {
        match journal.record_restriction(context, causal_parent, event.clone()) {
            Ok(acknowledgement) => {
                audit_event_id = Some(acknowledgement.event_id);
                RestrictionAuditStatus::Recorded
            }
            Err(error) => {
                gap = Some(AuditGapReason::from(&error));
                RestrictionAuditStatus::Unavailable
            }
        }
    })?;
    Ok(EmergencyRestrictionResult {
        application,
        audit_event_id,
        gap,
    })
}

impl From<&JournalError> for AuditGapReason {
    fn from(error: &JournalError) -> Self {
        match error {
            JournalError::Event(_)
            | JournalError::InvalidResolution
            | JournalError::AlreadyReserved { .. }
            | JournalError::AlreadyResolved { .. }
            | JournalError::EventChain(_) => Self::InvalidEvent,
            JournalError::StorageUnavailable | JournalError::IntegrityRootUnavailable => {
                Self::StorageUnavailable
            }
            JournalError::IntegrityRootConflict => Self::IntegrityRootConflict,
            JournalError::IntegrityRootInvalid => Self::IntegrityRootInvalid,
            JournalError::CommitUnknown { .. } => Self::CommitUnknown,
            JournalError::RecoveryRequired | JournalError::ReconciliationRequired => {
                Self::RecoveryRequired
            }
            JournalError::ConcurrentWriter => Self::ConcurrentWriter,
            JournalError::CapacityExceeded => Self::CapacityExceeded,
            JournalError::ProducerMismatch
            | JournalError::InvalidOwner
            | JournalError::InvalidConfig
            | JournalError::WrongEventKind
            | JournalError::AmbiguousCommitMismatch
            | JournalError::Serialization => Self::InvalidEvent,
        }
    }
}
