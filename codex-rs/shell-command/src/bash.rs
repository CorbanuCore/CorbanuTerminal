use std::path::PathBuf;

use tree_sitter::Node;
use tree_sitter::Parser;
use tree_sitter::Tree;
use tree_sitter_bash::LANGUAGE as BASH;

use crate::shell_detect::ShellType;
use crate::shell_detect::detect_shell_type;

mod script_input;
pub use script_input::shell_lc_script_inputs;
#[cfg(test)]
#[path = "bash/script_input_tests.rs"]
mod script_input_tests;

/// Parse the provided bash source using tree-sitter-bash, returning a Tree on
/// success or None if parsing failed.
pub fn try_parse_shell(shell_lc_arg: &str) -> Option<Tree> {
    let lang = BASH.into();
    let mut parser = Parser::new();
    #[expect(clippy::expect_used)]
    parser.set_language(&lang).expect("load bash grammar");
    let old_tree: Option<&Tree> = None;
    parser.parse(shell_lc_arg, old_tree)
}

/// The grammar's node type, so callers can walk a [`Tree`] without a
/// tree-sitter dependency of their own.
pub use tree_sitter::Node as SyntaxNode;

/// Why [`try_parse_shell_bounded`] stopped without a tree.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParseStop {
    /// Every parse in progress had met a syntax error. Error recovery can
    /// take time quadratic in the text, so it is not attempted.
    SyntaxError,
    /// The parse used more than its progress checks.
    Budget,
}

/// [`try_parse_shell`] for text that must be parsed in bounded time: it
/// stops at the first syntax error every parse in progress has met, or once
/// tree-sitter has checked its progress more than `max_checks` times (it
/// checks about once every 100 parse operations). Also returns the number
/// of checks made. The bounds count work rather than time, so the same text
/// always gets the same answer. A tree it returns may still hold an error
/// (one met between two checks).
pub fn try_parse_shell_bounded(
    script: &str,
    max_checks: usize,
) -> (Result<Tree, ParseStop>, usize) {
    let lang = BASH.into();
    let mut parser = Parser::new();
    #[expect(clippy::expect_used)]
    parser.set_language(&lang).expect("load bash grammar");
    let mut checks = 0usize;
    let mut stop = ParseStop::Budget;
    let mut progress = |state: &tree_sitter::ParseState| {
        checks += 1;
        if state.has_error() {
            stop = ParseStop::SyntaxError;
            return true;
        }
        checks > max_checks
    };
    let bytes = script.as_bytes();
    let tree = parser.parse_with_options(
        &mut |offset, _| bytes.get(offset..).unwrap_or_default(),
        /*old_tree*/ None,
        Some(tree_sitter::ParseOptions::new().progress_callback(&mut progress)),
    );
    (tree.ok_or(stop), checks)
}

/// Parse a script which may contain multiple simple commands joined only by
/// the safe logical/pipe/sequencing operators: `&&`, `||`, `;`, `|`.
///
/// Returns `Some(Vec<command_words>)` if every command is a plain word‑only
/// command and the parse tree does not contain disallowed constructs
/// (parentheses, redirections, substitutions, control flow, etc.). Otherwise
/// returns `None`.
pub fn try_parse_word_only_commands_sequence(tree: &Tree, src: &str) -> Option<Vec<Vec<String>>> {
    let mut commands = Vec::new();
    for node in word_only_command_nodes(tree)? {
        commands.push(parse_plain_command_from_node(node, src)?);
    }
    Some(commands)
}

