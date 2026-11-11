//! #932 phase 2: the grammar's reading of each command line, beside the
//! hand-written reader's. The two are compared by the commands they find and
//! a disagreement is logged (without the line's text, which may hold
//! secrets). Nothing here changes a classification: the reader stays
//! authoritative until phase 3 (docs/development/shell-grammar-classifier.md).

use super::grammar;
use super::grammar::Refusal;
use super::quoted;
use super::work;

/// Bytes of shell text one classification gives the grammar, each line
/// costing [`LINE_COST`] more than its length; lines past it are not
/// compared. With [`grammar::MAX_BYTES`] per line this bounds the grammar's
/// work per classification (lines too long to read are refused unread and
/// logged, each, without charge). Only the command lines
/// `quoted_pass` reads (a line's own text and nested shells' scripts) are
/// compared, not the quoted words it re-reads.
pub(super) const MAX_COMPARED_BYTES: usize = 64 * 1024;

/// What each compared line costs beyond its length, so the budget also
/// bounds the number of lines (`bash -c '' -c '' ...`): at most 256.
const LINE_COST: usize = 256;

/// The comparisons one classification may still make.
pub(super) struct Budget {
    bytes: usize,
    /// A line was not compared for want of budget, and that was logged
    /// (once per classification).
    skip_logged: bool,
}

impl Budget {
    pub(super) fn new() -> Self {
        Self {
            bytes: MAX_COMPARED_BYTES,
            skip_logged: false,
        }
    }
}

/// Reserved words the reader keeps as a command's words.
const RESERVED: &[&str] = &[
    "!", "{", "}", "(", ")", "if", "then", "else", "elif", "fi", "do", "done", "while", "until",
    "time", "esac",
];

/// How the two readings of a line compare.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Comparison {
    /// The same commands, by name.
    Agree,
    /// The reader finds commands the grammar does not (it reads arithmetic
    /// and some patterns as commands on purpose), and not the reverse.
    ReaderFindsMore,
    /// The grammar finds commands the reader does not, and not the reverse.
    GrammarFindsMore,
    /// Each finds commands the other does not.
    Differ,
    GrammarRefused(Refusal),
    /// The reader's reading is unreadable to the classifier (too deep, a
    /// `case` it does not follow, or unterminated).
    ReaderUnreadable,
    /// The classification's [`MAX_COMPARED_BYTES`] were spent.
    NotCompared,
}

/// Compare the reader's `reading` of the command line `text` with the
/// grammar's and log a difference, spending `text`'s length from `budget`.
pub(super) fn shadow(text: &str, reading: &quoted::Reading, budget: &mut Budget) {
    let (comparison, grammar) = compare(text, reading, &mut budget.bytes);
    #[cfg(test)]
    tests::RECORDED.with(|recorded| recorded.borrow_mut().push(comparison));
    if comparison == Comparison::NotCompared && std::mem::replace(&mut budget.skip_logged, true) {
        return;
    }
    if comparison != Comparison::Agree {
        let data_words = grammar.as_ref().map(|grammar| {
            grammar
                .commands
                .iter()
                .flat_map(|command| &command.words)
                .filter(|word| word.data)
                .count()
        });
        tracing::debug!(
            ?comparison,
            ?data_words,
            substituted = ?grammar.as_ref().map(|grammar| {
                grammar
                    .commands
                    .iter()
                    .filter(|command| command.substituted)
                    .count()
            }),
            parse_checks = ?grammar.as_ref().map(|grammar| grammar.parse_checks),
            zsh_unsure = ?grammar.as_ref().map(|grammar| grammar.zsh_unsure),
            bash_only = ?grammar.as_ref().map(|grammar| grammar.bash_only),
            "#932: the grammar's reading of a command line differs from the reader's"
        );
    }
}

pub(super) fn compare(
    text: &str,
    reading: &quoted::Reading,
    budget: &mut usize,
) -> (Comparison, Option<grammar::Reading>) {
    // Refused unread, at no cost: the lines after it are still compared.
    if text.len() > grammar::MAX_BYTES {
        return (Comparison::GrammarRefused(Refusal::TooLarge), None);
    }
    let Some(left) = budget.checked_sub(text.len() + LINE_COST) else {
        return (Comparison::NotCompared, None);
    };
    *budget = left;
    let grammar = match grammar::read(text) {
        Ok(grammar) => grammar,
        Err(refusal) => return (Comparison::GrammarRefused(refusal), None),
    };
    let Some(reader) = reader_names(reading) else {
        return (Comparison::ReaderUnreadable, Some(grammar));
    };
    let names = grammar_names(&grammar);
    let comparison = match (missing(&reader, &names), missing(&names, &reader)) {
        (false, false) => Comparison::Agree,
        (true, false) => Comparison::ReaderFindsMore,
        (false, true) => Comparison::GrammarFindsMore,
        (true, true) => Comparison::Differ,
    };
    (comparison, Some(grammar))
}

/// The reader's commands as sorted names (an expansion is `$`), without
/// the reserved words, assignments and loop or `case` headers it keeps as
/// words; `None` when the classifier treats the reading as unreadable.
pub(super) fn reader_names(reading: &quoted::Reading) -> Option<Vec<String>> {
    if reading.too_deep || reading.unsure || reading.unterminated {
        return None;
    }
    let mut names = Vec::new();
    for command in &reading.commands {
        work::step();
        let first = command.words.iter().find(|word| {
            !(is_assignment(&word.text) || (!word.quoted && RESERVED.contains(&word.text.as_str())))
        });
        let Some(first) = first else { continue };
        if !first.quoted && matches!(first.text.as_str(), "for" | "select" | "case" | "function") {
            continue;
        }
        names.push(if first.dynamic {
            "$".to_string()
        } else {
            first.text.clone()
        });
    }
    names.sort_unstable();
    Some(names)
}

/// The grammar's commands as sorted names (an expansion is `$`).
pub(super) fn grammar_names(reading: &grammar::Reading) -> Vec<String> {
    let mut names: Vec<String> = reading
        .commands
        .iter()
        .filter_map(|command| command.words.first())
        .map(|name| {
            if name.dynamic {
                "$".to_string()
            } else {
                name.text.clone()
            }
        })
        .collect();
    names.sort_unstable();
    names
}

fn is_assignment(word: &str) -> bool {
    let name = word.split(['=', '[', '+']).next().unwrap_or_default();
    word.len() > name.len()
        && word[name.len()..].contains('=')
        && name
            .chars()
            .next()
            .is_some_and(|ch| ch.is_alphabetic() || ch == '_')
        && name.chars().all(|ch| ch.is_alphanumeric() || ch == '_')
}

/// Whether sorted `from` has a name (counting repeats) sorted `other` lacks.
/// Linear: both are sorted.
pub(super) fn missing(from: &[String], other: &[String]) -> bool {
    let mut other = other.iter().peekable();
    for name in from {
        work::step();
        while other.next_if(|candidate| *candidate < name).is_some() {
            work::step();
        }
        if other.next_if(|candidate| *candidate == name).is_none() {
            return true;
        }
    }
    false
}

#[cfg(test)]
#[path = "shadow_tests.rs"]
mod tests;
