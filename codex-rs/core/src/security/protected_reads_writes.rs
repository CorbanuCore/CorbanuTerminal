//! #620, #737, #740: the words a command writes to, as spelled, and whether
//! every file it writes is among them; #736: whether it runs a macOS
//! keychain command. Text-based and incomplete; it only decides whether a
//! grant offer is withheld as useless (and explained), never what runs.

use super::super::tainted_action::KEYCHAIN_VERBS;

/// Shells whose `-c` argument is a script.
const SHELLS: [&str; 6] = ["sh", "bash", "zsh", "dash", "ksh", "fish"];

/// Programs whose path arguments (after the first, for `sed`'s script) are
/// written, and those whose last argument is (`mv` also removes the others).
const WRITES_EVERY_ARGUMENT: [&str; 7] =
    ["tee", "touch", "truncate", "rm", "chmod", "mkdir", "rmdir"];
const WRITES_LAST_ARGUMENT: [&str; 5] = ["cp", "mv", "ln", "install", "rsync"];

/// #740: programs that write no file themselves (a redirection of their
/// output may, and is counted), whatever their arguments. #789: `sleep`
/// and other commands that only report or wait, too.
const WRITES_NOTHING: [&str; 46] = [
    "sleep",
    "date",
    "seq",
    "expr",
    "nproc",
    "uptime",
    "cat",
    "ls",
    "head",
    "tail",
    "grep",
    "egrep",
    "fgrep",
    "wc",
    "echo",
    "printf",
    "pwd",
    "stat",
    "diff",
    "cmp",
    "cut",
    "tr",
    "true",
    "false",
    "test",
    "[",
    "which",
    "type",
    "whoami",
    "id",
    "uname",
    "basename",
    "dirname",
    "realpath",
    "readlink",
    "du",
    "df",
    "nl",
    "od",
    "strings",
    "md5",
    "md5sum",
    "shasum",
    "sha256sum",
    "jq",
    "printenv",
];

/// #737: programs whose config file [`writes`] names (`python` for
/// `python -m pip`).
const CONFIG_WRITERS: [&str; 5] = ["git", "npm", "pip", "python", "gh"];

/// #740: `find` options that write or run something.
const FIND_ACTIONS: [&str; 9] = [
    "-exec", "-execdir", "-ok", "-okdir", "-delete", "-fprint", "-fprint0", "-fprintf", "-fls",
];

/// Shell scripts inside scripts (`bash -c '…'`) looked into.
const MAX_SCRIPT_DEPTH: usize = 4;

/// What a command writes, as far as its text shows.
#[derive(Debug, Default, PartialEq, Eq)]
pub(super) struct Writes {
    /// The words it writes to, as spelled.
    pub(super) targets: Vec<String>,
    /// #740: every program it runs is one whose writes are all in
    /// `targets` (a known reader or writer, through a shell's `-c` script),
    /// so a grant could only matter for one of them.
    pub(super) understood: bool,
}

impl Writes {
    fn known(targets: Vec<String>) -> Self {
        Self {
            targets,
            understood: true,
        }
    }

    fn unknown() -> Self {
        Self {
            targets: Vec::new(),
            understood: false,
        }
    }

    fn add(&mut self, other: Writes) {
        self.targets.extend(other.targets);
        self.understood &= other.understood;
    }
}

/// #620: the words a command writes to, as spelled: in a shell script,
/// redirection targets (`>`, `>>`, `2>`, `&>`, `>&file`, `<>`, attached or
/// not), and for each simple command every path argument of `tee`,
/// `touch`, `truncate`, `rm`, `chmod`, `mkdir` and `rmdir`, the destination
/// of `cp`, `mv`, `ln`, `install` and `rsync` (and what `mv` moves away), the
/// files of `sed -i`, `dd of=`, and (#737) the config file of `git config`,
/// `npm config`, `pip config` and `gh config` writes; a shell's `-c` script
/// is looked into. Text-based and incomplete (newlines, `;` inside a word
/// and `|` attached to a word do not separate commands, and quotes inside a
/// script are not kept); such a script is not [`Writes::understood`].
pub(super) fn writes(command: &[String]) -> Writes {
    simple_command_writes(command, MAX_SCRIPT_DEPTH)
}

