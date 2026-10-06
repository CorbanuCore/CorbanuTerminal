//! PF-23-S01: the typed protected surface at the Core tool dispatch boundary.
//!
//! Every tool route a model can call is listed here with how a protected
//! action on it is checked once the session holds untrusted content (PF-30):
//! through the shared approval seam (`tools/orchestrator.rs`), by a check of
//! its own (MCP tool calls, typing into a running process), only through the
//! tools it calls (code mode), or not at all because it reaches no protected
//! resource. A route missing from the list is unclassified and needs a fresh
//! human approval after untrusted content, refused when approvals are off.
//!
//! Like PF-30-S03, none of this relaxes anything: without `source_envelopes`
//! and a protected level, or before untrusted content, nothing changes.

use crate::security::tainted_action::ProtectedActionKind;
use crate::security::tainted_action::classify;
use crate::tools::sandboxing::ApprovalAction;
use codex_tools::ToolName;
use codex_utils_path_uri::PathUri;
use serde_json::Value;
use std::path::Path;

mod gate;

pub(crate) use gate::Admission;
pub(crate) use gate::Route;
pub(crate) use gate::admit;
pub(crate) use gate::ask_human;
pub(crate) use gate::check_dispatch;

/// Who provides a tool. Only first-party code says `Builtin` or `Extension`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ToolOrigin {
    /// A Core tool handler.
    Builtin,
    /// A first-party extension crate (`ext/`).
    Extension,
    /// An MCP server tool.
    Mcp,
    /// A tool the client defined for this thread.
    Dynamic,
}

/// How a protected action on a route is checked after untrusted content.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Coverage {
    /// Shell, exec and patch tools: the shared approval seam classifies the
    /// exact command or patch (PF-30-S03).
    ApprovalSeam,
    /// Checked by the route itself: MCP tool calls, `write_stdin`.
    OwnCheck,
    /// Code mode: the cell has no host access of its own; each tool it calls
    /// is dispatched and checked like a direct call.
    NestedCallsOnly,
    /// Hands work to another agent, which inherits the taint and the level.
    ChildAgent,
    /// Changes what the agent may do (permissions, plugins): asks the human
    /// itself unless an automatic reviewer would answer.
    PolicyRequest,
    /// Reads, reports or waits; reaches no protected resource.
    NoProtectedEffect,
    /// Not on this list: needs a fresh human approval after untrusted content.
    Unclassified,
}

const APPROVAL_SEAM: &[&str] = &[
    "shell",
    "shell_command",
    "exec_command",
    "apply_patch",
    "structured_edit",
    "structured_write",
];
const CHILD_AGENT: &[&str] = &[
    "spawn_agent",
    "spawn_agent_plaintext",
    "send_input",
    "send_message",
    "send_message_plaintext",
    "followup_task",
    "followup_task_plaintext",
    "close_agent",
    "resume_agent",
    "interrupt_agent",
    "list_agents",
    "wait_agent",
];
const NO_PROTECTED_EFFECT: &[&str] = &[
    "update_plan",
    "request_user_input",
    "view_image",
    "sleep",
    "curr_time",
    "get_context_remaining",
    "new_context",
    "tool_search",
    "list_mcp_resources",
    "list_mcp_resource_templates",
    "read_mcp_resource",
    "wait_for_environment",
    "list_available_plugins_to_install",
    "test_sync_tool",
];
/// First-party extension tools, by namespace and name.
const EXTENSION_NO_PROTECTED_EFFECT: &[(Option<&str>, &str)] = &[
    (None, "get_goal"),
    (None, "create_goal"),
    (None, "update_goal"),
    (Some("web"), "run"),
    (Some("image_gen"), "imagegen"),
    (Some("memories"), "list"),
    (Some("memories"), "read"),
    (Some("memories"), "search"),
    // A note keeps its source; recalled memory is never trusted (PF-30-S02).
    (Some("memories"), "add_ad_hoc_note"),
    (Some("skills"), "list"),
    (Some("skills"), "read"),
];

