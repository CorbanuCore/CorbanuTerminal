#[cfg(test)]
use std::cell::Cell;
use std::fs;
#[cfg(not(windows))]
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Arc;

use codex_security_policy::AuthorizationRequest;
use codex_security_policy::BoundedText;
use codex_security_policy::PolicyPrincipal;
use codex_security_policy::RevocationState;
use codex_utils_absolute_path::AbsolutePathBuf;
use serde::Deserialize;
use serde::Serialize;
use thiserror::Error;

use crate::ActionId;
use crate::AuthorityIdentity;
use crate::DispatchResolution;
use crate::EventContext;
use crate::PendingDispatch;
use crate::ReservationId;
use crate::SecurityEvent;
use crate::SecurityEventError;
use crate::SecurityEventId;
use crate::SecurityEventKind;
use crate::recovery::RecoveryBlocker;
use crate::recovery::RecoveryReport;
use crate::recovery::RecoveryState;
use crate::storage::EventChainState;
use crate::storage::GENESIS_HASH;
use crate::storage::JournalRecord;
use crate::storage::ScanResult;
use crate::storage::compare_checkpoint;
use crate::storage::is_lower_hex_sha256;
use crate::storage::map_blocker;
use crate::storage::validate_event_chain;

pub(crate) const INTEGRITY_CHECKPOINT_SCHEMA_VERSION: u32 = 1;
const MAX_RECORD_BYTES: usize = 32 * 1024;

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
            && is_lower_hex_sha256(&self.record_sha256)
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
    pub event_id: SecurityEventId,
    pub sequence: u64,
    pub checkpoint: IntegrityCheckpoint,
    pub duplicate: bool,
}

/// Non-serializable, single-resolution proof that dispatch intent reached the
/// protected root.
///
/// This is not authority. Consumers must still validate the live PF-16–20
/// request, grant or mandate, revocation state and dispatch fence immediately
/// before the effect. The value is deliberately non-`Clone` and is consumed by
/// [`ReferenceJournal::resolve_dispatch`].
#[derive(Debug)]
pub struct DispatchPermit {
    context: EventContext,
    intent_event_id: SecurityEventId,
    action_id: ActionId,
    reservation_id: ReservationId,
    authority: AuthorityIdentity,
}

impl DispatchPermit {
    pub fn action_id(&self) -> &ActionId {
        &self.action_id
    }

    pub fn reservation_id(&self) -> &ReservationId {
        &self.reservation_id
    }
}

pub struct ReferenceJournal {
    pub(crate) root: AbsolutePathBuf,
    pub(crate) owner: JournalOwner,
    pub(crate) root_store: Arc<dyn IntegrityRootStore>,
    pub(crate) config: JournalConfig,
    pub(crate) blocked: bool,
    pub(crate) reconciliation_required: bool,
    pub(crate) minimum_policy_generation: u64,
    validated: Option<ValidatedJournalState>,
    #[cfg(test)]
    fault: Option<(FaultPoint, InjectedFault)>,
    #[cfg(test)]
    scan_count: Cell<usize>,
}

#[derive(Clone, Debug)]
struct ValidatedJournalState {
    event_count: usize,
    tail_record_sha256: String,
    checkpoint: Option<IntegrityCheckpoint>,
    chain: EventChainState,
}

#[derive(Clone, Debug)]
struct DuplicateDispatch {
    action_id: ActionId,
    reservation_id: ReservationId,
    resolved: bool,
}

impl std::fmt::Debug for ReferenceJournal {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ReferenceJournal")
            .field("owner", &self.owner)
            .field("config", &self.config)
            .field("blocked", &self.blocked)
            .field("reconciliation_required", &self.reconciliation_required)
            .field("minimum_policy_generation", &self.minimum_policy_generation)
            .finish_non_exhaustive()
    }
}

impl ReferenceJournal {
    pub fn new(
        root: AbsolutePathBuf,
        owner: JournalOwner,
        root_store: Arc<dyn IntegrityRootStore>,
        config: JournalConfig,
    ) -> Self {
        Self {
            root,
            owner,
            root_store,
            config,
            blocked: true,
            reconciliation_required: false,
            minimum_policy_generation: 0,
            validated: None,
            #[cfg(test)]
            fault: None,
            #[cfg(test)]
            scan_count: Cell::new(0),
        }
    }

