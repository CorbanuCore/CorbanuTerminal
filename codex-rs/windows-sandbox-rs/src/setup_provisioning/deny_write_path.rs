use anyhow::Context;
use anyhow::Result;
use std::ffi::c_void;
use std::fs::File;
use std::os::windows::ffi::OsStrExt as _;
use std::os::windows::fs::OpenOptionsExt as _;
use std::os::windows::io::AsRawHandle as _;
use std::os::windows::io::FromRawHandle as _;
use std::path::Component;
use std::path::Path;
use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::Foundation::NTSTATUS;
use windows_sys::Win32::Foundation::UNICODE_STRING;
use windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_DIRECTORY;
use windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_TAG_INFO;
use windows_sys::Win32::Storage::FileSystem::FILE_FLAG_BACKUP_SEMANTICS;
use windows_sys::Win32::Storage::FileSystem::FILE_FLAG_OPEN_REPARSE_POINT;
use windows_sys::Win32::Storage::FileSystem::FILE_GENERIC_READ;
use windows_sys::Win32::Storage::FileSystem::FILE_SHARE_READ;
use windows_sys::Win32::Storage::FileSystem::FILE_SHARE_WRITE;
use windows_sys::Win32::Storage::FileSystem::FileAttributeTagInfo;
use windows_sys::Win32::Storage::FileSystem::GetFileInformationByHandleEx;
use windows_sys::Win32::Storage::FileSystem::WRITE_DAC;
use windows_sys::Win32::System::IO::IO_STATUS_BLOCK;
use windows_sys::Win32::System::IO::IO_STATUS_BLOCK_0;

/// #1953: open/create one component relative to its checked parent handle.
/// Attribute-only handles can retag an open folder despite sharing flags;
/// neither creation nor the caller's ACL update may resolve the path again.
/// Keep delete unshared, but permit existing writers. Filter-managed objects
/// (e.g. OneDrive folders) stay supported; name-surrogate links are refused.
pub(super) fn pin(path: &Path) -> Result<Vec<File>> {
    anyhow::ensure!(
        path.is_absolute()
            && !path
                .components()
                .any(|part| matches!(part, Component::ParentDir)),
        "deny-write path must be absolute without parent traversal: {}",
        path.display()
    );
    let mut pins: Vec<File> = Vec::new();
    let ancestors: Vec<_> = path.ancestors().collect();
    for part in ancestors.into_iter().rev() {
        if !part.has_root() {
            continue;
        }
        let access = FILE_GENERIC_READ | if part == path { WRITE_DAC } else { 0 };
        let file = if let Some(parent) = pins.last() {
            let name: Vec<u16> = part
                .file_name()
                .context("missing path component")?
                .encode_wide()
                .collect();
            // Opening an existing file or directory first preserves file carveouts.
            match child(parent, &name, access, Disposition::Open) {
                Ok(file) => file,
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                    child(parent, &name, access, Disposition::CreateDirectory)?
                }
                Err(err) => return Err(err).with_context(|| format!("open {}", part.display())),
            }
        } else {
            std::fs::OpenOptions::new()
                .access_mode(access)
                .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
                .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
                .open(part)
                .with_context(|| format!("open volume root {}", part.display()))?
        };
        let info = attributes(&file)?;
        anyhow::ensure!(
            info.ReparseTag & 0x2000_0000 == 0,
            "refusing a link in deny-write path: {}",
            part.display()
        );
        anyhow::ensure!(
            part == path || info.FileAttributes & FILE_ATTRIBUTE_DIRECTORY != 0,
            "not a directory: {}",
            part.display()
        );
        pins.push(file);
        #[cfg(test)]
        AFTER_PIN.with(|hook| {
            if let Some(hook) = hook.borrow_mut().as_mut() {
                hook(part);
            }
        });
    }
    Ok(pins)
}

fn attributes(file: &File) -> std::io::Result<FILE_ATTRIBUTE_TAG_INFO> {
    let mut info: FILE_ATTRIBUTE_TAG_INFO = unsafe { std::mem::zeroed() };
    // SAFETY: a live handle and a correctly sized output buffer.
    if unsafe {
        GetFileInformationByHandleEx(
            file.as_raw_handle() as _,
            FileAttributeTagInfo,
            std::ptr::from_mut(&mut info).cast(),
            std::mem::size_of::<FILE_ATTRIBUTE_TAG_INFO>() as u32,
        )
    } == 0
    {
        return Err(std::io::Error::last_os_error());
    }
    Ok(info)
}

