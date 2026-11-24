//! D553 / #2165: argv boundaries for exec-policy rules (sudo, env, chroot;
//! #2455 / D670 adds the launchers in `launchers`). Reuse the classifier's
//! option tables and chroot reader, but fail closed on unknown option syntax.
//! Unlike shell classification, these words are already argv: do not interpret
//! shell keywords, comments, or command arguments as additional programs.

use super::super::shell::SHELLS;
use super::FLAG_LETTERS;
use super::SimpleCommand;
use super::WRAPPERS;
use super::basename;
use super::chroot_reads;
use super::option_words;
use super::sudo_long_option;
use super::sudo_option_words;
use super::sudo_value_letter;
use super::work;

#[path = "indirect_exec_policy_launchers.rs"]
mod indirect_exec_policy_launchers;
use indirect_exec_policy_launchers as launchers;

#[path = "indirect_exec_policy_names.rs"]
mod indirect_exec_policy_names;
use indirect_exec_policy_names as names;

#[cfg(any(windows, test))]
#[path = "indirect_exec_policy_powershell.rs"]
mod indirect_exec_policy_powershell;
#[cfg(windows)]
pub(crate) use indirect_exec_policy_powershell::policy_powershell_script;

pub(crate) struct PolicyWrapperCommands {
    pub(crate) starts: Vec<usize>,
    pub(crate) uncertain: bool,
    /// #2455 (D677): an `xargs` in the chain adds words read from stdin.
    pub(crate) open: Option<OpenArgv>,
}

/// The argv an `xargs` runs: its end is open, and from `known_end` on its
/// words may be replaced by input.
pub(crate) struct OpenArgv {
    pub(crate) known_end: usize,
}

pub(crate) fn policy_wrapper_commands(words: &[String]) -> PolicyWrapperCommands {
    policy_wrapper_commands_for(words, names::PLATFORM)
}