    pub fn record_decision(
        &mut self,
        event: SecurityEvent,
    ) -> Result<AppendAcknowledgement, JournalError> {
        if !matches!(event.kind, SecurityEventKind::Decision { .. }) {
            return Err(JournalError::WrongEventKind);
        }
        self.append(event)
            .map(|(acknowledgement, _)| acknowledgement)
    }

    pub fn reserve_dispatch(
        &mut self,
        context: EventContext,
        causal_parent: Option<SecurityEventId>,
        request: &AuthorizationRequest,
        authority: AuthorityIdentity,
        deduplication_key: BoundedText,
        occurred_at_unix_seconds: i64,
    ) -> Result<(DispatchPermit, AppendAcknowledgement), JournalError> {
        if self.reconciliation_required {
            return Err(JournalError::ReconciliationRequired);
        }
        let event = SecurityEvent::dispatch_intent(
            context.clone(),
            causal_parent,
            request,
            authority.clone(),
            deduplication_key,
            occurred_at_unix_seconds,
        )?;
        let SecurityEventKind::DispatchIntent {
            action_id,
            reservation_id,
            ..
        } = &event.kind
        else {
            return Err(JournalError::WrongEventKind);
        };
        let permit = DispatchPermit {
            context,
            intent_event_id: event.event_id.clone(),
            action_id: action_id.clone(),
            reservation_id: reservation_id.clone(),
            authority,
        };
        let (acknowledgement, duplicate) = self.append(event)?;
        if let Some(duplicate) = duplicate {
            return Err(if duplicate.resolved {
                JournalError::AlreadyResolved {
                    event_id: acknowledgement.event_id,
                    action_id: duplicate.action_id,
                    reservation_id: duplicate.reservation_id,
                    sequence: acknowledgement.sequence,
                }
            } else {
                JournalError::AlreadyReserved {
                    event_id: acknowledgement.event_id,
                    action_id: duplicate.action_id,
                    reservation_id: duplicate.reservation_id,
                    sequence: acknowledgement.sequence,
                }
            });
        }
        Ok((permit, acknowledgement))
    }

    pub fn resolve_dispatch(
        &mut self,
        permit: DispatchPermit,
        current_context: EventContext,
        resolution: DispatchResolution,
        occurred_at_unix_seconds: i64,
    ) -> Result<AppendAcknowledgement, JournalError> {
        if current_context.producer != permit.context.producer
            || current_context.policy_generation < permit.context.policy_generation
            || current_context.run_generation < permit.context.run_generation
        {
            return Err(JournalError::InvalidResolution);
        }
        validate_resolution(&permit.authority, &resolution)?;
        let event = SecurityEvent::dispatch_resolution(
            current_context,
            permit.intent_event_id,
            permit.action_id,
            permit.reservation_id,
            resolution,
            occurred_at_unix_seconds,
        )?;
        self.append(event)
            .map(|(acknowledgement, _)| acknowledgement)
    }

    /// Reconciles an intent found during recovery as explicitly unknown.
    ///
    /// This never authorizes or replays the external effect. The current
    /// context must come from the live PF-20 state and advance monotonically.
    pub fn reconcile_dispatch_as_unknown(
        &mut self,
        pending: &PendingDispatch,
        current_context: EventContext,
        reason: crate::UnknownOutcomeReason,
        occurred_at_unix_seconds: i64,
    ) -> Result<AppendAcknowledgement, JournalError> {
        if !self.reconciliation_required {
            return Err(JournalError::InvalidResolution);
        }
        let event = SecurityEvent::dispatch_resolution(
            current_context,
            pending.intent_event_id.clone(),
            pending.action_id.clone(),
            pending.reservation_id.clone(),
            DispatchResolution::Unknown { reason },
            occurred_at_unix_seconds,
        )?;
        self.append(event)
            .map(|(acknowledgement, _)| acknowledgement)
    }