// Sync: windows-sys's UNICODE_STRING and IO_STATUS_BLOCK, so this declaration
// matches upstream's NtCreateFile in no_reparse_dir.rs (clashing_extern_declarations).
#[repr(C)]
struct ObjectAttributes {
    length: u32,
    root_directory: HANDLE,
    object_name: *const UNICODE_STRING,
    attributes: u32,
    security_descriptor: *const c_void,
    security_quality_of_service: *const c_void,
}
#[link(name = "ntdll")]
unsafe extern "system" {
    fn NtCreateFile(
        file_handle: *mut HANDLE,
        desired_access: u32,
        object_attributes: *const ObjectAttributes,
        io_status: *mut IO_STATUS_BLOCK,
        allocation_size: *const i64,
        file_attributes: u32,
        share_access: u32,
        disposition: u32,
        options: u32,
        ea_buffer: *const c_void,
        ea_length: u32,
    ) -> NTSTATUS;
    fn RtlNtStatusToDosError(status: i32) -> u32;
}
enum Disposition {
    Open,
    CreateDirectory,
}
fn child(
    parent: &File,
    name: &[u16],
    access: u32,
    disposition: Disposition,
) -> std::io::Result<File> {
    if name.is_empty() || name.iter().any(|ch| [0, 47, 58, 92].contains(ch)) {
        return Err(std::io::Error::other("not a single path component"));
    }
    let bytes = u16::try_from(std::mem::size_of_val(name)).map_err(std::io::Error::other)?;
    let mut name = name.to_vec();
    let unicode = UNICODE_STRING {
        Length: bytes,
        MaximumLength: bytes,
        Buffer: name.as_mut_ptr(),
    };
    let attributes = ObjectAttributes {
        length: std::mem::size_of::<ObjectAttributes>() as u32,
        root_directory: parent.as_raw_handle() as _,
        object_name: &unicode,
        // OBJ_CASE_INSENSITIVE | OBJ_DONT_REPARSE.
        attributes: 0x40 | 0x1000,
        security_descriptor: std::ptr::null(),
        security_quality_of_service: std::ptr::null(),
    };
    let (disposition, directory) = match disposition {
        Disposition::Open => (1, 0),            // FILE_OPEN
        Disposition::CreateDirectory => (3, 1), // FILE_OPEN_IF | FILE_DIRECTORY_FILE
    };
    let mut io = IO_STATUS_BLOCK {
        Anonymous: IO_STATUS_BLOCK_0 { Status: 0 },
        Information: 0,
    };
    // Upstream's windows-sys makes HANDLE a pointer (main's is an integer).
    let mut handle: HANDLE = std::ptr::null_mut();
    // SAFETY: ABI matches the NT API; all pointed-to buffers live through the
    // call. The returned handle is owned by File. The name is one component.
    let status = unsafe {
        NtCreateFile(
            &mut handle,
            access,
            &attributes,
            &mut io,
            std::ptr::null(),
            /*file_attributes*/ 0,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            disposition,
            // FILE_OPEN_REPARSE_POINT | FILE_OPEN_FOR_BACKUP_INTENT |
            // FILE_SYNCHRONOUS_IO_NONALERT, plus FILE_DIRECTORY_FILE if creating.
            0x0020_0000 | 0x4000 | 0x20 | directory,
            std::ptr::null(),
            /*ea_length*/ 0,
        )
    };
    if status < 0 {
        return Err(std::io::Error::from_raw_os_error(
            unsafe { RtlNtStatusToDosError(status) } as i32,
        ));
    }
    Ok(unsafe { File::from_raw_handle(handle as _) })
}

#[cfg(test)]
type PinHook = Box<dyn FnMut(&Path)>;

#[cfg(test)]
thread_local! {
    static AFTER_PIN: std::cell::RefCell<Option<PinHook>> = const { std::cell::RefCell::new(None) };
}

#[cfg(test)]
#[path = "deny_write_path_tests.rs"]
mod tests;
