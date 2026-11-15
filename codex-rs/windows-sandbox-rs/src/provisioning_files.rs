//! Service-only leaf validation and ACL inheritance. Never let a pathname-based
//! recursive ACL setter visit user-controlled children on SYSTEM's behalf.

use anyhow::Context;
use anyhow::Result;
use anyhow::ensure;
use std::ffi::OsStr;
use std::ffi::c_void;
use std::os::windows::ffi::OsStrExt;
use std::os::windows::ffi::OsStringExt;
use std::os::windows::io::AsRawHandle;
use std::os::windows::io::BorrowedHandle;
use std::os::windows::io::OwnedHandle;
use std::path::Path;
use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::Foundation::LocalFree;
use windows_sys::Win32::Foundation::NTSTATUS;
use windows_sys::Win32::Foundation::RtlNtStatusToDosError;
use windows_sys::Win32::Security as security;
use windows_sys::Win32::Security::Authorization::GetSecurityInfo;
use windows_sys::Win32::Security::Authorization::SE_FILE_OBJECT;
use windows_sys::Win32::Storage::FileSystem as fs;

use crate::no_reparse_dir::DirectoryOpenDisposition;
use crate::no_reparse_dir::open_directory_no_reparse;
use crate::no_reparse_dir::open_no_reparse;

#[cfg(test)]
#[path = "provisioning_files_tests.rs"]
mod tests;

const FILE_OPEN: u32 = 1;
const FILE_SYNCHRONOUS_IO_NONALERT: u32 = 0x20;
const MAX_ENTRIES: usize = 10_000;

#[link(name = "ntdll")]
unsafe extern "system" {
    fn NtSetSecurityObject(handle: HANDLE, information: u32, descriptor: *const c_void)
    -> NTSTATUS;
}

/// Refuses a reparse point or multiply linked leaf through its verified parent.
/// The returned handle identifies exactly the object checked, even if renamed.
pub fn open_provisioning_entry(parent: BorrowedHandle<'_>, name: &OsStr) -> Result<OwnedHandle> {
    checked_entry(parent.as_raw_handle(), name, fs::FILE_READ_ATTRIBUTES)
}

fn checked_entry(parent: HANDLE, name: &OsStr, access: u32) -> Result<OwnedHandle> {
    let mut wide: Vec<u16> = name.encode_wide().collect();
    ensure!(
        !wide.is_empty()
            && name != "."
            && name != ".."
            && !wide.iter().any(|c| [0, 47, 58, 92].contains(c)),
        "provisioning requires a single file name"
    );
    wide.push(0);
    let handle = open_no_reparse(
        parent,
        &mut wide,
        access | fs::FILE_READ_ATTRIBUTES,
        fs::FILE_SHARE_READ | fs::FILE_SHARE_WRITE | fs::FILE_SHARE_DELETE,
        FILE_OPEN,
        /*create_options*/ 0,
    )
    .with_context(|| {
        format!(
            "refusing unsafe sandbox entry {}; \
             remove the link or restore a regular file before retrying setup",
            name.to_string_lossy(),
        )
    })?;
    check_handle(handle.as_raw_handle())
        .with_context(|| format!("sandbox entry {}", name.to_string_lossy()))?;
    Ok(handle)
}

fn check_handle(handle: HANDLE) -> Result<fs::BY_HANDLE_FILE_INFORMATION> {
    let mut info = unsafe { std::mem::zeroed::<fs::BY_HANDLE_FILE_INFORMATION>() };
    ensure!(
        unsafe { fs::GetFileInformationByHandle(handle, &mut info) } != 0,
        "inspect sandbox entry: {}",
        std::io::Error::last_os_error()
    );
    ensure!(
        info.dwFileAttributes & fs::FILE_ATTRIBUTE_REPARSE_POINT == 0,
        "refusing reparse point; remove the link and retry setup"
    );
    ensure!(
        info.dwFileAttributes & fs::FILE_ATTRIBUTE_DIRECTORY != 0 || info.nNumberOfLinks == 1,
        "refusing hard-linked sandbox file; remove the extra links and retry setup"
    );
    Ok(info)
}

/// Strict service output admission; replacement itself remains atomic and never
/// opens the destination for writing. A racing replacement cannot redirect a write.
pub(crate) fn check_output(path: &Path) -> Result<()> {
    let parent = open_directory_no_reparse(
        path.parent().context("output parent")?,
        fs::FILE_TRAVERSE | fs::FILE_READ_ATTRIBUTES,
        fs::FILE_SHARE_READ | fs::FILE_SHARE_WRITE,
        DirectoryOpenDisposition::OpenExisting,
    )?;
    match checked_entry(
        parent.as_raw_handle(),
        path.file_name().context("output name")?,
        fs::FILE_READ_ATTRIBUTES,
    ) {
        Ok(_) => Ok(()),
        Err(error)
            if error
                .downcast_ref::<std::io::Error>()
                .is_some_and(|e| e.raw_os_error() == Some(2)) =>
        {
            Ok(())
        }
        Err(error) => Err(error),
    }
}

