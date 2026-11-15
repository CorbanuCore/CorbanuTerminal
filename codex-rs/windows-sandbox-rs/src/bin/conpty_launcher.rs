//! #1842: reset inherited Ctrl-C suppression inside the sandbox, before the
//! command starts. Core's own console state must remain untouched. This must
//! be a console-subsystem executable: a GUI process does not join the ConPTY.

#[cfg(windows)]
fn main() -> anyhow::Result<()> {
    std::process::exit(win::run()?);
}

#[cfg(not(windows))]
fn main() {
    panic!("codex-conpty-launcher is Windows-only");
}

#[cfg(windows)]
mod win {
    use anyhow::Context;
    use anyhow::Result;
    use std::os::windows::ffi::OsStrExt;
    use std::os::windows::io::AsRawHandle;
    use std::os::windows::io::FromRawHandle;
    use std::os::windows::io::OwnedHandle;
    use windows_sys::Win32::Foundation::WAIT_OBJECT_0;
    use windows_sys::Win32::System::Console::CTRL_BREAK_EVENT;
    use windows_sys::Win32::System::Console::CTRL_C_EVENT;
    use windows_sys::Win32::System::Console::SetConsoleCtrlHandler;
    use windows_sys::Win32::System::Threading::CreateProcessW;
    use windows_sys::Win32::System::Threading::GetExitCodeProcess;
    use windows_sys::Win32::System::Threading::INFINITE;
    use windows_sys::Win32::System::Threading::PROCESS_INFORMATION;
    use windows_sys::Win32::System::Threading::STARTUPINFOW;
    use windows_sys::Win32::System::Threading::WaitForSingleObject;

    // A registered handler protects this waiting launcher without setting the
    // inheritable ignore flag. The command gets its own normal control events
    // and may handle them itself, or disable processed input for raw Ctrl-C.
    unsafe extern "system" fn control_handler(event: u32) -> i32 {
        i32::from(event == CTRL_C_EVENT || event == CTRL_BREAK_EVENT)
    }

    pub(super) fn run() -> Result<i32> {
        let mut args = std::env::args_os().skip(1);
        let command_line = args.next().context("missing ConPTY command line")?;
        anyhow::ensure!(args.next().is_none(), "expected one ConPTY command line");
        let mut command_line: Vec<u16> = command_line.encode_wide().chain([0]).collect();
        let mut startup: STARTUPINFOW = unsafe { std::mem::zeroed() };
        startup.cb = std::mem::size_of::<STARTUPINFOW>() as u32;
        let mut process: PROCESS_INFORMATION = unsafe { std::mem::zeroed() };
        // SAFETY: this dedicated process is already inside the command's
        // restricted token, job and ConPTY. No token, ACL or job is changed.
        unsafe {
            if SetConsoleCtrlHandler(None, 0) == 0
                || SetConsoleCtrlHandler(Some(control_handler), 1) == 0
            {
                return Err(std::io::Error::last_os_error()).context("enable command Ctrl-C");
            }
            // Inherit this process's token, environment, cwd, console and job.
            // No handles, new process group or job breakaway are requested.
            if CreateProcessW(
                std::ptr::null(),
                command_line.as_mut_ptr(),
                std::ptr::null(),
                std::ptr::null(),
                0,
                0,
                std::ptr::null(),
                std::ptr::null(),
                &startup,
                &mut process,
            ) == 0
            {
                return Err(std::io::Error::last_os_error()).context("start ConPTY command");
            }
            let child = OwnedHandle::from_raw_handle(process.hProcess as _);
            let thread = OwnedHandle::from_raw_handle(process.hThread as _);
            drop(thread);
            if WaitForSingleObject(child.as_raw_handle() as _, INFINITE) != WAIT_OBJECT_0 {
                return Err(std::io::Error::last_os_error()).context("wait for ConPTY command");
            }
            let mut code = 0;
            if GetExitCodeProcess(child.as_raw_handle() as _, &mut code) == 0 {
                return Err(std::io::Error::last_os_error())
                    .context("read ConPTY command exit code");
            }
            Ok(code as i32)
        }
    }
}