    pub fn record_restriction(
        &mut self,
        context: EventContext,
        causal_parent: Option<SecurityEventId>,
        event: codex_security_policy::RevocationEvent,
    ) -> Result<AppendAcknowledgement, JournalError> {
        self.append(SecurityEvent::restriction(context, causal_parent, event)?)
            .map(|(acknowledgement, _)| acknowledgement)
    }

    pub fn recover(
        &mut self,
        expected_policy_generation: u64,
        expected_revocations: &RevocationState,
    ) -> RecoveryReport {
        let Ok(_lock) = self.writer_lock() else {
            self.blocked = true;
            return RecoveryReport::blocked(RecoveryBlocker::ConcurrentWriter);
        };
        if self.clean_temps().is_err() {
            self.blocked = true;
            return RecoveryReport::blocked(RecoveryBlocker::InterruptedWrite);
        }
        let (report, validated) =
            self.inspect(Some((expected_policy_generation, expected_revocations)));
        self.blocked = !report.permits_journal_writes();
        self.reconciliation_required = !self.blocked && !report.pending_dispatches.is_empty();
        if !self.blocked {
            self.minimum_policy_generation = expected_policy_generation;
        }
        self.validated = validated;
        report
    }

    /// Accepts the single fully published record left by an ambiguous root
    /// commit after an operator has matched its event identity to the failed
    /// operation.
    ///
    /// This advances only the protected integrity root. It never creates a
    /// dispatch permit, replays an effect, or unblocks the journal. The caller
    /// must run [`Self::recover`] successfully afterwards, and a recovered
    /// dispatch intent must still be reconciled as explicitly unknown.
    pub fn reconcile_ambiguous_commit(
        &mut self,
        expected_event_id: &SecurityEventId,
        expected_policy_generation: u64,
        expected_revocations: &RevocationState,
    ) -> Result<IntegrityCheckpoint, JournalError> {
        if !self.blocked {
            return Err(JournalError::AmbiguousCommitMismatch);
        }
        let _lock = self.writer_lock()?;
        let scan = self.scan_records().map_err(map_blocker)?;
        if scan.interrupted_temps != 0 || scan.chain.revocations() != expected_revocations {
            return Err(JournalError::AmbiguousCommitMismatch);
        }
        let checkpoint = self
            .load_checkpoint()
            .map_err(map_blocker)?
            .ok_or(JournalError::AmbiguousCommitMismatch)?;
        if checkpoint.producer != self.owner.producer
            || checkpoint.owner_generation != self.owner.owner_generation
            || checkpoint.integrity_key_id != self.owner.integrity_key_id
        {
            return Err(JournalError::AmbiguousCommitMismatch);
        }
        let committed_count = usize::try_from(checkpoint.sequence)
            .map_err(|_| JournalError::AmbiguousCommitMismatch)?;
        if scan.records.len() != committed_count.saturating_add(1) {
            return Err(JournalError::AmbiguousCommitMismatch);
        }
        let anchored = scan
            .records
            .get(committed_count.saturating_sub(1))
            .ok_or(JournalError::AmbiguousCommitMismatch)?;
        if checkpoint.record_sha256 != anchored.record_sha256
            || checkpoint.policy_generation != anchored.event.context.policy_generation
            || checkpoint.run_generation != anchored.event.context.run_generation
        {
            return Err(JournalError::AmbiguousCommitMismatch);
        }
        let last = scan
            .records
            .last()
            .ok_or(JournalError::AmbiguousCommitMismatch)?;
        if &last.event.event_id != expected_event_id
            || last.event.context.policy_generation > expected_policy_generation
        {
            return Err(JournalError::AmbiguousCommitMismatch);
        }
        let next = IntegrityCheckpoint {
            schema_version: INTEGRITY_CHECKPOINT_SCHEMA_VERSION,
            sequence: last.sequence,
            record_sha256: last.record_sha256.clone(),
            producer: self.owner.producer.clone(),
            owner_generation: self.owner.owner_generation,
            integrity_key_id: self.owner.integrity_key_id.clone(),
            policy_generation: last.event.context.policy_generation,
            run_generation: last.event.context.run_generation,
        };
        self.root_store
            .compare_and_store(Some(&checkpoint), &next)
            .map_err(|_| JournalError::CommitUnknown {
                event_id: expected_event_id.clone(),
            })?;
        self.validated = None;
        Ok(next)
    }