struct Descriptor(*mut c_void);
impl Descriptor {
    fn read(handle: HANDLE) -> Result<Self> {
        let mut sd = std::ptr::null_mut();
        let status = unsafe {
            GetSecurityInfo(
                handle,
                SE_FILE_OBJECT,
                security::OWNER_SECURITY_INFORMATION
                    | security::GROUP_SECURITY_INFORMATION
                    | security::DACL_SECURITY_INFORMATION,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                &mut sd,
            )
        };
        ensure!(status == 0, "read provisioning ACL: {status}");
        Ok(Self(sd))
    }
}
impl Drop for Descriptor {
    fn drop(&mut self) {
        unsafe {
            LocalFree(self.0);
        }
    }
}

struct Inherited(*mut c_void);
impl Drop for Inherited {
    fn drop(&mut self) {
        unsafe {
            security::DestroyPrivateObjectSecurity(&self.0);
        }
    }
}
fn inherit(parent: *const c_void, current: *const c_void, directory: bool) -> Result<Inherited> {
    let mapping = security::GENERIC_MAPPING {
        GenericRead: fs::FILE_GENERIC_READ,
        GenericWrite: fs::FILE_GENERIC_WRITE,
        GenericExecute: fs::FILE_GENERIC_EXECUTE,
        GenericAll: fs::FILE_ALL_ACCESS,
    };
    let mut sd = std::ptr::null_mut();
    ensure!(
        unsafe {
            security::CreatePrivateObjectSecurityEx(
                parent.cast_mut(),
                current.cast_mut(),
                &mut sd,
                std::ptr::null(),
                i32::from(directory),
                security::SEF_DACL_AUTO_INHERIT
                    | security::SEF_AVOID_OWNER_CHECK
                    | security::SEF_AVOID_PRIVILEGE_CHECK,
                std::ptr::null_mut(),
                &mapping,
            )
        } != 0,
        "compute provisioning ACL: {}",
        std::io::Error::last_os_error()
    );
    Ok(Inherited(sd))
}
fn set(handle: HANDLE, descriptor: *mut c_void) -> Result<()> {
    let mut control = 0;
    let mut revision = 0;
    ensure!(
        unsafe { security::GetSecurityDescriptorControl(descriptor, &mut control, &mut revision) }
            != 0,
        "read computed ACL control"
    );
    // The file system requires AUTO_INHERIT_REQ to retain AUTO_INHERITED on a
    // direct write, as in acl::delete_child's checked, non-recursive setter.
    if control & security::SE_DACL_AUTO_INHERITED != 0 {
        ensure!(
            unsafe {
                security::SetSecurityDescriptorControl(
                    descriptor,
                    security::SE_DACL_AUTO_INHERIT_REQ,
                    security::SE_DACL_AUTO_INHERIT_REQ,
                )
            } != 0,
            "retain auto-inherited ACL control"
        );
    }
    // Unlike SetSecurityInfo, this touches only the checked object, never children.
    let status =
        unsafe { NtSetSecurityObject(handle, security::DACL_SECURITY_INFORMATION, descriptor) };
    if status < 0 {
        return Err(std::io::Error::from_raw_os_error(
            unsafe { RtlNtStatusToDosError(status) } as i32,
        ))
        .context("set provisioning ACL");
    }
    Ok(())
}

