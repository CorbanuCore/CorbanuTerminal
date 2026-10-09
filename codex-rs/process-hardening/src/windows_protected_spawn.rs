//! PF-27-S07: starts a process that no other process of the user can open
//! at any point of its life.
//!
//! `std::process::Command` creates the process and its first thread with the
//! creator's default DACL, so a same-user process can open either before the
//! child hardens itself and keep the handle. Here both get the protected
//! DACLs at creation, the process starts suspended, its token's default DACL
//! becomes the protected thread DACL (so every thread it starts later is
//! protected from creation too), and only then does it run.
//!
//! PF-27-S08: the process runs under the broker token
//! (`windows_broker_token`), which already carries that default DACL.

use crate::windows_broker_token::BrokerDefaultDacl;
use crate::windows_broker_token::create_broker_token;
use crate::windows_handle_holder::HandleHolder;
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
use windows_sys::Win32::System::Threading::PROC_THREAD_ATTRIBUTE_PARENT_PROCESS;
use windows_sys::Win32::System::Threading::PROCESS_INFORMATION;
use windows_sys::Win32::System::Threading::ResumeThread;
use windows_sys::Win32::System::Threading::STARTF_USESTDHANDLES;
use windows_sys::Win32::System::Threading::STARTUPINFOEXW;
use windows_sys::Win32::System::Threading::TerminateProcess;
use windows_sys::Win32::System::Threading::UpdateProcThreadAttribute;
use windows_sys::Win32::System::Threading::WaitForSingleObject;

/// Set by [`spawn_protected`] in the child's environment: the id of the
/// process that started it (Windows reports a [`HandleHolder`] as its
/// parent).
pub const PROTECTED_SPAWNER_PID_ENV: &str = "CODEX_PROTECTED_SPAWNER_PID";

/// In a process started by [`spawn_protected`]: the id of the process that
/// started it. Like a parent process id, it can name a process that has
/// exited (and whose id was reused).
pub fn protected_spawner_pid() -> Option<u32> {
    std::env::var(PROTECTED_SPAWNER_PID_ENV).ok()?.parse().ok()
}

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
/// is inherited, and it is never inheritable in this process's handle table
/// (#320): it reaches the child through a [`HandleHolder`] (with the
/// protected DACLs too), so the child's parent process is that holder, which
/// is gone once this returns. The child finds this process's id in
/// [`PROTECTED_SPAWNER_PID_ENV`] instead ([`protected_spawner_pid`]).
///
/// PF-27-S08: the child runs under a new broker token (low integrity,
/// write-restricted to a fresh capability SID, no privileges; see
/// `windows_broker_token`).
pub fn spawn_protected(
    program: &Path,
    args: &[OsString],
    env: &[(OsString, OsString)],
) -> io::Result<(ProtectedChild, File)> {
    spawn_protected_with(program, args, env, Confinement::BrokerToken)
}

/// What a protected child runs under.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Confinement {
    /// A copy of this process's token (PF-27-S07), its default DACL made
    /// the protected thread DACL before the child runs.
    #[cfg_attr(not(test), allow(dead_code))]
    SameToken,
    /// The broker token (PF-27-S08).
    BrokerToken,
}

pub(crate) fn spawn_protected_with(
    program: &Path,
    args: &[OsString],
    env: &[(OsString, OsString)],
    confinement: Confinement,
) -> io::Result<(ProtectedChild, File)> {
    let (mut child, stdout, thread) = spawn_protected_suspended(program, args, env, confinement)?;
    // SAFETY: the suspended first thread; resumed once.
    if unsafe { ResumeThread(thread.as_raw_handle() as HANDLE) } == u32::MAX {
        let error = io::Error::last_os_error();
        kill_unstarted(&mut child);
        return Err(error);
    }
    Ok((child, stdout))
}

/// [`spawn_protected`] up to, not including, resuming the first thread,
/// whose handle is returned.
pub(crate) fn spawn_protected_suspended(
    program: &Path,
    args: &[OsString],
    env: &[(OsString, OsString)],
    confinement: Confinement,
) -> io::Result<(ProtectedChild, File, OwnedHandle)> {
    let token = match confinement {
        Confinement::BrokerToken => Some(create_broker_token(BrokerDefaultDacl::Protected)?),
        Confinement::SameToken => None,
    };
    let user_sid = current_user_sid_string()?;
    let process_descriptor = SecurityDescriptor::from_sddl(&process_dacl_sddl(&user_sid))?;
    let thread_descriptor = SecurityDescriptor::from_sddl(&thread_dacl_sddl(&user_sid))?;
    let process_attributes = security_attributes(&process_descriptor);
    let thread_attributes = security_attributes(&thread_descriptor);

    let (stdout_read, stdout_write) = pipe()?;
    // The child inherits the holder's token.
    let holder = HandleHolder::start_with(
        program,
        Some(&process_attributes),
        Some(&thread_attributes),
        token.as_ref().map(|token| token.as_raw_handle() as HANDLE),
    )?;
    let mut handles = [holder.hold(stdout_write.as_raw_handle() as HANDLE)?];
    // The holder has its own copy now.
    drop(stdout_write);
    let mut attributes = AttributeList::new(holder.raw(), &mut handles)?;

    let mut command_line = command_line(program, args)?;
    // Later entries win, so the child cannot be handed another spawner.
    let spawner = (
        OsString::from(PROTECTED_SPAWNER_PID_ENV),
        OsString::from(std::process::id().to_string()),
    );
    let mut environment = environment_block(&[env, &[spawner]].concat())?;
    let application: Vec<u16> = program.as_os_str().encode_wide().chain([0]).collect();
    // SAFETY: zeroed POD; the fields set below are the only ones used.
    let mut startup: STARTUPINFOEXW = unsafe { std::mem::zeroed() };
    startup.StartupInfo.cb = std::mem::size_of::<STARTUPINFOEXW>() as u32;
    startup.StartupInfo.dwFlags = STARTF_USESTDHANDLES;
    // A value in the holder's table, as the parent-process attribute requires.
    startup.StartupInfo.hStdOutput = handles[0];
    startup.lpAttributeList = attributes.as_mut_ptr();
    // SAFETY: zeroed out-structure.
    let mut info: PROCESS_INFORMATION = unsafe { std::mem::zeroed() };
    // SAFETY: every pointer refers to a live, NUL-terminated or sized buffer
    // owned by this function; the child inherits from the holder, and only
    // the one inheritable handle in the list.
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
    // The child holds its own copy now; ending the holder closes the other.
    drop(holder);
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
    // The broker token has its default DACL already.
    if confinement == Confinement::SameToken
        && let Err(err) = protect_child_token(&child)
    {
        kill_unstarted(&mut child);
        return Err(err);
    }
    Ok((child, File::from(stdout_read), thread))
}

