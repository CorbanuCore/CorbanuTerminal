//! Starts a process as the elevated sandbox's user with
//! `CreateProcessWithLogonW` (the secondary logon service).
//!
//! #295: the service opens the caller's process as the caller and, from a
//! normal session, needs `PROCESS_QUERY_INFORMATION`, `PROCESS_DUP_HANDLE`
//! and `PROCESS_CREATE_PROCESS` on it (measured on Windows 11 26200). Core's
//! protected process DACL (secretless agent launch) grants the user none of
//! them, and granting them would undo it: `PROCESS_DUP_HANDLE` amounts to full
//! access. So when Core's DACL is protected and the call is refused with
//! `ERROR_ACCESS_DENIED`, Core starts the command runner as itself in
//! launcher mode ([`LOGON_LAUNCH_ARG`]), an ordinary short-lived process that
//! makes the call, and duplicates the new process's handle out of it.
//!
//! The launcher inherits only its own two pipes and an environment holding
//! just `SystemRoot`. The pipe ends are never inheritable in Core's own
//! handle table (#307), where any process Core started on another thread
//! meanwhile (`std::process::Command` always inherits) would get them too:
//! they are duplicated, inheritable, into a holder process that never runs
//! (`codex_process_hardening::HandleHolder`, shared with `spawn_protected`),
//! and the launcher is started as the holder's child; Core starts nothing
//! else from the holder. (Like the launcher, the holder has the default DACL,
//! so this user's other processes could open it; the sandbox's users cannot.)
//! The launcher receives
//! the sandbox user's password, which is not a secret from this user (it is
//! stored under `.sandbox-secrets`, which this user can read). Its binary is
//! in the sandbox's helper directory, which the sandbox's users can only read
//! and execute. Core checks that the returned handle is the process the
//! launcher named, and the launcher ends that process unless Core
//! acknowledges it.

use crate::proc_thread_attr::ProcThreadAttributeList;
use crate::winutil::quote_windows_arg;
use crate::winutil::to_wide;
use anyhow::Context;
use codex_process_hardening::HandleHolder;
use serde::Deserialize;
use serde::Serialize;
use std::ffi::c_void;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::os::windows::io::FromRawHandle;
use std::path::Path;
use std::path::PathBuf;
use std::ptr;
use std::sync::mpsc;
use std::time::Duration;
use windows_sys::Win32::Foundation::CloseHandle;
use windows_sys::Win32::Foundation::DUPLICATE_SAME_ACCESS;
use windows_sys::Win32::Foundation::DuplicateHandle;
use windows_sys::Win32::Foundation::ERROR_ACCESS_DENIED;
use windows_sys::Win32::Foundation::ERROR_SUCCESS;
use windows_sys::Win32::Foundation::GetLastError;
use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::Foundation::HLOCAL;
use windows_sys::Win32::Foundation::LocalFree;
use windows_sys::Win32::Security::Authorization::GetSecurityInfo;
use windows_sys::Win32::Security::Authorization::SE_KERNEL_OBJECT;
use windows_sys::Win32::Security::DACL_SECURITY_INFORMATION;
use windows_sys::Win32::Security::GetSecurityDescriptorControl;
use windows_sys::Win32::Security::PSECURITY_DESCRIPTOR;
use windows_sys::Win32::Security::SE_DACL_PROTECTED;
use windows_sys::Win32::System::Diagnostics::Debug::SetErrorMode;
use windows_sys::Win32::System::Pipes::CreatePipe;
use windows_sys::Win32::System::Threading::CREATE_NO_WINDOW;
use windows_sys::Win32::System::Threading::CREATE_UNICODE_ENVIRONMENT;
use windows_sys::Win32::System::Threading::CreateProcessW;
use windows_sys::Win32::System::Threading::CreateProcessWithLogonW;
use windows_sys::Win32::System::Threading::EXTENDED_STARTUPINFO_PRESENT;
use windows_sys::Win32::System::Threading::GetCurrentProcess;
use windows_sys::Win32::System::Threading::GetProcessId;
use windows_sys::Win32::System::Threading::PROCESS_INFORMATION;
use windows_sys::Win32::System::Threading::STARTF_USESTDHANDLES;
use windows_sys::Win32::System::Threading::STARTUPINFOEXW;
use windows_sys::Win32::System::Threading::STARTUPINFOW;
use windows_sys::Win32::System::Threading::TerminateProcess;
use windows_sys::Win32::System::Threading::WaitForSingleObject;

