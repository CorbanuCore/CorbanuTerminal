use crate::bash::LiteralWordMode;
use crate::bash::parse_shell_lc_literal_commands;
use std::path::Path;
#[cfg(windows)]
#[path = "windows_dangerous_commands.rs"]
mod windows_dangerous_commands;

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
    dangerous_command_match_with_depth(command, /*wrapper_depth*/ 0, LiteralWordMode::Verbatim)
        .or_else(|| {
            // Only add forced-rm matches. Keeping the old pass means decoding a
            // previously omitted "--" cannot weaken an existing refusal.
            dangerous_command_match_with_depth(
                command,
                /*wrapper_depth*/ 0,
                LiteralWordMode::DecodeForForcedRm,
            )
            .filter(|matched| *matched == DangerousCommandMatch::ForcedRm)
        })
}

fn dangerous_command_match_with_depth(
    command: &[String],
    wrapper_depth: usize,
    mode: LiteralWordMode,
) -> Option<DangerousCommandMatch> {
    if wrapper_depth > MAX_DANGEROUS_COMMAND_WRAPPER_DEPTH {
        return None;
    }

    if let Some(dangerous_match) = dangerous_command_match_for_exec(command, wrapper_depth, mode) {
        return Some(dangerous_match);
    }

    // Support shell scripts where any literal command might be dangerous,
    // including commands nested in control flow or substitutions.
    if let Some(dangerous_match) =
        parse_shell_lc_literal_commands(command, mode).and_then(|commands| {
            commands.iter().find_map(|command| {
                dangerous_command_match_with_depth(command, wrapper_depth + 1, mode)
            })
        })
    {
        return Some(dangerous_match);
    }

    #[cfg(windows)]
    {
        if mode == LiteralWordMode::Verbatim
            && windows_dangerous_commands::is_dangerous_command_windows(command)
        {
            return Some(DangerousCommandMatch::Other);
        }
    }

    None
}

/// Returns the dangerous-command rule matched by tokenized PowerShell words.
pub fn dangerous_powershell_words_match(command: &[String]) -> Option<DangerousCommandMatch> {
    #[cfg(windows)]
    {
        windows_dangerous_commands::is_dangerous_powershell_words(command)
            .then_some(DangerousCommandMatch::Other)
    }

    #[cfg(not(windows))]
    {
        let _ = command;
        None
    }
}

fn is_git_global_option_with_value(arg: &str) -> bool {
    matches!(
        arg,
        "-C" | "-c"
            | "--config-env"
            | "--exec-path"
            | "--git-dir"
            | "--namespace"
            | "--super-prefix"
            | "--work-tree"
    )
}

fn is_git_global_option_with_inline_value(arg: &str) -> bool {
    matches!(
        arg,
        s if s.starts_with("--config-env=")
            || s.starts_with("--exec-path=")
            || s.starts_with("--git-dir=")
            || s.starts_with("--namespace=")
            || s.starts_with("--super-prefix=")
            || s.starts_with("--work-tree=")
    ) || ((arg.starts_with("-C") || arg.starts_with("-c")) && arg.len() > 2)
}

pub(crate) fn executable_name_lookup_key(raw: &str) -> Option<String> {
    #[cfg(windows)]
    {
        Path::new(raw)
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| {
                let name = name.to_ascii_lowercase();
                for suffix in [".exe", ".cmd", ".bat", ".com"] {
                    if let Some(stripped) = name.strip_suffix(suffix) {
                        return stripped.to_string();
                    }
                }
                name
            })
    }

    #[cfg(not(windows))]
    {
        Path::new(raw)
            .file_name()
            .and_then(|name| name.to_str())
            .map(std::borrow::ToOwned::to_owned)
    }
}

/// Find the first matching git subcommand, skipping known global options that
/// may appear before it (e.g., `-C`, `-c`, `--git-dir`).
///
/// Shared with `is_safe_command` to avoid git-global-option bypasses.
pub(crate) fn find_git_subcommand<'a>(
    command: &'a [String],
    subcommands: &[&str],
) -> Option<(usize, &'a str)> {
    let cmd0 = command.first().map(String::as_str)?;
    if executable_name_lookup_key(cmd0).as_deref() != Some("git") {
        return None;
    }

    let mut skip_next = false;
    for (idx, arg) in command.iter().enumerate().skip(1) {
        if skip_next {
            skip_next = false;
            continue;
        }

        let arg = arg.as_str();

        if is_git_global_option_with_inline_value(arg) {
            continue;
        }

        if is_git_global_option_with_value(arg) {
            skip_next = true;
            continue;
        }

        if arg == "--" || arg.starts_with('-') {
            continue;
        }

        if subcommands.contains(&arg) {
            return Some((idx, arg));
        }

        // In git, the first non-option token is the subcommand. If it isn't
        // one of the subcommands we're looking for, we must stop scanning to
        // avoid misclassifying later positional args (e.g., branch names).
        return None;
    }

    None
}

