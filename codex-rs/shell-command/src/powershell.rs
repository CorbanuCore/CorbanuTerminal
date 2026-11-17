use std::path::PathBuf;
use std::sync::OnceLock;

use codex_utils_absolute_path::AbsolutePathBuf;

use crate::command_safety::PowershellParseOutcome;
use crate::command_safety::parse_with_powershell_ast;
use crate::shell_detect::ShellType;
use crate::shell_detect::detect_shell_type;

const POWERSHELL_FLAGS: &[&str] = &["-nologo", "-noprofile", "-command", "-c"];

/// Prefixed command for powershell shell calls to request UTF-8 console output.
pub const UTF8_OUTPUT_PREFIX: &str =
    "try { [Console]::OutputEncoding=[System.Text.Encoding]::UTF8 } catch {}\n";

pub fn prefix_powershell_script_with_utf8(command: &[String]) -> Vec<String> {
    let Some((_, script)) = extract_powershell_command(command) else {
        return command.to_vec();
    };

    let trimmed = script.trim_start();
    let script = if trimmed.starts_with(UTF8_OUTPUT_PREFIX) {
        script.to_string()
    } else {
        format!("{UTF8_OUTPUT_PREFIX}{script}")
    };

    let mut command: Vec<String> = command[..(command.len() - 1)]
        .iter()
        .map(std::string::ToString::to_string)
        .collect();
    command.push(script);
    command
}

/// Extract the PowerShell script body from an invocation such as:
///
/// - ["pwsh", "-NoProfile", "-Command", "Get-ChildItem -Recurse | Select-String foo"]
/// - ["powershell.exe", "-Command", "Write-Host hi"]
/// - ["powershell", "-NoLogo", "-NoProfile", "-Command", "...script..."]
///
/// Returns (`shell`, `script`) when the first arg is a PowerShell executable and a
/// `-Command` (or `-c`) flag is present followed by a script string.
pub fn extract_powershell_command(command: &[String]) -> Option<(&str, &str)> {
    if command.len() < 3 {
        return None;
    }

    let shell = &command[0];
    if !matches!(
        detect_shell_type(PathBuf::from(shell)),
        Some(ShellType::PowerShell)
    ) {
        return None;
    }

    // Find the first occurrence of -Command (accept common short alias -c as well)
    let mut i = 1usize;
    while i + 1 < command.len() {
        let flag = &command[i];
        // Reject unknown flags
        if !POWERSHELL_FLAGS.contains(&flag.to_ascii_lowercase().as_str()) {
            return None;
        }
        if flag.eq_ignore_ascii_case("-Command") || flag.eq_ignore_ascii_case("-c") {
            let script = &command[i + 1];
            return Some((shell, script));
        }
        i += 1;
    }
    None
}

/// Parse the script body from a top-level PowerShell wrapper into argv-like commands.
///
/// This is intentionally narrower than the Windows safe-command parser: it only unwraps the
/// `-Command`/`-c` body from a PowerShell invocation we already recognize, then delegates the
/// script itself to the PowerShell AST parser.
pub fn parse_powershell_command_into_plain_commands(
    command: &[String],
) -> Option<Vec<Vec<String>>> {
    match read_powershell_command_script(command)? {
        PowershellScriptRead::Commands(commands) => Some(commands),
        PowershellScriptRead::NotPlain | PowershellScriptRead::Unreadable => None,
    }
}

/// What a trusted PowerShell reads of the script in a PowerShell `-Command`
/// invocation.
#[derive(Debug, PartialEq, Eq)]
pub enum PowershellScriptRead {
    /// The script's commands.
    Commands(Vec<Vec<String>>),
    /// The parser read the script but cannot list its commands (constructs
    /// it does not read, syntax errors).
    NotPlain,
    /// #2546 (D553): no trusted PowerShell could read the script: none was
    /// found, or it could not be run.
    Unreadable,
}

/// Read the script of a PowerShell `-Command` invocation (see
/// [`extract_powershell_command`]); `None` for any other command.
///
/// #2546 (D553): the script is parsed by a PowerShell found on the system
/// ([`trusted_powershell_parser`]), never by the invocation's own `argv[0]`:
/// that program may be any file named `pwsh` or `powershell`, and parsing
/// runs before approval, outside the sandbox.
pub fn read_powershell_command_script(command: &[String]) -> Option<PowershellScriptRead> {
    let (shell, script) = extract_powershell_command(command)?;
    Some(read_script_with_trusted_parser(shell, script))
}

