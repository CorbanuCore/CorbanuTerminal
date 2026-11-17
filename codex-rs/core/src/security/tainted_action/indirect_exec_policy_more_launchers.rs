//! #2455 follow-up (D670): more launchers that exec their trailing argv —
//! the shell builtins `command` and `exec`, `time`, `caffeinate` and
//! `systemd-run` (getopt tables read by the parent), and `pkexec`, `bwrap`,
//! `firejail`, `busybox` and `toybox`, and zsh's precommand modifiers,
//! whose own parsers are below.

use super::super::basename;
use super::Arity::Flag;
use super::Arity::Value;
use super::HELP;
use super::Launch;
use super::Opt;
use super::VERSION;
use super::work;

/// bash, zsh and dash: `command [-pvV] [--] name [arg ...]`. `-v`/`-V`
/// only print what `name` is.
pub(super) const COMMAND: &[Opt] = &[('p', "", Flag), ('v', "", Flag), ('V', "", Flag)];

/// bash and zsh: `exec [-cl] [-a name] [command [arg ...]]`.
pub(super) const EXEC: &[Opt] = &[('c', "", Flag), ('l', "", Flag), ('a', "", Value)];

/// GNU time (`+af:o:pqvV`) and BSD time (`-a -h -l -o file -p`): no letter
/// takes a value in one and not the other.
pub(super) const TIME: &[Opt] = &[
    ('\0', "help", Flag),
    VERSION,
    ('a', "append", Flag),
    ('f', "format", Value),
    ('o', "output", Value),
    ('p', "portability", Flag),
    ('q', "quiet", Flag),
    ('v', "verbose", Flag),
    ('h', "", Flag),
    ('l', "", Flag),
];

/// macOS: `caffeinate [-disu] [-t timeout] [-w pid] [utility arg ...]`.
pub(super) const CAFFEINATE: &[Opt] = &[
    ('d', "", Flag),
    ('i', "", Flag),
    ('m', "", Flag),
    ('s', "", Flag),
    ('u', "", Flag),
    ('t', "", Value),
    ('w', "", Value),
];

/// systemd 255 (checked option by option on glitch). Newer options are
/// unknown, so uncertain.
pub(super) const SYSTEMD_RUN: &[Opt] = &[
    HELP,
    ('\0', "version", Flag),
    ('\0', "no-ask-password", Flag),
    ('\0', "user", Flag),
    ('\0', "system", Flag),
    ('H', "host", Value),
    ('M', "machine", Value),
    ('\0', "scope", Flag),
    ('u', "unit", Value),
    ('p', "property", Value),
    ('\0', "description", Value),
    ('\0', "slice", Value),
    ('\0', "slice-inherit", Flag),
    ('\0', "expand-environment", Value),
    ('\0', "no-block", Flag),
    ('r', "remain-after-exit", Flag),
    ('\0', "wait", Flag),
    ('\0', "send-sighup", Flag),
    ('\0', "service-type", Value),
    ('\0', "uid", Value),
    ('\0', "gid", Value),
    ('\0', "nice", Value),
    ('\0', "working-directory", Value),
    ('d', "same-dir", Flag),
    ('E', "setenv", Value),
    ('t', "pty", Flag),
    ('P', "pipe", Flag),
    ('q', "quiet", Flag),
    ('G', "collect", Flag),
    ('S', "shell", Flag),
    ('\0', "path-property", Value),
    ('\0', "socket-property", Value),
    ('\0', "on-active", Value),
    ('\0', "on-boot", Value),
    ('\0', "on-startup", Value),
    ('\0', "on-unit-active", Value),
    ('\0', "on-unit-inactive", Value),
    ('\0', "on-calendar", Value),
    ('\0', "on-timezone-change", Flag),
    ('\0', "on-clock-change", Flag),
    ('\0', "timer-property", Value),
];

