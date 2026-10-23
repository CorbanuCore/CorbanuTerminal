//! #623: the paths a command creates links to, read from its text (see
//! `protected_links`). Text-based and bounded: it decides only whether a
//! command is refused, and finding too much refuses more, never less.

use std::path::Path;

/// Shells whose `-c` argument is a script.
const SHELLS: [&str; 6] = ["sh", "bash", "zsh", "dash", "ksh", "fish"];
/// Interpreters whose `-c`/`-e` argument (or heredoc) is code.
const INTERPRETERS: [&str; 10] = [
    "python", "python3", "pypy3", "py", "node", "bun", "deno", "perl", "ruby", "php",
];
/// Programs that run the command after their own options (`sudo -u root
/// ln …`, `env -i ln …`, `timeout 5 ln …`, `find . -exec ln …`): the first
/// later word naming a program below is read as that command.
const WRAPPERS: [&str; 22] = [
    "sudo",
    "doas",
    "env",
    "nice",
    "nohup",
    "timeout",
    "time",
    "command",
    "exec",
    "builtin",
    "xargs",
    "find",
    "stdbuf",
    "ionice",
    "chrt",
    "taskset",
    "caffeinate",
    "busybox",
    "uv",
    "poetry",
    "pipenv",
    "npx",
];
/// Shell words that may start a command without being its program.
const KEYWORDS: [&str; 12] = [
    "if", "then", "else", "elif", "do", "while", "until", "!", "{", "(", "fi", "done",
];
/// Calls in code that create a link (`os.symlink(`, `fs.symlinkSync(`,
/// `Path.symlink_to(`, `os.link(`, Ruby's `FileUtils.ln_s(`, perl's
/// `symlink "a", "b"`…), matched where no letter, digit or `_` comes before
/// them, so `unlink(`, `readlink(` and `is_symlink(` do not count.
const LINK_CALLS: [&str; 15] = [
    "symlink(",
    "symlink ",
    "symlinkSync(",
    "symlink_to(",
    "hardlink_to(",
    "link_to(",
    "link(",
    "linkSync(",
    "ln_s(",
    "ln_sf(",
    "ln_s ",
    "ln_sf ",
    "ln(",
    "CreateSymbolicLink(",
    "CreateHardLink(",
];
/// Scripts inside scripts (and code) looked into.
const MAX_DEPTH: usize = 4;
/// Paths (and folders) kept for one command; past it the command is
/// refused (see [`Found::overflow`]).
const MAX_CANDIDATES: usize = 256;

/// A path a command links to, as spelled, and the folders a relative one may
/// be read from besides the command's own.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Candidate {
    pub(crate) target: String,
    /// The link's folder (a relative target is relative to it), and folders
    /// the script changes to.
    pub(crate) link_folders: Vec<String>,
}

/// The paths `command` creates links to: `ln` (`-s` or hard), `cp -s`/`-l`,
/// `mklink`, PowerShell's `New-Item -ItemType SymbolicLink|Junction|HardLink`,
/// also inside `sh -c` scripts, behind wrappers ([`WRAPPERS`]) and shell
/// keywords, and code an interpreter runs that calls a link function or runs
/// such a command (`os.system('ln -s …')`, `['ln', '-s', …]`).
///
/// A relative target is read from the command's folder, the link's folder
/// and every folder a script changes to (`cd sub && ln -s ../.agents x`):
/// more folders than the one it is relative to, so a link may be refused that
/// would not point into a protected path, never the reverse.
pub(crate) fn link_targets(command: &[String]) -> Found {
    let mut scan = Scan::default();
    simple_command(command, MAX_DEPTH, &mut scan);
    let Scan {
        mut found,
        folders,
        overflow,
    } = scan;
    for candidate in &mut found {
        for folder in &folders {
            if !candidate.link_folders.contains(folder) {
                candidate.link_folders.push(folder.clone());
            }
        }
    }
    Found {
        candidates: found,
        overflow,
    }
}

