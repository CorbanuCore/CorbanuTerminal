//! Tool approvals for contained Claude panes (issue #218).
//!
//! A contained pane runs Claude Code with `--permission-mode default` and
//! `--permission-prompt-tool stdio` instead of `bypassPermissions`. Corbanu's
//! settings add `ask` rules, so every tool that runs commands, edits files or
//! reaches the network asks first, even what Claude Code would consider
//! read-only; reading and searching files stays automatic. Claude Code asks by
//! writing a `can_use_tool` control request to stdout. Each request is shown
//! to a person (`approval_view`) with everything it would do, and the answer
//! goes back on Claude Code's stdin. With nobody to ask (no TUI), every
//! request is denied. The `claude-pane-smoke` and workflow commands never run
//! contained, so they never get here.

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;

use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::text::Span;
use serde_json::Value;
use serde_json::json;
use tokio::sync::oneshot;

/// Characters of tool input shown in full; past this a request can only be
/// denied, since nobody can approve what they cannot see.
pub(crate) const DETAIL_MAX_CHARS: usize = 60_000;
/// Runs of this many spaces or more are shown as a count, so padding cannot
/// push the rest of a command out of sight.
const SPACE_RUN_MIN: usize = 8;
const DENIED_MESSAGE: &str = "The person using Corbanu Terminal denied this tool use.";
const TIMED_OUT_MESSAGE: &str =
    "Nobody answered this tool request in Corbanu Terminal in time, so it was denied.";

/// One `can_use_tool` request waiting for a person.
#[derive(Debug)]
pub(crate) struct ClaudeApprovalRequest {
    pub(crate) pane_id: String,
    pub(crate) pane_title: String,
    pub(crate) cwd: PathBuf,
    pub(crate) tool_name: String,
    pub(crate) tool_use_id: Option<String>,
    pub(crate) details: ApprovalDetails,
    pub(crate) responder: ApprovalResponder,
}

impl ClaudeApprovalRequest {
    /// `claude-1234abcd` for `claude-1234abcd-…`: enough to tell panes apart.
    pub(crate) fn short_pane_id(&self) -> &str {
        let end = self
            .pane_id
            .char_indices()
            .nth("claude-".len() + 8)
            .map_or(self.pane_id.len(), |(index, _)| index);
        &self.pane_id[..end]
    }
}

/// Answers one request once; later answers are ignored. When the waiting turn
/// stops waiting (cancelled, timed out, the turn ended) the request is
/// settled and its popup closes.
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

    /// Answered, or nobody is waiting for an answer any more.
    pub(crate) fn is_settled(&self) -> bool {
        self.0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
            .is_none_or(oneshot::Sender::is_closed)
    }
}

/// The prompt as Claude Code's first stream-json input message.
pub(crate) fn user_message(prompt: &str) -> Value {
    json!({
        "type": "user",
        "message": { "role": "user", "content": prompt },
    })
}

/// A control message from Claude Code, as far as Corbanu needs it.
#[derive(Debug, PartialEq)]
pub(crate) enum ControlMessage<'a> {
    /// `can_use_tool`: ask a person.
    ToolRequest {
        request_id: &'a str,
        tool_name: &'a str,
        tool_use_id: Option<&'a str>,
        input: &'a Value,
    },
    /// Any other request: refused with an error response.
    Unsupported { request_id: &'a str },
    /// Claude Code no longer waits for this request.
    Cancel { request_id: &'a str },
    /// A control message without a request id: nothing can answer it, so the
    /// turn ends rather than hang.
    Malformed,
}

/// Classifies a stdout line; `None` when it is not a control message.
pub(crate) fn control_message(value: &Value) -> Option<ControlMessage<'_>> {
    let kind = value.get("type").and_then(Value::as_str)?;
    if kind != "control_request" && kind != "control_cancel_request" {
        return None;
    }
    let Some(request_id) = value.get("request_id").and_then(Value::as_str) else {
        return Some(ControlMessage::Malformed);
    };
    if kind == "control_cancel_request" {
        return Some(ControlMessage::Cancel { request_id });
    }
    let request = value.get("request");
    let tool_name = request
        .filter(|request| request.get("subtype").and_then(Value::as_str) == Some("can_use_tool"))
        .and_then(|request| request.get("tool_name"))
        .and_then(Value::as_str);
    Some(match (request, tool_name) {
        (Some(request), Some(tool_name)) => ControlMessage::ToolRequest {
            request_id,
            tool_name,
            tool_use_id: request.get("tool_use_id").and_then(Value::as_str),
            input: request.get("input").unwrap_or(&Value::Null),
        },
        _ => ControlMessage::Unsupported { request_id },
    })
}