/// Read after systemd-run's options (`options`, its option words).
/// `None`: uncertain; else whether to keep an approval floor. `floor`: a
/// long name was abbreviated (that resolves differently as systemd adds
/// options), or some word has a `$` (services expand `${NAME}` in their
/// command line, `--expand-environment`).
pub(super) fn systemd_run(options: &[String], shell: bool, floor: bool) -> Option<bool> {
    // `--shell` runs `$SHELL`; an `Exec*=` property (`-p ExecStartPre=…`)
    // runs a command line of its own. Properties that change what the
    // command's name resolves to (`RootDirectory=`, `BindPaths=`) or start
    // other units (`OnFailure=`, `Unit=`) are like chroot, `bwrap --bind`
    // or `systemctl start`: the command itself is still the one matched.
    if shell
        || options.iter().any(|word| {
            work::step();
            word.contains("Exec")
        })
    {
        return None;
    }
    Some(floor)
}

/// Launchers with their own parsers. zsh's precommand modifiers (`-`,
/// `noglob`, `nocorrect`) and `builtin` take no options.
const OWN_PARSERS: &[&str] = &[
    "pkexec",
    "bwrap",
    "firejail",
    "-",
    "noglob",
    "nocorrect",
    "builtin",
];

pub(super) fn is_launcher(name: &str) -> bool {
    OWN_PARSERS.contains(&name) || multicall_name(name).is_some()
}

pub(super) fn launch(name: &str, words: &[String], start: usize) -> Option<Launch> {
    if let Some(multicall_name) = multicall_name(name) {
        return Some(multicall(multicall_name, words, start));
    }
    Some(match name {
        "pkexec" => pkexec(words, start),
        "bwrap" => bwrap(words, start),
        "firejail" => firejail(words, start),
        "-" | "noglob" | "nocorrect" | "builtin" => modifier(words, start),
        _ => return None,
    })
}

/// busybox runs its own main for any name starting `busybox`
/// (`busybox.static`), toybox for any starting `toybox`.
fn multicall_name(name: &str) -> Option<&'static str> {
    if name.starts_with("busybox") {
        Some("busybox")
    } else if name.starts_with("toybox") {
        Some("toybox")
    } else {
        None
    }
}

/// `noglob cmd`, `builtin exec cmd`: the next word is the command (bash's
/// `builtin` also accepts `--`).
fn modifier(words: &[String], start: usize) -> Launch {
    let mut at = start + 1;
    if words[start] == "builtin" && words.get(at).is_some_and(|word| word == "--") {
        at += 1;
    }
    match words.get(at) {
        None => Launch::NoCommand,
        Some(word) if word.starts_with('-') && word != "-" => Launch::Uncertain,
        Some(_) => Launch::Command {
            at,
            uncertain: false,
        },
    }
}

/// polkit's pkexec compares whole words (no `=`, `--` or clusters); the
/// first other word is the program. Help or version only print. Without a
/// program it runs a shell.
fn pkexec(words: &[String], start: usize) -> Launch {
    let mut at = start + 1;
    let mut informational = false;
    while let Some(word) = words.get(at) {
        work::step();
        match word.as_str() {
            "--help" | "--version" => informational = true,
            "--disable-internal-agent" | "--keep-cwd" => {}
            "--user" | "-u" => at += 1,
            _ if word.starts_with('-') => return Launch::Uncertain,
            _ => break,
        }
        at += 1;
    }
    if informational {
        Launch::NoCommand
    } else if at < words.len() {
        Launch::Command {
            at,
            uncertain: false,
        }
    } else {
        Launch::Uncertain
    }
}

