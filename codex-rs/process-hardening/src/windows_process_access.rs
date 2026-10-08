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
//! stopped. Threads this image starts get the protected DACL at creation
//! (PF-27-S07, `windows_thread_creation`); a thread any other module starts
//! (Windows thread pools, RPC, injected DLLs) gets the token's default DACL
//! (the user has full access) and the protected one in its TLS callback, so
//! a same-user process outside the sandbox that opens it in that short
//! window keeps its handle, and threads created without loader notifications
//! (the loader's own workers) keep the default DACL. The credential broker
//! closes that too by changing its token's default DACL, which Core cannot
//! (`protect_new_objects_by_default`). A thread created protected has, on its
//! own pseudo-handle, only what Windows computes from that DACL plus its
//! baseline (terminate, set and query information): it cannot change its own
//! DACL or impersonate; Core and the broker do neither.
//! Commands under the elevated sandbox run as another user and are not
//! granted either way. Handles opened before the call keep their access, and the
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
use windows_sys::Win32::Foundation::ERROR_ACCESS_DENIED;
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
/// New threads the TLS callback could not protect.
static THREAD_PROTECT_FAILURES: AtomicUsize = AtomicUsize::new(0);

/// Replaces the current process's DACL, and that of each of its threads, so
/// that no other process (of this or another user, short of `SYSTEM` or a
/// debug-privileged administrator) can read its memory, read its
/// environment, duplicate its handles, read or set a thread's context, or
/// change either DACL. Threads this image starts later get the thread DACL
/// at creation, others as they start. Idempotent.
pub fn restrict_current_process_access() -> io::Result<()> {
    if !thread_callback_registered() {
        return Err(io::Error::other(
            "the thread-protection TLS callback is not in this image's TLS directory",
        ));
    }
    let user_sid = current_user_sid_string()?;
    // Even if thread protection fails, the process DACL still applies.
    let threads = protect_threads(&user_sid);
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
    // A process started by `spawn_protected` already has this DACL and, its
    // own handle being computed from it, may not rewrite it.
    if status != ERROR_SUCCESS
        && !(status == ERROR_ACCESS_DENIED && current_process_dacl_is_protected())
    {
        return Err(io::Error::from_raw_os_error(status as i32));
    }
    threads
}

/// True when this process's DACL grants nothing beyond query-limited,
/// synchronize and read-control to anyone but `SYSTEM`.
fn current_process_dacl_is_protected() -> bool {
    use windows_sys::Win32::Security::ACCESS_ALLOWED_ACE;
    use windows_sys::Win32::Security::ACE_HEADER;
    use windows_sys::Win32::Security::ACL_SIZE_INFORMATION;
    use windows_sys::Win32::Security::AclSizeInformation;
    use windows_sys::Win32::Security::Authorization::GetSecurityInfo;
    use windows_sys::Win32::Security::GetAce;
    use windows_sys::Win32::Security::GetAclInformation;
    use windows_sys::Win32::Security::IsWellKnownSid;
    use windows_sys::Win32::Security::WinLocalSystemSid;
    const ACCESS_ALLOWED_ACE_TYPE: u8 = 0;
    const ACCESS_DENIED_ACE_TYPE: u8 = 1;
    const READ_CONTROL: u32 = 0x0002_0000;
    let allowed = PROCESS_QUERY_LIMITED_INFORMATION | SYNCHRONIZE | READ_CONTROL;
    let mut dacl: *mut ACL = ptr::null_mut();
    let mut descriptor: PSECURITY_DESCRIPTOR = ptr::null_mut();
    // SAFETY: the pseudo-handle is valid; `descriptor` is freed below.
    let status = unsafe {
        GetSecurityInfo(
            GetCurrentProcess(),
            SE_KERNEL_OBJECT,
            DACL_SECURITY_INFORMATION,
            ptr::null_mut(),
            ptr::null_mut(),
            &mut dacl,
            ptr::null_mut(),
            &mut descriptor,
        )
    };
    if status != ERROR_SUCCESS {
        return false;
    }
    let mut protected = !dacl.is_null();
    // SAFETY: zeroed out-structure; `dacl` points into `descriptor`.
    let mut info: ACL_SIZE_INFORMATION = unsafe { std::mem::zeroed() };
    // SAFETY: `info` is valid for the size passed; `dacl` is non-null here.
    protected = protected
        && unsafe {
            GetAclInformation(
                dacl,
                (&mut info as *mut ACL_SIZE_INFORMATION).cast(),
                std::mem::size_of::<ACL_SIZE_INFORMATION>() as u32,
                AclSizeInformation,
            )
        } != 0;
    for index in 0..if protected { info.AceCount } else { 0 } {
        let mut ace: *mut c_void = ptr::null_mut();
        // SAFETY: `index` is below the ACE count of a valid ACL.
        if unsafe { GetAce(dacl, index, &mut ace) } == 0 {
            protected = false;
            break;
        }
        // SAFETY: every ACE starts with a header.
        let kind = unsafe { (*(ace as *const ACE_HEADER)).AceType };
        if kind == ACCESS_DENIED_ACE_TYPE {
            continue;
        }
        if kind != ACCESS_ALLOWED_ACE_TYPE {
            protected = false;
            break;
        }
        // SAFETY: an access-allowed ACE: header, mask, then the SID.
        let mask = unsafe { (*(ace as *const ACCESS_ALLOWED_ACE)).Mask };
        let sid = (ace as usize + std::mem::size_of::<ACE_HEADER>() + std::mem::size_of::<u32>())
            as *mut c_void;
        // SAFETY: `sid` points to the ACE's SID.
        let system = unsafe { IsWellKnownSid(sid, WinLocalSystemSid) } != 0;
        if !system && mask & !allowed != 0 {
            protected = false;
            break;
        }
    }
    // SAFETY: allocated by GetSecurityInfo.
    unsafe { LocalFree(descriptor as HLOCAL) };
    protected
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

/// Arms the TLS callback and the `CreateThread` redirect for threads started
/// later, then applies the thread DACL to every existing thread.
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
    // Existing threads are protected even if the redirect fails.
    let redirected = crate::windows_thread_creation::redirect_thread_creation();
    let descriptor = thread_descriptor();
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
    result.and(redirected)
}