    fn append(
        &mut self,
        event: SecurityEvent,
    ) -> Result<(AppendAcknowledgement, Option<DuplicateDispatch>), JournalError> {
        if self.blocked {
            return Err(JournalError::RecoveryRequired);
        }
        event.validate()?;
        if event.context.producer != self.owner.producer {
            return Err(JournalError::ProducerMismatch);
        }
        if event.context.policy_generation < self.minimum_policy_generation {
            return Err(JournalError::InvalidEventSequence);
        }
        let _lock = self.writer_lock()?;
        let validated = self
            .validated
            .clone()
            .ok_or(JournalError::RecoveryRequired)?;
        let checkpoint = self.load_checkpoint().map_err(|blocker| {
            self.blocked = true;
            self.validated = None;
            map_blocker(blocker)
        })?;
        if checkpoint != validated.checkpoint {
            self.blocked = true;
            self.validated = None;
            return Err(JournalError::RecoveryRequired);
        }

        if let Some(sequence) = validated.chain.event_sequence(&event.event_id) {
            let checkpoint = checkpoint.ok_or(JournalError::RecoveryRequired)?;
            let duplicate = match &event.kind {
                SecurityEventKind::DispatchIntent {
                    action_id,
                    reservation_id,
                    ..
                } => Some(DuplicateDispatch {
                    action_id: action_id.clone(),
                    reservation_id: reservation_id.clone(),
                    resolved: validated.chain.reservation_is_resolved(reservation_id),
                }),
                _ => None,
            };
            return Ok((
                AppendAcknowledgement {
                    event_id: event.event_id,
                    sequence,
                    checkpoint,
                    duplicate: true,
                },
                duplicate,
            ));
        }
        if let SecurityEventKind::DispatchIntent {
            action_id,
            deduplication_digest,
            ..
        } = &event.kind
            && let Some((existing_event_id, existing_action_id, reservation_id, sequence, resolved)) =
                validated
                    .chain
                    .matching_dispatch(action_id, deduplication_digest)
        {
            let checkpoint = checkpoint.ok_or(JournalError::RecoveryRequired)?;
            return Ok((
                AppendAcknowledgement {
                    event_id: existing_event_id,
                    sequence,
                    checkpoint,
                    duplicate: true,
                },
                Some(DuplicateDispatch {
                    action_id: existing_action_id,
                    reservation_id,
                    resolved,
                }),
            ));
        }
        if validated.event_count >= self.config.max_records {
            self.blocked = true;
            self.validated = None;
            return Err(JournalError::CapacityExceeded);
        }

        let sequence = u64::try_from(validated.event_count)
            .map_err(|_| JournalError::CapacityExceeded)?
            .checked_add(1)
            .ok_or(JournalError::CapacityExceeded)?;
        let next_event_count = validated
            .event_count
            .checked_add(1)
            .ok_or(JournalError::CapacityExceeded)?;
        let previous = validated.tail_record_sha256;
        let record = JournalRecord::new(sequence, previous, event)?;
        let mut candidate_chain = validated.chain;
        candidate_chain
            .apply(&record.event, sequence, &self.owner)
            .map_err(|_| JournalError::InvalidEventSequence)?;

        #[cfg(test)]
        self.maybe_fault(FaultPoint::BeforeRecordWrite, &record.event.event_id)?;
        self.write_record(&record)?;
        #[cfg(test)]
        self.maybe_fault(FaultPoint::AfterRecordSync, &record.event.event_id)?;

        self.publish_record(&record)?;
        #[cfg(test)]
        self.maybe_fault(FaultPoint::AfterRecordRename, &record.event.event_id)?;

        let next_checkpoint = IntegrityCheckpoint {
            schema_version: INTEGRITY_CHECKPOINT_SCHEMA_VERSION,
            sequence,
            record_sha256: record.record_sha256,
            producer: self.owner.producer.clone(),
            owner_generation: self.owner.owner_generation,
            integrity_key_id: self.owner.integrity_key_id.clone(),
            policy_generation: record.event.context.policy_generation,
            run_generation: record.event.context.run_generation,
        };
        self.root_store
            .compare_and_store(checkpoint.as_ref(), &next_checkpoint)
            .map_err(|_| {
                self.blocked = true;
                self.validated = None;
                JournalError::CommitUnknown {
                    event_id: record.event.event_id.clone(),
                }
            })?;
        if self.reconciliation_required && candidate_chain.pending_dispatches().is_empty() {
            self.reconciliation_required = false;
        }
        self.validated = Some(ValidatedJournalState {
            event_count: next_event_count,
            tail_record_sha256: next_checkpoint.record_sha256.clone(),
            checkpoint: Some(next_checkpoint.clone()),
            chain: candidate_chain,
        });
        Ok((
            AppendAcknowledgement {
                event_id: record.event.event_id,
                sequence,
                checkpoint: next_checkpoint,
                duplicate: false,
            },
            None,
        ))
    }

