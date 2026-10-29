//! #1507: process-creation paths other than `CreateProcess` that could start
//! a legacy command's child outside the command's job, or under a token
//! other than its Low-integrity restricted one. #733 made the job refuse
//! breakaway; these routes don't ask the job.
//!
//! Each test runs a launcher in a legacy session that tries one route to
//! start [`launch_path_report_entry`], which writes where it ended up. A
//! route keeps containment when Windows refuses the launch or when the
//! report shows the child in the launcher's job, at Low integrity, with a
//! restricted token.
//!
//! Measured on the hosted Windows runner (elevated and in a normal session;
//! no Explorer running), #1507: WMI, the Task Scheduler and `ShellWindows`
//! refuse a legacy command with the errors their tests expect. Naming
//! another legacy session's process as the parent, `ShellBrowserWindow` and
//! `MMC20.Application` start the child outside the launcher's job; those
//! tests are ignored until #1507 decides how to close them.

use super::*;
use crate::proc_thread_attr::ProcThreadAttributeList;
use crate::winutil::quote_windows_arg;
use crate::winutil::to_wide;
use pretty_assertions::assert_eq;
use windows_sys::Win32::Foundation::BOOL;
use windows_sys::Win32::Foundation::CloseHandle;
use windows_sys::Win32::Foundation::GetLastError;
use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::Security::IsTokenRestricted;
use windows_sys::Win32::Security::TOKEN_QUERY;
use windows_sys::Win32::System::JobObjects::IsProcessInJob;
use windows_sys::Win32::System::JobObjects::JOBOBJECT_BASIC_PROCESS_ID_LIST;
use windows_sys::Win32::System::JobObjects::JobObjectBasicProcessIdList;
use windows_sys::Win32::System::JobObjects::QueryInformationJobObject;
use windows_sys::Win32::System::Threading::CREATE_NO_WINDOW;
use windows_sys::Win32::System::Threading::CreateProcessW;
use windows_sys::Win32::System::Threading::EXTENDED_STARTUPINFO_PRESENT;
use windows_sys::Win32::System::Threading::GetCurrentProcess;
use windows_sys::Win32::System::Threading::OpenProcessToken;
use windows_sys::Win32::System::Threading::PROCESS_CREATE_PROCESS;
use windows_sys::Win32::System::Threading::PROCESS_INFORMATION;
use windows_sys::Win32::System::Threading::PROCESS_QUERY_LIMITED_INFORMATION;
use windows_sys::Win32::System::Threading::STARTUPINFOEXW;
use windows_sys::Win32::System::Threading::STARTUPINFOW;

const REPORT_ENTRY: &str = "unified_exec::tests::launch_paths::launch_path_report_entry";
const LAUNCHER_ENTRY: &str = "unified_exec::tests::launch_paths::launch_path_launcher_entry";
const HOLD_ENTRY: &str = "unified_exec::tests::launch_paths::launch_path_hold_entry";
/// The report entry gets its settings as extra libtest filters, because
/// brokers such as WMI start it without the caller's environment.
const REPORT_ARG: &str = "codex-1507-report=";
const LAUNCHER_ARG: &str = "codex-1507-launcher=";
/// Replaced with the launcher's process id inside the session.
const LAUNCHER_PLACEHOLDER: &str = "@LAUNCHER@";
const LAUNCH_ENV: &str = "CODEX_1507_LAUNCH";
const PARENT_ENV: &str = "CODEX_1507_PARENT";
const COMMAND_ENV: &str = "CODEX_1507_CMD";
const EXE_ENV: &str = "CODEX_1507_EXE";
const ARGS_ENV: &str = "CODEX_1507_ARGS";
const DIR_ENV: &str = "CODEX_1507_DIR";
const TASK_ENV: &str = "CODEX_1507_TASK";
const HOLD_ENV: &str = "CODEX_1507_HOLD";
const REPORT_ENV: &str = "CODEX_1507_REPORT";
const LAUNCH_RESULT: &str = "launch: ";
const NEW_PROCESSES: &str = "new: ";
/// The start of a report from a child that stayed in the session.
const CONTAINED: &str = "in_launcher_job=true in_any_job=true low=true restricted=true";

