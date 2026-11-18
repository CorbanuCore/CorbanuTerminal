use std::ffi::c_void;
use std::io::Write;
use std::os::windows::ffi::OsStrExt as _;
use std::os::windows::io::AsRawHandle as _;
use std::os::windows::io::OwnedHandle;
use std::path::Path;
use std::path::PathBuf;

use crate::acl::grant_read_execute_aces_to_handle;
use crate::acl::handle_mask_allows;
use crate::no_reparse_dir::DirectoryOpenDisposition;
use crate::no_reparse_dir::open_directory_no_reparse;
use crate::no_reparse_dir::open_no_reparse;
use anyhow::Context;
use anyhow::Result;
use anyhow::ensure;
use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::Security::CONTAINER_INHERIT_ACE;
use windows_sys::Win32::Security::OBJECT_INHERIT_ACE;
use windows_sys::Win32::Storage::FileSystem::BY_HANDLE_FILE_INFORMATION;
use windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_DIRECTORY;
use windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_REPARSE_POINT;
use windows_sys::Win32::Storage::FileSystem::FILE_GENERIC_EXECUTE;
use windows_sys::Win32::Storage::FileSystem::FILE_GENERIC_READ;
use windows_sys::Win32::Storage::FileSystem::FILE_READ_ATTRIBUTES;
use windows_sys::Win32::Storage::FileSystem::FILE_SHARE_DELETE;
use windows_sys::Win32::Storage::FileSystem::FILE_SHARE_READ;
use windows_sys::Win32::Storage::FileSystem::FILE_SHARE_WRITE;
use windows_sys::Win32::Storage::FileSystem::GetFileInformationByHandle;
use windows_sys::Win32::Storage::FileSystem::READ_CONTROL;
use windows_sys::Win32::Storage::FileSystem::SYNCHRONIZE;
use windows_sys::Win32::Storage::FileSystem::WRITE_DAC;

#[cfg(test)]
#[path = "setup_runtime_bin_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "setup_runtime_long_paths_tests.rs"]
mod long_paths_tests;

/// D611: what an ACL update needs, and the attributes to refuse links with.
const ACL_ACCESS: u32 = READ_CONTROL | WRITE_DAC | FILE_READ_ATTRIBUTES | SYNCHRONIZE;
/// Others keep using a runtime while its ACL is updated.
const SHARE_ALL: u32 = FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE;
// NtCreateFile: FILE_OPEN; FILE_OPEN_REPARSE_POINT | FILE_OPEN_FOR_BACKUP_INTENT
// | FILE_SYNCHRONOUS_IO_NONALERT.
const FILE_OPEN: u32 = 1;
const OPEN_OPTIONS: u32 = 0x0020_0000 | 0x4000 | 0x20;

pub(super) fn ensure_codex_app_runtime_paths_readable(
    sandbox_group_psid: *mut c_void,
    refresh_errors: &mut Vec<String>,
    log: &mut dyn Write,
) -> Result<()> {
    let read_execute_mask = FILE_GENERIC_READ | FILE_GENERIC_EXECUTE;
    let local_app_data = local_app_data_root();
    let runtime_paths = runtime_paths(
        local_app_data.clone(),
        std::env::var_os("USERPROFILE").map(PathBuf::from),
    );

    for runtime_path in runtime_paths {
        // D611: no link anywhere on the way, and the grant goes through this
        // handle, so swapping a folder for a link cannot redirect it.
        let Some(handle) = open_runtime_directory(&runtime_path) else {
            continue;
        };
        let raw = handle.as_raw_handle() as HANDLE;

        let has_access = match unsafe {
            handle_mask_allows(
                raw,
                &[sandbox_group_psid],
                read_execute_mask,
                /*require_all_bits*/ true,
            )
        } {
            Ok(has_access) => has_access,
            Err(err) => {
                refresh_errors.push(format!(
                    "runtime read/execute mask check failed on {} for sandbox_group: {err}",
                    runtime_path.display()
                ));
                super::log_line(
                    log,
                    &format!(
                        "runtime read/execute mask check failed on {} for sandbox_group: {err}; continuing",
                        runtime_path.display()
                    ),
                )?;
                false
            }
        };
        if has_access {
            continue;
        }

        super::log_line(
            log,
            &format!(
                "granting read/execute ACE to {} for sandbox users",
                runtime_path.display()
            ),
        )?;
        let result = unsafe {
            grant_read_execute_aces_to_handle(
                raw,
                &[sandbox_group_psid],
                OBJECT_INHERIT_ACE | CONTAINER_INHERIT_ACE,
            )
        };
        if let Err(err) = result {
            refresh_errors.push(format!(
                "grant read/execute ACE failed on {} for sandbox_group: {err}",
                runtime_path.display()
            ));
            super::log_line(
                log,
                &format!(
                    "grant read/execute ACE failed on {} for sandbox_group: {err}",
                    runtime_path.display()
                ),
            )?;
        }
    }
    if let Some(local_app_data) = local_app_data {
        let runtime_root = local_app_data.join("OpenAI").join("Codex").join("runtimes");
        if let Err(err) = ensure_runtime_tree_readable(&runtime_root, sandbox_group_psid) {
            let message = format!("runtime read/execute validation failed: {err:#}");
            super::log_line(log, &message)?;
            refresh_errors.push(message);
        }
    }
    Ok(())
}

