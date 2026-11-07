//! Protect Windows socket rendezvous directories with an inheritable user-only
//! DACL at creation time. Reject unsafe existing directories instead of repairing them.

use std::io;
use std::os::windows::ffi::OsStrExt;
use std::os::windows::io::FromRawHandle;
use std::os::windows::io::OwnedHandle;
use std::path::Path;
use std::ptr;

use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE;
use windows_sys::Win32::Foundation::LocalFree;
use windows_sys::Win32::Security::ACCESS_ALLOWED_ACE;
use windows_sys::Win32::Security::ACE_HEADER;
use windows_sys::Win32::Security::Authorization::ConvertSidToStringSidW;
use windows_sys::Win32::Security::Authorization::ConvertStringSecurityDescriptorToSecurityDescriptorW;
use windows_sys::Win32::Security::Authorization::GetSecurityInfo;
use windows_sys::Win32::Security::Authorization::SE_FILE_OBJECT;
use windows_sys::Win32::Security::Authorization::SetSecurityInfo;
use windows_sys::Win32::Security::DACL_SECURITY_INFORMATION;
use windows_sys::Win32::Security::EqualSid;
use windows_sys::Win32::Security::GetAce;
use windows_sys::Win32::Security::GetSecurityDescriptorControl;
use windows_sys::Win32::Security::GetSecurityDescriptorDacl;
use windows_sys::Win32::Security::GetTokenInformation;
use windows_sys::Win32::Security::IsWellKnownSid;
use windows_sys::Win32::Security::OWNER_SECURITY_INFORMATION;
use windows_sys::Win32::Security::PROTECTED_DACL_SECURITY_INFORMATION;
use windows_sys::Win32::Security::PSECURITY_DESCRIPTOR;
use windows_sys::Win32::Security::PSID;
use windows_sys::Win32::Security::SE_DACL_PROTECTED;
use windows_sys::Win32::Security::SECURITY_ATTRIBUTES;
use windows_sys::Win32::Security::TOKEN_QUERY;
use windows_sys::Win32::Security::TOKEN_USER;
use windows_sys::Win32::Security::TokenUser;
use windows_sys::Win32::Security::WinLocalSystemSid;
use windows_sys::Win32::Storage::FileSystem::BY_HANDLE_FILE_INFORMATION;
use windows_sys::Win32::Storage::FileSystem::CreateDirectoryW;
use windows_sys::Win32::Storage::FileSystem::CreateFileW;
use windows_sys::Win32::Storage::FileSystem::FILE_ALL_ACCESS;
use windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_DIRECTORY;
use windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_REPARSE_POINT;
use windows_sys::Win32::Storage::FileSystem::FILE_FLAG_BACKUP_SEMANTICS;
use windows_sys::Win32::Storage::FileSystem::FILE_FLAG_OPEN_REPARSE_POINT;
use windows_sys::Win32::Storage::FileSystem::FILE_LIST_DIRECTORY;
use windows_sys::Win32::Storage::FileSystem::FILE_READ_ATTRIBUTES;
use windows_sys::Win32::Storage::FileSystem::FILE_SHARE_READ;
use windows_sys::Win32::Storage::FileSystem::FILE_SHARE_WRITE;
use windows_sys::Win32::Storage::FileSystem::GetFileInformationByHandle;
use windows_sys::Win32::Storage::FileSystem::OPEN_EXISTING;
use windows_sys::Win32::Storage::FileSystem::READ_CONTROL;
use windows_sys::Win32::Storage::FileSystem::WRITE_DAC;
use windows_sys::Win32::System::SystemServices::ACCESS_ALLOWED_ACE_TYPE;
use windows_sys::Win32::System::Threading::GetCurrentProcess;
use windows_sys::Win32::System::Threading::OpenProcessToken;