/// What the started child saw of itself.
fn own_containment(launcher: u32) -> String {
    // SAFETY: the pseudo handle needs no closing; the token handle is closed below.
    unsafe {
        let mut in_job: BOOL = 0;
        let in_any_job = IsProcessInJob(GetCurrentProcess(), 0, &mut in_job) != 0 && in_job != 0;
        // A null job handle queries the caller's innermost job.
        let mut buffer = vec![0usize; 1026];
        let in_launcher_job = QueryInformationJobObject(
            0,
            JobObjectBasicProcessIdList,
            buffer.as_mut_ptr().cast(),
            u32::try_from(buffer.len() * std::mem::size_of::<usize>()).unwrap_or(u32::MAX),
            std::ptr::null_mut(),
        ) != 0
            && {
                let list = &*(buffer.as_ptr() as *const JOBOBJECT_BASIC_PROCESS_ID_LIST);
                // The ids follow the two counts, at the second `usize`.
                let count = (list.NumberOfProcessIdsInList as usize).min(buffer.len() - 1);
                buffer[1..=count].contains(&(launcher as usize))
            };
        let (low, restricted) = token_state(GetCurrentProcess());
        format!(
            "in_launcher_job={in_launcher_job} in_any_job={in_any_job} low={low} restricted={restricted}"
        )
    }
}

/// Whether `process`'s token is Low integrity and restricted.
///
/// # Safety
/// `process` must be a process handle with query access.
unsafe fn token_state(process: HANDLE) -> (bool, bool) {
    let mut token: HANDLE = 0;
    // SAFETY: per the caller; the token handle is closed below.
    unsafe {
        if OpenProcessToken(process, TOKEN_QUERY, &mut token) == 0 {
            return (false, false);
        }
        let state = (
            crate::integrity::token_is_low_integrity(token),
            IsTokenRestricted(token) != 0,
        );
        CloseHandle(token);
        state
    }
}

/// Where process `pid` runs, seen from outside the sandbox.
fn inspect_pid(name: &str, pid: u32) -> String {
    // SAFETY: the handle is checked and closed here.
    unsafe {
        let process = windows_sys::Win32::System::Threading::OpenProcess(
            PROCESS_QUERY_LIMITED_INFORMATION,
            0,
            pid,
        );
        if process == 0 {
            return format!("{name}:{pid} gone");
        }
        let mut in_job: BOOL = 0;
        let in_any_job = IsProcessInJob(process, 0, &mut in_job) != 0 && in_job != 0;
        let (low, restricted) = token_state(process);
        CloseHandle(process);
        format!("{name}:{pid} in_any_job={in_any_job} low={low} restricted={restricted}")
    }
}

/// Writes [`own_containment`] to the report path in its arguments. Does
/// nothing otherwise.
#[test]
fn launch_path_report_entry() {
    let args: Vec<String> = std::env::args().collect();
    let Some(report) = args.iter().find_map(|arg| arg.strip_prefix(REPORT_ARG)) else {
        return;
    };
    let launcher = args
        .iter()
        .find_map(|arg| arg.strip_prefix(LAUNCHER_ARG))
        .and_then(|pid| pid.parse().ok())
        .unwrap_or(0);
    fs::write(report, own_containment(launcher)).expect("write launch report");
}

/// Waits (up to a minute) in a second legacy session, so a launcher can try
/// to start a child under it. Does nothing otherwise.
#[test]
fn launch_path_hold_entry() {
    let Some(dir) = std::env::var_os(HOLD_ENV).map(PathBuf::from) else {
        return;
    };
    fs::write(dir.join("hold.pid"), std::process::id().to_string()).expect("write hold pid");
    wait_for_path(&dir.join("hold.release"), Duration::from_secs(/*secs*/ 60));
}

