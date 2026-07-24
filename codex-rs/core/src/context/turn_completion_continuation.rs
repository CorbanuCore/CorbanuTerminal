use super::ContextualUserFragment;

/// Request-local instruction used after an action provider stops at a progress
/// checkpoint. Callers must not persist this fragment in conversation history.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TurnCompletionContinuation;

impl ContextualUserFragment for TurnCompletionContinuation {
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
        concat!(
            "Continue executing the current user request. ",
            "The previous response was a progress checkpoint, not a completed result."
        )
        .to_string()
    }
}
