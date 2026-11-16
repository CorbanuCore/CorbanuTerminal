//! #1661: additional readings only. Existing readings remain the floor.
use super::WRITERS;
use super::braces;
use super::indirect;
use super::nested::leading_expansion;
use super::outputs;
use super::quoted::Command;
use super::quoted::Word;
use super::sed;
use super::unknown_folder;
use super::work;
use std::collections::HashSet;

/// zsh expands an equals word after brace and parameter expansion. Quoted
/// literal equals words are left to the existing checks.
fn program_target(word: &Word) -> bool {
    // These readings only expose an equals sign already present in the word.
    // Avoid expanding unrelated (potentially long) brace chains.
    if !word.text.contains('=') {
        return false;
    }
    if word.equals_first && outputs::zsh_program_path(&word.text) {
        return true;
    }
    let after_empty = |text: &str| {
        let mut rest = text;
        // Same bounded leading-expansion reader as git's option readings.
        for _ in 0..16 {
            work::step();
            if outputs::zsh_program_path(rest) {
                return true;
            }
            if !word.dynamic {
                break;
            }
            let Some((length, _)) = leading_expansion(rest) else {
                break;
            };
            rest = &rest[length..];
        }
        false
    };
    if word.brace_expansion {
        for old_bash in [false, true] {
            work::step();
            let expanded = braces::expand(&word.text, /*limit*/ 16, old_bash);
            if expanded.truncated && word.text.contains('=')
                || expanded.words.iter().any(|text| after_empty(text))
            {
                return true;
            }
        }
    }
    word.dynamic && after_empty(&word.text)
}

/// Redirections and writers behind launchers, including writers recognized
/// by their output operands rather than WRITERS (sort, cc, uniq).
pub(super) fn writes_program(command: &Command, texts: &[String]) -> bool {
    if command.writes_to.iter().any(program_target) {
        return true;
    }
    let (starts, _) = sed::command_starts(texts);
    for start in starts {
        work::step();
        let Some((index, mut program)) = indirect::command_word(&texts[start..]) else {
            continue;
        };
        let mut at = start + index;
        while unknown_folder::MODIFIERS.contains(&program.as_str()) {
            work::step();
            at += 1;
            let Some(text) = texts.get(at) else {
                break;
            };
            program = super::shell::basename(text).to_lowercase();
        }
        let Some(args) = command.words.get(at + 1..) else {
            continue;
        };
        if !args.iter().any(program_target) {
            continue;
        }
        if WRITERS.contains(&program.as_str()) || program == "strip" {
            return true;
        }
        let files: HashSet<String> =
            outputs::output_files(&program, &texts[at + 1..], /*unchecked*/ false)
                .into_iter()
                .flatten()
                .collect();
        if args.iter().any(|word| {
            work::step();
            files.contains(&word.text) && program_target(word)
        }) {
            return true;
        }
    }
    false
}

/// An anchored replacement can make an option when its input is empty.
/// This is an extra possible reading, never a substitute for the words
/// as written (which may have a nonempty parameter value).
pub(super) fn git_options(words: &[Word]) -> Option<Vec<Word>> {
    let mut changed = false;
    let made = words
        .iter()
        .map(|word| {
            work::step();
            let replacement = if word.dynamic {
                word.text
                    .strip_prefix("${")
                    .and_then(|text| text.split_once("/#/"))
                    .filter(|(name, _)| {
                        !name.is_empty()
                            && name
                                .bytes()
                                .all(|ch| ch.is_ascii_alphanumeric() || ch == b'_')
                    })
                    .and_then(|(_, value)| value.strip_suffix('}'))
                    .filter(|value| value.starts_with('-') && !value.contains(['$', '{', '}']))
                    .map(str::to_string)
            } else {
                None
            };
            if let Some(text) = replacement {
                changed = true;
                Word {
                    text,
                    globs: false,
                    ..word.clone()
                }
            } else {
                word.clone()
            }
        })
        .collect();
    changed.then_some(made)
}
