use crate::bash::LiteralWordMode;
use crate::bash::parse_shell_lc_literal_commands;
#[path = "windows_dangerous_commands.rs"]
mod windows_dangerous_commands;
pub use windows_dangerous_commands::powershell_commands_are_independent;

/// The platform whose command semantics should be used for safety checks.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DangerousCommandPlatform {
    /// POSIX command and path semantics.
    Posix,
    /// Windows command and path semantics.
    Windows,
}

impl DangerousCommandPlatform {
    /// Returns the platform where the classifier is running.
    pub fn host() -> Self {
        if cfg!(windows) {
            Self::Windows
        } else {
            Self::Posix
        }
    }
}

/// Identifies the dangerous-command rule matched by a command invocation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DangerousCommandMatch {
    /// An `rm` invocation includes the force option.
    ForcedRm,
    /// Another dangerous-command rule matched.
    Other,
}

const MAX_DANGEROUS_COMMAND_WRAPPER_DEPTH: usize = 8;

/// Returns the dangerous-command rule matched by an already-tokenized command.
pub fn dangerous_command_match(command: &[String]) -> Option<DangerousCommandMatch> {
    dangerous_command_match_for_platform(command, DangerousCommandPlatform::host())
}

/// Returns the dangerous-command rule matched using the target platform's semantics.
pub fn dangerous_command_match_for_platform(
    command: &[String],
    platform: DangerousCommandPlatform,
) -> Option<DangerousCommandMatch> {
    dangerous_command_match_with_depth(
        command,
        /*wrapper_depth*/ 0,
        platform,
        LiteralWordMode::Verbatim,
    )
    .or_else(|| {
        // Only add forced-rm matches. Keeping the old pass means decoding a
        // previously omitted "--" cannot weaken an existing refusal.
        dangerous_command_match_with_depth(
            command,
            /*wrapper_depth*/ 0,
            platform,
            LiteralWordMode::DecodeForForcedRm,
        )
        .filter(|matched| *matched == DangerousCommandMatch::ForcedRm)
    })
}

fn dangerous_command_match_with_depth(
    command: &[String],
    wrapper_depth: usize,
    platform: DangerousCommandPlatform,
    mode: LiteralWordMode,
) -> Option<DangerousCommandMatch> {
    if wrapper_depth > MAX_DANGEROUS_COMMAND_WRAPPER_DEPTH {
        return Some(DangerousCommandMatch::Other);
    }

    if let Some(dangerous_match) =
        dangerous_command_match_for_exec(command, wrapper_depth, platform, mode)
    {
        return Some(dangerous_match);
    }

    // Support shell scripts where any literal command might be dangerous,
    // including commands nested in control flow or substitutions.
    if let Some(dangerous_match) =
        parse_shell_lc_literal_commands(command, mode).and_then(|commands| {
            commands.iter().find_map(|command| {
                dangerous_command_match_with_depth(command, wrapper_depth + 1, platform, mode)
            })
        })
    {
        return Some(dangerous_match);
    }

    if platform == DangerousCommandPlatform::Windows
        && mode == LiteralWordMode::Verbatim
        && windows_dangerous_commands::is_dangerous_command_windows(command)
    {
        return Some(DangerousCommandMatch::Other);
    }

    None
}

/// Returns the PowerShell-specific rule matched using the target platform's semantics.
pub fn dangerous_powershell_words_match(
    command: &[String],
    platform: DangerousCommandPlatform,
) -> Option<DangerousCommandMatch> {
    if platform == DangerousCommandPlatform::Windows {
        windows_dangerous_commands::is_dangerous_powershell_words(command)
            .then_some(DangerousCommandMatch::Other)
    } else {
        None
    }
}

/// Classifies a strictly lowered command from an independent literal PowerShell script.
pub fn dangerous_powershell_literal_words_match(
    command: &[String],
) -> Option<DangerousCommandMatch> {
    windows_dangerous_commands::is_dangerous_powershell_literal_words(command)
        .then_some(DangerousCommandMatch::Other)
}

