//! D48 (c) (Travis, 2026-10-20, security sign-off, #891): after untrusted
//! content under Moderate on Linux, the one-run user-namespace allowance is
//! offered only for a command whose program is a known tool that needs one.
//!
//! An allow-list, decided on the parsed command, never by searching its text:
//! - The command is exactly one simple command: either the program's own
//!   words, or a `bash`, `sh` or `zsh` `-c`/`-lc` script that tree-sitter
//!   parses into one plain command (no `;`, `&&`, `|`, newlines,
//!   substitutions, expansions, redirections, assignments or functions).
//! - Its program, after the wrappers `env`, `sudo` and `nohup` (and, as the
//!   script's first word, the shell builtins `exec` and `command`), with only
//!   the options listed here, is named exactly (file name) as one of
//!   [`KNOWN_TOOLS`].
//! - Every program file involved (the shell, each wrapper and the tool) is
//!   in one of [`SYSTEM_DIRS`] that this run's sandbox can't write: named by
//!   that path, or by bare name found there first through the `PATH` the
//!   command runs with, with no folder the run can write (or a relative one)
//!   searched before it, and not itself writable. A copy the agent could
//!   have written is never it.
//!
//! Anything else, including anything this can't resolve, is not offered.
//!
//! Known limit: the `PATH` checked is the one Corbanu gives the command. The
//! shell's own start-up can still change it (a shell snapshot's saved
//! `PATH`, a login profile adding `~/bin`) or define a function of the
//! tool's name: those are the person's own files, which Moderate keeps the
//! agent from writing, but a folder they add that the agent can write isn't
//! checked here.

use std::path::Path;

use codex_sandboxing::user_namespaces::KNOWN_TOOLS;
use codex_shell_command::bash::extract_bash_command;
use codex_shell_command::bash::parse_shell_script_into_commands;

/// Folders a program may run from: the system's, and Nix's system-wide
/// profiles.
const SYSTEM_DIRS: [&str; 7] = [
    "/usr/bin",
    "/bin",
    "/usr/sbin",
    "/sbin",
    "/usr/local/bin",
    "/run/current-system/sw/bin",
    "/nix/var/nix/profiles/default/bin",
];

/// The shells whose `-c` script is parsed (exact file names).
const SHELLS: [&str; 3] = ["bash", "sh", "zsh"];

/// How program names are found for one run.
pub(crate) struct Lookup<'a> {
    /// The `PATH` the command runs with, where it runs on this host; `None`
    /// where unknown or remote (then a bare name isn't resolved).
    pub(crate) path: Option<&'a str>,
    /// Whether this run's sandbox lets it write `path` (a folder: in it).
    pub(crate) writable: &'a dyn Fn(&Path) -> bool,
    /// Whether `file` is an executable file.
    pub(crate) executable: &'a dyn Fn(&Path) -> bool,
}

/// The known tool `command` runs, if it resolves to exactly one (see the
/// module docs).
pub(crate) fn known_tool(command: &[String], lookup: &Lookup<'_>) -> Option<&'static str> {
    match extract_bash_command(command) {
        Some((shell, script)) => {
            // The shell itself, which the model may choose (`exec_command`'s
            // `shell`), must be a system one: the prompt shows only the
            // script.
            if !SHELLS.contains(&resolve(shell, lookup)?) {
                return None;
            }
            let mut commands = parse_shell_script_into_commands(script)?;
            if commands.len() != 1 {
                return None;
            }
            program_after_wrappers(&commands.pop()?, /*in_script*/ true, lookup)
        }
        None => program_after_wrappers(command, /*in_script*/ false, lookup),
    }
}

/// The known tool `words` (one simple command) runs, after the wrappers.
/// `in_script`: the words are a shell script's, so its first word may be the
/// builtin `exec` or `command`.
fn program_after_wrappers(
    mut words: &[String],
    in_script: bool,
    lookup: &Lookup<'_>,
) -> Option<&'static str> {
    let mut first_word = true;
    loop {
        let (first, rest) = words.split_first()?;
        // A shell builtin only as the script's first word, by bare name;
        // anywhere else `exec` or `command` is whatever program has that name.
        let builtin = in_script && first_word && matches!(first.as_str(), "exec" | "command");
        first_word = false;
        let name = if builtin {
            first.as_str()
        } else {
            resolve(first, lookup)?
        };
        // The wrapper's options without and with a value (a user, group or
        // name, never a command).
        let (flags, takes_value): (&[&str], &[&str]) = match name {
            "env" => (&["-i", "--ignore-environment"], &["-u", "--unset"]),
            "sudo" => (&["-n", "--non-interactive", "-E", "-H"], &["-u", "-g"]),
            "nohup" => (&[], &[]),
            "exec" if builtin => (&["-c", "-l", "-cl", "-lc"], &["-a"]),
            "command" if builtin => (&["-p"], &[]),
            _ => return KNOWN_TOOLS.iter().copied().find(|tool| *tool == name),
        };
        let mut at = 0;
        while let Some(option) = rest.get(at).map(String::as_str) {
            if option == "--" {
                at += 1;
                break;
            }
            if !option.starts_with('-') {
                break;
            }
            at += 1;
            if takes_value.contains(&option) {
                rest.get(at)?;
                at += 1;
            } else if !(flags.contains(&option)
                || (name == "env" && option.starts_with("--unset=")))
            {
                // An option this doesn't know (`env -S`, `sudo -s`,
                // `command -v`): not resolved.
                return None;
            }
        }
        words = &rest[at..];
    }
}

/// The file name of the program `word` runs, where that program is in one
/// of [`SYSTEM_DIRS`] the run can't write (see the module docs); `None`
/// otherwise, and for an assignment, an option or an empty word.
fn resolve<'w>(word: &'w str, lookup: &Lookup<'_>) -> Option<&'w str> {
    if word.is_empty() || word.contains('=') || word.starts_with('-') {
        return None;
    }
    match word.rsplit_once('/') {
        Some((folder, name)) => {
            (!name.is_empty() && system_dir(folder, lookup) && !(lookup.writable)(Path::new(word)))
                .then_some(name)
        }
        None => {
            for folder in lookup.path?.split(':') {
                // Paths as the Linux shell reads them (also where the tests
                // run on another system).
                let folder = folder.trim_end_matches('/');
                // A relative entry (also an empty one, the working folder),
                // one with `.` or `..`, or one the run can write: a copy
                // could be found there.
                if !folder.starts_with('/')
                    || folder.split('/').any(|part| matches!(part, "." | ".."))
                    || (lookup.writable)(Path::new(folder))
                {
                    return None;
                }
                let file = format!("{folder}/{word}");
                if (lookup.executable)(Path::new(&file)) {
                    return (system_dir(folder, lookup) && !(lookup.writable)(Path::new(&file)))
                        .then_some(word);
                }
            }
            None
        }
    }
}

/// Whether `folder` is one of [`SYSTEM_DIRS`] the run can't write.
fn system_dir(folder: &str, lookup: &Lookup<'_>) -> bool {
    SYSTEM_DIRS.contains(&folder) && !(lookup.writable)(Path::new(folder))
}

#[cfg(test)]
#[path = "user_namespaces_known_tool_tests.rs"]
mod tests;