/// Whether `command`'s text, read loosely, may create a link: a word `ln`,
/// `gln` or `mklink` anywhere (also inside `$(…)`, quotes, `eval` or a pipe
/// into a shell), a link call in code, or a PowerShell link item type. With
/// a `.codex`/`.agents` name anywhere in it the command is refused (see
/// `protected_links`), so a variable or substitution refuses more, not less.
pub(crate) fn makes_links(command: &[String]) -> bool {
    let text = command.join(" ");
    let lower = text.to_ascii_lowercase();
    calls_link(&text)
        || ["symboliclink", "hardlink", "junction"]
            .iter()
            .any(|kind| lower.contains(kind))
        || text
            .split(|c: char| {
                c.is_whitespace()
                    || matches!(
                        c,
                        '/' | '\\'
                            | '$'
                            | '('
                            | ')'
                            | '{'
                            | '}'
                            | '"'
                            | '\''
                            | '`'
                            | ';'
                            | '&'
                            | '|'
                            | '<'
                            | '>'
                            | '='
                    )
            })
            .any(|word| matches!(program_name(word).as_str(), "ln" | "gln" | "mklink"))
}

/// What [`link_targets`] found.
#[derive(Debug, Default)]
pub(crate) struct Found {
    pub(crate) candidates: Vec<Candidate>,
    /// More was found than is looked at: the command is refused, so a
    /// protected target cannot hide behind many others.
    pub(crate) overflow: bool,
}

#[derive(Default)]
struct Scan {
    found: Vec<Candidate>,
    /// Folders a script changes to.
    folders: Vec<String>,
    overflow: bool,
}

pub(super) fn program_name(word: &str) -> String {
    let name = word.rsplit(['/', '\\']).next().unwrap_or(word);
    let name = name.to_ascii_lowercase();
    let name = name
        .strip_suffix(".exe")
        .map(str::to_string)
        .unwrap_or(name);
    // `python3.12` reads as `python3`.
    match name.split_once('.') {
        Some((stem, version))
            if stem.starts_with("python") && version.chars().all(|c| c.is_ascii_digit()) =>
        {
            stem.to_string()
        }
        _ => name,
    }
}

/// Whether `name` is a program [`simple_command`] reads.
fn known(name: &str) -> bool {
    SHELLS.contains(&name)
        || INTERPRETERS.contains(&name)
        || matches!(
            name,
            "ln" | "gln"
                | "cp"
                | "gcp"
                | "mklink"
                | "new-item"
                | "ni"
                | "cmd"
                | "powershell"
                | "pwsh"
                | "cd"
                | "pushd"
        )
}

/// The command's words from its program on: shell keywords, grouping
/// (`(`, `{`, `$(`, `` ` ``, also in `x=$(…`) and `VAR=value` prefixes
/// dropped, closing `)`/`}` removed from word ends, and a wrapper's own
/// options skipped.
fn command_words(words: &[String]) -> Vec<&str> {
    let mut command: Vec<&str> = Vec::new();
    for word in words {
        // `f() {`, `f(){`: a function's header; its body is the command.
        if command.is_empty() && word.contains("()") {
            continue;
        }
        let word = word.trim_end_matches([')', '}', '`']);
        if !command.is_empty() {
            command.push(word);
            continue;
        }
        let word = match word.find("$(") {
            Some(index) => &word[index + 2..],
            None => word,
        };
        let word = word.trim_start_matches(['(', '{', '`']);
        if word.is_empty()
            || KEYWORDS.contains(&word)
            || (word.contains('=') && !word.starts_with('-'))
            || word.ends_with('{')
        {
            continue;
        }
        command.push(word);
    }
    match command.first() {
        Some(program) if WRAPPERS.contains(&program_name(program).as_str()) => {
            match command[1..]
                .iter()
                .position(|word| known(&program_name(word)))
            {
                Some(index) => command[1 + index..].to_vec(),
                None => Vec::new(),
            }
        }
        _ => command,
    }
}