/// Starts the report entry from inside a legacy session: as a plain child
/// (the control), or with `PROC_THREAD_ATTRIBUTE_PARENT_PROCESS` naming the
/// process in `CODEX_1507_PARENT`. Does nothing otherwise.
#[test]
fn launch_path_launcher_entry() {
    let Some(mode) = std::env::var_os(LAUNCH_ENV) else {
        return;
    };
    let command = std::env::var(COMMAND_ENV)
        .expect("report command")
        .replace(LAUNCHER_PLACEHOLDER, &std::process::id().to_string());
    let parent_pid: Option<u32> = (mode == "parent").then(|| {
        std::env::var(PARENT_ENV)
            .expect("parent pid")
            .parse()
            .expect("numeric parent pid")
    });
    // SAFETY: every pointer refers to a live local; handles are closed below.
    let outcome = unsafe {
        let mut attributes = None;
        let mut parent: HANDLE = 0;
        if let Some(pid) = parent_pid {
            parent =
                windows_sys::Win32::System::Threading::OpenProcess(PROCESS_CREATE_PROCESS, 0, pid);
            if parent == 0 {
                let error = GetLastError();
                println!("\n{LAUNCH_RESULT}started=false open_parent_error={error}");
                return;
            }
            let mut list = ProcThreadAttributeList::new(/*attr_count*/ 1).expect("attribute list");
            list.set_parent_process(parent).expect("parent attribute");
            attributes = Some(list);
        }
        let mut startup: STARTUPINFOEXW = std::mem::zeroed();
        startup.StartupInfo.cb = std::mem::size_of::<STARTUPINFOEXW>() as u32;
        let mut flags = CREATE_NO_WINDOW;
        if let Some(list) = attributes.as_mut() {
            startup.lpAttributeList = list.as_mut_ptr();
            flags |= EXTENDED_STARTUPINFO_PRESENT;
        } else {
            startup.StartupInfo.cb = std::mem::size_of::<STARTUPINFOW>() as u32;
        }
        let mut command_line = to_wide(&command);
        let mut info: PROCESS_INFORMATION = std::mem::zeroed();
        let started = CreateProcessW(
            std::ptr::null(),
            command_line.as_mut_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            /*binherithandles*/ 0,
            flags,
            std::ptr::null(),
            std::ptr::null(),
            std::ptr::addr_of!(startup).cast::<STARTUPINFOW>(),
            &mut info,
        ) != 0;
        let error = GetLastError();
        if parent != 0 {
            CloseHandle(parent);
        }
        if started {
            let process = OwnedHandle::from_raw_handle(info.hProcess as _);
            CloseHandle(info.hThread);
            let _ = wait_for_process_exit(&process, Duration::from_secs(/*secs*/ 30));
            "started=true".to_string()
        } else {
            format!("started=false create_error={error}")
        }
    };
    println!("\n{LAUNCH_RESULT}{outcome}");
}

/// The report entry's command line, with the launcher's id still a
/// placeholder.
fn report_args(report: &Path) -> String {
    [
        "--exact".to_string(),
        REPORT_ENTRY.to_string(),
        "--nocapture".to_string(),
        quote_windows_arg(&format!("{REPORT_ARG}{}", report.display())),
        format!("{LAUNCHER_ARG}{LAUNCHER_PLACEHOLDER}"),
    ]
    .join(" ")
}

fn test_exe() -> String {
    std::env::current_exe()
        .expect("test exe")
        .display()
        .to_string()
}

fn report_command(report: &Path) -> String {
    format!("{} {}", quote_windows_arg(&test_exe()), report_args(report))
}

/// One launch attempt: what the launcher said and what the child reported.
struct LaunchOutcome {
    launch: Option<String>,
    report: Option<String>,
    /// Processes the launch started that weren't there before, as seen
    /// from outside the sandbox (PowerShell launches only).
    new_processes: Vec<String>,
    stdout: String,
    log: String,
}

impl LaunchOutcome {
    /// Contained: refused with an error that contains every part of
    /// `refusal` (the step and error measured, so a broken probe doesn't pass
    /// as a refusal), or started in the launcher's job, Low and restricted.
    fn assert_contained(&self, route: &str, refusal: &[&str]) {
        let refused = self.launch.as_deref().is_some_and(|launch| {
            launch.starts_with("started=false") && refusal.iter().all(|part| launch.contains(part))
        });
        let contained = self
            .report
            .as_deref()
            .is_some_and(|report| report.starts_with(CONTAINED));
        println!(
            "\nlaunch-path {route}: launch={:?} report={:?} new={:?}",
            self.launch, self.report, self.new_processes
        );
        assert!(
            (refused && self.report.is_none()) || contained,
            "{route} left the session: launch={:?} report={:?} new={:?}\nstdout={:?}\n{}",
            self.launch,
            self.report,
            self.new_processes,
            self.stdout,
            self.log
        );
    }
}

