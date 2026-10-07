//! Tool approvals for contained Claude panes (issue #218).
//!
//! A contained pane runs Claude Code with `--permission-mode default` and
//! `--permission-prompt-tool stdio` instead of `bypassPermissions`: Claude
//! Code allows its own read-only tools, and asks before anything else by
//! writing a `can_use_tool` control request to stdout. Each request is shown
//! to a person here, and the answer goes back on Claude Code's stdin. With
//! nobody to ask (the smoke commands), every request is denied.

use std::sync::Arc;
use std::sync::Mutex;

use serde_json::Value;
use serde_json::json;
use tokio::sync::oneshot;

use crate::bottom_pane::SelectionItem;
use crate::bottom_pane::SelectionViewParams;

const SUMMARY_MAX_CHARS: usize = 400;
const DENIED_MESSAGE: &str = "The person using Corbanu Terminal denied this tool use.";

/// One `can_use_tool` request waiting for a person.
#[derive(Debug)]
pub(crate) struct ClaudeApprovalRequest {
    pub(crate) pane_title: String,
    pub(crate) tool_name: String,
    /// What the tool would do (command, file, URL), already redacted.
    pub(crate) summary: String,
    pub(crate) responder: ApprovalResponder,
}

/// Answers one request once; later answers and a dropped turn are ignored.
#[derive(Clone, Debug)]
pub(crate) struct ApprovalResponder(Arc<Mutex<Option<oneshot::Sender<bool>>>>);

impl ApprovalResponder {
    pub(crate) fn new() -> (Self, oneshot::Receiver<bool>) {
        let (tx, rx) = oneshot::channel();
        (Self(Arc::new(Mutex::new(Some(tx)))), rx)
    }

    pub(crate) fn respond(&self, allow: bool) {
        let sender = self
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
        if let Some(sender) = sender {
            let _ = sender.send(allow);
        }
    }
}

/// The prompt as Claude Code's first stream-json input message.
pub(crate) fn user_message(prompt: &str) -> Value {
    json!({
        "type": "user",
        "message": { "role": "user", "content": prompt },
    })
}

/// A `can_use_tool` request: `(request_id, tool_name, input)`.
pub(crate) fn tool_request(value: &Value) -> Option<(&str, &str, &Value)> {
    let request = value.get("request")?;
    if request.get("subtype").and_then(Value::as_str) != Some("can_use_tool") {
        return None;
    }
    Some((
        value.get("request_id")?.as_str()?,
        request.get("tool_name")?.as_str()?,
        request.get("input").unwrap_or(&Value::Null),
    ))
}

pub(crate) fn is_control_request(value: &Value) -> bool {
    value.get("type").and_then(Value::as_str) == Some("control_request")
}

/// The answer to a `can_use_tool` request. An allowed tool runs with the
/// input it asked for.
pub(crate) fn tool_response(request_id: &str, allow: bool, input: &Value) -> Value {
    let decision = if allow {
        json!({ "behavior": "allow", "updatedInput": input })
    } else {
        json!({ "behavior": "deny", "message": DENIED_MESSAGE })
    };
    json!({
        "type": "control_response",
        "response": {
            "subtype": "success",
            "request_id": request_id,
            "response": decision,
        },
    })
}

/// Any other control request is refused rather than left hanging.
pub(crate) fn unsupported_response(value: &Value) -> Option<Value> {
    let request_id = value.get("request_id")?.as_str()?;
    Some(json!({
        "type": "control_response",
        "response": {
            "subtype": "error",
            "request_id": request_id,
            "error": "Corbanu Terminal does not support this control request",
        },
    }))
}

/// What the tool would do, for the person deciding: the command, path or URL
/// when the tool has one, otherwise its input.
pub(crate) fn summary(tool_name: &str, input: &Value) -> String {
    let field = |name: &str| input.get(name).and_then(Value::as_str);
    let text = match tool_name {
        "Bash" => field("command").map(str::to_string),
        "Write" | "Edit" | "MultiEdit" | "NotebookEdit" | "Read" => field("file_path")
            .or_else(|| field("notebook_path"))
            .map(str::to_string),
        "WebFetch" => field("url").map(str::to_string),
        "WebSearch" => field("query").map(str::to_string),
        _ => None,
    }
    .unwrap_or_else(|| input.to_string());
    let mut chars = text.chars();
    let mut shown = chars.by_ref().take(SUMMARY_MAX_CHARS).collect::<String>();
    if chars.next().is_some() {
        shown.push('…');
    }
    shown
}

/// The approval popup: allow once, or deny (also Esc).
pub(crate) fn selection_view(request: ClaudeApprovalRequest) -> SelectionViewParams {
    let allow = request.responder.clone();
    let deny = request.responder.clone();
    let cancel = request.responder;
    SelectionViewParams {
        title: Some(format!(
            "{} wants to use {}",
            request.pane_title, request.tool_name
        )),
        subtitle: Some(request.summary),
        footer_hint: Some("Enter chooses · Esc denies".into()),
        items: vec![
            SelectionItem {
                name: "Allow once".to_string(),
                description: Some("Claude Code runs it inside the pane's sandbox".to_string()),
                actions: vec![Box::new(move |_| allow.respond(/*allow*/ true))],
                dismiss_on_select: true,
                ..Default::default()
            },
            SelectionItem {
                name: "Deny".to_string(),
                description: Some("Claude Code is told you denied it".to_string()),
                actions: vec![Box::new(move |_| deny.respond(/*allow*/ false))],
                dismiss_on_select: true,
                ..Default::default()
            },
        ],
        // A stray digit must not approve a tool.
        allow_number_shortcuts: false,
        allow_cancel: true,
        on_cancel: Some(Box::new(move |_| cancel.respond(/*allow*/ false))),
        ..Default::default()
    }
}

#[cfg(test)]
#[path = "approval_tests.rs"]
mod tests;