/// The words of [`writes`].
pub(super) fn write_targets(command: &[String]) -> Vec<String> {
    writes(command).targets
}

/// What one simple command (already split into words) writes.
fn simple_command_writes(words: &[String], depth: usize) -> Writes {
    // Skip `VAR=value` prefixes and `sudo`/`command`/`env`/`exec`/`nohup`.
    let all = words.len();
    let words: Vec<&str> = words
        .iter()
        .map(String::as_str)
        .skip_while(|word| {
            (word.contains('=') && !word.starts_with('-'))
                || matches!(*word, "sudo" | "command" | "env" | "exec" | "nohup")
        })
        .collect();
    let Some((program, arguments)) = words.split_first() else {
        return Writes::known(Vec::new());
    };
    let program = program.rsplit('/').next().unwrap_or(program);
    // A variable set for it may move a config file (`GIT_CONFIG_GLOBAL`,
    // `HOME=. git config --global`, `NPM_CONFIG_USERCONFIG`).
    if words.len() < all
        && CONFIG_WRITERS
            .iter()
            .any(|writer| program.starts_with(writer))
    {
        return Writes::unknown();
    }
    if SHELLS.contains(&program) {
        // Every argument after a `-c` option is the script (and its `$0`,
        // `$1`…, which parse as plain words). A script file is not looked
        // into.
        return arguments
            .iter()
            .skip_while(|argument| {
                !(argument.starts_with('-')
                    && !argument.starts_with("--")
                    && argument.contains('c'))
            })
            .nth(1)
            .map_or_else(Writes::unknown, |script| script_writes(script, depth));
    }
    let paths: Vec<&str> = arguments
        .iter()
        .copied()
        .filter(|word| !word.starts_with('-'))
        .collect();
    let owned = |paths: &[&str]| paths.iter().map(std::string::ToString::to_string).collect();
    if WRITES_EVERY_ARGUMENT.contains(&program) || (program == "mv" && paths.len() >= 2) {
        Writes::known(owned(&paths))
    } else if WRITES_LAST_ARGUMENT.contains(&program) {
        // With one path, `ln -s x` and `install x` write in the current
        // folder, not to it; with `-t DIR` the destination comes first.
        let target_first = arguments.iter().any(|word| {
            word.starts_with("--target-directory")
                || (word.starts_with('-') && !word.starts_with("--") && word.contains('t'))
        });
        if paths.len() >= 2 && !target_first {
            Writes::known(owned(&paths[paths.len() - 1..]))
        } else {
            Writes::unknown()
        }
    } else if program == "sed" {
        // Every path after the script (or all of them with `-e`); a sed
        // script may write (`w`) or run (`e`) more.
        let in_place = arguments.iter().any(|word| word.starts_with("-i"));
        let targets = if in_place {
            let skip = usize::from(!arguments.contains(&"-e")).min(paths.len());
            owned(&paths[skip..])
        } else {
            Vec::new()
        };
        // #789: a script that only prints, deletes or quits lines
        // (`sed -n '1,240p' file`) writes nothing.
        let understood = !in_place
            && sed_scripts(arguments)
                .is_some_and(|scripts| scripts.iter().all(|script| sed_script_only_reads(script)));
        Writes {
            targets,
            understood,
        }
    } else if program == "dd" {
        Writes::known(
            arguments
                .iter()
                .filter_map(|word| word.strip_prefix("of="))
                .map(str::to_string)
                .collect(),
        )
    } else if program == "git" {
        git_writes(arguments)
    } else if program == "npm" {
        npm_writes(arguments)
    } else if matches!(program, "pip" | "pip3") {
        pip_writes(arguments)
    } else if program.starts_with("python") && arguments.starts_with(&["-m", "pip"]) {
        pip_writes(&arguments[2..])
    } else if program == "gh" {
        match paths.as_slice() {
            ["config", "set", ..] => Writes::known(vec!["~/.config/gh/config.yml".to_string()]),
            ["config", "get" | "list", ..] => Writes::known(Vec::new()),
            _ => Writes::unknown(),
        }
    } else if program == "find" {
        if arguments.iter().any(|word| FIND_ACTIONS.contains(word)) {
            Writes::unknown()
        } else {
            Writes::known(Vec::new())
        }
    } else if WRITES_NOTHING.contains(&program) {
        Writes::known(Vec::new())
    } else {
        Writes::unknown()
    }
}

