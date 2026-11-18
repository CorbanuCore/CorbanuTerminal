//! #2548 (D553): what the program of a script command does with its words
//! and its input. Rules see a command's argv; a few programs run text that
//! is not their argv: `eval`, a shell reading a script from stdin or a
//! file, `source`, and builtins that define or run code. Default deny: every
//! builtin and reserved word of bash, zsh and dash that is not known to run
//! nothing but its argv is [`Role::Unread`].

use crate::security::tainted_action::policy_runs_powershell;
use crate::security::tainted_action::policy_runs_shell;
use codex_shell_command::bash::ARITHMETIC_TESTS;
use codex_shell_command::bash::CommandStdin;
use codex_shell_command::bash::Dialect;
use codex_shell_command::bash::RESERVED_WORDS;
use codex_shell_command::bash::numeric_operand;
use codex_shell_command::bash::runs_code_from_variable;
use codex_shell_command::bash::script_dialect;

/// What a command runs besides its argv.
#[derive(Debug, PartialEq, Eq)]
pub(super) enum Role {
    /// Nothing: rules see what it runs in its argv.
    Argv,
    /// This bash script text (`eval`'s words, a shell's here-document).
    /// `single`: the shell reads its script from the same input its
    /// commands read, so only a single command is read precisely.
    Script {
        text: String,
        single: bool,
        dialect: Dialect,
    },
    /// Code rules cannot read: a script from a pipe or file, a sourced
    /// file, a trap, an alias, arithmetic, an option that runs code.
    Unread,
}

/// The role of `program_and_args`, run after the launcher words `before`
/// (`env A=1`, `sudo`) with input `stdin`.
pub(super) fn role(
    before: &[String],
    program_and_args: &[String],
    stdin: &CommandStdin,
    dialect: Dialect,
) -> Role {
    if before.iter().any(|word| assigns_code_variable(word)) {
        return Role::Unread;
    }
    let Some((program, args)) = program_and_args.split_first() else {
        return Role::Argv;
    };
    if runs_shell(program) {
        return shell_role(program, args, stdin);
    }
    let known: Vec<Option<String>> = args.iter().cloned().map(Some).collect();
    match program.as_str() {
        "eval" => {
            let args = args.strip_prefix(&["--".to_string()]).unwrap_or(args);
            if args.is_empty() {
                Role::Argv
            } else if args[0].starts_with('-') {
                // bash refuses options; read nothing as precise.
                Role::Unread
            } else {
                Role::Script {
                    text: args.join(" "),
                    single: false,
                    dialect,
                }
            }
        }
        // A launcher with no command of its own: its redirections apply to
        // the rest of the script (`exec <<EOF … EOF; bash`).
        "exec" if matches!(stdin, CommandStdin::Inherited) => Role::Argv,
        name if builtin_runs_only_argv(name, &known) => Role::Argv,
        name if is_builtin_or_reserved(name) => Role::Unread,
        _ => Role::Argv,
    }
}

/// [`role`] for a command with words known only at run time (`None`): where
/// its program is one that runs more than its argv, those words may make it
/// run anything.
pub(super) fn runs_more_than_argv(program: &str, args: &[Option<String>]) -> bool {
    if runs_shell(program) {
        return true;
    }
    match program {
        "eval" | "exec" => true,
        name if builtin_runs_only_argv(name, args) => false,
        name => is_builtin_or_reserved(name),
    }
}

/// #2585: a POSIX shell, fish, csh, or PowerShell.
fn runs_shell(program: &str) -> bool {
    policy_runs_shell(program) || policy_runs_powershell(program)
}

/// `NAME=value` for a variable that runs code ([`runs_code_from_variable`]).
fn assigns_code_variable(word: &str) -> bool {
    word.split_once('=')
        .is_some_and(|(name, _)| runs_code_from_variable(name))
}

