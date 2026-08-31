use crate::ReferenceJournal;
use crate::SecurityEventId;
use crate::journal_types::JournalError;

impl ReferenceJournal {
    pub(crate) fn inject_once(&mut self, point: FaultPoint, fault: InjectedFault) {
        self.fault = Some((point, fault));
    }

    pub(crate) fn scan_count(&self) -> usize {
        self.scan_count.get()
    }

    pub(crate) fn maybe_fault(
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
        self.mark_blocked();
        match fault {
            InjectedFault::DiskFull => Err(JournalError::StorageUnavailable),
            InjectedFault::Crash => Err(JournalError::CommitUnknown {
                event_id: event_id.clone(),
            }),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FaultPoint {
    BeforeRecordWrite,
    AfterRecordSync,
    BeforeDirectorySync,
    AfterRecordRename,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum InjectedFault {
    DiskFull,
    Crash,
}
