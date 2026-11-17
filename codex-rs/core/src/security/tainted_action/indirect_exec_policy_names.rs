//! #2503 (D688): the name an exec-policy launcher word runs as. Launchers
//! (`sudo`, `env`, `xargs`, `su`, ...) are recognized by this name only, so
//! every launcher, present and future, matches names the same way.
//!
//! macOS (case-insensitive APFS, the default) and Windows (NTFS) look
//! program names up without regard to case: there `SUDO`, `/usr/bin/Env`
//! and `ſu` run `sudo`, `env` and `su`. APFS applies full Unicode case
//! folding, which maps a few non-ASCII letters and ligatures to ASCII
//! ([`FOLDS`]; checked on APFS over every code point). Windows also drops an
//! `.exe` suffix and trailing dots and spaces, and separates directories
//! with `\` too. Not read: characters legacy HFS+ volumes ignore (such as
//! U+200D), Windows 8.3 short names (`BUSYBO~1.EXE`) and `::$DATA` stream
//! suffixes (#2510).
//!
//! Linux file systems are case-sensitive (ext4/f2fs case-folded directories
//! are opt-in and not used for program directories), so names there stay as
//! written: `SUDO` is a different program. Reading more names as launchers
//! only adds rule checks and approval floors, so folding too much cannot
//! allow anything.

use std::borrow::Cow;

use super::basename;
use super::work;

/// How program names compare on a platform.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum NameCase {
    /// Exact names (Linux and other Unix).
    Sensitive,
    /// Case-folded names (macOS).
    Insensitive,
    /// Case-folded names, `\` separators and Windows file-name trimming.
    Windows,
}

pub(super) const PLATFORM: NameCase = if cfg!(windows) {
    NameCase::Windows
} else if cfg!(target_os = "macos") {
    NameCase::Insensitive
} else {
    NameCase::Sensitive
};

/// Non-ASCII characters whose full Unicode case folding is ASCII: the long
/// s, the Kelvin sign, sharp s and the Latin ligatures. Any other non-ASCII
/// character keeps a name from equalling an ASCII launcher name.
const FOLDS: &[(char, &str)] = &[
    ('\u{17f}', "s"),
    ('\u{212a}', "k"),
    ('\u{df}', "ss"),
    ('\u{1e9e}', "ss"),
    ('\u{fb00}', "ff"),
    ('\u{fb01}', "fi"),
    ('\u{fb02}', "fl"),
    ('\u{fb03}', "ffi"),
    ('\u{fb04}', "ffl"),
    ('\u{fb05}', "st"),
    ('\u{fb06}', "st"),
];

/// The launcher name `word` runs as where names compare as `case` says
/// ([`PLATFORM`] for this platform).
pub(super) fn launcher_name_for(word: &str, case: NameCase) -> Cow<'_, str> {
    let name = match case {
        NameCase::Sensitive => return Cow::Borrowed(basename(word)),
        NameCase::Insensitive => basename(word),
        NameCase::Windows => windows_file_name(word),
    };
    let folded = fold(name);
    if case != NameCase::Windows {
        return folded;
    }
    // `sudo.exe`, `SUDO.EXE.`: CreateProcess adds `.exe`, and Win32 path
    // normalization drops trailing dots and spaces.
    let trimmed = trim_windows(&folded);
    let trimmed = trim_windows(trimmed.strip_suffix(".exe").unwrap_or(trimmed));
    if trimmed.len() == folded.len() {
        folded
    } else {
        Cow::Owned(trimmed.to_string())
    }
}

/// The last path component, with `/` or `\` separators and an optional
/// drive (`C:sudo`).
fn windows_file_name(word: &str) -> &str {
    let name = word
        .trim_end_matches(['/', '\\'])
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(word);
    match name.as_bytes() {
        [drive, b':', ..] if drive.is_ascii_alphabetic() => &name[2..],
        _ => name,
    }
}

fn trim_windows(name: &str) -> &str {
    name.trim_end_matches(['.', ' '])
}

/// `name` with ASCII letters lowercased and [`FOLDS`] applied.
fn fold(name: &str) -> Cow<'_, str> {
    if name.is_ascii() && !name.bytes().any(|byte| byte.is_ascii_uppercase()) {
        return Cow::Borrowed(name);
    }
    let mut folded = String::with_capacity(name.len());
    for c in name.chars() {
        work::step();
        match FOLDS.iter().find(|(from, _)| *from == c) {
            Some((_, to)) => folded.push_str(to),
            None => folded.push(c.to_ascii_lowercase()),
        }
    }
    Cow::Owned(folded)
}