/// #789: the scripts of a `sed` that takes them as arguments (`-e`,
/// `--expression`, or else its first operand), or `None` when it may write
/// or read a script from elsewhere: in place (`-i`, `--in-place`), a script
/// file (`-f`), an option this does not know, `-l` (a value on GNU sed, a
/// flag on macOS), or an option after an operand (macOS stops reading
/// options at the first operand; GNU sed does not).
fn sed_scripts<'a>(arguments: &[&'a str]) -> Option<Vec<&'a str>> {
    let (mut scripts, mut operands) = (Vec::new(), Vec::new());
    let mut expression = false;
    let mut words = arguments.iter().copied();
    while let Some(word) = words.next() {
        if word == "--" {
            operands.extend(words.by_ref());
            break;
        }
        let option = word.starts_with('-') && word != "-";
        if option && !operands.is_empty() {
            return None;
        }
        if let Some(long) = word.strip_prefix("--") {
            match long {
                "expression" => {
                    scripts.push(words.next()?);
                    expression = true;
                }
                _ if long.starts_with("expression=") => {
                    scripts.push(&word["--expression=".len()..]);
                    expression = true;
                }
                "quiet" | "silent" | "posix" | "debug" | "regexp-extended" | "null-data"
                | "zero-terminated" | "separate" | "unbuffered" => {}
                _ => return None,
            }
        } else if let Some(flags) = word.strip_prefix('-').filter(|flags| !flags.is_empty()) {
            for (index, flag) in flags.char_indices() {
                match flag {
                    'n' | 'E' | 'r' | 's' | 'u' | 'z' => {}
                    // The rest of the word, or the next one, is its value.
                    'e' => {
                        let rest = &flags[index + 1..];
                        scripts.push(if rest.is_empty() { words.next()? } else { rest });
                        expression = true;
                        break;
                    }
                    _ => return None,
                }
            }
        } else {
            // Review r2: an operand a shell expands (`"$f"` holding
            // `-e1wx`) or a glob may become an option or a script.
            if word.contains(['$', '*', '?', '[']) {
                return None;
            }
            operands.push(word);
        }
    }
    if !expression {
        scripts.push(operands.first().copied()?);
    }
    Some(scripts)
}

/// #789: whether a `sed` script only selects lines by number and prints,
/// deletes, lists or counts them, or quits: no `w` (write), `e` (run), `r`
/// (read) or substitution, whose flags could write or run. A `$` is the
/// last line only where no shell would expand it (`1,$`, `$!N`: `$!` is a
/// process number at most); one a shell may have expanded (`"$p"`, `${p}`,
/// `$@`, zsh's `$=p`) is refused, since the text no longer shows whether it
/// was quoted, and so are `$p` and `$=` (the last line). Only scripts given
/// with `-e` get here with a `$`: a first operand holding one is refused as
/// an operand a shell expands. A script that starts with `=` and goes on
/// is refused too: zsh turns `=nl` into a program's path.
fn sed_script_only_reads(script: &str) -> bool {
    let chars: Vec<char> = script.chars().collect();
    !script.trim().is_empty()
        && (script == "=" || !script.starts_with('='))
        && chars.iter().enumerate().all(|(index, c)| {
            c.is_ascii_digit()
                || " \t\n,;!{}pdqQ=nNl".contains(*c)
                || (*c == '$'
                    && chars
                        .get(index + 1)
                        .is_none_or(|next| next.is_whitespace() || ",;!}".contains(*next)))
        })
}