/// The command nodes of a "word only commands sequence" (see
/// [`try_parse_word_only_commands_sequence`]) in source order, before their
/// words are read; `None` for any other script.
fn word_only_command_nodes(tree: &Tree) -> Option<Vec<Node<'_>>> {
    if tree.root_node().has_error() {
        return None;
    }

    // List of allowed (named) node kinds for a "word only commands sequence".
    // If we encounter a named node that is not in this list we reject.
    const ALLOWED_KINDS: &[&str] = &[
        // top level containers
        "program",
        "list",
        "pipeline",
        // commands & words
        "command",
        "command_name",
        "word",
        "string",
        "string_content",
        "raw_string",
        "number",
        "concatenation",
    ];
    // Allow only safe punctuation / operator tokens; anything else causes reject.
    const ALLOWED_PUNCT_TOKENS: &[&str] = &["&&", "||", ";", "|", "\"", "'"];

    let root = tree.root_node();
    let mut cursor = root.walk();
    let mut stack = vec![root];
    let mut command_nodes = Vec::new();
    while let Some(node) = stack.pop() {
        let kind = node.kind();
        if node.is_named() {
            if !ALLOWED_KINDS.contains(&kind) {
                return None;
            }
            if kind == "command" {
                command_nodes.push(node);
            }
        } else {
            // Reject any punctuation / operator tokens that are not explicitly allowed.
            if kind.chars().any(|c| "&;|".contains(c)) && !ALLOWED_PUNCT_TOKENS.contains(&kind) {
                return None;
            }
            if !(ALLOWED_PUNCT_TOKENS.contains(&kind) || kind.trim().is_empty()) {
                // If it's a quote token or operator it's allowed above; we also allow whitespace tokens.
                // Any other punctuation like parentheses, braces, redirects, backticks, etc are rejected.
                return None;
            }
        }
        for child in node.children(&mut cursor) {
            stack.push(child);
        }
    }

    // Walk uses a stack (LIFO), so re-sort by position to restore source order.
    command_nodes.sort_by_key(Node::start_byte);
    Some(command_nodes)
}

/// Parses a shell script consisting only of plain commands joined by safe operators.
pub fn parse_shell_script_into_commands(script: &str) -> Option<Vec<Vec<String>>> {
    let tree = try_parse_shell(script)?;
    try_parse_word_only_commands_sequence(&tree, script)
}

/// A word of a plain command (see [`parse_shell_script_into_plain_words`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlainWord {
    /// The word as [`parse_shell_script_into_commands`] returns it.
    pub text: String,
    /// Some of it is outside quotes, where bash still applies brace
    /// expansion, pathname expansion and backslash removal.
    pub unquoted: bool,
}

/// [`parse_shell_script_into_commands`], telling which words have parts
/// outside quotes.
pub fn parse_shell_script_into_plain_words(script: &str) -> Option<Vec<Vec<PlainWord>>> {
    let tree = try_parse_shell(script)?;
    let commands = try_parse_word_only_commands_sequence(&tree, script)?;
    // The same command nodes, in the same order, as that parse.
    let mut command_nodes = Vec::new();
    let mut stack = vec![tree.root_node()];
    while let Some(node) = stack.pop() {
        if node.kind() == "command" {
            command_nodes.push(node);
        }
        let mut cursor = node.walk();
        stack.extend(node.children(&mut cursor));
    }
    command_nodes.sort_by_key(Node::start_byte);
    if command_nodes.len() != commands.len() {
        return None;
    }
    let mut plain = Vec::with_capacity(commands.len());
    for (node, words) in command_nodes.into_iter().zip(commands) {
        let mut cursor = node.walk();
        let unquoted: Vec<bool> = node
            .named_children(&mut cursor)
            .map(|child| match child.kind() {
                "string" | "raw_string" => false,
                "concatenation" => {
                    let mut parts = child.walk();
                    child
                        .named_children(&mut parts)
                        .any(|part| matches!(part.kind(), "word" | "number"))
                }
                _ => true,
            })
            .collect();
        if unquoted.len() != words.len() {
            return None;
        }
        plain.push(
            words
                .into_iter()
                .zip(unquoted)
                .map(|(text, unquoted)| PlainWord { text, unquoted })
                .collect(),
        );
    }
    Some(plain)
}

pub fn extract_bash_command(command: &[String]) -> Option<(&str, &str)> {
    let [shell, flag, script] = command else {
        return None;
    };
    if !matches!(flag.as_str(), "-lc" | "-c")
        || !matches!(
            detect_shell_type(PathBuf::from(shell)),
            Some(ShellType::Zsh) | Some(ShellType::Bash) | Some(ShellType::Sh)
        )
    {
        return None;
    }
    Some((shell, script))
}

/// Returns the sequence of plain commands within a `bash -lc "..."` or
/// `zsh -lc "..."` invocation when the script only contains word-only commands
/// joined by safe operators.
pub fn parse_shell_lc_plain_commands(command: &[String]) -> Option<Vec<Vec<String>>> {
    let (_, script) = extract_bash_command(command)?;
    parse_shell_script_into_commands(script)
}

