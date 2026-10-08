//! PF-27-S06: on Windows, keep other processes from reading this process's
//! memory or environment, or taking over its threads.
//!
//! A process's default DACL gives its user full access, so any process of the
//! same user (including a command under the restricted-token sandbox, whose
//! token restricts writes only) can open it with `PROCESS_VM_READ` and read
//! its memory and environment block, and open its threads to read or set
//! their register context. This replaces the process DACL with one that gives
//! the user only `PROCESS_QUERY_LIMITED_INFORMATION` and `SYNCHRONIZE`, and
//! every thread's (existing ones now, later ones as they start) with
//! `THREAD_QUERY_LIMITED_INFORMATION` and `SYNCHRONIZE`. An `OWNER RIGHTS`
//! entry limited to `READ_CONTROL` removes the owner's implicit `WRITE_DAC`,
//! so a same-user process cannot grant itself access back.
//!
//! Limits: `SYSTEM` and administrators with `SeDebugPrivilege` enabled are not
//! stopped. Handles opened before the call keep their access, and the
//! environment block exists from process start, so call this before any
//! untrusted process can run, and never hand secrets over through the
//! environment of a process started after it. The user can still read the
//! image path and command line (query-limited access).

use std::ffi::c_void;
use std::io;
use std::ptr;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use windows_sys::Win32::Foundation::CloseHandle;
use windows_sys::Win32::Foundation::ERROR_SUCCESS;
use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::Foundation::HLOCAL;
use windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE;
use windows_sys::Win32::Foundation::LocalFree;
use windows_sys::Win32::Security::ACL;
use windows_sys::Win32::Security::Authorization::ConvertSidToStringSidW;
use windows_sys::Win32::Security::Authorization::ConvertStringSecurityDescriptorToSecurityDescriptorW;
use windows_sys::Win32::Security::Authorization::SDDL_REVISION_1;
use windows_sys::Win32::Security::Authorization::SE_KERNEL_OBJECT;
use windows_sys::Win32::Security::Authorization::SetSecurityInfo;
use windows_sys::Win32::Security::DACL_SECURITY_INFORMATION;
use windows_sys::Win32::Security::GetSecurityDescriptorDacl;
use windows_sys::Win32::Security::GetTokenInformation;
use windows_sys::Win32::Security::PROTECTED_DACL_SECURITY_INFORMATION;
use windows_sys::Win32::Security::PSECURITY_DESCRIPTOR;
use windows_sys::Win32::Security::TOKEN_QUERY;
use windows_sys::Win32::Security::TOKEN_USER;
use windows_sys::Win32::Security::TokenUser;
use windows_sys::Win32::System::Diagnostics::ToolHelp::CreateToolhelp32Snapshot;
use windows_sys::Win32::System::Diagnostics::ToolHelp::TH32CS_SNAPTHREAD;
use windows_sys::Win32::System::Diagnostics::ToolHelp::THREADENTRY32;
use windows_sys::Win32::System::Diagnostics::ToolHelp::Thread32First;
use windows_sys::Win32::System::Diagnostics::ToolHelp::Thread32Next;
use windows_sys::Win32::System::Threading::GetCurrentProcess;
use windows_sys::Win32::System::Threading::GetCurrentProcessId;
use windows_sys::Win32::System::Threading::GetCurrentThread;
use windows_sys::Win32::System::Threading::OpenProcessToken;
use windows_sys::Win32::System::Threading::OpenThread;
use windows_sys::Win32::System::Threading::PROCESS_QUERY_LIMITED_INFORMATION;
use windows_sys::Win32::System::Threading::THREAD_QUERY_LIMITED_INFORMATION;

/// `SYNCHRONIZE`: lets the user wait for the process to exit.
const SYNCHRONIZE: u32 = 0x0010_0000;

/// `WRITE_DAC`: needed to replace an existing thread's DACL.
const WRITE_DAC: u32 = 0x0004_0000;
/// `DLL_THREAD_ATTACH`, the TLS callback reason for a new thread.
const DLL_THREAD_ATTACH: u32 = 2;

/// The thread descriptor (a leaked, self-relative allocation) once hardened;
/// 0 before. Read by the TLS callback for every new thread.
static THREAD_DESCRIPTOR: AtomicUsize = AtomicUsize::new(0);

