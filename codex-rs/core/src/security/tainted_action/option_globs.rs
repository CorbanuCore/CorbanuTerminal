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
//!
//! D484 (b): a value the line does not show asks only in two forms. A loop
//! variable taken from such a glob (`for f in *; do tar rf a.tar "$f";
//! done`; see [`GlobLoops`]), and `xargs` fed by `ls` or by `find` without
//! `-print0` and `xargs -0` (`ls | xargs tar cf a.tar`; see [`Feed`]),
//! whose input may hold such a name, or one split at a blank. Other unseen
//! operands are not judged here.

use super::indirect;
use super::may_be_built;
use super::quoted::Command;
use super::quoted::Word;
use super::shell::UNSEEN_OUTPUT;
use super::shell::basename;
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
/// word that may name one of its options: a glob that may match a file
/// named so, or a [`GlobLoops`] variable holding one. `words` is the
/// quote-aware reading of the command, `texts` their texts.
pub(super) fn runs_option_glob(
    name: &str,
    at: usize,
    texts: &[String],
    words: &[Word],
    loops: &GlobLoops,
) -> bool {
    runs_option(name, at, texts, words, loops, /*fed*/ None)
}

/// D484 (b): whether `xargs`, given names by `feed`, runs a
/// [`LINE_RUNNERS`] or [`PROGRAM_RUNNERS`] command with them where they
/// may name an option (`ls | xargs tar cf a.tar`, `ls | xargs -I{} rg x
/// {}`), or with a word that may otherwise (as [`runs_option_glob`]).
/// `xargs`' own options are read here, not by `indirect`'s wrapper table
/// (BSD `-J %`, GNU `--replace` without a value).
pub(super) fn xargs_runs_option(
    texts: &[String],
    words: &[Word],
    loops: &GlobLoops,
    feed: Feed,
) -> bool {
    // The first `xargs` the command runs reads the pipeline.
    let Some(start) = indirect::programs(texts).into_iter().find(|&index| {
        work::step();
        XARGS.contains(&basename(&texts[index]).to_lowercase().as_str())
    }) else {
        return false;
    };
    let Some(input) = xargs_input(texts, start + 1) else {
        return false;
    };
    // Without names that may be options, its command's own words are still
    // judged (`xargs -J % tar rf a.tar *`, which `indirect` reads as `%`).
    let fed = match feed {
        Feed::Other => None,
        Feed::FindNul if input.nul => None,
        _ => Some(input.replace),
    };
    let Some((offset, name)) = indirect::command_word(&texts[input.command..]) else {
        return false;
    };
    runs_option(&name, input.command + offset, texts, words, loops, fed)
}

/// The scan of [`runs_option_glob`]; `fed`: `xargs`' input, added after the
/// last word (`Some(None)`) or put where its replacement string is.
fn runs_option(
    name: &str,
    at: usize,
    texts: &[String],
    words: &[Word],
    loops: &GlobLoops,
    fed: Option<Option<String>>,
) -> bool {
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
        let open = if may_match_option(word) {
            Some(ends_open(&word.text))
        } else if let Some(Some(replace)) = &fed
            && word.text.starts_with(replace.as_str())
        {
            Some(true)
        } else {
            loops.option(word)
        };
        if open.is_some_and(|open| open || !program_only) {
            return true;
        }
    }
    // Input added after the last word.
    matches!(fed, Some(None))
}

/// What a pipeline feeds a command's input, by its earlier stages.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Feed {
    /// Nothing judged here.
    #[default]
    Other,
    /// `find … -print0` (without `-printf` or a command of its own),
    /// through filters only: its names start with a start point and are
    /// split only at NULs, read so by `xargs -0`.
    FindNul,
    /// `ls`, or `find` otherwise: bare names (`ls`), or names `xargs`
    /// splits at blanks (`./a --checkpoint=1`).
    Listing,
}