pub(super) fn prepare_private_directory(path: &Path) -> io::Result<()> {
    let user = current_user()?;
    let user_sid = unsafe { (*(user.as_ptr().cast::<TOKEN_USER>())).User.Sid };
    let mut sid_string = ptr::null_mut();
    if unsafe { ConvertSidToStringSidW(user_sid, &mut sid_string) } == 0 {
        return Err(io::Error::last_os_error());
    }
    let mut sid_length = 0;
    while unsafe { *sid_string.add(sid_length) } != 0 {
        sid_length += 1;
    }
    let sid =
        String::from_utf16_lossy(unsafe { std::slice::from_raw_parts(sid_string, sid_length) });
    unsafe { LocalFree(sid_string as _) };
    let sddl: Vec<u16> = format!("O:{sid}D:P(A;OICI;FA;;;{sid})")
        .encode_utf16()
        .chain(Some(0))
        .collect();
    let mut descriptor = ptr::null_mut();
    if unsafe {
        ConvertStringSecurityDescriptorToSecurityDescriptorW(
            sddl.as_ptr(),
            /*stringsdrevision*/ 1,
            &mut descriptor,
            ptr::null_mut(),
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    let result = (|| {
        // Canonicalize only the parent: this supplies Win32's extended-length
        // prefix without following a reparse point at the directory being secured.
        let absolute = std::path::absolute(path)?;
        let path = match (absolute.parent(), absolute.file_name()) {
            (Some(parent), Some(name)) => {
                std::fs::create_dir_all(parent)?;
                std::fs::canonicalize(parent)?.join(name)
            }
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "private directory must not be a filesystem root",
                ));
            }
        };
        let path_wide: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
        let attributes = SECURITY_ATTRIBUTES {
            nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: descriptor,
            bInheritHandle: 0,
        };
        if unsafe { CreateDirectoryW(path_wide.as_ptr(), &attributes) } != 0 {
            return Ok(());
        }
        let err = io::Error::last_os_error();
        if err.kind() != io::ErrorKind::AlreadyExists {
            return Err(err);
        }
        // Changing a DACL cannot revoke handles already held by another user.
        // Existing directories must already meet our private-directory contract.
        match crate::windows_socket_validation::validate_private_directory(&path) {
            Ok(_guard) => Ok(()),
            Err(err) if err.kind() == io::ErrorKind::PermissionDenied => {
                // Corbanu releases before the rust-v0.167.0 sync left their own layout.
                if !upgrade_legacy_private_directory(&path, user_sid, descriptor)? {
                    return Err(err);
                }
                let _guard = crate::windows_socket_validation::validate_private_directory(&path)?;
                Ok(())
            }
            Err(err) => Err(err),
        }
    })();
    unsafe { LocalFree(descriptor as _) };
    result
}

