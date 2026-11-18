//! #2548 (D553): what command rules can know of a shell script before it runs.
//!
//! Default deny: the reader walks the syntax tree and accepts only node
//! kinds whose effect on what runs it fully understands (commands, plain
//! words and quoting, `$name`/`${name…}` expansions, `$(…)`, the redirects
//! below, lists, pipelines, groups, subshells, `if`/`while`/`for`/`case`,
//! function bodies, simple assignments and declarations). Any other kind, in
//! any position, marks the script [`ScriptRead::unread`]: callers then ask
//! whenever a prompt or forbidden rule exists. Commands inside an unread part
//! are still collected, so a command a rule refuses stays refused.
//!
//! What a command's words do (`eval`, a shell reading stdin, `source`) is for
//! the caller, which knows launchers; this reader reports each command's
//! words after the reserved prefixes `!`, `time [-p] [--]` and `coproc`, and
//! where its standard input comes from.

use super::heredoc::heredoc_boundaries_readable;
use super::heredoc::literal_heredoc;
use super::script_command_words;
use super::script_word;
use super::try_parse_shell;
use super::word_only_command_nodes;
use tree_sitter::Node;

/// Where a command's standard input comes from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandStdin {
    /// The script's own input, or that of the group, loop or function around
    /// the command: nothing the reader knows.
    Inherited,
    /// The command's own here-document or here-string, as the shell supplies
    /// it.
    Literal(String),
    /// The command's own input redirection, a pipe, or a coprocess's pipe:
    /// not known.
    Redirected,
}

/// A command of the script.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadCommand {
    /// Its argv after quote removal; `None` where a word is built only as the
    /// script runs (it may also become no word or several).
    pub words: Vec<Option<String>>,
    pub stdin: CommandStdin,
}

/// See the module documentation.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ScriptRead {
    /// Every command the script contains, in source order: those in
    /// substitutions, control flow and function bodies too.
    pub commands: Vec<ReadCommand>,
    /// Some part of the script is not a shape this reader fully understands,
    /// or the script does not parse: it may run commands not in `commands`.
    pub unread: bool,
    /// Plain commands joined only by `&&`, `||`, `;` and `|`, every word
    /// known: rules may approve it as these commands.
    pub plain: bool,
    /// Text other than whitespace (a comment too) follows the last command.
    pub text_after_commands: bool,
}

/// Reads `script` as bash would run it; see the module documentation.
pub fn read_script(script: &str) -> ScriptRead {
    let unread = ScriptRead {
        unread: true,
        ..ScriptRead::default()
    };
    let Some(tree) = try_parse_shell(script) else {
        return unread;
    };
    let root = tree.root_node();
    if root.has_error() || !heredoc_boundaries_readable(root, script) {
        return unread;
    }
    let mut reader = Reader {
        src: script,
        commands: Vec::new(),
        unread: false,
    };
    reader.read(root);
    let mut commands = reader.commands;
    commands.sort_by_key(|(start, _)| *start);
    let commands: Vec<ReadCommand> = commands
        .into_iter()
        .map(|(_, command)| command)
        .filter(|command| !command.words.is_empty())
        .collect();
    let plain = !reader.unread
        && word_only_command_nodes(&tree).is_some()
        && commands
            .iter()
            .all(|command| command.words.iter().all(Option::is_some));
    let mut end = 0;
    for (_, child) in fields(root, /*named_only*/ true) {
        if child.kind() != "comment" {
            end = end.max(child.end_byte());
        }
    }
    ScriptRead {
        commands,
        unread: reader.unread,
        plain,
        text_after_commands: !script[end..].trim().is_empty(),
    }
}

/// Words that are reserved at the start of a command in bash or zsh. One
/// the reader does not strip there, or that follows `coproc`, is a shape it
/// does not read.
pub const RESERVED_WORDS: &[&str] = &[
    "!", "{", "}", "[[", "]]", "case", "coproc", "do", "done", "elif", "else", "esac", "fi", "for",
    "function", "if", "in", "select", "then", "time", "until", "while",
    // zsh's own.
    "repeat", "foreach", "end", "always",
];