/// #2543: what command rules can know of a shell script before it runs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScriptCommands {
    /// Plain commands joined only by `&&`, `||`, `;` and `|` (see
    /// [`parse_shell_script_into_commands`]), each word as bash passes it
    /// on once quotes are removed (`'git' push`, `g"i"t push`).
    Plain(Vec<Vec<String>>),
    /// Any other script: every command in it in source order, including
    /// those in substitutions, control flow and function bodies, with
    /// assignments and redirections left out. A word bash builds only as the
    /// script runs (expansions, globs, braces, `~`) is `None`; it may also
    /// become no word or several.
    Partial(Vec<Vec<Option<String>>>),
    /// The script does not parse.
    Unreadable,
}

/// [`read_script_commands`] for a `bash -lc "..."`, `sh -c "..."` or
/// `zsh -lc "..."` invocation; `None` for any other command.
pub fn shell_lc_script_commands(command: &[String]) -> Option<ScriptCommands> {
    let (_, script) = extract_bash_command(command)?;
    Some(read_script_commands(script))
}

/// #2543: the commands of `script` as far as they are known before it runs.
/// Unlike [`parse_shell_script_into_commands`], quoted command names are
/// read (`'git' push` runs `git push`), and a word bash would still change
/// (`g\it`, `/usr/bin/gi?`, `{git,}`) is never read as written.
pub fn read_script_commands(script: &str) -> ScriptCommands {
    let Some(tree) = try_parse_shell(script) else {
        return ScriptCommands::Unreadable;
    };
    let root = tree.root_node();
    if root.has_error() || !script_input::heredoc_boundaries_readable(root, script) {
        return ScriptCommands::Unreadable;
    }
    let mut command_nodes = Vec::new();
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        if node.kind() == "command" {
            command_nodes.push(node);
        }
        let mut cursor = node.walk();
        stack.extend(node.named_children(&mut cursor));
    }
    command_nodes.sort_by_key(Node::start_byte);
    let commands: Vec<Vec<Option<String>>> = command_nodes
        .into_iter()
        .map(|node| script_command_words(node, script))
        .filter(|words| !words.is_empty())
        .collect();
    if word_only_command_nodes(&tree).is_some() && commands.iter().flatten().all(Option::is_some) {
        return ScriptCommands::Plain(
            commands
                .into_iter()
                .map(|words| words.into_iter().flatten().collect())
                .collect(),
        );
    }
    ScriptCommands::Partial(commands)
}

/// The argv words of a command node; assignments, redirections and
/// comments are not argv.
fn script_command_words(command: Node<'_>, src: &str) -> Vec<Option<String>> {
    let mut words = Vec::new();
    let mut cursor = command.walk();
    for child in command.named_children(&mut cursor) {
        match child.kind() {
            "variable_assignment"
            | "variable_assignments"
            | "file_redirect"
            | "heredoc_redirect"
            | "herestring_redirect"
            | "comment" => {}
            "command_name" => {
                words.push(child.named_child(0).and_then(|name| script_word(name, src)))
            }
            _ => words.push(script_word(child, src)),
        }
    }
    words
}

/// The word bash makes of `node` after quote removal, or `None` when the
/// word is built as the script runs.
fn script_word(node: Node<'_>, src: &str) -> Option<String> {
    let text = node.utf8_text(src.as_bytes()).ok()?;
    match node.kind() {
        "word" | "number" if is_literal_word_or_number(node) => unquoted_word(text),
        "raw_string" => parse_raw_string(node, src),
        "string" => {
            parse_double_quoted_string(node, src)?;
            double_quoted_text(text.strip_prefix('"')?.strip_suffix('"')?)
        }
        // A decoded byte above ASCII is not the character it is read as.
        "ansi_c_string" => ansi_c::decode(text).filter(|word| word.is_ascii()),
        "concatenation" => {
            // The grammar splits `gi[t]` and `{a,b}` into several words;
            // look for expansions in their unquoted text as a whole.
            let mut word = String::new();
            let mut unquoted = String::new();
            let mut cursor = node.walk();
            for part in node.named_children(&mut cursor) {
                word.push_str(&script_word(part, src)?);
                match part.kind() {
                    "word" | "number" => unquoted.push_str(part.utf8_text(src.as_bytes()).ok()?),
                    _ => unquoted.push('\0'),
                }
            }
            unquoted_word(&unquoted)?;
            Some(word)
        }
        _ => None,
    }
}