/// #2503 (D688): launcher names compare as `case` says (see [`names`]).
fn policy_wrapper_commands_for(words: &[String], case: names::NameCase) -> PolicyWrapperCommands {
    let mut result = PolicyWrapperCommands {
        starts: vec![0],
        uncertain: false,
        open: None,
    };
    let mut start = 0;
    // Computed on the first launcher only.
    let mut scan = None;
    // A copy of `words` for the chroot reader, made when first needed.
    let mut chroot_words: Option<Vec<String>> = None;
    while let Some(program) = words.get(start) {
        work::step();
        if names::unsure(program, case) {
            // #2510: a name this platform may run as a launcher or a shell
            // spelled another way; its argv boundaries are unknown.
            result.uncertain = true;
            break;
        }
        let name = names::launcher_name_for(program, case);
        let name = name.as_ref();
        let after = if name == "chroot" {
            // #2503: the classifier's chroot reader knows names spelled
            // chroot in any ASCII case only, and (#2510) reads its words as
            // a shell would, skipping an assignment-like or comment-like
            // first word (`a=b/chroot`, `#x/chroot`) that argv runs as a
            // program. For any other spelling (`/usr/sbin/chroot` too),
            // read a copy of the argv, made once, with just this word
            // spelled chroot.
            let argv = if program.eq_ignore_ascii_case("chroot") {
                &words[start..]
            } else {
                let spelled = chroot_words.get_or_insert_with(|| words.to_vec());
                spelled[start] = "chroot".to_string();
                &spelled[start..]
            };
            // No held redirections: callers supply argv after shell parsing.
            let reads = chroot_reads(&SimpleCommand::default(), argv, /*from*/ 0);
            let Some(read) = reads.first() else {
                return no_command(result);
            };
            result.uncertain |= read.unknown_option;
            read.command.map(|at| start + at)
        } else if matches!(name, "sudo" | "env") {
            let Some((_, value_options, _)) =
                WRAPPERS.iter().find(|(wrapper, _, _)| *wrapper == name)
            else {
                result.uncertain = true;
                return result;
            };
            let mut at = start + 1;
            let mut ended_options = false;
            let mut env_after_dash = None;
            while let Some(option) = words.get(at) {
                work::step();
                if option == "--" {
                    ended_options = true;
                    at += 1;
                    break;
                }
                if name == "env" && option == "-" {
                    at += 1;
                    // GNU stops option parsing here; BSD keeps reading flags.
                    // Keep GNU's command as well as the later BSD command.
                    if env_after_dash.is_none() {
                        let mut command = at;
                        while words.get(command).is_some_and(|word| word.contains('=')) {
                            work::step();
                            command += 1;
                        }
                        env_after_dash = Some(command);
                        if command < words.len() {
                            result.starts.push(command);
                        }
                    }
                    continue;
                }
                // sudo's is_envar excludes absolute and equals-leading
                // command names. This loop stops at -- before assignments.
                if name == "sudo" && option.contains('=') && !option.starts_with(['-', '/', '=']) {
                    at += 1;
                    continue;
                }
                if !option.starts_with('-') || option == "-" {
                    break;
                }
                if !known_option(name, option, value_options) {
                    result.uncertain = true;
                    if let Some(command) = env_after_dash {
                        // The BSD option is unknown; retain GNU's command
                        // as well as the approval floor.
                        at = command;
                        break;
                    }
                    return result;
                }
                at += if name == "sudo" {
                    sudo_option_words(option, value_options)
                } else {
                    option_words(name, option, value_options)
                };
            }
            // After env's options end, even -NAME=value is an assignment.
            // sudo has already consumed its assignments, before any --.
            if name == "env" {
                // GNU also consumes a lone - after --. Retain the BSD
                // interpretation (a command named -) before following it.
                if ended_options
                    && env_after_dash.is_none()
                    && words.get(at).is_some_and(|word| word == "-")
                {
                    result.starts.push(at);
                    at += 1;
                }
                while words.get(at).is_some_and(|word| word.contains('=')) {
                    work::step();
                    at += 1;
                }
                // With complete options and no command, env only prints its
                // environment. A missing option value advances past len and
                // still fails closed below.
                if at == words.len() {
                    return no_command(result);
                }
            }
            (at < words.len()).then_some(at)
        } else if matches!(name, "xargs" | "gxargs") {
            match launchers::xargs(words, start) {
                launchers::Xargs::Command { at, replace } => {
                    match &mut result.open {
                        None => {
                            result.open = Some(OpenArgv {
                                known_end: launchers::known_end(words, at, replace),
                            });
                        }
                        // A nested xargs with a replacement string: treat
                        // its whole argv as unknown (one scan per command).
                        Some(open) if replace.iter().any(Option::is_some) => {
                            open.known_end = open.known_end.min(at);
                        }
                        Some(_) => {}
                    }
                    Some(at)
                }
                // xargs runs `echo`.
                launchers::Xargs::NoCommand => return no_command(result),
                launchers::Xargs::Uncertain => None,
            }
        } else if matches!(name, "sg" | "watch")
            || (name == "su" && (result.open.is_some() || !su_login_only(&words[start..])))
        {
            // #2455 (D676, D678): these run a shell string (su -c, sg, watch)
            // or pass words to a shell; approval-only for now. Under xargs,
            // stdin may add `-c` to any su.
            None
        } else if launchers::is_launcher(name, start) {
            // #2455 (D670): unshare, nsenter, timeout and the like.
            let mut scan = *scan.get_or_insert_with(|| launchers::scan(words));
            if result.open.is_some() {
                // D677: stdin may add option words or a `$` at the end.
                scan = scan.open_end();
            }
            match launchers::launch(name, words, start, scan) {
                launchers::Launch::Command { at, uncertain } => {
                    result.uncertain |= uncertain;
                    Some(at)
                }
                launchers::Launch::NoCommand => return no_command(result),
                launchers::Launch::Uncertain => None,
            }
        } else {
            // D677: under xargs, stdin may give a shell its script.
            result.uncertain |= result.open.is_some() && SHELLS.contains(&name);
            break;
        };
        let Some(after) = after else {
            result.uncertain = true;
            break;
        };
        if result.starts.last().copied() != Some(after) {
            result.starts.push(after);
        }
        start = after;
    }
    result
}

/// #2510 (D688): `[shell, -c|-lc, script]` whose `shell` runs bash, zsh or
/// sh on this platform (see [`names`]) under a spelling the shell parser
/// does not read (`BASH -lc`, `/bin/Sh -c`, `Bash.exe -c`), or (#2543)
/// another POSIX shell (`dash -c`): the argv with the shell spelled as that
/// parser reads it, so rules see the script's commands. `None` where the
/// parser reads `words` already.
pub(crate) fn policy_shell_script(words: &[String]) -> Option<Vec<String>> {
    policy_shell_script_for(words, names::PLATFORM)
}

