#[cfg(test)]
use std::cell::Cell;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Arc;

use codex_security_policy::AuthorizationRequest;
use codex_security_policy::BoundedText;
use codex_security_policy::RevocationState;
use codex_utils_absolute_path::AbsolutePathBuf;

use crate::ActionId;
use crate::AuthorityIdentity;
use crate::DispatchResolution;
use crate::EventContext;
use crate::PendingDispatch;
use crate::ReservationId;
use crate::SecurityEvent;
use crate::SecurityEventId;
use crate::SecurityEventKind;
#[cfg(test)]
pub(crate) use crate::journal_faults::FaultPoint;
#[cfg(test)]
pub(crate) use crate::journal_faults::InjectedFault;
use crate::journal_support::WriterLock;
use crate::journal_support::ensure_directory;
use crate::journal_support::parse_record_name;
use crate::journal_support::parse_segment_name;
use crate::journal_support::parse_temp_name;
use crate::journal_support::segment_number_for_path;
use crate::journal_support::sync_directory;
use crate::journal_support::validate_resolution;
use crate::journal_types::AppendAcknowledgement;
use crate::journal_types::DispatchPermit;
use crate::journal_types::INTEGRITY_CHECKPOINT_SCHEMA_VERSION;
use crate::journal_types::IntegrityCheckpoint;
use crate::journal_types::IntegrityRootError;
use crate::journal_types::IntegrityRootStore;
use crate::journal_types::JournalConfig;
use crate::journal_types::JournalError;
use crate::journal_types::JournalOwner;
use crate::recovery::RecoveryBlocker;
use crate::recovery::RecoveryReport;
use crate::recovery::RecoveryState;
use crate::storage::EventChainState;
use crate::storage::GENESIS_HASH;
use crate::storage::JournalRecord;
use crate::storage::ScanResult;
use crate::storage::compare_checkpoint;
use crate::storage::map_blocker;
use crate::storage::validate_event_chain;

const MAX_RECORD_BYTES: usize = 32 * 1024;

pub struct ReferenceJournal {
    pub(crate) root: AbsolutePathBuf,
    pub(crate) owner: JournalOwner,
    pub(crate) root_store: Arc<dyn IntegrityRootStore>,
    pub(crate) config: JournalConfig,
    pub(crate) blocked: bool,
    pub(crate) reconciliation_required: bool,
    pub(crate) minimum_policy_generation: u64,
    pub(crate) validated: Option<ValidatedJournalState>,
    #[cfg(test)]
    pub(crate) fault: Option<(FaultPoint, InjectedFault)>,
    #[cfg(test)]
    pub(crate) scan_count: Cell<usize>,
}

#[derive(Clone, Debug)]
pub(crate) struct ValidatedJournalState {
    pub(crate) event_count: usize,
    pub(crate) tail_record_sha256: String,
    pub(crate) checkpoint: Option<IntegrityCheckpoint>,
    pub(crate) chain: EventChainState,
}

#[derive(Clone, Debug)]
struct DuplicateDispatch {
    action_id: ActionId,
    reservation_id: ReservationId,
    resolved: bool,
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

    pub(crate) fn mark_blocked(&mut self) {
        self.blocked = true;
        self.validated = None;
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
            self.mark_blocked();
            return RecoveryReport::blocked(RecoveryBlocker::ConcurrentWriter);
        };
        if self.clean_temps().is_err() {
            self.mark_blocked();
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
            .map_err(|error| match error {
                IntegrityRootError::Timeout => JournalError::CommitUnknown {
                    event_id: expected_event_id.clone(),
                },
                IntegrityRootError::MissingKey | IntegrityRootError::Unavailable => {
                    JournalError::IntegrityRootUnavailable
                }
                IntegrityRootError::Conflict | IntegrityRootError::Invalid => {
                    JournalError::AmbiguousCommitMismatch
                }
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
            self.mark_blocked();
            map_blocker(blocker)
        })?;
        if checkpoint != validated.checkpoint {
            self.mark_blocked();
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
        if matches!(event.kind, SecurityEventKind::DispatchIntent { .. })
            && !validated.chain.pending_dispatches().is_empty()
        {
            return Err(JournalError::ReconciliationRequired);
        }
        if validated.event_count >= self.config.max_records {
            self.mark_blocked();
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
                self.mark_blocked();
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

    fn writer_lock(&mut self) -> Result<WriterLock, JournalError> {
        let root_was_absent = !self.root.as_path().exists();
        if ensure_directory(self.root.as_path()).is_err() {
            self.mark_blocked();
            return Err(JournalError::StorageUnavailable);
        }
        if root_was_absent
            && let Some(parent) = self.root.parent()
            && sync_directory(&parent).is_err()
        {
            self.mark_blocked();
            return Err(JournalError::StorageUnavailable);
        }
        let path = self.root.join(".writer.lock");
        let Ok(mut lock) = fslock::LockFile::open(path.as_path()) else {
            self.mark_blocked();
            return Err(JournalError::StorageUnavailable);
        };
        let acquired = match lock.try_lock() {
            Ok(acquired) => acquired,
            Err(_) => {
                self.mark_blocked();
                return Err(JournalError::StorageUnavailable);
            }
        };
        if !acquired {
            self.mark_blocked();
            return Err(JournalError::ConcurrentWriter);
        }
        Ok(WriterLock { _lock: lock })
    }

    fn write_record(&mut self, record: &JournalRecord) -> Result<(), JournalError> {
        let segment = self.segment_path(record.sequence);
        let segment_was_absent = !segment.as_path().exists();
        ensure_directory(segment.as_path()).map_err(|_| {
            self.mark_blocked();
            JournalError::StorageUnavailable
        })?;
        if segment_was_absent && sync_directory(&self.root).is_err() {
            self.mark_blocked();
            return Err(JournalError::StorageUnavailable);
        }
        let bytes = serde_json::to_vec(record).map_err(|_| JournalError::Serialization)?;
        if bytes.len() > MAX_RECORD_BYTES {
            self.mark_blocked();
            return Err(JournalError::StorageUnavailable);
        }
        let temp = self.temp_path(record.sequence);
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(temp.as_path())
            .map_err(|_| {
                self.mark_blocked();
                JournalError::StorageUnavailable
            })?;
        file.write_all(&bytes).map_err(|_| {
            self.mark_blocked();
            JournalError::StorageUnavailable
        })?;
        file.sync_all().map_err(|_| {
            self.mark_blocked();
            JournalError::StorageUnavailable
        })
    }

    fn publish_record(&mut self, record: &JournalRecord) -> Result<(), JournalError> {
        let temp = self.temp_path(record.sequence);
        let final_path = self.record_path(record.sequence);
        fs::hard_link(temp.as_path(), final_path.as_path()).map_err(|_| {
            self.mark_blocked();
            JournalError::StorageUnavailable
        })?;
        fs::remove_file(temp.as_path()).map_err(|_| {
            self.mark_blocked();
            JournalError::CommitUnknown {
                event_id: record.event.event_id.clone(),
            }
        })?;
        #[cfg(test)]
        self.maybe_fault(FaultPoint::BeforeDirectorySync, &record.event.event_id)?;
        let Some(parent) = final_path.parent() else {
            self.mark_blocked();
            return Err(JournalError::CommitUnknown {
                event_id: record.event.event_id.clone(),
            });
        };
        sync_directory(&parent).map_err(|_| {
            self.mark_blocked();
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
}