/// An unquoted word after backslash removal; `None` where bash (or zsh or
/// ksh, read alike) may expand it: pathname expansion (`*`, `?`, `[…]`,
/// `@(…)`), brace expansion (`{a,b}`, `{1..3}`), tilde expansion or zsh's
/// `=cmd`. Approximate on the side of `None`.
fn unquoted_word(text: &str) -> Option<String> {
    let mut word = String::new();
    let mut chars = text.chars();
    while let Some(ch) = chars.next() {
        let rest = || chars.clone();
        match ch {
            '\\' => match chars.next() {
                Some('\n') => {}
                Some(escaped) => word.push(escaped),
                None => word.push('\\'),
            },
            '*' | '?' | '~' => return None,
            // zsh's `=cmd` is the program's path.
            '=' if word.is_empty() => return None,
            // ksh's and extglob's `@(…)`, `+(…)`, `!(…)`.
            '@' | '+' | '!' if rest().next() == Some('(') => return None,
            '[' if rest().any(|next| next == ']') => return None,
            '{' if rest().any(|next| matches!(next, ',' | '.'))
                && rest().any(|next| next == '}') =>
            {
                return None;
            }
            _ => word.push(ch),
        }
    }
    Some(word)
}

/// The text between double quotes after bash removes its backslashes:
/// before `$`, `` ` ``, `"`, `\` and a newline only.
fn double_quoted_text(text: &str) -> Option<String> {
    let mut word = String::new();
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            word.push(ch);
            continue;
        }
        match chars.peek() {
            Some('\n') => {
                chars.next();
            }
            Some(&escaped @ ('$' | '`' | '"' | '\\')) => {
                chars.next();
                word.push(escaped);
            }
            _ => word.push('\\'),
        }
    }
    Some(word)
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum LiteralWordMode {
    Verbatim,
    DecodeForForcedRm,
}

/// Extracts the literal portions of command invocations from a shell script.
///
/// Unlike [`parse_shell_lc_plain_commands`], this accepts complex shell syntax
/// and returns the statically known words from every command node in a valid
/// syntax tree. Dynamic words and redirections are omitted. This is suitable
/// for identifying dangerous literal commands, but must not be used to prove
/// that a command is safe.
pub(crate) fn parse_shell_lc_literal_commands(
    command: &[String],
    mode: LiteralWordMode,
) -> Option<Vec<Vec<String>>> {
    let (_, script) = extract_bash_command(command)?;
    let tree = try_parse_shell(script)?;
    let root = tree.root_node();
    if root.has_error() {
        return None;
    }

    let mut commands = Vec::new();
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        if node.kind() == "command"
            && let Some(command) = parse_literal_command_from_node(node, script, mode)
        {
            commands.push(command);
        }

        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            stack.push(child);
        }
    }

    Some(commands)
}

/// Returns the parsed argv for a single shell command in a here-doc style
/// script (`<<`), as long as the script contains exactly one command node.
pub fn parse_shell_lc_single_command_prefix(command: &[String]) -> Option<Vec<String>> {
    let (_, script) = extract_bash_command(command)?;
    let tree = try_parse_shell(script)?;
    let root = tree.root_node();
    if root.has_error() || !script_input::heredoc_boundaries_readable(root, script) {
        return None;
    }
    if !has_named_descendant_kind(root, "heredoc_redirect") {
        return None;
    }
    if has_named_descendant_kind(root, "file_redirect") {
        return None;
    }

    let command_node = find_single_command_node(root)?;
    parse_heredoc_command_words(command_node, script)
}

fn parse_plain_command_from_node(cmd: tree_sitter::Node, src: &str) -> Option<Vec<String>> {
    if cmd.kind() != "command" {
        return None;
    }
    let mut words = Vec::new();
    let mut cursor = cmd.walk();
    for child in cmd.named_children(&mut cursor) {
        match child.kind() {
            "command_name" => {
                let word_node = child.named_child(0)?;
                if word_node.kind() != "word" {
                    return None;
                }
                words.push(word_node.utf8_text(src.as_bytes()).ok()?.to_owned());
            }
            "word" | "number" => {
                words.push(child.utf8_text(src.as_bytes()).ok()?.to_owned());
            }
            "string" => {
                let parsed = parse_double_quoted_string(child, src)?;
                words.push(parsed);
            }
            "raw_string" => {
                let parsed = parse_raw_string(child, src)?;
                words.push(parsed);
            }
            "concatenation" => {
                // Handle concatenated arguments like -g"*.py"
                let mut concatenated = String::new();
                let mut concat_cursor = child.walk();
                for part in child.named_children(&mut concat_cursor) {
                    match part.kind() {
                        "word" | "number" => {
                            concatenated
                                .push_str(part.utf8_text(src.as_bytes()).ok()?.to_owned().as_str());
                        }
                        "string" => {
                            let parsed = parse_double_quoted_string(part, src)?;
                            concatenated.push_str(&parsed);
                        }
                        "raw_string" => {
                            let parsed = parse_raw_string(part, src)?;
                            concatenated.push_str(&parsed);
                        }
                        _ => return None,
                    }
                }
                if concatenated.is_empty() {
                    return None;
                }
                words.push(concatenated);
            }
            _ => return None,
        }
    }
    Some(words)
}