/// The route matrix.
pub(crate) fn coverage(origin: ToolOrigin, name: &ToolName) -> Coverage {
    let plain = name.name.as_str();
    match origin {
        ToolOrigin::Mcp => Coverage::OwnCheck,
        ToolOrigin::Dynamic => Coverage::Unclassified,
        ToolOrigin::Extension => {
            let listed = EXTENSION_NO_PROTECTED_EFFECT
                .iter()
                .any(|(namespace, tool)| *namespace == name.namespace.as_deref() && *tool == plain);
            if listed {
                Coverage::NoProtectedEffect
            } else {
                Coverage::Unclassified
            }
        }
        ToolOrigin::Builtin => match plain {
            _ if APPROVAL_SEAM.contains(&plain) => Coverage::ApprovalSeam,
            "write_stdin" => Coverage::OwnCheck,
            "exec" | "wait" => Coverage::NestedCallsOnly,
            "request_permissions" | "request_plugin_install" => Coverage::PolicyRequest,
            _ if CHILD_AGENT.contains(&plain) => Coverage::ChildAgent,
            _ if NO_PROTECTED_EFFECT.contains(&plain) => Coverage::NoProtectedEffect,
            _ => Coverage::Unclassified,
        },
    }
}

/// One MCP tool call, as the host will send it.
pub(crate) struct McpCall<'a> {
    pub(crate) server: &'a str,
    pub(crate) tool: &'a str,
    pub(crate) title: Option<&'a str>,
    /// The tool may change or send data: it is not marked read-only and is
    /// marked (or not ruled out as) destructive or open-world.
    pub(crate) acts_outside: bool,
    pub(crate) arguments: Option<&'a Value>,
    pub(crate) cwd: &'a PathUri,
}

/// Argument keys whose string value is a command line.
const COMMAND_KEYS: &[&str] = &[
    "command",
    "cmd",
    "script",
    "shell",
    "commandline",
    "command_line",
];
/// Tool-name words that move value on their own.
const TRANSFER_WORDS: &[&str] = &[
    "transfer",
    "pay",
    "payment",
    "payout",
    "swap",
    "withdraw",
    "withdrawal",
    "purchase",
    "buy",
    "sell",
    "trade",
    "remit",
];
/// What `send` moves when it is a value transfer (`send_sol`, `sendTokens`).
const SENT_VALUE_WORDS: &[&str] = &[
    "sol",
    "eth",
    "btc",
    "usdc",
    "usdt",
    "token",
    "tokens",
    "funds",
    "money",
    "crypto",
    "coin",
    "coins",
    "lamports",
    "transaction",
];
/// Argument strings read from one call; more fail closed.
const MAX_ARGUMENT_STRINGS: usize = 1024;
const MAX_ARGUMENT_DEPTH: usize = 16;

/// The protected kind of an MCP tool call. The tool's own description of
/// itself can only add protection: a read-only tool is still judged by the
/// paths and commands in its arguments.
pub(crate) fn classify_mcp(call: &McpCall<'_>, codex_home: &Path) -> Option<ProtectedActionKind> {
    let names = [Some(call.tool), call.title].into_iter().flatten();
    let words: Vec<String> = names.flat_map(name_words).collect();
    let sends_value = words.iter().any(|word| word == "send")
        && words
            .iter()
            .any(|word| SENT_VALUE_WORDS.contains(&word.as_str()));
    if sends_value
        || words
            .iter()
            .any(|word| TRANSFER_WORDS.contains(&word.as_str()))
    {
        return Some(ProtectedActionKind::ValueTransfer);
    }
    let mut found = call.acts_outside.then_some(ProtectedActionKind::Disclosure);
    let mut strings = Vec::new();
    if let Some(arguments) = call.arguments
        && !argument_strings(arguments, /*key*/ None, /*depth*/ 0, &mut strings)
    {
        return strongest(found, Some(ProtectedActionKind::UnseenCode));
    }
    let exec = |command: Vec<String>| ApprovalAction::ExecCommand {
        id: String::new(),
        environment_id: call.server.to_string(),
        command,
        cwd: call.cwd.clone(),
        sandbox_permissions: crate::sandboxing::SandboxPermissions::UseDefault,
        additional_permissions: None,
        justification: None,
        tty: false,
    };
    let mut paths = vec!["mcp-arguments".to_string()];
    for (key, text) in strings {
        if key.is_some_and(|key| COMMAND_KEYS.contains(&key.to_lowercase().as_str())) {
            let command = vec!["sh".to_string(), "-c".to_string(), text.to_string()];
            found = strongest(found, classify(&exec(command), codex_home));
        } else if text.contains('/') || text.starts_with(['~', '.', '$']) {
            paths.push(text.to_string());
        }
    }
    if paths.len() > 1 {
        found = strongest(found, classify(&exec(paths), codex_home));
    }
    found
}

