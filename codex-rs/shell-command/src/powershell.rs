use std::borrow::Cow;
use std::path::PathBuf;

use codex_utils_absolute_path::AbsolutePathBuf;

use crate::command_safety::try_parse_powershell_command_words;
use crate::command_safety::try_parse_powershell_commands;
use crate::shell_detect::ShellType;
use crate::shell_detect::detect_shell_type;

mod allowlist;
mod cli;
mod command_scope;

pub use command_scope::PowershellCommandScope;
pub use command_scope::PowershellCommandScopeGuard;
pub use command_scope::enter_powershell_command_scope;

macro_rules! utf8_output_prefix {
    () => {
        "try { [Console]::OutputEncoding=[System.Text.Encoding]::UTF8 } catch {}\n"
    };
}

macro_rules! module_hardening_prefix {
    () => {
        "$global:PSModuleAutoLoadingPreference = 'None'; & { foreach ($m in 'Microsoft.PowerShell.Management', 'Microsoft.PowerShell.Utility', 'Microsoft.PowerShell.Security', 'Microsoft.PowerShell.Archive', 'CimCmdlets') { Import-Module -Name \"$PSHOME\\Modules\\$m\\$m.psd1\" -Global -ErrorAction SilentlyContinue } }\n"
    };
}

/// Prefixed command for powershell shell calls to request UTF-8 console output.
pub const UTF8_OUTPUT_PREFIX: &str = utf8_output_prefix!();

/// D729 (A), D731 (#2661): module autoloading off, and the system modules
/// PowerShell's own commands come from loaded from its install folder
/// (`$PSHOME\Modules`, by full path), so a module a command wrote into a user
/// module folder cannot replace a cmdlet the agent's script calls. Other
/// modules load only when a script imports them (which asks under strict
/// rules). Not covered: a profile script, which runs before this when the
/// shell is started without `-NoProfile` (D729 (A) reads such a script in
/// part where a profile exists); PowerShell processes, jobs and runspaces
/// the script starts; command lines other than `-Command`. No .NET calls,
/// so it also runs in Constrained Language Mode.
pub const MODULE_HARDENING_PREFIX: &str = module_hardening_prefix!();

/// What [`prefix_powershell_script_with_utf8`] puts before an agent script.
pub const AGENT_SCRIPT_PREFIX: &str = concat!(module_hardening_prefix!(), utf8_output_prefix!());

/// `command` with [`AGENT_SCRIPT_PREFIX`] (module hardening, then
/// [`UTF8_OUTPUT_PREFIX`]) before its `-Command` script, which becomes one
/// argument (PowerShell joins a script's arguments with spaces anyway). Any
/// other command is returned as it is.
pub fn prefix_powershell_script_with_utf8(command: &[String]) -> Vec<String> {
    let Some((
        _,
        cli::Invocation {
            run: cli::Run::Command { start, script },
            ..
        },
    )) = read_invocation(command)
    else {
        return command.to_vec();
    };

    let script = if script.trim_start().starts_with(AGENT_SCRIPT_PREFIX) {
        script.into_owned()
    } else {
        format!("{AGENT_SCRIPT_PREFIX}{script}")
    };

    let mut prefixed = command[..start].to_vec();
    prefixed.push(script);
    prefixed
}

/// Which PowerShell a command line is written for: `pwsh` (PowerShell 7) and
/// `powershell` (Windows PowerShell 5.1) read different parameters.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PowershellKind {
    Pwsh,
    WindowsPowershell,
}

impl PowershellKind {
    /// The kind a PowerShell program name (`pwsh`, `C:\…\PowerShell.EXE`)
    /// runs.
    pub(crate) fn of(shell: &str) -> Self {
        let name = std::path::Path::new(shell)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(shell)
            .to_ascii_lowercase();
        // #2568: Windows drops trailing dots and spaces (`pwsh.exe.`).
        let name = name.trim_end_matches(['.', ' ']);
        let stem = name.strip_suffix(".exe").unwrap_or(name);
        if stem.trim_end_matches(['.', ' ']) == "pwsh" {
            Self::Pwsh
        } else {
            Self::WindowsPowershell
        }
    }
}

/// Whether `program` names a PowerShell this crate reads command lines of
/// (`pwsh`, `powershell.exe`, `C:\…\pwsh.exe`).
pub fn is_powershell_program(program: &str) -> bool {
    matches!(
        detect_shell_type(PathBuf::from(program)),
        Some(ShellType::PowerShell)
    )
}

