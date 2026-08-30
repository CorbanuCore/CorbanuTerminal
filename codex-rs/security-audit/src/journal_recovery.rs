use std::fs;

use codex_security_policy::RevocationState;

use crate::DispatchResolution;
use crate::EventContext;
use crate::PendingDispatch;
use crate::SecurityEvent;
use crate::SecurityEventId;
use crate::UnknownOutcomeReason;
use crate::journal::ReferenceJournal;
use crate::journal::ValidatedJournalState;
use crate::journal_support::MAX_RECORD_BYTES;
use crate::journal_support::parse_record_name;
use crate::journal_support::parse_segment_name;
use crate::journal_support::parse_temp_name;
use crate::journal_support::segment_number_for_path;
use crate::journal_support::sync_directory;
use crate::journal_types::AppendAcknowledgement;
use crate::journal_types::INTEGRITY_CHECKPOINT_SCHEMA_VERSION;
use crate::journal_types::IntegrityCheckpoint;
use crate::journal_types::IntegrityRootError;
use crate::journal_types::JournalError;
use crate::recovery::RecoveryBlocker;
use crate::recovery::RecoveryReport;
use crate::recovery::RecoveryState;
use crate::storage::GENESIS_HASH;
use crate::storage::JournalRecord;
use crate::storage::ScanResult;
use crate::storage::compare_checkpoint;
use crate::storage::map_blocker;
use crate::storage::validate_event_chain;

impl ReferenceJournal {
    /// Reconciles an intent found during recovery as explicitly unknown.
    ///
    /// This never authorizes or replays the external effect. The current
    /// context must come from the live PF-20 state and advance monotonically.
    pub fn reconcile_dispatch_as_unknown(
        &mut self,
        pending: &PendingDispatch,
        current_context: EventContext,
        reason: UnknownOutcomeReason,
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
            occurred_at_unix_seconds.max(pending.occurred_at_unix_seconds),
        )?;
        self.append(event)
            .map(|(acknowledgement, _)| acknowledgement)
    }

    pub fn recover(
        &mut self,
        expected_policy_generation: u64,
        expected_run_generation: u64,
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
        let (report, validated) = self.inspect(Some((
            expected_policy_generation,
            expected_run_generation,
            expected_revocations,
        )));
        self.blocked = !report.permits_journal_writes();
        self.reconciliation_required = !self.blocked && !report.pending_dispatches.is_empty();
        if !self.blocked {
            self.minimum_policy_generation = expected_policy_generation;
            self.minimum_run_generation = expected_run_generation;
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

    fn inspect(
        &self,
        expected: Option<(u64, u64, &RevocationState)>,
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
        if let Some((policy_generation, run_generation, revocations)) = expected {
            if checkpoint
                .as_ref()
                .is_some_and(|root| root.policy_generation > policy_generation)
            {
                return (
                    RecoveryReport::blocked(RecoveryBlocker::PolicyGenerationMismatch),
                    None,
                );
            }
            if checkpoint
                .as_ref()
                .is_some_and(|root| root.run_generation > run_generation)
            {
                return (
                    RecoveryReport::blocked(RecoveryBlocker::RunGenerationMismatch),
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

    pub(crate) fn load_checkpoint(&self) -> Result<Option<IntegrityCheckpoint>, RecoveryBlocker> {
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
        crate::journal_support::ensure_directory(self.root.as_path())
            .map_err(|_| RecoveryBlocker::InvalidRecord)?;
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

    pub(crate) fn clean_temps(&self) -> Result<(), JournalError> {
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
            let segment_path =
                codex_utils_absolute_path::AbsolutePathBuf::from_absolute_path_checked(
                    segment.path(),
                )
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
}
