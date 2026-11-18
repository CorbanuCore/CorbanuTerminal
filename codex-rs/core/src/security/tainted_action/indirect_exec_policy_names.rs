//! #2503 (D688): the name an exec-policy launcher word runs as. Launchers
//! (`sudo`, `env`, `xargs`, `su`, ...) are recognized by this name only, so
//! every launcher, present and future, matches names the same way.
//!
//! macOS (case-insensitive APFS, the default) and Windows (NTFS) look
//! program names up without regard to case: there `SUDO`, `/usr/bin/Env`
//! and `ſu` run `sudo`, `env` and `su`. APFS applies full Unicode case
//! folding, which maps a few non-ASCII letters and ligatures to ASCII
//! ([`FOLDS`]; checked on APFS over every code point). Windows also drops an
//! `.exe` suffix and trailing dots and spaces, separates directories with
//! `\` too, and folds the dotless `ı` ([`WINDOWS_FOLDS`]). #2510: names this module cannot canonicalize confidently
//! are [`unsure`]: characters legacy HFS+ volumes ignore (such as U+200D)
//! on macOS, and Windows 8.3 short names (`BUSYBO~1.EXE`) and stream
//! suffixes (`sudo::$DATA`, `sudo:s`). Callers fail closed on them.
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

/// #2510: NTFS compares names after mapping them through its upcase table,
/// which follows Unicode's simple uppercase: besides the ASCII letters and
/// `ſ` (in [`FOLDS`]), only the dotless `ı` uppercases to ASCII (`I`).
const WINDOWS_FOLDS: &[(char, &str)] = &[('\u{131}', "i")];

/// The launcher name `word` runs as where names compare as `case` says
/// ([`PLATFORM`] for this platform).
pub(super) fn launcher_name_for(word: &str, case: NameCase) -> Cow<'_, str> {
    let name = match case {
        NameCase::Sensitive => return Cow::Borrowed(basename(word)),
        NameCase::Insensitive => basename(word),
        NameCase::Windows => windows_file_name(word),
    };
    let folded = fold(name, case);
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

/// #2510: whether the program `word` may run as a name [`launcher_name_for`]
/// cannot tell (a launcher or shell spelled another way), so its argv
/// boundaries are unknown. macOS: the name holds a default-ignorable code
/// point, which legacy HFS+ volumes drop from names (`su\u{200d}do` is
/// `sudo` there). Windows: the file name is an 8.3 short name (`~` then a
/// digit: `BUSYBO~1.EXE` may be `busybox.exe`) or names an NTFS stream
/// (`sudo::$DATA`). Only the file name counts: short directory names such
/// as `C:\PROGRA~1\` are common and do not change which program runs.
pub(super) fn unsure(word: &str, case: NameCase) -> bool {
    match case {
        NameCase::Sensitive => false,
        NameCase::Insensitive => basename(word).chars().any(|c| {
            work::step();
            default_ignorable(c)
        }),
        NameCase::Windows => {
            let name = windows_file_name(word);
            name.contains(':')
                || name
                    .as_bytes()
                    .windows(2)
                    .any(|pair| pair[0] == b'~' && pair[1].is_ascii_digit())
        }
    }
}

/// Unicode's Default_Ignorable_Code_Point property (a superset of the
/// characters HFS+ ignores in names: U+200C..U+200F, U+202A..U+202E,
/// U+206A..U+206F and U+FEFF).
fn default_ignorable(c: char) -> bool {
    matches!(
        u32::from(c),
        0xAD | 0x34F
            | 0x61C
            | 0x115F..=0x1160
            | 0x17B4..=0x17B5
            | 0x180B..=0x180F
            | 0x200B..=0x200F
            | 0x202A..=0x202E
            | 0x2060..=0x206F
            | 0x3164
            | 0xFE00..=0xFE0F
            | 0xFEFF
            | 0xFFA0
            | 0xFFF0..=0xFFF8
            | 0x1BCA0..=0x1BCA3
            | 0x1D173..=0x1D17A
            | 0xE0000..=0xE0FFF
    )
}

