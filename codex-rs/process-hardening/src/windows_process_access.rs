//! PF-27-S06: on Windows, keep other processes from reading this process's
//! memory or environment.
//!
//! A process's default DACL gives its user full access, so any process of the
//! same user (including a command under the restricted-token sandbox, whose
//! token restricts writes only) can open it with `PROCESS_VM_READ` and read
//! its memory and environment block. This replaces the DACL with one that
//! gives the user only `PROCESS_QUERY_LIMITED_INFORMATION` and `SYNCHRONIZE`.
//! An `OWNER RIGHTS` entry limited to `READ_CONTROL` removes the owner's
//! implicit `WRITE_DAC`, so a same-user process cannot grant itself access
//! back. Handles this process already holds to itself, and the handles it
//! receives when it creates a child, are unaffected.

use std::io;
use std::ptr;
use windows_sys::Win32::Foundation::CloseHandle;
use windows_sys::Win32::Foundation::ERROR_SUCCESS;
use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::Foundation::HLOCAL;
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
use windows_sys::Win32::System::Threading::GetCurrentProcess;
use windows_sys::Win32::System::Threading::OpenProcessToken;
use windows_sys::Win32::System::Threading::PROCESS_QUERY_LIMITED_INFORMATION;

/// `SYNCHRONIZE`: lets the user wait for the process to exit.
const SYNCHRONIZE: u32 = 0x0010_0000;

/// Replaces the current process's DACL so that no other process (of this or
/// another user, short of `SYSTEM` or a debug-privileged administrator) can
/// read its memory, read its environment, duplicate its handles or change
/// its DACL.
pub fn restrict_current_process_access() -> io::Result<()> {
    let user_sid = current_user_sid_string()?;
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
    let user_mask = PROCESS_QUERY_LIMITED_INFORMATION | SYNCHRONIZE;
    // GA for SYSTEM; READ_CONTROL only for OWNER RIGHTS (S-1-3-4).
    format!("D:P(A;;0x{user_mask:x};;;{user_sid})(A;;GA;;;SY)(A;;RC;;;OW)")
}

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