/// Commands that list file names.
const LISTERS: &[&str] = &["ls", "gls"];
const FINDERS: &[&str] = &["find", "gfind"];
const XARGS: &[&str] = &["xargs", "gxargs"];

/// `find` words that print other text or run a command whose output joins
/// the names (`-exec basename {} ;`).
const FIND_OUTPUTS: &[&str] = &[
    "printf", "-exec", "-execdir", "-ok", "-okdir", "-ls", "-fls", "-print", "-fprint",
];

/// Filters whose options' values may be the next word, by the letters and
/// long options that take one (so `grep -e -z` is not `-z`).
const NUL_FILTERS: &[(&str, &str, &[&str])] = &[
    (
        "grep",
        "efmABCdD",
        &[
            "regexp",
            "file",
            "max-count",
            "after-context",
            "before-context",
            "context",
            "label",
            "include",
            "exclude",
            "exclude-dir",
            "exclude-from",
            "binary-files",
            "directories",
            "devices",
        ],
    ),
    (
        "sort",
        "kotTS",
        &[
            "key",
            "output",
            "field-separator",
            "temporary-directory",
            "buffer-size",
            "parallel",
            "files0-from",
            "batch-size",
            "compress-program",
            "random-source",
        ],
    ),
    (
        "tail",
        "ncs",
        &[
            "lines",
            "bytes",
            "pid",
            "sleep-interval",
            "max-unchanged-stats",
        ],
    ),
    ("uniq", "fsw", &["skip-fields", "skip-chars", "check-chars"]),
];

/// Whether a pipeline stage passes on `find -print0`'s names with each
/// still starting where find started it: `tee` and `head` (which keep
/// whole lines or cut a tail), `uniq` unless it drops every copy of a line
/// (`-u`, `-d`, `-D` without `-z`), or `grep`, `sort` and `tail` that read
/// NUL-ended records (`-z`, `--zero-terminated`, `--null-data`) and keep
/// them whole (not `grep -o`, `tail -c`). Line-based ones may start
/// an item at a newline in a name (`./x<newline>--checkpoint-action=…`).
fn keeps_nul_names(name: &str, args: &[String]) -> bool {
    // GNU tools as Homebrew installs them (`gsort`); `grep` itself starts
    // with `g`.
    let name = match name.strip_prefix('g') {
        Some(rest @ ("grep" | "egrep" | "fgrep" | "sort" | "tail" | "head" | "uniq" | "tee")) => {
            rest
        }
        _ => name,
    };
    let name = match name {
        "egrep" | "fgrep" => "grep",
        "tee" | "head" => return true,
        name => name,
    };
    let Some((_, value_letters, value_longs)) =
        NUL_FILTERS.iter().find(|(filter, _, _)| *filter == name)
    else {
        return false;
    };
    let (mut zero, mut cut) = (false, false);
    let mut index = 0;
    while let Some(word) = args.get(index) {
        work::step();
        index += 1;
        if word == "--" {
            break;
        }
        if let Some(long) = word.strip_prefix("--") {
            let (option, value) = match long.split_once('=') {
                Some((option, value)) => (option, Some(value)),
                None => (long, None),
            };
            // Only the full names count as NUL-ended reading; any
            // abbreviation counts as cutting or taking a value.
            zero |= option == "zero-terminated" || (name == "grep" && option == "null-data");
            let abbreviates = |full: &str| !option.is_empty() && full.starts_with(option);
            cut |= (name == "grep" && abbreviates("only-matching"))
                || (name == "tail" && abbreviates("bytes"))
                || (name == "uniq"
                    && ["unique", "repeated", "all-repeated"]
                        .iter()
                        .any(|full| abbreviates(full)));
            if value.is_none() && value_longs.iter().any(|long| abbreviates(long)) {
                index += 1;
            }
            continue;
        }
        let Some(letters) = word.strip_prefix('-').filter(|rest| !rest.is_empty()) else {
            continue;
        };
        for (offset, letter) in letters.char_indices() {
            work::step();
            match letter {
                'z' => zero = true,
                'o' if name == "grep" => cut = true,
                'c' if name == "tail" => cut = true,
                'u' | 'd' | 'D' if name == "uniq" => cut = true,
                _ => {}
            }
            if value_letters.contains(letter) {
                index += usize::from(offset + letter.len_utf8() == letters.len());
                break;
            }
        }
    }
    // `uniq` keeps a line's first copy, so line by line it keeps each
    // record's start unless it drops all copies (`-u`, `-d`, `-D`).
    if name == "uniq" {
        zero || !cut
    } else {
        zero && !cut
    }
}

