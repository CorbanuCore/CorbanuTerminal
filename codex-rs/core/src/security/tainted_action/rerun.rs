//! #1064: besides `eval` and `trap`, commands that run text as code, now or
//! later: zsh `print -P` (with `promptsubst` the prompt expansion runs the
//! substitutions in its text), builtins that keep a command or load code by
//! a name (`fc -e`, `bind -x`, `complete -C`, `zle -N`, `autoload`,
//! `enable -f`), terminal multiplexers that start or type a command (`tmux
//! new-session`, `tmux send-keys`), `parallel` running its arguments, and
//! options whose value is a command: `find -exec`, `split --filter`, git's
//! `rebase -x`, `submodule foreach`, `bisect run`, `filter-branch
//! --tree-filter`, `difftool -x`. An operand holding a value the classifier
//! cannot see is code it cannot read.
//!
//! #1159: git options that make git run a program the classifier does not
//! read whatever their value: `--upload-pack`/`--receive-pack`, `--exec` (of
//! `push`, `archive`, `send-pack`, `fetch-pack`), `clone -u`, hooks copied by `clone`/`init --template`, and config set on
//! the new repository by `clone -c`/`--config`.

use super::git_helpers;
use super::work;

/// Commands that run, or keep as code, their operands. Without a command
/// before `:::`, `parallel` runs its arguments; which of its words are
/// option values is not known, so any of them counts.
const RUNS_OPERANDS: &[&str] = &[
    "fc", "bind", "complete", "compgen", "zle", "autoload", "zmodload", "enable", "tmux", "screen",
    "parallel",
];

/// Commands whose words after one of these words are a command it runs:
/// (command, word that must also be on the line or "", words starting the
/// command). For git, as its option parser reads them, a short option also
/// starts it in a group or with its value attached (`-ix`, `-x"$v"`), and a
/// long one by any prefix from four characters, its value attached or not
/// (`--exe="$v"`).
const COMMAND_OPTIONS: &[(&str, &str, &[&str])] = &[
    ("find", "", &["-exec", "-execdir", "-ok", "-okdir"]),
    ("git", "rebase", &["-x", "--exec"]),
    ("git", "submodule", &["foreach"]),
    ("git", "bisect", &["run"]),
    ("git", "difftool", &["-x", "--extcmd"]),
    (
        "git",
        "filter-branch",
        &[
            "--env-filter",
            "--tree-filter",
            "--index-filter",
            "--parent-filter",
            "--msg-filter",
            "--commit-filter",
            "--tag-name-filter",
        ],
    ),
];

/// Whether a simple command runs an operand holding a value the classifier
/// cannot see. `index` and `name` are its command word's index and
/// lowercase basename; `unseen` marks the words holding such a value.
pub(super) fn runs_unseen(name: &str, words: &[String], unseen: &[bool], index: usize) -> bool {
    let after = |from: usize| unseen.iter().skip(from).any(|unseen| *unseen);
    if RUNS_OPERANDS.contains(&name) {
        return after(index + 1);
    }
    // Literal substitutions in its text (`print -P '$(cmd)'`) are read as
    // commands already.
    if name == "print" && prints_prompt(&words[index + 1..]) {
        return after(index + 1);
    }
    // `split --filter` runs its command on each piece of a file: its input
    // is code too (`--filter=sh`).
    if name == "split"
        && words[index + 1..]
            .iter()
            .any(|word| word.starts_with("--fi"))
    {
        return true;
    }
    // A dashed helper (`git-rebase`) is `git rebase`; `git.exe` is git.
    let helper = git_helpers::git_name(name);
    let subcommand = helper.as_ref().and_then(Option::as_deref);
    let name = if helper.is_some() { "git" } else { name };
    for (command, required, starts) in COMMAND_OPTIONS {
        work::step();
        if *command != name
            || !(required.is_empty()
                || subcommand == Some(*required)
                || words[index + 1..].iter().any(|word| word == required))
        {
            continue;
        }
        let git = name == "git";
        let start = words[index + 1..].iter().position(|word| {
            starts.iter().any(|start| {
                let long = word.split_once('=').map_or(word.as_str(), |(name, _)| name);
                word == start
                    || (start.starts_with("--")
                        && (long == *start || (git && long.len() >= 4 && start.starts_with(long))))
                    || (git
                        && start.len() == 2
                        && start.starts_with('-')
                        && word.starts_with('-')
                        && !word.starts_with("--")
                        && word[1..].contains(&start[1..]))
            })
        });
        if let Some(start) = start
            && after(index + 1 + start)
        {
            return true;
        }
    }
    false
}