/// Opens an existing runtime directory with no reparse point in any path
/// component. `None` when it is missing, not a local directory, or behind a
/// link: such a runtime is left alone, as before.
fn open_runtime_directory(path: &Path) -> Option<OwnedHandle> {
    let handle = open_directory_no_reparse(
        path,
        ACL_ACCESS,
        SHARE_ALL,
        DirectoryOpenDisposition::OpenExisting,
    )
    .ok()?;
    attributes(&handle)
        .is_ok_and(|attributes| {
            attributes & FILE_ATTRIBUTE_DIRECTORY != 0
                && attributes & FILE_ATTRIBUTE_REPARSE_POINT == 0
        })
        .then_some(handle)
}

/// Opens `name` inside the directory `parent` names, refusing to follow a
/// reparse point. `Ok(None)` when the entry is gone or is itself a link.
fn open_runtime_child(parent: &OwnedHandle, name: &std::ffi::OsStr) -> Result<Option<OwnedHandle>> {
    let mut wide: Vec<u16> = name.encode_wide().chain(std::iter::once(0)).collect();
    let handle = match open_no_reparse(
        parent.as_raw_handle() as HANDLE,
        &mut wide,
        ACL_ACCESS,
        SHARE_ALL,
        FILE_OPEN,
        OPEN_OPTIONS,
    ) {
        Ok(handle) => handle,
        Err(err)
            if err.chain().any(|cause| {
                cause.to_string() == "path contains a reparse point"
                    || cause
                        .downcast_ref::<std::io::Error>()
                        .is_some_and(|err| err.kind() == std::io::ErrorKind::NotFound)
            }) =>
        {
            return Ok(None);
        }
        Err(err) => return Err(err),
    };
    Ok((attributes(&handle)? & FILE_ATTRIBUTE_REPARSE_POINT == 0).then_some(handle))
}

fn attributes(handle: &OwnedHandle) -> std::io::Result<u32> {
    let mut info: BY_HANDLE_FILE_INFORMATION = unsafe { std::mem::zeroed() };
    // SAFETY: a live handle and a correctly sized output buffer.
    if unsafe { GetFileInformationByHandle(handle.as_raw_handle() as HANDLE, &mut info) } == 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(info.dwFileAttributes)
}

fn ensure_runtime_tree_readable(
    runtime_root: &Path,
    sandbox_group_psid: *mut c_void,
) -> Result<()> {
    // D611: a missing root, or one behind a link anywhere on its path, is left
    // alone, as before. Each entry below is opened through its pinned parent.
    let Some(root) = open_runtime_directory(runtime_root) else {
        return Ok(());
    };

    let read_execute_mask = FILE_GENERIC_READ | FILE_GENERIC_EXECUTE;
    let mut entries = vec![(runtime_root.to_path_buf(), root, true)];
    while let Some((path, handle, is_dir)) = entries.pop() {
        let raw = handle.as_raw_handle() as HANDLE;
        // A readable parent does not prove that existing runtime children inherited its ACL.
        unsafe {
            grant_read_execute_aces_to_handle(
                raw,
                &[sandbox_group_psid],
                if is_dir {
                    OBJECT_INHERIT_ACE | CONTAINER_INHERIT_ACE
                } else {
                    0
                },
            )
        }
        .and_then(|changed| {
            if changed {
                ensure!(
                    unsafe {
                        handle_mask_allows(
                            raw,
                            &[sandbox_group_psid],
                            read_execute_mask,
                            /*require_all_bits*/ true,
                        )
                    }?,
                    "runtime read/execute access still missing"
                );
            }
            Ok(())
        })
        .with_context(|| format!("validate runtime read/execute access on {}", path.display()))?;
        if !is_dir {
            continue;
        }
        // Names only: each one is opened relative to `handle`, so a renamed
        // or relinked folder at `path` cannot redirect them.
        let names = match std::fs::read_dir(&path) {
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => continue,
            result => {
                result.with_context(|| format!("enumerate runtime directory {}", path.display()))?
            }
        };
        let mut children = Vec::new();
        for name in names {
            match name {
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => continue,
                result => children.push(result?.file_name()),
            }
        }
        #[cfg(test)]
        tests::after_listing(&path);
        for name in children {
            let Some(child) = open_runtime_child(&handle, &name)
                .with_context(|| format!("open runtime entry {}", path.join(&name).display()))?
            else {
                continue;
            };
            let child_is_dir = attributes(&child)? & FILE_ATTRIBUTE_DIRECTORY != 0;
            entries.push((path.join(&name), child, child_is_dir));
        }
    }
    Ok(())
}

fn runtime_paths(local_app_data: Option<PathBuf>, user_profile: Option<PathBuf>) -> Vec<PathBuf> {
    let mut runtime_paths = Vec::new();
    if let Some(local_app_data) = local_app_data {
        // D611: only the folders main granted, not the whole OpenAI\Codex folder.
        let codex_root = local_app_data.join("OpenAI").join("Codex");
        runtime_paths.extend([codex_root.join("bin"), codex_root.join("runtimes")]);
    }
    // The managed primary runtime is installed outside the LocalAppData runtime roots.
    if let Some(user_profile) = user_profile {
        runtime_paths.push(user_profile.join(".cache").join("codex-runtimes"));
    }

    runtime_paths
}

fn local_app_data_root() -> Option<PathBuf> {
    std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("USERPROFILE")
                .map(PathBuf::from)
                .map(|profile| profile.join("AppData").join("Local"))
        })
}
