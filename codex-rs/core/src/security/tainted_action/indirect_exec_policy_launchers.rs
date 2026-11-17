//! #2455 (D670): launchers that exec their trailing argv, for exec-policy
//! rules. Each table follows the program's own option parser (getopt with
//! a leading `+`, so options end at the first operand). An unknown or
//! ambiguous option, a missing value, or a form that runs a shell string
//! instead of argv is uncertain: the caller keeps an approval floor rather
//! than matching the launcher only.

use super::work;

#[path = "indirect_exec_policy_more_launchers.rs"]
mod indirect_exec_policy_more_launchers;
use indirect_exec_policy_more_launchers as more;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Arity {
    Flag,
    /// A required value: the rest of the word, or the next word.
    Value,
    /// An optional value: only `--name=value` or the rest of a short word.
    Optional,
    /// A flag in short form; an optional `--name=value` in long form
    /// (unshare's `-m` but `--mount[=file]`).
    LongOptional,
}

use Arity::Flag;
use Arity::LongOptional;
use Arity::Optional;
use Arity::Value;

/// One option: its short letter (`'\0'` for none), long name (`""` for
/// none) and arity.
type Opt = (char, &'static str, Arity);

const HELP: Opt = ('h', "help", Flag);
const VERSION: Opt = ('V', "version", Flag);

const UNSHARE: &[Opt] = &[
    HELP,
    VERSION,
    ('m', "mount", LongOptional),
    ('u', "uts", LongOptional),
    ('i', "ipc", LongOptional),
    ('n', "net", LongOptional),
    ('p', "pid", LongOptional),
    ('U', "user", LongOptional),
    ('C', "cgroup", LongOptional),
    ('T', "time", LongOptional),
    ('f', "fork", Flag),
    ('\0', "kill-child", Optional),
    ('\0', "mount-proc", Optional),
    ('\0', "mount-binfmt", Optional),
    ('\0', "map-user", Value),
    ('\0', "map-users", Value),
    ('\0', "map-group", Value),
    ('\0', "map-groups", Value),
    ('r', "map-root-user", Flag),
    ('c', "map-current-user", Flag),
    ('\0', "map-auto", Flag),
    ('\0', "propagation", Value),
    ('\0', "setgroups", Value),
    ('\0', "keep-caps", Flag),
    ('S', "setuid", Value),
    ('G', "setgid", Value),
    ('R', "root", Value),
    ('w', "wd", Value),
    ('\0', "monotonic", Value),
    ('\0', "boottime", Value),
    ('l', "load-interp", Value),
];

/// Only options common to current util-linux releases; newer ones
/// (`--wdns`, `--env`, `--join-cgroup`, ...) are uncertain.
const NSENTER: &[Opt] = &[
    HELP,
    VERSION,
    ('a', "all", Flag),
    ('t', "target", Value),
    ('m', "mount", Optional),
    ('u', "uts", Optional),
    ('i', "ipc", Optional),
    ('n', "net", Optional),
    ('p', "pid", Optional),
    ('C', "cgroup", Optional),
    ('U', "user", Optional),
    ('T', "time", Optional),
    ('S', "setuid", Value),
    ('G', "setgid", Value),
    ('\0', "preserve-credentials", Flag),
    ('r', "root", Optional),
    ('w', "wd", Optional),
    ('F', "no-fork", Flag),
    ('Z', "follow-context", Flag),
];

const SETPRIV: &[Opt] = &[
    HELP,
    VERSION,
    ('d', "dump", Flag),
    ('\0', "nnp", Flag),
    ('\0', "no-new-privs", Flag),
    ('\0', "inh-caps", Value),
    ('\0', "ambient-caps", Value),
    ('\0', "list-caps", Flag),
    ('\0', "ruid", Value),
    ('\0', "euid", Value),
    ('\0', "rgid", Value),
    ('\0', "egid", Value),
    ('\0', "reuid", Value),
    ('\0', "regid", Value),
    ('\0', "clear-groups", Flag),
    ('\0', "keep-groups", Flag),
    ('\0', "init-groups", Flag),
    ('\0', "groups", Value),
    ('\0', "bounding-set", Value),
    ('\0', "securebits", Value),
    ('\0', "pdeathsig", Value),
    ('\0', "selinux-label", Value),
    ('\0', "apparmor-profile", Value),
    ('\0', "landlock-access", Value),
    ('\0', "landlock-rule", Value),
    ('\0', "seccomp-filter", Value),
    ('\0', "reset-env", Flag),
];

const TIMEOUT: &[Opt] = &[
    ('\0', "help", Flag),
    ('\0', "version", Flag),
    ('f', "foreground", Flag),
    ('k', "kill-after", Value),
    ('p', "preserve-status", Flag),
    ('s', "signal", Value),
    ('v', "verbose", Flag),
];

const NICE: &[Opt] = &[
    ('\0', "help", Flag),
    ('\0', "version", Flag),
    ('n', "adjustment", Value),
];

const NOHUP: &[Opt] = &[('\0', "help", Flag), ('\0', "version", Flag)];

const STDBUF: &[Opt] = &[
    ('\0', "help", Flag),
    ('\0', "version", Flag),
    ('i', "input", Value),
    ('o', "output", Value),
    ('e', "error", Value),
];

const SETSID: &[Opt] = &[
    HELP,
    VERSION,
    ('c', "ctty", Flag),
    ('f', "fork", Flag),
    ('w', "wait", Flag),
];

const IONICE: &[Opt] = &[
    HELP,
    VERSION,
    ('n', "classdata", Value),
    ('c', "class", Value),
    ('p', "pid", Value),
    ('P', "pgid", Value),
    ('u', "uid", Value),
    ('t', "ignore", Flag),
];

const CHRT: &[Opt] = &[
    HELP,
    VERSION,
    ('a', "all-tasks", Flag),
    ('b', "batch", Flag),
    ('d', "deadline", Flag),
    ('f', "fifo", Flag),
    ('i', "idle", Flag),
    ('p', "pid", Flag),
    ('m', "max", Flag),
    ('o', "other", Flag),
    ('r', "rr", Flag),
    ('T', "sched-runtime", Value),
    ('P', "sched-period", Value),
    ('D', "sched-deadline", Value),
    ('R', "reset-on-fork", Flag),
    ('v', "verbose", Flag),
];

const TASKSET: &[Opt] = &[
    HELP,
    VERSION,
    ('a', "all-tasks", Flag),
    ('p', "pid", Flag),
    ('c', "cpu-list", Flag),
];

const FLOCK: &[Opt] = &[
    HELP,
    VERSION,
    ('?', "", Flag),
    ('s', "shared", Flag),
    ('x', "exclusive", Flag),
    ('e', "", Flag),
    ('u', "unlock", Flag),
    ('n', "nonblocking", Flag),
    ('\0', "nb", Flag),
    ('w', "timeout", Value),
    ('\0', "wait", Value),
    ('E', "conflict-exit-code", Value),
    ('o', "close", Flag),
    ('F', "no-fork", Flag),
    ('\0', "verbose", Flag),
    ('\0', "fcntl", Flag),
];

/// OpenBSD `doas` and OpenDoas: short options only.
const DOAS: &[Opt] = &[
    ('C', "", Value),
    ('L', "", Flag),
    ('n', "", Flag),
    ('s', "", Flag),
    ('u', "", Value),
];

const RUNUSER: &[Opt] = &[
    HELP,
    VERSION,
    ('c', "command", Value),
    ('\0', "session-command", Value),
    ('f', "fast", Flag),
    ('g', "group", Value),
    ('G', "supp-group", Value),
    ('l', "login", Flag),
    ('m', "", Flag),
    ('p', "preserve-environment", Flag),
    ('P', "pty", Flag),
    ('s', "shell", Value),
    ('u', "user", Value),
    ('w', "whitelist-environment", Value),
];

const LAUNCHERS: &[(&str, &[Opt])] = &[
    ("unshare", UNSHARE),
    ("nsenter", NSENTER),
    ("setpriv", SETPRIV),
    ("timeout", TIMEOUT),
    ("nice", NICE),
    ("nohup", NOHUP),
    ("stdbuf", STDBUF),
    ("setsid", SETSID),
    ("ionice", IONICE),
    ("chrt", CHRT),
    ("taskset", TASKSET),
    ("flock", FLOCK),
    ("doas", DOAS),
    ("runuser", RUNUSER),
    ("command", more::COMMAND),
    ("exec", more::EXEC),
    ("time", more::TIME),
    ("caffeinate", more::CAFFEINATE),
    ("systemd-run", more::SYSTEMD_RUN),
];

/// `start`: where `name` is in the command (zsh's `-` modifier only
/// begins one; after a program, `-` is a program named `-`).
pub(super) fn is_launcher(name: &str, start: usize) -> bool {
    LAUNCHERS.iter().any(|(launcher, _)| {
        work::step();
        *launcher == name
    }) || more::is_launcher(name, start)
}

/// Where the command a launcher runs starts.
pub(super) enum Launch {
    /// The trailing argv from this index. `uncertain`: the launcher may
    /// read its words differently (option permutation), so an approval
    /// floor stays as well.
    Command { at: usize, uncertain: bool },
    /// It runs no command (help, a query, or a change to other processes).
    NoCommand,
    /// Its words cannot be read with confidence, or it runs a shell.
    Uncertain,
}

/// What [`launch`] needs from the whole command: one scan per command,
/// however many launchers it chains.
#[derive(Clone, Copy)]
pub(super) struct Scan {
    /// The last word that looks like an option (the permutation check).
    last_option: Option<usize>,
    /// Some word has a `$` (systemd-run may expand it).
    dollar: bool,
}

pub(super) fn scan(words: &[String]) -> Scan {
    let mut scan = Scan {
        last_option: None,
        dollar: false,
    };
    for (at, word) in words.iter().enumerate() {
        work::step();
        if word.starts_with('-') && word != "-" {
            scan.last_option = Some(at);
        }
        scan.dollar |= word.contains('$');
    }
    scan
}

/// `words[start]` names a launcher in [`LAUNCHERS`] or
/// [`more::is_launcher`]; `scan` is [`scan`]`(words)`.
pub(super) fn launch(name: &str, words: &[String], start: usize, scan: Scan) -> Launch {
    let last_option = scan.last_option;
    if let Some(launch) = more::launch(name, words, start) {
        return launch;
    }
    let Some((_, options)) = LAUNCHERS.iter().find(|(launcher, _)| {
        work::step();
        *launcher == name
    }) else {
        return Launch::Uncertain;
    };
    let mut seen: Vec<Opt> = Vec::new();
    let mut abbreviated = false;
    let mut at = start + 1;
    let mut ended = false;
    while let Some(word) = words.get(at) {
        work::step();
        if word == "--" {
            ended = true;
            at += 1;
            break;
        }
        if !word.starts_with('-') || word == "-" {
            break;
        }
        // GNU nice's `-N`, `--N` and `-+N` adjustments.
        if name == "nice" && nice_adjustment(word) {
            at += 1;
            continue;
        }
        let Some(words_used) = option(options, word, &mut seen, &mut abbreviated) else {
            return Launch::Uncertain;
        };
        at += words_used;
        if at > words.len() {
            return Launch::Uncertain; // A value is missing.
        }
    }
    let has = |long: &str, short: char| {
        seen.iter().any(|(letter, name, _)| {
            work::step();
            (!long.is_empty() && *name == long) || (short != '\0' && *letter == short)
        })
    };
    let informational = has("help", 'h') || has("version", 'V');
    let mut uncertain = false;
    match name {
        // Operating on other processes: the operands are ids or a mask.
        "ionice" if has("pid", 'p') || has("pgid", 'P') || has("uid", 'u') => {
            return Launch::NoCommand;
        }
        "chrt" if has("pid", 'p') || has("max", 'm') => return Launch::NoCommand,
        "command" if has("", 'v') || has("", 'V') => return Launch::NoCommand,
        // zsh's `-` modifier, which `exec` passes on (`exec - cmd`). zsh's
        // `command -` fails; bash's may not, so keep a floor there.
        "exec" | "command" if words.get(at).is_some_and(|word| word == "-") => {
            uncertain = name == "command";
            at += 1;
        }
        "systemd-run" => {
            // Its option words and values (and any `--`).
            match more::systemd_run(
                &words[start + 1..at],
                has("shell", 'S'),
                abbreviated || scan.dollar,
            ) {
                Some(floor) => uncertain = floor,
                None => return Launch::Uncertain,
            }
        }
        "taskset" if has("pid", 'p') => return Launch::NoCommand,
        "timeout" | "taskset" => at += 1, // The duration or the CPU mask.
        // A priority, or (util-linux 2.42+, for policies without one) the
        // command itself. Before 2.42 any integer `strtoimax` reads whole
        // is the priority; since, only plain digits are: for `+0`, `-0` or
        // ` 0` the version decides, so keep an approval floor as well.
        "chrt" if words.get(at).is_some_and(|word| is_integer(word)) => {
            uncertain = !words[at].bytes().all(|byte| {
                work::step();
                byte.is_ascii_digit()
            });
            at += 1;
        }
        "flock" => {
            // A lock file or descriptor, then argv or `-c` with a script.
            at += 1;
            if words
                .get(at)
                .is_some_and(|word| word == "-c" || word == "--command")
            {
                return Launch::Uncertain;
            }
            if at >= words.len() {
                return Launch::NoCommand;
            }
        }
        "runuser" => {
            // Only `-u USER` runs argv; otherwise it is su-compatible and
            // runs a shell or a `-c` string.
            if !has("user", 'u')
                || has("command", 'c')
                || has("session-command", '\0')
                || has("shell", 's')
                || has("login", 'l')
                || has("fast", 'f')
            {
                return Launch::Uncertain;
            }
            uncertain = !ended && permuted(last_option, at);
        }
        "doas" => {
            if has("", 'L') && at >= words.len() {
                return Launch::NoCommand;
            }
            uncertain = !ended && permuted(last_option, at);
        }
        _ => {}
    }
    if at < words.len() {
        return Launch::Command { at, uncertain };
    }
    // No command: some only print or query; the rest run a shell, or fail.
    let quiet = informational
        || matches!(
            name,
            "nice" | "ionice" | "command" | "exec" | "time" | "caffeinate"
        )
        || (name == "setpriv" && (has("dump", 'd') || has("list-caps", '\0')));
    if quiet {
        Launch::NoCommand
    } else {
        Launch::Uncertain
    }
}

/// The words option `word` uses (1, or 2 when its value is the next word),
/// recording what it set; `None` when it is unknown or ambiguous.
/// `abbreviated`: set when a long name was a unique abbreviation.
fn option(
    options: &[Opt],
    word: &str,
    seen: &mut Vec<Opt>,
    abbreviated: &mut bool,
) -> Option<usize> {
    if let Some(long) = word.strip_prefix("--") {
        let (name, attached) = long
            .split_once('=')
            .map_or((long, false), |(name, _)| (name, true));
        let found = long_option(options, name)?;
        *abbreviated |= found.1 != name;
        seen.push(found);
        return match (found.2, attached) {
            (Flag, true) => None,
            (Value, false) => Some(2),
            (Value | Optional | LongOptional, _) | (Flag, false) => Some(1),
        };
    }
    let letters = word.strip_prefix('-')?;
    for (index, letter) in letters.char_indices() {
        work::step();
        let found = *options.iter().find(|(short, _, _)| {
            work::step();
            *short != '\0' && *short == letter
        })?;
        seen.push(found);
        match found.2 {
            Flag | LongOptional => {}
            // The rest of the word is its value.
            Optional => return Some(1),
            Value => return Some(1 + usize::from(index + letter.len_utf8() == letters.len())),
        }
    }
    Some(1)
}

/// getopt_long: an exact name, else a unique abbreviation.
fn long_option(options: &[Opt], name: &str) -> Option<Opt> {
    if name.is_empty() {
        return None;
    }
    let mut unique = None;
    for option in options {
        work::step();
        if option.1.is_empty() {
            continue;
        }
        if option.1 == name {
            return Some(*option);
        }
        if option.1.starts_with(name) {
            if unique.is_some() {
                return None;
            }
            unique = Some(*option);
        }
    }
    unique
}

fn nice_adjustment(word: &str) -> bool {
    let rest = word.strip_prefix('-').unwrap_or(word);
    let rest = rest
        .strip_prefix('-')
        .or_else(|| rest.strip_prefix('+'))
        .unwrap_or(rest);
    rest.starts_with(|c: char| c.is_ascii_digit())
}

/// What `strtoimax` reads whole: leading white space, a sign, digits.
fn is_integer(word: &str) -> bool {
    let unsigned = word.trim_start_matches([' ', '\t', '\n', '\u{b}', '\u{c}', '\r']);
    let digits = unsigned.strip_prefix(['+', '-']).unwrap_or(unsigned);
    !digits.is_empty()
        && digits.bytes().all(|byte| {
            work::step();
            byte.is_ascii_digit()
        })
}

/// Whether GNU getopt, if the launcher permutes, could read a word after
/// the command name `at` as its own option and so change the command.
fn permuted(last_option: Option<usize>, at: usize) -> bool {
    last_option.is_some_and(|last| last > at)
}

#[cfg(test)]
mod tests {
    use super::super::policy_wrapper_commands;
    use super::work;

    /// Chained launchers, including those with a permutation check, cost
    /// linear work.
    #[test]
    fn launcher_chains_are_linear() {
        for launcher in [
            "doas",
            "runuser -u x",
            "unshare -r",
            "timeout 5",
            "nice -n 1",
            "command -p",
            "exec -a x",
            "time -p",
            "caffeinate -i",
            "systemd-run --scope -p A=1",
            "pkexec --user x",
            "bwrap --bind / /",
            "firejail --quiet",
            "busybox",
        ] {
            work::assert_linear(launcher, |scale| {
                let layers = scale * 64;
                let mut words = Vec::new();
                for _ in 0..layers {
                    words.extend(launcher.split(' ').map(str::to_string));
                }
                words.extend(["cmd".to_string(), "-x".to_string()]);
                assert!(policy_wrapper_commands(&words).starts.len() > layers);
            });
        }
    }
}