/// Parse `script`, written for the PowerShell named `shell`, with the
/// trusted PowerShell of the same kind; `shell` itself is never started.
pub(crate) fn read_script_with_trusted_parser(shell: &str, script: &str) -> PowershellScriptRead {
    // REVERT CHECK ONLY (#2546): parse with the command's own argv[0], as main did.
    let parser = shell;
    match parse_with_powershell_ast(parser, script) {
        PowershellParseOutcome::Commands(commands) => PowershellScriptRead::Commands(commands),
        PowershellParseOutcome::Unsupported | PowershellParseOutcome::Failed => {
            PowershellScriptRead::NotPlain
        }
        PowershellParseOutcome::Unavailable => PowershellScriptRead::Unreadable,
    }
}

/// Which PowerShell a script is written for: `pwsh` (PowerShell 7) and
/// `powershell` (Windows PowerShell 5.1) read different languages, so each
/// is parsed by its own kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PowershellKind {
    Pwsh,
    WindowsPowershell,
}

impl PowershellKind {
    /// The kind a PowerShell program name (`pwsh`, `C:\…\PowerShell.EXE`)
    /// runs.
    pub(crate) fn of(shell: &str) -> Self {
        let stem = std::path::Path::new(shell)
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or(shell);
        if stem.eq_ignore_ascii_case("pwsh") {
            Self::Pwsh
        } else {
            Self::WindowsPowershell
        }
    }
}

thread_local! {
    static NO_TRUSTED_POWERSHELL: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Test support: while the returned guard lives, this thread finds no
/// trusted PowerShell, so scripts read as [`PowershellScriptRead::Unreadable`].
/// It can only make policy stricter.
#[doc(hidden)]
pub fn hide_trusted_powershell_on_this_thread() -> HiddenTrustedPowershell {
    NO_TRUSTED_POWERSHELL.with(|hidden| hidden.set(true));
    HiddenTrustedPowershell(())
}

#[doc(hidden)]
pub struct HiddenTrustedPowershell(());

impl Drop for HiddenTrustedPowershell {
    fn drop(&mut self) {
        NO_TRUSTED_POWERSHELL.with(|hidden| hidden.set(false));
    }
}

/// #2546 (D553): the PowerShell that parses scripts of `kind`, found once:
/// its fixed system location (`%SystemRoot%\System32\WindowsPowerShell\v1.0`,
/// `%ProgramFiles%\PowerShell\7`), else the first on `PATH` outside the
/// current directory. Never a command's own `argv[0]`, and never through
/// `cmd`, which looks in the current directory first.
pub(crate) fn trusted_powershell_parser(kind: PowershellKind) -> Option<&'static std::path::Path> {
    static PWSH: OnceLock<Option<PathBuf>> = OnceLock::new();
    static WINDOWS_POWERSHELL: OnceLock<Option<PathBuf>> = OnceLock::new();
    if NO_TRUSTED_POWERSHELL.with(std::cell::Cell::get) {
        return None;
    }
    let found = match kind {
        PowershellKind::Pwsh => &PWSH,
        PowershellKind::WindowsPowershell => &WINDOWS_POWERSHELL,
    };
    found
        .get_or_init(|| find_trusted_powershell(kind))
        .as_deref()
}

fn find_trusted_powershell(kind: PowershellKind) -> Option<PathBuf> {
    let (fixed, name): (&[(&str, &str)], &str) = match kind {
        PowershellKind::Pwsh => (
            &[
                ("ProgramFiles", r"PowerShell\7\pwsh.exe"),
                ("ProgramW6432", r"PowerShell\7\pwsh.exe"),
            ],
            if cfg!(windows) { "pwsh.exe" } else { "pwsh" },
        ),
        PowershellKind::WindowsPowershell => (
            &[
                (
                    "SystemRoot",
                    r"System32\WindowsPowerShell\v1.0\powershell.exe",
                ),
                ("windir", r"System32\WindowsPowerShell\v1.0\powershell.exe"),
            ],
            if cfg!(windows) {
                "powershell.exe"
            } else {
                "powershell"
            },
        ),
    };
    if cfg!(windows) {
        for (variable, relative) in fixed {
            let Some(base) = std::env::var_os(variable).map(PathBuf::from) else {
                continue;
            };
            let candidate = base.join(relative);
            if base.is_absolute() && candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    let current_dir = std::env::current_dir().ok();
    which::which(name).ok().filter(|found| {
        found.is_absolute()
            && current_dir
                .as_deref()
                .is_none_or(|current| found.parent() != Some(current))
    })
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
mod tests {
    use super::UTF8_OUTPUT_PREFIX;
    use super::extract_powershell_command;
    #[cfg(windows)]
    use super::parse_powershell_command_into_plain_commands;
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
                format!("{UTF8_OUTPUT_PREFIX}Write-Host hi"),
            ]
        );
    }

    #[test]
    fn does_not_duplicate_utf8_prefix() {
        let cmd = vec![
            "powershell".to_string(),
            "-Command".to_string(),
            format!("{UTF8_OUTPUT_PREFIX}Write-Host hi"),
        ];

        assert_eq!(prefix_powershell_script_with_utf8(&cmd), cmd);
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
