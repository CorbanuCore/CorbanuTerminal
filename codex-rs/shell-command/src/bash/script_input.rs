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
/// by a command in a group. `eval_context` preserves the execution-text boundary
/// while reading an eval body. A syntax error is handled by the script reader.
pub fn shell_lc_script_inputs(
    command: &[String],
    eval_context: bool,
    is_shell: impl Fn(&str) -> bool,
) -> Vec<ScriptInput> {
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
    let mut precise = true;
    let mut interprets = eval_context;
    let mut function = false;
    let mut heredoc = false;
    let mut stack = vec![tree.root_node()];
    while let Some(node) = stack.pop() {
        precise &= precise_input_node(node);
        function |= node.kind() == "function_definition";
        heredoc |= node.kind() == "heredoc_redirect";
        if node.kind() == "command" {
            let words = script_command_words(node, script);
            interprets |= words
                .iter()
                .flatten()
                .any(|word| matches!(word.as_str(), "eval" | "exec" | "coproc" | "." | "source"));
            // Only these direct commands have understood stdin semantics here.
            // Launchers and all other prefixes retain an unknown-script floor.
            precise &= words
                .first()
                .and_then(Option::as_deref)
                .is_some_and(|word| {
                    matches!(
                        word.rsplit('/').next(),
                        Some(
                            "bash"
                                | "sh"
                                | "dash"
                                | "ash"
                                | "ksh"
                                | "zsh"
                                | "eval"
                                | "exec"
                                | "echo"
                                | "printf"
                                | "cat"
                                | "git"
                        )
                    )
                });
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
        stack.extend(node.children(&mut cursor));
    }
    // Precise handling is opt-in. Unsupported syntax or prefixes in a script
    // with execution text cannot silently erase a possible stdin recipient.
    // Eval/exec may also change how later commands receive even literal input.
    let shell_input = inputs
        .iter()
        .any(|input| input.words.iter().flatten().any(|word| is_shell(word)));
    if ((heredoc || shell_input || interprets || function) && !precise)
        || (!inputs.is_empty() && interprets)
    {
        // Prefixes can forward stdin without an identifiable receiver node.
        inputs.push(ScriptInput {
            words: vec![None],
            literal: None,
        });
        for node in commands {
            let input = ScriptInput {
                words: script_command_words(node, script),
                literal: None,
            };
            if !inputs.contains(&input) {
                inputs.push(input);
            }
        }
    }
    inputs
}

/// Static argv, simple lists/pipes and direct redirects only. In particular,
/// functions, loops, substitutions, assignments and reserved prefixes are not
/// precise, even if the grammar exposes their descendants as plain commands.
fn precise_input_node(node: Node<'_>) -> bool {
    const KINDS: &[&str] = &[
        "program",
        "list",
        "pipeline",
        "command",
        "command_name",
        "word",
        "string",
        "string_content",
        "raw_string",
        "number",
        "concatenation",
        "redirected_statement",
        "file_redirect",
        "herestring_redirect",
        "heredoc_redirect",
        "heredoc_start",
        "heredoc_body",
        "heredoc_content",
        "heredoc_end",
        "comment",
    ];
    const TOKENS: &[&str] = &[
        "&&", "||", ";", "|", "\"", "'", "<", ">", "<<", "<<-", "<<<", ">>",
    ];
    if node.is_named() {
        KINDS.contains(&node.kind())
    } else {
        TOKENS.contains(&node.kind())
    }
}

/// The scanner only removes quotes reliably for simple whole-word quoting.
/// A mixed delimiter can swallow real outer commands into a fictitious body.
/// Reject the entire script, including non-shell here-documents, in that case.
pub(super) fn heredoc_boundaries_readable(root: Node<'_>, script: &str) -> bool {
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        // Bash joins escaped newlines before recognizing an unquoted end
        // marker. The scanner can miss that end and swallow outer commands.
        // It also omits argv placed after a here-document start from commands.
        if (node.kind() == "heredoc_body"
            && node
                .utf8_text(script.as_bytes())
                .is_ok_and(|text| text.contains("\\\n")))
            || (node.kind() == "heredoc_redirect" && node.child_by_field_name("argument").is_some())
        {
            return false;
        }
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
        || redirect.child_by_field_name("argument").is_some()
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