    fn inspect(
        &self,
        expected: Option<(u64, &RevocationState)>,
    ) -> (RecoveryReport, Option<ValidatedJournalState>) {
        let scan = match self.scan_records() {
            Ok(scan) => scan,
            Err(blocker) => return (RecoveryReport::blocked(blocker), None),
        };
        if scan.interrupted_temps != 0 {
            return (
                RecoveryReport::blocked(RecoveryBlocker::InterruptedWrite),
                None,
            );
        }
        let checkpoint = match self.load_checkpoint() {
            Ok(checkpoint) => checkpoint,
            Err(blocker) => return (RecoveryReport::blocked(blocker), None),
        };
        let state = match compare_checkpoint(&scan.records, checkpoint.as_ref(), &self.owner) {
            Ok(state) => state,
            Err(blocker) => return (RecoveryReport::blocked(blocker), None),
        };
        if let Some((policy_generation, revocations)) = expected {
            if checkpoint
                .as_ref()
                .is_some_and(|root| root.policy_generation > policy_generation)
            {
                return (
                    RecoveryReport::blocked(RecoveryBlocker::PolicyGenerationMismatch),
                    None,
                );
            }
            if scan.chain.revocations() != revocations {
                return (
                    RecoveryReport::blocked(RecoveryBlocker::RestrictionAuditGap),
                    None,
                );
            }
        }
        let pending_dispatches = scan.chain.pending_dispatches();
        let state = if pending_dispatches.is_empty() {
            state
        } else {
            RecoveryState::ReconciliationRequired
        };
        let event_count = scan.records.len();
        let tail_record_sha256 = scan.records.last().map_or_else(
            || GENESIS_HASH.to_string(),
            |record| record.record_sha256.clone(),
        );
        let validated = ValidatedJournalState {
            event_count,
            tail_record_sha256,
            checkpoint: checkpoint.clone(),
            chain: scan.chain,
        };
        (
            RecoveryReport {
                state,
                event_count,
                checkpoint,
                pending_dispatches,
            },
            Some(validated),
        )
    }

    fn load_checkpoint(&self) -> Result<Option<IntegrityCheckpoint>, RecoveryBlocker> {
        let checkpoint = self.root_store.load().map_err(|error| match error {
            IntegrityRootError::MissingKey => RecoveryBlocker::MissingIntegrityKey,
            IntegrityRootError::Unavailable | IntegrityRootError::Timeout => {
                RecoveryBlocker::IntegrityRootUnavailable
            }
            IntegrityRootError::Conflict | IntegrityRootError::Invalid => {
                RecoveryBlocker::IntegrityRootMismatch
            }
        })?;
        if checkpoint.as_ref().is_some_and(|root| !root.validate()) {
            return Err(RecoveryBlocker::IntegrityRootMismatch);
        }
        Ok(checkpoint)
    }