/// What each of `commands` gets from the pipeline stages before it: the
/// most open [`Feed`] of any of them.
pub(super) fn feeds(commands: &[Command]) -> Vec<Feed> {
    let mut feeds = vec![Feed::Other; commands.len()];
    for (index, command) in commands.iter().enumerate() {
        work::step();
        let Some(to) = command.pipe_to.filter(|to| *to < commands.len()) else {
            continue;
        };
        let texts: Vec<String> = command.words.iter().map(|word| word.text.clone()).collect();
        let command_at = indirect::command_word(&texts);
        let own = match &command_at {
            Some((_, name)) if LISTERS.contains(&name.as_str()) => Feed::Listing,
            Some((at, name)) if FINDERS.contains(&name.as_str()) => {
                let nul = texts[at + 1..].iter().any(|text| text == "-print0")
                    && !texts[at + 1..].iter().any(|text| {
                        work::step();
                        FIND_OUTPUTS
                            .iter()
                            .any(|output| text.contains(output) && !text.contains("-print0"))
                    });
                if nul { Feed::FindNul } else { Feed::Listing }
            }
            _ => Feed::Other,
        };
        // A stage other than such a filter may change `find`'s names
        // (`xargs -0 basename -z`, `sort`, `grep -zo`).
        let passed = match (feeds[index], &command_at) {
            (Feed::FindNul, Some((at, name))) if keeps_nul_names(name, &texts[at + 1..]) => {
                Feed::FindNul
            }
            (Feed::FindNul, _) => Feed::Listing,
            (feed, _) => feed,
        };
        feeds[to] = feeds[to].max(passed).max(own);
    }
    feeds
}

/// How `xargs` passes on its input.
struct XargsInput {
    /// Where the command it runs starts.
    command: usize,
    /// Where it puts its input: at the end (`None`), or where this string is
    /// (`-I{}`, `-i`, `--replace`, BSD `-J`).
    replace: Option<String>,
    /// It splits its input at NULs only (`-0`, `--null`).
    nul: bool,
}

/// `xargs` letters (GNU and BSD) whose value is the rest of the word or,
/// when that is empty, the next one.
const XARGS_VALUE_LETTERS: &str = "adEILJnPRSs";

/// GNU `xargs`' long options, and whether each takes a value (`None`: only
/// after `=`). getopt takes an unambiguous abbreviation.
const XARGS_LONG: &[(&str, Option<bool>)] = &[
    ("arg-file", Some(true)),
    ("delimiter", Some(true)),
    ("eof", None),
    ("exit", Some(false)),
    ("help", Some(false)),
    ("interactive", Some(false)),
    ("max-args", Some(true)),
    ("max-chars", Some(true)),
    ("max-lines", None),
    ("max-procs", Some(true)),
    ("no-run-if-empty", Some(false)),
    ("null", Some(false)),
    ("open-tty", Some(false)),
    ("process-slot-var", Some(true)),
    ("replace", None),
    ("show-limits", Some(false)),
    ("verbose", Some(false)),
    ("version", Some(false)),
];