/// Replaces the current process's DACL, and that of each of its threads, so
/// that no other process (of this or another user, short of `SYSTEM` or a
/// debug-privileged administrator) can read its memory, read its
/// environment, duplicate its handles, read or set a thread's context, or
/// change either DACL. Threads started later get the thread DACL as they
/// start. Idempotent.
pub fn restrict_current_process_access() -> io::Result<()> {
    let user_sid = current_user_sid_string()?;
    protect_threads(&user_sid)?;
    let sddl = process_dacl_sddl(&user_sid);
    let descriptor = SecurityDescriptor::from_sddl(&sddl)?;
    let dacl = descriptor.dacl()?;
    // SAFETY: the pseudo-handle is always valid; `dacl` points into
    // `descriptor`, which outlives the call.
    let status = unsafe {
        SetSecurityInfo(
            GetCurrentProcess(),
            SE_KERNEL_OBJECT,
            DACL_SECURITY_INFORMATION | PROTECTED_DACL_SECURITY_INFORMATION,
            ptr::null_mut(),
            ptr::null_mut(),
            dacl,
            ptr::null(),
        )
    };
    if status == ERROR_SUCCESS {
        Ok(())
    } else {
        Err(io::Error::from_raw_os_error(status as i32))
    }
}

/// The protected DACL applied by [`restrict_current_process_access`].
pub fn process_dacl_sddl(user_sid: &str) -> String {
    protected_dacl_sddl(user_sid, PROCESS_QUERY_LIMITED_INFORMATION | SYNCHRONIZE)
}

/// The protected DACL applied to every thread.
pub fn thread_dacl_sddl(user_sid: &str) -> String {
    protected_dacl_sddl(user_sid, THREAD_QUERY_LIMITED_INFORMATION | SYNCHRONIZE)
}

fn protected_dacl_sddl(user_sid: &str, user_mask: u32) -> String {
    // GA for SYSTEM; READ_CONTROL only for OWNER RIGHTS (S-1-3-4).
    format!("D:P(A;;0x{user_mask:x};;;{user_sid})(A;;GA;;;SY)(A;;RC;;;OW)")
}

#[link(name = "ntdll")]
unsafe extern "system" {
    /// Sets a kernel object's security from a self-relative descriptor; safe
    /// to call under the loader lock (unlike most advapi32 functions).
    fn NtSetSecurityObject(handle: HANDLE, information: u32, descriptor: *mut c_void) -> i32;
}

/// Applies the thread DACL to every existing thread of this process and arms
/// the TLS callback for threads started later.
fn protect_threads(user_sid: &str) -> io::Result<()> {
    if THREAD_DESCRIPTOR.load(Ordering::Acquire) == 0 {
        let descriptor = SecurityDescriptor::from_sddl(&thread_dacl_sddl(user_sid))?;
        // Kept for the life of the process: new threads use it.
        let raw = descriptor.0 as usize;
        std::mem::forget(descriptor);
        if THREAD_DESCRIPTOR
            .compare_exchange(0, raw, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            // SAFETY: allocated above and never published.
            unsafe { LocalFree(raw as HLOCAL) };
        }
    }
    let descriptor = THREAD_DESCRIPTOR.load(Ordering::Acquire) as *mut c_void;
    // SAFETY: a snapshot of all threads; closed below.
    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0) };
    if snapshot == INVALID_HANDLE_VALUE {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: plain query.
    let own_pid = unsafe { GetCurrentProcessId() };
    // SAFETY: zeroed POD with its size set, as Thread32First requires.
    let mut entry: THREADENTRY32 = unsafe { std::mem::zeroed() };
    entry.dwSize = std::mem::size_of::<THREADENTRY32>() as u32;
    let mut result = Ok(());
    // SAFETY: `snapshot` is open and `entry` is valid.
    let mut more = unsafe { Thread32First(snapshot, &mut entry) } != 0;
    while more {
        if entry.th32OwnerProcessID == own_pid {
            // SAFETY: opens one of our own threads for WRITE_DAC; closed below.
            let thread = unsafe { OpenThread(WRITE_DAC, 0, entry.th32ThreadID) };
            // A thread that exited since the snapshot cannot be opened.
            if thread != 0 {
                // SAFETY: `thread` is open; `descriptor` is a live descriptor.
                let status =
                    unsafe { NtSetSecurityObject(thread, DACL_SECURITY_INFORMATION, descriptor) };
                // SAFETY: opened above.
                unsafe { CloseHandle(thread) };
                if status < 0 && result.is_ok() {
                    result = Err(io::Error::other(format!(
                        "NtSetSecurityObject(thread) failed: {status:#x}"
                    )));
                }
            }
        }
        // SAFETY: as above.
        more = unsafe { Thread32Next(snapshot, &mut entry) } != 0;
    }
    // SAFETY: opened above.
    unsafe { CloseHandle(snapshot) };
    result
}