/// A shell given no `-c` script runs one from a file operand or stdin.
fn shell_role(program: &str, args: &[String], stdin: &CommandStdin) -> Role {
    // A script on the command line is read (or asks) as `-c` scripts are.
    let command_flag = |arg: &String| {
        arg == "--command"
            || arg.starts_with("--command=")
            || arg
                .strip_prefix('-')
                .is_some_and(|letters| !letters.starts_with('-') && letters.contains('c'))
    };
    if policy_runs_powershell(program) {
        // PowerShell's command flags compare in any case and abbreviate.
        let script_flag = args.iter().any(|arg| {
            let arg = arg.to_ascii_lowercase();
            ["-c", "-e", "-f", "/c", "/e", "/f"]
                .iter()
                .any(|flag| arg.starts_with(flag))
        });
        // On Windows a trusted PowerShell reads `-Command` scripts (or asks,
        // #2546); elsewhere none is read.
        return if script_flag && cfg!(windows) {
            Role::Argv
        } else {
            Role::Unread
        };
    }
    if args.iter().any(command_flag) {
        return Role::Argv;
    }
    // `bash`, `bash -s`, `bash -s ARG…`, `bash -`: the script is stdin.
    let mut stdin_script = true;
    let mut operands = false;
    for arg in args {
        match arg.as_str() {
            _ if operands => {}
            "-s" => operands = true,
            "-" | "--" => operands = true,
            _ => {
                stdin_script = false;
                break;
            }
        }
    }
    let posix = shell_name_is_posix(program);
    match stdin {
        CommandStdin::Literal(text) if stdin_script && posix => Role::Script {
            text: text.clone(),
            single: true,
            dialect: script_dialect(program),
        },
        _ => Role::Unread,
    }
}

/// Shells whose stdin script reads as bash's. Not fish or csh, and not the
/// ksh family, whose builtins (`shift`, `ulimit`, `printf %d`) evaluate
/// their arguments as arithmetic.
fn shell_name_is_posix(program: &str) -> bool {
    let name = program
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(program)
        .to_ascii_lowercase();
    let name = name.strip_suffix(".exe").unwrap_or(&name);
    matches!(name, "sh" | "bash" | "zsh" | "dash" | "ash")
}

/// Builtins of bash, zsh and dash that run nothing but their argv whatever
/// it holds.
const ARGV_ONLY_BUILTINS: &[&str] = &[
    // Launchers that found no command of their own (`command -v git`).
    "command", "builtin", "noglob", "-", ":", "true", "false", "echo", "pwd", "cd", "chdir",
    "pushd", "popd", "dirs", "type", "which", "whence", "where", "rehash", "unhash", "times",
    "help", "suspend", "unalias", "log",
    // Jobs, processes and the file mode: no shell evaluates these.
    "kill", "disown", "bg", "fg", "umask",
];

/// Builtins whose arguments are numbers, which zsh and ksh evaluate as
/// arithmetic (`shift 'a[$(…)]'` runs the substitution): read only where
/// every argument is an option or a number.
const NUMERIC_BUILTINS: &[&str] = &[
    "exit", "return", "break", "continue", "shift", "logout", "bye", "caller", "ulimit",
];

/// Every other builtin of bash, zsh (with its code-running module builtins)
/// and dash. Besides those checked in [`builtin_runs_only_argv`], they
/// source, define, load, rename or run code, or evaluate arithmetic, which
/// can.
const OTHER_BUILTINS: &[&str] = &[
    ".",
    "source",
    "eval",
    "exec",
    "jobs",
    "wait",
    "alias",
    "bind",
    "complete",
    "compgen",
    "compopt",
    "declare",
    "typeset",
    "local",
    "export",
    "readonly",
    "enable",
    "disable",
    "fc",
    "r",
    "history",
    "let",
    "mapfile",
    "readarray",
    "trap",
    "printf",
    "print",
    "test",
    "[",
    "read",
    "set",
    "shopt",
    "getopts",
    "unset",
    "hash",
    "limit",
    "unlimit",
    "autoload",
    "bindkey",
    "emulate",
    "echotc",
    "echoti",
    "float",
    "integer",
    "functions",
    "getln",
    "pushln",
    "sched",
    "setopt",
    "unsetopt",
    "ttyctl",
    "unfunction",
    "vared",
    "zcompile",
    "zformat",
    "zle",
    "zmodload",
    "zparseopts",
    "zregexparse",
    "zstyle",
    "zpty",
    "zsocket",
    "ztcp",
    "zselect",
    "zsystem",
    "private",
];