/// Reads `xargs`' options from `texts[from..]`, up to the first word that
/// is not one or its value (as `xargs` does; later options are the
/// command's: `xargs env -i tar`).
fn xargs_input(texts: &[String], from: usize) -> Option<XargsInput> {
    let (mut replace, mut nul) = (None, false);
    let mut index = from;
    while let Some(word) = texts.get(index) {
        work::step();
        index += 1;
        if word == "--" {
            break;
        }
        if let Some(long) = word.strip_prefix("--") {
            let (name, value) = match long.split_once('=') {
                Some((name, value)) => (name, Some(value)),
                None => (long, None),
            };
            let exact = XARGS_LONG.iter().find(|(option, _)| *option == name);
            let mut prefixed = XARGS_LONG.iter().filter(|(option, _)| {
                work::step();
                !name.is_empty() && option.starts_with(name)
            });
            let found = exact.or_else(|| match (prefixed.next(), prefixed.next()) {
                (Some(only), None) => Some(only),
                _ => None,
            });
            match found {
                Some(("null", _)) => nul = true,
                // The last of `-0` and `-d` sets the delimiter.
                Some(("delimiter", _)) => {
                    nul = false;
                    index += usize::from(value.is_none());
                }
                Some(("replace", _)) => {
                    replace = Some(value.unwrap_or("{}").to_string());
                }
                Some((_, Some(true))) if value.is_none() => index += 1,
                _ => {}
            }
            continue;
        }
        let Some(letters) = word.strip_prefix('-').filter(|rest| !rest.is_empty()) else {
            index -= 1;
            break;
        };
        for (offset, letter) in letters.char_indices() {
            work::step();
            let rest = &letters[offset + letter.len_utf8()..];
            match letter {
                '0' => nul = true,
                // Optional values, in the rest of the word only.
                'i' => {
                    replace = Some(if rest.is_empty() { "{}" } else { rest }.to_string());
                    break;
                }
                'e' | 'l' => break,
                'd' => {
                    nul = false;
                    index += usize::from(rest.is_empty());
                    break;
                }
                _ if XARGS_VALUE_LETTERS.contains(letter) => {
                    let value = if rest.is_empty() {
                        index += 1;
                        texts.get(index - 1).map_or("", String::as_str)
                    } else {
                        rest
                    };
                    if matches!(letter, 'I' | 'J') {
                        replace = Some(value.to_string());
                    }
                    break;
                }
                _ => {}
            }
        }
    }
    // An empty replacement string would match every word: read it as added
    // at the end, which asks as well.
    let replace = replace.filter(|replace: &String| !replace.is_empty());
    (index < texts.len()).then_some(XargsInput {
        command: index,
        replace,
        nul,
    })
}

/// A loop variable taken from a glob, and what that glob may give it.
struct GlobLoop {
    name: String,
    /// A value starting with `-` (`*`, `*.txt`; not `./*`)...
    dash: bool,
    /// ...and ending in any text (`*`; not `*.txt`).
    open: bool,
}

/// D484 (b): the variables of `for`/`select`/`foreach` loops over a glob
/// met so far in a script, read on to its end. Past `MAX_GLOB_LOOPS`, any
/// variable may be one.
#[derive(Default)]
pub(super) struct GlobLoops {
    loops: Vec<GlobLoop>,
    any: bool,
    /// A `for NAME` whose `in` may start the next command (bash reads
    /// `for f` and `in *` on lines of their own).
    pending: Option<String>,
}

const MAX_GLOB_LOOPS: usize = 8;

/// Reserved words the reading keeps before a loop in the same command.
const LOOP_PREFIXES: &[&str] = &[
    "!", "if", "then", "elif", "else", "do", "while", "until", "time", "{", "coproc",
];