    fn scan_records(&self) -> Result<ScanResult, RecoveryBlocker> {
        #[cfg(test)]
        self.scan_count.set(self.scan_count.get().saturating_add(1));
        ensure_directory(self.root.as_path()).map_err(|_| RecoveryBlocker::InvalidRecord)?;
        let mut paths = Vec::new();
        let mut interrupted_temps = 0;
        let entries =
            fs::read_dir(self.root.as_path()).map_err(|_| RecoveryBlocker::InvalidRecord)?;
        for entry in entries {
            let entry = entry.map_err(|_| RecoveryBlocker::InvalidRecord)?;
            let name = entry.file_name();
            let name = name.to_str().ok_or(RecoveryBlocker::InvalidRecord)?;
            if name == ".writer.lock" {
                continue;
            }
            if parse_segment_name(name).is_none()
                || !entry.file_type().is_ok_and(|kind| kind.is_dir())
            {
                return Err(RecoveryBlocker::InvalidRecord);
            }
            for record in fs::read_dir(entry.path()).map_err(|_| RecoveryBlocker::InvalidRecord)? {
                let record = record.map_err(|_| RecoveryBlocker::InvalidRecord)?;
                let record_name = record.file_name();
                let record_name = record_name.to_str().ok_or(RecoveryBlocker::InvalidRecord)?;
                if parse_temp_name(record_name).is_some() {
                    interrupted_temps += 1;
                    continue;
                }
                if parse_record_name(record_name).is_none() {
                    return Err(RecoveryBlocker::InvalidRecord);
                }
                paths.push(record.path());
            }
        }
        paths.sort();
        if paths.len() > self.config.max_records {
            return Err(RecoveryBlocker::InvalidRecord);
        }
        let mut records = Vec::with_capacity(paths.len());
        let mut previous = GENESIS_HASH.to_string();
        for (index, path) in paths.into_iter().enumerate() {
            let bytes = fs::read(&path).map_err(|_| RecoveryBlocker::InvalidRecord)?;
            if bytes.len() > MAX_RECORD_BYTES {
                return Err(RecoveryBlocker::InvalidRecord);
            }
            let record = serde_json::from_slice::<JournalRecord>(&bytes)
                .map_err(|_| RecoveryBlocker::InvalidRecord)?;
            let sequence = u64::try_from(index)
                .map_err(|_| RecoveryBlocker::InvalidRecord)?
                .checked_add(1)
                .ok_or(RecoveryBlocker::InvalidRecord)?;
            if !record.validate(sequence, &previous)
                || self.segment_number(sequence) != segment_number_for_path(&path, self.config)?
            {
                return Err(RecoveryBlocker::InvalidRecord);
            }
            previous = record.record_sha256.clone();
            records.push(record);
        }
        let chain = validate_event_chain(&records, &self.owner)?;
        Ok(ScanResult {
            records,
            chain,
            interrupted_temps,
        })
    }

    fn writer_lock(&self) -> Result<WriterLock, JournalError> {
        let root_was_absent = !self.root.as_path().exists();
        ensure_directory(self.root.as_path()).map_err(|_| JournalError::StorageUnavailable)?;
        if root_was_absent && let Some(parent) = self.root.parent() {
            sync_directory(&parent)?;
        }
        let path = self.root.join(".writer.lock");
        let mut lock =
            fslock::LockFile::open(path.as_path()).map_err(|_| JournalError::StorageUnavailable)?;
        if !lock
            .try_lock()
            .map_err(|_| JournalError::StorageUnavailable)?
        {
            return Err(JournalError::ConcurrentWriter);
        }
        Ok(WriterLock { _lock: lock })
    }

    fn write_record(&mut self, record: &JournalRecord) -> Result<(), JournalError> {
        let segment = self.segment_path(record.sequence);
        let segment_was_absent = !segment.as_path().exists();
        ensure_directory(segment.as_path()).map_err(|_| {
            self.blocked = true;
            JournalError::StorageUnavailable
        })?;
        if segment_was_absent {
            sync_directory(&self.root)?;
        }
        let bytes = serde_json::to_vec(record).map_err(|_| JournalError::Serialization)?;
        if bytes.len() > MAX_RECORD_BYTES {
            self.blocked = true;
            return Err(JournalError::StorageUnavailable);
        }
        let temp = self.temp_path(record.sequence);
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(temp.as_path())
            .map_err(|_| {
                self.blocked = true;
                JournalError::StorageUnavailable
            })?;
        file.write_all(&bytes).map_err(|_| {
            self.blocked = true;
            JournalError::StorageUnavailable
        })?;
        file.sync_all().map_err(|_| {
            self.blocked = true;
            JournalError::StorageUnavailable
        })
    }

