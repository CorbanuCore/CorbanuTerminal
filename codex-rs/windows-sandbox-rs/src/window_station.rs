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
//! every command timed out. Core grants the sandbox's user access to such a
//! window station and its desktop before starting the runner, and the runner
//! starts its commands on desktops in the window station it is on, not on
//! `WinSta0`.

use crate::winutil::to_wide;
use anyhow::Context;
use anyhow::Result;
use std::ffi::c_void;
use std::ptr;
use windows_sys::Win32::Foundation::ERROR_SUCCESS;
use windows_sys::Win32::Foundation::HLOCAL;
use windows_sys::Win32::Foundation::LocalFree;
use windows_sys::Win32::Security::ACL;
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

/// The window station rights a Windows OpenSSH session grants its own user
/// (`WINSTA_READATTRIBUTES`, `WINSTA_ACCESSCLIPBOARD`, `WINSTA_CREATEDESKTOP`,
/// `WINSTA_ACCESSGLOBALATOMS`, `WINSTA_EXITWINDOWS`), and `READ_CONTROL`,
/// without which `user32` still fails to start (measured on Windows 11 26200).
/// No `WRITE_DAC`, `WRITE_OWNER` or `DELETE`.
pub(crate) const WINDOW_STATION_ACCESS: u32 = 0x006E | READ_CONTROL;
/// The desktop rights an SSH session grants its own user
/// (`DESKTOP_READOBJECTS`, `DESKTOP_CREATEWINDOW`, `DESKTOP_CREATEMENU`,
/// `DESKTOP_HOOKCONTROL`, `DESKTOP_ENUMERATE`, `DESKTOP_WRITEOBJECTS`), and
/// `READ_CONTROL`.
pub(crate) const DESKTOP_ACCESS: u32 = 0x00CF | READ_CONTROL;

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
    let mut buffer = [0u16; 256];
    let mut needed = 0u32;
    // SAFETY: `buffer` is writable for its byte length.
    let ok = unsafe {
        GetUserObjectInformationW(
            handle,
            UOI_NAME,
            buffer.as_mut_ptr().cast(),
            std::mem::size_of_val(&buffer) as u32,
            &mut needed,
        )
    };
    if ok == 0 {
        return None;
    }
    let len = buffer.iter().position(|&c| c == 0).unwrap_or(buffer.len());
    Some(String::from_utf16_lossy(&buffer[..len]))
}

pub(crate) fn is_interactive_window_station(name: &str) -> bool {
    name.eq_ignore_ascii_case(INTERACTIVE_WINDOW_STATION)
}

/// The window station that desktops for this process's children are named
/// in: its own (the interactive one when it cannot be read).
pub(crate) fn launch_window_station() -> String {
    current_window_station_name().unwrap_or_else(|| INTERACTIVE_WINDOW_STATION.to_string())
}

/// What [`grant_window_access`] changed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WindowAccessGrant {
    /// On the interactive window station: left to the secondary logon service.
    Interactive,
    /// Both objects already granted `sid` the access.
    AlreadyGranted,
    /// Added the access for `sid` to the window station and/or desktop.
    Granted,
}

/// Before a process is started as `sid` with `CreateProcessWithLogonW` from
/// this process, grants `sid` the access it needs to this process's window
/// station and desktop, unless they are the interactive ones.
pub(crate) fn grant_window_access(sid: &[u8]) -> Result<WindowAccessGrant> {
    let station = current_window_station_name().context("read the window station's name")?;
    if is_interactive_window_station(&station) {
        return Ok(WindowAccessGrant::Interactive);
    }
    let desktop = current_desktop_name().context("read the desktop's name")?;
    let station_wide = to_wide(&station);
    // SAFETY: opens a named window station; closed below.
    let station_handle =
        unsafe { OpenWindowStationW(station_wide.as_ptr(), 0, READ_CONTROL | WRITE_DAC) };
    if station_handle == 0 {
        return Err(std::io::Error::last_os_error())
            .with_context(|| format!("open window station {station}"));
    }
    let station_result = grant_object_access(station_handle, sid, WINDOW_STATION_ACCESS);
    // SAFETY: opened above.
    unsafe { CloseWindowStation(station_handle) };
    let desktop_wide = to_wide(&desktop);
    // SAFETY: opens a desktop of this process's window station; closed below.
    let desktop_handle =
        unsafe { OpenDesktopW(desktop_wide.as_ptr(), 0, 0, READ_CONTROL | WRITE_DAC) };
    if desktop_handle == 0 {
        return Err(std::io::Error::last_os_error())
            .with_context(|| format!("open desktop {station}\\{desktop}"));
    }
    let desktop_result = grant_object_access(desktop_handle, sid, DESKTOP_ACCESS);
    // SAFETY: opened above.
    unsafe { CloseDesktop(desktop_handle) };
    let changed = station_result.with_context(|| format!("window station {station}"))?
        | desktop_result.with_context(|| format!("desktop {station}\\{desktop}"))?;
    Ok(if changed {
        WindowAccessGrant::Granted
    } else {
        WindowAccessGrant::AlreadyGranted
    })
}

/// True when `handle`'s DACL has an allow entry for `sid` with all of
/// `access`.
pub(crate) fn object_grants(handle: isize, sid: &[u8], access: u32) -> Result<bool> {
    let (dacl, descriptor) = object_dacl(handle)?;
    let mut sid = sid.to_vec();
    // SAFETY: `dacl` belongs to `descriptor`, freed below.
    let granted =
        unsafe { crate::acl::dacl_mask_allows(dacl, &[sid.as_mut_ptr().cast()], access, true) };
    // SAFETY: allocated by GetSecurityInfo.
    unsafe { LocalFree(descriptor as HLOCAL) };
    Ok(granted)
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
/// Returns whether it changed the DACL.
fn grant_object_access(handle: isize, sid: &[u8], access: u32) -> Result<bool> {
    if object_grants(handle, sid, access)? {
        return Ok(false);
    }
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
    Ok(true)
}

#[cfg(test)]
#[path = "window_station_tests.rs"]
mod tests;
