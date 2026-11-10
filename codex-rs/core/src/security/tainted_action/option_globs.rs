//! #1912: a glob may match a file whose name is an option. `tar cf a.tar *`
//! in a folder holding files named `--checkpoint=1` and
//! `--checkpoint-action=exec=sh x` runs `sh x`; so does `rg data *` with
//! `--pre=sh`, `sed -i s/a/b/ *` with `--expression=1e sh x`, and `sort`'s
//! and `rsync`'s program options. The GNU tools read options after their
//! operands; BSD `tar` and `sed` read them up to the first operand, which a
//! name starting with `-` usually sorts before. So a glob before `--` that
//! may make such a word asks, as one the line does not show. `./*`, `src/*`, `"$PWD"/*` and `--exclude=*.log`
//! cannot start with an option name; `*`, `*.txt`, `-*`, `?*` and `[-]*`
//! can (for `rg` and `sort`, only a glob that ends in its pattern: see
//! [`PROGRAM_RUNNERS`]). Git's transports are judged in `rerun`.

use super::may_be_built;
use super::quoted::Word;
use super::work;

/// Commands with an option whose value they run as a command line, or as
/// a program with arguments (see `rerun`'s `OPERAND_RUNNERS`): `tar`'s
/// `--checkpoint-action=exec=`, `--to-command`, `-I`; `sed`'s `e`
/// command (`--expression=1e sh x`); `rsync -e`. Any text after the
/// pattern becomes an argument (`--to-command=sh x.txt` matches `*.txt`).
/// macOS's `tar` is `bsdtar` (`--use-compress-program`).
const LINE_RUNNERS: &[&str] = &["tar", "gtar", "bsdtar", "sed", "gsed", "rsync"];

/// Commands with an option whose value is a program name, looked up on
/// `PATH` (`rg --pre`, `sort --compress-program`): a glob with literal
/// text after its pattern (`*.rs`) can only name a program ending in it,
/// which a file name cannot give a folder for (a `PATH` holding `.` or an
/// empty entry would; that is not judged here).
const PROGRAM_RUNNERS: &[&str] = &["rg", "sort", "gsort"];

/// Characters at which a word's text stops being literal for this check:
/// glob characters, braces and extended-glob groups.
const PATTERN: &[char] = &['*', '?', '[', '^', '{', '(', '@', '!', '+', '#'];

/// Characters a pattern may end with (`*`, `x?`, `[ab]`, `@(a|b)`,
/// `{a,*}`).
const PATTERN_END: &[char] = &['*', '?', ']', ')', '}'];

/// Whether a glob's matches may end in any text: it ends in its pattern,
/// or it may negate or exclude (zsh `^a.txt`, `*~x.rs`, `#`; bash
/// `!(a).rs`), which a literal ending does not bound.
fn ends_open(text: &str) -> bool {
    text.ends_with(PATTERN_END) || text.contains(['^', '~', '!', '#'])
}

/// Old-style `tar` letters that take the next word as their value
/// (`tar cf -- *`: `--` is the archive): GNU tar's (as in `rerun`) and
/// bsdtar's `s` and `W`. An extra letter only reads `--` as a value, which
/// asks.
const TAR_VALUE_LETTERS: &str = "bCfFgHIKLNTVXsW";

/// Whether the command whose command word is at `at` (lowercase basename
/// `name`) is a [`LINE_RUNNERS`] or [`PROGRAM_RUNNERS`] command given a
/// glob that may match a file named as one of its options. `words` is the
/// quote-aware reading of the command, `texts` their texts.
pub(super) fn runs_option_glob(name: &str, at: usize, texts: &[String], words: &[Word]) -> bool {
    let program_only = if LINE_RUNNERS.contains(&name) {
        false
    } else if PROGRAM_RUNNERS.contains(&name) {
        true
    } else {
        return false;
    };
    let tar = matches!(name, "tar" | "gtar" | "bsdtar");
    // Old-style tar's first word holds letters whose values follow it.
    let old_end = match texts.get(at + 1) {
        Some(letters) if tar && !letters.starts_with('-') => {
            at + 2
                + letters
                    .chars()
                    .filter(|letter| {
                        work::step();
                        TAR_VALUE_LETTERS.contains(*letter)
                    })
                    .count()
        }
        _ => 0,
    };
    for index in at + 1..texts.len().min(words.len()) {
        work::step();
        // Options end at `--`, unless it is an option's value (`tar -cf
        // --`, `tar cf --`).
        if texts[index] == "--" && !texts[index - 1].starts_with('-') && index >= old_end {
            return false;
        }
        let word = &words[index];
        if may_match_option(word) && (!program_only || ends_open(&word.text)) {
            return true;
        }
    }
    false
}

/// Whether the shell may glob `word` into one that starts with `-` and
/// names an option: its literal text before the first pattern character
/// (less a character zsh's `#` makes optional) is empty, an option name
/// not ending its value at `=` (`-*`, `--pre*`, `-ne=*`: a short option's
/// value may be the `=`), or a value the line does not show (`$d*`;
/// `"$d"/*` holds such a value as a folder, like `"$d"` alone, which is not
/// judged here).
fn may_match_option(word: &Word) -> bool {
    if !word.globs {
        return false;
    }
    let text = word.text.as_str();
    let end = text.find(PATTERN).unwrap_or(text.len());
    let mut prefix = &text[..end];
    // zsh's `x#`, `x##` and `x(#c0,1)` make the `x` before them optional.
    if text[end..].starts_with('#') || text[end..].starts_with("(#") {
        prefix = prefix
            .char_indices()
            .last()
            .map_or(prefix, |(last, _)| &prefix[..last]);
    }
    prefix.is_empty()
        || (prefix.starts_with('-') && !(prefix.starts_with("--") && prefix.contains('=')))
        || (may_be_built(prefix) && !prefix.contains('/'))
}