impl GlobLoops {
    /// Takes in a simple command (`for f in *`). zsh's `for f (*)` and
    /// `foreach f (*)`, which this reading cuts at `(`, count as `*`.
    pub(super) fn read(&mut self, command: &Command) {
        let pending = self.pending.take();
        // Past reserved words that may start it (`do for f in *`, `then
        // for …`, `time -p for …`).
        let mut skipped = 0;
        while let Some(word) = command.words.get(skipped) {
            work::step();
            let time_option =
                word.text == "-p" && skipped > 0 && command.words[skipped - 1].text == "time";
            if !(LOOP_PREFIXES.contains(&word.text.as_str()) || time_option) {
                break;
            }
            skipped += 1;
        }
        let words = &command.words[skipped..];
        let keyword = words.first().is_some_and(|word| {
            matches!(
                word.text.to_lowercase().as_str(),
                "for" | "select" | "foreach"
            )
        });
        // `for f` then `in *` (a newline between them).
        let (name, list) = match (pending, command.words.first()) {
            (Some(name), Some(first)) if first.text == "in" => (name, Some(&command.words[1..])),
            _ => {
                let Some(name) = words
                    .get(1)
                    .filter(|_| keyword)
                    .map(|word| word.text.clone())
                else {
                    return;
                };
                let list = words
                    .get(2)
                    .is_some_and(|word| word.text == "in")
                    .then(|| &words[3..]);
                (name, list)
            }
        };
        if name.is_empty()
            || !name
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
        {
            return;
        }
        let (mut glob, mut dash, mut open) = (false, false, false);
        if let Some(list) = list {
            for word in list.iter().take_while(|word| word.text != "do") {
                work::step();
                glob |= word.globs;
                if may_match_option(word) {
                    dash = true;
                    open |= ends_open(&word.text);
                }
            }
        } else if words.len() == 2 && command.cut {
            (glob, dash, open) = (true, true, true);
        } else if words.len() == 2 {
            self.pending = Some(name);
            return;
        }
        if !glob {
            return;
        }
        if let Some(known) = self.loops.iter_mut().find(|known| known.name == name) {
            known.dash |= dash;
            known.open |= open;
        } else if self.loops.len() == MAX_GLOB_LOOPS {
            self.any = true;
        } else {
            self.loops.push(GlobLoop { name, dash, open });
        }
    }

    /// Whether `word` may name an option through a loop variable: it starts
    /// with one whose glob may start with `-` (`"$f"`, `${f%.txt}`), it
    /// changes one's start (`${f#./}`, `${f##*/}`), or the shell may split
    /// it (`$f` unquoted, `./a --x` from `./*`). `Some(open)`: whether
    /// its value may end in any text.
    pub(super) fn option(&self, word: &Word) -> Option<bool> {
        if !word.dynamic || (self.loops.is_empty() && !self.any) {
            return None;
        }
        let text = word.text.as_str();
        // `"$x$f"`: values before it may be empty; a `/` before it starts
        // the word.
        let built_start = text.starts_with(['$', '`']) || text.starts_with(UNSEEN_OUTPUT);
        let slash = text.find('/');
        let mut found: Option<bool> = None;
        for (at, _) in text.match_indices('$') {
            work::step();
            let after = &text[at + 1..];
            let braced = after.starts_with('{');
            let after = after.strip_prefix('{').unwrap_or(after);
            let end = after
                .find(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
                .unwrap_or(after.len());
            let (name, rest) = after.split_at(end);
            if name.is_empty() {
                continue;
            }
            let known = self.loops.iter().find(|known| known.name == name);
            let (dash, open) = match known {
                Some(known) => (known.dash, known.open),
                None if self.any => (true, true),
                None => continue,
            };
            // `${f}`, `${f%x}`, `${f:-x}`: the value's start is kept.
            let kept = !braced
                || rest.starts_with(['}', '%', '-', '=', '?', '+'])
                || (rest.starts_with(':') && rest[1..].starts_with(['-', '=', '?', '+']));
            let option = if word.splits {
                // Split into words, one of which may start anywhere in it.
                Some(true)
            } else if at > 0 && !(built_start && slash.is_none_or(|slash| slash > at)) {
                None
            } else if kept {
                // `${f%.txt}` may end anywhere in the value.
                dash.then_some(open || rest.starts_with('%'))
            } else {
                Some(true)
            };
            found = found.max(option);
        }
        found
    }
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
