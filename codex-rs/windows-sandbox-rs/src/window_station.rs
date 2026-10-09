//! Window station and desktop access for the elevated sandbox's command
//! runner, and the commands it starts, outside the interactive window
//! station (#341, #345).
//!
//! A process that loads `user32.dll` attaches to a window station and desktop
//! when it starts; if it cannot open them, it dies with `0xC0000142`
//! (`STATUS_DLL_INIT_FAILED`) before running any code. `CreateProcessWithLogonW`
//! gives the new logon access to the interactive window station (`WinSta0`).
//! A Windows OpenSSH session (or a service) runs on a non-interactive window
//! station (`Service-0x0-<logon id>$`) that grants only the session's own user,
//! so there Core does it itself: it starts the runner suspended, gives the
//! runner's logon SID (not the sandbox user, which every sandbox logon on the
//! machine has) the least access a runner needs on this window station and
//! desktop, and resumes it. Once the runner has started its command, the
//! entries shrink to what the command and its children need to start: no
//! `WINSTA_CREATEDESKTOP`, and nothing on Core's desktop when they run on a
//! private desktop. They go when the runner exits.
//!
//! The commands hold the runner's logon SID, so what is left is theirs too;
//! nothing less lets them start. The runners Core starts from one logon
//! session share their logon SID, so while one runner starts, the commands
//! of the others hold its entries for that moment. No hooks, windows, menus
//! or clipboard either way.

use crate::token::get_current_token_for_restriction;
use crate::token::get_logon_sid_bytes;
use crate::winutil::to_wide;
use anyhow::Context;
use anyhow::Result;
use std::ffi::c_void;
use std::ptr;
use std::sync::Mutex;
use windows_sys::Win32::Foundation::CloseHandle;
use windows_sys::Win32::Foundation::ERROR_INSUFFICIENT_BUFFER;
use windows_sys::Win32::Foundation::ERROR_SUCCESS;
use windows_sys::Win32::Foundation::GetLastError;
use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::Foundation::HLOCAL;
use windows_sys::Win32::Foundation::LocalFree;
use windows_sys::Win32::Foundation::WAIT_ABANDONED;
use windows_sys::Win32::Foundation::WAIT_OBJECT_0;
use windows_sys::Win32::Security::ACE_HEADER;
use windows_sys::Win32::Security::ACL;
use windows_sys::Win32::Security::ACL_REVISION;
use windows_sys::Win32::Security::ACL_SIZE_INFORMATION;
use windows_sys::Win32::Security::AclSizeInformation;
use windows_sys::Win32::Security::AddAce;
use windows_sys::Win32::Security::Authorization::GetSecurityInfo;
use windows_sys::Win32::Security::Authorization::SE_WINDOW_OBJECT;
use windows_sys::Win32::Security::Authorization::SetSecurityInfo;
use windows_sys::Win32::Security::DACL_SECURITY_INFORMATION;
use windows_sys::Win32::Security::EqualSid;
use windows_sys::Win32::Security::GetAce;
use windows_sys::Win32::Security::GetAclInformation;
use windows_sys::Win32::Security::INHERIT_ONLY_ACE;
use windows_sys::Win32::Security::INHERITED_ACE;
use windows_sys::Win32::Security::InitializeAcl;
use windows_sys::Win32::Security::TOKEN_QUERY;
use windows_sys::Win32::Storage::FileSystem::READ_CONTROL;
use windows_sys::Win32::Storage::FileSystem::WRITE_DAC;
use windows_sys::Win32::System::Pipes::PeekNamedPipe;
use windows_sys::Win32::System::StationsAndDesktops::CloseDesktop;
use windows_sys::Win32::System::StationsAndDesktops::CloseWindowStation;
use windows_sys::Win32::System::StationsAndDesktops::GetProcessWindowStation;
use windows_sys::Win32::System::StationsAndDesktops::GetThreadDesktop;
use windows_sys::Win32::System::StationsAndDesktops::GetUserObjectInformationW;
use windows_sys::Win32::System::StationsAndDesktops::OpenDesktopW;
use windows_sys::Win32::System::StationsAndDesktops::OpenWindowStationW;
use windows_sys::Win32::System::StationsAndDesktops::UOI_NAME;
use windows_sys::Win32::System::Threading::CreateMutexW;
use windows_sys::Win32::System::Threading::GetCurrentThreadId;
use windows_sys::Win32::System::Threading::OpenProcessToken;
use windows_sys::Win32::System::Threading::ReleaseMutex;
use windows_sys::Win32::System::Threading::WaitForSingleObject;