/// One simple command (already split into words).
fn simple_command(words: &[String], depth: usize, found: &mut Scan) {
    let words = command_words(words);
    let Some((program, arguments)) = words.split_first() else {
        return;
    };
    let name = program_name(program);
    let name = name.as_str();
    if SHELLS.contains(&name) {
        let script = arguments
            .iter()
            .skip_while(|argument| {
                !(argument.starts_with('-')
                    && !argument.starts_with("--")
                    && argument.contains('c'))
            })
            .nth(1);
        if let Some(script) = script {
            shell_script(script, depth, found);
        }
        return;
    }
    match name {
        "cd" | "pushd" => {
            if let Some(folder) = arguments.iter().find(|word| !word.starts_with('-')) {
                if found.folders.len() < MAX_CANDIDATES {
                    found.folders.push((*folder).to_string());
                } else {
                    found.overflow = true;
                }
            }
        }
        "ln" | "gln" => ln(arguments, found),
        "cp" | "gcp" => cp(arguments, found),
        "mklink" => {
            // `mklink [/D|/H|/J] link target`
            let paths: Vec<&&str> = arguments
                .iter()
                .filter(|word| !word.starts_with('/'))
                .collect();
            if let [link, target] = paths.as_slice() {
                push(found, target, vec![parent_of(link)]);
            }
        }
        "new-item" | "ni" => new_item(arguments, found),
        // `cmd /c a && mklink …`: cmd's own quoting is not shell quoting.
        "cmd" => {
            if let Some(index) = arguments
                .iter()
                .position(|word| word.eq_ignore_ascii_case("/c") || word.eq_ignore_ascii_case("/k"))
            {
                let script = arguments[index + 1..].join(" ");
                for statement in script.split(['&', '|']) {
                    let words: Vec<String> = statement
                        .split_whitespace()
                        .map(|word| word.trim_matches('"').to_string())
                        .collect();
                    simple_command(&words, depth.saturating_sub(1), found);
                }
            }
        }
        "powershell" | "pwsh" => {
            if let Some(index) = arguments.iter().position(|word| {
                word.eq_ignore_ascii_case("-command") || word.eq_ignore_ascii_case("-c")
            }) {
                let script = arguments[index + 1..].join(" ");
                for statement in script.split([';', '\n', '|']) {
                    let words: Vec<String> = shlex::split(statement).unwrap_or_else(|| {
                        statement.split_whitespace().map(str::to_string).collect()
                    });
                    simple_command(&words, depth.saturating_sub(1), found);
                }
                // `[IO.File]::CreateSymbolicLink('ag', '.agents')`.
                code(&script, depth, found);
            }
        }
        _ if INTERPRETERS.contains(&name) => {
            // Inline code (`-c`, `-e`, `--eval`); a heredoc arrives through
            // the shell script instead.
            for word in arguments {
                code(word, depth, found);
            }
        }
        _ => {}
    }
}

/// The option clusters of `ln` and `cp` taking a value: `-t folder` (or
/// `-tfolder`, `-st folder`, `--target-directory=folder`) and `-S suffix`.
struct Parsed<'a> {
    paths: Vec<&'a str>,
    folder: Option<String>,
    flags: String,
}

fn parse<'a>(arguments: &[&'a str]) -> Parsed<'a> {
    let mut parsed = Parsed {
        paths: Vec::new(),
        folder: None,
        flags: String::new(),
    };
    let mut options_done = false;
    let mut iter = arguments.iter();
    while let Some(word) = iter.next() {
        if options_done || !word.starts_with('-') || *word == "-" {
            parsed.paths.push(word);
        } else if *word == "--" {
            options_done = true;
        } else if let Some(long) = word.strip_prefix("--") {
            match long.split_once('=') {
                Some(("target-directory", folder)) => parsed.folder = Some(folder.to_string()),
                Some(_) => {}
                None if matches!(long, "target-directory" | "suffix") => {
                    let value = iter.next().map(|value| (*value).to_string());
                    if long == "target-directory" {
                        parsed.folder = value;
                    }
                }
                None => parsed.flags.push_str(&format!(" --{long} ")),
            }
        } else {
            let cluster = &word[1..];
            parsed.flags.push(' ');
            match cluster.find(['t', 'S']) {
                Some(index) => {
                    parsed.flags.push_str(&cluster[..index]);
                    let rest = &cluster[index + 1..];
                    let value = if rest.is_empty() {
                        iter.next().map(|value| (*value).to_string())
                    } else {
                        Some(rest.to_string())
                    };
                    if cluster.as_bytes()[index] == b't' {
                        parsed.folder = value;
                    }
                }
                None => parsed.flags.push_str(cluster),
            }
        }
    }
    parsed
}