/// Runs `argv` in a legacy session on `root` with `env` added to this
/// process's environment, then waits for the report the launch produced.
async fn run_launch(
    root: &Path,
    codex_home: &Path,
    argv: Vec<String>,
    env: Vec<(&str, String)>,
    report: &Path,
) -> LaunchOutcome {
    let mut env_map: HashMap<String, String> = std::env::vars().collect();
    for (key, value) in env {
        env_map.insert(key.to_string(), value);
    }
    let spawned = spawn_windows_sandbox_session_legacy(
        &PermissionProfile::workspace_write(),
        workspace_roots_for(root).as_slice(),
        codex_home,
        argv,
        root,
        env_map,
        /*timeout_ms*/ Some(90_000),
        &[],
        &[],
        /*tty*/ false,
        /*stdin_open*/ false,
        /*use_private_desktop*/ true,
    )
    .await
    .expect("spawn legacy launcher");
    let (stdout, _exit_code) =
        collect_stdout_and_exit(spawned, codex_home, Duration::from_secs(/*secs*/ 120)).await;
    let stdout = String::from_utf8_lossy(&stdout).into_owned();
    let launch = stdout
        .lines()
        .find_map(|line| line.trim().strip_prefix(LAUNCH_RESULT))
        .map(str::to_string);
    // Brokers start the child asynchronously.
    let wait = if launch
        .as_deref()
        .is_some_and(|launch| launch.starts_with("started=true"))
    {
        Duration::from_secs(/*secs*/ 30)
    } else {
        Duration::from_secs(/*secs*/ 5)
    };
    let deadline = Instant::now() + wait;
    let mut report_text = None;
    while Instant::now() < deadline {
        if let Ok(text) = fs::read_to_string(report)
            && !text.is_empty()
        {
            report_text = Some(text.trim().to_string());
            break;
        }
        tokio::time::sleep(Duration::from_millis(/*millis*/ 100)).await;
    }
    let new_processes = stdout
        .lines()
        .find_map(|line| line.trim().strip_prefix(NEW_PROCESSES))
        .unwrap_or_default()
        .split(',')
        .filter_map(|entry| entry.rsplit_once(':'))
        .filter_map(|(name, pid)| pid.parse().ok().map(|pid| inspect_pid(name, pid)))
        .collect();
    LaunchOutcome {
        launch,
        report: report_text,
        new_processes,
        stdout,
        log: sandbox_log(codex_home),
    }
}

fn launcher_argv() -> Vec<String> {
    vec![
        test_exe(),
        "--exact".to_string(),
        LAUNCHER_ENTRY.to_string(),
        "--nocapture".to_string(),
        "--test-threads=1".to_string(),
    ]
}

/// The control: a plain child of a legacy command stays in its job, Low and
/// restricted, and the report entry sees that.
#[test]
fn legacy_launch_plain_child_stays_in_session() {
    let _guard = legacy_process_test_guard();
    let root = user_owned_test_root();
    let runtime = current_thread_runtime();
    let outcome = runtime.block_on(async {
        let codex_home = sandbox_home("launch-control");
        let report = root.path().join("control.report");
        run_launch(
            root.path(),
            codex_home.path(),
            launcher_argv(),
            vec![
                (LAUNCH_ENV, "plain".to_string()),
                (COMMAND_ENV, report_command(&report)),
            ],
            &report,
        )
        .await
    });
    println!("\nlaunch-path control: report={:?}", outcome.report);
    assert_eq!(
        (
            outcome.launch.as_deref(),
            outcome
                .report
                .as_deref()
                .is_some_and(|report| report.starts_with(CONTAINED)),
        ),
        (Some("started=true"), true),
        "report={:?}\nstdout={:?}\n{}",
        outcome.report,
        outcome.stdout,
        outcome.log
    );
}

/// #1507 route 1: a legacy command names a process of another, concurrent
/// legacy session as its child's parent. The child would join that
/// session's job, outside the launcher's.
#[test]
#[ignore = "#1507: the child joins the other session's job with its token; closing it needs a per-session default DACL, a product decision"]
fn legacy_launch_under_another_session_process_stays_in_session() {
    let _guard = legacy_process_test_guard();
    let root = user_owned_test_root();
    let runtime = current_thread_runtime();
    let outcome = runtime.block_on(async {
        let codex_home = sandbox_home("launch-parent");
        let hold_dir = root.path().join("hold");
        fs::create_dir_all(&hold_dir).expect("create hold dir");
        let mut env_map: HashMap<String, String> = std::env::vars().collect();
        env_map.insert(HOLD_ENV.to_string(), hold_dir.display().to_string());
        let holder = spawn_windows_sandbox_session_legacy(
            &PermissionProfile::workspace_write(),
            workspace_roots_for(root.path()).as_slice(),
            codex_home.path(),
            vec![
                test_exe(),
                "--exact".to_string(),
                HOLD_ENTRY.to_string(),
                "--nocapture".to_string(),
                "--test-threads=1".to_string(),
            ],
            root.path(),
            env_map,
            /*timeout_ms*/ Some(90_000),
            &[],
            &[],
            /*tty*/ false,
            /*stdin_open*/ false,
            /*use_private_desktop*/ true,
        )
        .await
        .expect("spawn holding session");
        let pid_path = hold_dir.join("hold.pid");
        let deadline = Instant::now() + Duration::from_secs(/*secs*/ 60);
        let holder_pid = loop {
            if let Ok(pid) = fs::read_to_string(&pid_path)
                && !pid.is_empty()
            {
                break pid;
            }
            assert!(
                Instant::now() < deadline,
                "holding session never started\n{}",
                sandbox_log(codex_home.path())
            );
            tokio::time::sleep(Duration::from_millis(/*millis*/ 100)).await;
        };
        let report = root.path().join("parent.report");
        let outcome = run_launch(
            root.path(),
            codex_home.path(),
            launcher_argv(),
            vec![
                (LAUNCH_ENV, "parent".to_string()),
                (PARENT_ENV, holder_pid),
                (COMMAND_ENV, report_command(&report)),
            ],
            &report,
        )
        .await;
        fs::write(hold_dir.join("hold.release"), "").expect("release holder");
        let _ =
            collect_stdout_and_exit(holder, codex_home.path(), Duration::from_secs(/*secs*/ 90))
                .await;
        outcome
    });
    outcome.assert_contained("parent-process attribute", &["open_parent_error=5"]);
}