/// `command` read as PowerShell reads its command line (#2568); `None` when
/// `command[0]` is not a PowerShell.
fn read_invocation(command: &[String]) -> Option<(&str, cli::Invocation<'_>)> {
    let shell = command.first()?;
    if !is_powershell_program(shell) {
        return None;
    }
    Some((shell, cli::read(command, PowershellKind::of(shell))))
}

/// Extract the script a PowerShell invocation runs, as PowerShell reads it:
///
/// - ["pwsh", "-NoProfile", "-Command", "Get-ChildItem -Recurse | Select-String foo"]
/// - ["powershell.exe", "-c", "git status", "; git push"] runs
///   `git status ; git push`: PowerShell joins every argument after
///   `-Command` with spaces (#2568).
/// - ["pwsh", "-EncodedCommand", "<base64 of UTF-16LE>"], decoded.
///
/// Returns (`shell`, `script`); `None` for any other command, including a
/// PowerShell invocation whose script cannot be read (see
/// [`unread_powershell_command_line`]).
pub fn extract_powershell_command(command: &[String]) -> Option<(&str, Cow<'_, str>)> {
    let (shell, invocation) = read_invocation(command)?;
    match invocation.run {
        cli::Run::Command { script, .. } => Some((shell, script)),
        cli::Run::Encoded(script) => Some((shell, Cow::Owned(script))),
        cli::Run::File(_) | cli::Run::Unreadable | cli::Run::Interactive => None,
    }
}

/// [`extract_powershell_command`] for a plain `-Command` (or `-c`)
/// invocation only, with no other parameter than `-NoLogo`, `-NoProfile`,
/// `-NonInteractive`, `-STA` or `-MTA`.
pub fn extract_plain_powershell_command(command: &[String]) -> Option<(&str, Cow<'_, str>)> {
    let (shell, invocation) = read_invocation(command)?;
    match invocation.run {
        cli::Run::Command { script, .. } if invocation.plain => Some((shell, script)),
        _ => None,
    }
}

/// #2568 (D553): a PowerShell command line whose script rules cannot read.
#[derive(Debug, PartialEq, Eq)]
pub enum UnreadPowershell<'a> {
    /// `-File`: the script file and its arguments, which rules see as a
    /// command; the file's content is not read.
    File(&'a [String]),
    /// Known literal prefixes in a script that cannot be fully lowered.
    /// These may only add prompt or forbidden matches.
    Partial(Vec<Vec<String>>),
    /// A command line this reader cannot read the way PowerShell reads it:
    /// an unknown or unsupported parameter, a script from stdin,
    /// `-CommandWithArgs` with arguments, an `-EncodedCommand` that does not
    /// decode.
    Unreadable,
}

/// The way `command`, a PowerShell invocation, runs a script rules cannot
/// read; `None` for other commands and scripts fully lowered into a plain
/// command list. A PowerShell invocation without a script can read stdin.
pub fn unread_powershell_command_line(command: &[String]) -> Option<UnreadPowershell<'_>> {
    let (shell, invocation) = read_invocation(command)?;
    match invocation.run {
        cli::Run::File(file) => Some(UnreadPowershell::File(file)),
        cli::Run::Unreadable | cli::Run::Interactive => Some(UnreadPowershell::Unreadable),
        cli::Run::Command { script, .. } => unread_script(shell, invocation.no_profile, &script),
        cli::Run::Encoded(script) => unread_script(shell, invocation.no_profile, &script),
    }
}

fn unread_script(shell: &str, no_profile: bool, script: &str) -> Option<UnreadPowershell<'static>> {
    if let Some(commands) = try_parse_powershell_command_words(script) {
        let read_in_part = allowlist_reads_in_part(shell, no_profile, &commands);
        let commands = word_texts(commands);
        return read_in_part.then_some(UnreadPowershell::Partial(commands));
    }
    let prefixes = crate::command_safety::powershell_restrictive_prefixes(script);
    Some(if prefixes.is_empty() {
        UnreadPowershell::Unreadable
    } else {
        UnreadPowershell::Partial(prefixes)
    })
}

/// D729 (A): where a prompt or forbidden rule exists (D733), a lowered script
/// is read as its commands only when every command is allowlisted (or a
/// program on the command's `PATH`); otherwise in part, its literal commands
/// adding prompts and refusals only.
fn allowlist_reads_in_part(
    shell: &str,
    no_profile: bool,
    commands: &[Vec<crate::command_safety::LoweredWord>],
) -> bool {
    command_scope::allowlist_applies()
        && allowlist::review(shell, PowershellKind::of(shell), no_profile, commands).is_some()
}