/// `ln [options] target… [link | folder]`, `ln -t folder target…`.
fn ln(arguments: &[&str], found: &mut Scan) {
    let parsed = parse(arguments);
    link_paths(&parsed, found);
}

/// `cp -s`/`--symbolic-link` and `cp -l`/`--link` make links.
fn cp(arguments: &[&str], found: &mut Scan) {
    let parsed = parse(arguments);
    let links = parsed.flags.contains("--symbolic-link")
        || parsed.flags.contains("--link")
        || parsed
            .flags
            .split(' ')
            .filter(|flags| !flags.starts_with("--"))
            .any(|flags| flags.contains('s') || flags.contains('l'));
    if links && (parsed.folder.is_some() || parsed.paths.len() >= 2) {
        link_paths(&parsed, found);
    }
}

fn link_paths(parsed: &Parsed<'_>, found: &mut Scan) {
    let (targets, link_folders): (&[&str], Vec<String>) =
        match (&parsed.folder, parsed.paths.as_slice()) {
            (Some(folder), paths) => (paths, vec![folder.clone()]),
            (None, [target]) => (std::slice::from_ref(target), Vec::new()),
            (None, [targets @ .., last]) => (targets, vec![parent_of(last), (*last).to_string()]),
            (None, []) => (&[], Vec::new()),
        };
    for target in targets {
        push(found, target, link_folders.clone());
    }
}

/// PowerShell `New-Item -ItemType SymbolicLink|Junction|HardLink -Path link
/// -Target target` (or `-Value`).
fn new_item(arguments: &[&str], found: &mut Scan) {
    let value_of = |names: &[&str]| {
        arguments
            .iter()
            .position(|word| names.iter().any(|name| word.eq_ignore_ascii_case(name)))
            .and_then(|index| arguments.get(index + 1))
            .map(|value| value.trim_matches(['\'', '"']).to_string())
    };
    let is_link = value_of(&["-ItemType", "-Type"]).is_some_and(|kind| {
        ["symboliclink", "junction", "hardlink"].contains(&kind.to_ascii_lowercase().as_str())
    });
    if !is_link {
        return;
    }
    if let Some(target) = value_of(&["-Target", "-Value"]) {
        let link_folders = value_of(&["-Path", "-Name"])
            .map(|link| vec![parent_of(&link)])
            .unwrap_or_default();
        push(found, &target, link_folders);
    }
}

fn parent_of(path: &str) -> String {
    Path::new(path)
        .parent()
        .map(|parent| parent.to_string_lossy().into_owned())
        .unwrap_or_default()
}

fn push(found: &mut Scan, target: &str, link_folders: Vec<String>) {
    let target = target.trim_matches(['\'', '"']).to_string();
    let candidate = Candidate {
        target,
        link_folders,
    };
    if candidate.target.is_empty() || found.found.contains(&candidate) {
        return;
    }
    if found.found.len() < MAX_CANDIDATES {
        found.found.push(candidate);
    } else {
        found.overflow = true;
    }
}

/// A shell script: each simple command in it, and, when it runs an
/// interpreter over several lines (a heredoc), that code.
fn shell_script(script: &str, depth: usize, found: &mut Scan) {
    let Some(depth) = depth.checked_sub(1) else {
        return;
    };
    let mut segment: Vec<String> = Vec::new();
    let mut runs_interpreter = false;
    let mut finish = |segment: &mut Vec<String>, found: &mut Scan| {
        if command_words(segment)
            .first()
            .is_some_and(|program| INTERPRETERS.contains(&program_name(program).as_str()))
        {
            runs_interpreter = true;
        }
        simple_command(&std::mem::take(segment), depth, found);
    };
    // A line ends a command (a quoted string over several lines is cut into
    // words too), and so do `;`, `&&`, `||`, `|` and `&` between words (not
    // inside quoted code).
    // A backslash at a line's end continues it.
    let joined = script.replace("\\\r\n", " ").replace("\\\n", " ");
    for line in joined.lines() {
        let line = spaced_operators(line);
        let words = shlex::split(&line)
            .unwrap_or_else(|| line.split_whitespace().map(str::to_string).collect());
        for word in words {
            if matches!(word.as_str(), "&&" | "||" | "|" | "&" | ";") {
                finish(&mut segment, found);
            } else if let Some(word) = word.strip_suffix(';') {
                if !word.is_empty() {
                    segment.push(word.to_string());
                }
                finish(&mut segment, found);
            } else {
                segment.push(word);
            }
        }
        finish(&mut segment, found);
    }
    // Code fed through a heredoc (`python3 - <<'EOF'`), or over several
    // lines (cut into words above).
    if runs_interpreter && (script.contains("<<") || script.contains('\n')) {
        code(script, depth, found);
    }
}

