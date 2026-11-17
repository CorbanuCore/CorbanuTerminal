//! D553 / #2165: argv boundaries for exec-policy rules (sudo, env, chroot;
//! #2455 / D670 adds the launchers in `launchers`). Reuse the classifier's
//! option tables and chroot reader, but fail closed on unknown option syntax.
//! Unlike shell classification, these words are already argv: do not interpret
//! shell keywords, comments, or command arguments as additional programs.

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

pub(crate) struct PolicyWrapperCommands {
    pub(crate) starts: Vec<usize>,
    pub(crate) uncertain: bool,
}

pub(crate) fn policy_wrapper_commands(words: &[String]) -> PolicyWrapperCommands {
    let mut result = PolicyWrapperCommands {
        starts: vec![0],
        uncertain: false,
    };
    let mut start = 0;
    // Computed on the first launcher only.
    let mut scan = None;
    while let Some(program) = words.get(start) {
        work::step();
        let name = basename(program);
        let after = if name == "chroot" {
            // No held redirections: callers supply argv after shell parsing.
            let reads = chroot_reads(&SimpleCommand::default(), &words[start..], /*from*/ 0);
            let Some(read) = reads.first() else {
                break;
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
                    return result;
                }
            }
            (at < words.len()).then_some(at)
        } else if launchers::is_launcher(name, start) {
            // #2455 (D670): unshare, nsenter, timeout and the like.
            let scan = *scan.get_or_insert_with(|| launchers::scan(words));
            match launchers::launch(name, words, start, scan) {
                launchers::Launch::Command { at, uncertain } => {
                    result.uncertain |= uncertain;
                    Some(at)
                }
                launchers::Launch::NoCommand => return result,
                launchers::Launch::Uncertain => None,
            }
        } else {
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
