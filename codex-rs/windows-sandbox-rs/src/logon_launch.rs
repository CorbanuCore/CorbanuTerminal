//! Starts a process as the elevated sandbox's user with
//! `CreateProcessWithLogonW` (the secondary logon service).
//!
//! #295: from a medium-integrity caller the service opens the caller's
//! process as the caller and needs `PROCESS_QUERY_INFORMATION`,
//! `PROCESS_DUP_HANDLE` and `PROCESS_CREATE_PROCESS` on it (measured on
//! Windows 11 26200; an elevated caller needs none). Core's protected process
//! DACL (secretless agent launch) grants the user none of them, and granting
//! them would undo it: `PROCESS_DUP_HANDLE` amounts to full access. So when
//! the call is refused with `ERROR_ACCESS_DENIED`, Core starts the command
//! runner as itself in launcher mode ([`LOGON_LAUNCH_ARG`]), an ordinary
//! short-lived process that makes the call, and duplicates the new process's
//! handle out of it. The launcher holds nothing the user cannot already read
//! (the sandbox user's password is stored DPAPI-protected for this user), gets
//! an empty environment, and exits when Core closes its stdin. The runner
//! binary is in the sandbox's helper directory, which the sandbox's users can
//! only read and execute.

use crate::winutil::to_wide;
use anyhow::Context;
use serde::Deserialize;
use serde::Serialize;
use std::ffi::c_void;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Read;
use std::io::Write;
use std::os::windows::io::AsRawHandle;
use std::os::windows::process::CommandExt;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;
use std::ptr;
use std::sync::mpsc;
use std::time::Duration;
use windows_sys::Win32::Foundation::CloseHandle;
use windows_sys::Win32::Foundation::DUPLICATE_SAME_ACCESS;
use windows_sys::Win32::Foundation::DuplicateHandle;
use windows_sys::Win32::Foundation::ERROR_ACCESS_DENIED;
use windows_sys::Win32::Foundation::GetLastError;
use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::System::Diagnostics::Debug::SetErrorMode;
use windows_sys::Win32::System::Threading::CREATE_NO_WINDOW;
use windows_sys::Win32::System::Threading::CREATE_UNICODE_ENVIRONMENT;
use windows_sys::Win32::System::Threading::CreateProcessWithLogonW;
use windows_sys::Win32::System::Threading::GetCurrentProcess;
use windows_sys::Win32::System::Threading::PROCESS_INFORMATION;
use windows_sys::Win32::System::Threading::STARTUPINFOW;

/// No error dialogs from the new process (`SEM_FAILCRITICALERRORS |
/// SEM_NOGPFAULTERRORBOX`).
const ERROR_MODE_FLAGS: u32 = 0x0001 | 0x0002;

/// The command runner's first argument for [`run_logon_launcher`].
pub const LOGON_LAUNCH_ARG: &str = "--logon-launch";

/// How long Core waits for the launcher's reply.
const LAUNCHER_REPLY_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Serialize, Deserialize)]
struct LauncherRequest {
    username: String,
    password: String,
    application: PathBuf,
    command_line: String,
    cwd: PathBuf,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum LauncherReply {
    /// `process` is a handle value in the launcher's handle table.
    Started {
        pid: u32,
        process: usize,
    },
    Failed {
        code: u32,
    },
}

/// `CreateProcessWithLogonW` failed with this Win32 error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LogonError {
    pub code: u32,
}

impl std::fmt::Display for LogonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CreateProcessWithLogonW failed: {}", self.code)
    }
}

impl std::error::Error for LogonError {}

/// What to start, as which local user. The new process gets the user's own
/// environment and no window.
pub struct LogonLaunchRequest<'a> {
    pub username: &'a str,
    pub password: &'a str,
    pub application: &'a Path,
    pub command_line: &'a str,
    pub cwd: &'a Path,
}

/// A started process. The caller owns `process` and must close it.
#[derive(Debug)]
pub struct LaunchedProcess {
    pub pid: u32,
    pub process: HANDLE,
}

/// Starts `request`. When the secondary logon service refuses this process
/// (its DACL is protected and it is not elevated), `launcher_exe` (the
/// command runner) makes the call instead; see the module docs.
pub fn create_process_with_logon(
    request: &LogonLaunchRequest<'_>,
    launcher_exe: &Path,
) -> anyhow::Result<LaunchedProcess> {
    match create_process_with_logon_here(request) {
        Err(LogonError {
            code: ERROR_ACCESS_DENIED,
        }) => create_process_with_logon_via_launcher(request, launcher_exe),
        result => Ok(result?),
    }
}