fn parse_literal_command_from_node(
    cmd: Node<'_>,
    src: &str,
    mode: LiteralWordMode,
) -> Option<Vec<String>> {
    if cmd.kind() != "command" {
        return None;
    }

    let mut words = Vec::new();
    let mut found_command_name = false;
    let mut cursor = cmd.walk();
    for child in cmd.named_children(&mut cursor) {
        if child.kind() == "command_name" {
            let command_name = parse_literal_shell_word(child.named_child(0)?, src, mode)?;
            words.push(command_name);
            found_command_name = true;
        } else if found_command_name && let Some(word) = parse_literal_shell_word(child, src, mode)
        {
            words.push(word);
        }
    }

    found_command_name.then_some(words)
}

#[path = "bash/ansi_c.rs"]
mod ansi_c;

fn parse_literal_shell_word(node: Node<'_>, src: &str, mode: LiteralWordMode) -> Option<String> {
    if mode == LiteralWordMode::DecodeForForcedRm {
        match node.kind() {
            "ansi_c_string" => return ansi_c::decode(node.utf8_text(src.as_bytes()).ok()?),
            "string" => {
                // Reject substitutions before removing shell quote/escape syntax.
                parse_double_quoted_string(node, src)?;
                return shlex::split(node.utf8_text(src.as_bytes()).ok()?)
                    .filter(|words| words.len() == 1)
                    .and_then(|words| words.into_iter().next());
            }
            "translated_string" => {
                // Inspect the literal source message; never load locale catalogs.
                return parse_literal_shell_word(node.named_child(0)?, src, mode);
            }
            "word" | "number" if is_literal_word_or_number(node) => {
                // Tree-sitter has already identified this as a word, including
                // a literal '#' joined to a quoted segment. Do not lex it
                // again as standalone shell source (where '#' is a comment).
                let mut chars = node.utf8_text(src.as_bytes()).ok()?.chars();
                let mut decoded = String::new();
                while let Some(ch) = chars.next() {
                    if ch == '\\' {
                        let escaped = chars.next()?;
                        if escaped != '\n' {
                            decoded.push(escaped);
                        }
                    } else {
                        decoded.push(ch);
                    }
                }
                return Some(decoded);
            }
            _ => {}
        }
    }
    match node.kind() {
        "word" | "number" if is_literal_word_or_number(node) => {
            Some(node.utf8_text(src.as_bytes()).ok()?.to_owned())
        }
        "string" => parse_double_quoted_string(node, src),
        "raw_string" => parse_raw_string(node, src),
        "concatenation" => {
            let mut concatenated = String::new();
            let mut cursor = node.walk();
            for part in node.named_children(&mut cursor) {
                concatenated.push_str(&parse_literal_shell_word(part, src, mode)?);
            }
            (!concatenated.is_empty()).then_some(concatenated)
        }
        _ => None,
    }
}

fn parse_heredoc_command_words(cmd: Node<'_>, src: &str) -> Option<Vec<String>> {
    if cmd.kind() != "command" {
        return None;
    }

    let mut words = Vec::new();
    let mut cursor = cmd.walk();
    for child in cmd.named_children(&mut cursor) {
        match child.kind() {
            "command_name" => {
                let word_node = child.named_child(0)?;
                if !matches!(word_node.kind(), "word" | "number")
                    || !is_literal_word_or_number(word_node)
                {
                    return None;
                }
                words.push(word_node.utf8_text(src.as_bytes()).ok()?.to_owned());
            }
            "word" | "number" => {
                if !is_literal_word_or_number(child) {
                    return None;
                }
                words.push(child.utf8_text(src.as_bytes()).ok()?.to_owned());
            }
            // Allow heredoc constructs that attach stdin to a single command
            // without changing argv matching semantics for the executable
            // prefix. Other file redirects may write outside the sandbox and
            // must not be collapsed to the executable prefix for execpolicy.
            "comment" => {}
            kind if is_allowed_heredoc_attachment_kind(kind) => {}
            _ => return None,
        }
    }

    if words.is_empty() { None } else { Some(words) }
}