/// bubblewrap 0.9-0.11: whole-word options with a fixed number of values.
/// `--args FD` reads more options from a descriptor, so it is uncertain.
const BWRAP: &[(&str, usize)] = &[
    ("--unshare-all", 0),
    ("--share-net", 0),
    ("--unshare-user", 0),
    ("--unshare-user-try", 0),
    ("--unshare-ipc", 0),
    ("--unshare-pid", 0),
    ("--unshare-net", 0),
    ("--unshare-uts", 0),
    ("--unshare-cgroup", 0),
    ("--unshare-cgroup-try", 0),
    ("--disable-userns", 0),
    ("--assert-userns-disabled", 0),
    ("--clearenv", 0),
    ("--new-session", 0),
    ("--die-with-parent", 0),
    ("--as-pid-1", 0),
    ("--argv0", 1),
    ("--userns", 1),
    ("--userns2", 1),
    ("--pidns", 1),
    ("--uid", 1),
    ("--gid", 1),
    ("--hostname", 1),
    ("--chdir", 1),
    ("--unsetenv", 1),
    ("--lock-file", 1),
    ("--sync-fd", 1),
    ("--remount-ro", 1),
    ("--exec-label", 1),
    ("--file-label", 1),
    ("--proc", 1),
    ("--dev", 1),
    ("--tmpfs", 1),
    ("--mqueue", 1),
    ("--dir", 1),
    ("--seccomp", 1),
    ("--add-seccomp-fd", 1),
    ("--block-fd", 1),
    ("--userns-block-fd", 1),
    ("--info-fd", 1),
    ("--json-status-fd", 1),
    ("--cap-add", 1),
    ("--cap-drop", 1),
    ("--perms", 1),
    ("--size", 1),
    ("--overlay-src", 1),
    ("--tmp-overlay", 1),
    ("--ro-overlay", 1),
    ("--setenv", 2),
    ("--bind", 2),
    ("--bind-try", 2),
    ("--dev-bind", 2),
    ("--dev-bind-try", 2),
    ("--ro-bind", 2),
    ("--ro-bind-try", 2),
    ("--bind-fd", 2),
    ("--ro-bind-fd", 2),
    ("--file", 2),
    ("--bind-data", 2),
    ("--ro-bind-data", 2),
    ("--symlink", 2),
    ("--chmod", 2),
    ("--overlay", 3),
];

fn bwrap(words: &[String], start: usize) -> Launch {
    let mut at = start + 1;
    while let Some(word) = words.get(at) {
        work::step();
        if word == "--" {
            at += 1;
            break;
        }
        if !word.starts_with('-') {
            break;
        }
        // These print and exit as soon as they are read.
        if word == "--help" || word == "--version" {
            return Launch::NoCommand;
        }
        let Some((_, values)) = BWRAP.iter().find(|(option, _)| {
            work::step();
            option == word
        }) else {
            return Launch::Uncertain;
        };
        at += 1 + values;
    }
    if at < words.len() {
        Launch::Command {
            at,
            uncertain: false,
        }
    } else {
        Launch::Uncertain
    }
}

/// firejail's options are `--name` or `--name=value`, ending at the first
/// other word or `--`. Firejail before 0.9.64 ran the program through a
/// shell by default (`--shell`), so the command keeps an approval floor.
fn firejail(words: &[String], start: usize) -> Launch {
    let mut at = start + 1;
    while let Some(word) = words.get(at) {
        work::step();
        if word == "--" {
            at += 1;
            break;
        }
        let Some(option) = word.strip_prefix("--") else {
            if word.starts_with('-') {
                return Launch::Uncertain; // `-c`: login-shell mode.
            }
            break;
        };
        let name = option.split_once('=').map_or(option, |(name, _)| name);
        // File operations on a sandbox take path operands, not a command.
        if matches!(name, "ls" | "get" | "put" | "cat") {
            return Launch::Uncertain;
        }
        at += 1;
    }
    if at < words.len() {
        Launch::Command {
            at,
            uncertain: true,
        }
    } else {
        Launch::Uncertain
    }
}

/// `busybox APPLET [arg ...]` runs the applet named by the word's base
/// name; so does toybox. Alone or with a listing option they only print.
fn multicall(name: &str, words: &[String], start: usize) -> Launch {
    let Some(word) = words.get(start + 1) else {
        return Launch::NoCommand;
    };
    if !word.starts_with('-') {
        // `busybox ./rm` runs the `rm` applet, which a rule for `./rm` or
        // a resolved path need not match: keep a floor. A further
        // `busybox…` name is read as a launcher in turn.
        return Launch::Command {
            at: start + 1,
            uncertain: word.contains('/') && multicall_name(basename(word)).is_none(),
        };
    }
    let listing: &[&str] = if name == "busybox" {
        &["--help", "--list", "--list-full"]
    } else {
        &["--help", "--version", "--long"]
    };
    if listing.contains(&word.as_str()) {
        Launch::NoCommand
    } else {
        Launch::Uncertain
    }
}
