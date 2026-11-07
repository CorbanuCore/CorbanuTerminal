//! PROBE ONLY (win-1560m2, never merged): two candidate hardening designs,
//! chosen at build time by `CORBANU_PROBE_VARIANT`.
//!
//! - `d367p` (#1721 (a′)): elevated commands run as the sandbox account
//!   (offline/online) while the command runner runs as a separate account
//!   ([`RUNNER_USERNAME`]) that no command ever holds. Core logs the
//!   command's account on itself and hands the runner that token; the runner
//!   needs `SeAssignPrimaryTokenPrivilege` to start a process with it.
//! - `x1560` (#1560): each command's own process, main thread and token get
//!   an explicit DACL when it is created (the process starts suspended until
//!   its token has one); the token's default DACL is main's.
//! - `px`: both.

use crate::token::well_known_sid_bytes;
use crate::winutil::to_wide;
use anyhow::Result;
use anyhow::anyhow;
use std::collections::HashMap;
use std::ffi::c_void;
use std::path::Path;
use std::sync::Mutex;
use windows_sys::Win32::Foundation::CloseHandle;
use windows_sys::Win32::Foundation::DUPLICATE_SAME_ACCESS;
use windows_sys::Win32::Foundation::DuplicateHandle;
use windows_sys::Win32::Foundation::ERROR_SUCCESS;
use windows_sys::Win32::Foundation::GetLastError;
use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::Foundation::HLOCAL;
use windows_sys::Win32::Foundation::LocalFree;
use windows_sys::Win32::Security::ACL;
use windows_sys::Win32::Security::Authorization::SE_KERNEL_OBJECT;
use windows_sys::Win32::Security::Authorization::SetSecurityInfo;
use windows_sys::Win32::Security::DACL_SECURITY_INFORMATION;
use windows_sys::Win32::Security::GetLengthSid;
use windows_sys::Win32::Security::InitializeSecurityDescriptor;
use windows_sys::Win32::Security::LOGON32_LOGON_INTERACTIVE;
use windows_sys::Win32::Security::LOGON32_PROVIDER_DEFAULT;
use windows_sys::Win32::Security::LogonUserW;
use windows_sys::Win32::Security::PROTECTED_DACL_SECURITY_INFORMATION;
use windows_sys::Win32::Security::SECURITY_ATTRIBUTES;
use windows_sys::Win32::Security::SECURITY_DESCRIPTOR;
use windows_sys::Win32::Security::SetSecurityDescriptorDacl;
use windows_sys::Win32::System::Threading::GetCurrentProcess;
use windows_sys::Win32::System::Threading::OpenProcessToken;
use windows_sys::Win32::System::Threading::PROCESS_INFORMATION;
use windows_sys::Win32::System::Threading::ResumeThread;

const VARIANT: Option<&str> = option_env!("CORBANU_PROBE_VARIANT");

/// The separate runner account of `d367p` (at most 20 characters).
pub const RUNNER_USERNAME: &str = "CodexSbxRunner";

pub fn d367p() -> bool {
    matches!(VARIANT, Some("d367p") | Some("px"))
}

pub fn x1560() -> bool {
    matches!(VARIANT, Some("x1560") | Some("px"))
}

const GENERIC_ALL: u32 = 0x1000_0000;
const DELETE: u32 = 0x0001_0000;
const READ_CONTROL: u32 = 0x0002_0000;
const WRITE_DAC: u32 = 0x0004_0000;
const SYNCHRONIZE: u32 = 0x0010_0000;
/// Standard rights the shared normal SID keeps; `WRITE_DAC` and `DELETE` are
/// write rights of a write-restricted token (#158), so only this command
/// passes the restricted check for them.
const STANDARD: u32 = READ_CONTROL | WRITE_DAC | DELETE;
/// Process: every write right (the restricted check keeps them to this
/// command) plus query and wait; not `PROCESS_VM_READ` (0x10), which isn't a
/// write right, nor `WRITE_OWNER`.
const PROCESS_SHARED: u32 = 0x1FEB | SYNCHRONIZE | STANDARD;
/// Thread: terminate, suspend, alert, set context/information, query
/// (limited); not get-context (0x8), set-thread-token (0x80), impersonate
/// (0x100) or direct impersonation (0x200).
const THREAD_SHARED: u32 = 0x0C77 | SYNCHRONIZE | STANDARD;
/// Token: query, query source and the adjust rights; not assign-primary,
/// duplicate or impersonate (none is a write right).
const TOKEN_SHARED: u32 = 0x01F8 | STANDARD;
const WIN_LOCAL_SYSTEM_SID: i32 = 22;
const WIN_CREATOR_OWNER_RIGHTS_SID: i32 = 71;

/// The command SID and shared normal SID (legacy: the user; elevated: the
/// sandbox users' group) of each command token made in this process, by
/// handle value.
static COMMAND_TOKENS: Mutex<Option<HashMap<isize, (Vec<u8>, Vec<u8>)>>> = Mutex::new(None);