fn word_texts(commands: Vec<Vec<crate::command_safety::LoweredWord>>) -> Vec<Vec<String>> {
    commands
        .into_iter()
        .map(|command| command.into_iter().map(|word| word.text).collect())
        .collect()
}

/// The script of `command`, a PowerShell command line, and its commands when
/// rules read it in full: lowered, and (D729 (A), within a
/// [`PowershellCommandScope`] where the allowlist applies) every command
/// allowlisted. `None` for any other command line, which
/// [`unread_powershell_command_line`] reads in part.
pub fn read_powershell_plain_commands(
    command: &[String],
) -> Option<(Cow<'_, str>, Vec<Vec<String>>)> {
    let (shell, invocation) = read_invocation(command)?;
    let script = match invocation.run {
        cli::Run::Command { script, .. } => script,
        cli::Run::Encoded(script) => Cow::Owned(script),
        cli::Run::File(_) | cli::Run::Unreadable | cli::Run::Interactive => return None,
    };
    let commands = try_parse_powershell_command_words(&script)?;
    if allowlist_reads_in_part(shell, invocation.no_profile, &commands) {
        return None;
    }
    Some((script, word_texts(commands)))
}

/// Parse the script body from a top-level PowerShell wrapper into argv-like commands.
///
/// This only unwraps the script of a PowerShell invocation we already
/// recognize (every argument after `-Command`, `-EncodedCommand` decoded),
/// then lowers the script in-process.
pub fn parse_powershell_command_into_plain_commands(
    command: &[String],
) -> Option<Vec<Vec<String>>> {
    let (_, script) = extract_powershell_command(command)?;
    try_parse_powershell_commands(&script)
}

/// Parse literal PowerShell commands without starting a PowerShell executable.
///
/// Unknown or dynamic syntax stays opaque so unfamiliar executables can be
/// evaluated without running them before approval.
pub fn parse_powershell_script_into_plain_commands(script: &str) -> Option<Vec<Vec<String>>> {
    try_parse_powershell_commands(script)
}

/// This function attempts to find a powershell.exe executable on the system.
pub fn try_find_powershell_executable_blocking() -> Option<AbsolutePathBuf> {
    try_find_powershellish_executable_in_path(&["powershell.exe"])
}

/// This function attempts to find a pwsh.exe executable on the system.
/// Note that pwsh.exe and powershell.exe are different executables:
///
/// - pwsh.exe is the cross-platform PowerShell Core (v6+) executable
/// - powershell.exe is the Windows PowerShell (v5.1 and earlier) executable
///
/// Further, while powershell.exe is included by default on Windows systems,
/// pwsh.exe must be installed separately by the user. And even when the user
/// has installed pwsh.exe, it may not be available in the system PATH, in which
/// case we attempt to locate it via other means.
pub fn try_find_pwsh_executable_blocking() -> Option<AbsolutePathBuf> {
    if let Some(ps_home) = std::process::Command::new("cmd")
        .args(["/C", "pwsh", "-NoProfile", "-Command", "$PSHOME"])
        .output()
        .ok()
        .and_then(|out| {
            if !out.status.success() {
                return None;
            }
            let stdout = String::from_utf8_lossy(&out.stdout);
            let trimmed = stdout.trim();
            (!trimmed.is_empty()).then(|| trimmed.to_string())
        })
    {
        let candidate = AbsolutePathBuf::resolve_path_against_base("pwsh.exe", &ps_home);

        if is_powershellish_executable_available(candidate.as_path()) {
            return Some(candidate);
        }
    }

    try_find_powershellish_executable_in_path(&["pwsh.exe"])
}

fn try_find_powershellish_executable_in_path(candidates: &[&str]) -> Option<AbsolutePathBuf> {
    for candidate in candidates {
        let Ok(resolved_path) = which::which(candidate) else {
            continue;
        };

        if !is_powershellish_executable_available(&resolved_path) {
            continue;
        }

        let Ok(abs_path) = AbsolutePathBuf::from_absolute_path(resolved_path) else {
            continue;
        };

        return Some(abs_path);
    }

    None
}

