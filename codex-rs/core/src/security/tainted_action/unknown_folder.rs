//! #1309: more ways the folder a nested script runs in becomes one the
//! quote-aware reading (`nested.rs`) cannot know, beyond `cd "$d"` and a
//! literal `-C` (#1283). Such a script's relative writes and output files
//! ask (`Classifier::unknown_place`).

use super::FOLDER_FLAGS;
use super::shell::basename;
use super::work;

/// Long options that run a command in another folder (`sudo --chdir`,
/// `nsenter --wd`, `docker exec --workdir`, `systemd-run
/// --working-directory`, `fd --base-directory`). getopt takes any
/// unambiguous abbreviation (`env --chd`), so a prefix of one counts too.
const LONG_FOLDER_FLAGS: &[&str] = &[
    "--chdir",
    "--cwd",
    "--wdns",
    "--cd",
    "--directory",
    "--wd",
    "--workdir",
    "--work-dir",
    "--working-dir",
    "--working-directory",
    "--base-directory",
];

/// Short folder options of wrappers, also in a cluster (`env -iC dir`) or
/// with the folder attached (`sudo -D~`).
const SHORT_FOLDER_FLAGS: &[(&str, char)] = &[
    ("env", 'C'),
    ("sudo", 'D'),
    ("nsenter", 'w'),
    ("nsenter", 'W'),
    ("unshare", 'w'),
    ("docker", 'w'),
    ("podman", 'w'),
    ("nerdctl", 'w'),
    ("proot", 'w'),
    ("start-stop-daemon", 'd'),
];

/// Commands that run what they are given in another folder whatever their
/// options.
const ELSEWHERE: &[&str] = &["chroot"];

/// Options that run a command in the folder of each file found.
const IN_FOUND_FOLDER: &[&str] = &["-execdir", "-okdir"];

/// Words that start a loop: a later iteration runs its first commands in
/// the folder the earlier one left.
const LOOPS: &[&str] = &["for", "while", "until", "select", "repeat", "foreach"];

/// Words that define code run later than it is read: functions, traps and
/// aliases.
const DEFINERS: &[&str] = &["function", "trap", "alias"];

/// Commands that run text in the current shell, so a `cd` in it moves the
/// line (`eval 'cd ~'`, `alias go='cd ~'`).
const CARRIERS: &[&str] = &["eval", "alias", "trap", "bind", "fc"];

/// Folder changes.
pub(super) const CHANGERS: &[&str] = &["cd", "pushd", "popd", "chdir"];

/// Names that change where `cd` goes (`CDPATH`, zsh `cdpath`, bash
/// `cdable_vars`, `OLDPWD` for `cd -`, zsh `AUTO_CD`), lowercased with
/// underscores removed (zsh option spelling).
const SEARCHES: &[&str] = &["cdpath", "cdablevars", "oldpwd", "autocd"];

/// Whether a command with these words runs the scripts it is given in a
/// folder other than the line's: a wrapper's folder option in any
/// spelling, `find -execdir`, `chroot` or a systemd `WorkingDirectory=`.
pub(super) fn runs_elsewhere(texts: &[String]) -> bool {
    let mut letters: Vec<char> = Vec::new();
    for text in texts {
        work::step();
        let flag = text.split_once('=').map_or(text.as_str(), |(flag, _)| flag);
        if FOLDER_FLAGS.contains(&flag)
            // `env -C"$d"`: the folder attached.
            || text.starts_with("-C")
            || IN_FOUND_FOLDER.contains(&text.as_str())
            || ELSEWHERE.contains(&basename(text))
            || (flag.len() > 2
                && flag.starts_with("--")
                && LONG_FOLDER_FLAGS.iter().any(|long| {
                    work::step();
                    long.starts_with(flag)
                }))
            || text.as_bytes().windows(16).any(|window| {
                work::step();
                window.eq_ignore_ascii_case(b"workingdirectory")
            })
        {
            return true;
        }
        if text.len() > 1
            && text.starts_with('-')
            && !text.starts_with("--")
            && text[1..].chars().any(|letter| {
                work::step();
                letters.contains(&letter)
            })
        {
            return true;
        }
        for (wrapper, letter) in SHORT_FOLDER_FLAGS {
            work::step();
            if basename(text) == *wrapper && !letters.contains(letter) {
                letters.push(*letter);
            }
        }
    }
    false
}

