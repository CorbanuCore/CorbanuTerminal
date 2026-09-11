use std::collections::HashSet;
use std::sync::Mutex;
use std::sync::PoisonError;

/// Permission to display native UI is consumed once, independently of the
/// result. Never cache secrets: later operations still consult the OS store.
#[derive(Default)]
pub(super) struct PromptBudget {
    attempted: Mutex<HashSet<(String, String)>>,
}

impl PromptBudget {
    pub(super) fn take(&self, service: &str, account: &str) -> bool {
        self.attempted
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .insert((service.to_owned(), account.to_owned()))
    }
}

#[cfg(test)]
#[path = "prompt_budget_tests.rs"]
mod tests;
