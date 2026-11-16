//! D553 / #2165: argv boundaries for exec-policy rules. Reuse the classifier's
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
            while let Some(option) = words.get(at) {
                work::step();
                if option == "--" {
                    at += 1;
                    break;
                }
                if name == "env" && option == "-" {
                    at += 1;
                    break;
                }
                if name == "sudo" && option.contains('=') && !option.starts_with('-') {
                    at += 1;
                    continue;
                }
                if !option.starts_with('-') || option == "-" {
                    break;
                }
                if !known_option(name, option, value_options) {
                    result.uncertain = true;
                    return result;
                }
                at += if name == "sudo" {
                    sudo_option_words(option, value_options)
                } else {
                    option_words(name, option, value_options)
                };
            }
            while words
                .get(at)
                .is_some_and(|word| word.contains('=') && !word.starts_with('-'))
            {
                work::step();
                at += 1;
            }
            (at < words.len()).then_some(at)
        } else {
            break;
        };
        let Some(after) = after else {
            result.uncertain = true;
            break;
        };
        result.starts.push(after);
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