fn is_builtin_or_reserved(name: &str) -> bool {
    ARGV_ONLY_BUILTINS.contains(&name)
        || NUMERIC_BUILTINS.contains(&name)
        || OTHER_BUILTINS.contains(&name)
        || RESERVED_WORDS.contains(&name)
}

/// A number as a builtin reads one: digits (a sign, a fraction for
/// timeouts), a job (`%1`), or `$?`-like specials.
fn is_number(word: &str) -> bool {
    let digits = word.strip_prefix(['-', '+', '%']).unwrap_or(word);
    (!digits.is_empty()
        && digits
            .bytes()
            .all(|byte| byte.is_ascii_digit() || byte == b'.'))
        || numeric_operand(word)
}

/// Whether builtin `name` with `args` (`None` where known only at run time)
/// runs nothing but its argv.
fn builtin_runs_only_argv(name: &str, args: &[Option<String>]) -> bool {
    let known = || args.iter().all(Option::is_some);
    let words = || args.iter().flatten();
    // A variable name with a subscript evaluates it (`read 'a[$(…)]'`), and
    // so does assigning to an integer special (`read OPTIND`).
    let plain_names =
        || known() && words().all(|arg| !arg.contains('[') && !runs_code_from_variable(arg));
    match name {
        _ if ARGV_ONLY_BUILTINS.contains(&name) => true,
        // Options are letters; everything else a number.
        _ if NUMERIC_BUILTINS.contains(&name) => {
            known()
                && words().all(|arg| {
                    is_number(arg)
                        || (arg.starts_with('-')
                            && arg[1..].bytes().all(|byte| byte.is_ascii_alphabetic()))
                        || (name == "ulimit" && arg == "unlimited")
                })
        }
        // `jobs -x` runs a command.
        "jobs" => known() && words().all(|arg| !(arg.starts_with('-') && arg.contains('x'))),
        // `wait -p NAME` assigns, so a word known only at run time may.
        "wait" => plain_names(),
        // `-v NAME` assigns; options end at the format. A numeric
        // conversion (`%d`, `%*s`) reads its argument as arithmetic in zsh
        // and ksh.
        "printf" | "print" => {
            let Some(Some(format)) = args.first() else {
                return args.is_empty();
            };
            let slots = numeric_slots(format);
            // `%2$d` picks its argument by number: then any may be numeric.
            let positional = format.contains('$') && slots.contains(&true);
            !format.starts_with('-')
                && args[1..].iter().enumerate().all(|(index, arg)| {
                    slots.is_empty()
                        || !(positional || slots[index % slots.len()])
                        || arg.as_deref().is_some_and(is_number)
                })
        }
        "test" | "[" => {
            known()
                && args.iter().enumerate().all(|(index, arg)| {
                    let arg = arg.as_deref().unwrap_or_default();
                    if matches!(arg, "-v" | "-R") {
                        return false;
                    }
                    // zsh's `-t FD` evaluates FD as arithmetic.
                    if arg == "-t" {
                        return args
                            .get(index + 1)
                            .is_none_or(|fd| fd.as_deref().is_some_and(numeric_operand));
                    }
                    // zsh's `test` reads these operands as arithmetic.
                    !ARITHMETIC_TESTS.contains(&arg)
                        || [index.wrapping_sub(1), index + 1].iter().all(|&side| {
                            args.get(side)
                                .and_then(Option::as_deref)
                                .is_some_and(numeric_operand)
                        })
                })
        }
        "read" => plain_names() && read_option_values_are_numbers(words()),
        "getopts" | "unset" => plain_names(),
        // No arguments: forget every remembered path.
        "hash" => words().all(|arg| arg == "-r") && known(),
        "set" => known() && set_runs_only_argv(words()),
        "shopt" => {
            known()
                && words().all(|arg| {
                    matches!(
                        arg.as_str(),
                        "-s" | "-u"
                            | "-q"
                            | "-p"
                            | "nullglob"
                            | "globstar"
                            | "extglob"
                            | "dotglob"
                            | "failglob"
                            | "nocaseglob"
                            | "nocasematch"
                            | "lastpipe"
                            | "inherit_errexit"
                            | "globasciiranges"
                    )
                })
        }
        _ => false,
    }
}