/// The interactive window station, where the secondary logon service grants
/// the new user access itself.
pub(crate) const INTERACTIVE_WINDOW_STATION: &str = "WinSta0";

const WINSTA_READATTRIBUTES: u32 = 0x0002;
const WINSTA_CREATEDESKTOP: u32 = 0x0008;
const WINSTA_ACCESSGLOBALATOMS: u32 = 0x0020;
const WINSTA_EXITWINDOWS: u32 = 0x0040;
const DESKTOP_READOBJECTS: u32 = 0x0001;
const DESKTOP_WRITEOBJECTS: u32 = 0x0080;
const ACCESS_ALLOWED_ACE_TYPE: u8 = 0;
const ACCESS_DENIED_ACE_TYPE: u8 = 1;
const GENERIC_ALL: u32 = 0x1000_0000;

/// The least a runner needs on the window station, measured on Windows 11
/// 26200 over SSH: without `READ_CONTROL`, `WINSTA_ACCESSGLOBALATOMS` or
/// `WINSTA_EXITWINDOWS` the runner dies with `0xC0000142`, and without
/// `WINSTA_READATTRIBUTES` its commands do; `WINSTA_CREATEDESKTOP` is for its
/// private desktop. Not the clipboard.
pub(crate) const RUNNER_STATION_ACCESS: u32 = READ_CONTROL
    | WINSTA_READATTRIBUTES
    | WINSTA_ACCESSGLOBALATOMS
    | WINSTA_EXITWINDOWS
    | WINSTA_CREATEDESKTOP;
/// What is left on the window station once the runner has started its
/// command: what the command and its children need to start (measured
/// likewise).
pub(crate) const COMMAND_STATION_ACCESS: u32 =
    READ_CONTROL | WINSTA_READATTRIBUTES | WINSTA_ACCESSGLOBALATOMS | WINSTA_EXITWINDOWS;
/// The least a runner needs on Core's desktop, and its commands when they
/// start there (measured likewise): read and write objects. No hooks,
/// windows or menus.
pub(crate) const DESKTOP_ACCESS: u32 = DESKTOP_READOBJECTS | DESKTOP_WRITEOBJECTS;
/// Serializes window-object DACL edits across Core processes in a session.
const DACL_MUTEX_NAME: &str = "Local\\CodexSandboxWindowAccess";
const DACL_MUTEX_TIMEOUT_MS: u32 = 10_000;

/// The name of this process's window station (`WinSta0` in an interactive
/// session, `Service-0x0-<logon id>$` in an SSH session or a service).
pub(crate) fn current_window_station_name() -> Option<String> {
    // SAFETY: returns a pseudo-handle owned by the process; not closed.
    object_name(unsafe { GetProcessWindowStation() })
}

/// The name of this thread's desktop.
pub(crate) fn current_desktop_name() -> Option<String> {
    // SAFETY: returns a handle owned by the thread; not closed.
    object_name(unsafe { GetThreadDesktop(GetCurrentThreadId()) })
}