unsafe fn sid_bytes(sid: *mut c_void) -> Vec<u8> {
    let len = GetLengthSid(sid) as usize;
    std::slice::from_raw_parts(sid as *const u8, len).to_vec()
}

/// Called by `create_token` for every command token.
pub(crate) unsafe fn register_command_token(token: HANDLE, command: *mut c_void, shared: *mut c_void) {
    if !x1560() {
        return;
    }
    let entry = (sid_bytes(command), sid_bytes(shared));
    if let Ok(mut guard) = COMMAND_TOKENS.lock() {
        guard.get_or_insert_with(HashMap::new).insert(token, entry);
    }
}

fn command_token_sids(token: HANDLE) -> Option<(Vec<u8>, Vec<u8>)> {
    COMMAND_TOKENS
        .lock()
        .ok()
        .and_then(|mut guard| guard.as_mut().and_then(|map| map.remove(&token)))
}

/// The ACL `SYSTEM: GENERIC_ALL, command: GENERIC_ALL, shared: shared_mask,
/// OWNER RIGHTS: READ_CONTROL` (the owner, the shared account, gets no
/// implicit `WRITE_DAC`). Free with `LocalFree`.
unsafe fn explicit_acl(command: &[u8], shared: &[u8], shared_mask: u32) -> Result<*mut ACL> {
    let mut system = well_known_sid_bytes(WIN_LOCAL_SYSTEM_SID)?;
    let mut owner_rights = well_known_sid_bytes(WIN_CREATOR_OWNER_RIGHTS_SID)?;
    let mut command = command.to_vec();
    let mut shared = shared.to_vec();
    crate::token::acl_from_grants_pub(&[
        (system.as_mut_ptr().cast(), GENERIC_ALL),
        (command.as_mut_ptr().cast(), GENERIC_ALL),
        (shared.as_mut_ptr().cast(), shared_mask),
        (owner_rights.as_mut_ptr().cast(), READ_CONTROL),
    ])
}

/// A security descriptor with an explicit protected DACL, for
/// `lpProcessAttributes`/`lpThreadAttributes`.
pub(crate) struct ExplicitSd {
    sd: Box<SECURITY_DESCRIPTOR>,
    acl: *mut ACL,
    pub attributes: SECURITY_ATTRIBUTES,
}

impl ExplicitSd {
    unsafe fn new(command: &[u8], shared: &[u8], mask: u32) -> Result<Box<Self>> {
        let acl = explicit_acl(command, shared, mask)?;
        let mut sd: Box<SECURITY_DESCRIPTOR> = Box::new(std::mem::zeroed());
        let psd = (&mut *sd as *mut SECURITY_DESCRIPTOR).cast::<c_void>();
        if InitializeSecurityDescriptor(psd, 1) == 0
            || SetSecurityDescriptorDacl(psd, 1, acl, 0) == 0
        {
            let err = GetLastError();
            LocalFree(acl as HLOCAL);
            return Err(anyhow!("security descriptor: {err}"));
        }
        let mut this = Box::new(Self {
            sd,
            acl,
            attributes: std::mem::zeroed(),
        });
        this.attributes = SECURITY_ATTRIBUTES {
            nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: (&mut *this.sd as *mut SECURITY_DESCRIPTOR).cast(),
            bInheritHandle: 0,
        };
        Ok(this)
    }
}

impl Drop for ExplicitSd {
    fn drop(&mut self) {
        // SAFETY: allocated by `SetEntriesInAclW`.
        unsafe { LocalFree(self.acl as HLOCAL) };
    }
}

/// x1560: what a command's creation takes, if `token` is a command token.
pub(crate) struct Explicit {
    pub process: Box<ExplicitSd>,
    pub thread: Box<ExplicitSd>,
    command: Vec<u8>,
    shared: Vec<u8>,
}

impl Explicit {
    pub(crate) fn process_attributes(&self) -> *const SECURITY_ATTRIBUTES {
        &self.process.attributes
    }

    pub(crate) fn thread_attributes(&self) -> *const SECURITY_ATTRIBUTES {
        &self.thread.attributes
    }
}

/// `None` unless `x1560` and `token` is a registered command token.
pub(crate) unsafe fn explicit_for(token: HANDLE, log_dir: Option<&Path>) -> Option<Explicit> {
    if !x1560() {
        return None;
    }
    let Some((command, shared)) = command_token_sids(token) else {
        crate::logging::log_note("x1560: unregistered token, main's DACLs", log_dir);
        return None;
    };
    let built = (|| -> Result<Explicit> {
        // SAFETY: plain allocations, freed on drop.
        Ok(Explicit {
            process: unsafe { ExplicitSd::new(&command, &shared, PROCESS_SHARED) }?,
            thread: unsafe { ExplicitSd::new(&command, &shared, THREAD_SHARED) }?,
            command: command.clone(),
            shared: shared.clone(),
        })
    })();
    match built {
        Ok(explicit) => {
            // The token object first, in case the process keeps it.
            let pre = set_token_dacl(token, &explicit);
            crate::logging::log_note(&format!("x1560: token pre={pre}"), log_dir);
            Some(explicit)
        }
        Err(err) => {
            crate::logging::log_note(&format!("x1560: {err:#}"), log_dir);
            None
        }
    }
}