    fn publish_record(&mut self, record: &JournalRecord) -> Result<(), JournalError> {
        let temp = self.temp_path(record.sequence);
        let final_path = self.record_path(record.sequence);
        fs::hard_link(temp.as_path(), final_path.as_path()).map_err(|_| {
            self.blocked = true;
            JournalError::StorageUnavailable
        })?;
        fs::remove_file(temp.as_path()).map_err(|_| {
            self.blocked = true;
            JournalError::CommitUnknown {
                event_id: record.event.event_id.clone(),
            }
        })?;
        #[cfg(test)]
        self.maybe_fault(FaultPoint::BeforeDirectorySync, &record.event.event_id)?;
        sync_directory(
            final_path
                .parent()
                .as_ref()
                .ok_or(JournalError::StorageUnavailable)?,
        )
        .map_err(|_| {
            self.blocked = true;
            JournalError::CommitUnknown {
                event_id: record.event.event_id.clone(),
            }
        })
    }

    fn clean_temps(&self) -> Result<(), JournalError> {
        if !self.root.as_path().exists() {
            return Ok(());
        }
        for segment in
            fs::read_dir(self.root.as_path()).map_err(|_| JournalError::StorageUnavailable)?
        {
            let segment = segment.map_err(|_| JournalError::StorageUnavailable)?;
            if !segment.file_type().is_ok_and(|kind| kind.is_dir()) {
                continue;
            }
            let segment_path = AbsolutePathBuf::from_absolute_path_checked(segment.path())
                .map_err(|_| JournalError::StorageUnavailable)?;
            let mut removed_temp = false;
            for entry in
                fs::read_dir(segment.path()).map_err(|_| JournalError::StorageUnavailable)?
            {
                let entry = entry.map_err(|_| JournalError::StorageUnavailable)?;
                if entry
                    .file_name()
                    .to_str()
                    .and_then(parse_temp_name)
                    .is_some()
                {
                    fs::remove_file(entry.path()).map_err(|_| JournalError::StorageUnavailable)?;
                    removed_temp = true;
                }
            }
            if removed_temp {
                sync_directory(&segment_path)?;
            }
        }
        Ok(())
    }

    fn segment_number(&self, sequence: u64) -> u64 {
        (sequence - 1) / u64::try_from(self.config.records_per_segment).unwrap_or(1) + 1
    }

    fn segment_path(&self, sequence: u64) -> AbsolutePathBuf {
        self.root
            .join(format!("segment-{:020}", self.segment_number(sequence)))
    }

    fn record_path(&self, sequence: u64) -> AbsolutePathBuf {
        self.segment_path(sequence)
            .join(format!("record-{sequence:020}.json"))
    }

    fn temp_path(&self, sequence: u64) -> AbsolutePathBuf {
        self.segment_path(sequence)
            .join(format!("record-{sequence:020}.json.tmp"))
    }

    #[cfg(test)]
    pub(crate) fn inject_once(&mut self, point: FaultPoint, fault: InjectedFault) {
        self.fault = Some((point, fault));
    }

    #[cfg(test)]
    pub(crate) fn scan_count(&self) -> usize {
        self.scan_count.get()
    }