fn is_powershellish_executable_available(powershell_or_pwsh_exe: &std::path::Path) -> bool {
    // This test works for both powershell.exe and pwsh.exe.
    std::process::Command::new(powershell_or_pwsh_exe)
        .args(["-NoLogo", "-NoProfile", "-Command", "Write-Output ok"])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

#[cfg(test)]
#[cfg(windows)]
#[path = "powershell/module_hardening_tests.rs"]
mod module_hardening_tests;

#[cfg(test)]
mod tests {
    use super::AGENT_SCRIPT_PREFIX;
    use super::MODULE_HARDENING_PREFIX;
    use super::UTF8_OUTPUT_PREFIX;
    use super::extract_powershell_command;
    #[cfg(windows)]
    use super::parse_powershell_command_into_plain_commands;
    use super::parse_powershell_script_into_plain_commands;
    use super::prefix_powershell_script_with_utf8;

    #[test]
    fn extracts_basic_powershell_command() {
        let cmd = vec![
            "powershell".to_string(),
            "-Command".to_string(),
            "Write-Host hi".to_string(),
        ];
        let (_shell, script) = extract_powershell_command(&cmd).expect("extract");
        assert_eq!(script, "Write-Host hi");
    }

    #[test]
    fn extracts_lowercase_flags() {
        let cmd = vec![
            "powershell".to_string(),
            "-nologo".to_string(),
            "-command".to_string(),
            "Write-Host hi".to_string(),
        ];
        let (_shell, script) = extract_powershell_command(&cmd).expect("extract");
        assert_eq!(script, "Write-Host hi");
    }

    #[test]
    fn extracts_full_path_powershell_command() {
        let command = if cfg!(windows) {
            "C:\\windows\\System32\\WindowsPowerShell\\v1.0\\powershell.exe".to_string()
        } else {
            "/usr/local/bin/powershell.exe".to_string()
        };
        let cmd = vec![command, "-Command".to_string(), "Write-Host hi".to_string()];
        let (_shell, script) = extract_powershell_command(&cmd).expect("extract");
        assert_eq!(script, "Write-Host hi");
    }

    #[test]
    fn extracts_with_noprofile_and_alias() {
        let cmd = vec![
            "pwsh".to_string(),
            "-NoProfile".to_string(),
            "-c".to_string(),
            "Get-ChildItem | Select-String foo".to_string(),
        ];
        let (_shell, script) = extract_powershell_command(&cmd).expect("extract");
        assert_eq!(script, "Get-ChildItem | Select-String foo");
    }

    #[test]
    fn parses_powershell_command_chains_and_preserves_windows_paths() {
        assert_eq!(
            parse_powershell_script_into_plain_commands(r"echo safe && Remove-Item C:\important"),
            Some(vec![
                vec!["echo".to_string(), "safe".to_string()],
                vec!["Remove-Item".to_string(), r"C:\important".to_string()],
            ]),
        );
    }

    #[test]
    fn prefixes_powershell_command_with_best_effort_utf8() {
        let cmd = vec![
            "powershell".to_string(),
            "-Command".to_string(),
            "Write-Host hi".to_string(),
        ];

        let prefixed = prefix_powershell_script_with_utf8(&cmd);

        assert_eq!(
            prefixed,
            vec![
                "powershell".to_string(),
                "-Command".to_string(),
                format!("{AGENT_SCRIPT_PREFIX}Write-Host hi"),
            ]
        );
    }

    #[test]
    fn agent_prefix_is_module_hardening_then_utf8_output() {
        assert_eq!(
            AGENT_SCRIPT_PREFIX,
            format!("{MODULE_HARDENING_PREFIX}{UTF8_OUTPUT_PREFIX}")
        );
    }

    #[test]
    fn does_not_duplicate_utf8_prefix() {
        let cmd = vec![
            "powershell".to_string(),
            "-Command".to_string(),
            format!("{AGENT_SCRIPT_PREFIX}Write-Host hi"),
        ];

        assert_eq!(prefix_powershell_script_with_utf8(&cmd), cmd);
    }

    fn words(line: &[&str]) -> Vec<String> {
        line.iter().map(ToString::to_string).collect()
    }

    #[test]
    fn powershell_kind_follows_windows_name_rules() {
        use super::PowershellKind;
        for (shell, kind) in [
            ("pwsh", PowershellKind::Pwsh),
            ("pwsh.exe", PowershellKind::Pwsh),
            ("PWSH.EXE", PowershellKind::Pwsh),
            ("pwsh.exe.", PowershellKind::Pwsh),
            ("pwsh.exe. ", PowershellKind::Pwsh),
            ("powershell", PowershellKind::WindowsPowershell),
            ("powershell.exe.", PowershellKind::WindowsPowershell),
        ] {
            assert_eq!(PowershellKind::of(shell), kind, "{shell:?}");
        }
    }

    #[test]
    fn prefix_keeps_every_script_argument() {
        // #2568: the trailing arguments stay in the script PowerShell runs,
        // joined as PowerShell joins them.
        assert_eq!(
            prefix_powershell_script_with_utf8(&words(&[
                "pwsh",
                "-NoProfile",
                "-Command",
                "git status",
                "; git push",
            ])),
            words(&[
                "pwsh",
                "-NoProfile",
                "-Command",
                &format!("{AGENT_SCRIPT_PREFIX}git status ; git push"),
            ])
        );
        for line in [
            &["pwsh", "-EncodedCommand", "ZwBpAHQA"][..],
            &["pwsh", "-File", "x.ps1", "-c", "y"],
            &["pwsh", "-NoExit", "-Command", "git status"],
            &["pwsh"],
            &["bash", "-c", "git status"],
        ] {
            let command = words(line);
            assert_eq!(prefix_powershell_script_with_utf8(&command), command);
        }
    }

    #[test]
    fn extracts_the_script_powershell_runs() {
        let script = |line: &[&str]| {
            extract_powershell_command(&words(line)).map(|(_, script)| script.into_owned())
        };
        assert_eq!(
            script(&["powershell.exe", "-Command", "git status", "; git push"]).as_deref(),
            Some("git status ; git push")
        );
        assert_eq!(
            script(&["pwsh", "-NonInteractive", "-c", "git push"]).as_deref(),
            Some("git push")
        );
        assert_eq!(
            script(&["pwsh", "-EncodedCommand", "ZwBpAHQA"]).as_deref(),
            Some("git")
        );
        assert_eq!(script(&["pwsh", "-NoExit", "-c", "git push"]), None);
        assert_eq!(script(&["pwsh", "-File", "x.ps1"]), None);
        assert_eq!(script(&["bash", "-c", "git push"]), None);

        let plain = |line: &[&str]| {
            super::extract_plain_powershell_command(&words(line))
                .map(|(_, script)| script.into_owned())
        };
        assert_eq!(
            plain(&["pwsh", "-NoProfile", "-c", "git", "push"]).as_deref(),
            Some("git push")
        );
        assert_eq!(plain(&["pwsh", "-ep", "Bypass", "-c", "git push"]), None);
        assert_eq!(plain(&["pwsh", "-ec", "ZwBpAHQA"]), None);
    }

    #[test]
    fn reports_unread_command_lines() {
        use super::UnreadPowershell;
        use super::unread_powershell_command_line;
        for line in [
            &["pwsh", "-NoExit", "-c", "git push"][..],
            &["powershell.exe", "-Version", "2", "-Command", "git push"],
            &["powershell.exe", "git", "push"],
            &["pwsh", "-Command", "-"],
            &["pwsh", "-ec", "!!!"],
            &["pwsh", "-cwa", "$args", "x"],
            &["pwsh", "-nop"],
        ] {
            assert_eq!(
                unread_powershell_command_line(&words(line)),
                Some(UnreadPowershell::Unreadable),
                "{line:?}"
            );
        }
        let file = words(&["pwsh", "-File", "x.ps1", "a"]);
        assert_eq!(
            unread_powershell_command_line(&file),
            Some(UnreadPowershell::File(&file[2..]))
        );
        for line in [
            &["pwsh", "-c", "git push"][..],
            &["pwsh", "-ec", "ZwBpAHQA"],
            &["git", "push"],
        ] {
            assert_eq!(
                unread_powershell_command_line(&words(line)),
                None,
                "{line:?}"
            );
        }
    }

    #[cfg(windows)]
    #[test]
    fn parses_plain_powershell_commands() {
        let commands = parse_powershell_command_into_plain_commands(&[
            "powershell.exe".to_string(),
            "-NoProfile".to_string(),
            "-Command".to_string(),
            "echo hi".to_string(),
        ])
        .expect("parse");

        assert_eq!(commands, vec![vec!["echo".to_string(), "hi".to_string()]]);
    }

    #[cfg(windows)]
    #[test]
    fn parses_multiple_plain_powershell_commands() {
        let commands = parse_powershell_command_into_plain_commands(&[
            "powershell.exe".to_string(),
            "-NoProfile".to_string(),
            "-Command".to_string(),
            "Write-Output foo | Measure-Object".to_string(),
        ])
        .expect("parse");

        assert_eq!(
            commands,
            vec![
                vec!["Write-Output".to_string(), "foo".to_string()],
                vec!["Measure-Object".to_string()],
            ]
        );
    }
}
