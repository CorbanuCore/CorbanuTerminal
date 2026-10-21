//! D85: whether a command line provably never reaches zsh, so the zsh-only
//! checks (glob qualifiers #1028, glob substitution #1084) need not ask for
//! it (`git commit -m "fix(tests)"` under bash). #932 phase 3
//! ([`grammar_proves_data`]) also lifts them under any system shell, zsh
//! and any `sh` included, for lines whose every word the grammar proves is
//! data.
//!
//! Fail closed: only a line the host runs under the bash, dash or sh its
//! argv names by system path (an sh that is not zsh), whose script is plain
//! commands with literal words (a real bash parse: no substitutions,
//! parameter expansions, redirections, groups, assignments or control flow),
//! each a program that never hands its words to a shell. Where the parse
//! and bash may split words differently ([`splits_differently`]) the line
//! keeps the checks; braces and glob characters, which bash still expands,
//! must stay out of words whose meaning matters. Anything else keeps the
//! checks, a shell named without a system path included.

use super::grammar;
use super::quoted;
use super::shadow;
use super::shadow::Comparison;
use codex_shell_command::bash::PlainWord;

/// Longer scripts are not parsed (they keep the checks).
const MAX_SCRIPT_BYTES: usize = 64 * 1024;
/// Shells, by system path, that are never zsh.
const NOT_ZSH: &[&str] = &["/bin/bash", "/usr/bin/bash", "/bin/dash", "/usr/bin/dash"];
/// `sh`, which counts only when it is one of those.
const SYSTEM_SH: &[&str] = &["/bin/sh", "/usr/bin/sh"];
/// Programs that only print, read or name files and never run their words,
/// whatever their words expand to (not `printf`: `-v` assigns a variable).
const DATA_ONLY: &[&str] = &[
    "echo", "cd", "pwd", "true", "false", "ls", "cat", "head", "tail", "wc", "grep", "mkdir",
    "touch",
];
/// Git subcommands that run nothing named on the line (hooks, pagers and
/// filters come from configuration, which the classifier protects), without
/// `--output` (a file the classifier does not judge as written, #1141) and
/// without words bash may expand into it: unquoted braces or glob
/// characters (`{--output=x,}`, a glob matching a file so named).
const GIT_DATA_ONLY: &[&str] = &[
    "status", "diff", "log", "show", "add", "commit", "restore", "switch", "checkout", "branch",
    "rm", "mv", "stash",
];

/// Whether `command`, run under the shell it names, never reaches zsh.
pub(super) fn avoids_zsh(command: &[String]) -> bool {
    let [shell, flag, script] = command else {
        return false;
    };
    if !matches!(flag.as_str(), "-c" | "-lc")
        || script.len() > MAX_SCRIPT_BYTES
        || splits_differently(script)
        || !not_zsh(shell)
    {
        return false;
    }
    codex_shell_command::bash::parse_shell_script_into_plain_words(script).is_some_and(|commands| {
        !commands.is_empty() && commands.iter().all(|words| data_only(words))
    })
}

/// #932 phase 3: the zsh-only checks need not ask for a line, under a
/// system bash, zsh, dash or sh, when the bash grammar proves every word of it is
/// data no shell expands (`grammar.rs`) and the line is D85's plain
/// commands of programs that never run their words. No unquoted glob
/// character, parenthesis or expansion is left for a zsh glob qualifier or
/// glob substitution to act on.
///
/// Fail closed: the grammar must read the line (no refusal), find nothing
/// zsh may read differently, and find the same commands as the quote-aware
/// reader; the line must hold no substitution, no `''` (zsh's `RC_QUOTES`
/// reads `'a''b'` as `a'b`) and no `cd` (zsh runs `chpwd` hooks).
pub(super) fn grammar_proves_data(command: &[String]) -> bool {
    /// Shells by system path (as D85, a shell named otherwise may be
    /// anything).
    const SYSTEM_SHELLS: &[&str] = &[
        "/bin/bash",
        "/usr/bin/bash",
        "/bin/zsh",
        "/usr/bin/zsh",
        "/bin/sh",
        "/usr/bin/sh",
        "/bin/dash",
        "/usr/bin/dash",
    ];
    let [shell, flag, script] = command else {
        return false;
    };
    if !matches!(flag.as_str(), "-c" | "-lc")
        || !SYSTEM_SHELLS.contains(&shell.as_str())
        || script.len() > grammar::MAX_BYTES
        || splits_differently(script)
        || script.contains("''")
    {
        return false;
    }
    let Ok(reading) = grammar::read(script) else {
        return false;
    };
    let mut budget = usize::MAX;
    let (comparison, _) = shadow::compare(script, &quoted::commands(script), &mut budget);
    !reading.zsh_unsure
        && !reading.bash_only
        && comparison == Comparison::Agree
        && !reading.commands.is_empty()
        && reading.commands.iter().all(|command| {
            !command.substituted
                && command.words.first().is_some_and(|name| name.text != "cd")
                && command.words.iter().all(|word| word.data)
        })
        && codex_shell_command::bash::parse_shell_script_into_plain_words(script).is_some_and(
            |commands| {
                commands.len() == reading.commands.len()
                    && commands.iter().all(|words| data_only(words))
            },
        )
}

/// Text the parse may split into words other than bash's: a backslash
/// (`diff\<newline>tool` is one bash word, two parsed words), a control
/// character other than newline or tab, or a non-ASCII character before
/// `|`, a brace, a bracket or `~` (in a GBK, Big5 or Shift-JIS locale its
/// last byte may start a character that takes that one in).
fn splits_differently(script: &str) -> bool {
    let mut previous_non_ascii = false;
    script.chars().any(|ch| {
        let differs = ch == '\\'
            || (ch.is_control() && !matches!(ch, '\n' | '\t'))
            || (previous_non_ascii && matches!(ch, '|' | '{' | '}' | '[' | ']' | '~'));
        previous_non_ascii = !ch.is_ascii();
        differs
    })
}

fn not_zsh(shell: &str) -> bool {
    NOT_ZSH.contains(&shell) || (SYSTEM_SH.contains(&shell) && system_sh_not_zsh(shell))
}

/// macOS runs `/bin/sh` as the shell `/private/var/select/sh` names.
#[cfg(target_os = "macos")]
fn system_sh_not_zsh(_shell: &str) -> bool {
    std::fs::read_link("/private/var/select/sh")
        .is_ok_and(|target| matches!(target.to_str(), Some("/bin/bash" | "/bin/dash")))
}

#[cfg(all(unix, not(target_os = "macos")))]
fn system_sh_not_zsh(shell: &str) -> bool {
    std::fs::canonicalize(shell).is_ok_and(|target| {
        target
            .to_str()
            .is_some_and(|target| NOT_ZSH.contains(&target))
    })
}

#[cfg(not(unix))]
fn system_sh_not_zsh(_shell: &str) -> bool {
    false
}

fn data_only(words: &[PlainWord]) -> bool {
    let Some((name, rest)) = words.split_first() else {
        return false;
    };
    match name.text.as_str() {
        "git" => {
            rest.first()
                .is_some_and(|subcommand| GIT_DATA_ONLY.contains(&subcommand.text.as_str()))
                && !rest.iter().any(|word| {
                    word.text.starts_with("--output")
                        || (word.unquoted && word.text.contains(['{', '*', '?', '[']))
                })
        }
        name => DATA_ONLY.contains(&name),
    }
}
