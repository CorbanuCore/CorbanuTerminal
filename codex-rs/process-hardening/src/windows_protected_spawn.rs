//! PF-27-S07: starts a process that no other process of the user can open
//! at any point of its life.
//!
//! `std::process::Command` creates the process and its first thread with the
//! creator's default DACL, so a same-user process can open either before the
//! child hardens itself and keep the handle. Here both get the protected
//! DACLs at creation, the process starts suspended, its token's default DACL
//! becomes the protected thread DACL (so every thread it starts later is
//! protected from creation too), and only then does it run.

use crate::windows_process_access::SecurityDescriptor;
use crate::windows_process_access::current_user_sid_string;
use crate::windows_process_access::process_dacl_sddl;
use crate::windows_process_access::thread_dacl_sddl;
use crate::windows_thread_creation::set_protected_default_dacl;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::fs::File;
use std::io;
use std::os::windows::ffi::OsStrExt as _;
use std::os::windows::io::AsRawHandle as _;
use std::os::windows::io::FromRawHandle as _;
use std::os::windows::io::OwnedHandle;
use std::os::windows::process::ExitStatusExt as _;
use std::path::Path;
use std::process::ExitStatus;
use windows_sys::Win32::Foundation::CloseHandle;
use windows_sys::Win32::Foundation::ERROR_ACCESS_DENIED;
use windows_sys::Win32::Foundation::GetLastError;
use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::Foundation::HANDLE_FLAG_INHERIT;
use windows_sys::Win32::Foundation::SetHandleInformation;
use windows_sys::Win32::Foundation::WAIT_OBJECT_0;
use windows_sys::Win32::Security::SECURITY_ATTRIBUTES;
use windows_sys::Win32::Security::TOKEN_ADJUST_DEFAULT;
use windows_sys::Win32::Security::TOKEN_QUERY;
use windows_sys::Win32::System::Pipes::CreatePipe;
use windows_sys::Win32::System::Threading::CREATE_SUSPENDED;
use windows_sys::Win32::System::Threading::CREATE_UNICODE_ENVIRONMENT;
use windows_sys::Win32::System::Threading::CreateProcessW;
use windows_sys::Win32::System::Threading::DETACHED_PROCESS;
use windows_sys::Win32::System::Threading::DeleteProcThreadAttributeList;
use windows_sys::Win32::System::Threading::EXTENDED_STARTUPINFO_PRESENT;
use windows_sys::Win32::System::Threading::GetExitCodeProcess;
use windows_sys::Win32::System::Threading::INFINITE;
use windows_sys::Win32::System::Threading::InitializeProcThreadAttributeList;
use windows_sys::Win32::System::Threading::LPPROC_THREAD_ATTRIBUTE_LIST;
use windows_sys::Win32::System::Threading::OpenProcessToken;
use windows_sys::Win32::System::Threading::PROC_THREAD_ATTRIBUTE_HANDLE_LIST;
use windows_sys::Win32::System::Threading::PROCESS_INFORMATION;
use windows_sys::Win32::System::Threading::ResumeThread;
use windows_sys::Win32::System::Threading::STARTF_USESTDHANDLES;
use windows_sys::Win32::System::Threading::STARTUPINFOEXW;
use windows_sys::Win32::System::Threading::TerminateProcess;
use windows_sys::Win32::System::Threading::UpdateProcThreadAttribute;
use windows_sys::Win32::System::Threading::WaitForSingleObject;

/// A process started by [`spawn_protected`]. Like [`std::process::Child`],
/// dropping it neither kills nor waits for the process.
#[derive(Debug)]
pub struct ProtectedChild {
    process: OwnedHandle,
    pid: u32,
}

impl ProtectedChild {
    pub fn id(&self) -> u32 {
        self.pid
    }

    /// Terminates the process; succeeds if it already exited.
    pub fn kill(&mut self) -> io::Result<()> {
        // SAFETY: `process` is an open handle with terminate access.
        if unsafe { TerminateProcess(self.raw(), 1) } != 0 {
            return Ok(());
        }
        // SAFETY: reads the thread's last error.
        let error = unsafe { GetLastError() };
        if error == ERROR_ACCESS_DENIED && matches!(self.try_wait(), Ok(Some(_))) {
            return Ok(());
        }
        Err(io::Error::from_raw_os_error(error as i32))
    }

    pub fn wait(&mut self) -> io::Result<ExitStatus> {
        // SAFETY: `process` is an open handle with synchronize access.
        if unsafe { WaitForSingleObject(self.raw(), INFINITE) } != WAIT_OBJECT_0 {
            return Err(io::Error::last_os_error());
        }
        self.exit_status()
    }