/// The name a program word runs as. D692 (#2543), stricter only: where
/// programs are found without regard to case (Windows NTFS; macOS APFS
/// when this host is a Mac), `RM`, `/bin/Rm` and `ſudo` run `rm` and
/// `sudo`, so names compare case-folded; Windows also ignores trailing dots
/// and spaces. Used by the dangerous-command check only.
fn executable_name_lookup_key(raw: &str, platform: DangerousCommandPlatform) -> Option<String> {
    match platform {
        DangerousCommandPlatform::Posix => {
            let name = raw.rsplit('/').next().filter(|name| !name.is_empty())?;
            Some(if cfg!(target_os = "macos") {
                fold_name(name, /*windows*/ false)
            } else {
                name.to_owned()
            })
        }
        DangerousCommandPlatform::Windows => {
            let name = raw
                .rsplit(['/', '\\'])
                .next()
                .filter(|name| !name.is_empty())?;
            let name = match name.as_bytes() {
                [drive, b':', ..] if drive.is_ascii_alphabetic() => &name[2..],
                _ => name,
            };
            let name = fold_name(name.trim_end_matches(['.', ' ']), /*windows*/ true);
            for suffix in [".exe", ".cmd", ".bat", ".com"] {
                if let Some(stripped) = name.strip_suffix(suffix) {
                    return Some(stripped.to_string());
                }
            }
            (!name.is_empty()).then_some(name)
        }
    }
}

/// ASCII lowercase, and the non-ASCII letters whose case folding is ASCII.
fn fold_name(name: &str, windows: bool) -> String {
    name.chars()
        .map(|ch| match ch {
            '\u{17f}' => 's',
            '\u{212a}' => 'k',
            '\u{131}' if windows => 'i',
            _ => ch.to_ascii_lowercase(),
        })
        .collect()
}

fn dangerous_command_match_for_exec(
    command: &[String],
    wrapper_depth: usize,
    platform: DangerousCommandPlatform,
    mode: LiteralWordMode,
) -> Option<DangerousCommandMatch> {
    let cmd0 = command
        .first()
        .and_then(|command| executable_name_lookup_key(command, platform));

    match cmd0.as_deref() {
        Some("rm") if rm_args_include_force_option(&command[1..]) => {
            Some(DangerousCommandMatch::ForcedRm)
        }

        // For sudo <cmd>, simply check <cmd>.
        Some("sudo") => {
            dangerous_command_match_with_depth(&command[1..], wrapper_depth + 1, platform, mode)
        }

        // Skip environment assignments before checking the command run by env.
        Some("env") => dangerous_command_match_for_env(command, wrapper_depth, platform, mode),

        // A trap action is shell source stored in the first operand.
        Some("trap") => dangerous_command_match_for_trap(command, wrapper_depth, platform, mode),

        // ── anything else ─────────────────────────────────────────────────
        _ => None,
    }
}

fn dangerous_command_match_for_env(
    command: &[String],
    wrapper_depth: usize,
    platform: DangerousCommandPlatform,
    mode: LiteralWordMode,
) -> Option<DangerousCommandMatch> {
    let mut command_index = 1;
    while let Some(argument) = command.get(command_index) {
        if argument == "--" {
            command_index += 1;
            break;
        }
        if matches!(argument.as_str(), "-i" | "--ignore-environment")
            || argument
                .split_once('=')
                .is_some_and(|(name, _)| !name.is_empty() && !name.starts_with('-'))
        {
            command_index += 1;
            continue;
        }
        break;
    }
    dangerous_command_match_with_depth(&command[command_index..], wrapper_depth + 1, platform, mode)
}

fn dangerous_command_match_for_trap(
    command: &[String],
    wrapper_depth: usize,
    platform: DangerousCommandPlatform,
    mode: LiteralWordMode,
) -> Option<DangerousCommandMatch> {
    let mut action_index = 1;
    if command
        .get(action_index)
        .is_some_and(|argument| argument == "--")
    {
        action_index += 1;
    }
    let action = command
        .get(action_index)
        .filter(|action| !action.starts_with('-'))?;
    let shell_command = vec!["sh".to_string(), "-c".to_string(), action.clone()];
    dangerous_command_match_with_depth(&shell_command, wrapper_depth + 1, platform, mode)
}

