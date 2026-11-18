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
    let mut inputs = Vec::new();
    let mut stack = vec![tree.root_node()];
    while let Some(node) = stack.pop() {
        if node.kind() == "command" {
            let mut input = None;
            let mut child = node;
            while let Some(parent) = child.parent() {
                // A pipeline after a here-doc delimiter is nested inside the
                // redirect by this grammar; its first node is still a receiver.
                if parent.kind() == "pipeline"
                    && (parent.named_child(0) != Some(child)
                        || parent
                            .parent()
                            .is_some_and(|node| node.kind() == "heredoc_redirect"))
                {
                    input = Some(None);
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
                        let literal = if input.is_none() && redirects.len() == 1 {
                            literal_heredoc(redirects[0], script)
                        } else {
                            None
                        };
                        input = Some(literal);
                    }
                }
                child = parent;
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
    inputs
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