/// How a request ended without a person allowing it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Denial {
    Person,
    TimedOut,
}

/// The answer to a `can_use_tool` request. An allowed tool runs with exactly
/// the input the person saw.
pub(crate) fn tool_response(
    request_id: &str,
    decision: Result<(), Denial>,
    input: &Value,
) -> Value {
    let decision = match decision {
        Ok(()) => json!({ "behavior": "allow", "updatedInput": input }),
        Err(Denial::Person) => json!({ "behavior": "deny", "message": DENIED_MESSAGE }),
        Err(Denial::TimedOut) => json!({ "behavior": "deny", "message": TIMED_OUT_MESSAGE }),
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
pub(crate) fn unsupported_response(request_id: &str) -> Value {
    json!({
        "type": "control_response",
        "response": {
            "subtype": "error",
            "request_id": request_id,
            "error": "Corbanu Terminal does not support this control request",
        },
    })
}

/// Everything a tool request would do, as display lines: no field is left
/// out, nothing is shortened silently, and characters that would not show
/// (control, format and bidi characters, long runs of spaces) are shown
/// escaped.
#[derive(Debug, Default)]
pub(crate) struct ApprovalDetails {
    pub(crate) lines: Vec<Line<'static>>,
    /// Characters left out past [`DETAIL_MAX_CHARS`]; when non-zero the
    /// request can only be denied.
    pub(crate) hidden_chars: usize,
}

/// The details of one request. `redact` is applied to every value first.
pub(crate) fn details(
    tool_name: &str,
    input: &Value,
    redact: impl Fn(&str) -> String,
) -> ApprovalDetails {
    let mut out = DetailsBuilder {
        details: ApprovalDetails::default(),
        budget: DETAIL_MAX_CHARS,
        redact: &redact,
    };
    let Some(fields) = input.as_object() else {
        out.value("input", input);
        return out.details;
    };
    let text = |name: &str| fields.get(name).and_then(Value::as_str);
    let mut shown: Vec<&str> = Vec::new();
    let primary: &[&str] = match tool_name {
        "Bash" => &["command"],
        "Write" => &["file_path", "content"],
        "Read" | "NotebookEdit" | "NotebookRead" => &["file_path", "notebook_path"],
        "WebFetch" => &["url"],
        "WebSearch" => &["query"],
        _ => &[],
    };
    for &name in primary {
        if let Some(value) = fields.get(name) {
            out.value(name, value);
            shown.push(name);
        }
    }
    match tool_name {
        "Edit" => {
            if let Some(path) = text("file_path") {
                out.value("file_path", &Value::String(path.to_string()));
                shown.push("file_path");
            }
            if fields.contains_key("old_string") || fields.contains_key("new_string") {
                out.diff(
                    text("old_string").unwrap_or_default(),
                    text("new_string").unwrap_or_default(),
                );
                shown.extend(["old_string", "new_string"]);
            }
        }
        "MultiEdit" => {
            if let Some(path) = text("file_path") {
                out.value("file_path", &Value::String(path.to_string()));
                shown.push("file_path");
            }
            if let Some(edits) = fields.get("edits").and_then(Value::as_array) {
                shown.push("edits");
                for (index, edit) in edits.iter().enumerate() {
                    out.label(format!("edit {}:", index + 1));
                    let edit_text = |name: &str| edit.get(name).and_then(Value::as_str);
                    out.diff(
                        edit_text("old_string").unwrap_or_default(),
                        edit_text("new_string").unwrap_or_default(),
                    );
                    if let Some(rest) = edit.as_object() {
                        for (name, value) in rest {
                            if name != "old_string" && name != "new_string" {
                                out.value(name, value);
                            }
                        }
                    }
                }
            }
        }
        _ => {}
    }
    // Every other field, so nothing that changes what the tool does is
    // hidden (Bash's `run_in_background`, `timeout`,
    // `dangerouslyDisableSandbox`, ...).
    for (name, value) in fields {
        if !shown.contains(&name.as_str()) {
            out.value(name, value);
        }
    }
    out.details
}

struct DetailsBuilder<'a> {
    details: ApprovalDetails,
    budget: usize,
    redact: &'a dyn Fn(&str) -> String,
}

impl DetailsBuilder<'_> {
    fn label(&mut self, label: String) {
        if self.details.hidden_chars == 0 {
            self.details.lines.push(Line::from(label.bold()));
        }
    }

    /// `name: value`; strings with line breaks and structured values go on
    /// the lines below their name.
    fn value(&mut self, name: &str, value: &Value) {
        let text = match value {
            Value::String(text) => text.clone(),
            Value::Object(_) | Value::Array(_) => {
                serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string())
            }
            other => other.to_string(),
        };
        let text = (self.redact)(&text);
        let name = escape_plain(name);
        if !text.contains('\n') && text.chars().count() <= 200 {
            self.text(&text, vec![format!("{name}: ").bold()], Style::Plain);
        } else {
            self.label(format!("{name}:"));
            self.text(&text, Vec::new(), Style::Plain);
        }
    }

    fn diff(&mut self, old: &str, new: &str) {
        let old = (self.redact)(old);
        let new = (self.redact)(new);
        if !old.is_empty() {
            self.text(&old, Vec::new(), Style::Removed);
        }
        self.text(&new, Vec::new(), Style::Added);
    }

    /// Appends `text`, one display line per line of text, each but the last
    /// ending in a visible `⏎`.
    fn text(&mut self, text: &str, prefix: Vec<Span<'static>>, style: Style) {
        if self.details.hidden_chars > 0 {
            self.details.hidden_chars += text.chars().count();
            return;
        }
        let total = text.chars().count();
        let (text, hidden) = if total > self.budget {
            let end = text
                .char_indices()
                .nth(self.budget)
                .map_or(text.len(), |(index, _)| index);
            (&text[..end], total - self.budget)
        } else {
            (text, 0)
        };
        self.budget -= total - hidden;
        let marker = match style {
            Style::Plain => None,
            Style::Removed => Some("- ".red()),
            Style::Added => Some("+ ".green()),
        };
        let mut first = Some(prefix);
        let mut pieces = text.split('\n').peekable();
        while let Some(piece) = pieces.next() {
            let mut spans = first.take().unwrap_or_default();
            spans.extend(marker.clone());
            spans.extend(escape(piece).into_iter().map(|(text, is_special)| {
                let span = Span::from(text);
                match (is_special, style) {
                    (true, _) => span.magenta().reversed(),
                    (false, Style::Plain) => span,
                    (false, Style::Removed) => span.red(),
                    (false, Style::Added) => span.green(),
                }
            }));
            if pieces.peek().is_some() {
                spans.push("⏎".magenta());
            }
            self.details.lines.push(Line::from(spans));
        }
        self.details.hidden_chars += hidden;
    }
}