/// Whether zsh `print`'s words (`args`, those after it) include `-P`. Read
/// loosely, so as to ask: any option word before `-` or `--` counts, past
/// option values and operands.
fn prints_prompt(args: &[String]) -> bool {
    for word in args {
        work::step();
        if word == "-" || word == "--" {
            return false;
        }
        if word.starts_with('-') && !word.starts_with("--") && word.contains('P') {
            return true;
        }
    }
    false
}

/// #1159: whether git's words after `git` (`args`) name a program git runs
/// that the classifier does not read, or copy hooks or set config on a new
/// repository: `--upload-pack`, `--receive-pack`, `push`/`archive --exec`,
/// `clone -u`, `clone`/`init --template`, `clone -c`/`--config`. git takes
/// any unambiguous prefix of a long option (`--upload`, `--templ`).
/// `subcommand`: a dashed helper's (`git-clone`). Read once per command,
/// from its first git word to its end.
pub(super) fn git_runs_program(args: &[String], subcommand: Option<&str>) -> bool {
    // A later dashed helper (`git-clone`) names its subcommand too.
    let has = |word: &str| {
        subcommand == Some(word)
            || args.iter().any(|arg| {
                work::step();
                arg == word || git_helpers::git_name(arg).flatten().as_deref() == Some(word)
            })
    };
    let clone = has("clone");
    let creates = clone || has("init");
    let exec = has("push") || has("archive") || has("send-pack") || has("fetch-pack");
    let long = |word: &str, option: &str, shortest: usize| {
        let name = word.split_once('=').map_or(word, |(name, _)| name);
        name.len() >= shortest && option.starts_with(name)
    };
    args.iter().any(|word| {
        work::step();
        if word.starts_with("--") {
            long(word, "--upload-pack", "--up".len())
                || long(word, "--receive-pack", "--rec".len())
                || (exec && long(word, "--exec", "--ex".len()))
                || (creates && long(word, "--template", "--te".len()))
                || (clone && long(word, "--config", "--con".len()))
        } else {
            // A group of short options (`-qc k=v`, `-u./x`).
            clone
                && word
                    .strip_prefix('-')
                    .is_some_and(|cluster| cluster.contains(['c', 'u']))
        }
    })
}

/// #1064: the names a nameref declaration makes (`declare -n r=v`, `local
/// -n r`, `typeset -n`, ksh `nameref r=v`): each reads another variable's
/// value, which the substituted words do not follow. `command` is the
/// command word's index and lowercase name.
pub(super) fn namerefs<'a>(command: Option<&(usize, String)>, words: &'a [String]) -> Vec<&'a str> {
    let Some((index, name)) = command else {
        return Vec::new();
    };
    let operands = &words[index + 1..];
    let nameref = name == "nameref"
        || (matches!(name.as_str(), "declare" | "typeset" | "local")
            && operands.iter().any(|word| {
                word.starts_with('-') && !word.starts_with("--") && word.contains('n')
            }));
    if !nameref {
        return Vec::new();
    }
    operands
        .iter()
        .filter(|word| !word.starts_with(['-', '+']))
        .map(|word| word.split_once('=').map_or(word.as_str(), |(name, _)| name))
        .collect()
}
