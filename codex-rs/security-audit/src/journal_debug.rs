use crate::ReferenceJournal;

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