/// Kills a child that never ran; reaps it only if the kill succeeded.
fn kill_unstarted(child: &mut ProtectedChild) {
    if child.kill().is_ok() {
        let _ = child.wait();
    }
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

/// A process attribute list naming the parent process to inherit from and
/// the only handles (values in the parent's table) to inherit.
struct AttributeList {
    buffer: Vec<u64>,
    /// Boxed: the list refers to it by address.
    parent: Box<HANDLE>,
}

impl AttributeList {
    /// `handles` must outlive the list; `parent` needs
    /// `PROCESS_CREATE_PROCESS` and `PROCESS_DUP_HANDLE` access.
    fn new(parent: HANDLE, handles: &mut [HANDLE]) -> io::Result<Self> {
        let mut size = 0_usize;
        // SAFETY: a size query.
        unsafe { InitializeProcThreadAttributeList(std::ptr::null_mut(), 2, 0, &mut size) };
        let mut list = Self {
            buffer: vec![0_u64; size.div_ceil(8)],
            parent: Box::new(parent),
        };
        // SAFETY: `buffer` holds at least `size` bytes.
        if unsafe { InitializeProcThreadAttributeList(list.as_mut_ptr(), 2, 0, &mut size) } == 0 {
            return Err(io::Error::last_os_error());
        }
        let parent = std::ptr::from_mut::<HANDLE>(&mut *list.parent).cast();
        // SAFETY: `handles` outlives the list (caller contract); the boxed
        // parent handle stays at its address as long as the list.
        let ok = unsafe {
            UpdateProcThreadAttribute(
                list.as_mut_ptr(),
                0,
                PROC_THREAD_ATTRIBUTE_PARENT_PROCESS as usize,
                parent,
                std::mem::size_of::<HANDLE>(),
                std::ptr::null_mut(),
                std::ptr::null(),
            ) != 0
                && UpdateProcThreadAttribute(
                    list.as_mut_ptr(),
                    0,
                    PROC_THREAD_ATTRIBUTE_HANDLE_LIST as usize,
                    handles.as_mut_ptr().cast(),
                    std::mem::size_of_val(handles),
                    std::ptr::null_mut(),
                    std::ptr::null(),
                ) != 0
        };
        if !ok {
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
        // SAFETY: initialized in `new`.
        unsafe { DeleteProcThreadAttributeList(self.as_mut_ptr()) };
    }
}

/// `program` and `args` quoted for `CommandLineToArgvW` / the MSVC runtime.
/// Refuses NUL, which would cut the command line short.
pub(crate) fn command_line(program: &Path, args: &[OsString]) -> io::Result<Vec<u16>> {
    let mut line = Vec::new();
    append_argument(&mut line, program.as_os_str());
    for arg in args {
        line.push(u16::from(b' '));
        append_argument(&mut line, arg);
    }
    if line.contains(&0) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "NUL in the program or an argument",
        ));
    }
    line.push(0);
    Ok(line)
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

/// A sorted, double-NUL-terminated `NAME=value` block. Names compare as
/// Windows does (ordinal, ASCII case-insensitive); a later duplicate wins.
/// A leading `=` is allowed (`cmd.exe`'s per-drive `=C:` variables).
pub(crate) fn environment_block(env: &[(OsString, OsString)]) -> io::Result<Vec<u16>> {
    let mut entries: Vec<(Vec<u16>, Vec<u16>)> = Vec::new();
    for (name, value) in env {
        let name: Vec<u16> = name.encode_wide().collect();
        entries.retain(|(known, _)| !same_name(known, &name));
        entries.push((name, value.encode_wide().collect()));
    }
    if entries.iter().any(|(name, value)| {
        name.is_empty()
            || name[1..].contains(&u16::from(b'='))
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
        a.iter()
            .map(|&unit| ascii_upper(unit))
            .cmp(b.iter().map(|&unit| ascii_upper(unit)))
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

fn ascii_upper(unit: u16) -> u16 {
    if (u16::from(b'a')..=u16::from(b'z')).contains(&unit) {
        unit - 32
    } else {
        unit
    }
}

fn same_name(a: &[u16], b: &[u16]) -> bool {
    a.len() == b.len()
        && a.iter()
            .zip(b)
            .all(|(&x, &y)| ascii_upper(x) == ascii_upper(y))
}
