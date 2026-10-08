//! Starts a process as the elevated sandbox's user with
//! `CreateProcessWithLogonW` (the secondary logon service).

use crate::winutil::to_wide;
use std::ffi::c_void;
use std::path::Path;
use std::ptr;
use windows_sys::Win32::Foundation::CloseHandle;
use windows_sys::Win32::Foundation::GetLastError;
use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::System::Diagnostics::Debug::SetErrorMode;
use windows_sys::Win32::System::Threading::CREATE_NO_WINDOW;
use windows_sys::Win32::System::Threading::CREATE_UNICODE_ENVIRONMENT;
use windows_sys::Win32::System::Threading::CreateProcessWithLogonW;
use windows_sys::Win32::System::Threading::PROCESS_INFORMATION;
use windows_sys::Win32::System::Threading::STARTUPINFOW;

/// No error dialogs from the new process (`SEM_FAILCRITICALERRORS |
/// SEM_NOGPFAULTERRORBOX`).
const ERROR_MODE_FLAGS: u32 = 0x0001 | 0x0002;

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

/// Starts `request`. `launcher_exe` is the command runner, which can make
/// the call on this process's behalf.
pub fn create_process_with_logon(
    request: &LogonLaunchRequest<'_>,
    launcher_exe: &Path,
) -> anyhow::Result<LaunchedProcess> {
    let _ = launcher_exe;
    Ok(create_process_with_logon_here(request)?)
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
