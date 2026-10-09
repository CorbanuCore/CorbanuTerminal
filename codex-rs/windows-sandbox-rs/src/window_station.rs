//! Window station and desktop access for processes started as the sandbox's
//! users (#341).
//!
//! A process that loads `user32.dll` attaches to a window station and desktop
//! when it starts; if it cannot open them, it dies with `0xC0000142`
//! (`STATUS_DLL_INIT_FAILED`) before running any code. `CreateProcessWithLogonW`
//! gives the new user access to the interactive window station (`WinSta0`).
//! A Windows OpenSSH session (or a service) runs on a non-interactive window
//! station (`Service-0x0-<logon id>$`) that grants only the session's own user,
//! so the command runner started there never connected to Core's pipes and
//! every command timed out. Core grants the sandbox's user the least access
//! the runner needs to start on such a window station and its desktop, and the
//! runner starts its commands on desktops in the window station it is on, not
//! on `WinSta0`.
//!
//! The entries are for the sandbox user's SID, so the commands the runner
//! starts hold them too, and they stay until the window station goes away
//! (the SSH session ends). They exclude hooks, windows, menus and the
//! clipboard; see #345 for scoping them to the runner's logon.

use crate::winutil::resolve_sid;
use crate::winutil::to_wide;
use anyhow::Context;
use anyhow::Result;
use std::ffi::c_void;
use std::ptr;
use windows_sys::Win32::Foundation::ERROR_INSUFFICIENT_BUFFER;
use windows_sys::Win32::Foundation::ERROR_SUCCESS;
use windows_sys::Win32::Foundation::GetLastError;
use windows_sys::Win32::Foundation::HLOCAL;
use windows_sys::Win32::Foundation::LocalFree;
use windows_sys::Win32::Security::ACE_HEADER;
use windows_sys::Win32::Security::ACL;
use windows_sys::Win32::Security::ACL_SIZE_INFORMATION;
use windows_sys::Win32::Security::AclSizeInformation;
use windows_sys::Win32::Security::Authorization::EXPLICIT_ACCESS_W;
use windows_sys::Win32::Security::Authorization::GRANT_ACCESS;
use windows_sys::Win32::Security::Authorization::GetSecurityInfo;
use windows_sys::Win32::Security::Authorization::SE_WINDOW_OBJECT;
use windows_sys::Win32::Security::Authorization::SetEntriesInAclW;
use windows_sys::Win32::Security::Authorization::SetSecurityInfo;
use windows_sys::Win32::Security::Authorization::TRUSTEE_IS_SID;
use windows_sys::Win32::Security::Authorization::TRUSTEE_IS_UNKNOWN;
use windows_sys::Win32::Security::Authorization::TRUSTEE_W;
use windows_sys::Win32::Security::DACL_SECURITY_INFORMATION;
use windows_sys::Win32::Security::EqualSid;
use windows_sys::Win32::Security::GetAce;
use windows_sys::Win32::Security::GetAclInformation;
use windows_sys::Win32::Security::INHERIT_ONLY_ACE;
use windows_sys::Win32::Storage::FileSystem::READ_CONTROL;
use windows_sys::Win32::Storage::FileSystem::WRITE_DAC;
use windows_sys::Win32::System::StationsAndDesktops::CloseDesktop;
use windows_sys::Win32::System::StationsAndDesktops::CloseWindowStation;
use windows_sys::Win32::System::StationsAndDesktops::GetProcessWindowStation;
use windows_sys::Win32::System::StationsAndDesktops::GetThreadDesktop;
use windows_sys::Win32::System::StationsAndDesktops::GetUserObjectInformationW;
use windows_sys::Win32::System::StationsAndDesktops::OpenDesktopW;
use windows_sys::Win32::System::StationsAndDesktops::OpenWindowStationW;
use windows_sys::Win32::System::StationsAndDesktops::UOI_NAME;
use windows_sys::Win32::System::Threading::GetCurrentThreadId;

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