/// Moves a socket directory written by a Corbanu release before the rust-v0.167.0 sync to the
/// user-only DACL in `descriptor`. Those releases created the directory and then set a protected
/// DACL granting only LocalSystem and the current user (no inheritance), with the user as owner.
/// No other account could open it, so changing it revokes no foreign handle. Any other existing
/// layout is left alone and still rejected. Returns whether the directory was upgraded.
fn upgrade_legacy_private_directory(
    path: &Path,
    user_sid: PSID,
    descriptor: PSECURITY_DESCRIPTOR,
) -> io::Result<bool> {
    let path_wide: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
    // Pin the directory itself (no reparse point, no delete sharing) while it is checked and changed.
    let handle = unsafe {
        CreateFileW(
            path_wide.as_ptr(),
            READ_CONTROL | WRITE_DAC | FILE_READ_ATTRIBUTES | FILE_LIST_DIRECTORY,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            ptr::null(),
            OPEN_EXISTING,
            FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT,
            ptr::null_mut(),
        )
    };
    if handle == INVALID_HANDLE_VALUE {
        return Ok(false);
    }
    let _handle = unsafe { OwnedHandle::from_raw_handle(handle) };
    let mut info: BY_HANDLE_FILE_INFORMATION = unsafe { std::mem::zeroed() };
    if unsafe { GetFileInformationByHandle(handle, &mut info) } == 0
        || info.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY == 0
        || info.dwFileAttributes & FILE_ATTRIBUTE_REPARSE_POINT != 0
    {
        return Ok(false);
    }
    let mut owner = ptr::null_mut();
    let mut dacl = ptr::null_mut();
    let mut current = ptr::null_mut();
    let status = unsafe {
        GetSecurityInfo(
            handle,
            SE_FILE_OBJECT,
            OWNER_SECURITY_INFORMATION | DACL_SECURITY_INFORMATION,
            &mut owner,
            ptr::null_mut(),
            &mut dacl,
            ptr::null_mut(),
            &mut current,
        )
    };
    if status != 0 {
        return Err(io::Error::from_raw_os_error(status as i32));
    }
    let legacy = unsafe {
        let mut control = 0;
        let mut revision = 0;
        let is_full_access_ace = |index: u32, sid_matches: &dyn Fn(PSID) -> bool| {
            let mut ace = ptr::null_mut();
            GetAce(dacl, index, &mut ace) != 0
                && (*ace.cast::<ACE_HEADER>()).AceType == ACCESS_ALLOWED_ACE_TYPE as u8
                && (*ace.cast::<ACE_HEADER>()).AceFlags == 0
                && (*ace.cast::<ACCESS_ALLOWED_ACE>()).Mask == FILE_ALL_ACCESS
                && sid_matches(ptr::addr_of_mut!((*ace.cast::<ACCESS_ALLOWED_ACE>()).SidStart).cast())
        };
        !owner.is_null()
            && EqualSid(owner, user_sid) != 0
            && GetSecurityDescriptorControl(current, &mut control, &mut revision) != 0
            && control & SE_DACL_PROTECTED != 0
            && !dacl.is_null()
            && (*dacl).AceCount == 2
            && is_full_access_ace(0, &|sid| IsWellKnownSid(sid, WinLocalSystemSid) != 0)
            && is_full_access_ace(1, &|sid| EqualSid(sid, user_sid) != 0)
    };
    unsafe { LocalFree(current as _) };
    if !legacy {
        return Ok(false);
    }
    // The owner is already the current user; only the DACL changes.
    let mut dacl_present = 0;
    let mut dacl_defaulted = 0;
    let mut new_dacl = ptr::null_mut();
    if unsafe {
        GetSecurityDescriptorDacl(
            descriptor,
            &mut dacl_present,
            &mut new_dacl,
            &mut dacl_defaulted,
        )
    } == 0
        || dacl_present == 0
    {
        return Err(io::Error::last_os_error());
    }
    let status = unsafe {
        SetSecurityInfo(
            handle,
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION | PROTECTED_DACL_SECURITY_INFORMATION,
            ptr::null_mut(),
            ptr::null_mut(),
            new_dacl,
            ptr::null(),
        )
    };
    if status != 0 {
        return Err(io::Error::from_raw_os_error(status as i32));
    }
    Ok(true)
}

// Keep the pointer-aligned buffer alive while using its TOKEN_USER SID.
pub(super) fn current_user() -> io::Result<Vec<usize>> {
    // Token information contains pointers, so allocate pointer-aligned storage.
    let mut token = ptr::null_mut();
    if unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) } == 0 {
        return Err(io::Error::last_os_error());
    }
    let _token = unsafe { OwnedHandle::from_raw_handle(token) };
    token_user(token)
}

pub(super) fn token_user(token: HANDLE) -> io::Result<Vec<usize>> {
    let mut length = 0;
    unsafe {
        GetTokenInformation(
            token,
            TokenUser,
            ptr::null_mut(),
            /*tokeninformationlength*/ 0,
            &mut length,
        )
    };
    let mut user = vec![0usize; (length as usize).div_ceil(std::mem::size_of::<usize>())];
    if unsafe {
        GetTokenInformation(
            token,
            TokenUser,
            user.as_mut_ptr().cast(),
            length,
            &mut length,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    Ok(user)
}
