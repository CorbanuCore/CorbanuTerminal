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
use crate::journal_support::validate_resolution;
use crate::journal_types::EventChainError;
use crate::journal_types::IntegrityCheckpoint;
use crate::journal_types::JournalError;
use crate::journal_types::JournalOwner;
use crate::recovery::PendingDispatch;
use crate::recovery::RecoveryBlocker;
use crate::recovery::RecoveryState;

const JOURNAL_RECORD_SCHEMA_VERSION: u32 = 1;
pub(crate) const GENESIS_HASH: &str =
    "0000000000000000000000000000000000000000000000000000000000000000";

#[derive(Debug)]
pub(crate) struct ScanResult {
    pub(crate) records: Vec<JournalRecord>,
    pub(crate) chain: EventChainState,
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
    sequence: u64,
    action_id: ActionId,
    deduplication_digest: codex_security_policy::BoundedText,
    authority: AuthorityIdentity,
    occurred_at_unix_seconds: i64,
    resolved: bool,
}

#[derive(Clone, Debug)]
pub(crate) struct EventChainState {
    event_ids: BTreeMap<SecurityEventId, u64>,
    restrictions: BTreeSet<codex_security_policy::BoundedText>,
    intents: BTreeMap<ReservationId, IntentState>,
    dispatch_identities: BTreeMap<(ActionId, codex_security_policy::BoundedText), ReservationId>,
    revocations: RevocationState,
    policy_generation: u64,
    run_generation: u64,
}

impl EventChainState {
    fn new() -> Self {
        Self {
            event_ids: BTreeMap::new(),
            restrictions: BTreeSet::new(),
            intents: BTreeMap::new(),
            dispatch_identities: BTreeMap::new(),
            revocations: RevocationState::new(),
            policy_generation: 0,
            run_generation: 0,
        }
    }

    pub(crate) fn apply(
        &mut self,
        event: &SecurityEvent,
        sequence: u64,
        owner: &JournalOwner,
    ) -> Result<(), EventChainError> {
        if event.context.producer != owner.producer {
            return Err(EventChainError::ProducerMismatch);
        }
        if self.event_ids.contains_key(&event.event_id) {
            return Err(EventChainError::DuplicateEventId);
        }
        if event.context.policy_generation < self.policy_generation {
            return Err(EventChainError::PolicyGenerationRegression);
        }
        if event.context.run_generation < self.run_generation {
            return Err(EventChainError::RunGenerationRegression);
        }
        self.event_ids.insert(event.event_id.clone(), sequence);
        self.policy_generation = event.context.policy_generation;
        self.run_generation = event.context.run_generation;
        match &event.kind {
            SecurityEventKind::Decision { .. } => {}
            SecurityEventKind::DispatchIntent {
                action_id,
                reservation_id,
                authority,
                deduplication_digest,
                ..
            } => {
                if self
                    .dispatch_identities
                    .insert(
                        (action_id.clone(), deduplication_digest.clone()),
                        reservation_id.clone(),
                    )
                    .is_some()
                {
                    return Err(EventChainError::DuplicateDispatchIdentity);
                }
                if self
                    .intents
                    .insert(
                        reservation_id.clone(),
                        IntentState {
                            event_id: event.event_id.clone(),
                            sequence,
                            action_id: action_id.clone(),
                            deduplication_digest: deduplication_digest.clone(),
                            authority: authority.clone(),
                            occurred_at_unix_seconds: event.occurred_at_unix_seconds,
                            resolved: false,
                        },
                    )
                    .is_some()
                {
                    return Err(EventChainError::DuplicateReservation);
                }
            }
            SecurityEventKind::DispatchResolution {
                action_id,
                reservation_id,
                resolution,
            } => {
                let intent = self
                    .intents
                    .get_mut(reservation_id)
                    .ok_or(EventChainError::UnknownReservation)?;
                if intent.resolved {
                    return Err(EventChainError::AlreadyResolved);
                }
                if &intent.action_id != action_id {
                    return Err(EventChainError::ActionMismatch);
                }
                if event.causal_parent.as_ref() != Some(&intent.event_id) {
                    return Err(EventChainError::CausalParentMismatch);
                }
                if event.occurred_at_unix_seconds < intent.occurred_at_unix_seconds {
                    return Err(EventChainError::TimestampRegression);
                }
                if validate_resolution(&intent.authority, resolution).is_err() {
                    return Err(EventChainError::InvalidResolutionAuthority);
                }
                intent.resolved = true;
            }
            SecurityEventKind::Restriction { event } => {
                if !self.restrictions.insert(event.event_id.clone()) {
                    return Err(EventChainError::DuplicateRestriction);
                }
                self.revocations
                    .apply(event)
                    .map_err(|_| EventChainError::InvalidRestriction)?;
            }
        }
        Ok(())
    }

    pub(crate) fn revocations(&self) -> &RevocationState {
        &self.revocations
    }

    pub(crate) fn pending_dispatches(&self) -> Vec<PendingDispatch> {
        self.intents
            .iter()
            .filter(|(_, intent)| !intent.resolved)
            .map(|(reservation_id, intent)| PendingDispatch {
                intent_event_id: intent.event_id.clone(),
                action_id: intent.action_id.clone(),
                reservation_id: reservation_id.clone(),
                occurred_at_unix_seconds: intent.occurred_at_unix_seconds,
            })
            .collect()
    }

    pub(crate) fn reservation_is_resolved(&self, reservation_id: &ReservationId) -> bool {
        self.intents
            .get(reservation_id)
            .is_some_and(|intent| intent.resolved)
    }

    pub(crate) fn event_sequence(&self, event_id: &SecurityEventId) -> Option<u64> {
        self.event_ids.get(event_id).copied()
    }

    pub(crate) fn matching_dispatch(
        &self,
        action_id: &ActionId,
        deduplication_digest: &codex_security_policy::BoundedText,
    ) -> Option<(SecurityEventId, ActionId, ReservationId, u64, bool)> {
        let reservation_id = self
            .dispatch_identities
            .get(&(action_id.clone(), deduplication_digest.clone()))?;
        let intent = self.intents.get(reservation_id)?;
        debug_assert_eq!(&intent.action_id, action_id);
        debug_assert_eq!(&intent.deduplication_digest, deduplication_digest);
        Some((
            intent.event_id.clone(),
            intent.action_id.clone(),
            reservation_id.clone(),
            intent.sequence,
            intent.resolved,
        ))
    }
}

pub(crate) fn validate_event_chain(
    records: &[JournalRecord],
    owner: &JournalOwner,
) -> Result<EventChainState, RecoveryBlocker> {
    let mut state = EventChainState::new();
    for record in records {
        state
            .apply(&record.event, record.sequence, owner)
            .map_err(|_| RecoveryBlocker::InvalidRecord)?;
    }
    Ok(state)
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
        | RecoveryBlocker::RunGenerationMismatch
        | RecoveryBlocker::RestrictionAuditGap => JournalError::RecoveryRequired,
    }
}