/// The least the runner and its commands need on the window station,
/// measured on Windows 11 26200 over SSH: without `READ_CONTROL`,
/// `WINSTA_ACCESSGLOBALATOMS` or `WINSTA_EXITWINDOWS` the runner dies with
/// `0xC0000142`, and without `WINSTA_READATTRIBUTES` its commands do;
/// `WINSTA_CREATEDESKTOP` is for its private desktop. Not the clipboard.
pub(crate) const WINDOW_STATION_ACCESS: u32 = READ_CONTROL
    | WINSTA_READATTRIBUTES
    | WINSTA_ACCESSGLOBALATOMS
    | WINSTA_EXITWINDOWS
    | WINSTA_CREATEDESKTOP;
/// The least they need on the desktop (measured likewise): read and write
/// objects. No hooks, windows or menus.
pub(crate) const DESKTOP_ACCESS: u32 = DESKTOP_READOBJECTS | DESKTOP_WRITEOBJECTS;

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

/// What [`grant_window_access`] changed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WindowAccessGrant {
    /// On the interactive window station: left to the secondary logon service.
    Interactive,
    /// Both objects already granted the user the access.
    AlreadyGranted,
    /// Added the access for the user to the window station and/or desktop.
    Granted,
}

/// Before a process is started as `username` with `CreateProcessWithLogonW`
/// from this process, grants that user the access it needs to this process's
/// window station and desktop, unless they are the interactive ones.
pub(crate) fn grant_window_access(username: &str) -> Result<WindowAccessGrant> {
    let station = current_window_station_name().context("read the window station's name")?;
    if is_interactive_window_station(&station) {
        return Ok(WindowAccessGrant::Interactive);
    }
    let desktop = current_desktop_name().context("read the desktop's name")?;
    let sid = resolve_sid(username)?;
    let station_wide = to_wide(&station);
    // SAFETY: opens a named window station; closed below.
    let station_handle =
        unsafe { OpenWindowStationW(station_wide.as_ptr(), 0, READ_CONTROL | WRITE_DAC) };
    let station_result = if station_handle == 0 {
        Err(std::io::Error::last_os_error().into())
    } else {
        let result = grant_object_access(station_handle, &sid, WINDOW_STATION_ACCESS);
        // SAFETY: opened above.
        unsafe { CloseWindowStation(station_handle) };
        result
    };
    let desktop_wide = to_wide(&desktop);
    // SAFETY: opens a desktop of this process's window station; closed below.
    let desktop_handle =
        unsafe { OpenDesktopW(desktop_wide.as_ptr(), 0, 0, READ_CONTROL | WRITE_DAC) };
    let desktop_result = if desktop_handle == 0 {
        Err(std::io::Error::last_os_error().into())
    } else {
        let result = grant_object_access(desktop_handle, &sid, DESKTOP_ACCESS);
        // SAFETY: opened above.
        unsafe { CloseDesktop(desktop_handle) };
        result
    };
    // Both errors, if both failed. A grant that succeeded on one object stays.
    match (station_result, desktop_result) {
        (Ok(station_changed), Ok(desktop_changed)) => Ok(if station_changed || desktop_changed {
            WindowAccessGrant::Granted
        } else {
            WindowAccessGrant::AlreadyGranted
        }),
        (Err(err), Ok(_)) => Err(err).context(format!("window station {station}")),
        (Ok(_), Err(err)) => Err(err).context(format!("desktop {station}\\{desktop}")),
        (Err(station_err), Err(desktop_err)) => Err(anyhow::anyhow!(
            "window station {station}: {station_err:#}; desktop {station}\\{desktop}: {desktop_err:#}"
        )),
    }
}