/// No error dialogs from the new process (`SEM_FAILCRITICALERRORS |
/// SEM_NOGPFAULTERRORBOX`).
const ERROR_MODE_FLAGS: u32 = 0x0001 | 0x0002;

/// The command runner's first argument for [`run_logon_launcher`].
pub const LOGON_LAUNCH_ARG: &str = "--logon-launch";

/// How long Core waits for the launcher's reply, and for it to exit.
const LAUNCHER_REPLY_TIMEOUT: Duration = Duration::from_secs(30);
const LAUNCHER_EXIT_TIMEOUT_MS: u32 = 5_000;
/// Core's acknowledgement: it holds its own handle to the started process.
const ACK: &str = "ack";

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
    /// Started through the launcher (see the module docs).
    pub via_launcher: bool,
}

/// Starts `request`. When this process's DACL is protected and the secondary
/// logon service refuses it, `launcher_exe` (the command runner) makes the
/// call instead; see the module docs.
pub fn create_process_with_logon(
    request: &LogonLaunchRequest<'_>,
    launcher_exe: &Path,
) -> anyhow::Result<LaunchedProcess> {
    match create_process_with_logon_here(request) {
        // Only an absolute path to the installed runner: a bare name would be
        // looked up in the working directory (the workspace), and the launcher
        // runs as the real user. A runner next to Core's own executable is
        // trusted like Core itself.
        Err(LogonError {
            code: ERROR_ACCESS_DENIED,
        }) if current_process_dacl_is_protected()
            && launcher_exe.is_absolute()
            && launcher_exe.is_file() =>
        {
            create_process_with_logon_via_launcher(request, launcher_exe)
        }
        result => result.map_err(Into::into),
    }
}

/// True once this process's DACL is protected (process hardening).
fn current_process_dacl_is_protected() -> bool {
    let mut descriptor: PSECURITY_DESCRIPTOR = ptr::null_mut();
    // SAFETY: queries this process's DACL into a buffer freed below.
    let status = unsafe {
        GetSecurityInfo(
            GetCurrentProcess(),
            SE_KERNEL_OBJECT,
            DACL_SECURITY_INFORMATION,
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            &mut descriptor,
        )
    };
    if status != ERROR_SUCCESS {
        return false;
    }
    let mut control = 0;
    let mut revision = 0;
    // SAFETY: `descriptor` was returned by GetSecurityInfo above.
    let read = unsafe { GetSecurityDescriptorControl(descriptor, &mut control, &mut revision) };
    // SAFETY: allocated by GetSecurityInfo.
    unsafe { LocalFree(descriptor as HLOCAL) };
    read != 0 && control & SE_DACL_PROTECTED != 0
}

/// A Windows handle closed on drop.
struct OwnedHandle(HANDLE);

impl Drop for OwnedHandle {
    fn drop(&mut self) {
        if self.0 != 0 {
            // SAFETY: owned by this value.
            unsafe { CloseHandle(self.0) };
        }
    }
}

/// A pipe whose `child` end the launcher gets. Neither end is inheritable
/// here.
fn launcher_pipe(child_reads: bool) -> anyhow::Result<(OwnedHandle, File)> {
    let (mut read, mut write) = (0, 0);
    // SAFETY: creates an anonymous pipe; both ends are owned below.
    if unsafe { CreatePipe(&mut read, &mut write, ptr::null(), 0) } == 0 {
        return Err(std::io::Error::last_os_error()).context("create launcher pipe");
    }
    let (child, parent) = if child_reads {
        (read, write)
    } else {
        (write, read)
    };
    let child = OwnedHandle(child);
    // SAFETY: `parent` is a fresh pipe end owned by the returned File.
    let parent = unsafe { File::from_raw_handle(parent as _) };
    Ok((child, parent))
}

