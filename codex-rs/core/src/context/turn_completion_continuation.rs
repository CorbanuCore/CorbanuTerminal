use super::ContextualUserFragment;
use codex_protocol::models::ContentItem;
use codex_protocol::models::ContentItemKind;
use codex_protocol::models::input_text;

/// Request-local instruction used after an action provider stops at a progress
/// checkpoint. Callers must not persist this fragment in conversation history.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TurnCompletionContinuation;

impl ContextualUserFragment for TurnCompletionContinuation {
    fn content_kind(&self) -> ContentItemKind {
        ContentItemKind("generic.turn_completion_continuation".to_string())
    }

    fn role(&self) -> &'static str {
        "developer"
    }

    fn markers(&self) -> (&'static str, &'static str) {
        Self::type_markers()
    }

    fn type_markers() -> (&'static str, &'static str) {
        ("", "")
    }

    fn body(&self) -> ContentItem {
        input_text(concat!(
            "Continue executing the current user request. ",
            "The previous response was a progress checkpoint, not a completed result."
        ))
    }
}
