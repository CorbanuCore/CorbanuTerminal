//! #2584: the records folder needs Full control (WRITE_OWNER) for its label.
//! CI also runs these `sec_win_` tests with a normal session's medium token.

use super::*;
use crate::integrity::LabelAce;
use crate::integrity::read_explicit_labels;
use pretty_assertions::assert_eq;
use windows_sys::Win32::Foundation::ERROR_SUCCESS;
use windows_sys::Win32::Foundation::HLOCAL;
use windows_sys::Win32::Foundation::LocalFree;
use windows_sys::Win32::Security::ACL;
use windows_sys::Win32::Security::Authorization::ConvertStringSecurityDescriptorToSecurityDescriptorW;
use windows_sys::Win32::Security::Authorization::SE_FILE_OBJECT;
use windows_sys::Win32::Security::Authorization::SetNamedSecurityInfoW;
use windows_sys::Win32::Security::DACL_SECURITY_INFORMATION;
use windows_sys::Win32::Security::GetSecurityDescriptorDacl;
use windows_sys::Win32::Security::PROTECTED_DACL_SECURITY_INFORMATION;

/// Modify without WRITE_DAC/WRITE_OWNER, as Authenticated Users get below
/// `C:\` (the 0.1.49 QA profile's AppData\Local).
const MODIFY: &str = "0x1301bf";

/// A private stand-in for AppData\Local whose only entries are `user_rights`
/// for this user and Full control for SYSTEM, protected from inheritance.
fn local_app_data(user_rights: &str) -> (tempfile::TempDir, PathBuf) {
    let base = tempfile::tempdir().expect("temp");
    let local = base.path().join("LocalAppData");
    std::fs::create_dir(&local).expect("LocalAppData");
    let user = crate::deny_read_sessions::current_user_sid().expect("user");
    let sddl = format!("D:P(A;OICI;{user_rights};;;{user})(A;OICI;FA;;;SY)");
    // SAFETY: the descriptor is freed below; its DACL is used before that.
    unsafe {
        let mut descriptor = std::ptr::null_mut();
        assert_ne!(
            ConvertStringSecurityDescriptorToSecurityDescriptorW(
                crate::winutil::to_wide(&sddl).as_ptr(),
                1,
                &mut descriptor,
                std::ptr::null_mut(),
            ),
            0,
            "{sddl}"
        );
        let (mut present, mut defaulted) = (0, 0);
        let mut dacl: *mut ACL = std::ptr::null_mut();
        assert_ne!(
            GetSecurityDescriptorDacl(descriptor, &mut present, &mut dacl, &mut defaulted),
            0
        );
        let code = SetNamedSecurityInfoW(
            crate::winutil::to_wide(&local).as_ptr(),
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION | PROTECTED_DACL_SECURITY_INFORMATION,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            dacl,
            std::ptr::null(),
        );
        LocalFree(descriptor as HLOCAL);
        assert_eq!(code, ERROR_SUCCESS, "set the DACL of {}", local.display());
    }
    (base, local)
}

/// The fresh QA profile's case: the user may create the records folder but
/// not label it. The command is refused (fail closed) with the Win32 cause
/// and what the folder needs, not just "label ... Medium".
#[test]
fn sec_win_2584_records_folder_without_write_owner_names_the_cause() {
    let (_base, local) = local_app_data(MODIFY);
    let _local = use_test_local_app_data(&local);
    let parent = registry_parent_dir().expect("parent");

    let err = ensure_registry().expect_err("a folder that can't be labelled is refused");

    let message = err.to_string();
    for needed in [
        format!("label {} Medium", parent.display()),
        "SetNamedSecurityInfoW(label) failed".to_string(),
        std::io::Error::from_raw_os_error(5).to_string(),
        "WRITE_OWNER".to_string(),
    ] {
        assert!(message.contains(&needed), "{needed:?} missing: {message}");
    }
    // Nothing was trusted or made below the unlabelled folder.
    // SAFETY: reads the label of a folder this test made.
    assert_eq!(
        unsafe { read_explicit_labels(&parent) }.expect("labels"),
        Vec::<LabelAce>::new()
    );
    assert!(!parent.join(REGISTRY_DIR).exists());
}

/// Positive control: with Full control, as in a user's own AppData\Local,
/// the same token makes and labels both records folders.
#[test]
fn sec_win_2584_records_folder_with_full_control_is_labelled_medium() {
    let (_base, local) = local_app_data("FA");
    let _local = use_test_local_app_data(&local);
    let parent = registry_parent_dir().expect("parent");

    let registry = ensure_registry().expect("records folder");

    assert_eq!(registry, parent.join(REGISTRY_DIR));
    let private = LabelAce {
        flags: 0x3, // OBJECT_INHERIT_ACE | CONTAINER_INHERIT_ACE
        mask: 0x7,  // no-write-up, no-read-up, no-execute-up
        sid: "S-1-16-8192".to_string(),
    };
    for dir in [&parent, &registry] {
        // SAFETY: reads the label of a folder this test made.
        let labels = unsafe { read_explicit_labels(dir) }.expect("labels");
        assert_eq!(labels, [private.clone()], "{}", dir.display());
    }
}