#[derive(Clone, Copy)]
enum Style {
    Plain,
    Removed,
    Added,
}

/// Whether `c` would not show as itself: control characters, Unicode format
/// characters (zero-width, bidi overrides and isolates, BOM, tags) and other
/// blank fillers.
fn is_hidden(c: char) -> bool {
    c.is_control()
        || matches!(
            c,
            '\u{00AD}'
                | '\u{034F}'
                | '\u{0600}'..='\u{0605}'
                | '\u{061C}'
                | '\u{06DD}'
                | '\u{070F}'
                | '\u{08E2}'
                | '\u{115F}'
                | '\u{1160}'
                | '\u{17B4}'
                | '\u{17B5}'
                | '\u{180E}'
                | '\u{2000}'..='\u{200F}'
                | '\u{2028}'..='\u{202F}'
                | '\u{205F}'..='\u{206F}'
                | '\u{2800}'
                | '\u{3000}'
                | '\u{3164}'
                | '\u{FE00}'..='\u{FE0F}'
                | '\u{FEFF}'
                | '\u{FFA0}'
                | '\u{FFF9}'..='\u{FFFB}'
                | '\u{1D173}'..='\u{1D17A}'
                | '\u{E0000}'..='\u{E0FFF}'
        )
}

/// `text` with hidden characters escaped (`\u{202e}`, `\t`) and long runs of
/// spaces counted, in pieces that say whether they are escapes.
pub(crate) fn escape(text: &str) -> Vec<(String, bool)> {
    let mut pieces: Vec<(String, bool)> = Vec::new();
    let mut push = |piece: String, special: bool| match pieces.last_mut() {
        Some((last, last_special)) if *last_special == special && !special => last.push_str(&piece),
        _ => pieces.push((piece, special)),
    };
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == ' ' {
            let mut run = 1;
            while chars.next_if_eq(&' ').is_some() {
                run += 1;
            }
            if run >= SPACE_RUN_MIN {
                push(format!("[{run} spaces]"), true);
            } else {
                push(" ".repeat(run), false);
            }
        } else if is_hidden(c) {
            let escaped = match c {
                '\t' => "\\t".to_string(),
                '\r' => "\\r".to_string(),
                _ => format!("\\u{{{:x}}}", c as u32),
            };
            push(escaped, true);
        } else {
            push(c.to_string(), false);
        }
    }
    pieces
}

/// [`escape`] as one string.
pub(crate) fn escape_plain(text: &str) -> String {
    escape(text).into_iter().map(|(piece, _)| piece).collect()
}

#[cfg(test)]
#[path = "approval_tests.rs"]
mod tests;