fn object_name(handle: isize) -> Option<String> {
    if handle == 0 {
        return None;
    }
    let mut buffer = vec![0u16; 256];
    loop {
        let mut needed = 0u32;
        // SAFETY: `buffer` is writable for its byte length.
        let ok = unsafe {
            GetUserObjectInformationW(
                handle,
                UOI_NAME,
                buffer.as_mut_ptr().cast(),
                (buffer.len() * 2) as u32,
                &mut needed,
            )
        };
        if ok != 0 {
            break;
        }
        // SAFETY: read right after the failed call.
        let err = unsafe { GetLastError() };
        let needed = (needed as usize).div_ceil(2);
        if err != ERROR_INSUFFICIENT_BUFFER || needed <= buffer.len() {
            return None;
        }
        buffer.resize(needed, 0);
    }
    let len = buffer.iter().position(|&c| c == 0).unwrap_or(buffer.len());
    Some(String::from_utf16_lossy(&buffer[..len]))
}

pub(crate) fn is_interactive_window_station(name: &str) -> bool {
    name.eq_ignore_ascii_case(INTERACTIVE_WINDOW_STATION)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WindowObject {
    Station,
    Desktop,
}

/// Allow entries for one logon SID on this process's non-interactive window
/// station and desktop, removed on drop (#345).
///
/// Each value adds entries of its own, even when an equal entry is there
/// already, and removes exactly those: the runners Core starts from one
/// logon session share their logon SID (the secondary logon service reuses
/// the logon), so the entries count the runners that need them, across
/// Core processes. Edits are serialized by [`with_dacl_lock`].
#[derive(Debug)]
pub struct WindowAccess {
    station: String,
    desktop: String,
    sid: Vec<u8>,
    /// The masks of the entries this value added and has not removed.
    station_entry: Option<u32>,
    desktop_entry: Option<u32>,
}

impl WindowAccess {
    /// Before a command runner whose token has the logon SID `sid` starts on
    /// this process's window station: gives that logon what a runner needs
    /// on the window station and desktop. `None` on the interactive window
    /// station, where the secondary logon service does it.
    pub(crate) fn grant_runner(sid: &[u8]) -> Result<Option<Self>> {
        let Some(mut access) = Self::off_interactive_station(sid)? else {
            return Ok(None);
        };
        access.add(WindowObject::Station, RUNNER_STATION_ACCESS)?;
        access.add(WindowObject::Desktop, DESKTOP_ACCESS)?;
        Ok(Some(access))
    }

    /// Gives the logon SID `sid` [`DESKTOP_ACCESS`] on this process's
    /// desktop, off the interactive window station: what commands need to
    /// start on it (`sandbox_private_desktop = false`).
    pub(crate) fn grant_desktop(sid: &[u8]) -> Result<Option<Self>> {
        let Some(mut access) = Self::off_interactive_station(sid)? else {
            return Ok(None);
        };
        access.add(WindowObject::Desktop, DESKTOP_ACCESS)?;
        Ok(Some(access))
    }

    fn off_interactive_station(sid: &[u8]) -> Result<Option<Self>> {
        let station = current_window_station_name().context("read the window station's name")?;
        if is_interactive_window_station(&station) {
            return Ok(None);
        }
        let desktop = current_desktop_name().context("read the desktop's name")?;
        Ok(Some(Self {
            station,
            desktop,
            sid: sid.to_vec(),
            station_entry: None,
            desktop_entry: None,
        }))
    }

    /// Once the runner has started its command: what the commands started
    /// on the runner's logon need, and nothing they don't. The window station
    /// entry loses `WINSTA_CREATEDESKTOP`, and the desktop entry goes unless
    /// the commands start on this desktop (`commands_use_this_desktop`, no
    /// private desktop).
    pub(crate) fn narrow_for_commands(&mut self, commands_use_this_desktop: bool) -> Result<()> {
        if let Some(mask) = self.station_entry
            && mask != COMMAND_STATION_ACCESS
        {
            self.replace(WindowObject::Station, Some(COMMAND_STATION_ACCESS))?;
        }
        if !commands_use_this_desktop && self.desktop_entry.is_some() {
            self.replace(WindowObject::Desktop, None)?;
        }
        Ok(())
    }

    fn entry(&mut self, object: WindowObject) -> &mut Option<u32> {
        match object {
            WindowObject::Station => &mut self.station_entry,
            WindowObject::Desktop => &mut self.desktop_entry,
        }
    }

    fn name(&self, object: WindowObject) -> String {
        match object {
            WindowObject::Station => self.station.clone(),
            WindowObject::Desktop => format!("{}\\{}", self.station, self.desktop),
        }
    }

    fn add(&mut self, object: WindowObject, mask: u32) -> Result<()> {
        debug_assert!(self.entry(object).is_none());
        self.edit(object, None, Some(mask))?;
        *self.entry(object) = Some(mask);
        Ok(())
    }

    /// Replaces this value's entry on `object` with one with `mask`, or
    /// removes it.
    fn replace(&mut self, object: WindowObject, mask: Option<u32>) -> Result<()> {
        let current = *self.entry(object);
        self.edit(object, current, mask)?;
        *self.entry(object) = mask;
        Ok(())
    }

    fn edit(&self, object: WindowObject, remove: Option<u32>, add: Option<u32>) -> Result<()> {
        let name = match object {
            WindowObject::Station => &self.station,
            WindowObject::Desktop => &self.desktop,
        };
        with_dacl_lock(|| edit_window_object(object, name, &self.sid, remove, add))
            .with_context(|| format!("{:?} {}", object, self.name(object)))
    }
}

impl Drop for WindowAccess {
    fn drop(&mut self) {
        for object in [WindowObject::Station, WindowObject::Desktop] {
            if self.entry(object).is_some() {
                // Nothing to do on failure: the entry stays until the window
                // station goes away.
                let _ = self.replace(object, None);
            }
        }
    }
}

/// In the runner, off the interactive window station: waits until the
/// console host of a ConPTY it just created has started, or `timeout`
/// passes (returns false). The host starts on the runner's desktop, Core's,
/// whose entry Core removes once the runner reports its command started
/// ([`WindowAccess::narrow_for_commands`]); a host still starting then
/// would die. It writes its first output (terminal mode requests) once it
/// runs; this peeks at `output_read` for it and leaves it there.
pub fn wait_for_console_host_start(output_read: HANDLE, timeout: std::time::Duration) -> bool {
    if current_window_station_name().is_some_and(|name| is_interactive_window_station(&name)) {
        return true;
    }
    let deadline = std::time::Instant::now() + timeout;
    loop {
        let mut available = 0u32;
        // SAFETY: peeks without reading; `output_read` is a live pipe end.
        let ok = unsafe {
            PeekNamedPipe(
                output_read,
                ptr::null_mut(),
                0,
                ptr::null_mut(),
                &mut available,
                ptr::null_mut(),
            )
        };
        if ok == 0 || available > 0 {
            // A broken pipe means the host is gone; nothing to wait for.
            return ok != 0;
        }
        if std::time::Instant::now() >= deadline {
            return false;
        }
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
}

/// The logon SID of `process`'s token (a process Core started).
pub(crate) fn process_logon_sid(process: HANDLE) -> Result<Vec<u8>> {
    let mut token: HANDLE = 0;
    // SAFETY: `process` is a live process handle; the token is closed below.
    if unsafe { OpenProcessToken(process, TOKEN_QUERY, &mut token) } == 0 {
        return Err(std::io::Error::last_os_error()).context("open the process's token");
    }
    // SAFETY: `token` was opened above with TOKEN_QUERY.
    let sid = unsafe { get_logon_sid_bytes(token) };
    // SAFETY: opened above.
    unsafe { CloseHandle(token) };
    sid
}

/// The logon SID of this process's token.
pub(crate) fn current_logon_sid() -> Result<Vec<u8>> {
    // SAFETY: the token is closed right after reading its logon SID.
    unsafe {
        let token = get_current_token_for_restriction()?;
        let sid = get_logon_sid_bytes(token);
        CloseHandle(token);
        sid
    }
}

/// Serializes DACL edits on window objects, in this process and, through a
/// named mutex in this session's namespace, across Core processes. Without
/// it, two read-modify-write edits can lose one. If the named mutex can't be
/// had (another user created it), only this process's edits are serialized.
fn with_dacl_lock<T>(edit: impl FnOnce() -> T) -> T {
    static LOCK: Mutex<()> = Mutex::new(());
    let _guard = LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let name = to_wide(DACL_MUTEX_NAME);
    // SAFETY: opens or creates a named mutex; closed below.
    let mutex = unsafe { CreateMutexW(ptr::null(), 0, name.as_ptr()) };
    // SAFETY: waits on the mutex opened above (abandoned counts as owned).
    let owned = mutex != 0
        && matches!(
            unsafe { WaitForSingleObject(mutex, DACL_MUTEX_TIMEOUT_MS) },
            WAIT_OBJECT_0 | WAIT_ABANDONED
        );
    let result = edit();
    // SAFETY: released only if owned; the handle is closed once.
    unsafe {
        if owned {
            ReleaseMutex(mutex);
        }
        if mutex != 0 {
            CloseHandle(mutex);
        }
    }
    result
}

fn edit_window_object(
    object: WindowObject,
    name: &str,
    sid: &[u8],
    remove: Option<u32>,
    add: Option<u32>,
) -> Result<()> {
    let wide = to_wide(name);
    let handle = match object {
        // SAFETY: opens a named window station; closed below.
        WindowObject::Station => unsafe {
            OpenWindowStationW(wide.as_ptr(), 0, READ_CONTROL | WRITE_DAC)
        },
        // SAFETY: opens a desktop of this process's window station; closed below.
        WindowObject::Desktop => unsafe {
            OpenDesktopW(wide.as_ptr(), 0, 0, READ_CONTROL | WRITE_DAC)
        },
    };
    if handle == 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    let result = rewrite_dacl(handle, sid, remove, add);
    // SAFETY: opened above.
    unsafe {
        match object {
            WindowObject::Station => CloseWindowStation(handle),
            WindowObject::Desktop => CloseDesktop(handle),
        }
    };
    result
}

/// True when `handle`'s DACL has an allow entry for `sid` with all of
/// `access`, and no deny entry for `sid` that takes any of it away.
pub(crate) fn object_grants(handle: isize, sid: &[u8], access: u32) -> Result<bool> {
    Ok(object_access(handle, sid, access)? == AccessState::Granted)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AccessState {
    Granted,
    NotGranted,
    /// A deny entry for the SID takes some of the access away.
    Denied,
}

fn object_access(handle: isize, sid: &[u8], access: u32) -> Result<AccessState> {
    let (dacl, descriptor) = object_dacl(handle)?;
    // SAFETY: `dacl` belongs to `descriptor`, freed below.
    let state = unsafe { dacl_access(dacl, sid, access) };
    // SAFETY: allocated by GetSecurityInfo.
    unsafe { LocalFree(descriptor as HLOCAL) };
    Ok(state)
}

/// Window-object entries are stored with specific rights; `GENERIC_ALL`
/// counts as everything.
unsafe fn dacl_access(dacl: *mut ACL, sid: &[u8], access: u32) -> AccessState {
    if dacl.is_null() {
        return AccessState::NotGranted;
    }
    let mut info: ACL_SIZE_INFORMATION = unsafe { std::mem::zeroed() };
    let ok = unsafe {
        GetAclInformation(
            dacl,
            ptr::from_mut(&mut info).cast(),
            std::mem::size_of::<ACL_SIZE_INFORMATION>() as u32,
            AclSizeInformation,
        )
    };
    if ok == 0 {
        return AccessState::NotGranted;
    }
    let sid = sid.as_ptr() as *mut c_void;
    let mut allowed = 0u32;
    for index in 0..info.AceCount {
        let mut ace: *mut c_void = ptr::null_mut();
        if unsafe { GetAce(dacl, index, &mut ace) } == 0 {
            continue;
        }
        let header = unsafe { &*(ace as *const ACE_HEADER) };
        let allow = match header.AceType {
            ACCESS_ALLOWED_ACE_TYPE => true,
            ACCESS_DENIED_ACE_TYPE => false,
            _ => continue,
        };
        if header.AceFlags as u32 & INHERIT_ONLY_ACE != 0 {
            continue;
        }
        // Both entry types: header, mask, then the SID.
        let mask = unsafe { *((ace as usize + std::mem::size_of::<ACE_HEADER>()) as *const u32) };
        let ace_sid = (ace as usize + std::mem::size_of::<ACE_HEADER>() + 4) as *mut c_void;
        if unsafe { EqualSid(ace_sid, sid) } == 0 {
            continue;
        }
        let mask = if mask & GENERIC_ALL != 0 {
            u32::MAX
        } else {
            mask
        };
        if !allow && mask & access != 0 {
            return AccessState::Denied;
        }
        if allow {
            allowed |= mask;
        }
    }
    if allowed & access == access {
        AccessState::Granted
    } else {
        AccessState::NotGranted
    }
}

fn object_dacl(handle: isize) -> Result<(*mut ACL, *mut c_void)> {
    let mut dacl: *mut ACL = ptr::null_mut();
    let mut descriptor: *mut c_void = ptr::null_mut();
    // SAFETY: reads the DACL into a descriptor the caller frees.
    let status = unsafe {
        GetSecurityInfo(
            handle,
            SE_WINDOW_OBJECT,
            DACL_SECURITY_INFORMATION,
            ptr::null_mut(),
            ptr::null_mut(),
            &mut dacl,
            ptr::null_mut(),
            &mut descriptor,
        )
    };
    if status != ERROR_SUCCESS {
        anyhow::bail!("GetSecurityInfo failed: {status}");
    }
    Ok((dacl, descriptor))
}

/// Rewrites `handle`'s DACL: the same entries in the same order, less the
/// first allow entry for `sid` with exactly the mask `remove` (if given and
/// present), plus an allow entry for `sid` with the mask `add` (if given),
/// placed before any inherited entries. Fails if a deny entry for `sid`
/// takes away any of `add`, or if the access added is not kept.
fn rewrite_dacl(handle: isize, sid: &[u8], remove: Option<u32>, add: Option<u32>) -> Result<()> {
    let (dacl, descriptor) = object_dacl(handle)?;
    // SAFETY: `dacl` belongs to `descriptor`, freed right after.
    let rebuilt = unsafe { rebuild_dacl(dacl, sid, remove, add) };
    // SAFETY: allocated by GetSecurityInfo; `dacl` is not used after this.
    unsafe { LocalFree(descriptor as HLOCAL) };
    let Some(mut rebuilt) = rebuilt? else {
        return Ok(());
    };
    // SAFETY: `rebuilt` holds a valid ACL built by `rebuild_dacl`.
    let status = unsafe {
        SetSecurityInfo(
            handle,
            SE_WINDOW_OBJECT,
            DACL_SECURITY_INFORMATION,
            ptr::null_mut(),
            ptr::null_mut(),
            rebuilt.as_mut_ptr().cast(),
            ptr::null(),
        )
    };
    if status != ERROR_SUCCESS {
        anyhow::bail!("SetSecurityInfo failed: {status}");
    }
    if let Some(add) = add
        && !object_grants(handle, sid, add)?
    {
        anyhow::bail!("the access granted was not kept");
    }
    Ok(())
}

/// The ACL [`rewrite_dacl`] writes, as `u32`s for alignment, or `None` when
/// there is nothing to change.
unsafe fn rebuild_dacl(
    dacl: *mut ACL,
    sid: &[u8],
    remove: Option<u32>,
    add: Option<u32>,
) -> Result<Option<Vec<u32>>> {
    if dacl.is_null() {
        // A NULL DACL grants everyone everything; nothing to add or remove.
        return Ok(None);
    }
    if let Some(add) = add
        && unsafe { dacl_access(dacl, sid, add) } == AccessState::Denied
    {
        anyhow::bail!("a deny entry for the logon is in the way");
    }
    let mut info: ACL_SIZE_INFORMATION = unsafe { std::mem::zeroed() };
    if unsafe {
        GetAclInformation(
            dacl,
            ptr::from_mut(&mut info).cast(),
            std::mem::size_of::<ACL_SIZE_INFORMATION>() as u32,
            AclSizeInformation,
        )
    } == 0
    {
        return Err(std::io::Error::last_os_error()).context("GetAclInformation");
    }
    let new_entry = add.map(|mask| allow_entry(sid, mask));
    let size = info.AclBytesInUse as usize + new_entry.as_ref().map_or(0, Vec::len);
    let size = size.next_multiple_of(4);
    let mut rebuilt = vec![0u32; size / 4];
    let revision = u32::from(unsafe { (*dacl).AclRevision }).max(ACL_REVISION);
    let acl: *mut ACL = rebuilt.as_mut_ptr().cast();
    if unsafe { InitializeAcl(acl, size as u32, revision) } == 0 {
        return Err(std::io::Error::last_os_error()).context("InitializeAcl");
    }
    let mut to_remove = remove;
    let mut to_add = new_entry;
    let append = |entry: *const c_void, len: u16| -> Result<()> {
        // SAFETY: `entry` is a whole entry of `len` bytes; `acl` has room.
        if unsafe { AddAce(acl, revision, u32::MAX, entry, u32::from(len)) } == 0 {
            return Err(std::io::Error::last_os_error()).context("AddAce");
        }
        Ok(())
    };
    let sid_ptr = sid.as_ptr() as *mut c_void;
    for index in 0..info.AceCount {
        let mut ace: *mut c_void = ptr::null_mut();
        if unsafe { GetAce(dacl, index, &mut ace) } == 0 {
            return Err(std::io::Error::last_os_error()).context("GetAce");
        }
        let header = unsafe { &*(ace as *const ACE_HEADER) };
        if let Some(mask) = to_remove
            && header.AceType == ACCESS_ALLOWED_ACE_TYPE
            && header.AceFlags == 0
            && unsafe { *((ace as usize + std::mem::size_of::<ACE_HEADER>()) as *const u32) }
                == mask
            && unsafe {
                EqualSid(
                    (ace as usize + std::mem::size_of::<ACE_HEADER>() + 4) as *mut c_void,
                    sid_ptr,
                )
            } != 0
        {
            to_remove = None;
            continue;
        }
        if u32::from(header.AceFlags) & INHERITED_ACE != 0
            && let Some(entry) = to_add.take()
        {
            append(entry.as_ptr().cast(), entry.len() as u16)?;
        }
        append(ace, header.AceSize)?;
    }
    let removed = remove.is_some() && to_remove.is_none();
    if let Some(entry) = to_add.take() {
        append(entry.as_ptr().cast(), entry.len() as u16)?;
    } else if !removed {
        // Neither added nor removed: the entry to remove was already gone.
        return Ok(None);
    }
    Ok(Some(rebuilt))
}

/// An `ACCESS_ALLOWED_ACE` for `sid` with `mask`, no flags.
fn allow_entry(sid: &[u8], mask: u32) -> Vec<u8> {
    let len = std::mem::size_of::<ACE_HEADER>() + 4 + sid.len();
    let len = len.next_multiple_of(4);
    let mut entry = Vec::with_capacity(len);
    entry.push(ACCESS_ALLOWED_ACE_TYPE);
    entry.push(0);
    entry.extend_from_slice(&(len as u16).to_le_bytes());
    entry.extend_from_slice(&mask.to_le_bytes());
    entry.extend_from_slice(sid);
    entry.resize(len, 0);
    entry
}

#[cfg(test)]
#[path = "window_station_tests.rs"]
mod tests;
