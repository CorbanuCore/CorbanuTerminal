use std::borrow::Cow;

use codex_protocol::models::SearchToolCallParams;

/// Canonical payload shapes accepted by model-visible tool runtimes.
#[derive(Clone, Debug, PartialEq)]
pub enum ToolPayload {
    Function { arguments: String },
    ToolSearch { arguments: SearchToolCallParams },
    Custom { input: String },
}

impl ToolPayload {
    pub fn log_payload(&self) -> Cow<'_, str> {
        match self {
            ToolPayload::Function { arguments } => Cow::Borrowed(arguments),
            ToolPayload::ToolSearch { arguments } => Cow::Owned(arguments.query.clone()),
            ToolPayload::Custom { input } => Cow::Borrowed(input),
        }
    }

    /// Returns the raw input of a freeform tool call.
    ///
    /// Responses API models call freeform tools with a custom payload. Chat
    /// Completions and Anthropic adapters expose the same tools as functions
    /// with a single `input` string argument, so those calls arrive as
    /// function payloads and must resolve to the same raw input.
    pub fn freeform_input(&self) -> Option<String> {
        match self {
            ToolPayload::Custom { input } => Some(input.clone()),
            ToolPayload::Function { arguments } => {
                match serde_json::from_str::<serde_json::Value>(arguments).ok()? {
                    serde_json::Value::String(input) => Some(input),
                    serde_json::Value::Object(mut map) => match map.remove("input")? {
                        serde_json::Value::String(input) => Some(input),
                        _ => None,
                    },
                    _ => None,
                }
            }
            ToolPayload::ToolSearch { .. } => None,
        }
    }
}

#[cfg(test)]
#[path = "tool_payload_tests.rs"]
mod tests;