/// Variables a shell or the commands it starts read as code or as a file
/// of code (`BASH_ENV=./x.sh bash -c …`, `PS4='$(…)'` with xtrace,
/// exported functions). Assigning one is not read.
pub fn runs_code_from_variable(name: &str) -> bool {
    matches!(
        name,
        "BASH_ENV" | "ENV" | "PS4" | "SHELLOPTS" | "BASHOPTS" | "PROMPT_COMMAND" | "ZDOTDIR"
    ) || name.starts_with("BASH_FUNC_")
}

#[derive(Clone, Copy)]
enum Visit {
    Statement,
    Word,
    /// Inside a part not read: collect its commands only.
    Collect,
}

struct Reader<'a> {
    src: &'a str,
    commands: Vec<(usize, ReadCommand)>,
    unread: bool,
}

impl<'a> Reader<'a> {
    fn text(&self, node: Node<'_>) -> &'a str {
        node.utf8_text(self.src.as_bytes()).unwrap_or_default()
    }

    /// A work list, not recursion: scripts may nest deeply.
    fn read(&mut self, root: Node<'_>) {
        let mut stack = vec![(root, Visit::Statement)];
        while let Some((node, visit)) = stack.pop() {
            match visit {
                Visit::Statement => self.statement(node, &mut stack),
                Visit::Word => self.word(node, &mut stack),
                Visit::Collect => {
                    if node.kind() == "command" {
                        self.commands.push((
                            node.start_byte(),
                            ReadCommand {
                                words: script_command_words(node, self.src),
                                stdin: CommandStdin::Redirected,
                            },
                        ));
                    }
                    push_named_children(node, Visit::Collect, &mut stack);
                }
            }
        }
    }

    fn deny<'t>(&mut self, node: Node<'t>, stack: &mut Vec<(Node<'t>, Visit)>) {
        self.unread = true;
        stack.push((node, Visit::Collect));
    }

    fn statement<'t>(&mut self, node: Node<'t>, stack: &mut Vec<(Node<'t>, Visit)>) {
        match node.kind() {
            "comment" => {}
            "program" | "list" | "compound_statement" | "subshell" | "negated_command"
            | "do_group" | "if_statement" | "elif_clause" | "else_clause" | "while_statement" => {
                push_named_children(node, Visit::Statement, stack);
            }
            "pipeline" => {
                let mut cursor = node.walk();
                for (index, child) in node.named_children(&mut cursor).enumerate() {
                    if index > 0 && self.command_statement(child, /*piped*/ true, stack) {
                        continue;
                    }
                    stack.push((child, Visit::Statement));
                }
            }
            "command" | "redirected_statement" => {
                if !self.command_statement(node, /*piped*/ false, stack) {
                    self.redirected_statement(node, stack);
                }
            }
            "for_statement" => {
                // `for`/`select NAME in WORDS; do …; done`.
                for (field, child) in fields(node, /*named_only*/ true) {
                    match field {
                        Some("variable") if child.kind() == "variable_name" => {}
                        Some("value") => stack.push((child, Visit::Word)),
                        Some("body") => stack.push((child, Visit::Statement)),
                        _ => self.deny(child, stack),
                    }
                }
            }
            "case_statement" => {
                for (field, child) in fields(node, /*named_only*/ true) {
                    match (field, child.kind()) {
                        (Some("value"), _) => stack.push((child, Visit::Word)),
                        (_, "case_item") => self.case_item(child, stack),
                        (_, "comment") => {}
                        _ => self.deny(child, stack),
                    }
                }
            }
            "function_definition" => {
                // The body is read where it is defined, with input from
                // wherever it is called: whatever runs it, rules see its
                // commands.
                for (field, child) in fields(node, /*named_only*/ true) {
                    match field {
                        Some("name") if child.kind() == "word" => {}
                        Some("body") => stack.push((child, Visit::Statement)),
                        _ => self.deny(child, stack),
                    }
                }
            }
            "variable_assignment" | "variable_assignments" => self.assignment(node, stack),
            "declaration_command" => self.declaration(node, stack),
            "unset_command" => {
                for (_, child) in fields(node, /*named_only*/ true) {
                    // `unset 'a[$(…)]'` evaluates the subscript.
                    let known = script_word(child, self.src)
                        .is_some_and(|word| !word.contains('[') && !word.starts_with('-'))
                        || child.kind() == "variable_name"
                        || (child.kind() == "word" && matches!(self.text(child), "-v" | "-f"));
                    if !known {
                        self.deny(child, stack);
                    }
                }
            }
            "test_command" => self.test_command(node, stack),
            _ => self.deny(node, stack),
        }
    }

    fn case_item<'t>(&mut self, node: Node<'t>, stack: &mut Vec<(Node<'t>, Visit)>) {
        for (field, child) in fields(node, /*named_only*/ true) {
            match (field, child.kind()) {
                (Some("value"), "extglob_pattern") => {}
                (Some("value"), _) => stack.push((child, Visit::Word)),
                _ => stack.push((child, Visit::Statement)),
            }
        }
    }

    /// `[ … ]` and `[[ … ]]`. `-v NAME[…]` evaluates a subscript, and an
    /// arithmetic comparison (`[[`'s, and zsh's `[`) evaluates its operands,
    /// subscripts included: read only where both are numbers or `$?`, `$#`.
    fn test_command<'t>(&mut self, node: Node<'t>, stack: &mut Vec<(Node<'t>, Visit)>) {
        let mut inner = vec![node];
        while let Some(part) = inner.pop() {
            let children = fields(part, /*named_only*/ true);
            let arithmetic = children.iter().any(|(_, child)| {
                child.kind() == "test_operator" && ARITHMETIC_TESTS.contains(&self.text(*child))
            });
            for (_, child) in children {
                match child.kind() {
                    "binary_expression" | "unary_expression" | "parenthesized_expression"
                        if !arithmetic =>
                    {
                        inner.push(child);
                    }
                    "test_operator" if !matches!(self.text(child), "-v" | "-R") => {}
                    "regex" | "extglob_pattern" if !arithmetic => {}
                    _ if arithmetic && !numeric_operand(self.text(child)) => {
                        self.deny(node, stack);
                        return;
                    }
                    "test_operator" => {
                        self.deny(node, stack);
                        return;
                    }
                    _ => stack.push((child, Visit::Word)),
                }
            }
        }
    }

    /// A command, alone or with its redirections (`redirected_statement`).
    /// Returns false for a redirected statement whose body is not a command.
    fn command_statement<'t>(
        &mut self,
        node: Node<'t>,
        piped: bool,
        stack: &mut Vec<(Node<'t>, Visit)>,
    ) -> bool {
        let (command, outer) = match node.kind() {
            "command" => (node, None),
            "redirected_statement" => match node.child_by_field_name("body") {
                Some(body) if body.kind() == "command" => (body, Some(node)),
                _ => return false,
            },
            _ => return false,
        };
        let mut inputs = Vec::new();
        if let Some(outer) = outer {
            for (_, child) in fields(outer, /*named_only*/ true) {
                if child != command {
                    self.redirect(child, &mut inputs, stack);
                }
            }
        }
        for (field, child) in fields(command, /*named_only*/ true) {
            match child.kind() {
                "command_name" => match child.named_child(0) {
                    Some(name) if child.named_child_count() == 1 => {
                        stack.push((name, Visit::Word));
                    }
                    _ => self.deny(child, stack),
                },
                "variable_assignment" => self.assignment(child, stack),
                "file_redirect" | "herestring_redirect" | "heredoc_redirect" => {
                    self.redirect(child, &mut inputs, stack);
                }
                "comment" => {}
                _ if field == Some("argument") => stack.push((child, Visit::Word)),
                _ => self.deny(child, stack),
            }
        }
        let mut nodes = command_word_nodes(command);
        let mut words = script_command_words(command, self.src);
        // The reserved prefixes bash reads before a pipeline: any sequence
        // of `!` and `time [-p] [--]`, then `coproc`.
        let mut skip = 0;
        loop {
            match nodes.get(skip).and_then(|node| self.reserved(*node)) {
                Some("!") => skip += 1,
                Some("time") => {
                    skip += 1;
                    for option in ["-p", "--"] {
                        if nodes.get(skip).and_then(|node| self.unquoted(*node)) == Some(option) {
                            skip += 1;
                        }
                    }
                }
                _ => break,
            }
        }
        let coproc = nodes.get(skip).and_then(|node| self.reserved(*node)) == Some("coproc");
        if coproc {
            skip += 1;
        }
        nodes.drain(..skip.min(nodes.len()));
        words.drain(..skip.min(words.len()));
        // `coproc NAME { …; }` and any other reserved word left at the start
        // (or after `coproc`) are shapes this reader does not read.
        let reserved_left = if coproc {
            nodes.iter().any(|node| self.reserved(*node).is_some())
        } else {
            nodes
                .first()
                .is_some_and(|node| self.reserved(*node).is_some())
        };
        if reserved_left {
            self.unread = true;
        }
        let stdin = match inputs.as_slice() {
            [] if piped || coproc => CommandStdin::Redirected,
            [] => CommandStdin::Inherited,
            [Some(literal)] => CommandStdin::Literal(literal.clone()),
            _ => CommandStdin::Redirected,
        };
        self.commands
            .push((command.start_byte(), ReadCommand { words, stdin }));
        true
    }

    /// The text of `node` when it is a word without quotes or expansions.
    fn unquoted(&self, node: Node<'_>) -> Option<&'a str> {
        (node.kind() == "word").then(|| self.text(node))
    }

    /// The reserved word `node` spells, unquoted.
    fn reserved(&self, node: Node<'_>) -> Option<&'static str> {
        let text = self.unquoted(node)?;
        RESERVED_WORDS.iter().copied().find(|word| *word == text)
    }

    /// The redirections of a statement whose body is not a command (a group,
    /// a loop): its commands keep [`CommandStdin::Inherited`].
    fn redirected_statement<'t>(&mut self, node: Node<'t>, stack: &mut Vec<(Node<'t>, Visit)>) {
        if node.kind() != "redirected_statement" {
            self.deny(node, stack);
            return;
        }
        let mut inputs = Vec::new();
        for (field, child) in fields(node, /*named_only*/ true) {
            if field == Some("body") {
                stack.push((child, Visit::Statement));
            } else {
                self.redirect(child, &mut inputs, stack);
            }
        }
    }

    /// Reads a redirection; adds an input to fd 0 to `inputs`, as its
    /// literal text where known.
    fn redirect<'t>(
        &mut self,
        node: Node<'t>,
        inputs: &mut Vec<Option<String>>,
        stack: &mut Vec<(Node<'t>, Visit)>,
    ) {
        let descriptor = node
            .child_by_field_name("descriptor")
            .map(|descriptor| self.text(descriptor));
        let to_stdin = descriptor.is_none_or(|descriptor| descriptor == "0");
        match node.kind() {
            "file_redirect" => {
                let mut operator = None;
                for (field, child) in fields(node, /*named_only*/ false) {
                    match field {
                        Some("descriptor") => {}
                        Some("destination") => stack.push((child, Visit::Word)),
                        _ if child.is_named() => self.deny(child, stack),
                        _ => operator = operator.or(Some(child.kind())),
                    }
                }
                if to_stdin && operator.is_some_and(|operator| operator.starts_with('<')) {
                    inputs.push(None);
                }
            }
            "herestring_redirect" => {
                let mut literal = None;
                for (field, child) in fields(node, /*named_only*/ true) {
                    if field == Some("descriptor") {
                        continue;
                    }
                    literal = script_word(child, self.src).map(|word| word + "\n");
                    stack.push((child, Visit::Word));
                }
                if to_stdin {
                    inputs.push(literal);
                }
            }
            "heredoc_redirect" => {
                // Words after the start marker (`cat <<EOF | bash`,
                // `bash <<EOF -c …`) nest inside this node in the grammar.
                for (field, child) in fields(node, /*named_only*/ true) {
                    match child.kind() {
                        "heredoc_start" | "heredoc_end" => {}
                        "heredoc_body" => {
                            for (_, part) in fields(child, /*named_only*/ true) {
                                if part.kind() != "heredoc_content" {
                                    // Expanded as the here-document is read.
                                    stack.push((part, Visit::Word));
                                }
                            }
                        }
                        _ if field == Some("descriptor") => {}
                        // `cat <<EOF > out`: the grammar nests the command's
                        // later redirections here.
                        "file_redirect" if field == Some("redirect") => {
                            self.redirect(child, inputs, stack);
                        }
                        _ => self.deny(child, stack),
                    }
                }
                if to_stdin {
                    inputs.push(literal_heredoc(node, self.src));
                }
            }
            _ => self.deny(node, stack),
        }
    }

    fn word<'t>(&mut self, node: Node<'t>, stack: &mut Vec<(Node<'t>, Visit)>) {
        match node.kind() {
            "word"
            | "number"
            | "raw_string"
            | "ansi_c_string"
            | "string_content"
            | "variable_name"
            | "special_variable_name"
            | "regex"
            | "extglob_pattern" => {}
            "string" | "concatenation" | "brace_expression" => {
                push_named_children(node, Visit::Word, stack);
            }
            "simple_expansion" => {
                for (_, child) in fields(node, /*named_only*/ true) {
                    if !matches!(child.kind(), "variable_name" | "special_variable_name") {
                        self.deny(child, stack);
                    }
                }
            }
            // The grammar starts an expansion in a string at the space
            // before it.
            "expansion" if simple_expansion_text(self.text(node).trim_start()) => {
                for (_, child) in fields(node, /*named_only*/ true) {
                    if child.kind() == "subscript" {
                        self.deny(child, stack);
                    } else {
                        stack.push((child, Visit::Word));
                    }
                }
            }
            // Backquotes nest by escaping, which the grammar does not read.
            "command_substitution" if !self.text(node).starts_with('`') => {
                push_named_children(node, Visit::Statement, stack);
            }
            "process_substitution" => push_named_children(node, Visit::Statement, stack),
            _ => self.deny(node, stack),
        }
    }

    /// `NAME=value`, `NAME+=value`, `NAME=(…)`. A subscript is arithmetic,
    /// which may run commands; so is an array element `[i]=v`.
    fn assignment<'t>(&mut self, node: Node<'t>, stack: &mut Vec<(Node<'t>, Visit)>) {
        if node.kind() == "variable_assignments" {
            for (_, child) in fields(node, /*named_only*/ true) {
                if child.kind() == "variable_assignment" {
                    self.assignment(child, stack);
                } else {
                    self.deny(child, stack);
                }
            }
            return;
        }
        if node.kind() != "variable_assignment" {
            self.deny(node, stack);
            return;
        }
        for (field, child) in fields(node, /*named_only*/ true) {
            match (field, child.kind()) {
                (Some("name"), "variable_name") if !runs_code_from_variable(self.text(child)) => {}
                (Some("value"), "array") => {
                    for (_, element) in fields(child, /*named_only*/ true) {
                        if self.text(element).starts_with('[') {
                            self.deny(element, stack);
                        } else {
                            stack.push((element, Visit::Word));
                        }
                    }
                }
                (Some("value"), _) => stack.push((child, Visit::Word)),
                _ => self.deny(child, stack),
            }
        }
    }

    /// `export`, `local`, `readonly`, `declare`, `typeset` with names,
    /// assignments and attribute options that run no code (`-i` makes later
    /// assignments arithmetic, `-n` names another variable, `-a`/`-A` read
    /// a quoted value as elements with subscripts).
    fn declaration<'t>(&mut self, node: Node<'t>, stack: &mut Vec<(Node<'t>, Visit)>) {
        let keyword = node.child(0).map(|keyword| self.text(keyword));
        if !matches!(
            keyword,
            Some("export" | "local" | "readonly" | "declare" | "typeset")
        ) {
            self.deny(node, stack);
            return;
        }
        for (_, child) in fields(node, /*named_only*/ true) {
            match child.kind() {
                "variable_name" if !runs_code_from_variable(self.text(child)) => {}
                "variable_assignment" => self.assignment(child, stack),
                "word"
                    if matches!(
                        self.text(child),
                        "-x" | "+x" | "-r" | "-g" | "-p" | "-f" | "-F"
                    ) => {}
                _ => self.deny(child, stack),
            }
        }
    }
}

fn push_named_children<'t>(node: Node<'t>, visit: Visit, stack: &mut Vec<(Node<'t>, Visit)>) {
    let children = fields(node, /*named_only*/ true);
    // Reversed, so the work list visits them in source order.
    stack.extend(children.into_iter().rev().map(|(_, child)| (child, visit)));
}

/// Test operators that compare numbers, evaluating their operands as
/// arithmetic in `[[` (and in zsh's `[` and `test`).
pub const ARITHMETIC_TESTS: &[&str] = &["-eq", "-ne", "-lt", "-le", "-gt", "-ge"];

/// An operand arithmetic reads as a number: digits, or `$?`, `$#`, `$$`,
/// `$!`, optionally in double quotes.
pub fn numeric_operand(text: &str) -> bool {
    let text = text
        .strip_prefix('"')
        .and_then(|text| text.strip_suffix('"'))
        .unwrap_or(text);
    let digits = text.strip_prefix('-').unwrap_or(text);
    (!digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_digit()))
        || matches!(text, "$?" | "$#" | "$$" | "$!")
}

/// The nodes [`script_command_words`] reads as words, in the same order.
fn command_word_nodes(command: Node<'_>) -> Vec<Node<'_>> {
    let mut nodes = Vec::new();
    for (_, child) in fields(command, /*named_only*/ true) {
        match child.kind() {
            "variable_assignment"
            | "variable_assignments"
            | "file_redirect"
            | "heredoc_redirect"
            | "herestring_redirect"
            | "comment" => {}
            "command_name" => {
                if let Some(name) = child.named_child(0) {
                    nodes.push(name);
                }
            }
            _ => nodes.push(child),
        }
    }
    nodes
}

/// `${NAME}`, `${#NAME}` and `${NAME OP word}` for the operators that
/// substitute, trim or replace text. Not indirection (`${!x}`), substrings
/// (`${x:1}`, arithmetic), transformations (`${x@P}` expands prompts) or
/// zsh's flags (`${(e)x}`).
fn simple_expansion_text(text: &str) -> bool {
    let Some(inner) = text
        .strip_prefix("${")
        .and_then(|inner| inner.strip_suffix('}'))
    else {
        return false;
    };
    if let Some(name) = inner.strip_prefix('#')
        && !name.is_empty()
    {
        return is_parameter_name(name);
    }
    let name_end = if inner.starts_with(|ch: char| ch.is_ascii_alphabetic() || ch == '_') {
        inner
            .find(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
            .unwrap_or(inner.len())
    } else if inner.starts_with(|ch: char| ch.is_ascii_digit()) {
        inner
            .find(|ch: char| !ch.is_ascii_digit())
            .unwrap_or(inner.len())
    } else if inner.starts_with(['@', '*', '#', '?', '$', '-', '0']) {
        1
    } else {
        return false;
    };
    let rest = &inner[name_end..];
    rest.is_empty()
        || [
            ":-", ":=", ":+", ":?", "-", "=", "+", "?", "#", "%", "/", "^", ",",
        ]
        .iter()
        .any(|operator| rest.starts_with(operator))
}

fn is_parameter_name(name: &str) -> bool {
    name.chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
        || matches!(name, "@" | "*" | "#" | "?" | "$" | "-")
}

/// The children of `node` with their field names.
fn fields(node: Node<'_>, named_only: bool) -> Vec<(Option<&'static str>, Node<'_>)> {
    (0..node.child_count())
        .filter_map(|index| {
            let child = node.child(index)?;
            (child.is_named() || !named_only)
                .then(|| (node.field_name_for_child(index as u32), child))
        })
        .collect()
}