/// #737: the config file `git config` writes (`--global`: `~/.gitconfig`;
/// `--system`; `--file`; otherwise the repository's), and nothing for a
/// read (`--get`, `--list`, `get`, a key alone). Other git commands are not
/// understood.
fn git_writes(arguments: &[&str]) -> Writes {
    // Git's own options before the subcommand.
    let mut folder: Option<String> = None;
    let mut git_dir: Option<String> = None;
    let mut words = arguments.iter().copied();
    let subcommand = loop {
        match words.next() {
            None => return Writes::known(Vec::new()),
            Some("-C") => {
                let Some(next) = words.next() else {
                    return Writes::unknown();
                };
                folder = Some(join(folder.as_deref(), next));
            }
            Some("--git-dir") => git_dir = words.next().map(str::to_string),
            Some("-c" | "--work-tree" | "--namespace" | "--config-env" | "--exec-path") => {
                words.next();
            }
            Some(word) if word.starts_with("--git-dir=") => {
                git_dir = Some(word["--git-dir=".len()..].to_string());
            }
            Some(word) if word.starts_with('-') => {}
            Some(word) => break word,
        }
    };
    if subcommand != "config" {
        return Writes::unknown();
    }
    let arguments: Vec<&str> = words.collect();
    let (mut write, mut read) = (false, false);
    let (mut scope, mut file) = ("local", None::<&str>);
    let mut positionals = Vec::new();
    let mut index = 0;
    while let Some(word) = arguments.get(index).copied() {
        match word {
            "--global" | "--system" | "--local" | "--worktree" => scope = word,
            "-f" | "--file" => {
                index += 1;
                file = arguments.get(index).copied();
            }
            "--blob" => {
                index += 1;
                read = true;
            }
            "--type" | "--default" | "--comment" | "--value" => index += 1,
            "--add" | "--unset" | "--unset-all" | "--replace-all" | "--rename-section"
            | "--remove-section" | "-e" | "--edit" => write = true,
            "--get" | "--get-all" | "--get-regexp" | "--get-urlmatch" | "--get-color"
            | "--get-colorbool" | "-l" | "--list" => read = true,
            _ if word.starts_with("--file=") => file = Some(&word["--file=".len()..]),
            _ if word.starts_with("--blob=") => read = true,
            _ if word.starts_with('-') => {}
            _ => positionals.push(word),
        }
        index += 1;
    }
    match positionals.first().copied() {
        Some("set" | "unset" | "rename-section" | "remove-section" | "edit") => write = true,
        Some("get" | "list") => read = true,
        _ => {}
    }
    if !write && (read || positionals.len() < 2) {
        return Writes::known(Vec::new());
    }
    let in_folder = |path: &str| {
        if path.starts_with(['/', '~', '$']) {
            path.to_string()
        } else {
            join(folder.as_deref(), path)
        }
    };
    let repository = || in_folder(git_dir.as_deref().unwrap_or(".git"));
    let target = match (file, scope) {
        (Some(file), _) => in_folder(file),
        (None, "--global") => "~/.gitconfig".to_string(),
        (None, "--system") => "/etc/gitconfig".to_string(),
        (None, "--worktree") => format!("{}/config.worktree", repository()),
        (None, _) => format!("{}/config", repository()),
    };
    Writes::known(vec![target])
}

/// #737: the file `npm config set` (or `delete`, `edit`, `fix`, and `npm
/// set`) writes: `~/.npmrc`, `--userconfig`'s, or the project's `.npmrc`
/// (`--location=project`); a global write is not understood (its file is in
/// npm's install prefix). Other npm commands are not understood.
fn npm_writes(arguments: &[&str]) -> Writes {
    let (mut user_config, mut location) = (None::<&str>, "user");
    let mut positionals = Vec::new();
    let mut index = 0;
    while let Some(word) = arguments.get(index).copied() {
        match word {
            "--userconfig" => {
                index += 1;
                user_config = arguments.get(index).copied();
            }
            "--location" | "-L" => {
                index += 1;
                location = arguments.get(index).copied().unwrap_or("user");
            }
            "-g" | "--global" => location = "global",
            _ if word.starts_with("--userconfig=") => {
                user_config = Some(&word["--userconfig=".len()..]);
            }
            _ if word.starts_with("--location=") => location = &word["--location=".len()..],
            _ if word.starts_with('-') => {}
            _ => positionals.push(word),
        }
        index += 1;
    }
    let verb = match positionals.as_slice() {
        ["config" | "c", verb, ..] => *verb,
        ["set" | "get", ..] => positionals[0],
        _ => return Writes::unknown(),
    };
    match (verb, user_config, location) {
        ("get" | "list" | "ls", _, _) => Writes::known(Vec::new()),
        ("set" | "delete" | "edit" | "fix", Some(file), _) => Writes::known(vec![file.to_string()]),
        ("set" | "delete" | "edit" | "fix", None, "project") => {
            Writes::known(vec![".npmrc".to_string()])
        }
        ("set" | "delete" | "edit" | "fix", None, "user") => {
            Writes::known(vec!["~/.npmrc".to_string()])
        }
        _ => Writes::unknown(),
    }
}