/// The protected kind of typing `chars` into a running process started as
/// `process_command`: that of the process itself, or of the text read as
/// code by the program it most likely is (a shell unless it is a known
/// interpreter).
pub(crate) fn classify_typed_input(
    process_command: &str,
    chars: &str,
    cwd: &PathUri,
    codex_home: &Path,
) -> Option<ProtectedActionKind> {
    let exec = |command: Vec<String>| ApprovalAction::ExecCommand {
        id: String::new(),
        environment_id: String::new(),
        command,
        cwd: cwd.clone(),
        sandbox_permissions: crate::sandboxing::SandboxPermissions::UseDefault,
        additional_permissions: None,
        justification: None,
        tty: true,
    };
    let sh = |text: &str| vec!["sh".to_string(), "-c".to_string(), text.to_string()];
    let process = classify(&exec(sh(process_command)), codex_home);
    let program = shlex::split(process_command)
        .and_then(|words| words.into_iter().next())
        .map(|word| word.rsplit('/').next().unwrap_or_default().to_lowercase())
        .unwrap_or_default();
    let (interpreter, flag) = match program.as_str() {
        name if name.starts_with("python") || name == "ipython" => ("python3", "-c"),
        "node" | "deno" | "bun" => ("node", "-e"),
        "ruby" | "irb" => ("ruby", "-e"),
        "perl" => ("perl", "-e"),
        _ => ("sh", "-c"),
    };
    let typed = vec![interpreter.to_string(), flag.to_string(), chars.to_string()];
    strongest(process, classify(&exec(typed), codex_home))
}

fn strongest(
    left: Option<ProtectedActionKind>,
    right: Option<ProtectedActionKind>,
) -> Option<ProtectedActionKind> {
    match (left, right) {
        (Some(left), Some(right)) => Some(left.strongest(right)),
        (left, right) => left.or(right),
    }
}

/// Lowercase words of a tool name: split at punctuation and camel case.
fn name_words(name: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();
    let mut previous_lower = false;
    for ch in name.chars() {
        if (!ch.is_alphanumeric() || (ch.is_uppercase() && previous_lower))
            && !current.is_empty() {
                words.push(std::mem::take(&mut current));
            }
        if ch.is_alphanumeric() {
            current.extend(ch.to_lowercase());
        }
        previous_lower = ch.is_lowercase() || ch.is_ascii_digit();
    }
    if !current.is_empty() {
        words.push(current);
    }
    words
}

/// Every string in `value` with the object key it sits under. False when the
/// value is too large or too deep to read in full.
fn argument_strings<'a>(
    value: &'a Value,
    key: Option<&'a str>,
    depth: usize,
    out: &mut Vec<(Option<&'a str>, &'a str)>,
) -> bool {
    if depth > MAX_ARGUMENT_DEPTH || out.len() > MAX_ARGUMENT_STRINGS {
        return false;
    }
    match value {
        Value::String(text) => {
            out.push((key, text));
            out.len() <= MAX_ARGUMENT_STRINGS
        }
        Value::Array(items) => items
            .iter()
            .all(|item| argument_strings(item, key, depth + 1, out)),
        Value::Object(map) => map.iter().all(|(name, item)| {
            out.push((None, name));
            argument_strings(item, Some(name), depth + 1, out)
        }),
        Value::Null | Value::Bool(_) | Value::Number(_) => true,
    }
}

#[cfg(test)]
#[path = "protected_surface_tests.rs"]
mod tests;
