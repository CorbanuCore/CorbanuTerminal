//! #307/#320: hands a child process its inherited handles without making
//! them inheritable in this process's own handle table, where any process
//! started meanwhile on another thread with inheritance on
//! (`std::process::Command` always) would get them too.
//!
//! The handles are duplicated, inheritable, into a [`HandleHolder`]: a
//! process that never runs (created suspended, inheriting nothing, empty
//! environment). The child is then started with
//! `PROC_THREAD_ATTRIBUTE_PARENT_PROCESS` set to the holder, so it inherits
//! from the holder's table, and its handle list and std handles use the
//! holder's values. Nothing else is ever started from a holder.
//!
//! The child inherits the holder's token and job (copies of this process's)
//! and reports the holder as its parent process; the holder is ended right
//! after the child starts.

use std::ffi::OsStr;
use std::io;
use std::os::windows::ffi::OsStrExt as _;
use std::os::windows::io::AsRawHandle as _;
use std::os::windows::io::FromRawHandle as _;
use std::os::windows::io::OwnedHandle;
use std::path::Path;
use windows_sys::Win32::Foundation::CloseHandle;
use windows_sys::Win32::Foundation::DUPLICATE_SAME_ACCESS;
use windows_sys::Win32::Foundation::DuplicateHandle;
use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::Security::SECURITY_ATTRIBUTES;
use windows_sys::Win32::System::Threading::CREATE_SUSPENDED;
use windows_sys::Win32::System::Threading::CREATE_UNICODE_ENVIRONMENT;
use windows_sys::Win32::System::Threading::CreateProcessW;
use windows_sys::Win32::System::Threading::DETACHED_PROCESS;
use windows_sys::Win32::System::Threading::GetCurrentProcess;
use windows_sys::Win32::System::Threading::PROCESS_INFORMATION;
use windows_sys::Win32::System::Threading::STARTUPINFOW;
use windows_sys::Win32::System::Threading::TerminateProcess;
use windows_sys::Win32::System::Threading::WaitForSingleObject;

/// How long dropping a holder waits for it to go (so the copies it holds,
/// e.g. a pipe end whose EOF the caller waits for, are closed).
const HOLDER_EXIT_TIMEOUT_MS: u32 = 1_000;

/// A process of this process's that never runs; ended on drop. See the
/// module docs.
#[derive(Debug)]
pub struct HandleHolder {
    process: OwnedHandle,
}

impl HandleHolder {
    /// A holder from `image` (any executable; it never runs) with the
    /// default security descriptors, so this user's other processes can open
    /// it. Use it only for handles that are no secret from them.
    pub fn start(image: &Path) -> io::Result<Self> {
        Self::start_with(image, /*process*/ None, /*thread*/ None)
    }

    /// A holder whose process and first thread get these descriptors (e.g.
    /// the protected DACLs, so no other process of the user can take the
    /// handles out of it).
    pub(crate) fn start_with(
        image: &Path,
        process: Option<&SECURITY_ATTRIBUTES>,
        thread: Option<&SECURITY_ATTRIBUTES>,
    ) -> io::Result<Self> {
        let application = wide(image.as_os_str());
        let mut command_line = crate::windows_protected_spawn::command_line(image, &[])?;
        // Never this process's environment: the holder's could be readable.
        let environment = [0_u16, 0];
        // SAFETY: zeroed POD with its size set, as the API requires.
        let mut startup: STARTUPINFOW = unsafe { std::mem::zeroed() };
        startup.cb = std::mem::size_of::<STARTUPINFOW>() as u32;
        // SAFETY: zeroed POD filled in by the call.
        let mut info: PROCESS_INFORMATION = unsafe { std::mem::zeroed() };
        // SAFETY: every pointer refers to a live, NUL-terminated buffer or
        // descriptor above, or is null.
        let ok = unsafe {
            CreateProcessW(
                application.as_ptr(),
                command_line.as_mut_ptr(),
                process.map_or(std::ptr::null(), std::ptr::from_ref),
                thread.map_or(std::ptr::null(), std::ptr::from_ref),
                /*binherithandles*/ 0,
                CREATE_SUSPENDED | DETACHED_PROCESS | CREATE_UNICODE_ENVIRONMENT,
                environment.as_ptr().cast(),
                std::ptr::null(),
                &startup,
                &mut info,
            )
        };
        if ok == 0 {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: returned by the call above and not used again.
        unsafe { CloseHandle(info.hThread) };
        Ok(Self {
            // SAFETY: returned by the call above; owned from here on.
            process: unsafe { OwnedHandle::from_raw_handle(info.hProcess as _) },
        })
    }

    /// An inheritable copy of `handle` in the holder's table; the value is
    /// only meaningful there.
    pub fn hold(&self, handle: HANDLE) -> io::Result<HANDLE> {
        let mut held: HANDLE = 0;
        // SAFETY: `handle` is the caller's and live; the holder's handle has
        // full access (this process created it).
        let ok = unsafe {
            DuplicateHandle(
                GetCurrentProcess(),
                handle,
                self.raw(),
                &mut held,
                /*dwdesiredaccess*/ 0,
                /*binherithandle*/ 1,
                DUPLICATE_SAME_ACCESS,
            )
        };
        if ok == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(held)
    }

    /// The holder's process handle (full access), for
    /// `PROC_THREAD_ATTRIBUTE_PARENT_PROCESS`.
    pub fn raw(&self) -> HANDLE {
        self.process.as_raw_handle() as HANDLE
    }
}

impl Drop for HandleHolder {
    fn drop(&mut self) {
        // SAFETY: the holder never ran; ending it closes the copies it
        // holds. Waited for, so a pipe's EOF is not delayed by them.
        unsafe {
            if TerminateProcess(self.raw(), 1) != 0 {
                WaitForSingleObject(self.raw(), HOLDER_EXIT_TIMEOUT_MS);
            }
        }
    }
}

fn wide(value: &OsStr) -> Vec<u16> {
    value.encode_wide().chain([0]).collect()
}