/// #737: the file `pip config set` (or `unset`, `edit`) writes: your own
/// pip config (`--user`, the default) or the system one (`--global`); a
/// virtualenv's (`--site`) is not understood. Other pip commands are not
/// understood.
fn pip_writes(arguments: &[&str]) -> Writes {
    let positionals: Vec<&str> = arguments
        .iter()
        .copied()
        .filter(|word| !word.starts_with('-'))
        .collect();
    let has = |flag: &str| arguments.contains(&flag);
    match positionals.as_slice() {
        ["config", "get" | "list" | "debug", ..] => Writes::known(Vec::new()),
        ["config", "set" | "unset" | "edit", ..] if has("--site") => Writes::unknown(),
        ["config", "set" | "unset" | "edit", ..] if has("--global") => {
            Writes::known(vec![if cfg!(target_os = "macos") {
                "/Library/Application Support/pip/pip.conf".to_string()
            } else {
                "/etc/pip.conf".to_string()
            }])
        }
        ["config", "set" | "unset" | "edit", ..] => {
            Writes::known(vec!["~/.config/pip/pip.conf".to_string()])
        }
        _ => Writes::unknown(),
    }
}

/// `path` relative to `folder` (a `-C` folder), as spelled.
fn join(folder: Option<&str>, path: &str) -> String {
    match folder {
        Some(folder) if !path.starts_with(['/', '~', '$']) => {
            format!("{}/{path}", folder.trim_end_matches('/'))
        }
        _ => path.to_string(),
    }
}

/// What a shell script writes: its redirection targets, and those of each
/// simple command in it (see [`writes`]). Not understood when a word may
/// hide another command (newlines, substitutions, subshells, groups, or a
/// separator inside a word).
fn script_writes(script: &str, depth: usize) -> Writes {
    let Some(depth) = depth.checked_sub(1) else {
        return Writes::unknown();
    };
    let mut writes = Writes::known(Vec::new());
    if script.contains(['\n', '`']) || ["$(", "<(", ">("].iter().any(|s| script.contains(s)) {
        writes.understood = false;
    }
    let words = shlex::split(script)
        .unwrap_or_else(|| script.split_whitespace().map(str::to_string).collect());
    let mut segment: Vec<String> = Vec::new();
    let mut redirect_next = false;
    let mut input_next = false;
    for word in words {
        let ends_segment = word.ends_with(';') || matches!(word.as_str(), "&&" | "||" | "|" | "&");
        let trimmed = word.trim_end_matches(';');
        if redirect_next {
            redirect_next = false;
            writes
                .targets
                .push(trimmed.trim_end_matches(['&', '|', ')']).to_string());
        } else if input_next {
            // `< file` is read, not written.
            input_next = false;
        } else if let Some(target) = trimmed.strip_prefix("<>") {
            // Opened for reading and writing.
            if target.is_empty() {
                redirect_next = !ends_segment;
            } else {
                writes.targets.push(target.to_string());
            }
        } else if trimmed.starts_with('<') {
            // `<file`, `<<EOF`; a lone `<` takes the next word.
            input_next = trimmed == "<" && !ends_segment;
        } else if let Some(index) = trimmed.find('>') {
            // `>`, `>>`, `2>`, `&>`, `>|`, `>&file`, alone or attached to its
            // target or to the word before (`x>>~/.zshrc`); `2>&1` and `>&-`
            // write no file.
            let before =
                trimmed[..index].trim_end_matches(|c: char| c.is_ascii_digit() || c == '&');
            if !before.is_empty() {
                segment.push(before.to_string());
            }
            let target = trimmed[index..].trim_start_matches(['>', '|']);
            let target = target.strip_prefix('&').unwrap_or(target).trim();
            let duplicates = target.starts_with(|c: char| c.is_ascii_digit() || c == '-')
                && trimmed[index..].contains(">&");
            match (duplicates, target.is_empty()) {
                // A file descriptor, not a file.
                (true, _) => {}
                (false, true) => redirect_next = !ends_segment,
                (false, false) => writes.targets.push(target.to_string()),
            }
        } else if !trimmed.is_empty() && !matches!(trimmed, "&&" | "||" | "|" | "&") {
            if trimmed.contains([';', '|', '&', '(', ')', '{', '}']) {
                writes.understood = false;
            }
            segment.push(trimmed.to_string());
        }
        if ends_segment {
            writes.add(simple_command_writes(&std::mem::take(&mut segment), depth));
        }
    }
    writes.add(simple_command_writes(&segment, depth));
    writes
}