/// Runs a Windows PowerShell script as the legacy command. The script sees
/// the report command in `CODEX_1507_CMD` (and its parts in `CODEX_1507_EXE`,
/// `CODEX_1507_ARGS`, `CODEX_1507_DIR`) and prints one `launch: ` line.
fn run_powershell_launch(
    name: &str,
    script: &str,
    extra_env: Vec<(&str, String)>,
) -> LaunchOutcome {
    let _guard = legacy_process_test_guard();
    let root = user_owned_test_root();
    let runtime = current_thread_runtime();
    runtime.block_on(async {
        let codex_home = sandbox_home(name);
        let report = root.path().join(format!("{name}.report"));
        let encoded = BASE64.encode(
            script
                .encode_utf16()
                .flat_map(u16::to_le_bytes)
                .collect::<Vec<_>>(),
        );
        let mut env = vec![
            (COMMAND_ENV, report_command(&report)),
            (REPORT_ENV, report.display().to_string()),
            (EXE_ENV, test_exe()),
            (ARGS_ENV, report_args(&report)),
            (DIR_ENV, root.path().display().to_string()),
        ];
        env.extend(extra_env);
        run_launch(
            root.path(),
            codex_home.path(),
            vec![
                system32_exe("WindowsPowerShell\\v1.0\\powershell.exe")
                    .display()
                    .to_string(),
                "-NoProfile".to_string(),
                "-NonInteractive".to_string(),
                "-EncodedCommand".to_string(),
                encoded,
            ],
            env,
            &report,
        )
        .await
    })
}

/// Shared PowerShell prologue: `$cmd`, `$exe`, `$args1` and `$dir`, with
/// the launcher's id filled in, and `Report-Error`.
const POWERSHELL_PROLOGUE: &str = r#"
$ErrorActionPreference = 'Stop'
$cmd = $env:CODEX_1507_CMD.Replace('@LAUNCHER@', "$PID")
$args1 = $env:CODEX_1507_ARGS.Replace('@LAUNCHER@', "$PID")
$exe = $env:CODEX_1507_EXE
$dir = $env:CODEX_1507_DIR
$before = @(Get-Process | ForEach-Object Id)
$step = 'create'
function Report-Error($err) { "launch: started=false error=[$step] " + ($err.Exception.Message -replace '\s+', ' ') }
# Keeps the session (and its job) alive until the child reports, then lists
# the processes the launch added.
function Finish-Launch {
  $deadline = (Get-Date).AddSeconds(30)
  while (-not (Test-Path -LiteralPath $env:CODEX_1507_REPORT) -and (Get-Date) -lt $deadline) { Start-Sleep -Milliseconds 200 }
  $new = Get-Process | Where-Object { $before -notcontains $_.Id -and $_.Id -ne $PID } | ForEach-Object { "$($_.ProcessName):$($_.Id)" }
  "new: " + ($new -join ',')
}
"#;