fn rm_args_include_force_option(args: &[String]) -> bool {
    args.iter()
        .take_while(|arg| arg.as_str() != "--")
        .any(|arg| {
            // GNU getopt_long accepts every nonempty, unambiguous prefix.
            arg.strip_prefix("--")
                .is_some_and(|option| !option.is_empty() && "force".starts_with(option))
                || arg
                    .strip_prefix('-')
                    .is_some_and(|flags| !flags.starts_with('-') && flags.contains('f'))
        })
}

#[cfg(test)]
#[path = "forced_rm_spelling_tests.rs"]
mod forced_rm_spelling_tests;

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn vec_str(items: &[&str]) -> Vec<String> {
        items.iter().map(std::string::ToString::to_string).collect()
    }

    #[test]
    fn rm_rf_is_dangerous() {
        assert_eq!(
            dangerous_command_match(&vec_str(&["rm", "-rf", "/"])),
            Some(DangerousCommandMatch::ForcedRm)
        );
    }

    #[test]
    fn rm_f_is_dangerous() {
        assert_eq!(
            dangerous_command_match(&vec_str(&["rm", "-f", "/"])),
            Some(DangerousCommandMatch::ForcedRm)
        );
    }

    #[test]
    fn forced_rm_variants_are_dangerous() {
        for command in [
            vec_str(&["/bin/rm", "-fr", "/tmp/example"]),
            vec_str(&["rm", "-r", "-f", "/tmp/example"]),
            vec_str(&["rm", "--force", "/tmp/example"]),
            vec_str(&["rm", "/tmp/example", "-f"]),
            vec_str(&["sudo", "rm", "-rf", "/tmp/example"]),
            vec_str(&["env", "TARGET=/tmp/example", "rm", "-rf", "/tmp/example"]),
        ] {
            assert_eq!(
                dangerous_command_match(&command),
                Some(DangerousCommandMatch::ForcedRm),
                "{command:?}"
            );
        }
    }

    #[test]
    fn deeply_nested_command_wrappers_fail_closed() {
        for (depth, expected) in [
            (
                MAX_DANGEROUS_COMMAND_WRAPPER_DEPTH,
                DangerousCommandMatch::ForcedRm,
            ),
            (
                MAX_DANGEROUS_COMMAND_WRAPPER_DEPTH + 1,
                DangerousCommandMatch::Other,
            ),
        ] {
            let command = std::iter::repeat_n("env", depth)
                .chain(["rm", "-rf", "/tmp/example"])
                .map(str::to_owned)
                .collect::<Vec<_>>();

            assert_eq!(dangerous_command_match(&command), Some(expected));
        }
    }

    #[test]
    fn forced_rm_in_complex_shell_syntax_is_dangerous() {
        for script in [
            "printf x | rm -rf /tmp/example",
            "if test -d /tmp/example; then rm --force /tmp/example; fi",
            "rm -rf \"$TARGET\" >/dev/null",
            "for target in /tmp/a /tmp/b; do rm -r -f \"$target\"; done",
            "echo \"$(rm -rf /tmp/example)\"",
            "bash -c 'rm -rf /tmp/example'",
            "trap 'rm -rf /tmp/example' EXIT",
            "for a in '-C5a25KeRr' '--' '--json' '--bogus'; do HOME=$(mktemp -d) MDE_URL=http://127.0.0.1:1 MDE_TOKEN=x node cli/mde.cjs ls \"$a\" >/tmp/mde-review-out 2>/tmp/mde-review-err; code=$?; printf '%s\\t%s\\t%s\\n' \"$a\" \"$code\" \"$(tr '\\n' ' ' </tmp/mde-review-err)\"; rm -rf \"$HOME\"; done",
        ] {
            let command = vec_str(&["bash", "-lc", script]);
            assert_eq!(
                dangerous_command_match(&command),
                Some(DangerousCommandMatch::ForcedRm),
                "{script}"
            );
        }
    }

    /// Corbanu (S3b): words that may still expand keep their source spelling
    /// in the danger check, as on main.
    #[test]
    fn forced_rm_with_expandable_words_is_dangerous() {
        for script in [
            "rm -r\\f /",
            "rm -{r,f} /",
            "rm -rf{,} /tmp/example",
            "rm -rf* /tmp/example",
            "rm \"-rf\\\n\" /tmp/example",
            "rm -r\"f\\\n\" /tmp/example",
            "rm \"-r\\f\" /tmp/example",
        ] {
            let command = vec_str(&["bash", "-lc", script]);
            assert_eq!(
                dangerous_command_match(&command),
                Some(DangerousCommandMatch::ForcedRm),
                "{script}"
            );
        }
    }

    #[test]
    fn non_forced_or_non_literal_rm_is_not_dangerous() {
        for command in [
            vec_str(&["rm", "-r", "/tmp/example"]),
            vec_str(&["rm", "--", "-f"]),
            vec_str(&["bash", "-lc", "echo 'rm -rf /tmp/example'"]),
            vec_str(&["bash", "-lc", "cmd=rm; $cmd -rf /tmp/example"]),
            vec_str(&["bash", "-lc", "if then rm -rf /tmp/example"]),
            vec_str(&["env", "TARGET=/tmp/example", "rm", "-r", "/tmp/example"]),
            vec_str(&["bash", "-lc", "trap 'echo rm -rf /tmp/example' EXIT"]),
        ] {
            assert_eq!(dangerous_command_match(&command), None, "{command:?}");
        }
    }

    #[test]
    fn direct_powershell_words_return_other_match_on_windows() {
        let command = vec_str(&["Remove-Item", "test", "-Force"]);

        if cfg!(windows) {
            assert_eq!(
                dangerous_powershell_words_match(&command, DangerousCommandPlatform::host()),
                Some(DangerousCommandMatch::Other)
            );
        } else {
            assert_eq!(
                dangerous_powershell_words_match(&command, DangerousCommandPlatform::host()),
                None
            );
        }
    }
    #[test]
    fn program_names_fold_case_where_the_platform_does() {
        // D692 (#2543, stricter only): Windows, and macOS hosts, run `RM`,
        // `/bin/Rm` and `ſudo` as `rm` and `sudo`; Linux runs other programs.
        for (platform, folds) in [
            (DangerousCommandPlatform::Windows, true),
            (DangerousCommandPlatform::Posix, cfg!(target_os = "macos")),
        ] {
            for command in [
                vec_str(&["RM", "-rf", "x"]),
                vec_str(&["Rm", "-f", "x"]),
                vec_str(&["/bin/RM", "-rf", "x"]),
                vec_str(&["SUDO", "rm", "-rf", "x"]),
                vec_str(&["\u{17f}\u{75}do", "RM", "-rf", "x"]),
                vec_str(&["Env", "A=1", "rM", "-rf", "x"]),
                vec_str(&["bash", "-lc", "RM -rf x"]),
                vec_str(&["bash", "-lc", "echo hi && TRAP 'RM -rf x' EXIT"]),
            ] {
                let expected = folds.then_some(DangerousCommandMatch::ForcedRm);
                assert_eq!(
                    dangerous_command_match_for_platform(&command, platform),
                    expected,
                    "{platform:?} {command:?}"
                );
            }
        }
        for program in ["RM.EXE", "rm.exe.", "Rm.Exe . "] {
            let command = vec_str(&[program, "-rf", "x"]);
            assert_eq!(
                dangerous_command_match_for_platform(&command, DangerousCommandPlatform::Windows),
                Some(DangerousCommandMatch::ForcedRm),
                "{command:?}"
            );
        }
        for command in [vec_str(&["RM", "-r", "x"]), vec_str(&["RMX", "-rf", "x"])] {
            for platform in [
                DangerousCommandPlatform::Windows,
                DangerousCommandPlatform::Posix,
            ] {
                assert_eq!(
                    dangerous_command_match_for_platform(&command, platform),
                    None,
                    "{command:?}"
                );
            }
        }
    }
}