/// `STATUS_ACCESS_DENIED`.
const STATUS_ACCESS_DENIED: i32 = 0xC000_0022_u32 as i32;

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
        let status = unsafe {
            NtSetSecurityObject(
                GetCurrentThread(),
                DACL_SECURITY_INFORMATION,
                descriptor as *mut c_void,
            )
        };
        // Access denied: the thread was created with a restrictive DACL
        // (ours, through the `CreateThread` redirect), which leaves it no
        // `WRITE_DAC` on itself; there is nothing to do.
        if status < 0 && status != STATUS_ACCESS_DENIED {
            THREAD_PROTECT_FAILURES.fetch_add(1, Ordering::Relaxed);
        }
    }
}

/// The protected thread descriptor once hardened (self-relative, never
/// freed), or null.
pub(crate) fn thread_descriptor() -> *mut c_void {
    THREAD_DESCRIPTOR.load(Ordering::Acquire) as *mut c_void
}

/// How many new threads could not be given the protected DACL.
pub fn thread_protection_failures() -> usize {
    THREAD_PROTECT_FAILURES.load(Ordering::Relaxed)
}

/// True when [`on_thread_event`] is listed in the TLS directory of the image
/// that contains it, so the loader calls it for new threads (the linker could
/// otherwise drop the section).
pub fn thread_callback_registered() -> bool {
    use windows_sys::Win32::System::LibraryLoader::GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS;
    use windows_sys::Win32::System::LibraryLoader::GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT;
    use windows_sys::Win32::System::LibraryLoader::GetModuleHandleExW;
    // SAFETY: a volatile read keeps the registration referenced.
    let callback = unsafe { std::ptr::read_volatile(&raw const THREAD_ATTACH_CALLBACK) } as usize;
    let mut module = 0;
    // SAFETY: looks up the module containing `callback`; no reference taken.
    let found = unsafe {
        GetModuleHandleExW(
            GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
            callback as *const u16,
            &mut module,
        )
    };
    if found == 0 {
        return false;
    }
    // SAFETY: `module` is the base of a mapped PE image; offsets follow the
    // PE format and stay inside its headers and TLS data.
    unsafe { tls_callbacks(module as *const u8).contains(&callback) }
}

/// The TLS callbacks listed in a mapped PE image.
///
/// # Safety
/// `base` must be the base address of a mapped PE image.
unsafe fn tls_callbacks(base: *const u8) -> Vec<usize> {
    const TLS_DIRECTORY_INDEX: usize = 9;
    let read_u32 = |offset: usize| unsafe { base.add(offset).cast::<u32>().read_unaligned() };
    let nt = read_u32(0x3c) as usize;
    if read_u32(nt) != 0x0000_4550 {
        return Vec::new();
    }
    let optional = nt + 4 + 20;
    let magic = unsafe { base.add(optional).cast::<u16>().read_unaligned() };
    let (directories, count_offset, wide) = match magic {
        0x20b => (optional + 112, optional + 108, true),
        0x10b => (optional + 96, optional + 92, false),
        _ => return Vec::new(),
    };
    if (read_u32(count_offset) as usize) <= TLS_DIRECTORY_INDEX {
        return Vec::new();
    }
    let tls_rva = read_u32(directories + TLS_DIRECTORY_INDEX * 8) as usize;
    if tls_rva == 0 {
        return Vec::new();
    }
    // AddressOfCallBacks is the fourth field: a virtual address.
    let mut entry = if wide {
        unsafe { base.add(tls_rva + 24).cast::<u64>().read_unaligned() as usize }
    } else {
        read_u32(tls_rva + 12) as usize
    };
    let mut callbacks = Vec::new();
    while entry != 0 && callbacks.len() < 64 {
        let callback = unsafe { (entry as *const usize).read_unaligned() };
        if callback == 0 {
            break;
        }
        callbacks.push(callback);
        entry += std::mem::size_of::<usize>();
    }
    callbacks
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

pub(crate) struct SecurityDescriptor(PSECURITY_DESCRIPTOR);

impl SecurityDescriptor {
    pub(crate) fn from_sddl(sddl: &str) -> io::Result<Self> {
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

    pub(crate) fn as_ptr(&self) -> *mut c_void {
        self.0
    }

    pub(crate) fn dacl(&self) -> io::Result<*const ACL> {
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
