use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::OnceLock;

use codex_utils_absolute_path::AbsolutePathBuf;

use crate::command_safety::PowershellParseOutcome;
use crate::command_safety::parse_with_powershell_ast;
use crate::shell_detect::ShellType;
use crate::shell_detect::detect_shell_type;

mod cli;
mod write_scope;

pub use write_scope::PowershellWriteScope;
pub use write_scope::PowershellWriteScopeGuard;
pub use write_scope::enter_powershell_write_scope;
pub(crate) use write_scope::names_resolve;

/// Prefixed command for powershell shell calls to request UTF-8 console output.
pub const UTF8_OUTPUT_PREFIX: &str =
    "try { [Console]::OutputEncoding=[System.Text.Encoding]::UTF8 } catch {}\n";

/// `command` with [`UTF8_OUTPUT_PREFIX`] before its `-Command` script, which
/// becomes one argument (PowerShell joins a script's arguments with spaces
/// anyway). Any other command is returned as it is.
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

    let script = if script.trim_start().starts_with(UTF8_OUTPUT_PREFIX) {
        script.into_owned()
    } else {
        format!("{UTF8_OUTPUT_PREFIX}{script}")
    };

    let mut prefixed = command[..start].to_vec();
    prefixed.push(script);
    prefixed
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
/// [`read_powershell_command_script`]).
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
        PowershellScriptRead::NotPlain
        | PowershellScriptRead::Partial(_)
        | PowershellScriptRead::File(_)
        | PowershellScriptRead::Unreadable => None,
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
    /// Literal commands or prefixes recovered from an otherwise unread
    /// script. These may only add restrictions, never authorize the script.
    Partial(Vec<Vec<String>>),
    /// `-File`: the script file and its arguments, which rules see as a
    /// command; the file's content is not read (#2568).
    File(Vec<String>),
    /// No trusted PowerShell could read the script: none was found, or it
    /// could not be run (#2546, D553). Or the command line cannot be read
    /// the way PowerShell reads it (#2568): an unknown or unsupported
    /// parameter, a script from stdin, `-CommandWithArgs` with arguments, an
    /// `-EncodedCommand` that does not decode.
    Unreadable,
}

/// Read the script of a PowerShell invocation (see
/// [`extract_powershell_command`]); `None` for any other command. A shell
/// with no script argument can read stdin, so its script is unreadable.
///
/// #2546 (D553): the script is parsed by a PowerShell found on the system
/// ([`trusted_powershell_parser`]), never by the invocation's own `argv[0]`:
/// that program may be any file named `pwsh` or `powershell`, and parsing
/// runs before approval, outside the sandbox.
pub fn read_powershell_command_script(command: &[String]) -> Option<PowershellScriptRead> {
    let (shell, invocation) = read_invocation(command)?;
    let shell_options = ShellOptions {
        no_profile: invocation.no_profile,
        keeps_cwd: !invocation.working_directory,
    };
    match invocation.run {
        cli::Run::Command { script, .. } => Some(read_script_in(shell, &script, shell_options)),
        cli::Run::Encoded(script) => Some(read_script_in(shell, &script, shell_options)),
        cli::Run::File(words) => Some(PowershellScriptRead::File(words.to_vec())),
        cli::Run::Unreadable => Some(PowershellScriptRead::Unreadable),
        cli::Run::Interactive => Some(PowershellScriptRead::Unreadable),
    }
}

/// D729 (A): how the shell running a script starts.
#[derive(Clone, Copy)]
struct ShellOptions {
    /// `-NoProfile`: no profile script runs before the script.
    no_profile: bool,
    /// No `-WorkingDirectory`: the script runs in the command's folder.
    keeps_cwd: bool,
}

/// Parse `script`, written for the PowerShell named `shell`, with the
/// trusted PowerShell of the same kind; `shell` itself is never started.
fn read_script_in(shell: &str, script: &str, options: ShellOptions) -> PowershellScriptRead {
    let Some(parser) =
        trusted_powershell_parser(PowershellKind::of(shell)).and_then(|parser| parser.to_str())
    else {
        return PowershellScriptRead::Unreadable;
    };
    match parse_with_powershell_ast(parser, script, options.no_profile) {
        PowershellParseOutcome::Commands(commands) => PowershellScriptRead::Commands(commands),
        // D729 (A): a script that writes is read in full only where its writes stay
        // where the current scope lets them (`write_scope`); otherwise in part.
        PowershellParseOutcome::Writes {
            commands,
            writes,
            removes,
            protected,
        } => {
            // `-WorkingDirectory` runs it elsewhere than the scope's folder.
            if options.keeps_cwd && write_scope::writes_allowed(&writes, &removes, &protected) {
                PowershellScriptRead::Commands(commands)
            } else {
                PowershellScriptRead::Partial(commands)
            }
        }
        PowershellParseOutcome::Partial(commands) => PowershellScriptRead::Partial(commands),
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
    let current_dir = std::env::current_dir().ok();
    find_trusted_powershell_in(
        kind,
        &|variable| std::env::var_os(variable),
        current_dir.as_deref(),
    )
}

/// The file name a PowerShell of `kind` has on `PATH`.
fn powershell_file_name(kind: PowershellKind) -> &'static str {
    match (kind, cfg!(windows)) {
        (PowershellKind::Pwsh, true) => "pwsh.exe",
        (PowershellKind::Pwsh, false) => "pwsh",
        (PowershellKind::WindowsPowershell, true) => "powershell.exe",
        (PowershellKind::WindowsPowershell, false) => "powershell",
    }
}