fn create_process_with_logon_via_launcher(
    request: &LogonLaunchRequest<'_>,
    launcher_exe: &Path,
) -> anyhow::Result<LaunchedProcess> {
    let mut launcher = Command::new(launcher_exe);
    launcher
        .arg(LOGON_LAUNCH_ARG)
        .env_clear()
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .creation_flags(CREATE_NO_WINDOW);
    if let Some(system_root) = std::env::var_os("SystemRoot") {
        launcher.env("SystemRoot", system_root);
    }
    let mut child = launcher
        .spawn()
        .with_context(|| format!("start logon launcher {}", launcher_exe.display()))?;
    let result = (|| -> anyhow::Result<LaunchedProcess> {
        // Held until the handle is duplicated; closing it ends the launcher.
        let mut stdin = child.stdin.take().context("launcher stdin")?;
        let stdout = child.stdout.take().context("launcher stdout")?;
        let mut line = serde_json::to_vec(&LauncherRequest {
            username: request.username.to_string(),
            password: request.password.to_string(),
            application: request.application.to_path_buf(),
            command_line: request.command_line.to_string(),
            cwd: request.cwd.to_path_buf(),
        })?;
        line.push(b'\n');
        let written = stdin.write_all(&line).and_then(|()| stdin.flush());
        line.fill(0);
        written.context("send launch request")?;
        let (reply_tx, reply_rx) = mpsc::channel();
        std::thread::spawn(move || {
            let mut reply = String::new();
            let read = BufReader::new(stdout).read_line(&mut reply).map(|_| reply);
            let _ = reply_tx.send(read);
        });
        let reply = reply_rx
            .recv_timeout(LAUNCHER_REPLY_TIMEOUT)
            .map_err(|_| anyhow::anyhow!("logon launcher did not reply"))?
            .context("read launcher reply")?;
        match serde_json::from_str(&reply).context("parse launcher reply")? {
            LauncherReply::Failed { code } => Err(LogonError { code }.into()),
            LauncherReply::Started { pid, process } => {
                let mut local: HANDLE = 0;
                // SAFETY: `child` is our own child process, opened with full
                // access; `process` is a handle in its table that it keeps
                // open until we close its stdin.
                let ok = unsafe {
                    DuplicateHandle(
                        child.as_raw_handle() as HANDLE,
                        process as HANDLE,
                        GetCurrentProcess(),
                        &mut local,
                        0,
                        0,
                        DUPLICATE_SAME_ACCESS,
                    )
                };
                if ok == 0 {
                    return Err(std::io::Error::last_os_error())
                        .context("duplicate the runner handle from the launcher");
                }
                Ok(LaunchedProcess {
                    pid,
                    process: local,
                })
            }
        }
    })();
    if result.is_err() {
        let _ = child.kill();
    }
    let _ = child.wait();
    result
}

/// The command runner's launcher mode: reads one launch request from stdin,
/// starts it with `CreateProcessWithLogonW`, writes the reply to stdout, and
/// keeps the new process's handle open until stdin closes (Core duplicates
/// it first).
pub fn run_logon_launcher() -> anyhow::Result<()> {
    let mut stdin = std::io::stdin().lock();
    let mut line = String::new();
    stdin.read_line(&mut line).context("read launch request")?;
    let parsed = serde_json::from_str::<LauncherRequest>(&line);
    // SAFETY: overwriting with zeros keeps the string valid UTF-8.
    unsafe { line.as_bytes_mut().fill(0) };
    let mut request = parsed.context("parse launch request")?;
    let result = create_process_with_logon_here(&LogonLaunchRequest {
        username: &request.username,
        password: &request.password,
        application: &request.application,
        command_line: &request.command_line,
        cwd: &request.cwd,
    });
    // SAFETY: as above.
    unsafe { request.password.as_bytes_mut().fill(0) };
    let reply = match &result {
        Ok(launched) => LauncherReply::Started {
            pid: launched.pid,
            process: launched.process as usize,
        },
        Err(err) => LauncherReply::Failed { code: err.code },
    };
    let mut stdout = std::io::stdout().lock();
    serde_json::to_writer(&mut stdout, &reply)?;
    stdout.write_all(b"\n")?;
    stdout.flush()?;
    let _ = stdin.read_to_end(&mut Vec::new());
    if let Ok(launched) = result {
        // SAFETY: returned by the call above and not used again.
        unsafe { CloseHandle(launched.process) };
    }
    Ok(())
}

/// Calls `CreateProcessWithLogonW` from this process.
fn create_process_with_logon_here(
    request: &LogonLaunchRequest<'_>,
) -> Result<LaunchedProcess, LogonError> {
    let user = to_wide(request.username);
    let domain = to_wide(".");
    let password = to_wide(request.password);
    let application = to_wide(request.application);
    let mut command_line = to_wide(request.command_line);
    let cwd = to_wide(request.cwd);
    // SAFETY: zeroed POD with its size set, as the API requires.
    let mut startup: STARTUPINFOW = unsafe { std::mem::zeroed() };
    startup.cb = std::mem::size_of::<STARTUPINFOW>() as u32;
    // SAFETY: zeroed POD filled in by the call.
    let mut info: PROCESS_INFORMATION = unsafe { std::mem::zeroed() };
    // SAFETY: only changes this process's error mode; restored below.
    let previous_error_mode = unsafe { SetErrorMode(ERROR_MODE_FLAGS) };
    // SAFETY: every pointer refers to a live, NUL-terminated buffer above.
    let ok = unsafe {
        CreateProcessWithLogonW(
            user.as_ptr(),
            domain.as_ptr(),
            password.as_ptr(),
            /*dwlogonflags*/ 0,
            application.as_ptr(),
            command_line.as_mut_ptr(),
            CREATE_NO_WINDOW | CREATE_UNICODE_ENVIRONMENT,
            ptr::null::<c_void>(),
            cwd.as_ptr(),
            &startup,
            &mut info,
        )
    };
    // SAFETY: read right after the call, before anything else can set it.
    let code = unsafe { GetLastError() };
    // SAFETY: as above.
    unsafe { SetErrorMode(previous_error_mode) };
    if ok == 0 {
        return Err(LogonError { code });
    }
    if info.hThread != 0 {
        // SAFETY: returned by the call above and not used again.
        unsafe { CloseHandle(info.hThread) };
    }
    Ok(LaunchedProcess {
        pid: info.dwProcessId,
        process: info.hProcess,
    })
}