/// Starts the launcher with only its two pipe ends inherited.
fn spawn_launcher(launcher_exe: &Path) -> anyhow::Result<(OwnedHandle, File, BufReader<File>)> {
    let (child_stdin, stdin) = launcher_pipe(/*child_reads*/ true)?;
    let (child_stdout, stdout) = launcher_pipe(/*child_reads*/ false)?;
    let mut environment = Vec::new();
    if let Some(system_root) = std::env::var_os("SystemRoot") {
        let mut entry = std::ffi::OsString::from("SystemRoot=");
        entry.push(system_root);
        environment.extend(to_wide(entry));
    }
    environment.push(0);
    let mut command_line = to_wide(format!(
        "{} {LOGON_LAUNCH_ARG}",
        quote_windows_arg(&launcher_exe.to_string_lossy())
    ));
    let application = to_wide(launcher_exe);
    // #307: the pipe ends are inheritable only in a holder's table, and the
    // launcher is started as its child.
    let holder = HandleHolder::start(launcher_exe).context("start the launcher's handle holder")?;
    let held_stdin = holder
        .hold(child_stdin.0)
        .context("hand a launcher pipe to the holder")?;
    let held_stdout = holder
        .hold(child_stdout.0)
        .context("hand a launcher pipe to the holder")?;
    drop((child_stdin, child_stdout));
    let mut attributes = ProcThreadAttributeList::new(/*attr_count*/ 2)?;
    attributes.set_parent_process(holder.raw())?;
    attributes.set_handle_list(vec![held_stdin, held_stdout])?;
    // SAFETY: zeroed POD with its size set, as the API requires.
    let mut startup: STARTUPINFOEXW = unsafe { std::mem::zeroed() };
    startup.StartupInfo.cb = std::mem::size_of::<STARTUPINFOEXW>() as u32;
    startup.StartupInfo.dwFlags = STARTF_USESTDHANDLES;
    // Values in the holder's table, as the parent-process attribute requires.
    startup.StartupInfo.hStdInput = held_stdin;
    startup.StartupInfo.hStdOutput = held_stdout;
    // No stderr: nothing but the reply may reach the reply pipe.
    startup.StartupInfo.hStdError = 0;
    startup.lpAttributeList = attributes.as_mut_ptr();
    // Never the workspace: the launcher runs in its own directory.
    let cwd = launcher_exe.parent().map(to_wide);
    // SAFETY: zeroed POD filled in by the call.
    let mut info: PROCESS_INFORMATION = unsafe { std::mem::zeroed() };
    // SAFETY: every pointer refers to a live buffer above; the launcher
    // inherits from the holder, and only the two pipe ends in the list.
    let ok = unsafe {
        CreateProcessW(
            application.as_ptr(),
            command_line.as_mut_ptr(),
            ptr::null(),
            ptr::null(),
            /*binherithandles*/ 1,
            CREATE_NO_WINDOW | CREATE_UNICODE_ENVIRONMENT | EXTENDED_STARTUPINFO_PRESENT,
            environment.as_ptr().cast(),
            cwd.as_ref().map_or(ptr::null(), Vec::as_ptr),
            &startup.StartupInfo,
            &mut info,
        )
    };
    if ok == 0 {
        return Err(std::io::Error::last_os_error())
            .with_context(|| format!("start logon launcher {}", launcher_exe.display()));
    }
    drop(holder);
    // SAFETY: returned by the call above and not used again.
    unsafe { CloseHandle(info.hThread) };
    Ok((OwnedHandle(info.hProcess), stdin, BufReader::new(stdout)))
}

