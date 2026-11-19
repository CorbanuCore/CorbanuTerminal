use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::io::BufRead;
use std::io::BufReader;
use std::io::ErrorKind;
use std::io::Write;
use std::process::Child;
use std::process::ChildStdin;
use std::process::ChildStdout;
use std::process::Command;
use std::process::Stdio;
use std::sync::LazyLock;
use std::sync::Mutex;
use std::sync::PoisonError;

const POWERSHELL_PARSER_SCRIPT: &str = include_str!("powershell_parser.ps1");
/// D729 (A): the commands a script may hold and still be read in full.
const POWERSHELL_ALLOWLIST_SCRIPT: &str = include_str!("powershell_allowlist.ps1");

/// The command line starts this loader, which reads the parser (too long for a command
/// line) from the first line of stdin. Module autoloading is off and the system modules
/// it uses are loaded from PowerShell's own install folder, so no module from a folder
/// a command could write runs in the parser (D729, #2661).
const POWERSHELL_PARSER_LOADER: &str = r#"$ProgressPreference = 'SilentlyContinue'
$PSModuleAutoLoadingPreference = 'None'
foreach ($m in 'Microsoft.PowerShell.Utility', 'Microsoft.PowerShell.Management', 'Microsoft.PowerShell.Security', 'Microsoft.PowerShell.Archive', 'CimCmdlets') {
    $p = [System.IO.Path]::Combine($PSHOME, 'Modules', $m, $m + '.psd1')
    if ([System.IO.File]::Exists($p)) { Import-Module -Name $p -ErrorAction SilentlyContinue }
}
$utf8 = [System.Text.UTF8Encoding]::new($false)
$stdin = [System.IO.StreamReader]::new([Console]::OpenStandardInput(), $utf8, $false)
. ([scriptblock]::Create($utf8.GetString([Convert]::FromBase64String($stdin.ReadLine()))))
"#;

/// Cache one long-lived parser process per executable path so repeated safety checks reuse
/// PowerShell startup work while still consulting the real parser every time.
///
/// We keep the cache behind one mutex because each child process speaks a simple
/// request/response protocol over a single stdin/stdout pair, so callers targeting the same
/// executable must serialize access anyway.
///
/// #2546 (D553): `executable` is started to parse, before any approval and
/// outside the sandbox; callers pass a trusted PowerShell
/// ([`crate::powershell::trusted_powershell_parser`]), never a command's own
/// `argv[0]`.
///
/// D729 (A): `no_profile` says the shell runs with `-NoProfile`; otherwise a
/// script is read in full only where no profile script exists.
pub(crate) fn parse_with_powershell_ast(
    executable: &str,
    script: &str,
    no_profile: bool,
) -> PowershellParseOutcome {
    static PARSER_PROCESSES: LazyLock<Mutex<HashMap<String, PowershellParserProcess>>> =
        LazyLock::new(|| Mutex::new(HashMap::new()));

    let mut parser_processes = PARSER_PROCESSES
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    parse_with_cached_process(&mut parser_processes, executable, script, no_profile)
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum PowershellParseOutcome {
    Commands(Vec<Vec<String>>),
    /// D729 (A): commands that write `writes` and remove `removes` (literal
    /// paths, as written), with the folders PowerShell and Windows run code
    /// from (`protected`). Read in full only where every path stays inside
    /// the writable roots and outside `protected` (see `write_scope`).
    Writes {
        commands: Vec<Vec<String>>,
        writes: Vec<String>,
        removes: Vec<String>,
        protected: Vec<String>,
    },
    Partial(Vec<Vec<String>>),
    Unsupported,
    /// The parser answered that it could not parse the script.
    Failed,
    /// #2546: the parser could not be started or did not answer.
    Unavailable,
}

fn parse_with_cached_process(
    parser_processes: &mut HashMap<String, PowershellParserProcess>,
    executable: &str,
    script: &str,
    no_profile: bool,
) -> PowershellParseOutcome {
    // `powershell.exe` and `pwsh.exe` do not accept the same language surface, so each
    // executable keeps its own parser process and request stream.
    let parser_key = executable.to_string();
    for attempt in 0..=1 {
        if !parser_processes.contains_key(&parser_key) {
            match PowershellParserProcess::spawn(executable) {
                Ok(process) => {
                    parser_processes.insert(parser_key.clone(), process);
                }
                Err(_) => return PowershellParseOutcome::Unavailable,
            }
        }

        let Some(parser_process) = parser_processes.get_mut(&parser_key) else {
            return PowershellParseOutcome::Unavailable;
        };
        let parse_result = parser_process.parse_request(script, no_profile);
        match parse_result {
            Ok(outcome) => return outcome,
            Err(_) if attempt == 0 => {
                // The common failure mode here is that a previously cached child exited or its
                // stdio stream became unusable between requests. Drop that process and retry once
                // with a fresh child before giving up.
                parser_processes.remove(&parser_key);
            }
            Err(_) => return PowershellParseOutcome::Unavailable,
        }
    }

    PowershellParseOutcome::Unavailable
}

fn encode_powershell_base64(script: &str) -> String {
    let mut utf16 = Vec::with_capacity(script.len() * 2);
    for unit in script.encode_utf16() {
        utf16.extend_from_slice(&unit.to_le_bytes());
    }
    BASE64_STANDARD.encode(utf16)
}

fn encoded_parser_loader() -> &'static str {
    static ENCODED: LazyLock<String> =
        LazyLock::new(|| encode_powershell_base64(POWERSHELL_PARSER_LOADER));
    &ENCODED
}