/// True when `handle`'s DACL has an allow entry for `sid` with all of
/// `access`, and no deny entry for `sid` that takes any of it away.
pub(crate) fn object_grants(handle: isize, sid: &[u8], access: u32) -> Result<bool> {
    let (dacl, descriptor) = object_dacl(handle)?;
    // SAFETY: `dacl` belongs to `descriptor`, freed below.
    let granted = unsafe { dacl_grants(dacl, sid, access) };
    // SAFETY: allocated by GetSecurityInfo.
    unsafe { LocalFree(descriptor as HLOCAL) };
    Ok(granted)
}

/// Window-object entries are stored with specific rights; `GENERIC_ALL`
/// counts as everything.
unsafe fn dacl_grants(dacl: *mut ACL, sid: &[u8], access: u32) -> bool {
    if dacl.is_null() {
        return false;
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
        return false;
    }
    let sid = sid.as_ptr() as *mut c_void;
    let mut allowed = 0u32;
    for index in 0..info.AceCount {
        let mut ace: *mut c_void = ptr::null_mut();
        if unsafe { GetAce(dacl, index, &mut ace) } == 0 {
            continue;
        }
        let header = unsafe { &*(ace as *const ACE_HEADER) };
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
        match header.AceType {
            ACCESS_DENIED_ACE_TYPE if mask & access != 0 => return false,
            ACCESS_ALLOWED_ACE_TYPE => allowed |= mask,
            _ => {}
        }
    }
    allowed & access == access
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

/// Adds an allow entry for `sid` with `access` unless one is there already.
/// Returns whether it changed the DACL. Another process granting another
/// user at the same moment can write over this entry, so it checks once more
/// and adds it again if it is gone.
fn grant_object_access(handle: isize, sid: &[u8], access: u32) -> Result<bool> {
    let mut changed = false;
    for _ in 0..2 {
        if object_grants(handle, sid, access)? {
            return Ok(changed);
        }
        add_allow_entry(handle, sid, access)?;
        changed = true;
    }
    if object_grants(handle, sid, access)? {
        Ok(changed)
    } else {
        anyhow::bail!("the access granted was not kept")
    }
}

fn add_allow_entry(handle: isize, sid: &[u8], access: u32) -> Result<()> {
    let (dacl, descriptor) = object_dacl(handle)?;
    let mut sid = sid.to_vec();
    let entry = EXPLICIT_ACCESS_W {
        grfAccessPermissions: access,
        grfAccessMode: GRANT_ACCESS,
        grfInheritance: 0,
        Trustee: TRUSTEE_W {
            pMultipleTrustee: ptr::null_mut(),
            MultipleTrusteeOperation: 0,
            TrusteeForm: TRUSTEE_IS_SID,
            TrusteeType: TRUSTEE_IS_UNKNOWN,
            ptstrName: sid.as_mut_ptr().cast(),
        },
    };
    let mut updated: *mut ACL = ptr::null_mut();
    // SAFETY: merges one entry into the current DACL; `updated` is freed below.
    let status = unsafe { SetEntriesInAclW(1, &entry, dacl, &mut updated) };
    // SAFETY: allocated by GetSecurityInfo; `dacl` is not used after this.
    unsafe { LocalFree(descriptor as HLOCAL) };
    if status != ERROR_SUCCESS {
        anyhow::bail!("SetEntriesInAclW failed: {status}");
    }
    // SAFETY: `updated` is a valid ACL from SetEntriesInAclW.
    let status = unsafe {
        SetSecurityInfo(
            handle,
            SE_WINDOW_OBJECT,
            DACL_SECURITY_INFORMATION,
            ptr::null_mut(),
            ptr::null_mut(),
            updated,
            ptr::null(),
        )
    };
    // SAFETY: allocated by SetEntriesInAclW.
    unsafe { LocalFree(updated as HLOCAL) };
    if status != ERROR_SUCCESS {
        anyhow::bail!("SetSecurityInfo failed: {status}");
    }
    Ok(())
}

#[cfg(test)]
#[path = "window_station_tests.rs"]
mod tests;