unsafe fn set_token_dacl(token: HANDLE, explicit: &Explicit) -> String {
    match explicit_acl(&explicit.command, &explicit.shared, TOKEN_SHARED) {
        Ok(acl) => {
            let res = SetSecurityInfo(
                token,
                SE_KERNEL_OBJECT,
                DACL_SECURITY_INFORMATION | PROTECTED_DACL_SECURITY_INFORMATION,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                acl,
                std::ptr::null(),
            );
            LocalFree(acl as HLOCAL);
            if res == ERROR_SUCCESS {
                "ok".to_string()
            } else {
                format!("err{res}")
            }
        }
        Err(err) => format!("{err:#}"),
    }
}

/// After the suspended creation: the explicit DACL on the token the process
/// actually got, then resumes it. Returns a note for the log.
pub(crate) unsafe fn finish(pi: &PROCESS_INFORMATION, explicit: &Explicit, log_dir: Option<&Path>) {
    let mut token: HANDLE = 0;
    let post = if OpenProcessToken(pi.hProcess, READ_CONTROL | WRITE_DAC, &mut token) == 0 {
        format!("open_err{}", GetLastError())
    } else {
        let result = set_token_dacl(token, explicit);
        CloseHandle(token);
        result
    };
    let resumed = ResumeThread(pi.hThread);
    crate::logging::log_note(
        &format!("x1560: token post={post} resume={resumed}"),
        log_dir,
    );
}

/// The creation flag x1560 adds.
pub(crate) const CREATE_SUSPENDED: u32 = 0x0000_0004;

// ---- d367p: Core side ----

/// Logs `username` on, gives that logon this window station and desktop,
/// and duplicates the token into `runner` (Core's handle to the suspended or
/// running runner). Returns the handle value in the runner.
pub(crate) fn command_token_for_runner(
    username: &str,
    password: &str,
    runner: HANDLE,
    log_dir: Option<&Path>,
) -> Result<u64> {
    let user = to_wide(username);
    let domain = to_wide(".");
    let pass = to_wide(password);
    let mut token: HANDLE = 0;
    // SAFETY: valid NUL-terminated buffers; the token is closed below.
    let ok = unsafe {
        LogonUserW(
            user.as_ptr(),
            domain.as_ptr(),
            pass.as_ptr(),
            LOGON32_LOGON_INTERACTIVE,
            LOGON32_PROVIDER_DEFAULT,
            &mut token,
        )
    };
    if ok == 0 {
        return Err(anyhow!("d367p: LogonUserW({username}) failed: {}", unsafe {
            GetLastError()
        }));
    }
    let result = (|| -> Result<u64> {
        // SAFETY: a live token handle.
        let logon = unsafe { crate::token::get_logon_sid_bytes(token) }?;
        match crate::window_station::WindowAccess::probe_grant_command_logon(&logon) {
            // Kept until the window station goes away (probe only).
            Ok(access) => std::mem::forget(access),
            Err(err) => crate::logging::log_note(
                &format!("d367p: window access for the command logon: {err:#}"),
                log_dir,
            ),
        }
        let mut remote: HANDLE = 0;
        // SAFETY: both process handles are live; `remote` belongs to the runner.
        let ok = unsafe {
            DuplicateHandle(
                GetCurrentProcess(),
                token,
                runner,
                &mut remote,
                0,
                0,
                DUPLICATE_SAME_ACCESS,
            )
        };
        if ok == 0 {
            return Err(anyhow!("d367p: DuplicateHandle into the runner failed: {}", unsafe {
                GetLastError()
            }));
        }
        Ok(remote as u64)
    })();
    // SAFETY: from LogonUserW above.
    unsafe { CloseHandle(token) };
    result
}

// ---- d367p: runner side ----

static EXTRA_DESKTOP_SID: Mutex<Option<Vec<u8>>> = Mutex::new(None);

/// In the runner: the logon SID of the command token, which the private
/// desktop must grant as well as the runner's own.
pub fn set_extra_desktop_sid(token: HANDLE) -> Result<()> {
    // SAFETY: a live token handle.
    let sid = unsafe { crate::token::get_logon_sid_bytes(token) }?;
    if let Ok(mut guard) = EXTRA_DESKTOP_SID.lock() {
        *guard = Some(sid);
    }
    Ok(())
}

pub(crate) fn extra_desktop_sid() -> Option<Vec<u8>> {
    EXTRA_DESKTOP_SID.lock().ok().and_then(|guard| guard.clone())
}

/// In the runner: enables `SeAssignPrimaryTokenPrivilege` on `token` (its
/// own, opened before hardening).
pub fn enable_assign_primary(token: HANDLE) -> Result<()> {
    // SAFETY: a live token handle with `TOKEN_ADJUST_PRIVILEGES`.
    unsafe { crate::token::enable_single_privilege_pub(token, "SeAssignPrimaryTokenPrivilege") }
}