#[cfg(test)]
mod tests {
    use super::NameCase;
    use super::launcher_name_for;
    use pretty_assertions::assert_eq;

    #[test]
    fn names_fold_only_where_the_platform_does() {
        for (word, sensitive, insensitive, windows) in [
            ("sudo", "sudo", "sudo", "sudo"),
            ("SUDO", "SUDO", "sudo", "sudo"),
            ("/usr/bin/Env", "Env", "env", "env"),
            ("XArgs", "XArgs", "xargs", "xargs"),
            ("\u{17f}u", "\u{17f}u", "su", "su"),
            ("P\u{212a}EXEC", "P\u{212a}EXEC", "pkexec", "pkexec"),
            ("\u{fb01}rejail", "\u{fb01}rejail", "firejail", "firejail"),
            (
                "\u{17f}y\u{fb06}emd-run",
                "\u{17f}y\u{fb06}emd-run",
                "systemd-run",
                "systemd-run",
            ),
            ("unsha\u{df}e", "unsha\u{df}e", "unshasse", "unshasse"),
            ("sud\u{f3}", "sud\u{f3}", "sud\u{f3}", "sud\u{f3}"),
            ("sudo.exe", "sudo.exe", "sudo.exe", "sudo"),
            ("SUDO.EXE. ", "SUDO.EXE. ", "sudo.exe. ", "sudo"),
            (
                "C:\\Tools\\Env.Exe",
                "C:\\Tools\\Env.Exe",
                "c:\\tools\\env.exe",
                "env",
            ),
            ("C:xargs", "C:xargs", "c:xargs", "xargs"),
            ("sudo.com", "sudo.com", "sudo.com", "sudo.com"),
            ("-", "-", "-", "-"),
        ] {
            assert_eq!(
                [
                    launcher_name_for(word, NameCase::Sensitive).as_ref(),
                    launcher_name_for(word, NameCase::Insensitive).as_ref(),
                    launcher_name_for(word, NameCase::Windows).as_ref(),
                ],
                [sensitive, insensitive, windows],
                "{word}"
            );
        }
    }

    /// Launchers with options, then a command.
    const LAUNCHERS: &[&str] = &[
        "sudo -n",
        "env FOO=1",
        "chroot /",
        "unshare -r",
        "nsenter -t 1 -m",
        "setpriv --nnp",
        "timeout 5",
        "nice -n 10",
        "nohup",
        "stdbuf -oL",
        "setsid",
        "ionice -c 3",
        "chrt 0",
        "taskset 1",
        "flock /tmp/lock",
        "doas --",
        "runuser -u root --",
        "command",
        "exec",
        "time -p",
        "caffeinate -i",
        "systemd-run --scope",
        "pkexec",
        "firejail",
        "bwrap --ro-bind / / --dev /dev",
        "busybox",
        "toybox",
        "noglob",
        "nocorrect",
        "builtin",
        "xargs",
        "gxargs -0",
        "su -c",
        "sg wheel",
        "watch",
        "- noglob",
    ];

    type Shape = (Vec<usize>, bool, Option<usize>);

    fn shape(line: &str, case: NameCase) -> Shape {
        let words = line
            .split_whitespace()
            .map(str::to_string)
            .collect::<Vec<_>>();
        let commands = super::super::policy_wrapper_commands_for(&words, case);
        (
            commands.starts,
            commands.uncertain,
            commands.open.map(|open| open.known_end),
        )
    }