fn is_literal_word_or_number(node: Node<'_>) -> bool {
    if !matches!(node.kind(), "word" | "number") {
        return false;
    }
    let mut cursor = node.walk();
    node.named_children(&mut cursor).next().is_none()
}

fn is_allowed_heredoc_attachment_kind(kind: &str) -> bool {
    matches!(
        kind,
        "heredoc_body"
            | "simple_heredoc_body"
            | "heredoc_redirect"
            | "herestring_redirect"
            | "redirected_statement"
    )
}

fn find_single_command_node(root: Node<'_>) -> Option<Node<'_>> {
    let mut stack = vec![root];
    let mut single_command = None;
    while let Some(node) = stack.pop() {
        if node.kind() == "command" {
            if single_command.is_some() {
                return None;
            }
            single_command = Some(node);
        }

        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            stack.push(child);
        }
    }
    single_command
}

fn has_named_descendant_kind(node: Node<'_>, kind: &str) -> bool {
    let mut stack = vec![node];
    while let Some(current) = stack.pop() {
        if current.kind() == kind {
            return true;
        }
        let mut cursor = current.walk();
        for child in current.named_children(&mut cursor) {
            stack.push(child);
        }
    }
    false
}

fn parse_double_quoted_string(node: Node, src: &str) -> Option<String> {
    if node.kind() != "string" {
        return None;
    }

    let mut cursor = node.walk();
    for part in node.named_children(&mut cursor) {
        if part.kind() != "string_content" {
            return None;
        }
    }
    let raw = node.utf8_text(src.as_bytes()).ok()?;
    let stripped = raw
        .strip_prefix('"')
        .and_then(|text| text.strip_suffix('"'))?;
    Some(stripped.to_string())
}