/// The name a program word runs as, for the dangerous-command check only
/// (D692: stricter only, so never for the safe-command list). Where this
/// platform finds programs without regard to case (macOS APFS, Windows
/// NTFS), `RM`, `/bin/Rm` and `ſudo` run `rm` and `sudo`, so they compare
/// case-folded; Windows also ignores trailing dots and spaces.
fn dangerous_name_key(raw: &str) -> Option<String> {
    let name = executable_name_lookup_key(raw)?;
    if !cfg!(any(windows, target_os = "macos")) {
        return Some(name);
    }
    let name = if cfg!(windows) {
        name.trim_end_matches(['.', ' '])
    } else {
        &name
    };
    let mut folded = String::with_capacity(name.len());
    for ch in name.chars() {
        match ch {
            // Non-ASCII letters whose case folding is ASCII.
            '\u{17f}' => folded.push('s'),
            '\u{212a}' => folded.push('k'),
            '\u{131}' if cfg!(windows) => folded.push('i'),
            _ => folded.push(ch.to_ascii_lowercase()),
        }
    }
    if cfg!(windows) {
        for suffix in [".exe", ".cmd", ".bat", ".com"] {
            if let Some(stripped) = folded.strip_suffix(suffix) {
                return Some(stripped.to_string());
            }
        }
    }
    Some(folded)
}

fn dangerous_command_match_for_exec(
    command: &[String],
    wrapper_depth: usize,
    mode: LiteralWordMode,
) -> Option<DangerousCommandMatch> {
    let cmd0 = command
        .first()
        .and_then(|command| dangerous_name_key(command));

    match cmd0.as_deref() {
        Some("rm") if rm_args_include_force_option(&command[1..]) => {
            Some(DangerousCommandMatch::ForcedRm)
        }

        // For sudo <cmd>, simply check <cmd>.
        Some("sudo") => dangerous_command_match_with_depth(&command[1..], wrapper_depth + 1, mode),

        // Skip environment assignments before checking the command run by env.
        Some("env") => dangerous_command_match_for_env(command, wrapper_depth, mode),

        // A trap action is shell source stored in the first operand.
        Some("trap") => dangerous_command_match_for_trap(command, wrapper_depth, mode),

        // ── anything else ─────────────────────────────────────────────────
        _ => None,
    }
}

fn dangerous_command_match_for_env(
    command: &[String],
    wrapper_depth: usize,
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
    dangerous_command_match_with_depth(&command[command_index..], wrapper_depth + 1, mode)
}

fn dangerous_command_match_for_trap(
    command: &[String],
    wrapper_depth: usize,
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
    dangerous_command_match_with_depth(&shell_command, wrapper_depth + 1, mode)
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
                dangerous_powershell_words_match(&command),
                Some(DangerousCommandMatch::Other)
            );
        } else {
            assert_eq!(dangerous_powershell_words_match(&command), None);
        }
    }
    #[test]
    fn program_names_fold_case_where_the_platform_does() {
        // D692 (stricter only): macOS and Windows run `RM`, `/bin/Rm` and
        // `ſudo` as `rm` and `sudo`; Linux runs other programs.
        let folds = cfg!(any(windows, target_os = "macos"));
        for command in [
            vec_str(&["RM", "-rf", "x"]),
            vec_str(&["Rm", "-f", "x"]),
            vec_str(&["/bin/RM", "-rf", "x"]),
            vec_str(&["SUDO", "rm", "-rf", "x"]),
            vec_str(&["\u{17f}udo", "RM", "-rf", "x"]),
            vec_str(&["Env", "A=1", "rM", "-rf", "x"]),
            vec_str(&["bash", "-lc", "RM -rf x"]),
            vec_str(&["bash", "-lc", "echo hi && TRAP 'RM -rf x' EXIT"]),
        ] {
            let expected = folds.then_some(DangerousCommandMatch::ForcedRm);
            assert_eq!(dangerous_command_match(&command), expected, "{command:?}");
        }
        if cfg!(windows) {
            for program in ["RM.EXE", "rm.exe.", "Rm.Exe . "] {
                let command = vec_str(&[program, "-rf", "x"]);
                assert_eq!(
                    dangerous_command_match(&command),
                    Some(DangerousCommandMatch::ForcedRm),
                    "{command:?}"
                );
            }
        }
        // Folding is for the dangerous check only; still not dangerous:
        for command in [vec_str(&["RM", "-r", "x"]), vec_str(&["RMX", "-rf", "x"])] {
            assert_eq!(dangerous_command_match(&command), None, "{command:?}");
        }
    }

    #[test]
    fn safe_commands_do_not_fold_case() {
        // D692: folding never widens the safe list (Windows has its own).
        if !cfg!(windows) {
            let command = vec_str(&["LS", "-la"]);
            assert!(!crate::is_safe_command::is_known_safe_command(&command));
        }
    }
}
