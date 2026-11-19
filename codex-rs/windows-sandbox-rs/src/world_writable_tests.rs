use super::everyone_writable_folders;
use super::gather_candidates;
use pretty_assertions::assert_eq;
use std::collections::HashMap;
use std::fs;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;

/// Grants Everyone modify on `path` and its children (`icacls`, the user's
/// own temp folder only).
pub(crate) fn grant_everyone_modify(path: &Path) {
    let output = std::process::Command::new("icacls")
        .arg(path)
        .args(["/grant", "*S-1-1-0:(OI)(CI)M"])
        .output()
        .expect("run icacls");
    assert!(output.status.success(), "icacls failed: {output:?}");
}

#[test]
fn gathers_path_entries_by_list_separator() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let dir_a = tmp.path().join("Tools");
    let dir_b = tmp.path().join("Bin");
    let dir_space = tmp.path().join("Program Files");
    for dir in [&dir_a, &dir_b, &dir_space] {
        fs::create_dir_all(dir).expect("dir");
    }
    let env = HashMap::from([(
        "PATH".to_string(),
        format!(
            "{};{};{}",
            dir_a.display(),
            dir_b.display(),
            dir_space.display()
        ),
    )]);

    let candidates = gather_candidates(tmp.path(), &env);
    for dir in [&dir_a, &dir_b, &dir_space] {
        let canonical = dunce::canonicalize(dir).expect("canonical");
        assert!(candidates.contains(&canonical), "{}", canonical.display());
    }
}

/// D607: a folder that grants Everyone write access is flagged, from the
/// working folder's children and from `PATH`; a folder that does not, and a
/// link to the flagged one, are not.
#[test]
fn d607_everyone_writable_folders_are_flagged() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let workspace = tmp.path().join("workspace");
    let open = workspace.join("open");
    let closed = workspace.join("closed");
    let tools = tmp.path().join("tools");
    for dir in [&open, &closed, &tools] {
        fs::create_dir_all(dir).expect("dir");
    }
    grant_everyone_modify(&open);
    grant_everyone_modify(&tools);
    let link = workspace.join("link-to-tools");
    let output = std::process::Command::new("cmd")
        .args(["/C", "mklink", "/J"])
        .arg(&link)
        .arg(&tools)
        .output()
        .expect("create junction");
    assert!(output.status.success(), "mklink failed: {output:?}");
    let env = HashMap::from([("PATH".to_string(), tools.display().to_string())]);

    let flagged = everyone_writable_folders(&workspace, &env, /*logs_base_dir*/ None);
    let canonical = |path: &Path| dunce::canonicalize(path).expect("canonical");
    let flagged: Vec<_> = flagged.iter().map(|path| canonical(path)).collect();
    for expected in [&open, &tools] {
        assert!(
            flagged.contains(&canonical(expected)),
            "{} not flagged: {flagged:?}",
            expected.display()
        );
    }
    assert!(!flagged.contains(&canonical(&closed)), "{flagged:?}");
    assert_eq!(
        flagged
            .iter()
            .filter(|path| **path == canonical(&tools))
            .count(),
        1,
        "the link is not followed and the PATH entry is flagged once"
    );
    fs::remove_dir(&link).expect("remove junction");
}

/// Whether a write-restricted sandbox token holding capability `sid` (with
/// the logon SID and Everyone as the other restricting SIDs, the shape of
/// every sandbox token) may add a file to `folder`, by `AccessCheck` against
/// the folder's real security descriptor.
pub(crate) fn sandbox_token_may_add_files(sid: &str, folder: &Path) -> bool {
    use crate::LocalSid;
    use crate::token::create_workspace_write_token_with_caps_from;
    use crate::token::get_current_token_for_restriction;
    use std::os::windows::io::AsRawHandle;
    use std::os::windows::io::FromRawHandle;
    use std::os::windows::io::OwnedHandle;
    use windows_sys::Win32::Foundation::HANDLE;
    use windows_sys::Win32::Foundation::HLOCAL;
    use windows_sys::Win32::Foundation::LocalFree;
    use windows_sys::Win32::Security::AccessCheck;
    use windows_sys::Win32::Security::Authorization::GetNamedSecurityInfoW;
    use windows_sys::Win32::Security::DACL_SECURITY_INFORMATION;
    use windows_sys::Win32::Security::DuplicateToken;
    use windows_sys::Win32::Security::GENERIC_MAPPING;
    use windows_sys::Win32::Security::GROUP_SECURITY_INFORMATION;
    use windows_sys::Win32::Security::OWNER_SECURITY_INFORMATION;
    use windows_sys::Win32::Security::SecurityImpersonation;
    use windows_sys::Win32::Storage::FileSystem::FILE_ALL_ACCESS;
    use windows_sys::Win32::Storage::FileSystem::FILE_GENERIC_EXECUTE;
    use windows_sys::Win32::Storage::FileSystem::FILE_GENERIC_READ;
    use windows_sys::Win32::Storage::FileSystem::FILE_GENERIC_WRITE;
    use windows_sys::Win32::Storage::FileSystem::FILE_WRITE_DATA;

    unsafe {
        let base = OwnedHandle::from_raw_handle(
            get_current_token_for_restriction().expect("base token") as _,
        );
        let capability = LocalSid::from_string(sid).expect("capability SID");
        let token = OwnedHandle::from_raw_handle(
            create_workspace_write_token_with_caps_from(
                base.as_raw_handle() as HANDLE,
                &[capability.as_ptr()],
            )
            .expect("sandbox token") as _,
        );
        let mut impersonation = std::ptr::null_mut();
        assert!(
            DuplicateToken(
                token.as_raw_handle() as HANDLE,
                SecurityImpersonation,
                &mut impersonation
            ) != 0,
            "DuplicateToken failed"
        );
        let impersonation = OwnedHandle::from_raw_handle(impersonation as _);
        let wide: Vec<u16> = folder
            .as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        let mut descriptor = std::ptr::null_mut();
        let code = GetNamedSecurityInfoW(
            wide.as_ptr(),
            /*SE_FILE_OBJECT*/ 1,
            OWNER_SECURITY_INFORMATION | GROUP_SECURITY_INFORMATION | DACL_SECURITY_INFORMATION,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut descriptor,
        );
        assert_eq!(code, 0, "GetNamedSecurityInfoW failed");
        let mapping = GENERIC_MAPPING {
            GenericRead: FILE_GENERIC_READ,
            GenericWrite: FILE_GENERIC_WRITE,
            GenericExecute: FILE_GENERIC_EXECUTE,
            GenericAll: FILE_ALL_ACCESS,
        };
        let mut privileges = [0u64; 128];
        let mut privilege_bytes = std::mem::size_of_val(&privileges) as u32;
        let mut granted = 0;
        let mut allowed = 0;
        let ok = AccessCheck(
            descriptor,
            impersonation.as_raw_handle() as HANDLE,
            FILE_WRITE_DATA,
            &mapping,
            privileges.as_mut_ptr().cast(),
            &mut privilege_bytes,
            &mut granted,
            &mut allowed,
        );
        LocalFree(descriptor as HLOCAL);
        assert!(ok != 0, "AccessCheck failed");
        allowed != 0
    }
}