/// [`find_trusted_powershell`] with the environment and current directory
/// given.
fn find_trusted_powershell_in(
    kind: PowershellKind,
    env: &dyn Fn(&str) -> Option<std::ffi::OsString>,
    current_dir: Option<&std::path::Path>,
) -> Option<PathBuf> {
    if cfg!(windows)
        && let Some(found) = fixed_powershell_paths(kind, env)
            .into_iter()
            .find(|candidate| candidate.is_file())
    {
        return Some(found);
    }
    trusted_on_path(powershell_file_name(kind), &env("PATH")?, current_dir?)
}

/// #2568: the first `name` in the absolute directories of `path` (a `PATH`
/// value) that is not in `current_dir` or under it (`.venv\Scripts`,
/// `node_modules\.bin`), where a command may write. Compared canonical, so
/// the current directory spelled in another case, by its 8.3 short name, or
/// through a junction or link is excluded too, and so is a link to a file
/// in it. `None` where the current directory cannot be resolved.
fn trusted_on_path(
    name: &str,
    path: &std::ffi::OsStr,
    current_dir: &std::path::Path,
) -> Option<PathBuf> {
    let current_dir = current_dir.canonicalize().ok()?;
    let in_current_dir = |path: &std::path::Path| {
        path.canonicalize()
            .ok()
            .is_none_or(|canonical| canonical.starts_with(&current_dir))
    };
    std::env::split_paths(path).find_map(|dir| {
        // A relative entry names a directory under the current one.
        if !dir.is_absolute() || in_current_dir(&dir) {
            return None;
        }
        let found = which::which_in(name, Some(&dir), &dir).ok()?;
        let file_dir = found.canonicalize().ok()?.parent()?.to_path_buf();
        (!file_dir.starts_with(&current_dir) && !in_current_dir(found.parent()?)).then_some(found)
    })
}

/// #2546, #2568: where Windows installs a PowerShell of `kind`: under
/// `%SystemRoot%` (`%windir%`) or `%ProgramFiles%` (`%ProgramW6432%`) where
/// those are local absolute paths, then the default locations.
fn fixed_powershell_paths(
    kind: PowershellKind,
    env: &dyn Fn(&str) -> Option<std::ffi::OsString>,
) -> Vec<PathBuf> {
    let (variables, relative, default): (&[&str], &str, &str) = match kind {
        PowershellKind::Pwsh => (
            &["ProgramFiles", "ProgramW6432"],
            r"PowerShell\7\pwsh.exe",
            r"C:\Program Files\PowerShell\7\pwsh.exe",
        ),
        PowershellKind::WindowsPowershell => (
            &["SystemRoot", "windir"],
            r"System32\WindowsPowerShell\v1.0\powershell.exe",
            r"C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe",
        ),
    };
    variables
        .iter()
        .filter_map(|variable| env(variable).map(PathBuf::from))
        .filter(|base| is_local_absolute(base))
        .map(|base| base.join(relative))
        .chain(std::iter::once(PathBuf::from(default)))
        .collect()
}

/// #2568: whether `path` is a plain local absolute path (`C:\Windows`): not
/// a UNC (`\\server\share`) or device (`\\?\`, `\\.\`) path, and with no
/// `.` or `..` component.
fn is_local_absolute(path: &std::path::Path) -> bool {
    use std::path::Component;
    use std::path::Prefix;
    let mut components = path.components();
    matches!(
        components.next(),
        Some(Component::Prefix(prefix)) if matches!(prefix.kind(), Prefix::Disk(_))
    ) && matches!(components.next(), Some(Component::RootDir))
        && components.all(|component| matches!(component, Component::Normal(_)))
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
#[path = "powershell/trusted_path_tests.rs"]
mod trusted_path_tests;

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
            ("pwsh. ", PowershellKind::Pwsh),
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
                &format!("{UTF8_OUTPUT_PREFIX}git status ; git push"),
            ])
        );
        // Anything else runs as given.
        for line in [
            &["pwsh", "-EncodedCommand", "ZwBpAHQA"][..],
            &["pwsh", "-File", "x.ps1", "-c", "y"],
            &["pwsh", "-NoExit", "-Command", "git status"],
            &["pwsh", "-cwa", "$args", "x"],
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
    fn reads_unreadable_command_lines_as_unreadable() {
        use super::PowershellScriptRead;
        use super::read_powershell_command_script;
        for line in [
            &["pwsh", "-NoExit", "-c", "git push"][..],
            &["powershell.exe", "-Version", "2", "-Command", "git push"],
            &["powershell.exe", "git", "push"],
            &["pwsh", "-Command", "-"],
            &["pwsh", "-ec", "!!!"],
            &["pwsh", "-cwa", "$args", "x"],
        ] {
            assert_eq!(
                read_powershell_command_script(&words(line)),
                Some(PowershellScriptRead::Unreadable),
                "{line:?}"
            );
        }
        assert_eq!(
            read_powershell_command_script(&words(&["pwsh", "-File", "x.ps1", "a"])),
            Some(PowershellScriptRead::File(words(&["x.ps1", "a"])))
        );
        assert_eq!(
            read_powershell_command_script(&words(&["pwsh", "-nop"])),
            Some(PowershellScriptRead::Unreadable)
        );
        assert_eq!(
            read_powershell_command_script(&words(&["git", "push"])),
            None
        );
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