/// `name` with ASCII letters lowercased and [`FOLDS`] (and on Windows
/// [`WINDOWS_FOLDS`]) applied.
fn fold(name: &str, case: NameCase) -> Cow<'_, str> {
    if name.is_ascii() && !name.bytes().any(|byte| byte.is_ascii_uppercase()) {
        return Cow::Borrowed(name);
    }
    let mut folded = String::with_capacity(name.len());
    for c in name.chars() {
        work::step();
        let windows = if case == NameCase::Windows {
            WINDOWS_FOLDS
        } else {
            &[]
        };
        match FOLDS.iter().chain(windows).find(|(from, _)| *from == c) {
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
            // #2510: NTFS uppercases the dotless i to I; APFS does not fold it.
            ("t\u{131}meout", "t\u{131}meout", "t\u{131}meout", "timeout"),
            ("N\u{131}CE", "N\u{131}CE", "n\u{131}ce", "nice"),
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
    }

    #[test]
    fn chroot_named_like_an_assignment_reads_its_own_argv() {
        // #2510: argv runs its first word as a program, so a chroot named
        // like an assignment or a comment reads its own root and command,
        // not a later chroot's (main read no command, or the later one).
        for case in [
            NameCase::Sensitive,
            NameCase::Insensitive,
            NameCase::Windows,
        ] {
            for (line, starts) in [
                ("a=b/chroot / git push", vec![0, 2]),
                ("#x/chroot / git push", vec![0, 2]),
                ("a=b/chroot chroot / evil", vec![0, 2]),
                ("a=b/chroot --userspec=u / git push", vec![0, 3]),
                ("/usr/sbin/chroot / git push", vec![0, 2]),
                ("chroot / a=b/chroot / git push", vec![0, 2, 4]),
            ] {
                assert_eq!(
                    shape(line, case),
                    (starts, false, None),
                    "{line} ({case:?})"
                );
            }
        }
    }

    #[test]
    fn unsure_names_fail_closed() {
        let uncertain: Shape = (vec![0], true, None);
        let other_program: Shape = (vec![0], false, None);
        for (line, unsure_in) in [
            // Default-ignorable code points: HFS+ drops some from names.
            ("su\u{200d}do -n git push", &[NameCase::Insensitive][..]),
            ("/usr/bin/\u{feff}env git push", &[NameCase::Insensitive]),
            ("BA\u{200c}SH -c x", &[NameCase::Insensitive]),
            ("x\u{e0001} git push", &[NameCase::Insensitive]),
            // Windows short names and NTFS streams.
            ("BUSYBO~1.EXE git push", &[NameCase::Windows]),
            ("SUDO~1 -n git push", &[NameCase::Windows]),
            ("C:\\x\\BASH~1.EXE -c x", &[NameCase::Windows]),
            ("sudo::$DATA -n git push", &[NameCase::Windows]),
            ("C:env.exe:s git push", &[NameCase::Windows]),
            // Behind a launcher the platform reads.
            ("sudo -n su\u{200d}do git push", &[NameCase::Insensitive]),
            ("sudo -n XARG~1 git push", &[NameCase::Windows]),
        ] {
            for case in [
                NameCase::Sensitive,
                NameCase::Insensitive,
                NameCase::Windows,
            ] {
                let found = shape(line, case);
                if unsure_in.contains(&case) {
                    let expected = if line.starts_with("sudo -n") {
                        (vec![0, 2], true, None)
                    } else {
                        uncertain.clone()
                    };
                    assert_eq!(found, expected, "{line} ({case:?})");
                } else if !line.starts_with("sudo -n") {
                    assert_eq!(found, other_program, "{line} ({case:?})");
                }
            }
        }
        // Short directory names and ordinary names stay readable.
        for line in [
            "C:\\PROGRA~1\\Git\\bin\\git.exe push",
            "git~ push",
            "~1 push",
        ] {
            for case in [
                NameCase::Sensitive,
                NameCase::Insensitive,
                NameCase::Windows,
            ] {
                let expected = if case == NameCase::Windows && line == "~1 push" {
                    uncertain.clone()
                } else {
                    other_program.clone()
                };
                assert_eq!(shape(line, case), expected, "{line} ({case:?})");
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
    fn shells_respell_only_where_names_fold() {
        for (shell, flag, insensitive, windows) in [
            // The shell parser reads these already.
            ("bash", "-lc", None, None),
            ("/bin/sh", "-c", None, None),
            ("bash.exe", "-c", None, None),
            ("BASH", "-lc", Some("bash"), Some("bash")),
            ("/bin/Sh", "-c", Some("sh"), Some("sh")),
            ("Z\u{17f}h", "-c", Some("zsh"), Some("zsh")),
            ("Bash.exe", "-c", None, Some("bash")),
            ("C:\\x\\SH.EXE.", "-c", None, Some("sh")),
            ("BASH", "-x", None, None),
            ("BASH~1.EXE", "-c", None, None),
            ("BA\u{200d}SH", "-c", None, None),
            ("BASHX", "-c", None, None),
            ("FISH", "-c", None, None),
        ] {
            let words = [shell, flag, "git push"].map(str::to_string);
            let respelled = |case| {
                super::super::policy_shell_script_for(&words, case)
                    .map(|words| (words[0].clone(), words[1..] == [flag, "git push"]))
            };
            assert_eq!(respelled(NameCase::Sensitive), None, "{shell}");
            for (case, expected) in [
                (NameCase::Insensitive, insensitive),
                (NameCase::Windows, windows),
            ] {
                assert_eq!(
                    respelled(case),
                    expected.map(|name: &str| (name.to_string(), true)),
                    "{shell} ({case:?})"
                );
            }
        }
    }

    #[test]
    fn posix_shells_read_as_sh_everywhere() {
        // #2543: `dash -c` and the like read like `sh -c`.
        for (shell, folds) in [
            ("dash", false),
            ("/usr/bin/ash", false),
            ("posh", false),
            ("DASH", true),
            ("/bin/Ash", true),
        ] {
            let words = [shell, "-c", "git push"].map(str::to_string);
            for case in [
                NameCase::Sensitive,
                NameCase::Insensitive,
                NameCase::Windows,
            ] {
                let expected = (!(folds && case == NameCase::Sensitive))
                    .then(|| ["sh", "-c", "git push"].map(str::to_string).to_vec());
                assert_eq!(
                    super::super::policy_shell_script_for(&words, case),
                    expected,
                    "{shell} ({case:?})"
                );
            }
        }
        // Shells with other syntax are not read as sh.
        // #2548: nor the ksh family and the like, whose builtins run code
        // where sh's do not.
        for shell in [
            "fish",
            "csh",
            "tcsh",
            "ksh",
            "/usr/bin/mksh",
            "yash",
            "hush",
        ] {
            let words = [shell, "-c", "git push"].map(str::to_string);
            assert_eq!(
                super::super::policy_shell_script_for(&words, NameCase::Sensitive),
                None,
                "{shell}"
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
