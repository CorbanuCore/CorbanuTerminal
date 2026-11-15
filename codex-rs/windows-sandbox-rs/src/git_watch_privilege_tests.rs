//! Exercise direct retagging without the process privilege, independently of Developer Mode.

use pretty_assertions::assert_eq;
use windows_sys::Win32::Foundation::CloseHandle;
use windows_sys::Win32::Foundation::ERROR_NOT_ALL_ASSIGNED;
use windows_sys::Win32::Foundation::ERROR_SUCCESS;
use windows_sys::Win32::Foundation::GetLastError;
use windows_sys::Win32::Security::AdjustTokenPrivileges;
use windows_sys::Win32::Security::LUID_AND_ATTRIBUTES;
use windows_sys::Win32::Security::LookupPrivilegeValueW;
use windows_sys::Win32::Security::SE_PRIVILEGE_REMOVED;
use windows_sys::Win32::Security::TOKEN_ADJUST_PRIVILEGES;
use windows_sys::Win32::Security::TOKEN_PRIVILEGES;
use windows_sys::Win32::Security::TOKEN_QUERY;
use windows_sys::Win32::System::Threading::GetCurrentProcess;
use windows_sys::Win32::System::Threading::OpenProcessToken;

const CHILD: &str = "CORBANU_1909_NO_SYMLINK_PRIVILEGE";
const TEST: &str = "git_watch::tests::privilege_tests::direct_retag_without_the_privilege_is_refused_without_opening_the_target";

/// Only called in the isolated child: changing its primary token must not
/// affect other tests. A thread impersonation token alone does not exercise
/// the refusal on every supported Windows version.
fn remove_process_privilege() {
    // SAFETY: this child owns its process token; the opened handle is closed
    // before checking the adjustment result. No other tests run in this child.
    unsafe {
        let mut token = 0;
        assert_ne!(
            OpenProcessToken(
                GetCurrentProcess(),
                TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY,
                &mut token,
            ),
            0,
        );
        let mut luid = std::mem::zeroed();
        let name = crate::winutil::to_wide("SeCreateSymbolicLinkPrivilege");
        let found = LookupPrivilegeValueW(std::ptr::null(), name.as_ptr(), &mut luid);
        let privileges = TOKEN_PRIVILEGES {
            PrivilegeCount: 1,
            Privileges: [LUID_AND_ATTRIBUTES {
                Luid: luid,
                Attributes: SE_PRIVILEGE_REMOVED,
            }],
        };
        let removed = AdjustTokenPrivileges(
            token,
            /*disableallprivileges*/ 0,
            &privileges,
            /*bufferlength*/ 0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        );
        let error = GetLastError();
        CloseHandle(token);
        assert_ne!(found, 0);
        assert_ne!(removed, 0);
        assert!(error == ERROR_SUCCESS || error == ERROR_NOT_ALL_ASSIGNED);
    }
}

#[test]
fn direct_retag_without_the_privilege_is_refused_without_opening_the_target() {
    if std::env::var_os(CHILD).is_some() {
        assert_eq!(
            super::check_retagged_reopen(remove_process_privilege),
            Some(false)
        );
        return;
    }
    let output = std::process::Command::new(std::env::current_exe().expect("test executable"))
        .args(["--exact", TEST, "--nocapture"])
        .env(CHILD, "1")
        .output()
        .expect("isolated privilege test");
    assert!(
        output.status.success() && String::from_utf8_lossy(&output.stdout).contains("1 passed"),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}
