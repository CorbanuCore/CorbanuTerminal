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
use super::script_lexical::lexically_safe;
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
    /// Per word: it is always a number (`$?`, `$#`, `$$`, `$!`, digits).
    pub numbers: Vec<bool>,
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
    /// Every byte lies within the minimal lexical set (`script_lexical`),
    /// which tree-sitter-bash and the shells read alike. A script outside
    /// it fails closed however the syntax tree reads it.
    pub minimal: bool,
}

impl ScriptRead {
    /// Whether rules must ask (where any prompt or forbidden rule exists)
    /// besides what its commands match: part of it is not read, or it lies
    /// outside the minimal lexical set.
    pub fn fails_closed(&self) -> bool {
        self.unread || !self.minimal
    }
}

/// Reads `script` as bash would run it; see the module documentation.
pub fn read_script(script: &str) -> ScriptRead {
    let mut read = read_tree(script);
    // The lexical pre-filter: a script outside the minimal set fails closed
    // whatever the syntax tree says.
    read.minimal = lexically_safe(script);
    read.plain &= read.minimal;
    read
}

/// [`read_script`] by the syntax tree alone, without the lexical
/// pre-filter.
pub(super) fn read_tree(script: &str) -> ScriptRead {
    let unread = ScriptRead {
        unread: true,
        ..ScriptRead::default()
    };
    let Some(tree) = try_parse_shell(script) else {
        return unread;
    };
    let root = tree.root_node();
    if root.has_error()
        || !heredoc_boundaries_readable(root, script)
        || !continuations_read_as_bash(root, script)
    {
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
        // Set by `read_script`.
        minimal: false,
    }
}

/// Whether `node` holds one of the anonymous `tokens`.
fn has_token(node: Node<'_>, tokens: &[&str]) -> bool {
    let mut stack = vec![node];
    while let Some(node) = stack.pop() {
        if !node.is_named() && tokens.contains(&node.kind()) {
            return true;
        }
        // Their own parentheses and operators are not the test's.
        if matches!(
            node.kind(),
            "command_substitution"
                | "process_substitution"
                | "string"
                | "raw_string"
                | "ansi_c_string"
                | "expansion"
                | "simple_expansion"
        ) {
            continue;
        }
        stack.extend(
            fields(node, /*named_only*/ false)
                .into_iter()
                .map(|(_, child)| child),
        );
    }
    false
}

