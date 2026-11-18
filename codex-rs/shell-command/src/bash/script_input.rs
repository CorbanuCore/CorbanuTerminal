//! Stdin attached to commands in a shell script. Never read files or evaluate
//! producers: only an unexpanded here-document can supply known script text.

use super::script_command_words;
use super::try_parse_shell;
use crate::bash::extract_bash_command;
use tree_sitter::Node;

/// Input supplied to a command, after quote removal of its argv.
#[derive(Debug, PartialEq, Eq)]
pub struct ScriptInput {
    pub words: Vec<Option<String>>,
    /// None means a pipe, file, expansion, or ambiguous redirection.
    pub literal: Option<String>,
}

/// Find stdin supplied by a pipeline or redirection, including input inherited
/// by a command in a group. A syntax error is handled by the script reader.
pub fn shell_lc_script_inputs(command: &[String]) -> Vec<ScriptInput> {
    let Some((_, script)) = extract_bash_command(command) else {
        return Vec::new();
    };
    let Some(tree) = try_parse_shell(script) else {
        return Vec::new();
    };
    if !heredoc_boundaries_readable(tree.root_node(), script) {
        return Vec::new();
    }
    let mut inputs = Vec::new();
    let mut commands = Vec::new();
    let mut functions = Vec::new();
    let mut stack = vec![tree.root_node()];
    while let Some(node) = stack.pop() {
        if node.kind() == "function_definition"
            && let Some(name) = node.child_by_field_name("name")
            && let Ok(name) = name.utf8_text(script.as_bytes())
        {
            functions.push(name);
        }
        if node.kind() == "command" {
            commands.push(node);
            let mut input = None;
            let mut child = node;
            let mut ancestor = Some(node);
            while let Some(parent) = ancestor {
                // A pipeline after a here-doc delimiter is nested inside the
                // redirect by this grammar; its first node is still a receiver.
                if parent.kind() == "pipeline"
                    && (parent.named_child(0) != Some(child)
                        || parent
                            .parent()
                            .is_some_and(|node| node.kind() == "heredoc_redirect"))
                {
                    input.get_or_insert(None);
                }
                if matches!(parent.kind(), "redirected_statement" | "command") {
                    let mut cursor = parent.walk();
                    let redirects: Vec<_> = parent
                        .named_children(&mut cursor)
                        .filter(|n| {
                            matches!(n.kind(), "heredoc_redirect" | "herestring_redirect")
                                || (n.kind() == "file_redirect"
                                    && n.utf8_text(script.as_bytes())
                                        .is_ok_and(|text| text.contains('<')))
                        })
                        .collect();
                    if !redirects.is_empty() {
                        // A group/loop/substitution can consume input before
                        // this command sees it. Only its own redirect is literal.
                        let direct =
                            parent == node || parent.child_by_field_name("body") == Some(node);
                        let literal = if direct && input.is_none() && redirects.len() == 1 {
                            literal_heredoc(redirects[0], script)
                        } else {
                            None
                        };
                        // An outer group's redirect cannot replace this
                        // command's own literal input.
                        if direct || !matches!(input, Some(Some(_))) {
                            input = Some(literal);
                        }
                    }
                }
                child = parent;
                ancestor = parent.parent();
            }
            if let Some(literal) = input {
                inputs.push(ScriptInput {
                    words: script_command_words(node, script),
                    literal,
                });
            }
        }
        let mut cursor = node.walk();
        stack.extend(node.named_children(&mut cursor));
    }
    // An exec redirect can change the outer shell's stdin for later
    // commands. A function call forwards its input to commands in its body.
    // Without executing the script, conservatively taint all its commands.
    let inherited = inputs.iter().any(|input| {
        input_target(&input.words)
            .is_some_and(|program| program == "exec" || functions.contains(&program))
    }) || (!inputs.is_empty()
        && commands.iter().any(|node| {
            // Eval can define a function used by a later input recipient.
            input_target(&script_command_words(*node, script)) == Some("eval")
        }));
    if inherited {
        inputs.extend(commands.into_iter().map(|node| ScriptInput {
            words: script_command_words(node, script),
            literal: None,
        }));
    }
    inputs
}

fn input_target(mut words: &[Option<String>]) -> Option<&str> {
    while words
        .first()
        .is_some_and(|word| matches!(word.as_deref(), Some("command" | "builtin")))
    {
        words = &words[1..];
        while words
            .first()
            .is_some_and(|word| matches!(word.as_deref(), Some("--" | "-p")))
        {
            words = &words[1..];
        }
    }
    words.first().and_then(Option::as_deref)
}

/// The scanner only removes quotes reliably for simple whole-word quoting.
/// A mixed delimiter can swallow real outer commands into a fictitious body.
/// Reject the entire script, including non-shell here-documents, in that case.
pub(super) fn heredoc_boundaries_readable(root: Node<'_>, script: &str) -> bool {
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        if node.kind() == "heredoc_start" {
            let Ok(text) = node.utf8_text(script.as_bytes()) else {
                return false;
            };
            let word = text
                .strip_prefix('\'')
                .and_then(|s| s.strip_suffix('\''))
                .or_else(|| text.strip_prefix('"').and_then(|s| s.strip_suffix('"')))
                .or_else(|| text.strip_prefix('\\'))
                .unwrap_or(text);
            if word.is_empty()
                || !word
                    .bytes()
                    .all(|ch| ch.is_ascii_alphanumeric() || ch == b'_')
            {
                return false;
            }
            let Some(redirect) = node.parent() else {
                return false;
            };
            let mut cursor = redirect.walk();
            let Some(end) = redirect
                .named_children(&mut cursor)
                .find(|child| child.kind() == "heredoc_end")
            else {
                return false;
            };
            let before = script[..end.start_byte()].rsplit('\n').next().unwrap_or("");
            let after = script[end.end_byte()..].split('\n').next().unwrap_or("");
            let strip_tabs = script[redirect.start_byte()..node.start_byte()].contains("<<-");
            if end.utf8_text(script.as_bytes()).ok() != Some(word)
                || !after.is_empty()
                || !(before.is_empty() || (strip_tabs && before.bytes().all(|ch| ch == b'\t')))
            {
                return false;
            }
        }
        let mut cursor = node.walk();
        stack.extend(node.named_children(&mut cursor));
    }
    true
}

fn literal_heredoc(redirect: Node<'_>, script: &str) -> Option<String> {
    if redirect.kind() != "heredoc_redirect"
        || redirect.child_by_field_name("descriptor").is_some()
        || redirect.child_by_field_name("redirect").is_some()
        || redirect.child_by_field_name("right").is_some()
    {
        return None;
    }
    let mut cursor = redirect.walk();
    let start = redirect
        .named_children(&mut cursor)
        .find(|n| n.kind() == "heredoc_start")?;
    let mut cursor = redirect.walk();
    let body = redirect
        .named_children(&mut cursor)
        .find(|n| n.kind() == "heredoc_body")?;
    let delimiter = start.utf8_text(script.as_bytes()).ok()?;
    let text = body.utf8_text(script.as_bytes()).ok()?;
    let quoted = delimiter.contains(['\'', '"', '\\']);
    // Unquoted here-documents perform expansions and backslash processing.
    // Do not mistake their source text for the bytes the shell will read.
    if !quoted && text.contains(['$', '`', '\\']) {
        return None;
    }
    let prefix = &script[redirect.start_byte()..start.start_byte()];
    Some(if prefix.contains("<<-") {
        text.split_inclusive('\n')
            .map(|line| line.trim_start_matches('\t'))
            .collect()
    } else {
        text.to_string()
    })
}
