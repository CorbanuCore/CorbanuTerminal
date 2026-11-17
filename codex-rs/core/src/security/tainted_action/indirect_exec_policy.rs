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
        let name = names::launcher_name_for(program, case);
        let name = name.as_ref();
        let after = if name == "chroot" {
            // #2503: the classifier's chroot reader knows names spelled
            // chroot in any ASCII case only. For a spelling only this
            // platform reads as chroot (Windows `chroot.exe`), read a copy
            // of the argv, made once, with just this word spelled chroot.
            let argv = if basename(program).eq_ignore_ascii_case("chroot") {
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
