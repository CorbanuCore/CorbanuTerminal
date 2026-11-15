//! Exercise direct retagging without the process privilege, independently of Developer Mode.

use pretty_assertions::assert_eq;
use windows_sys::Win32::Foundation::CloseHandle;
use windows_sys::Win32::Foundation::ERROR_NOT_ALL_ASSIGNED;
use windows_sys::Win32::Foundation::ERROR_SUCCESS;
use windows_sys::Win32::Foundation::GetLastError;
use windows_sys::Win32::Security::AdjustTokenPrivileges;
use windows_sys::Win32::Security::LUID_AND_ATTRIBUTES;
use windows_sys::Win32::Security::LookupPrivilegeValueW;
use windows_sys::Win32::Security::SE_PRIVILEGE_ENABLED;
use windows_sys::Win32::Security::SE_PRIVILEGE_REMOVED;
use windows_sys::Win32::Security::TOKEN_ADJUST_PRIVILEGES;
use windows_sys::Win32::Security::TOKEN_PRIVILEGES;
use windows_sys::Win32::Security::TOKEN_QUERY;
use windows_sys::Win32::System::Threading::GetCurrentProcess;
use windows_sys::Win32::System::Threading::OpenProcessToken;

const CHILD: &str = "CORBANU_1909_NO_SYMLINK_PRIVILEGE";
const TEST: &str =
    "git_watch::tests::privilege_tests::direct_retag_without_the_privilege_never_opens_the_target";

/// Only called in the isolated child: changing its primary token must not
/// affect other tests. Verify its token state independently of whether the
/// filesystem refuses the retag on this Windows version.
fn remove_process_privilege() {
    // SAFETY: this child owns its process token; the opened handle is closed
    // before checking the adjustment result. No other tests run in this child.
    unsafe {
        // Upstream's windows-sys makes HANDLE a pointer (main's is an integer).
        let mut token: windows_sys::Win32::Foundation::HANDLE = std::ptr::null_mut();
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
        // A removed (or absent) privilege cannot be re-enabled. Check the
        // actual token state, not an assumption about the filesystem's reply.
        let mut enable = privileges;
        enable.Privileges[0].Attributes = SE_PRIVILEGE_ENABLED;
        let checked = AdjustTokenPrivileges(
            token,
            /*disableallprivileges*/ 0,
            &enable,
            /*bufferlength*/ 0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        );
        let enable_error = GetLastError();
        CloseHandle(token);
        assert_ne!(checked, 0);
        assert_eq!(enable_error, ERROR_NOT_ALL_ASSIGNED, "privilege is absent");
        assert_ne!(found, 0);
        assert_ne!(removed, 0);
        assert!(error == ERROR_SUCCESS || error == ERROR_NOT_ALL_ASSIGNED);
    }
}

/// The refusal branch must run on every platform, even where the native
/// filesystem allows a retag without the privilege. The PC also exercises
/// the real 1314 result; this injection makes that branch deterministic in CI.
#[test]
fn direct_retag_privilege_refusal_never_opens_the_target() {
    assert_eq!(
        super::check_retagged_reopen_with(
            || (),
            |_, _, _| Err(std::io::Error::from_raw_os_error(1314)),
        ),
        Some(false)
    );
}

#[test]
fn direct_retag_without_the_privilege_never_opens_the_target() {
    if std::env::var_os(CHILD).is_some() {
        // Windows 11 refuses the retag; Server 2022 can allow it even
        // without this privilege. Both paths must exercise the hook and
        // preserve the helper's strict no-target-open assertions.
        let retagged = super::check_retagged_reopen(remove_process_privilege);
        assert!(retagged.is_some(), "the retag hook must run");
        eprintln!("retag without the symlink privilege: {retagged:?}");
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
    eprintln!("{}", String::from_utf8_lossy(&output.stderr));
}
