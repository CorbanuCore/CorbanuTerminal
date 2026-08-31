use std::fs;
use std::fs::OpenOptions;
use std::io::Write;

use crate::journal::ReferenceJournal;
use crate::journal_support::MAX_RECORD_BYTES;
use crate::journal_support::WriterLock;
use crate::journal_support::ensure_directory;
use crate::journal_support::sync_directory;
use crate::journal_types::JournalError;
use crate::storage::JournalRecord;

impl ReferenceJournal {
    pub(crate) fn writer_lock(&mut self) -> Result<WriterLock, JournalError> {
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

    pub(crate) fn write_record(&mut self, record: &JournalRecord) -> Result<(), JournalError> {
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

    pub(crate) fn publish_record(&mut self, record: &JournalRecord) -> Result<(), JournalError> {
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
        self.maybe_fault(
            crate::journal_faults::FaultPoint::BeforeDirectorySync,
            &record.event.event_id,
        )?;
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

    pub(crate) fn segment_number(&self, sequence: u64) -> u64 {
        (sequence - 1) / u64::try_from(self.config.records_per_segment).unwrap_or(1) + 1
    }

    fn segment_path(&self, sequence: u64) -> codex_utils_absolute_path::AbsolutePathBuf {
        self.root
            .join(format!("segment-{:020}", self.segment_number(sequence)))
    }

    fn record_path(&self, sequence: u64) -> codex_utils_absolute_path::AbsolutePathBuf {
        self.segment_path(sequence)
            .join(format!("record-{sequence:020}.json"))
    }

    fn temp_path(&self, sequence: u64) -> codex_utils_absolute_path::AbsolutePathBuf {
        self.segment_path(sequence)
            .join(format!("record-{sequence:020}.json.tmp"))
    }
}
