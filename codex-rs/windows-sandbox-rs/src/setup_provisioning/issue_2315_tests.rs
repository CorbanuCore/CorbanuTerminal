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

#[test]
fn issue_2315_service_acl_refuses_cwd_junction_without_touching_target() {
    let temp = tempfile::tempdir().unwrap();
    let sandbox = temp.path().join(".sandbox");
    let outside = temp.path().join("outside");
    fs::create_dir_all(sandbox.join("cwd")).unwrap();
    fs::create_dir(&outside).unwrap();
    let victim = outside.join("file");
    fs::write(&victim, b"unchanged").unwrap();
    let before = dacl(&victim);
    let junction = sandbox.join("cwd").join("hash");
    let output = std::process::Command::new("cmd.exe")
        .args(["/d", "/c", "mklink", "/J"])
        .arg(&junction)
        .arg(&outside)
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
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
    .unwrap_err();
    assert!(
        format!("{error:#}").contains("refusing unsafe"),
        "{error:#}"
    );
    assert_eq!(dacl(&victim), before);
    assert_eq!(fs::read(victim).unwrap(), b"unchanged");
    fs::remove_dir(junction).unwrap();
}

#[test]
fn issue_2315_service_acl_matches_interactive_inheritance() {
    for legacy in [false, true] {
        check_inheritance_parity(legacy);
    }
}

fn check_inheritance_parity(legacy: bool) {
    for initially_protected in [false, true] {
        for inheritance in [DaclInheritance::Inherited, DaclInheritance::Protected] {
            let temp = tempfile::tempdir().unwrap();
            let user = owner();
            let sid = crate::resolve_sid("Users").unwrap();
            let apply = |path: &std::path::Path, mode, inheritance| {
                lock_sandbox_dir(
                    path,
                    &user,
                    &sid,
                    GRANT_ACCESS,
                    FILE_GENERIC_READ | FILE_GENERIC_EXECUTE,
                    FILE_GENERIC_READ | FILE_GENERIC_WRITE | FILE_GENERIC_EXECUTE | DELETE,
                    inheritance,
                    mode,
                )
                .unwrap();
            };
            for (folder, mode) in [
                ("full", SetupMode::Full),
                ("service", SetupMode::ProvisionOnly),
            ] {
                let root = temp.path().join(folder);
                fs::create_dir(&root).unwrap();
                if initially_protected {
                    apply(&root, SetupMode::Full, DaclInheritance::Protected);
                }
                fs::create_dir(root.join("nested")).unwrap();
                fs::write(root.join("nested/file"), b"ordinary").unwrap();
                apply(
                    &root.join("protected"),
                    SetupMode::Full,
                    DaclInheritance::Protected,
                );
                fs::write(root.join("protected/file"), b"protected child").unwrap();
                let explicit_sid = super::sid_bytes_to_psid(&sid).unwrap();
                for relative in ["explicit", "protected/explicit"] {
                    let file = root.join(relative);
                    fs::write(&file, b"explicit child").unwrap();
                    unsafe {
                        crate::ensure_allow_write_aces(&file, &[explicit_sid]).unwrap();
                    }
                }
                unsafe {
                    windows_sys::Win32::Foundation::LocalFree(explicit_sid);
                }
                if legacy {
                    for relative in ["nested/file", "nested"] {
                        make_legacy_acl(&root.join(relative));
                    }
                }
                apply(&root, mode, inheritance);
            }
            for relative in [
                "",
                "nested",
                "nested/file",
                "explicit",
                "protected",
                "protected/file",
                "protected/explicit",
            ] {
                assert_eq!(
                    acl_and_control(&temp.path().join("service").join(relative)),
                    acl_and_control(&temp.path().join("full").join(relative)),
                    "legacy={legacy}, protection={initially_protected}, desired_protection={}, path={relative}",
                    matches!(inheritance, DaclInheritance::Protected)
                );
            }
        }
    }
}

// Model an ACL created before automatic inheritance: equivalent access, without
// inherited ACE markers or the auto-inherited control bit. Do not propagate it.
fn make_legacy_acl(path: &std::path::Path) {
    use std::os::windows::fs::OpenOptionsExt;
    use std::os::windows::io::AsRawHandle;
    use windows_sys::Win32::Security as s;
    use windows_sys::Win32::Security::Authorization as a;
    use windows_sys::Win32::Storage::FileSystem as f;
    let file = fs::OpenOptions::new()
        .access_mode(f::READ_CONTROL | f::WRITE_DAC)
        .custom_flags(f::FILE_FLAG_BACKUP_SEMANTICS)
        .open(path)
        .unwrap();
    let mut sd = std::ptr::null_mut();
    let mut acl = std::ptr::null_mut();
    unsafe {
        assert_eq!(
            a::GetSecurityInfo(
                file.as_raw_handle(),
                a::SE_FILE_OBJECT,
                s::DACL_SECURITY_INFORMATION,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                &mut acl,
                std::ptr::null_mut(),
                &mut sd
            ),
            0
        );
        for index in 0..u32::from((*acl).AceCount) {
            let mut ace = std::ptr::null_mut();
            assert_ne!(s::GetAce(acl, index, &mut ace), 0);
            (*ace.cast::<s::ACE_HEADER>()).AceFlags &= !(s::INHERITED_ACE as u8);
        }
        assert_ne!(
            s::SetSecurityDescriptorControl(
                sd,
                s::SE_DACL_AUTO_INHERITED | s::SE_DACL_AUTO_INHERIT_REQ,
                0
            ),
            0
        );
        assert_ne!(
            s::SetKernelObjectSecurity(file.as_raw_handle(), s::DACL_SECURITY_INFORMATION, sd),
            0
        );
        windows_sys::Win32::Foundation::LocalFree(sd);
    }
    assert_eq!(
        acl_and_control(path).1,
        0,
        "legacy fixture must remain unprotected"
    );
}

fn dacl(path: &std::path::Path) -> Vec<u8> {
    acl_and_control(path).0
}

fn acl_and_control(path: &std::path::Path) -> (Vec<u8>, u16) {
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
    let mut control = 0;
    let mut revision = 0;
    unsafe {
        assert_ne!(
            windows_sys::Win32::Security::GetSecurityDescriptorControl(
                sd,
                &mut control,
                &mut revision
            ),
            0
        );
        windows_sys::Win32::Foundation::LocalFree(sd);
    }
    (
        bytes,
        control
            & (windows_sys::Win32::Security::SE_DACL_PROTECTED
                | windows_sys::Win32::Security::SE_DACL_AUTO_INHERITED),
    )
}