fn parse_raw_string(node: Node, src: &str) -> Option<String> {
    if node.kind() != "raw_string" {
        return None;
    }

    let raw_string = node.utf8_text(src.as_bytes()).ok()?;
    let stripped = raw_string
        .strip_prefix('\'')
        .and_then(|s| s.strip_suffix('\''));
    stripped.map(str::to_owned)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn parse_seq(src: &str) -> Option<Vec<Vec<String>>> {
        parse_shell_script_into_commands(src)
    }

    #[test]
    fn accepts_single_simple_command() {
        let cmds = parse_seq("ls -1").unwrap();
        assert_eq!(cmds, vec![vec!["ls".to_string(), "-1".to_string()]]);
    }

    #[test]
    fn accepts_multiple_commands_with_allowed_operators() {
        let src = "ls && pwd; echo 'hi there' | wc -l";
        let cmds = parse_seq(src).unwrap();
        let expected: Vec<Vec<String>> = vec![
            vec!["ls".to_string()],
            vec!["pwd".to_string()],
            vec!["echo".to_string(), "hi there".to_string()],
            vec!["wc".to_string(), "-l".to_string()],
        ];
        assert_eq!(cmds, expected);
    }

    #[test]
    fn extracts_double_and_single_quoted_strings() {
        let cmds = parse_seq("echo \"hello world\"").unwrap();
        assert_eq!(
            cmds,
            vec![vec!["echo".to_string(), "hello world".to_string()]]
        );

        let cmds2 = parse_seq("echo 'hi there'").unwrap();
        assert_eq!(
            cmds2,
            vec![vec!["echo".to_string(), "hi there".to_string()]]
        );
    }

    #[test]
    fn accepts_double_quoted_strings_with_newlines() {
        let cmds = parse_seq("git commit -m \"line1\nline2\"").unwrap();
        assert_eq!(
            cmds,
            vec![vec![
                "git".to_string(),
                "commit".to_string(),
                "-m".to_string(),
                "line1\nline2".to_string(),
            ]]
        );
    }

    #[test]
    fn accepts_mixed_quote_concatenation() {
        assert_eq!(
            parse_seq(r#"echo "/usr"'/'"local"/bin"#).unwrap(),
            vec![vec!["echo".to_string(), "/usr/local/bin".to_string()]]
        );
        assert_eq!(
            parse_seq(r#"echo '/usr'"/"'local'/bin"#).unwrap(),
            vec![vec!["echo".to_string(), "/usr/local/bin".to_string()]]
        );
    }

    #[test]
    fn rejects_double_quoted_strings_with_expansions() {
        assert!(parse_seq(r#"echo "hi ${USER}""#).is_none());
        assert!(parse_seq(r#"echo "$HOME""#).is_none());
    }

    #[test]
    fn accepts_numbers_as_words() {
        let cmds = parse_seq("echo 123 456").unwrap();
        assert_eq!(
            cmds,
            vec![vec![
                "echo".to_string(),
                "123".to_string(),
                "456".to_string()
            ]]
        );
    }

    #[test]
    fn plain_words_tell_unquoted_parts() {
        let words =
            parse_shell_script_into_plain_words("git log 'a*' \"b?\" c* x\"y\" 'p'q && echo 1")
                .expect("plain");
        let flags: Vec<Vec<(&str, bool)>> = words
            .iter()
            .map(|command| {
                command
                    .iter()
                    .map(|word| (word.text.as_str(), word.unquoted))
                    .collect()
            })
            .collect();
        assert_eq!(
            flags,
            vec![
                vec![
                    ("git", true),
                    ("log", true),
                    ("a*", false),
                    ("b?", false),
                    ("c*", true),
                    ("xy", true),
                    ("pq", true),
                ],
                vec![("echo", true), ("1", true)],
            ]
        );
        assert_eq!(parse_shell_script_into_plain_words("echo $x"), None);
    }

    #[test]
    fn rejects_parentheses_and_subshells() {
        assert!(parse_seq("(ls)").is_none());
        assert!(parse_seq("ls || (pwd && echo hi)").is_none());
    }

    #[test]
    fn rejects_redirections_and_unsupported_operators() {
        assert!(parse_seq("ls > out.txt").is_none());
        assert!(parse_seq("echo hi & echo bye").is_none());
    }

    #[test]
    fn rejects_command_and_process_substitutions_and_expansions() {
        assert!(parse_seq("echo $(pwd)").is_none());
        assert!(parse_seq("echo `pwd`").is_none());
        assert!(parse_seq("echo $HOME").is_none());
        assert!(parse_seq("echo \"hi $USER\"").is_none());
    }

    #[test]
    fn rejects_variable_assignment_prefix() {
        assert!(parse_seq("FOO=bar ls").is_none());
    }

    #[test]
    fn rejects_trailing_operator_parse_error() {
        assert!(parse_seq("ls &&").is_none());
    }

    #[test]
    fn rejects_empty_command_position_with_leading_operator() {
        assert!(parse_seq("&& ls").is_none());
    }

    #[test]
    fn rejects_empty_command_position_with_double_separator() {
        assert!(parse_seq("ls ;; pwd").is_none());
    }

    #[test]
    fn rejects_empty_command_position_with_empty_pipeline_segment() {
        assert!(parse_seq("ls | | wc").is_none());
    }

    #[test]
    fn parse_zsh_lc_plain_commands() {
        let command = vec!["zsh".to_string(), "-lc".to_string(), "ls".to_string()];
        let parsed = parse_shell_lc_plain_commands(&command).unwrap();
        assert_eq!(parsed, vec![vec!["ls".to_string()]]);
    }

    #[test]
    fn accepts_concatenated_flag_and_value() {
        // Test case: -g"*.py" (flag directly concatenated with quoted value)
        let cmds = parse_seq("rg -n \"foo\" -g\"*.py\"").unwrap();
        assert_eq!(
            cmds,
            vec![vec![
                "rg".to_string(),
                "-n".to_string(),
                "foo".to_string(),
                "-g*.py".to_string(),
            ]]
        );
    }

    #[test]
    fn accepts_concatenated_flag_with_single_quotes() {
        let cmds = parse_seq("grep -n 'pattern' -g'*.txt'").unwrap();
        assert_eq!(
            cmds,
            vec![vec![
                "grep".to_string(),
                "-n".to_string(),
                "pattern".to_string(),
                "-g*.txt".to_string(),
            ]]
        );
    }

    #[test]
    fn rejects_concatenation_with_variable_substitution() {
        // Environment variables in concatenated strings should be rejected
        assert!(parse_seq("rg -g\"$VAR\" pattern").is_none());
        assert!(parse_seq("rg -g\"${VAR}\" pattern").is_none());
    }

    #[test]
    fn rejects_concatenation_with_command_substitution() {
        // Command substitution in concatenated strings should be rejected
        assert!(parse_seq("rg -g\"$(pwd)\" pattern").is_none());
        assert!(parse_seq("rg -g\"$(echo '*.py')\" pattern").is_none());
    }

    #[test]
    fn parse_shell_lc_single_command_prefix_supports_heredoc() {
        let command = vec![
            "zsh".to_string(),
            "-lc".to_string(),
            "python3 <<'PY'\nprint('hello')\nPY".to_string(),
        ];
        let parsed = parse_shell_lc_single_command_prefix(&command);
        assert_eq!(parsed, Some(vec!["python3".to_string()]));

        let command_unquoted = vec![
            "zsh".to_string(),
            "-lc".to_string(),
            "python3 << PY\nprint('hello')\nPY".to_string(),
        ];
        let parsed_unquoted = parse_shell_lc_single_command_prefix(&command_unquoted);
        assert_eq!(parsed_unquoted, Some(vec!["python3".to_string()]));
    }

    #[test]
    fn parse_shell_lc_single_command_prefix_rejects_multi_command_scripts() {
        let command = vec![
            "bash".to_string(),
            "-lc".to_string(),
            "python3 <<'PY'\nprint('hello')\nPY\necho done".to_string(),
        ];
        assert_eq!(parse_shell_lc_single_command_prefix(&command), None);
    }

    #[test]
    fn parse_shell_lc_single_command_prefix_rejects_non_heredoc_redirects() {
        let command = vec![
            "bash".to_string(),
            "-lc".to_string(),
            "echo hello > /tmp/out.txt".to_string(),
        ];
        assert_eq!(parse_shell_lc_single_command_prefix(&command), None);
    }

    #[test]
    fn parse_shell_lc_single_command_prefix_rejects_heredoc_with_extra_file_redirect() {
        let command = vec![
            "bash".to_string(),
            "-lc".to_string(),
            "python3 <<'PY' > /tmp/out.txt\nprint('hello')\nPY".to_string(),
        ];
        assert_eq!(parse_shell_lc_single_command_prefix(&command), None);
    }

    #[test]
    fn parse_shell_lc_single_command_prefix_rejects_heredoc_with_variable_assignment() {
        let command = vec![
            "bash".to_string(),
            "-lc".to_string(),
            "PATH=/tmp/evil:$PATH cat <<'EOF'\nhello\nEOF".to_string(),
        ];
        assert_eq!(parse_shell_lc_single_command_prefix(&command), None);
    }

    #[test]
    fn parse_shell_lc_single_command_prefix_rejects_herestring_with_chaining() {
        let command = vec![
            "bash".to_string(),
            "-lc".to_string(),
            r#"echo hello > /tmp/out.txt && cat /tmp/out.txt"#.to_string(),
        ];
        assert_eq!(parse_shell_lc_single_command_prefix(&command), None);
    }

    #[test]
    fn parse_shell_lc_single_command_prefix_rejects_herestring_with_substitution() {
        let command = vec![
            "bash".to_string(),
            "-lc".to_string(),
            r#"python3 <<< "$(rm -rf /)""#.to_string(),
        ];
        assert_eq!(parse_shell_lc_single_command_prefix(&command), None);
    }

    #[test]
    fn parse_shell_lc_single_command_prefix_rejects_arithmetic_shift_non_heredoc_script() {
        let command = vec![
            "bash".to_string(),
            "-lc".to_string(),
            "echo $((1<<2))".to_string(),
        ];
        assert_eq!(parse_shell_lc_single_command_prefix(&command), None);
    }

    #[test]
    fn parse_shell_lc_single_command_prefix_rejects_heredoc_command_with_word_expansion() {
        let command = vec![
            "bash".to_string(),
            "-lc".to_string(),
            "python3 $((1<<2)) <<'PY'\nprint('hello')\nPY".to_string(),
        ];
        assert_eq!(parse_shell_lc_single_command_prefix(&command), None);
    }
}

#[cfg(test)]
#[path = "bash/script_commands_tests.rs"]
mod script_commands_tests;
