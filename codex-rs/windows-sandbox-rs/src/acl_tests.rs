//! PF-27-S07: removing the `CODEX_HOME` new-file deny leaves everything else.

use super::add_deny_read_ace;
use super::add_deny_read_ace_for_new_files;
use super::add_deny_write_ace;
use super::remove_deny_read_ace_for_new_files;
use crate::token::LocalSid;
use crate::winutil::to_wide;
use pretty_assertions::assert_eq;
use std::ffi::c_void;
use std::path::Path;
use windows_sys::Win32::Foundation::ERROR_SUCCESS;
use windows_sys::Win32::Foundation::HLOCAL;
use windows_sys::Win32::Foundation::LocalFree;
use windows_sys::Win32::Security::Authorization::ConvertSecurityDescriptorToStringSecurityDescriptorW;
use windows_sys::Win32::Security::Authorization::GetNamedSecurityInfoW;
use windows_sys::Win32::Security::Authorization::SDDL_REVISION_1;
use windows_sys::Win32::Security::Authorization::SE_FILE_OBJECT;
use windows_sys::Win32::Security::DACL_SECURITY_INFORMATION;

const SANDBOX_GROUP: &str = "S-1-5-21-2718281828-3141592653-1618033988-1001";
const OTHER_SID: &str = "S-1-5-21-2718281828-3141592653-1618033988-1002";

#[test]
fn pf_27_s07_removes_only_the_new_file_deny() {
    let dir = tempfile::tempdir().expect("dir");
    let dir = dir.path();
    let group = LocalSid::from_string(SANDBOX_GROUP).expect("group SID");
    let other = LocalSid::from_string(OTHER_SID).expect("other SID");
    let existing = dir.join("existing.txt");
    std::fs::write(&existing, "x").expect("existing file");
    // Unrelated entries, including other denies for the same SID.
    // SAFETY: valid SIDs and existing paths.
    unsafe {
        assert!(add_deny_read_ace(dir, other.as_ptr()).expect("other deny"));
        assert!(add_deny_write_ace(dir, group.as_ptr()).expect("write deny"));
        assert!(add_deny_read_ace(&existing, group.as_ptr()).expect("file deny"));
    }
    let dir_before = dacl_sddl(dir);
    let existing_before = dacl_sddl(&existing);

    // SAFETY: as above.
    let added = unsafe { add_deny_read_ace_for_new_files(dir, group.as_ptr()) };
    assert!(added.expect("new-file deny"));
    let replaced = dir.join("replaced.txt");
    std::fs::write(&replaced, "y").expect("file created while protected");
    assert_ne!(dacl_sddl(dir), dir_before);
    assert_ne!(dacl_sddl(&existing), existing_before);

    // SAFETY: as above.
    let removed = unsafe { remove_deny_read_ace_for_new_files(dir, group.as_ptr()) };
    assert!(removed.expect("remove"));
    assert_eq!(dacl_sddl(dir), dir_before);
    assert_eq!(dacl_sddl(&existing), existing_before);
    let fresh = dir.join("fresh.txt");
    std::fs::write(&fresh, "z").expect("file created after removal");
    assert_eq!(dacl_sddl(&replaced), dacl_sddl(&fresh));

    // SAFETY: as above.
    let again = unsafe { remove_deny_read_ace_for_new_files(dir, group.as_ptr()) };
    assert!(!again.expect("second remove"));
    assert_eq!(dacl_sddl(dir), dir_before);
}

/// Another SID's new-file deny is not ours to remove.
#[test]
fn pf_27_s07_keeps_a_new_file_deny_for_another_sid() {
    let dir = tempfile::tempdir().expect("dir");
    let dir = dir.path();
    let group = LocalSid::from_string(SANDBOX_GROUP).expect("group SID");
    let other = LocalSid::from_string(OTHER_SID).expect("other SID");
    // SAFETY: valid SIDs and an existing directory.
    unsafe {
        assert!(add_deny_read_ace_for_new_files(dir, other.as_ptr()).expect("other deny"));
    }
    let before = dacl_sddl(dir);
    // SAFETY: as above.
    let removed = unsafe { remove_deny_read_ace_for_new_files(dir, group.as_ptr()) };
    assert!(!removed.expect("remove"));
    assert_eq!(dacl_sddl(dir), before);
}

fn dacl_sddl(path: &Path) -> String {
    let mut sd: *mut c_void = std::ptr::null_mut();
    // SAFETY: valid path; `sd` is freed below.
    let code = unsafe {
        GetNamedSecurityInfoW(
            to_wide(path).as_ptr(),
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut sd,
        )
    };
    assert_eq!(code, ERROR_SUCCESS, "GetNamedSecurityInfoW");
    let mut text: *mut u16 = std::ptr::null_mut();
    // SAFETY: `sd` is a valid descriptor; `text` is freed below.
    let ok = unsafe {
        ConvertSecurityDescriptorToStringSecurityDescriptorW(
            sd,
            SDDL_REVISION_1,
            DACL_SECURITY_INFORMATION,
            &mut text,
            std::ptr::null_mut(),
        )
    };
    assert_ne!(
        ok, 0,
        "ConvertSecurityDescriptorToStringSecurityDescriptorW"
    );
    // SAFETY: `text` is NUL-terminated.
    let sddl = unsafe {
        let len = (0..).take_while(|&i| *text.add(i) != 0).count();
        String::from_utf16_lossy(std::slice::from_raw_parts(text, len))
    };
    // SAFETY: both allocated with LocalAlloc by the calls above.
    unsafe {
        LocalFree(text as HLOCAL);
        LocalFree(sd as HLOCAL);
    }
    sddl
}