    pub fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
        // SAFETY: as above, without waiting.
        match unsafe { WaitForSingleObject(self.raw(), 0) } {
            WAIT_OBJECT_0 => self.exit_status().map(Some),
            windows_sys::Win32::Foundation::WAIT_TIMEOUT => Ok(None),
            _ => Err(io::Error::last_os_error()),
        }
    }

    fn exit_status(&self) -> io::Result<ExitStatus> {
        let mut code = 0_u32;
        // SAFETY: `process` is an open handle; `code` is a valid out pointer.
        if unsafe { GetExitCodeProcess(self.raw(), &mut code) } == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(ExitStatus::from_raw(code))
    }

    fn raw(&self) -> HANDLE {
        self.process.as_raw_handle() as HANDLE
    }
}

/// Starts `program` with `args` and exactly the environment `env`, never
/// openable by another process of the user (short of `SYSTEM` or a
/// debug-privileged administrator): process and first thread are created
/// with the protected DACLs, and the token's default DACL is the protected
/// thread DACL before the process runs. Only for programs that start no
/// children and do not reopen objects they create without a descriptor
/// (the credential broker). No console; stdin and stderr are closed;
/// stdout is a pipe whose read end is returned. Only that pipe's write end
/// is inherited (it is inheritable for the duration of the call, so a
/// process spawned concurrently by `std` could inherit it too).
pub fn spawn_protected(
    program: &Path,
    args: &[OsString],
    env: &[(OsString, OsString)],
) -> io::Result<(ProtectedChild, File)> {
    let user_sid = current_user_sid_string()?;
    let process_descriptor = SecurityDescriptor::from_sddl(&process_dacl_sddl(&user_sid))?;
    let thread_descriptor = SecurityDescriptor::from_sddl(&thread_dacl_sddl(&user_sid))?;
    let process_attributes = security_attributes(&process_descriptor);
    let thread_attributes = security_attributes(&thread_descriptor);

    let (stdout_read, stdout_write) = pipe()?;
    let mut handles = [stdout_write.as_raw_handle() as HANDLE];
    let mut attributes = AttributeList::with_handles(&mut handles)?;

    let mut command_line = command_line(program, args);
    let mut environment = environment_block(env)?;
    let application: Vec<u16> = program.as_os_str().encode_wide().chain([0]).collect();
    // SAFETY: zeroed POD; the fields set below are the only ones used.
    let mut startup: STARTUPINFOEXW = unsafe { std::mem::zeroed() };
    startup.StartupInfo.cb = std::mem::size_of::<STARTUPINFOEXW>() as u32;
    startup.StartupInfo.dwFlags = STARTF_USESTDHANDLES;
    startup.StartupInfo.hStdOutput = handles[0];
    startup.lpAttributeList = attributes.as_mut_ptr();
    // SAFETY: zeroed out-structure.
    let mut info: PROCESS_INFORMATION = unsafe { std::mem::zeroed() };
    // SAFETY: the write end is ours; it is inheritable only for this call.
    if unsafe { SetHandleInformation(handles[0], HANDLE_FLAG_INHERIT, HANDLE_FLAG_INHERIT) } == 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: every pointer refers to a live, NUL-terminated or sized buffer
    // owned by this function; the handle list names one inheritable handle.
    let created = unsafe {
        CreateProcessW(
            application.as_ptr(),
            command_line.as_mut_ptr(),
            &process_attributes,
            &thread_attributes,
            1,
            CREATE_SUSPENDED
                | CREATE_UNICODE_ENVIRONMENT
                | DETACHED_PROCESS
                | EXTENDED_STARTUPINFO_PRESENT,
            environment.as_mut_ptr().cast(),
            std::ptr::null(),
            &startup.StartupInfo,
            &mut info,
        )
    };
    let create_error = io::Error::last_os_error();
    drop(attributes);
    // The child holds its own copy now.
    drop(stdout_write);
    if created == 0 {
        return Err(create_error);
    }
    // SAFETY: CreateProcessW returned owned handles.
    let process = unsafe { OwnedHandle::from_raw_handle(info.hProcess as _) };
    // SAFETY: as above.
    let thread = unsafe { OwnedHandle::from_raw_handle(info.hThread as _) };
    let mut child = ProtectedChild {
        process,
        pid: info.dwProcessId,
    };
    let started = protect_child_token(&child).and_then(|()| {
        // SAFETY: the suspended first thread; resumed once.
        if unsafe { ResumeThread(thread.as_raw_handle() as HANDLE) } == u32::MAX {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    });
    if let Err(err) = started {
        let _ = child.kill();
        let _ = child.wait();
        return Err(err);
    }
    Ok((child, File::from(stdout_read)))
}

/// Sets the suspended child's default DACL to the protected thread DACL.
fn protect_child_token(child: &ProtectedChild) -> io::Result<()> {
    let mut token: HANDLE = 0;
    // SAFETY: `child` holds a full-access process handle.
    if unsafe { OpenProcessToken(child.raw(), TOKEN_ADJUST_DEFAULT | TOKEN_QUERY, &mut token) } == 0
    {
        return Err(io::Error::last_os_error());
    }
    let result = set_protected_default_dacl(token);
    // SAFETY: opened above.
    unsafe { CloseHandle(token) };
    result
}

fn security_attributes(descriptor: &SecurityDescriptor) -> SECURITY_ATTRIBUTES {
    SECURITY_ATTRIBUTES {
        nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: descriptor.as_ptr(),
        bInheritHandle: 0,
    }
}

/// A pipe whose ends are not inheritable.
fn pipe() -> io::Result<(OwnedHandle, OwnedHandle)> {
    let mut read: HANDLE = 0;
    let mut write: HANDLE = 0;
    // SAFETY: valid out pointers; default security, not inheritable.
    if unsafe { CreatePipe(&mut read, &mut write, std::ptr::null(), 0) } == 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: CreatePipe returned two owned handles.
    Ok(unsafe {
        (
            OwnedHandle::from_raw_handle(read as _),
            OwnedHandle::from_raw_handle(write as _),
        )
    })
}

/// A process attribute list naming the only handles to inherit.
struct AttributeList {
    buffer: Vec<u64>,
}

impl AttributeList {
    /// `handles` must outlive the list.
    fn with_handles(handles: &mut [HANDLE]) -> io::Result<Self> {
        let mut size = 0_usize;
        // SAFETY: a size query.
        unsafe { InitializeProcThreadAttributeList(std::ptr::null_mut(), 1, 0, &mut size) };
        let mut list = Self {
            buffer: vec![0_u64; size.div_ceil(8)],
        };
        // SAFETY: `buffer` holds at least `size` bytes.
        if unsafe { InitializeProcThreadAttributeList(list.as_mut_ptr(), 1, 0, &mut size) } == 0 {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: `handles` outlives the list (caller contract).
        let ok = unsafe {
            UpdateProcThreadAttribute(
                list.as_mut_ptr(),
                0,
                PROC_THREAD_ATTRIBUTE_HANDLE_LIST as usize,
                handles.as_mut_ptr().cast(),
                std::mem::size_of_val(handles),
                std::ptr::null_mut(),
                std::ptr::null(),
            )
        };
        if ok == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(list)
    }

    fn as_mut_ptr(&mut self) -> LPPROC_THREAD_ATTRIBUTE_LIST {
        self.buffer.as_mut_ptr().cast()
    }
}

impl Drop for AttributeList {
    fn drop(&mut self) {
        // SAFETY: initialized in `with_handles`.
        unsafe { DeleteProcThreadAttributeList(self.as_mut_ptr()) };
    }
}

/// `program` and `args` quoted for `CommandLineToArgvW` / the MSVC runtime.
pub(crate) fn command_line(program: &Path, args: &[OsString]) -> Vec<u16> {
    let mut line = Vec::new();
    append_argument(&mut line, program.as_os_str());
    for arg in args {
        line.push(u16::from(b' '));
        append_argument(&mut line, arg);
    }
    line.push(0);
    line
}

fn append_argument(line: &mut Vec<u16>, arg: &OsStr) {
    let wide: Vec<u16> = arg.encode_wide().collect();
    let quote = wide.is_empty()
        || wide
            .iter()
            .any(|&c| c == u16::from(b' ') || c == u16::from(b'\t') || c == u16::from(b'"'));
    if !quote {
        line.extend(wide);
        return;
    }
    line.push(u16::from(b'"'));
    let mut backslashes = 0;
    for c in wide {
        if c == u16::from(b'\\') {
            backslashes += 1;
            continue;
        }
        // Backslashes before a quote are doubled, and the quote escaped.
        let repeat = if c == u16::from(b'"') {
            backslashes * 2 + 1
        } else {
            backslashes
        };
        line.extend(std::iter::repeat_n(u16::from(b'\\'), repeat));
        line.push(c);
        backslashes = 0;
    }
    // Backslashes before the closing quote are doubled.
    line.extend(std::iter::repeat_n(u16::from(b'\\'), backslashes * 2));
    line.push(u16::from(b'"'));
}

/// A sorted, double-NUL-terminated `NAME=value` block.
pub(crate) fn environment_block(env: &[(OsString, OsString)]) -> io::Result<Vec<u16>> {
    let mut entries: Vec<(Vec<u16>, Vec<u16>)> = env
        .iter()
        .map(|(name, value)| (name.encode_wide().collect(), value.encode_wide().collect()))
        .collect();
    if entries.iter().any(|(name, value)| {
        name.is_empty()
            || name.contains(&u16::from(b'='))
            || name.contains(&0)
            || value.contains(&0)
    }) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "invalid environment variable",
        ));
    }
    // Windows expects the block sorted case-insensitively.
    entries.sort_by(|(a, _), (b, _)| {
        String::from_utf16_lossy(a)
            .to_uppercase()
            .cmp(&String::from_utf16_lossy(b).to_uppercase())
    });
    let mut block = Vec::new();
    for (name, value) in entries {
        block.extend(name);
        block.push(u16::from(b'='));
        block.extend(value);
        block.push(0);
    }
    if block.is_empty() {
        block.push(0);
    }
    block.push(0);
    Ok(block)
}

#[cfg(test)]
#[path = "windows_protected_spawn_tests.rs"]
mod tests;
