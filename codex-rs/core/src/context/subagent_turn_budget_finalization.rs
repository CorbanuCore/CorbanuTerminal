use super::ContextualUserFragment;

/// Request-local instruction for the final allowed sampling request of a sub-agent turn.
///
/// This is deliberately not persisted: a later user/parent follow-up receives a fresh bounded
/// turn rather than inheriting a stale finalization order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SubagentTurnBudgetFinalization {
    pub(crate) max_model_requests: usize,
}

impl ContextualUserFragment for SubagentTurnBudgetFinalization {
    fn role(&self) -> &'static str {
        "developer"
    }

    fn markers(&self) -> (&'static str, &'static str) {
        Self::type_markers()
    }

    fn type_markers() -> (&'static str, &'static str) {
        ("", "")
    }

    fn body(&self) -> String {
        format!(
            "This is the final model request allowed for this sub-agent turn \
             (limit: {}). Do not call another tool. Return the best concise final result now, \
             including completed work, concrete evidence, and any remaining limitation.",
            self.max_model_requests
        )
    }
}