/// #736: whether `command` runs macOS's `security` tool with a keychain
/// verb ([`KEYCHAIN_VERBS`]) as its subcommand, in command position (after
/// `VAR=value`, `sudo`, `env`, `command`, `exec`, `nohup` and their
/// options), also inside a shell's `-c` script. Text-based: a match only
/// adds the explanation (keychain commands are never granted anyway).
pub(super) fn keychain_command(command: &[String]) -> bool {
    runs_program(command, MAX_SCRIPT_DEPTH, &keychain_invocation)
}

/// #826 follow-up: whether `command` runs a program that always needs a
/// user namespace (`unshare`, `bwrap`, rootless Podman, …), found as
/// [`keychain_command`] finds `security`. Text-based: a match only adds the
/// approval prompt's note.
pub(super) fn user_namespace_command(command: &[String]) -> bool {
    runs_program(command, MAX_SCRIPT_DEPTH, &|program, _| {
        codex_sandboxing::user_namespaces::needs_user_namespace(program)
    })
}

/// `security [-hilqv] [-p prompt] command …` with a keychain verb.
fn keychain_invocation(program: &str, arguments: &[&str]) -> bool {
    if program != "security" {
        return false;
    }
    let mut arguments = arguments.iter();
    while let Some(word) = arguments.next() {
        match *word {
            "-p" => {
                arguments.next();
            }
            _ if word.starts_with('-') => {}
            verb => return KEYCHAIN_VERBS.contains(&verb.trim_end_matches(';')),
        }
    }
    false
}

/// Whether `words` runs a program `matches` accepts (its file name and
/// arguments), in command position (after `VAR=value`, `sudo`, `env`,
/// `command`, `exec`, `nohup` and their options), also inside a shell's
/// `-c` script, as deep as `depth` scripts.
fn runs_program(words: &[String], depth: usize, matches: &dyn Fn(&str, &[&str]) -> bool) -> bool {
    let mut words = words.iter().map(String::as_str);
    let program = loop {
        match words.next() {
            None => return false,
            Some("sudo" | "command" | "env" | "exec" | "nohup") => {}
            // sudo's options that take a value.
            Some("-u" | "-g" | "-C" | "-h" | "-p" | "-U" | "-r" | "-t") => {
                words.next();
            }
            Some(word)
                if word.starts_with('-') || (word.contains('=') && !word.starts_with('=')) => {}
            Some(word) => break word.rsplit('/').next().unwrap_or(word),
        }
    };
    let arguments: Vec<&str> = words.collect();
    if SHELLS.contains(&program) {
        let Some(depth) = depth.checked_sub(1) else {
            return false;
        };
        return arguments
            .iter()
            .skip_while(|argument| {
                !(argument.starts_with('-')
                    && !argument.starts_with("--")
                    && argument.contains('c'))
            })
            .nth(1)
            .is_some_and(|script| {
                script_segments(script)
                    .iter()
                    .any(|segment| runs_program(segment, depth, matches))
            });
    }
    matches(program, &arguments)
}

/// The simple commands of a shell script, split at `;`, `&&`, `||`, `|`,
/// `&` and newlines (text-based, as [`script_writes`]).
fn script_segments(script: &str) -> Vec<Vec<String>> {
    let mut segments = Vec::new();
    for line in script.lines() {
        let words = shlex::split(line)
            .unwrap_or_else(|| line.split_whitespace().map(str::to_string).collect());
        let mut segment = Vec::new();
        for word in words {
            let ends = word.ends_with(';') || matches!(word.as_str(), "&&" | "||" | "|" | "&");
            let trimmed = word.trim_end_matches(';');
            if !trimmed.is_empty() && !matches!(trimmed, "&&" | "||" | "|" | "&") {
                segment.push(trimmed.to_string());
            }
            if ends {
                segments.push(std::mem::take(&mut segment));
            }
        }
        segments.push(segment);
    }
    segments
}