    #[test]
    fn launchers_match_in_any_case_only_where_names_fold() {
        let not_a_launcher: Shape = (vec![0], false, None);
        for launcher in LAUNCHERS {
            let (name, rest) = launcher.split_once(' ').unwrap_or((launcher, ""));
            let line = |name: &str| format!("{name} {rest} git push");
            let canonical = line(name);
            let upper = name.to_ascii_uppercase();
            let mut folded = vec![
                upper.clone(),
                format!("{}{}", &upper[..1], &name[1..]),
                format!("/usr/bin/{upper}"),
            ];
            for (from, to) in [
                ("s", "\u{17f}"),
                ("k", "\u{212a}"),
                ("fi", "\u{fb01}"),
                ("st", "\u{fb06}"),
            ] {
                if name.contains(from) {
                    folded.push(name.replace(from, to));
                }
            }
            let windows_only = [
                format!("{upper}.EXE"),
                format!("C:\\Tools\\{name}.exe"),
                format!("{name}.exe."),
            ];
            for case in [
                NameCase::Sensitive,
                NameCase::Insensitive,
                NameCase::Windows,
            ] {
                let expected = shape(&canonical, case);
                assert_ne!(expected, not_a_launcher, "{canonical} ({case:?})");
                if name == "-" {
                    continue;
                }
                for spelled in &folded {
                    let spelled = line(spelled);
                    let wanted = if case == NameCase::Sensitive {
                        &not_a_launcher
                    } else {
                        &expected
                    };
                    assert_eq!(&shape(&spelled, case), wanted, "{spelled} ({case:?})");
                }
                for spelled in &windows_only {
                    let spelled = line(spelled);
                    let found = shape(&spelled, case);
                    if case == NameCase::Windows {
                        assert_eq!(found, expected, "{spelled}");
                    } else if !matches!(name, "busybox" | "toybox") {
                        // (Any name starting busybox or toybox is theirs.)
                        assert_eq!(found, not_a_launcher, "{spelled} ({case:?})");
                    }
                }
            }
        }
    }

    #[test]
    fn windows_chroot_spellings_read_only_their_own_word() {
        for (spelled, canonical) in [
            (
                "C:\\x\\chroot.exe --userspec=u/chroot / git push",
                "chroot --userspec=u/chroot / git push",
            ),
            (
                "chroot.exe -ux/chroot / git push",
                "chroot -ux/chroot / git push",
            ),
            (
                "CHROOT.EXE / CHROOT.EXE / git push",
                "chroot / chroot / git push",
            ),
        ] {
            assert_eq!(
                shape(spelled, NameCase::Windows),
                shape(canonical, NameCase::Windows),
                "{spelled}"
            );
        }
        // As on main: names the classifier's reader skips (an assignment,
        // a comment) are read as no command.
        for case in [
            NameCase::Sensitive,
            NameCase::Insensitive,
            NameCase::Windows,
        ] {
            for line in ["a=b/chroot / git push", "#x/chroot / git push"] {
                assert_eq!(
                    shape(line, case),
                    (vec![0], false, None),
                    "{line} ({case:?})"
                );
            }
        }
    }

    #[test]
    fn respelled_chains_are_linear() {
        for (launcher, case) in [
            ("CHROOT /", NameCase::Insensitive),
            ("chroot.exe /", NameCase::Windows),
            ("C:\\x\\Chroot.EXE /", NameCase::Windows),
            ("SUDO -n", NameCase::Insensitive),
            ("XARGS -0", NameCase::Insensitive),
            ("\u{17f}etsid", NameCase::Insensitive),
            ("a=b/chroot r=1", NameCase::Windows),
            ("a=b/chroot r=1", NameCase::Sensitive),
        ] {
            super::work::assert_linear(launcher, |scale| {
                let layers = scale * 64;
                let mut words = Vec::new();
                for _ in 0..layers {
                    words.extend(launcher.split(' ').map(str::to_string));
                }
                words.extend(["cmd".to_string(), "-x".to_string()]);
                let found = super::super::policy_wrapper_commands_for(&words, case);
                assert!(
                    found.starts.len() > layers || launcher.starts_with("a=b"),
                    "{launcher}"
                );
            });
        }
    }

    #[test]
    fn path_named_builtin_keeps_its_floor() {
        // Only the shell builtin takes `--`; a program named builtin may
        // read it otherwise.
        for case in [
            NameCase::Sensitive,
            NameCase::Insensitive,
            NameCase::Windows,
        ] {
            assert_eq!(
                shape("/x/builtin -- git push", case),
                (vec![0], true, None),
                "{case:?}"
            );
            assert_eq!(
                shape("builtin -- git push", case),
                (vec![0, 2], false, None)
            );
        }
    }

    #[test]
    fn platform_mode() {
        let expected = if cfg!(windows) {
            NameCase::Windows
        } else if cfg!(target_os = "macos") {
            NameCase::Insensitive
        } else {
            NameCase::Sensitive
        };
        assert_eq!(super::PLATFORM, expected);
    }
}