/// #1507 route 2: WMI's `Win32_Process.Create` starts the process from the
/// WMI provider host, outside the caller's job.
#[test]
fn legacy_launch_through_wmi_stays_in_session() {
    let script = format!(
        r#"{POWERSHELL_PROLOGUE}
try {{
  $r = Invoke-CimMethod -ClassName Win32_Process -MethodName Create -Arguments @{{ CommandLine = $cmd }}
  if ($r.ReturnValue -eq 0) {{ "launch: started=true" }} else {{ "launch: started=false return_value=$($r.ReturnValue)" }}
}} catch {{ Report-Error $_ }}
Finish-Launch
"#
    );
    run_powershell_launch("launch-wmi", &script, Vec::new()).assert_contained(
        "WMI Win32_Process.Create",
        &["error=[create] ", "Access denied"],
    );
}

/// #1507 route 3: the Task Scheduler service starts a registered task as
/// the task's principal, outside the caller's job.
#[test]
fn legacy_launch_through_task_scheduler_stays_in_session() {
    let task = format!("codex-1507-{}", std::process::id());
    // Logon type 3: the caller's interactive token; flag 6: create or update.
    let script = format!(
        r#"{POWERSHELL_PROLOGUE}
try {{
  $service = New-Object -ComObject Schedule.Service
  $step = 'connect'
  $service.Connect()
  $step = 'folder'
  $folder = $service.GetFolder('\')
  $step = 'define'
  $definition = $service.NewTask(0)
  $action = $definition.Actions.Create(0)
  $action.Path = $exe
  $action.Arguments = $args1
  $action.WorkingDirectory = $dir
  $step = 'register'
  $null = $folder.RegisterTaskDefinition($env:CODEX_1507_TASK, $definition, 6, $null, $null, 3)
  $step = 'run'
  $null = $folder.GetTask($env:CODEX_1507_TASK).Run($null)
  "launch: started=true"
}} catch {{ Report-Error $_ }}
Finish-Launch
"#
    );
    let outcome = run_powershell_launch("launch-schtasks", &script, vec![(TASK_ENV, task.clone())]);
    // Remove the task in case the sandbox could register it.
    let _ = std::process::Command::new(system32_exe("schtasks.exe"))
        .args(["/delete", "/tn", task.as_str(), "/f"])
        .output();
    outcome.assert_contained("Task Scheduler", &["error=[folder] ", "0x80070003"]);
}

/// #1507 route 4: out-of-process COM servers that start programs on the
/// caller's behalf, from their own process. Each is tried in turn.
fn com_launch(name: &str, clsid: &str, invoke: &str, refusal: &[&str]) {
    let script = format!(
        r#"{POWERSHELL_PROLOGUE}
try {{
  $o = [Activator]::CreateInstance([Type]::GetTypeFromCLSID('{clsid}'))
  $step = 'invoke'
  {invoke}
  "launch: started=true"
}} catch {{ Report-Error $_ }}
Finish-Launch
"#
    );
    run_powershell_launch(name, &script, Vec::new()).assert_contained(name, refusal);
}

/// Explorer's `ShellWindows`: `ShellExecute` runs in Explorer.
#[test]
fn legacy_launch_through_com_shell_windows_stays_in_session() {
    com_launch(
        "launch-com-shellwindows",
        "9BA05972-F6A8-11CF-A442-00A0C90A8F39",
        "$w = $o.Item(); if ($null -eq $w) { throw 'no shell window' }; $w.Document.Application.ShellExecute($exe, $args1, $dir, $null, 0)",
        // Refused at activation, before any Explorer window is needed.
        &["error=[create] ", "80070005"],
    );
}

/// Explorer's `ShellBrowserWindow`: `ShellExecute` runs in Explorer.
#[test]
#[ignore = "#1507: starts the program outside the job with an unrestricted Low token (hosted runner, no Explorer running); closing it needs a product decision"]
fn legacy_launch_through_com_shell_browser_window_stays_in_session() {
    com_launch(
        "launch-com-shellbrowserwindow",
        "C08AFD90-F2A1-11D1-8455-00A0C91F3880",
        "$o.Document.Application.ShellExecute($exe, $args1, $dir, $null, 0)",
        &["80070005"],
    );
}

/// `MMC20.Application`: `ExecuteShellCommand` runs in MMC.
#[test]
#[ignore = "#1507: MMC starts as the caller, outside its job, and runs the program there (still Low and restricted); closing it needs a product decision"]
fn legacy_launch_through_com_mmc_stays_in_session() {
    com_launch(
        "launch-com-mmc",
        "49B2791A-B1AE-4C90-9B8E-E860BA07F889",
        "$o.Document.ActiveView.ExecuteShellCommand($exe, $dir, $args1, '7')",
        &["80070005"],
    );
}