/// The grammar drops a backslash-newline between tokens, and also before a
/// carriage return; bash joins the text around it (`git pu\<newline>sh` runs
/// `git push`) and escapes a carriage return. Both read alike only where a
/// continuation sits inside a token that removes it, or beside whitespace.
fn continuations_read_as_bash(root: Node<'_>, script: &str) -> bool {
    if script.contains("\\\r") {
        return false;
    }
    script.match_indices("\\\n").all(|(at, _)| {
        let inside_token = root
            .descendant_for_byte_range(at, at + 2)
            .is_some_and(|node| {
                matches!(
                    node.kind(),
                    "word"
                        | "string"
                        | "string_content"
                        | "raw_string"
                        | "ansi_c_string"
                        | "comment"
                        | "heredoc_body"
                        | "heredoc_content"
                )
            });
        let before = script[..at].chars().next_back();
        let after = script[at + 2..].chars().next();
        inside_token
            || before.is_none_or(char::is_whitespace)
            || after.is_none_or(char::is_whitespace)
    })
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
/// exported functions), or whose assigned value is arithmetic. Assigning
/// one is not read.
pub fn runs_code_from_variable(name: &str) -> bool {
    matches!(
        name,
        "BASH_ENV"
            | "ENV"
            | "PS4"
            // Prompts other shells or options may expand (zsh's PROMPT4 is
            // its xtrace prompt).
            | "PS0"
            | "PS1"
            | "PS2"
            | "PS3"
            | "PROMPT"
            | "PROMPT2"
            | "PROMPT3"
            | "PROMPT4"
            | "RPROMPT"
            | "RPROMPT2"
            | "RPS1"
            | "RPS2"
            | "SPROMPT"
            | "SHELLOPTS"
            | "BASHOPTS"
            | "PROMPT_COMMAND"
            | "ZDOTDIR"
            // zsh runs these for a line that is only a redirection.
            | "NULLCMD"
            | "READNULLCMD"
    ) || name.starts_with("BASH_FUNC_")
        // zsh's hook functions.
        || name.ends_with("_functions")
        // Tables that rename or define commands: zsh's special hashes, bash's
        // command hash and aliases.
        || matches!(
            name,
            "commands"
                | "functions"
                | "aliases"
                | "galiases"
                | "saliases"
                | "builtins"
                | "reswords"
                | "options"
                | "nameddirs"
                | "BASH_CMDS"
                | "BASH_ALIASES"
        )
        || matches!(
            name,
            "dis_functions"
                | "dis_aliases"
                | "dis_galiases"
                | "dis_saliases"
                | "dis_builtins"
                | "dis_reswords"
                | "dis_patchars"
        )
        || INTEGER_SPECIALS.contains(&name)
}

/// Special variables of bash, zsh or ksh that evaluate a value assigned to
/// them as arithmetic (`OPTIND='a[$(…)]'` runs the substitution); the same
/// list as the post-taint classifier's (`tainted_action/arithmetic.rs`).
const INTEGER_SPECIALS: &[&str] = &[
    "EGID",
    "EUID",
    "FUNCNEST",
    "GID",
    "UID",
    "BAUD",
    "COLUMNS",
    "DIRSTACKSIZE",
    "ERRNO",
    "HISTCMD",
    "HISTSIZE",
    "KEYTIMEOUT",
    "LINENO",
    "LINES",
    "LISTMAX",
    "MAILCHECK",
    "OPTIND",
    "RANDOM",
    "SAVEHIST",
    "SECONDS",
    "SHLVL",
    "TMOUT",
    "TRY_BLOCK_ERROR",
    "TRY_BLOCK_INTERRUPT",
    "ZLE_RPROMPT_INDENT",
];

/// Whether a token's text holds an expansion the grammar left in it: a
/// backquote (the scanner splits `$(`, `${` and `$[` from words, but not
/// backquotes in patterns, regexes or here-document text).
fn hides_expansion(text: &str) -> bool {
    text.contains('`') || text.contains("$(") || text.contains("${") || text.contains("$[")
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
                                numbers: Vec::new(),
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

    /// Bash starts a comment only at the start of a word, after removing
    /// escaped newlines; the grammar also does after some closing tokens
    /// (`x=(a)#; cmd` runs `cmd`) and after an escaped newline.
    fn bash_comment(&self, node: Node<'_>) -> bool {
        let mut before = &self.src[..node.start_byte()];
        while let Some(joined) = before.strip_suffix("\\\n") {
            before = joined;
        }
        before
            .chars()
            .next_back()
            .is_none_or(|before| matches!(before, ' ' | '\t' | '\n' | ';' | '&' | '|'))
    }

    fn deny<'t>(&mut self, node: Node<'t>, stack: &mut Vec<(Node<'t>, Visit)>) {
        self.unread = true;
        stack.push((node, Visit::Collect));
    }

    fn statement<'t>(&mut self, node: Node<'t>, stack: &mut Vec<(Node<'t>, Visit)>) {
        match node.kind() {
            "comment" if self.bash_comment(node) => {}
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
                        Some("variable")
                            if child.kind() == "variable_name"
                                && !runs_code_from_variable(self.text(child)) => {}
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
                        (_, "comment") if self.bash_comment(child) => {}
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
                (Some("value"), _) => stack.push((child, Visit::Word)),
                _ => stack.push((child, Visit::Statement)),
            }
        }
    }

    /// `[ … ]` and `[[ … ]]`. `-v NAME[…]` evaluates a subscript, and an
    /// arithmetic comparison (`[[`'s, and zsh's `[`) evaluates its operands,
    /// subscripts included: read only where both are numbers or `$?`, `$#`.
    fn test_command<'t>(&mut self, node: Node<'t>, stack: &mut Vec<(Node<'t>, Visit)>) {
        let single = !self.text(node).starts_with("[[");
        // `[` is a simple command for bash: it ends at a newline, and `<`,
        // `>` and `(` there are redirections or syntax, not test operators
        // (`[⏎! sh -c …` runs `sh`).
        if single && (self.text(node).contains('\n') || has_token(node, &["<", ">", "(", ")"])) {
            self.deny(node, stack);
            return;
        }
        let mut inner = vec![node];
        while let Some(part) = inner.pop() {
            // In `[ … ]` the grammar takes `|`, `&`, `&&` and `||` for test
            // operators; bash ends the command there (`[ x | sh … ]`).
            // zsh splits `|=`, `&=` and the like there too, and `>=` or `>>`
            // write a file.
            let operators = fields(part, /*named_only*/ false);
            if operators.iter().any(|(_, child)| {
                let token = child.kind();
                !child.is_named()
                    && token.contains(['<', '>', '|', '&'])
                    && (single || !matches!(token, "&&" | "||" | "<" | ">"))
            }) {
                self.deny(node, stack);
                return;
            }
            let children = fields(part, /*named_only*/ true);
            // zsh's `-t FD` evaluates FD as arithmetic too.
            let arithmetic = children.iter().any(|(_, child)| {
                child.kind() == "test_operator"
                    && (ARITHMETIC_TESTS.contains(&self.text(*child)) || self.text(*child) == "-t")
            });
            for (_, child) in children {
                match child.kind() {
                    "binary_expression" | "unary_expression" | "parenthesized_expression"
                        if !arithmetic =>
                    {
                        inner.push(child);
                    }
                    "test_operator" if !matches!(self.text(child), "-v" | "-R") => {}
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
                "comment" if self.bash_comment(child) => {}
                _ if field == Some("argument") => stack.push((child, Visit::Word)),
                _ => self.deny(child, stack),
            }
        }
        let mut nodes = command_word_nodes(command);
        let mut words = script_command_words(command, self.src);
        // The reserved prefixes bash reads before a pipeline: any sequence
        // of `!` and `time [-p] [--]` (and zsh's `nocorrect`, which allows an
        // assignment after it), with one `coproc`.
        let mut skip = 0;
        let mut coproc = false;
        loop {
            let unquoted = nodes.get(skip).and_then(|node| self.unquoted(*node));
            match nodes.get(skip).and_then(|node| self.reserved(*node)) {
                _ if unquoted == Some("nocorrect") => skip += 1,
                Some("coproc") if !coproc => {
                    coproc = true;
                    skip += 1;
                }
                Some("!") => skip += 1,
                // Where `time` is not a keyword (dash) it is a program whose
                // options may be followed by the command (`time -l git push`):
                // leave it, and its launcher reading, to the caller.
                Some("time")
                    if nodes
                        .get(skip + 1)
                        .and_then(|node| self.unquoted(*node))
                        .is_some_and(|next| {
                            next.starts_with('-') && !matches!(next, "-p" | "--")
                        }) =>
                {
                    break;
                }
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
        nodes.drain(..skip.min(nodes.len()));
        words.drain(..skip.min(words.len()));
        // zsh's extended_glob (a user may set it) makes `#` and `^` glob
        // characters: such a command name is not known.
        if let (Some(node), Some(word)) = (nodes.first(), words.first_mut())
            && self
                .unquoted(*node)
                .is_some_and(|text| text.contains(['#', '^']))
        {
            *word = None;
        }
        // `coproc NAME { …; }` and any other reserved word left at the start
        // (or after `coproc`) are shapes this reader does not read.
        let reserved_left = if coproc {
            nodes.iter().any(|node| self.reserved(*node).is_some())
        } else {
            nodes
                .first()
                .is_some_and(|node| self.reserved(*node).is_some())
        };
        // After a prefix bash reads `NAME=value` as an assignment, which the
        // grammar took for a word (`coproc OPTIND='a[$(…)]'`).
        let assignment_left = skip > 0
            && nodes.first().is_some_and(|node| {
                self.text(*node).split_once('=').is_some_and(|(name, _)| {
                    name.starts_with(|ch: char| ch.is_ascii_alphabetic() || ch == '_')
                })
            });
        if reserved_left || assignment_left {
            self.unread = true;
        }
        let stdin = match inputs.as_slice() {
            [] if piped || coproc => CommandStdin::Redirected,
            [] => CommandStdin::Inherited,
            [Some(literal)] => CommandStdin::Literal(literal.clone()),
            _ => CommandStdin::Redirected,
        };
        self.commands.push((
            command.start_byte(),
            ReadCommand {
                numbers: nodes
                    .iter()
                    .map(|node| numeric_operand(self.text(*node)))
                    .collect(),
                words,
                stdin,
            },
        ));
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
        // Any redirection onto fd 0 replaces the input (`0>&3`, `00<f`); a
        // descriptor that is not a number may be 0.
        let descriptor = node
            .child_by_field_name("descriptor")
            .map(|descriptor| self.text(descriptor).parse::<u32>().ok());
        let to_stdin = descriptor.map(|fd| fd.is_none_or(|fd| fd == 0));
        match node.kind() {
            "file_redirect" => {
                let mut operator = None;
                let mut destinations = 0;
                for (field, child) in fields(node, /*named_only*/ false) {
                    match field {
                        Some("descriptor") => {}
                        // The grammar takes later words for more
                        // destinations; bash runs them as arguments
                        // (`git 2>&1 push`).
                        Some("destination") if destinations > 0 => self.deny(child, stack),
                        Some("destination") => {
                            destinations += 1;
                            stack.push((child, Visit::Word));
                        }
                        _ if child.is_named() => self.deny(child, stack),
                        _ => operator = operator.or(Some(child.kind())),
                    }
                }
                if to_stdin.unwrap_or_else(|| operator.is_some_and(|op| op.starts_with('<'))) {
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
                if to_stdin.unwrap_or(true) {
                    inputs.push(literal);
                }
            }
            "heredoc_redirect" => {
                let delimiter = fields(node, /*named_only*/ true)
                    .into_iter()
                    .find(|(_, child)| child.kind() == "heredoc_start")
                    .map(|(_, start)| {
                        self.text(start)
                            .trim_matches(['\'', '"'])
                            .trim_start_matches('\\')
                            .to_string()
                    });
                // An unquoted here-document's text is expanded as it is read.
                let expanded = node
                    .child_by_field_name("start")
                    .or_else(|| {
                        fields(node, /*named_only*/ true)
                            .into_iter()
                            .map(|(_, child)| child)
                            .find(|child| child.kind() == "heredoc_start")
                    })
                    .is_none_or(|start| !self.text(start).contains(['\'', '"', '\\']));
                // The grammar nests what follows the start marker on its line
                // here: `cat <<EOF > out`, `cat <<EOF && cmd`, `cat <<EOF | cmd`.
                for (field, child) in fields(node, /*named_only*/ true) {
                    match child.kind() {
                        "heredoc_start" | "heredoc_end" => {}
                        // A backquote in it may not be a node of its own.
                        "heredoc_body" if expanded && self.text(child).contains('`') => {
                            self.deny(child, stack);
                        }
                        "heredoc_body" => {
                            for (_, part) in fields(child, /*named_only*/ true) {
                                // bash ends the body at the delimiter line even
                                // inside a substitution the grammar reads on.
                                if part.kind() != "heredoc_content"
                                    && delimiter.as_deref().is_some_and(|delimiter| {
                                        self.text(part)
                                            .lines()
                                            .any(|line| line.trim_start_matches('\t') == delimiter)
                                    })
                                {
                                    self.deny(part, stack);
                                } else if part.kind() != "heredoc_content" {
                                    stack.push((part, Visit::Word));
                                } else if expanded && hides_expansion(self.text(part)) {
                                    self.deny(part, stack);
                                }
                            }
                        }
                        _ if field == Some("descriptor") => {}
                        "file_redirect" if field == Some("redirect") => {
                            self.redirect(child, inputs, stack);
                        }
                        // Not piped from this command's input: a pipe's
                        // receiver reads the pipe, and a shell given any
                        // input but its own literal text is not read.
                        "pipeline" => stack.push((child, Visit::Statement)),
                        _ if field == Some("right") => stack.push((child, Visit::Statement)),
                        _ => self.deny(child, stack),
                    }
                }
                if to_stdin.unwrap_or(true) {
                    inputs.push(literal_heredoc(node, self.src));
                }
            }
            _ => self.deny(node, stack),
        }
    }

    /// Whether `${…}` holds a brace of its own (not in a nested expansion,
    /// substitution or quotes): the grammar nests `{` where bash ends the
    /// expansion at the first `}`.
    fn braces_in_expansion(&self, node: Node<'_>) -> bool {
        let text = self.text(node).trim_start();
        let start = node.end_byte() - text.len() + 2;
        let end = node.end_byte().saturating_sub(1);
        let mut own = String::new();
        let mut at = start;
        for (_, child) in fields(node, /*named_only*/ true) {
            if matches!(
                child.kind(),
                "expansion"
                    | "simple_expansion"
                    | "command_substitution"
                    | "process_substitution"
                    | "string"
                    | "raw_string"
                    | "ansi_c_string"
            ) && child.start_byte() >= at
            {
                own.push_str(&self.src[at..child.start_byte()]);
                at = child.end_byte();
            }
        }
        own.push_str(self.src.get(at..end).unwrap_or_default());
        own.contains(['{', '}'])
    }

    /// Whether a `regex` or `extglob_pattern` token is where bash reads a
    /// pattern as the grammar does.
    /// The scanner keeps quotes, `<(…)` and operators inside these tokens
    /// where bash reads them as code, so only plain glob text is read.
    fn pattern_in_place(&self, node: Node<'_>) -> bool {
        let plain = self
            .text(node)
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"_-./*?+,:@%^!=~".contains(&byte));
        plain
            && match node.parent().map(|parent| parent.kind()) {
                Some("binary_expression") => true,
                Some("expansion") => node.kind() == "regex",
                Some("case_item") => node.kind() == "extglob_pattern",
                _ => false,
            }
    }

    fn word<'t>(&mut self, node: Node<'t>, stack: &mut Vec<(Node<'t>, Visit)>) {
        match node.kind() {
            "word" | "number" | "string_content" | "regex" | "extglob_pattern"
                if hides_expansion(self.text(node)) =>
            {
                self.deny(node, stack);
            }
            // The scanner reads these tokens past `;`, `|` and newlines
            // where bash ends them (`echo == x; cmd`): read only in a test's
            // operand, a case pattern, or an expansion's pattern without
            // braces (the scanner nests `{`, bash does not).
            "regex" | "extglob_pattern" if !self.pattern_in_place(node) => {
                self.deny(node, stack);
            }
            "word"
            | "number"
            | "raw_string"
            | "ansi_c_string"
            | "string_content"
            | "variable_name"
            | "special_variable_name"
            | "regex"
            | "extglob_pattern" => {}
            // A backquote in a double-quoted string runs a command; the
            // grammar may leave it outside any node.
            "string" | "concatenation" if self.text(node).contains('`') => {
                self.deny(node, stack);
            }
            "string" | "concatenation" | "brace_expression" => {
                push_named_children(node, Visit::Word, stack);
            }
            // zsh reads `$name[…]` as a subscript, which it evaluates.
            "simple_expansion"
                if self.src[node.end_byte()..]
                    .trim_start_matches(|ch: char| ch.is_ascii_alphanumeric() || ch == '_')
                    .starts_with('[') =>
            {
                self.deny(node, stack);
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
            "expansion"
                if simple_expansion_text(self.text(node).trim_start())
                    && !self.braces_in_expansion(node) =>
            {
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
    /// assignments and attribute options that run no code (`-i` and zsh's
    /// and ksh's `-F` make assignments arithmetic, `-n` names another
    /// variable, `-a`/`-A` read a quoted value as elements with subscripts).
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
                "word" if matches!(self.text(child), "-x" | "+x" | "-r" | "-g" | "-p") => {}
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
    // `${OPTIND:=…}` assigns.
    if (rest.starts_with(":=") || rest.starts_with('='))
        && runs_code_from_variable(&inner[..name_end])
    {
        return false;
    }
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
