//! Service-only hard-link gate: no accounts, firewall, or service installation.
use super::DaclInheritance;
use super::SetupMode;
use super::lock_sandbox_dir;
use pretty_assertions::assert_eq;
use std::fs;
use windows_sys::Win32::Security::Authorization::GRANT_ACCESS;
use windows_sys::Win32::Storage::FileSystem::DELETE;
use windows_sys::Win32::Storage::FileSystem::FILE_GENERIC_EXECUTE;
use windows_sys::Win32::Storage::FileSystem::FILE_GENERIC_READ;
use windows_sys::Win32::Storage::FileSystem::FILE_GENERIC_WRITE;

fn owner() -> String {
    std::env::var("USERNAME").expect("Windows test account")
}

#[test]
fn issue_2315_marker_refuses_hard_link_without_removing_it() {
    let temp = tempfile::tempdir().unwrap();
    let sandbox = temp.path().join(".sandbox");
    fs::create_dir(&sandbox).unwrap();
    let victim = temp.path().join("outside");
    let marker = sandbox.join("setup_marker.json");
    fs::write(&victim, b"unchanged").unwrap();
    fs::hard_link(&victim, &marker).unwrap();
    let error =
        super::sandbox_users::prepare_setup_marker(temp.path(), &owner(), SetupMode::ProvisionOnly)
            .err()
            .expect("service marker must refuse a hard link");
    assert!(format!("{error:#}").contains("hard-linked"), "{error:#}");
    assert_eq!(fs::read(&victim).unwrap(), b"unchanged");
    assert_eq!(fs::read(&marker).unwrap(), b"unchanged");
}

#[test]
fn issue_2315_service_acl_refuses_hard_links_in_each_sandbox_folder() {
    for folder in [".sandbox", ".sandbox-secrets", ".sandbox-bin"] {
        let temp = tempfile::tempdir().unwrap();
        let sandbox = temp.path().join(folder);
        fs::create_dir(&sandbox).unwrap();
        let victim = temp.path().join("outside");
        fs::write(&victim, b"unchanged").unwrap();
        fs::hard_link(&victim, sandbox.join("planted")).unwrap();
        let before = dacl(&victim);
        let user = owner();
        let sid = crate::resolve_sid(&user).unwrap();
        let error = lock_sandbox_dir(
            &sandbox,
            &user,
            &sid,
            GRANT_ACCESS,
            FILE_GENERIC_READ | FILE_GENERIC_EXECUTE,
            FILE_GENERIC_READ | FILE_GENERIC_WRITE | FILE_GENERIC_EXECUTE | DELETE,
            DaclInheritance::Protected,
            SetupMode::ProvisionOnly,
        )
        .expect_err("service ACL update must refuse a hard link");
        assert!(format!("{error:#}").contains("hard-linked"), "{error:#}");
        assert_eq!(fs::read(&victim).unwrap(), b"unchanged");
        assert_eq!(dacl(&victim), before);
    }
}

#[test]
fn issue_2315_service_acl_refuses_nested_hard_link() {
    let temp = tempfile::tempdir().unwrap();
    let sandbox = temp.path().join(".sandbox-bin");
    fs::create_dir_all(sandbox.join("nested")).unwrap();
    let victim = temp.path().join("outside");
    fs::write(&victim, b"unchanged").unwrap();
    fs::hard_link(&victim, sandbox.join("nested/planted")).unwrap();
    let before = dacl(&victim);
    let user = owner();
    let sid = crate::resolve_sid(&user).unwrap();
    let error = lock_sandbox_dir(
        &sandbox,
        &user,
        &sid,
        GRANT_ACCESS,
        FILE_GENERIC_READ | FILE_GENERIC_EXECUTE,
        FILE_GENERIC_READ | FILE_GENERIC_WRITE | FILE_GENERIC_EXECUTE | DELETE,
        DaclInheritance::Protected,
        SetupMode::ProvisionOnly,
    )
    .expect_err("nested hard link must be refused");
    assert!(format!("{error:#}").contains("hard-linked"), "{error:#}");
    assert_eq!(dacl(&victim), before);
}

#[test]
fn issue_2315_service_marker_commits_through_retained_handle() {
    let temp = tempfile::tempdir().unwrap();
    let sandbox = temp.path().join(".sandbox");
    fs::create_dir(&sandbox).unwrap();
    let marker = sandbox.join("setup_marker.json");
    fs::write(&marker, b"old").unwrap();
    let prepared =
        super::sandbox_users::prepare_setup_marker(temp.path(), &owner(), SetupMode::ProvisionOnly)
            .unwrap();
    assert!(
        fs::remove_file(&marker).is_err(),
        "marker cannot be swapped while held"
    );
    super::sandbox_users::commit_setup_marker(
        prepared,
        temp.path(),
        "offline",
        "online",
        &[],
        /*allow_local_binding*/ false,
    )
    .unwrap();
    let content: serde_json::Value = serde_json::from_slice(&fs::read(marker).unwrap()).unwrap();
    assert_eq!(content["offline_username"], "offline");
}

fn dacl(path: &std::path::Path) -> Vec<u8> {
    use windows_sys::Win32::Security::Authorization::GetNamedSecurityInfoW;
    use windows_sys::Win32::Security::Authorization::SE_FILE_OBJECT;
    use windows_sys::Win32::Security::DACL_SECURITY_INFORMATION;
    let mut acl = std::ptr::null_mut();
    let mut sd = std::ptr::null_mut();
    let wide = crate::to_wide(path);
    let code = unsafe {
        GetNamedSecurityInfoW(
            wide.as_ptr(),
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut acl,
            std::ptr::null_mut(),
            &mut sd,
        )
    };
    assert_eq!(code, 0);
    assert!(!acl.is_null());
    let bytes =
        unsafe { std::slice::from_raw_parts(acl.cast::<u8>(), (*acl).AclSize as usize) }.to_vec();
    unsafe {
        windows_sys::Win32::Foundation::LocalFree(sd);
    }
    bytes
}