fn policy_shell_script_for(words: &[String], case: names::NameCase) -> Option<Vec<String>> {
    let [shell, flag, script] = words else {
        return None;
    };
    if !matches!(flag.as_str(), "-c" | "-lc")
        || names::unsure(shell, case)
        || codex_shell_command::bash::extract_bash_command(words).is_some()
    {
        return None;
    }
    let name = names::launcher_name_for(shell, case);
    let read_as = match name.as_ref() {
        "bash" | "zsh" | "sh" if case != names::NameCase::Sensitive => name.into_owned(),
        // #2543: on every platform, POSIX shells the parser does not know
        // (`dash -c`, `ksh -c`) read like `sh -c`; their quoting is sh's.
        other if POSIX_SHELLS.contains(&other) => "sh".to_string(),
        _ => return None,
    };
    Some(vec![read_as, flag.clone(), script.clone()])
}

/// Shells in [`SHELLS`] besides bash, zsh and sh that read a `-c` script
/// with sh's syntax and quoting.
/// #2548: not the ksh family, yash, hush or osh, whose builtins and
/// assignments run code (`alarm`, `typeset -F`, `x=(typeset -i …)`) where
/// sh's do not: their `-c` scripts are not read, so they ask.
const POSIX_SHELLS: &[&str] = &["dash", "ash", "rbash", "posh"];

/// #2543: whether `program` may run a shell on this platform (see
/// [`names`]), which runs any command its words give it.
pub(crate) fn policy_runs_shell(program: &str) -> bool {
    let case = names::PLATFORM;
    names::unsure(program, case)
        || SHELLS.contains(&names::launcher_name_for(program, case).as_ref())
        // #2734: a name shell detection reads through another extension
        // (`bash.foo.exe`, `bash.sh`), whose script is not read, may run one.
        || matches!(
            codex_shell_command::shell_detect::detect_shell_type(program),
            Some(
                codex_shell_command::shell_detect::ShellType::Bash
                    | codex_shell_command::shell_detect::ShellType::Zsh
                    | codex_shell_command::shell_detect::ShellType::Sh
            )
        )
}

/// #2548: the name `program` runs as on this platform (its file name,
/// compared as [`names`] says), for looking up builtins a wrapper script
/// such as macOS's `/usr/bin/read` runs.
pub(crate) fn policy_program_name(program: &str) -> String {
    names::launcher_name_for(program, names::PLATFORM).into_owned()
}

/// #2704: the name `program` runs as on Windows (its file name, folded, a
/// drive and trailing dots and spaces dropped, `.exe` dropped), on every
/// platform, for finding cmd.exe and batch files.
pub(crate) fn policy_windows_program_name(program: &str) -> String {
    names::launcher_name_for(program, names::NameCase::Windows).into_owned()
}

/// #2585: whether `program` may run PowerShell on this platform (see
/// [`names`]), which runs any command its words give it.
pub(crate) fn policy_runs_powershell(program: &str) -> bool {
    codex_shell_command::powershell::is_powershell_program(program)
        || matches!(
            names::launcher_name_for(program, names::PLATFORM).as_ref(),
            "pwsh" | "powershell"
        )
}

/// A launcher with no command of its own. Under xargs (D677) stdin may
/// supply one, which no rule has seen.
fn no_command(mut result: PolicyWrapperCommands) -> PolicyWrapperCommands {
    result.uncertain |= result.open.is_some();
    result
}

/// D676: `su` that only opens a login or interactive shell: `su`, then an
/// optional `-`, `-l` or `--login`, then an optional user. Anything more
/// (`-c`, `--command`, `--session-command`, `-s`, words the shell receives
/// as arguments, other options) may run a command.
fn su_login_only(words: &[String]) -> bool {
    let mut rest = &words[1..];
    if rest
        .first()
        .is_some_and(|word| matches!(word.as_str(), "-" | "-l" | "--login"))
    {
        rest = &rest[1..];
    }
    match rest {
        [] => true,
        [user] => !user.starts_with('-'),
        _ => false,
    }
}

fn known_option(wrapper: &str, option: &str, value_options: &[&str]) -> bool {
    if let Some(long) = option.strip_prefix("--") {
        let (name, attached) = long
            .split_once('=')
            .map_or((long, false), |(name, _)| (name, true));
        return if wrapper == "sudo" {
            sudo_long_option(name)
                .is_some_and(|(name, takes)| !attached || takes || name == "preserve-env")
        } else {
            matches!(name, "unset" | "chdir")
                || (!attached && matches!(name, "ignore-environment" | "null" | "debug"))
        };
    }
    let letters = option.strip_prefix('-').unwrap_or_default();
    let flags = FLAG_LETTERS
        .iter()
        .find(|(name, _)| *name == wrapper)
        .map_or("", |(_, flags)| *flags);
    for letter in letters.chars() {
        work::step();
        if sudo_value_letter(letter, value_options) {
            return true; // The rest is an attached value, not more options.
        }
        if !flags.contains(letter) {
            return false;
        }
    }
    !letters.is_empty()
}