/// Applies the existing setup ACL policy with explicit, checked handle-relative
/// inheritance. Hard links/reparses are errors, never skipped or followed.
pub(crate) fn set_directory_acl(
    directory: &OwnedHandle,
    path: &Path,
    acl: *mut security::ACL,
    protected: bool,
) -> Result<()> {
    let original = Descriptor::read(directory.as_raw_handle())?;
    let mut control = 0;
    let mut revision = 0;
    ensure!(
        unsafe { security::GetSecurityDescriptorControl(original.0, &mut control, &mut revision) }
            != 0,
        "read provisioning ACL control"
    );
    // DACL_SECURITY_INFORMATION preserves existing protection unless the caller
    // explicitly requests unprotection; setup never requests that.
    let protected = protected || control & security::SE_DACL_PROTECTED != 0;
    let mut owner = std::ptr::null_mut();
    let mut group = std::ptr::null_mut();
    let mut defaulted = 0;
    let mut sd = unsafe { std::mem::zeroed::<security::SECURITY_DESCRIPTOR>() };
    let raw = (&raw mut sd).cast();
    unsafe {
        ensure!(
            security::InitializeSecurityDescriptor(raw, 1) != 0,
            "initialize provisioning ACL"
        );
        ensure!(
            security::GetSecurityDescriptorOwner(original.0, &mut owner, &mut defaulted) != 0,
            "read owner"
        );
        ensure!(
            security::GetSecurityDescriptorGroup(original.0, &mut group, &mut defaulted) != 0,
            "read group"
        );
        ensure!(
            security::SetSecurityDescriptorOwner(raw, owner, 0) != 0,
            "set owner"
        );
        ensure!(
            security::SetSecurityDescriptorGroup(raw, group, 0) != 0,
            "set group"
        );
        ensure!(
            security::SetSecurityDescriptorDacl(raw, 1, acl, 0) != 0,
            "set DACL"
        );
        ensure!(
            security::SetSecurityDescriptorControl(
                raw,
                security::SE_DACL_PROTECTED,
                if protected {
                    security::SE_DACL_PROTECTED
                } else {
                    0
                }
            ) != 0,
            "set DACL control"
        );
    }
    let parent = open_directory_no_reparse(
        path.parent().context("sandbox directory parent")?,
        fs::READ_CONTROL,
        fs::FILE_SHARE_READ | fs::FILE_SHARE_WRITE,
        DirectoryOpenDisposition::OpenExisting,
    )?;
    let parent_sd = Descriptor::read(parent.as_raw_handle())?;
    let root = inherit(parent_sd.0, raw, /*directory*/ true)?;
    set(directory.as_raw_handle(), root.0)?;
    // An empty NT relative name reopens the same directory, without resolving
    // its former pathname (a literal dot is not a valid NT relative name).
    let mut same_directory = [0_u16];
    let listed = open_no_reparse(
        directory.as_raw_handle(),
        &mut same_directory,
        fs::FILE_LIST_DIRECTORY | fs::SYNCHRONIZE,
        fs::FILE_SHARE_READ | fs::FILE_SHARE_WRITE,
        FILE_OPEN,
        FILE_SYNCHRONOUS_IO_NONALERT | 1,
    )?;
    let mut budget = MAX_ENTRIES;
    propagate(
        listed.as_raw_handle(),
        root.0,
        &mut budget,
        /*depth*/ 0,
    )
}

fn propagate(
    directory: HANDLE,
    descriptor: *const c_void,
    budget: &mut usize,
    depth: usize,
) -> Result<()> {
    ensure!(
        depth < 64,
        "sandbox directories are too deep; clean them before retrying setup"
    );
    let (entries, truncated) = unsafe { crate::acl::walk::list_at_most(directory, *budget) }?;
    ensure!(
        !truncated,
        "too many sandbox entries; clean the sandbox directories before retrying setup"
    );
    *budget -= entries.len();
    for (name, attributes) in entries {
        let name = std::ffi::OsString::from_wide(&name);
        let is_directory = attributes.is_dir();
        let access = fs::READ_CONTROL
            | fs::WRITE_DAC
            | if is_directory {
                fs::FILE_LIST_DIRECTORY | fs::SYNCHRONIZE
            } else {
                0
            };
        let entry = checked_entry(directory, &name, access)?;
        let info = check_handle(entry.as_raw_handle())?;
        ensure!(
            (info.dwFileAttributes & fs::FILE_ATTRIBUTE_DIRECTORY != 0) == is_directory,
            "sandbox entry changed type; retry setup"
        );
        let current = Descriptor::read(entry.as_raw_handle())?;
        let inherited = inherit(descriptor, current.0, is_directory)?;
        check_handle(entry.as_raw_handle())?;
        set(entry.as_raw_handle(), inherited.0)?;
        if is_directory {
            // Use a synchronous handle for directory enumeration, still relative
            // to the same checked parent. Pin its identity across the second open.
            let mut wide: Vec<u16> = name.encode_wide().chain(Some(0)).collect();
            let listed = open_no_reparse(
                directory,
                &mut wide,
                access | fs::FILE_READ_ATTRIBUTES,
                fs::FILE_SHARE_READ | fs::FILE_SHARE_WRITE,
                FILE_OPEN,
                FILE_SYNCHRONOUS_IO_NONALERT | 1,
            )?;
            let listed_info = check_handle(listed.as_raw_handle())?;
            ensure!(
                (
                    info.dwVolumeSerialNumber,
                    info.nFileIndexHigh,
                    info.nFileIndexLow
                ) == (
                    listed_info.dwVolumeSerialNumber,
                    listed_info.nFileIndexHigh,
                    listed_info.nFileIndexLow
                ),
                "sandbox directory changed; retry setup"
            );
            propagate(listed.as_raw_handle(), inherited.0, budget, depth + 1)?;
        }
    }
    Ok(())
}