/// TLS callback: protects each thread as it starts, once hardened.
unsafe extern "system" fn on_thread_event(
    _module: *mut c_void,
    reason: u32,
    _reserved: *mut c_void,
) {
    if reason != DLL_THREAD_ATTACH {
        return;
    }
    let descriptor = THREAD_DESCRIPTOR.load(Ordering::Acquire);
    if descriptor != 0 {
        // SAFETY: the pseudo-handle is valid for this thread; the descriptor
        // is never freed once published.
        unsafe {
            NtSetSecurityObject(
                GetCurrentThread(),
                DACL_SECURITY_INFORMATION,
                descriptor as *mut c_void,
            );
        }
    }
}

/// Registers [`on_thread_event`] as a TLS callback (`.CRT$XL*`, where the C
/// runtime collects them and the loader calls them for each new thread).
#[used]
#[unsafe(link_section = ".CRT$XLC")]
static THREAD_ATTACH_CALLBACK: unsafe extern "system" fn(*mut c_void, u32, *mut c_void) =
    on_thread_event;

/// The current process user's SID in string form (`S-1-5-21-...`).
pub fn current_user_sid_string() -> io::Result<String> {
    let mut token: HANDLE = 0;
    // SAFETY: the pseudo-handle is valid and `token` is a valid out pointer.
    if unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) } == 0 {
        return Err(io::Error::last_os_error());
    }
    let result = token_user_sid_string(token);
    // SAFETY: `token` was opened above and is closed once.
    unsafe { CloseHandle(token) };
    result
}

fn token_user_sid_string(token: HANDLE) -> io::Result<String> {
    let mut needed = 0_u32;
    // SAFETY: a size query with a null buffer.
    unsafe { GetTokenInformation(token, TokenUser, ptr::null_mut(), 0, &mut needed) };
    if needed == 0 {
        return Err(io::Error::last_os_error());
    }
    // u64 storage keeps TOKEN_USER suitably aligned.
    let mut buffer = vec![0_u64; (needed as usize).div_ceil(8)];
    // SAFETY: `buffer` holds at least `needed` bytes.
    let ok = unsafe {
        GetTokenInformation(
            token,
            TokenUser,
            buffer.as_mut_ptr().cast(),
            needed,
            &mut needed,
        )
    };
    if ok == 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: the call above filled a TOKEN_USER at the start of `buffer`.
    let sid = unsafe { (*buffer.as_ptr().cast::<TOKEN_USER>()).User.Sid };
    let mut wide: *mut u16 = ptr::null_mut();
    // SAFETY: `sid` points into `buffer`; `wide` is freed below.
    if unsafe { ConvertSidToStringSidW(sid, &mut wide) } == 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: `wide` is a NUL-terminated string allocated by the call above.
    let text = unsafe {
        let len = (0..).take_while(|&i| *wide.add(i) != 0).count();
        String::from_utf16_lossy(std::slice::from_raw_parts(wide, len))
    };
    // SAFETY: allocated with LocalAlloc by ConvertSidToStringSidW.
    unsafe { LocalFree(wide as HLOCAL) };
    Ok(text)
}

struct SecurityDescriptor(PSECURITY_DESCRIPTOR);

impl SecurityDescriptor {
    fn from_sddl(sddl: &str) -> io::Result<Self> {
        let wide: Vec<u16> = sddl.encode_utf16().chain(std::iter::once(0)).collect();
        let mut descriptor: PSECURITY_DESCRIPTOR = ptr::null_mut();
        // SAFETY: `wide` is NUL-terminated; `descriptor` is freed on drop.
        let ok = unsafe {
            ConvertStringSecurityDescriptorToSecurityDescriptorW(
                wide.as_ptr(),
                SDDL_REVISION_1,
                &mut descriptor,
                ptr::null_mut(),
            )
        };
        if ok == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(Self(descriptor))
    }

    fn dacl(&self) -> io::Result<*const ACL> {
        let mut present = 0;
        let mut defaulted = 0;
        let mut dacl: *mut ACL = ptr::null_mut();
        // SAFETY: `self.0` is a valid descriptor; out pointers are valid.
        let ok =
            unsafe { GetSecurityDescriptorDacl(self.0, &mut present, &mut dacl, &mut defaulted) };
        if ok == 0 {
            return Err(io::Error::last_os_error());
        }
        if present == 0 || dacl.is_null() {
            return Err(io::Error::other("security descriptor has no DACL"));
        }
        Ok(dacl)
    }
}

impl Drop for SecurityDescriptor {
    fn drop(&mut self) {
        // SAFETY: allocated with LocalAlloc by the SDDL conversion.
        unsafe { LocalFree(self.0 as HLOCAL) };
    }
}

#[cfg(test)]
#[path = "windows_process_access_tests.rs"]
mod tests;