/// Per argument a `printf` format reads (it repeats for more arguments):
/// whether it may read a number, which zsh and ksh evaluate as arithmetic.
/// Only `%s`, `%b`, `%c` and `%q` read text; any other conversion, length
/// modifier (`%ld`) or `*` width reads a number.
fn numeric_slots(format: &str) -> Vec<bool> {
    let mut slots = Vec::new();
    let mut chars = format.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch != '%' {
            continue;
        }
        if chars.next_if_eq(&'%').is_some() {
            continue;
        }
        while let Some(&spec) = chars.peek() {
            match spec {
                '*' => {
                    slots.push(true);
                    chars.next();
                }
                '-' | '+' | ' ' | '#' | '0'..='9' | '.' | '$' => {
                    chars.next();
                }
                _ => break,
            }
        }
        slots.push(!matches!(chars.next(), Some('s' | 'b' | 'c' | 'q')));
    }
    slots
}

/// `read`'s timeout, count and file descriptor options (`-t`, `-n`, `-N`,
/// `-k`, `-u`) take numbers, which ksh evaluates as arithmetic.
fn read_option_values_are_numbers<'a>(mut words: impl Iterator<Item = &'a String>) -> bool {
    while let Some(word) = words.next() {
        let Some(letters) = word.strip_prefix('-') else {
            continue;
        };
        for (at, letter) in letters.char_indices() {
            let numeric = "tnNku".contains(letter);
            if !(numeric || "adpiAeEr".contains(letter)) {
                return false;
            }
            if numeric || "adpiA".contains(letter) {
                let rest = &letters[at + letter.len_utf8()..];
                let value = if rest.is_empty() {
                    words.next().map(String::as_str)
                } else {
                    Some(rest)
                };
                if numeric && !value.is_some_and(is_number) {
                    return false;
                }
                break;
            }
        }
    }
    true
}

/// `set` with options that run no code: not xtrace (`PS4` is expanded),
/// keyword (assignments anywhere), or functrace.
fn set_runs_only_argv<'a>(args: impl Iterator<Item = &'a String>) -> bool {
    let mut option_name = false;
    let mut positional = false;
    for arg in args {
        if positional {
            continue;
        }
        if option_name {
            option_name = false;
            if !matches!(
                arg.as_str(),
                "errexit"
                    | "nounset"
                    | "pipefail"
                    | "noglob"
                    | "noclobber"
                    | "allexport"
                    | "errtrace"
                    | "noexec"
                    | "notify"
                    | "hashall"
                    | "braceexpand"
                    | "physical"
                    | "monitor"
            ) {
                return false;
            }
            continue;
        }
        if arg == "--" || arg == "-" {
            positional = true;
            continue;
        }
        let Some(letters) = arg.strip_prefix(['-', '+']) else {
            // `set a b` sets the positional parameters.
            positional = true;
            continue;
        };
        if !letters
            .chars()
            .all(|letter| "abefhmnuBCEHPo".contains(letter))
        {
            return false;
        }
        option_name = letters.contains('o');
    }
    true
}
