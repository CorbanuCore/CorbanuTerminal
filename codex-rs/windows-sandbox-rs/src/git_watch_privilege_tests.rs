//! Exercise the direct retag without the privilege, independently of Developer Mode.

use pretty_assertions::assert_eq;
use windows_sys::Win32::Foundation::CloseHandle;
use windows_sys::Win32::Security::AdjustTokenPrivileges;
use windows_sys::Win32::Security::ImpersonateSelf;
use windows_sys::Win32::Security::RevertToSelf;
use windows_sys::Win32::Security::SecurityImpersonation;
use windows_sys::Win32::Security::TOKEN_ADJUST_PRIVILEGES;
use windows_sys::Win32::Security::TOKEN_QUERY;
use windows_sys::Win32::System::Threading::GetCurrentThread;
use windows_sys::Win32::System::Threading::OpenThreadToken;

struct WithoutPrivileges;

impl WithoutPrivileges {
    fn start() -> Self {
        // SAFETY: impersonate only on this test thread; the guard restores it
        // even on panic. The opened token is closed before checking the result.
        unsafe {
            assert_ne!(ImpersonateSelf(SecurityImpersonation), 0);
            let guard = Self;
            let mut token = 0;
            assert_ne!(
                OpenThreadToken(
                    GetCurrentThread(),
                    TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY,
                    /*openasself*/ 1,
                    &mut token,
                ),
                0,
            );
            let disabled = AdjustTokenPrivileges(
                token,
                /*disableallprivileges*/ 1,
                std::ptr::null(),
                /*bufferlength*/ 0,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            );
            CloseHandle(token);
            assert_ne!(disabled, 0);
            guard
        }
    }
}

impl Drop for WithoutPrivileges {
    fn drop(&mut self) {
        // SAFETY: ends this test thread's impersonation.
        unsafe {
            RevertToSelf();
        }
    }
}

#[test]
fn direct_retag_without_the_privilege_is_refused_without_opening_the_target() {
    assert_eq!(
        super::check_retagged_reopen(WithoutPrivileges::start),
        Some(false)
    );
}