fn create_process_with_logon_via_launcher(
    request: &LogonLaunchRequest<'_>,
    launcher_exe: &Path,
) -> anyhow::Result<LaunchedProcess> {
    let (launcher, mut stdin, mut stdout) = spawn_launcher(launcher_exe)?;
    let result = (|| -> anyhow::Result<LaunchedProcess> {
        let mut line = serde_json::to_vec(&LauncherRequest {
            username: request.username.to_string(),
            password: request.password.to_string(),
            application: request.application.to_path_buf(),
            command_line: request.command_line.to_string(),
            cwd: request.cwd.to_path_buf(),
        })?;
        line.push(b'\n');
        stdin.write_all(&line).context("send launch request")?;
        stdin.flush().context("send launch request")?;
        let (reply_tx, reply_rx) = mpsc::channel();
        std::thread::spawn(move || {
            let mut reply = String::new();
            let read = stdout.read_line(&mut reply).map(|_| reply);
            let _ = reply_tx.send(read);
        });
        let reply = match reply_rx.recv_timeout(LAUNCHER_REPLY_TIMEOUT) {
            Ok(reply) => reply.context("read launcher reply")?,
            Err(mpsc::RecvTimeoutError::Timeout) => anyhow::bail!("logon launcher did not reply"),
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                anyhow::bail!("logon launcher exited without replying")
            }
        };
        match serde_json::from_str(&reply).context("parse launcher reply")? {
            LauncherReply::Failed { code } => Err(LogonError { code }.into()),
            LauncherReply::Started { pid, process } => {
                let mut local: HANDLE = 0;
                // SAFETY: `launcher` is our own child, opened with full
                // access; `process` is a handle in its table that it keeps
                // open until it reads our acknowledgement.
                let ok = unsafe {
                    DuplicateHandle(
                        launcher.0,
                        process as HANDLE,
                        GetCurrentProcess(),
                        &mut local,
                        /*dwdesiredaccess*/ 0,
                        /*binherithandle*/ 0,
                        DUPLICATE_SAME_ACCESS,
                    )
                };
                if ok == 0 {
                    return Err(std::io::Error::last_os_error())
                        .context("duplicate the started process's handle from the launcher");
                }
                let local = OwnedHandle(local);
                // SAFETY: `local.0` is a live process handle.
                if unsafe { GetProcessId(local.0) } != pid {
                    anyhow::bail!("logon launcher returned a handle to another process");
                }
                if let Err(err) = writeln!(stdin, "{ACK}").and_then(|()| stdin.flush()) {
                    // The launcher is gone and cannot end it; Core can.
                    // SAFETY: `local.0` is the started process, checked above.
                    unsafe { TerminateProcess(local.0, 1) };
                    return Err(err).context("acknowledge the started process");
                }
                let process = local.0;
                std::mem::forget(local);
                Ok(LaunchedProcess {
                    pid,
                    process,
                    via_launcher: true,
                })
            }
        }
    })();
    drop(stdin);
    // SAFETY: `launcher.0` is a live process handle.
    unsafe {
        if WaitForSingleObject(launcher.0, LAUNCHER_EXIT_TIMEOUT_MS) != 0 {
            TerminateProcess(launcher.0, 1);
        }
    }
    result
}

/// The command runner's launcher mode: reads one launch request from stdin,
/// starts it with `CreateProcessWithLogonW`, writes the reply to stdout, and
/// keeps the new process's handle open until Core acknowledges it. Without
/// that acknowledgement (Core failed or went away) it ends the new process.
pub fn run_logon_launcher() -> anyhow::Result<()> {
    let mut stdin = std::io::stdin().lock();
    let mut line = String::new();
    stdin.read_line(&mut line).context("read launch request")?;
    let request: LauncherRequest = serde_json::from_str(&line).context("parse launch request")?;
    let result = create_process_with_logon_here(&LogonLaunchRequest {
        username: &request.username,
        password: &request.password,
        application: &request.application,
        command_line: &request.command_line,
        cwd: &request.cwd,
    });
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
    let Ok(launched) = result else {
        return Ok(());
    };
    let mut ack = String::new();
    let acknowledged = stdin.read_line(&mut ack).is_ok() && ack.trim() == ACK;
    // SAFETY: returned by CreateProcessWithLogonW and not used after this.
    unsafe {
        if !acknowledged {
            TerminateProcess(launched.process, 1);
        }
        CloseHandle(launched.process);
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
        via_launcher: false,
    })
}

#[cfg(test)]
#[path = "logon_launch_tests.rs"]
mod tests;
