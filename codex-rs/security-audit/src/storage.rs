use std::collections::BTreeMap;
use std::collections::BTreeSet;

use codex_security_policy::RevocationState;
use serde::Deserialize;
use serde::Serialize;

use crate::ActionId;
use crate::AuthorityIdentity;
use crate::ReservationId;
use crate::SecurityEvent;
use crate::SecurityEventId;
use crate::SecurityEventKind;
use crate::event::hash_value;
use crate::journal::IntegrityCheckpoint;
use crate::journal::JournalError;
use crate::journal::JournalOwner;
use crate::journal::validate_resolution;
use crate::recovery::RecoveryBlocker;
use crate::recovery::RecoveryState;

const JOURNAL_RECORD_SCHEMA_VERSION: u32 = 1;
pub(crate) const GENESIS_HASH: &str =
    "0000000000000000000000000000000000000000000000000000000000000000";

#[derive(Debug)]
pub(crate) struct ScanResult {
    pub(crate) records: Vec<JournalRecord>,
    pub(crate) revocations: RevocationState,
    pub(crate) interrupted_temps: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct JournalRecord {
    pub(crate) schema_version: u32,
    pub(crate) sequence: u64,
    pub(crate) previous_record_sha256: String,
    pub(crate) record_sha256: String,
    pub(crate) event: SecurityEvent,
}

impl JournalRecord {
    pub(crate) fn new(
        sequence: u64,
        previous_record_sha256: String,
        event: SecurityEvent,
    ) -> Result<Self, JournalError> {
        let mut record = Self {
            schema_version: JOURNAL_RECORD_SCHEMA_VERSION,
            sequence,
            previous_record_sha256,
            record_sha256: GENESIS_HASH.to_string(),
            event,
        };
        record.record_sha256 = record.expected_hash()?;
        Ok(record)
    }

    fn expected_hash(&self) -> Result<String, JournalError> {
        hash_value(&(
            self.schema_version,
            self.sequence,
            &self.previous_record_sha256,
            &self.event,
        ))
        .map_err(|_| JournalError::Serialization)
    }

    pub(crate) fn validate(&self, expected_sequence: u64, expected_previous: &str) -> bool {
        self.schema_version == JOURNAL_RECORD_SCHEMA_VERSION
            && self.sequence == expected_sequence
            && self.previous_record_sha256 == expected_previous
            && self.event.validate().is_ok()
            && self
                .expected_hash()
                .is_ok_and(|expected| expected == self.record_sha256)
    }
}

#[derive(Clone, Debug)]
struct IntentState {
    event_id: SecurityEventId,
    action_id: ActionId,
    authority: AuthorityIdentity,
    occurred_at_unix_seconds: i64,
    resolved: bool,
}

pub(crate) fn validate_event_chain(
    records: &[JournalRecord],
    owner: &JournalOwner,
) -> Result<RevocationState, RecoveryBlocker> {
    let mut event_ids = BTreeSet::new();
    let mut restrictions = BTreeSet::new();
    let mut intents = BTreeMap::<ReservationId, IntentState>::new();
    let mut revocations = RevocationState::new();
    let mut policy_generation = 0;
    let mut run_generation = 0;
    for record in records {
        let event = &record.event;
        if event.context.producer != owner.producer
            || !event_ids.insert(event.event_id.clone())
            || event.context.policy_generation < policy_generation
            || event.context.run_generation < run_generation
        {
            return Err(RecoveryBlocker::InvalidRecord);
        }
        policy_generation = event.context.policy_generation;
        run_generation = event.context.run_generation;
        match &event.kind {
            SecurityEventKind::Decision { .. } => {}
            SecurityEventKind::DispatchIntent {
                action_id,
                reservation_id,
                authority,
                ..
            } => {
                if intents
                    .insert(
                        reservation_id.clone(),
                        IntentState {
                            event_id: event.event_id.clone(),
                            action_id: action_id.clone(),
                            authority: authority.clone(),
                            occurred_at_unix_seconds: event.occurred_at_unix_seconds,
                            resolved: false,
                        },
                    )
                    .is_some()
                {
                    return Err(RecoveryBlocker::InvalidRecord);
                }
            }
            SecurityEventKind::DispatchResolution {
                action_id,
                reservation_id,
                resolution,
            } => {
                let intent = intents
                    .get_mut(reservation_id)
                    .ok_or(RecoveryBlocker::InvalidRecord)?;
                if intent.resolved
                    || &intent.action_id != action_id
                    || event.causal_parent.as_ref() != Some(&intent.event_id)
                    || event.occurred_at_unix_seconds < intent.occurred_at_unix_seconds
                    || validate_resolution(&intent.authority, resolution).is_err()
                {
                    return Err(RecoveryBlocker::InvalidRecord);
                }
                intent.resolved = true;
            }
            SecurityEventKind::Restriction { event } => {
                if !restrictions.insert(event.event_id.clone()) {
                    return Err(RecoveryBlocker::InvalidRecord);
                }
                revocations
                    .apply(event)
                    .map_err(|_| RecoveryBlocker::InvalidRecord)?;
            }
        }
    }
    Ok(revocations)
}

pub(crate) fn compare_checkpoint(
    records: &[JournalRecord],
    checkpoint: Option<&IntegrityCheckpoint>,
    owner: &JournalOwner,
) -> Result<RecoveryState, RecoveryBlocker> {
    match (records.last(), checkpoint) {
        (None, None) => Ok(RecoveryState::Empty),
        (Some(_), None) => Err(RecoveryBlocker::MissingIntegrityRoot),
        (None, Some(_)) => Err(RecoveryBlocker::TruncatedJournal),
        (Some(last), Some(root)) => {
            if root.producer != owner.producer
                || root.owner_generation != owner.owner_generation
                || root.integrity_key_id != owner.integrity_key_id
            {
                return Err(RecoveryBlocker::OwnerMismatch);
            }
            let record_count =
                u64::try_from(records.len()).map_err(|_| RecoveryBlocker::InvalidRecord)?;
            if root.sequence < record_count {
                return Err(RecoveryBlocker::RecordsAheadOfIntegrityRoot);
            }
            if root.sequence > record_count {
                return Err(RecoveryBlocker::TruncatedJournal);
            }
            if root.record_sha256 != last.record_sha256
                || root.policy_generation != last.event.context.policy_generation
                || root.run_generation != last.event.context.run_generation
            {
                return Err(RecoveryBlocker::IntegrityRootMismatch);
            }
            Ok(RecoveryState::Ready)
        }
    }
}

pub(crate) fn is_lower_hex_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

pub(crate) fn map_blocker(blocker: RecoveryBlocker) -> JournalError {
    match blocker {
        RecoveryBlocker::ConcurrentWriter => JournalError::ConcurrentWriter,
        RecoveryBlocker::IntegrityRootUnavailable | RecoveryBlocker::MissingIntegrityKey => {
            JournalError::IntegrityRootUnavailable
        }
        RecoveryBlocker::InvalidRecord
        | RecoveryBlocker::InterruptedWrite
        | RecoveryBlocker::MissingIntegrityRoot
        | RecoveryBlocker::TruncatedJournal
        | RecoveryBlocker::RecordsAheadOfIntegrityRoot
        | RecoveryBlocker::IntegrityRootMismatch
        | RecoveryBlocker::OwnerMismatch
        | RecoveryBlocker::PolicyGenerationMismatch
        | RecoveryBlocker::RestrictionAuditGap => JournalError::RecoveryRequired,
    }
}
