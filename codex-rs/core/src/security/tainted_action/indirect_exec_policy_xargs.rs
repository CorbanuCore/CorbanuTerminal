//! #2455 (D677): `xargs` runs its fixed argv with more words read from
//! stdin, appended or (with `-I`, `-i` or BSD `-J`) put in place of a
//! replacement string. The caller matches rules against the fixed part and
//! asks for approval whenever a rule could match once those words arrive.
//! GNU, BSD and BusyBox xargs all stop reading options at the first operand.

use super::Arity::Flag;
use super::Arity::LongOptional;
use super::Arity::Optional;
use super::Arity::Value;
use super::Opt;
use super::long_option;
use super::work;

/// GNU findutils options (long names are GNU only), plus BSD's `-J`, `-R`
/// and `-S`. `--help`, `--version` and anything else is uncertain.
const XARGS: &[Opt] = &[
    ('0', "null", Flag),
    ('a', "arg-file", Value),
    ('d', "delimiter", Value),
    ('E', "", Value),
    ('e', "eof", Optional),
    ('I', "", Value),
    ('i', "replace", Optional),
    ('J', "", Value),
    ('L', "", Value),
    ('l', "max-lines", Optional),
    ('n', "max-args", Value),
    ('o', "open-tty", Flag),
    ('p', "interactive", Flag),
    ('P', "max-procs", Value),
    ('R', "", Value),
    ('r', "no-run-if-empty", Flag),
    ('S', "", Value),
    ('s', "max-chars", Value),
    ('t', "verbose", Flag),
    ('x', "exit", Flag),
    ('\0', "process-slot-var", Value),
    ('\0', "show-limits", Flag),
];

/// GNU's replacement string for `-i` and `--replace` without a value.
const DEFAULT_REPLACE: &str = "{}";

pub(in super::super) enum Xargs<'a> {
    /// The fixed argv starts at `at`; words with a `replace` string in them
    /// are replaced by input.
    Command {
        at: usize,
        replace: [Option<&'a str>; 2],
    },
    /// No command: xargs runs `echo`.
    NoCommand,
    /// An unknown or malformed option.
    Uncertain,
}

/// The first word from `at` on that input may replace.
pub(in super::super) fn known_end(
    words: &[String],
    at: usize,
    replace: [Option<&str>; 2],
) -> usize {
    if replace.iter().all(Option::is_none) {
        return words.len();
    }
    (at..words.len())
        .find(|&index| {
            work::step();
            replace
                .iter()
                .flatten()
                .any(|replace| words[index].contains(replace))
        })
        .unwrap_or(words.len())
}

/// `words[start]` is `xargs`.
pub(in super::super) fn xargs(words: &[String], start: usize) -> Xargs<'_> {
    // The last `-I`/`-i` and the last `-J` (BSD keeps one of them, GNU
    // rejects `-J`): either may be in effect.
    let mut replace = [None, None];
    let mut at = start + 1;
    while let Some(word) = words.get(at) {
        work::step();
        if word == "--" {
            at += 1;
            break;
        }
        if !word.starts_with('-') || word == "-" {
            break;
        }
        let (letter, value, used) = if let Some(long) = word.strip_prefix("--") {
            let (name, attached) = long
                .split_once('=')
                .map_or((long, None), |(name, value)| (name, Some(value)));
            let Some((letter, _, arity)) = long_option(XARGS, name) else {
                return Xargs::Uncertain;
            };
            match (arity, attached) {
                (Flag, Some(_)) => return Xargs::Uncertain,
                (Value, None) => match words.get(at + 1) {
                    Some(value) => (letter, Some(value.as_str()), 2),
                    None => return Xargs::Uncertain,
                },
                (Flag | Value | Optional | LongOptional, attached) => (letter, attached, 1),
            }
        } else {
            let letters = &word[1..];
            let mut found = ('\0', None, 1);
            for (index, letter) in letters.char_indices() {
                work::step();
                let Some(&(_, _, arity)) = XARGS.iter().find(|(short, _, _)| {
                    work::step();
                    *short != '\0' && *short == letter
                }) else {
                    return Xargs::Uncertain;
                };
                if arity == Flag {
                    continue;
                }
                // The rest of the word, or for a required value the next.
                let rest = &letters[index + letter.len_utf8()..];
                found = if !rest.is_empty() {
                    (letter, Some(rest), 1)
                } else if arity == Optional {
                    (letter, None, 1)
                } else {
                    match words.get(at + 1) {
                        Some(value) => (letter, Some(value.as_str()), 2),
                        None => return Xargs::Uncertain,
                    }
                };
                break;
            }
            found
        };
        match letter {
            'I' => replace[0] = value,
            'i' => replace[0] = Some(value.unwrap_or(DEFAULT_REPLACE)),
            'J' => replace[1] = value,
            _ => {}
        }
        at += used;
    }
    if at >= words.len() {
        return Xargs::NoCommand;
    }
    Xargs::Command { at, replace }
}