/// `line` with a space on each side of every `;`, `&` and `|` outside
/// quotes (`true;ln …`, `cd x&&ln …`), so they separate words.
fn spaced_operators(line: &str) -> String {
    let mut spaced = String::with_capacity(line.len() + 8);
    let mut quote = None;
    let mut escaped = false;
    for c in line.chars() {
        if escaped {
            escaped = false;
        } else if quote == Some('\'') {
            if c == '\'' {
                quote = None;
            }
        } else if c == '\\' {
            escaped = true;
        } else if let Some(open) = quote {
            if c == open {
                quote = None;
            }
        } else if matches!(c, '\'' | '"' | '`') {
            quote = Some(c);
        } else if matches!(c, ';' | '&' | '|') {
            spaced.push(' ');
            spaced.push(c);
            spaced.push(' ');
            continue;
        }
        spaced.push(c);
    }
    spaced
}

/// Code an interpreter runs: when it calls a link function, every string it
/// quotes is a candidate target; a string that is a command (`os.system('ln
/// -s …')`) or the start of one (`['ln', '-s', …]`) is read as one.
fn code(text: &str, depth: usize, found: &mut Scan) {
    let Some(depth) = depth.checked_sub(1) else {
        return;
    };
    let (literals, overflow) = literals(text);
    // Past the bound only where a link could hide: code that calls a link
    // function or names `ln`.
    found.overflow |= overflow
        && (calls_link(text)
            || ["'ln'", "\"ln\"", "ln -", "mklink"]
                .iter()
                .any(|word| text.contains(word)));
    if calls_link(text) {
        for literal in &literals {
            if !literal.contains('\n') {
                push(found, literal, Vec::new());
            }
        }
    }
    for (index, literal) in literals.iter().enumerate() {
        // Code quoted inside the code (`python3 -c "…os.symlink('a', 'b')…"`
        // read whole from a script).
        if literal.contains(['\'', '"', '`']) {
            code(literal, depth, found);
        }
        if literal.contains(char::is_whitespace) {
            shell_script(literal, depth, found);
        } else if known(&program_name(literal)) {
            simple_command(&literals[index..], depth, found);
        }
    }
}

/// The strings `code` quotes (`'…'`, `"…"`, `` `…` ``), in order, and
/// whether there were more than are kept.
fn literals(code: &str) -> (Vec<String>, bool) {
    let mut literals = Vec::new();
    let mut chars = code.chars();
    while let Some(c) = chars.next() {
        if !matches!(c, '\'' | '"' | '`') {
            continue;
        }
        let mut literal = String::new();
        let mut escaped = false;
        for next in chars.by_ref() {
            if escaped {
                literal.push(next);
                escaped = false;
            } else if next == '\\' {
                escaped = true;
            } else if next == c {
                break;
            } else {
                literal.push(next);
            }
        }
        if literal.is_empty() {
            continue;
        }
        if literals.len() == MAX_CANDIDATES || literal.len() > 4096 {
            return (literals, true);
        }
        literals.push(literal);
    }
    (literals, false)
}

/// Whether `code` calls one of [`LINK_CALLS`].
pub(super) fn calls_link(code: &str) -> bool {
    LINK_CALLS.iter().any(|call| {
        code.match_indices(call).any(|(index, _)| {
            !code[..index]
                .chars()
                .next_back()
                .is_some_and(|before| before.is_alphanumeric() || before == '_')
        })
    })
}