    #[cfg(test)]
    fn maybe_fault(
        &mut self,
        point: FaultPoint,
        event_id: &SecurityEventId,
    ) -> Result<(), JournalError> {
        let Some((configured_point, fault)) = self.fault else {
            return Ok(());
        };
        if configured_point != point {
            return Ok(());
        }
        self.fault = None;
        match fault {
            InjectedFault::DiskFull => {
                self.blocked = true;
                Err(JournalError::StorageUnavailable)
            }
            InjectedFault::Crash => {
                self.blocked = true;
                Err(JournalError::CommitUnknown {
                    event_id: event_id.clone(),
                })
            }
        }
    }
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FaultPoint {
    BeforeRecordWrite,
    AfterRecordSync,
    BeforeDirectorySync,
    AfterRecordRename,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum InjectedFault {
    DiskFull,
    Crash,
}

pub(crate) fn validate_resolution(
    authority: &AuthorityIdentity,
    resolution: &DispatchResolution,
) -> Result<(), JournalError> {
    match (authority, resolution) {
        (
            AuthorityIdentity::Mandate { mandate_id },
            DispatchResolution::Completed {
                outcome,
                mandate_receipt: Some(receipt),
            },
        ) if &receipt.mandate_id == mandate_id && receipt.outcome == *outcome => receipt
            .validate()
            .map_err(|_| JournalError::InvalidResolution),
        (
            AuthorityIdentity::Grant { .. },
            DispatchResolution::Completed {
                mandate_receipt: None,
                ..
            },
        )
        | (_, DispatchResolution::Unknown { .. }) => Ok(()),
        _ => Err(JournalError::InvalidResolution),
    }
}

fn ensure_directory(path: &std::path::Path) -> std::io::Result<()> {
    if path.exists() {
        let metadata = fs::symlink_metadata(path)?;
        if !metadata.is_dir() || metadata.file_type().is_symlink() {
            return Err(std::io::Error::other(
                "audit directory is not a real directory",
            ));
        }
    } else {
        fs::create_dir_all(path)?;
    }
    Ok(())
}

#[cfg(not(windows))]
fn sync_directory(path: &AbsolutePathBuf) -> Result<(), JournalError> {
    File::open(path.as_path())
        .and_then(|directory| directory.sync_all())
        .map_err(|_| JournalError::StorageUnavailable)
}

// Windows does not support opening a directory with std::fs::File. The PF-20
// protected root remains authoritative: loss of an unsynced local entry is
// detected as truncation against that root and fails closed on recovery.
#[cfg(windows)]
fn sync_directory(_path: &AbsolutePathBuf) -> Result<(), JournalError> {
    Ok(())
}

fn segment_number_for_path(
    path: &std::path::Path,
    config: JournalConfig,
) -> Result<u64, RecoveryBlocker> {
    let parent = path.parent().ok_or(RecoveryBlocker::InvalidRecord)?;
    let parsed = parent
        .file_name()
        .and_then(|name| name.to_str())
        .and_then(parse_segment_name)
        .ok_or(RecoveryBlocker::InvalidRecord)?;
    let sequence = path
        .file_name()
        .and_then(|name| name.to_str())
        .and_then(parse_record_name)
        .ok_or(RecoveryBlocker::InvalidRecord)?;
    let expected = (sequence - 1)
        / u64::try_from(config.records_per_segment).map_err(|_| RecoveryBlocker::InvalidRecord)?
        + 1;
    if parsed != expected {
        return Err(RecoveryBlocker::InvalidRecord);
    }
    Ok(parsed)
}

fn parse_segment_name(name: &str) -> Option<u64> {
    let digits = name.strip_prefix("segment-")?;
    if digits.len() != 20 || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    let number = digits.parse::<u64>().ok()?;
    (number != 0).then_some(number)
}

fn parse_temp_name(name: &str) -> Option<u64> {
    let digits = name.strip_prefix("record-")?.strip_suffix(".json.tmp")?;
    if digits.len() != 20 || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    let sequence = digits.parse::<u64>().ok()?;
    (sequence != 0).then_some(sequence)
}

fn parse_record_name(name: &str) -> Option<u64> {
    let digits = name.strip_prefix("record-")?.strip_suffix(".json")?;
    if digits.len() != 20 || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    let sequence = digits.parse::<u64>().ok()?;
    (sequence != 0).then_some(sequence)
}

struct WriterLock {
    _lock: fslock::LockFile,
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
    #[error("unresolved recovered dispatches require explicit reconciliation")]
    ReconciliationRequired,
    #[error("security event violates journal ordering or causal invariants")]
    InvalidEventSequence,
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
    #[error("event commit is ambiguous; do not dispatch or replay {event_id}")]
    CommitUnknown { event_id: SecurityEventId },
    #[error("published record does not match the operator-selected ambiguous commit")]
    AmbiguousCommitMismatch,
    #[error("journal serialization failed")]
    Serialization,
}