/// Whether `text` defines code that runs later than it is read: a
/// function (`f() { ...; }`, `function f`), a trap or an alias.
pub(super) fn defines(text: &str) -> bool {
    // `f()`, `f ( )`.
    let mut open = false;
    for ch in text.chars() {
        work::step();
        match ch {
            '(' => open = true,
            ')' if open => return true,
            ch if ch.is_whitespace() => {}
            _ => open = false,
        }
    }
    tokens(text).any(|token| DEFINERS.contains(&token))
}

/// Whether a command whose words are `texts` may start a loop: the
/// reading keeps reserved words before it (`then for ...`, `do while
/// ...`), so any word counts.
pub(super) fn starts_loop(texts: &[String]) -> bool {
    texts.iter().any(|text| {
        work::step();
        LOOPS.contains(&text.as_str())
    })
}

/// Whether `text` names something that changes where `cd` goes.
pub(super) fn searches(text: &str) -> bool {
    if text == "HOME" || text.starts_with("HOME=") || text.starts_with("HOME+=") {
        return true;
    }
    let mut normal = String::with_capacity(text.len());
    for ch in text.chars() {
        work::step();
        if ch != '_' {
            normal.push(ch.to_ascii_lowercase());
        }
    }
    SEARCHES.iter().any(|name| {
        work::step();
        normal.contains(name)
    })
}

/// Whether `name` (a command word) runs text that may change the folder of
/// the line: a sourced file, or `eval`/`alias`/`trap` text with a `cd`, or
/// with any character the shell may quote, expand or match when it reads
/// the text again (`eval 'c\d ~'`, `alias go='c""d ~'`, `eval 'c?'`,
/// `eval '{c,}d'`).
pub(super) fn carries_change(name: &str, args: &[String]) -> bool {
    matches!(name, "source" | ".")
        || (CARRIERS.contains(&name)
            && args.iter().any(|arg| {
                work::step();
                !arg.chars().all(|ch| {
                    work::step();
                    ch.is_ascii_alphanumeric()
                        || matches!(ch, '_' | '-' | '.' | '/' | '=' | ':' | '@' | '%' | ' ')
                }) || tokens(arg)
                    .any(|token| CHANGERS.contains(&token) || CARRIED_LOADERS.contains(&token))
                    || arg.split([' ', '=']).any(|piece| piece == ".")
            }))
}

/// Commands that, run from carrier text, load more text into the current
/// shell (`eval . ./f`, `eval source f`, `alias go=eval`).
const CARRIED_LOADERS: &[&str] = &["source", "eval", "alias", "autoload"];

/// Whether a folder change goes where the walk does not follow: zsh
/// `chdir`, a place on the folder stack (`cd -2`, `pushd +1`, `popd +1`, a
/// bare `pushd`, `cd ~1`), `cd ~-`/`~+` and other tilde forms (a named
/// folder, `cd ~h`), `pushd -n`, zsh's two-word `cd old new`, operands
/// after `--`, or `cd -`: a change the walk followed may not have
/// happened (`false && cd x`, `(cd x)`), so it may go to the inherited
/// `OLDPWD`.
pub(super) fn odd_change(name: &str, args: &[String]) -> bool {
    if name == "chdir" {
        return true;
    }
    let mut operands = 0;
    for arg in args {
        work::step();
        if arg == "--" || arg.starts_with('+') {
            return true;
        }
        // `pushd -` passes `-` on to `cd`.
        if arg == "-" && matches!(name, "cd" | "pushd") {
            return true;
        }
        if arg.starts_with('-') && arg != "-" {
            // `pushd -n`, `popd -n`: the stack changes, the folder does not.
            if name != "cd" || arg[1..].starts_with(|ch: char| ch.is_ascii_digit()) {
                return true;
            }
            continue;
        }
        if arg.starts_with('~') && arg != "~" && !arg.starts_with("~/") {
            return true;
        }
        operands += 1;
    }
    match name {
        "popd" => !args.is_empty(),
        "pushd" => operands == 0,
        _ => operands > 1,
    }
}

/// The names in `text`: runs of letters, digits and underscores.
fn tokens(text: &str) -> impl Iterator<Item = &str> {
    text.split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
        .filter(|token| !token.is_empty())
}
