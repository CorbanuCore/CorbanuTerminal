use codex_security_policy::RestrictionApplication;
use codex_security_policy::RestrictionAuditStatus;
use codex_security_policy::RevocationError;
use codex_security_policy::RevocationEvent;
use codex_security_policy::RevocationState;
use serde::Deserialize;
use serde::Serialize;

use crate::EventContext;
use crate::IntegrityCheckpoint;
use crate::JournalError;
use crate::ReferenceJournal;
use crate::SecurityEventId;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryBlocker {
    ConcurrentWriter,
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
    RestrictionAuditGap,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RecoveryState {
    Empty,
    Ready,
    Blocked(RecoveryBlocker),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecoveryReport {
    pub state: RecoveryState,
    pub event_count: usize,
    pub checkpoint: Option<IntegrityCheckpoint>,
}

impl RecoveryReport {
    pub(crate) fn blocked(blocker: RecoveryBlocker) -> Self {
        Self {
            state: RecoveryState::Blocked(blocker),
            event_count: 0,
            checkpoint: None,
        }
    }

    pub fn permits_protected_dispatch(&self) -> bool {
        matches!(self.state, RecoveryState::Empty | RecoveryState::Ready)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditGapReason {
    InvalidEvent,
    StorageUnavailable,
    DeadlineExceeded,
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
            | JournalError::InvalidEventSequence => Self::InvalidEvent,
            JournalError::StorageUnavailable | JournalError::IntegrityRootUnavailable => {
                Self::StorageUnavailable
            }
            JournalError::DeadlineExceeded => Self::DeadlineExceeded,
            JournalError::CommitUnknown { .. } | JournalError::AcknowledgementLost { .. } => {
                Self::CommitUnknown
            }
            JournalError::RecoveryRequired => Self::RecoveryRequired,
            JournalError::ConcurrentWriter => Self::ConcurrentWriter,
            JournalError::CapacityExceeded => Self::CapacityExceeded,
            JournalError::ProducerMismatch
            | JournalError::InvalidOwner
            | JournalError::InvalidConfig
            | JournalError::WrongEventKind
            | JournalError::Serialization => Self::InvalidEvent,
        }
    }
}