/// The first line the loader reads: the allowlist, then the parser, as base64 UTF-8.
fn parser_script_line() -> &'static str {
    static LINE: LazyLock<String> = LazyLock::new(|| {
        let script = format!("{POWERSHELL_ALLOWLIST_SCRIPT}\n{POWERSHELL_PARSER_SCRIPT}");
        format!("{}\n", BASE64_STANDARD.encode(script))
    });
    &LINE
}

struct PowershellParserProcess {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    // Request ids are monotonic within one child process so the caller can detect protocol
    // desynchronization if stdout is contaminated or the child is unexpectedly replaced.
    next_request_id: u64,
}

impl PowershellParserProcess {
    fn spawn(executable: &str) -> std::io::Result<Self> {
        let mut child = Command::new(executable)
            .args([
                "-NoLogo",
                "-NoProfile",
                "-NonInteractive",
                "-EncodedCommand",
                encoded_parser_loader(),
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;
        let mut stdin = match take_child_stdin(&mut child) {
            Ok(stdin) => stdin,
            Err(error) => {
                kill_child(&mut child);
                return Err(error);
            }
        };
        let stdout = match take_child_stdout(&mut child) {
            Ok(stdout) => stdout,
            Err(error) => {
                kill_child(&mut child);
                return Err(error);
            }
        };
        if let Err(error) = stdin
            .write_all(parser_script_line().as_bytes())
            .and_then(|()| stdin.flush())
        {
            kill_child(&mut child);
            return Err(error);
        }
        Ok(Self {
            child,
            stdin,
            stdout,
            next_request_id: 0,
        })
    }

    #[cfg(all(test, windows))]
    fn parse(&mut self, script: &str) -> std::io::Result<PowershellParseOutcome> {
        self.parse_request(script, /*no_profile*/ true)
    }

    fn parse_request(
        &mut self,
        script: &str,
        no_profile: bool,
    ) -> std::io::Result<PowershellParseOutcome> {
        let request = PowershellParserRequest {
            id: self.next_request_id,
            payload: encode_powershell_base64(script),
            no_profile,
        };
        self.next_request_id = self.next_request_id.wrapping_add(1);
        let mut request_json = serialize_request(&request)?;
        request_json.push('\n');
        self.stdin.write_all(request_json.as_bytes())?;
        self.stdin.flush()?;

        let mut response_line = String::new();
        if self.stdout.read_line(&mut response_line)? == 0 {
            return Err(std::io::Error::new(
                ErrorKind::UnexpectedEof,
                "PowerShell parser closed stdout",
            ));
        }

        let response = deserialize_response(&response_line)?;
        // Requests are serialized today; the id still catches protocol desyncs if stdout is
        // contaminated or the child process is unexpectedly replaced mid-request. That turns an
        // ambiguous parser result into a hard failure so the caller can discard the cached child.
        if response.id != request.id {
            return Err(std::io::Error::new(
                ErrorKind::InvalidData,
                format!(
                    "PowerShell parser returned response id {} for request {}",
                    response.id, request.id
                ),
            ));
        }

        Ok(response.into_outcome())
    }
}

impl Drop for PowershellParserProcess {
    fn drop(&mut self) {
        kill_child(&mut self.child);
    }
}

fn take_child_stdin(child: &mut Child) -> std::io::Result<ChildStdin> {
    child.stdin.take().ok_or_else(|| {
        std::io::Error::new(
            ErrorKind::BrokenPipe,
            "PowerShell parser child did not expose stdin",
        )
    })
}

fn take_child_stdout(child: &mut Child) -> std::io::Result<BufReader<ChildStdout>> {
    child.stdout.take().map(BufReader::new).ok_or_else(|| {
        std::io::Error::new(
            ErrorKind::BrokenPipe,
            "PowerShell parser child did not expose stdout",
        )
    })
}

fn serialize_request(request: &PowershellParserRequest) -> std::io::Result<String> {
    serde_json::to_string(request).map_err(|error| {
        std::io::Error::new(
            ErrorKind::InvalidData,
            format!("failed to serialize PowerShell parser request: {error}"),
        )
    })
}

fn deserialize_response(response_line: &str) -> std::io::Result<PowershellParserResponse> {
    serde_json::from_str(response_line).map_err(|error| {
        std::io::Error::new(
            ErrorKind::InvalidData,
            format!("failed to parse PowerShell parser response: {error}"),
        )
    })
}

#[derive(Serialize)]
struct PowershellParserRequest {
    id: u64,
    payload: String,
    #[serde(rename = "noProfile")]
    no_profile: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PowershellParserResponse {
    id: u64,
    status: String,
    commands: Option<Vec<Vec<String>>>,
    #[serde(default)]
    writes: Option<Vec<String>>,
    #[serde(default)]
    removes: Option<Vec<String>>,
    #[serde(default)]
    protected: Option<Vec<String>>,
    #[serde(default)]
    programs: Option<Vec<String>>,
    #[serde(default)]
    cmdlets: Option<Vec<String>>,
    /// Why the script is read in part (diagnostics only).
    #[serde(default, rename = "cause")]
    _cause: Option<String>,
}

impl PowershellParserResponse {
    fn into_outcome(self) -> PowershellParseOutcome {
        match self.status.as_str() {
            "ok" => {
                let Some(commands) = self.commands.filter(|commands| {
                    !commands.is_empty()
                        && commands
                            .iter()
                            .all(|cmd| !cmd.is_empty() && cmd.iter().all(|word| !word.is_empty()))
                }) else {
                    return PowershellParseOutcome::Unsupported;
                };
                // D729 (A): each program found on the command's `PATH`, no
                // module cmdlet shadowed there; else read in part.
                if !crate::powershell::names_resolve(
                    &self.programs.unwrap_or_default(),
                    &self.cmdlets.unwrap_or_default(),
                ) {
                    return PowershellParseOutcome::Partial(commands);
                }
                let writes = self.writes.unwrap_or_default();
                let removes = self.removes.unwrap_or_default();
                if writes.is_empty() && removes.is_empty() {
                    PowershellParseOutcome::Commands(commands)
                } else {
                    PowershellParseOutcome::Writes {
                        commands,
                        writes,
                        removes,
                        protected: self.protected.unwrap_or_default(),
                    }
                }
            }
            "partial" => self
                .commands
                .filter(|commands| commands.iter().all(|command| !command.is_empty()))
                .map(PowershellParseOutcome::Partial)
                .unwrap_or(PowershellParseOutcome::Unsupported),
            "unsupported" => PowershellParseOutcome::Unsupported,
            _ => PowershellParseOutcome::Failed,
        }
    }
}

fn kill_child(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;
    use crate::powershell::try_find_powershell_executable_blocking;
    use pretty_assertions::assert_eq;

    #[test]
    fn parser_process_handles_multiple_requests() {
        let Some(powershell) = try_find_powershell_executable_blocking() else {
            return;
        };
        let powershell = powershell.as_path().to_str().unwrap();
        let mut parser = PowershellParserProcess::spawn(powershell).unwrap();

        let first = parser.parse("Get-Content 'foo bar'").unwrap();
        assert_eq!(
            first,
            PowershellParseOutcome::Commands(vec![vec![
                "Get-Content".to_string(),
                "foo bar".to_string(),
            ]]),
        );

        let second = parser.parse("Write-Output foo | Measure-Object").unwrap();
        assert_eq!(
            second,
            PowershellParseOutcome::Commands(vec![
                vec!["Write-Output".to_string(), "foo".to_string()],
                vec!["Measure-Object".to_string()],
            ]),
        );
    }

    #[test]
    fn parser_process_lists_writes_and_reads_unlisted_commands_in_part() {
        let Some(powershell) = try_find_powershell_executable_blocking() else {
            return;
        };
        let powershell = powershell.as_path().to_str().unwrap();
        let mut parser = PowershellParserProcess::spawn(powershell).unwrap();

        let PowershellParseOutcome::Writes {
            commands,
            writes,
            removes,
            protected,
        } = parser
            .parse("New-Item -Path build -Name notes.txt -Value x; Rename-Item a.txt b.txt")
            .unwrap()
        else {
            panic!("writes");
        };
        assert_eq!(commands.len(), 2);
        assert_eq!(
            writes,
            vec![
                "build\\notes.txt".to_string(),
                "a.txt\\..\\b.txt".to_string(),
            ]
        );
        assert_eq!(removes, vec!["a.txt".to_string()]);
        assert!(!protected.is_empty());

        // D729 (A): not on the allowlist, so read in part.
        assert_eq!(
            parser.parse("Get-Command git; git status").unwrap(),
            PowershellParseOutcome::Partial(vec![
                vec!["Get-Command".into(), "git".into()],
                vec!["git".into(), "status".into()],
            ])
        );
        assert_eq!(
            parser
                .parse("Get-ChildItem -File -Recurse | Select-Object -First 3")
                .unwrap(),
            PowershellParseOutcome::Commands(vec![
                vec!["Get-ChildItem".into(), "-File".into(), "-Recurse".into()],
                vec!["Select-Object".into(), "-First".into(), "3".into()],
            ])
        );
    }

    #[test]
    fn parser_process_rejects_stop_parsing_forms() {
        let Some(powershell) = try_find_powershell_executable_blocking() else {
            return;
        };
        let powershell = powershell.as_path().to_str().unwrap();
        let mut parser = PowershellParserProcess::spawn(powershell).unwrap();

        let parsed = parser
            .parse("git log --% HEAD --output=codex_poc.txt")
            .unwrap();
        assert_eq!(
            parsed,
            PowershellParseOutcome::Partial(vec![vec!["git".into(), "log".into()],])
        );
    }

    #[test]
    fn parser_process_rejects_param_blocks() {
        let Some(powershell) = try_find_powershell_executable_blocking() else {
            return;
        };
        let powershell = powershell.as_path().to_str().unwrap();
        let mut parser = PowershellParserProcess::spawn(powershell).unwrap();

        let parsed = parser
            .parse("param([string]$path = (Get-Location)) Write-Output test")
            .unwrap();
        assert_eq!(
            parsed,
            PowershellParseOutcome::Partial(vec![
                vec!["Get-Location".into()],
                vec!["Write-Output".into(), "test".into()],
            ])
        );
    }

    #[test]
    fn parser_process_rejects_named_blocks() {
        let Some(powershell) = try_find_powershell_executable_blocking() else {
            return;
        };
        let powershell = powershell.as_path().to_str().unwrap();
        let mut parser = PowershellParserProcess::spawn(powershell).unwrap();

        let parsed = parser
            .parse("begin { Set-Content codex_poc.txt pwned } end { Get-Content Cargo.toml }")
            .unwrap();
        assert_eq!(
            parsed,
            PowershellParseOutcome::Partial(vec![
                vec!["Set-Content".into(), "codex_poc.txt".into(), "pwned".into()],
                vec!["Get-Content".into(), "Cargo.toml".into()],
            ])
        );
    }

    #[test]
    fn parser_process_rejects_using_statements() {
        let Some(powershell) = try_find_powershell_executable_blocking() else {
            return;
        };
        let powershell = powershell.as_path().to_str().unwrap();
        let mut parser = PowershellParserProcess::spawn(powershell).unwrap();

        let parsed = parser
            .parse("using module ./codex_poc.psm1\nGet-Content Cargo.toml")
            .unwrap();
        assert_eq!(
            parsed,
            PowershellParseOutcome::Partial(vec![vec!["Get-Content".into(), "Cargo.toml".into()],])
        );
    }

    #[test]
    fn parser_process_rejects_trap_blocks() {
        let Some(powershell) = try_find_powershell_executable_blocking() else {
            return;
        };
        let powershell = powershell.as_path().to_str().unwrap();
        let mut parser = PowershellParserProcess::spawn(powershell).unwrap();

        let parsed = parser
            .parse(
                "trap { Set-Content codex_poc.txt pwned; continue } Get-Content missing -ErrorAction Stop",
            )
            .unwrap();
        assert_eq!(
            parsed,
            PowershellParseOutcome::Partial(vec![
                vec!["Set-Content".into(), "codex_poc.txt".into(), "pwned".into()],
                vec![
                    "Get-Content".into(),
                    "missing".into(),
                    "-ErrorAction".into(),
                    "Stop".into()
                ],
            ])
        );
    }
}
