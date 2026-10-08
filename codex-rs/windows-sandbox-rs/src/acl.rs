use crate::winutil::to_wide;
use anyhow::Result;
use anyhow::anyhow;
use std::ffi::c_void;
use std::path::Path;
use windows_sys::Win32::Foundation::CloseHandle;
use windows_sys::Win32::Foundation::ERROR_SUCCESS;
use windows_sys::Win32::Foundation::GetLastError;
use windows_sys::Win32::Foundation::HLOCAL;
use windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE;
use windows_sys::Win32::Foundation::LocalFree;
use windows_sys::Win32::Security::ACCESS_ALLOWED_ACE;
use windows_sys::Win32::Security::ACCESS_DENIED_ACE;
use windows_sys::Win32::Security::ACE_HEADER;
use windows_sys::Win32::Security::ACL;
use windows_sys::Win32::Security::ACL_SIZE_INFORMATION;
use windows_sys::Win32::Security::AclSizeInformation;
use windows_sys::Win32::Security::AddAce;
use windows_sys::Win32::Security::Authorization::EXPLICIT_ACCESS_W;
use windows_sys::Win32::Security::Authorization::GetNamedSecurityInfoW;
use windows_sys::Win32::Security::Authorization::GetSecurityInfo;
use windows_sys::Win32::Security::Authorization::SetEntriesInAclW;
use windows_sys::Win32::Security::Authorization::SetNamedSecurityInfoW;
use windows_sys::Win32::Security::Authorization::SetSecurityInfo;
use windows_sys::Win32::Security::Authorization::TRUSTEE_IS_SID;
use windows_sys::Win32::Security::Authorization::TRUSTEE_IS_UNKNOWN;
use windows_sys::Win32::Security::Authorization::TRUSTEE_W;
use windows_sys::Win32::Security::DACL_SECURITY_INFORMATION;
use windows_sys::Win32::Security::EqualSid;
use windows_sys::Win32::Security::GENERIC_MAPPING;
use windows_sys::Win32::Security::GetAce;
use windows_sys::Win32::Security::GetAclInformation;
use windows_sys::Win32::Security::GetSecurityDescriptorControl;
use windows_sys::Win32::Security::InitializeAcl;
use windows_sys::Win32::Security::MapGenericMask;
use windows_sys::Win32::Security::PROTECTED_DACL_SECURITY_INFORMATION;
use windows_sys::Win32::Security::SE_DACL_PROTECTED;
use windows_sys::Win32::Security::UNPROTECTED_DACL_SECURITY_INFORMATION;
use windows_sys::Win32::Storage::FileSystem::CreateFileW;
use windows_sys::Win32::Storage::FileSystem::DELETE;
use windows_sys::Win32::Storage::FileSystem::FILE_ALL_ACCESS;
use windows_sys::Win32::Storage::FileSystem::FILE_APPEND_DATA;
use windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_NORMAL;
use windows_sys::Win32::Storage::FileSystem::FILE_DELETE_CHILD;
use windows_sys::Win32::Storage::FileSystem::FILE_FLAG_BACKUP_SEMANTICS;
use windows_sys::Win32::Storage::FileSystem::FILE_GENERIC_EXECUTE;
use windows_sys::Win32::Storage::FileSystem::FILE_GENERIC_READ;
use windows_sys::Win32::Storage::FileSystem::FILE_GENERIC_WRITE;
use windows_sys::Win32::Storage::FileSystem::FILE_SHARE_DELETE;
use windows_sys::Win32::Storage::FileSystem::FILE_SHARE_READ;
use windows_sys::Win32::Storage::FileSystem::FILE_SHARE_WRITE;
use windows_sys::Win32::Storage::FileSystem::FILE_WRITE_ATTRIBUTES;
use windows_sys::Win32::Storage::FileSystem::FILE_WRITE_DATA;
use windows_sys::Win32::Storage::FileSystem::FILE_WRITE_EA;
use windows_sys::Win32::Storage::FileSystem::OPEN_EXISTING;
use windows_sys::Win32::Storage::FileSystem::READ_CONTROL;
const SE_KERNEL_OBJECT: u32 = 6;
const INHERIT_ONLY_ACE: u8 = 0x08;
const INHERITED_ACE: u8 = 0x10;
const ACCESS_ALLOWED_ACE_TYPE: u8 = 0;
const ACCESS_DENIED_ACE_TYPE: u8 = 1;
const GENERIC_READ_MASK: u32 = 0x8000_0000;
const GENERIC_WRITE_MASK: u32 = 0x4000_0000;
const DENY_ACCESS: i32 = 3;

/// Fetch DACL via handle-based query; caller must LocalFree the returned SD.
///
/// # Safety
/// Caller must free the returned security descriptor with `LocalFree` and pass an existing path.
pub unsafe fn fetch_dacl_handle(path: &Path) -> Result<(*mut ACL, *mut c_void)> {
    let wpath = to_wide(path);
    let h = CreateFileW(
        wpath.as_ptr(),
        READ_CONTROL,
        FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
        std::ptr::null_mut(),
        OPEN_EXISTING,
        FILE_FLAG_BACKUP_SEMANTICS,
        0,
    );
    if h == INVALID_HANDLE_VALUE {
        return Err(anyhow!("CreateFileW failed for {}", path.display()));
    }
    let mut p_sd: *mut c_void = std::ptr::null_mut();
    let mut p_dacl: *mut ACL = std::ptr::null_mut();
    let code = GetSecurityInfo(
        h,
        1, // SE_FILE_OBJECT
        DACL_SECURITY_INFORMATION,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        &mut p_dacl,
        std::ptr::null_mut(),
        &mut p_sd,
    );
    CloseHandle(h);
    if code != ERROR_SUCCESS {
        return Err(anyhow!(
            "GetSecurityInfo failed for {}: {}",
            path.display(),
            code
        ));
    }
    Ok((p_dacl, p_sd))
}

/// Fast mask-based check: does an ACE for provided SIDs grant the desired mask? Skips inherit-only.
/// When `require_all_bits` is true, all bits in `desired_mask` must be present; otherwise any bit suffices.
pub unsafe fn dacl_mask_allows(
    p_dacl: *mut ACL,
    psids: &[*mut c_void],
    desired_mask: u32,
    require_all_bits: bool,
) -> bool {
    dacl_mask_allows_with_scope(
        p_dacl,
        psids,
        desired_mask,
        require_all_bits,
        AceScope::Effective,
    )
}

#[derive(Clone, Copy)]
enum AceScope {
    Effective,
    Explicit,
}

unsafe fn dacl_mask_allows_with_scope(
    p_dacl: *mut ACL,
    psids: &[*mut c_void],
    desired_mask: u32,
    require_all_bits: bool,
    scope: AceScope,
) -> bool {
    if p_dacl.is_null() {
        return false;
    }
    let mut info: ACL_SIZE_INFORMATION = std::mem::zeroed();
    let ok = GetAclInformation(
        p_dacl as *const ACL,
        &mut info as *mut _ as *mut c_void,
        std::mem::size_of::<ACL_SIZE_INFORMATION>() as u32,
        AclSizeInformation,
    );
    if ok == 0 {
        return false;
    }
    let mapping = GENERIC_MAPPING {
        GenericRead: FILE_GENERIC_READ,
        GenericWrite: FILE_GENERIC_WRITE,
        GenericExecute: FILE_GENERIC_EXECUTE,
        GenericAll: FILE_ALL_ACCESS,
    };
    for i in 0..(info.AceCount as usize) {
        let mut p_ace: *mut c_void = std::ptr::null_mut();
        if GetAce(p_dacl as *const ACL, i as u32, &mut p_ace) == 0 {
            continue;
        }
        let hdr = &*(p_ace as *const ACE_HEADER);
        if hdr.AceType != ACCESS_ALLOWED_ACE_TYPE {
            continue; // not ACCESS_ALLOWED
        }
        if (hdr.AceFlags & INHERIT_ONLY_ACE) != 0 {
            continue;
        }
        // SET_ACCESS cannot replace an ACE inherited from an ancestor, so it cannot make
        // an explicit-only repair converge when that inherited ACE contains stale rights.
        if matches!(scope, AceScope::Explicit) && (hdr.AceFlags & INHERITED_ACE) != 0 {
            continue;
        }
        let base = p_ace as usize;
        let sid_ptr =
            (base + std::mem::size_of::<ACE_HEADER>() + std::mem::size_of::<u32>()) as *mut c_void;
        let mut matched = false;
        for sid in psids {
            if EqualSid(sid_ptr, *sid) != 0 {
                matched = true;
                break;
            }
        }
        if !matched {
            continue;
        }
        let ace = &*(p_ace as *const ACCESS_ALLOWED_ACE);
        let mut mask = ace.Mask;
        MapGenericMask(&mut mask, &mapping);
        if (require_all_bits && (mask & desired_mask) == desired_mask)
            || (!require_all_bits && (mask & desired_mask) != 0)
        {
            return true;
        }
    }
    false
}

/// Path-based wrapper around the mask check (single DACL fetch).
pub fn path_mask_allows(
    path: &Path,
    psids: &[*mut c_void],
    desired_mask: u32,
    require_all_bits: bool,
) -> Result<bool> {
    path_mask_allows_with_scope(
        path,
        psids,
        desired_mask,
        require_all_bits,
        AceScope::Effective,
    )
}

fn path_mask_allows_with_scope(
    path: &Path,
    psids: &[*mut c_void],
    desired_mask: u32,
    require_all_bits: bool,
    scope: AceScope,
) -> Result<bool> {
    unsafe {
        let (p_dacl, sd) = fetch_dacl_handle(path)?;
        let has = dacl_mask_allows_with_scope(p_dacl, psids, desired_mask, require_all_bits, scope);
        if !sd.is_null() {
            LocalFree(sd as HLOCAL);
        }
        Ok(has)
    }
}

pub unsafe fn dacl_has_write_allow_for_sid(p_dacl: *mut ACL, psid: *mut c_void) -> bool {
    if p_dacl.is_null() {
        return false;
    }
    let mut info: ACL_SIZE_INFORMATION = std::mem::zeroed();
    let ok = GetAclInformation(
        p_dacl as *const ACL,
        &mut info as *mut _ as *mut c_void,
        std::mem::size_of::<ACL_SIZE_INFORMATION>() as u32,
        AclSizeInformation,
    );
    if ok == 0 {
        return false;
    }
    let count = info.AceCount as usize;
    for i in 0..count {
        let mut p_ace: *mut c_void = std::ptr::null_mut();
        if GetAce(p_dacl as *const ACL, i as u32, &mut p_ace) == 0 {
            continue;
        }
        let hdr = &*(p_ace as *const ACE_HEADER);
        if hdr.AceType != ACCESS_ALLOWED_ACE_TYPE {
            continue; // ACCESS_ALLOWED_ACE_TYPE
        }
        // Ignore ACEs that are inherit-only (do not apply to the current object)
        if (hdr.AceFlags & INHERIT_ONLY_ACE) != 0 {
            continue;
        }
        let ace = &*(p_ace as *const ACCESS_ALLOWED_ACE);
        let mask = ace.Mask;
        let base = p_ace as usize;
        let sid_ptr =
            (base + std::mem::size_of::<ACE_HEADER>() + std::mem::size_of::<u32>()) as *mut c_void;
        let eq = EqualSid(sid_ptr, psid);
        if eq != 0 && (mask & FILE_GENERIC_WRITE) != 0 {
            return true;
        }
    }
    false
}

pub unsafe fn dacl_has_write_deny_for_sid(p_dacl: *mut ACL, psid: *mut c_void) -> bool {
    if p_dacl.is_null() {
        return false;
    }
    let mut info: ACL_SIZE_INFORMATION = std::mem::zeroed();
    let ok = GetAclInformation(
        p_dacl as *const ACL,
        &mut info as *mut _ as *mut c_void,
        std::mem::size_of::<ACL_SIZE_INFORMATION>() as u32,
        AclSizeInformation,
    );
    if ok == 0 {
        return false;
    }
    let deny_write_mask = FILE_GENERIC_WRITE
        | FILE_WRITE_DATA
        | FILE_APPEND_DATA
        | FILE_WRITE_EA
        | FILE_WRITE_ATTRIBUTES
        | GENERIC_WRITE_MASK
        | DELETE
        | FILE_DELETE_CHILD;
    for i in 0..info.AceCount {
        let mut p_ace: *mut c_void = std::ptr::null_mut();
        if GetAce(p_dacl as *const ACL, i, &mut p_ace) == 0 {
            continue;
        }
        let hdr = &*(p_ace as *const ACE_HEADER);
        if hdr.AceType != ACCESS_DENIED_ACE_TYPE {
            continue; // ACCESS_DENIED_ACE_TYPE
        }
        if (hdr.AceFlags & INHERIT_ONLY_ACE) != 0 {
            continue;
        }
        let ace = &*(p_ace as *const ACCESS_DENIED_ACE);
        let base = p_ace as usize;
        let sid_ptr =
            (base + std::mem::size_of::<ACE_HEADER>() + std::mem::size_of::<u32>()) as *mut c_void;
        if EqualSid(sid_ptr, psid) != 0 && (ace.Mask & deny_write_mask) != 0 {
            return true;
        }
    }
    false
}

pub unsafe fn dacl_has_read_deny_for_sid(p_dacl: *mut ACL, psid: *mut c_void) -> bool {
    if p_dacl.is_null() {
        return false;
    }
    let mut info: ACL_SIZE_INFORMATION = std::mem::zeroed();
    let ok = GetAclInformation(
        p_dacl as *const ACL,
        &mut info as *mut _ as *mut c_void,
        std::mem::size_of::<ACL_SIZE_INFORMATION>() as u32,
        AclSizeInformation,
    );
    if ok == 0 {
        return false;
    }
    let deny_read_mask = FILE_GENERIC_READ | GENERIC_READ_MASK;
    for i in 0..info.AceCount {
        let mut p_ace: *mut c_void = std::ptr::null_mut();
        if GetAce(p_dacl as *const ACL, i, &mut p_ace) == 0 {
            continue;
        }
        let hdr = &*(p_ace as *const ACE_HEADER);
        if hdr.AceType != ACCESS_DENIED_ACE_TYPE {
            continue; // ACCESS_DENIED_ACE_TYPE
        }
        if (hdr.AceFlags & INHERIT_ONLY_ACE) != 0 {
            continue;
        }
        let ace = &*(p_ace as *const ACCESS_DENIED_ACE);
        let base = p_ace as usize;
        let sid_ptr =
            (base + std::mem::size_of::<ACE_HEADER>() + std::mem::size_of::<u32>()) as *mut c_void;
        if EqualSid(sid_ptr, psid) != 0 && (ace.Mask & deny_read_mask) != 0 {
            return true;
        }
    }
    false
}

/// True when the DACL has an explicit (not inherited) entry denying `psid`
/// read access to the object itself.
unsafe fn dacl_has_explicit_read_deny_for_sid(p_dacl: *mut ACL, psid: *mut c_void) -> bool {
    if p_dacl.is_null() {
        return false;
    }
    let mut info: ACL_SIZE_INFORMATION = std::mem::zeroed();
    if GetAclInformation(
        p_dacl as *const ACL,
        &mut info as *mut _ as *mut c_void,
        std::mem::size_of::<ACL_SIZE_INFORMATION>() as u32,
        AclSizeInformation,
    ) == 0
    {
        return false;
    }
    for i in 0..info.AceCount {
        let mut p_ace: *mut c_void = std::ptr::null_mut();
        if GetAce(p_dacl as *const ACL, i, &mut p_ace) == 0 {
            continue;
        }
        let hdr = &*(p_ace as *const ACE_HEADER);
        if hdr.AceType != ACCESS_DENIED_ACE_TYPE
            || (hdr.AceFlags & (INHERIT_ONLY_ACE | INHERITED_ACE)) != 0
        {
            continue;
        }
        let ace = &*(p_ace as *const ACCESS_DENIED_ACE);
        let sid = (p_ace as usize + std::mem::size_of::<ACE_HEADER>() + std::mem::size_of::<u32>())
            as *mut c_void;
        if EqualSid(sid, psid) != 0 && (ace.Mask & FILE_GENERIC_READ) == FILE_GENERIC_READ {
            return true;
        }
    }
    false
}

/// PF-27-S07: makes sure `path` has its own explicit read deny for `psid`
/// (an inherited one does not count). Returns whether it is present after.
///
/// # Safety
/// Caller must ensure `psid` points to a valid SID and `path` exists.
pub unsafe fn ensure_explicit_deny_read_ace(path: &Path, psid: *mut c_void) -> Result<bool> {
    add_deny_ace(path, psid, DenyAceKind::ReadExplicit)?;
    has_explicit_deny_read_ace(path, psid)
}

/// PF-27-S07: whether `path` has its own explicit read deny for `psid`.
///
/// # Safety
/// Caller must ensure `psid` points to a valid SID and `path` exists.
pub unsafe fn has_explicit_deny_read_ace(path: &Path, psid: *mut c_void) -> Result<bool> {
    let (p_dacl, p_sd) = fetch_dacl_handle(path)?;
    let present = dacl_has_explicit_read_deny_for_sid(p_dacl, psid);
    if !p_sd.is_null() {
        LocalFree(p_sd as HLOCAL);
    }
    Ok(present)
}

/// PF-27-S07: the number of hard links to an open file.
pub fn file_link_count(file: &std::fs::File) -> std::io::Result<u32> {
    use std::os::windows::io::AsRawHandle as _;
    use windows_sys::Win32::Storage::FileSystem::BY_HANDLE_FILE_INFORMATION;
    use windows_sys::Win32::Storage::FileSystem::GetFileInformationByHandle;
    // SAFETY: zeroed out-structure; the handle is open for the call.
    let mut info: BY_HANDLE_FILE_INFORMATION = unsafe { std::mem::zeroed() };
    if unsafe { GetFileInformationByHandle(file.as_raw_handle() as _, &mut info) } == 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(info.nNumberOfLinks)
}

/// True when the DACL already has an inherit-only, files-only read deny for
/// `psid` (see [`add_deny_read_ace_for_new_files`]).
unsafe fn dacl_has_new_file_read_deny_for_sid(p_dacl: *mut ACL, psid: *mut c_void) -> bool {
    if p_dacl.is_null() {
        return false;
    }
    let mut info: ACL_SIZE_INFORMATION = std::mem::zeroed();
    let ok = GetAclInformation(
        p_dacl as *const ACL,
        &mut info as *mut _ as *mut c_void,
        std::mem::size_of::<ACL_SIZE_INFORMATION>() as u32,
        AclSizeInformation,
    );
    if ok == 0 {
        return false;
    }
    let wanted = OBJECT_INHERIT_ACE | u32::from(INHERIT_ONLY_ACE) | NO_PROPAGATE_INHERIT_ACE;
    for i in 0..info.AceCount {
        let mut p_ace: *mut c_void = std::ptr::null_mut();
        if GetAce(p_dacl as *const ACL, i, &mut p_ace) == 0 {
            continue;
        }
        let hdr = &*(p_ace as *const ACE_HEADER);
        if hdr.AceType != ACCESS_DENIED_ACE_TYPE || (u32::from(hdr.AceFlags) & wanted) != wanted {
            continue;
        }
        let ace = &*(p_ace as *const ACCESS_DENIED_ACE);
        let base = p_ace as usize;
        let sid_ptr =
            (base + std::mem::size_of::<ACE_HEADER>() + std::mem::size_of::<u32>()) as *mut c_void;
        if EqualSid(sid_ptr, psid) != 0 && (ace.Mask & FILE_GENERIC_READ) != 0 {
            return true;
        }
    }
    false
}

// Grant DELETE on each inheriting descendant instead of FILE_DELETE_CHILD on
// its parent. A parent delete-child grant would bypass a direct deny-write ACE
// on protected children such as `.git` or an explicit read-only subpath.
// WRITE_RESTRICTED tokens do not restrict FILE_DELETE_CHILD itself, so a parent
// whose DACL grants it to the user still opens that route; see
// `add_deny_delete_child_ace` and #158.
const WRITE_ALLOW_MASK: u32 =
    FILE_GENERIC_READ | FILE_GENERIC_WRITE | FILE_GENERIC_EXECUTE | DELETE;

unsafe fn dacl_allow_mask_needs_refresh(
    p_dacl: *mut ACL,
    psid: *mut c_void,
    allow_mask: u32,
    disallow_mask: u32,
) -> bool {
    !dacl_mask_allows(p_dacl, &[psid], allow_mask, /*require_all_bits*/ true)
        || dacl_mask_allows_with_scope(
            p_dacl,
            &[psid],
            disallow_mask,
            /*require_all_bits*/ false,
            AceScope::Explicit,
        )
}

/// Returns whether any provided SID needs its writable-root allow ACE refreshed.
pub fn path_write_aces_need_refresh(path: &Path, psids: &[*mut c_void]) -> Result<bool> {
    unsafe {
        let (p_dacl, p_sd) = fetch_dacl_handle(path)?;
        let needs_refresh = psids.iter().any(|psid| {
            dacl_allow_mask_needs_refresh(p_dacl, *psid, WRITE_ALLOW_MASK, FILE_DELETE_CHILD)
        });
        if !p_sd.is_null() {
            LocalFree(p_sd as HLOCAL);
        }
        Ok(needs_refresh)
    }
}

unsafe fn ensure_allow_mask_aces_with_inheritance_impl(
    path: &Path,
    sids: &[*mut c_void],
    allow_mask: u32,
    disallow_mask: u32,
    inheritance: u32,
) -> Result<bool> {
    let (p_dacl, p_sd) = fetch_dacl_handle(path)?;
    let mut entries: Vec<EXPLICIT_ACCESS_W> = Vec::new();
    for sid in sids {
        if !dacl_allow_mask_needs_refresh(p_dacl, *sid, allow_mask, disallow_mask) {
            continue;
        }
        entries.push(EXPLICIT_ACCESS_W {
            grfAccessPermissions: allow_mask,
            grfAccessMode: 2, // SET_ACCESS
            grfInheritance: inheritance,
            Trustee: TRUSTEE_W {
                pMultipleTrustee: std::ptr::null_mut(),
                MultipleTrusteeOperation: 0,
                TrusteeForm: TRUSTEE_IS_SID,
                TrusteeType: TRUSTEE_IS_UNKNOWN,
                ptstrName: *sid as *mut u16,
            },
        });
    }
    let mut added = false;
    if !entries.is_empty() {
        let mut p_new_dacl: *mut ACL = std::ptr::null_mut();
        let code2 = SetEntriesInAclW(
            entries.len() as u32,
            entries.as_ptr(),
            p_dacl,
            &mut p_new_dacl,
        );
        if code2 == ERROR_SUCCESS {
            let code3 = SetNamedSecurityInfoW(
                to_wide(path).as_ptr() as *mut u16,
                1,
                DACL_SECURITY_INFORMATION,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                p_new_dacl,
                std::ptr::null_mut(),
            );
            if code3 == ERROR_SUCCESS {
                added = true;
                if !p_new_dacl.is_null() {
                    LocalFree(p_new_dacl as HLOCAL);
                }
            } else {
                if !p_new_dacl.is_null() {
                    LocalFree(p_new_dacl as HLOCAL);
                }
                if !p_sd.is_null() {
                    LocalFree(p_sd as HLOCAL);
                }
                return Err(anyhow!("SetNamedSecurityInfoW failed: {code3}"));
            }
        } else {
            if !p_sd.is_null() {
                LocalFree(p_sd as HLOCAL);
            }
            return Err(anyhow!("SetEntriesInAclW failed: {code2}"));
        }
    }
    if !p_sd.is_null() {
        LocalFree(p_sd as HLOCAL);
    }
    Ok(added)
}

/// Ensure all provided SIDs have an allow ACE with the requested mask on the path.
/// Returns true if any ACE was added.
///
/// # Safety
/// Caller must pass valid SID pointers and an existing path; free the returned security descriptor with `LocalFree`.
pub unsafe fn ensure_allow_mask_aces_with_inheritance(
    path: &Path,
    sids: &[*mut c_void],
    allow_mask: u32,
    inheritance: u32,
) -> Result<bool> {
    ensure_allow_mask_aces_with_inheritance_impl(
        path,
        sids,
        allow_mask,
        /*disallow_mask*/ 0,
        inheritance,
    )
}

/// Ensure all provided SIDs have an allow ACE with the requested mask on the path.
/// Returns true if any ACE was added.
///
/// # Safety
/// Caller must pass valid SID pointers and an existing path; free the returned security descriptor with `LocalFree`.
pub unsafe fn ensure_allow_mask_aces(
    path: &Path,
    sids: &[*mut c_void],
    allow_mask: u32,
) -> Result<bool> {
    ensure_allow_mask_aces_with_inheritance(
        path,
        sids,
        allow_mask,
        CONTAINER_INHERIT_ACE | OBJECT_INHERIT_ACE,
    )
}

/// Ensure all provided SIDs have a write-capable allow ACE on the path.
/// Returns true if any ACE was added.
///
/// # Safety
/// Caller must pass valid SID pointers and an existing path; free the returned security descriptor with `LocalFree`.
pub unsafe fn ensure_allow_write_aces(path: &Path, sids: &[*mut c_void]) -> Result<bool> {
    ensure_allow_mask_aces_with_inheritance_impl(
        path,
        sids,
        WRITE_ALLOW_MASK,
        FILE_DELETE_CHILD,
        CONTAINER_INHERIT_ACE | OBJECT_INHERIT_ACE,
    )
}

/// Adds an allow ACE granting read/write/execute to the given SID on the target path.
///
/// # Safety
/// Caller must ensure `psid` points to a valid SID and `path` refers to an existing file or directory.
pub unsafe fn add_allow_ace(path: &Path, psid: *mut c_void) -> Result<bool> {
    let mut p_sd: *mut c_void = std::ptr::null_mut();
    let mut p_dacl: *mut ACL = std::ptr::null_mut();
    let code = GetNamedSecurityInfoW(
        to_wide(path).as_ptr(),
        1,
        DACL_SECURITY_INFORMATION,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        &mut p_dacl,
        std::ptr::null_mut(),
        &mut p_sd,
    );
    if code != ERROR_SUCCESS {
        return Err(anyhow!("GetNamedSecurityInfoW failed: {code}"));
    }
    // Already has write? Skip costly DACL rewrite.
    if dacl_has_write_allow_for_sid(p_dacl, psid) {
        if !p_sd.is_null() {
            LocalFree(p_sd as HLOCAL);
        }
        return Ok(false);
    }
    let mut added = false;
    // Always ensure write is present: if an allow ACE exists without write, add one with write+RX.
    let trustee = TRUSTEE_W {
        pMultipleTrustee: std::ptr::null_mut(),
        MultipleTrusteeOperation: 0,
        TrusteeForm: TRUSTEE_IS_SID,
        TrusteeType: TRUSTEE_IS_UNKNOWN,
        ptstrName: psid as *mut u16,
    };
    let mut explicit: EXPLICIT_ACCESS_W = std::mem::zeroed();
    explicit.grfAccessPermissions = FILE_GENERIC_READ | FILE_GENERIC_WRITE | FILE_GENERIC_EXECUTE;
    explicit.grfAccessMode = 2; // SET_ACCESS
    explicit.grfInheritance = CONTAINER_INHERIT_ACE | OBJECT_INHERIT_ACE;
    explicit.Trustee = trustee;
    let mut p_new_dacl: *mut ACL = std::ptr::null_mut();
    let code2 = SetEntriesInAclW(1, &explicit, p_dacl, &mut p_new_dacl);
    if code2 == ERROR_SUCCESS {
        let code3 = SetNamedSecurityInfoW(
            to_wide(path).as_ptr() as *mut u16,
            1,
            DACL_SECURITY_INFORMATION,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            p_new_dacl,
            std::ptr::null_mut(),
        );
        if code3 == ERROR_SUCCESS {
            added = !dacl_has_write_allow_for_sid(p_dacl, psid);
        }
        if !p_new_dacl.is_null() {
            LocalFree(p_new_dacl as HLOCAL);
        }
    }
    if !p_sd.is_null() {
        LocalFree(p_sd as HLOCAL);
    }
    Ok(added)
}

/// Adds a deny ACE to prevent write/append/delete for the given SID on the target path.
///
/// # Safety
/// Caller must ensure `psid` points to a valid SID and `path` refers to an existing file or directory.
pub unsafe fn add_deny_write_ace(path: &Path, psid: *mut c_void) -> Result<bool> {
    add_deny_ace(path, psid, DenyAceKind::Write)
}

/// Denies `FILE_DELETE_CHILD` to `psid` on a directory, and on its future and
/// existing subdirectories when `inherit_to_subdirs` is set.
///
/// WRITE_RESTRICTED sandbox tokens do not consult restricting SIDs for
/// `FILE_DELETE_CHILD` (#158). A parent directory that grants it to one of the
/// token's normal SIDs (usually the user, through full control) lets the sandbox
/// delete or move a child even when the child denies the capability SID
/// `DELETE`, so this deny must name a SID from the token's normal groups.
///
/// # Safety
/// Caller must ensure `psid` points to a valid SID and `path` is an existing directory.
pub unsafe fn add_deny_delete_child_ace(
    path: &Path,
    psid: *mut c_void,
    inherit_to_subdirs: bool,
) -> Result<bool> {
    add_deny_ace(path, psid, DenyAceKind::DeleteChild { inherit_to_subdirs })
}

/// PF-27-S06: denies `psid` reads of every file created directly in `path`
/// from now on (inherit-only, files only, not propagated below the first
/// level), so a file replaced by rename does not lose the deny. Existing
/// files get the inherited entry as well; subdirectories are unaffected.
///
/// # Safety
/// Caller must ensure `psid` points to a valid SID and `path` is a directory.
pub unsafe fn add_deny_read_ace_for_new_files(path: &Path, psid: *mut c_void) -> Result<bool> {
    add_deny_ace(path, psid, DenyAceKind::ReadNewFiles)?;
    // `add_deny_ace` reports a failed write as "not added"; confirm instead.
    has_deny_read_ace_for_new_files(path, psid)
}

/// Whether `path` has the entry [`add_deny_read_ace_for_new_files`] adds
/// for `psid`.
///
/// # Safety
/// Caller must ensure `psid` points to a valid SID.
pub unsafe fn has_deny_read_ace_for_new_files(path: &Path, psid: *mut c_void) -> Result<bool> {
    let mut p_sd: *mut c_void = std::ptr::null_mut();
    let mut p_dacl: *mut ACL = std::ptr::null_mut();
    let code = GetNamedSecurityInfoW(
        to_wide(path).as_ptr(),
        1,
        DACL_SECURITY_INFORMATION,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        &mut p_dacl,
        std::ptr::null_mut(),
        &mut p_sd,
    );
    if code != ERROR_SUCCESS {
        return Err(anyhow!("GetNamedSecurityInfoW failed: {code}"));
    }
    let present = dacl_has_new_file_read_deny_for_sid(p_dacl, psid);
    if !p_sd.is_null() {
        LocalFree(p_sd as HLOCAL);
    }
    Ok(present)
}

/// True when `ace` is exactly the entry [`add_deny_read_ace_for_new_files`]
/// adds for `psid` (any SID when `None`): an explicit deny, inherit-only for
/// files one level down, whose mask maps to file read and nothing else.
unsafe fn is_new_file_read_deny(ace: *const c_void, psid: Option<*mut c_void>) -> bool {
    let hdr = &*(ace as *const ACE_HEADER);
    let flags = OBJECT_INHERIT_ACE | u32::from(INHERIT_ONLY_ACE) | NO_PROPAGATE_INHERIT_ACE;
    if hdr.AceType != ACCESS_DENIED_ACE_TYPE || u32::from(hdr.AceFlags) != flags {
        return false;
    }
    let mut mask = (*(ace as *const ACCESS_DENIED_ACE)).Mask;
    let mapping = GENERIC_MAPPING {
        GenericRead: FILE_GENERIC_READ,
        GenericWrite: FILE_GENERIC_WRITE,
        GenericExecute: FILE_GENERIC_EXECUTE,
        GenericAll: FILE_ALL_ACCESS,
    };
    MapGenericMask(&mut mask, &mapping);
    let sid = (ace as usize + std::mem::size_of::<ACE_HEADER>() + std::mem::size_of::<u32>())
        as *mut c_void;
    mask == FILE_GENERIC_READ && psid.is_none_or(|psid| EqualSid(sid, psid) != 0)
}

/// PF-27-S07: whether `path` has exactly the entry
/// [`add_deny_read_ace_for_new_files`] adds, for `psid` or (`None`) for any
/// SID; what [`remove_deny_read_ace_for_new_files`] would remove.
///
/// # Safety
/// Caller must ensure `psid`, if given, points to a valid SID.
pub unsafe fn has_exact_deny_read_ace_for_new_files(
    path: &Path,
    psid: Option<*mut c_void>,
) -> Result<bool> {
    let mut p_sd: *mut c_void = std::ptr::null_mut();
    let mut p_dacl: *mut ACL = std::ptr::null_mut();
    let code = GetNamedSecurityInfoW(
        to_wide(path).as_ptr(),
        1,
        DACL_SECURITY_INFORMATION,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        &mut p_dacl,
        std::ptr::null_mut(),
        &mut p_sd,
    );
    if code != ERROR_SUCCESS {
        return Err(anyhow!("GetNamedSecurityInfoW failed: {code}"));
    }
    let mut found = false;
    let mut info: ACL_SIZE_INFORMATION = std::mem::zeroed();
    if !p_dacl.is_null()
        && GetAclInformation(
            p_dacl as *const ACL,
            &mut info as *mut _ as *mut c_void,
            std::mem::size_of::<ACL_SIZE_INFORMATION>() as u32,
            AclSizeInformation,
        ) != 0
    {
        for i in 0..info.AceCount {
            let mut p_ace: *mut c_void = std::ptr::null_mut();
            if GetAce(p_dacl as *const ACL, i, &mut p_ace) != 0
                && is_new_file_read_deny(p_ace, psid)
            {
                found = true;
                break;
            }
        }
    }
    if !p_sd.is_null() {
        LocalFree(p_sd as HLOCAL);
    }
    Ok(found)
}

/// PF-27-S07: removes exactly the entry [`add_deny_read_ace_for_new_files`]
/// added for `psid` on `path`, and with it the copies files directly in
/// `path` inherited. Every other entry, and whether the DACL is protected,
/// stays as it was. Returns whether an entry was removed.
///
/// # Safety
/// Caller must ensure `psid` points to a valid SID and `path` is a directory.
pub unsafe fn remove_deny_read_ace_for_new_files(path: &Path, psid: *mut c_void) -> Result<bool> {
    let mut p_sd: *mut c_void = std::ptr::null_mut();
    let mut p_dacl: *mut ACL = std::ptr::null_mut();
    let code = GetNamedSecurityInfoW(
        to_wide(path).as_ptr(),
        1,
        DACL_SECURITY_INFORMATION,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        &mut p_dacl,
        std::ptr::null_mut(),
        &mut p_sd,
    );
    if code != ERROR_SUCCESS {
        return Err(anyhow!("GetNamedSecurityInfoW failed: {code}"));
    }
    let result = remove_new_file_read_denies(path, p_sd, p_dacl, psid);
    if !p_sd.is_null() {
        LocalFree(p_sd as HLOCAL);
    }
    result
}

unsafe fn remove_new_file_read_denies(
    path: &Path,
    p_sd: *mut c_void,
    p_dacl: *mut ACL,
    psid: *mut c_void,
) -> Result<bool> {
    if p_dacl.is_null() {
        return Ok(false);
    }
    let mut info: ACL_SIZE_INFORMATION = std::mem::zeroed();
    if GetAclInformation(
        p_dacl as *const ACL,
        &mut info as *mut _ as *mut c_void,
        std::mem::size_of::<ACL_SIZE_INFORMATION>() as u32,
        AclSizeInformation,
    ) == 0
    {
        return Err(anyhow!("GetAclInformation failed: {}", GetLastError()));
    }
    let revision = u32::from((*p_dacl).AclRevision);
    let size = (*p_dacl).AclSize;
    // u32 storage keeps the ACL suitably aligned.
    let mut buffer = vec![0_u32; usize::from(size).div_ceil(4)];
    let new_dacl = buffer.as_mut_ptr() as *mut ACL;
    if InitializeAcl(new_dacl, u32::from(size), revision) == 0 {
        return Err(anyhow!("InitializeAcl failed: {}", GetLastError()));
    }
    let mut removed = false;
    for i in 0..info.AceCount {
        let mut p_ace: *mut c_void = std::ptr::null_mut();
        if GetAce(p_dacl as *const ACL, i, &mut p_ace) == 0 {
            return Err(anyhow!("GetAce failed: {}", GetLastError()));
        }
        if is_new_file_read_deny(p_ace, Some(psid)) {
            removed = true;
            continue;
        }
        let ace_size = u32::from((*(p_ace as *const ACE_HEADER)).AceSize);
        if AddAce(new_dacl, revision, u32::MAX, p_ace, ace_size) == 0 {
            return Err(anyhow!("AddAce failed: {}", GetLastError()));
        }
    }
    if !removed {
        return Ok(false);
    }
    let mut control: u16 = 0;
    let mut sd_revision: u32 = 0;
    if GetSecurityDescriptorControl(p_sd, &mut control, &mut sd_revision) == 0 {
        return Err(anyhow!(
            "GetSecurityDescriptorControl failed: {}",
            GetLastError()
        ));
    }
    let protection = if control & SE_DACL_PROTECTED != 0 {
        PROTECTED_DACL_SECURITY_INFORMATION
    } else {
        UNPROTECTED_DACL_SECURITY_INFORMATION
    };
    // Also recomputes the children's inherited entries, dropping the copies.
    let code = SetNamedSecurityInfoW(
        to_wide(path).as_ptr() as *mut u16,
        1,
        DACL_SECURITY_INFORMATION | protection,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        new_dacl,
        std::ptr::null_mut(),
    );
    if code != ERROR_SUCCESS {
        return Err(anyhow!("SetNamedSecurityInfoW failed: {code}"));
    }
    Ok(true)
}

#[derive(Clone, Copy)]
enum DenyAceKind {
    Read,
    /// Read, on the object itself only, and only an explicit entry counts as
    /// present (an inherited copy can disappear with its parent's entry).
    ReadExplicit,
    /// Read, inherited by files directly in the directory only.
    ReadNewFiles,
    Write,
    DeleteChild {
        inherit_to_subdirs: bool,
    },
}

impl DenyAceKind {
    fn mask(self) -> u32 {
        match self {
            Self::DeleteChild { .. } => FILE_DELETE_CHILD,
            Self::Read | Self::ReadExplicit | Self::ReadNewFiles => {
                FILE_GENERIC_READ | GENERIC_READ_MASK
            }
            Self::Write => {
                FILE_GENERIC_WRITE
                    | FILE_WRITE_DATA
                    | FILE_APPEND_DATA
                    | FILE_WRITE_EA
                    | FILE_WRITE_ATTRIBUTES
                    | GENERIC_WRITE_MASK
                    | DELETE
                    | FILE_DELETE_CHILD
            }
        }
    }

    fn inheritance(self) -> u32 {
        match self {
            Self::Read | Self::Write => CONTAINER_INHERIT_ACE | OBJECT_INHERIT_ACE,
            Self::ReadExplicit => 0,
            Self::ReadNewFiles => {
                OBJECT_INHERIT_ACE | u32::from(INHERIT_ONLY_ACE) | NO_PROPAGATE_INHERIT_ACE
            }
            Self::DeleteChild {
                inherit_to_subdirs: true,
            } => CONTAINER_INHERIT_ACE,
            Self::DeleteChild {
                inherit_to_subdirs: false,
            } => 0,
        }
    }

    unsafe fn already_present(self, p_dacl: *mut ACL, psid: *mut c_void) -> bool {
        match self {
            Self::Read => dacl_has_read_deny_for_sid(p_dacl, psid),
            Self::ReadExplicit => dacl_has_explicit_read_deny_for_sid(p_dacl, psid),
            Self::ReadNewFiles => dacl_has_new_file_read_deny_for_sid(p_dacl, psid),
            Self::Write => dacl_has_write_deny_for_sid(p_dacl, psid),
            Self::DeleteChild { inherit_to_subdirs } => {
                dacl_has_delete_child_deny_for_sid(p_dacl, psid, inherit_to_subdirs)
            }
        }
    }
}

/// Returns true when the DACL already denies `psid` `FILE_DELETE_CHILD` on the
/// object itself and, if `require_container_inherit`, propagates that deny to
/// subdirectories.
unsafe fn dacl_has_delete_child_deny_for_sid(
    p_dacl: *mut ACL,
    psid: *mut c_void,
    require_container_inherit: bool,
) -> bool {
    if p_dacl.is_null() {
        return false;
    }
    let mut info: ACL_SIZE_INFORMATION = std::mem::zeroed();
    let ok = GetAclInformation(
        p_dacl as *const ACL,
        &mut info as *mut _ as *mut c_void,
        std::mem::size_of::<ACL_SIZE_INFORMATION>() as u32,
        AclSizeInformation,
    );
    if ok == 0 {
        return false;
    }
    for i in 0..info.AceCount {
        let mut p_ace: *mut c_void = std::ptr::null_mut();
        if GetAce(p_dacl as *const ACL, i, &mut p_ace) == 0 {
            continue;
        }
        let hdr = &*(p_ace as *const ACE_HEADER);
        if hdr.AceType != ACCESS_DENIED_ACE_TYPE || (hdr.AceFlags & INHERIT_ONLY_ACE) != 0 {
            continue;
        }
        if require_container_inherit && (u32::from(hdr.AceFlags) & CONTAINER_INHERIT_ACE) == 0 {
            continue;
        }
        let ace = &*(p_ace as *const ACCESS_DENIED_ACE);
        let base = p_ace as usize;
        let sid_ptr =
            (base + std::mem::size_of::<ACE_HEADER>() + std::mem::size_of::<u32>()) as *mut c_void;
        if EqualSid(sid_ptr, psid) != 0 && (ace.Mask & FILE_DELETE_CHILD) != 0 {
            return true;
        }
    }
    false
}

unsafe fn add_deny_ace(path: &Path, psid: *mut c_void, kind: DenyAceKind) -> Result<bool> {
    let mut p_sd: *mut c_void = std::ptr::null_mut();
    let mut p_dacl: *mut ACL = std::ptr::null_mut();
    let code = GetNamedSecurityInfoW(
        to_wide(path).as_ptr(),
        1,
        DACL_SECURITY_INFORMATION,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        &mut p_dacl,
        std::ptr::null_mut(),
        &mut p_sd,
    );
    if code != ERROR_SUCCESS {
        return Err(anyhow!("GetNamedSecurityInfoW failed: {code}"));
    }
    let mut added = false;
    if !kind.already_present(p_dacl, psid) {
        let trustee = TRUSTEE_W {
            pMultipleTrustee: std::ptr::null_mut(),
            MultipleTrusteeOperation: 0,
            TrusteeForm: TRUSTEE_IS_SID,
            TrusteeType: TRUSTEE_IS_UNKNOWN,
            ptstrName: psid as *mut u16,
        };
        let mut explicit: EXPLICIT_ACCESS_W = std::mem::zeroed();
        explicit.grfAccessPermissions = kind.mask();
        explicit.grfAccessMode = DENY_ACCESS;
        explicit.grfInheritance = kind.inheritance();
        explicit.Trustee = trustee;
        let mut p_new_dacl: *mut ACL = std::ptr::null_mut();
        let code2 = SetEntriesInAclW(1, &explicit, p_dacl, &mut p_new_dacl);
        if code2 == ERROR_SUCCESS {
            let code3 = SetNamedSecurityInfoW(
                to_wide(path).as_ptr() as *mut u16,
                1,
                DACL_SECURITY_INFORMATION,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                p_new_dacl,
                std::ptr::null_mut(),
            );
            if code3 == ERROR_SUCCESS {
                added = true;
            }
            if !p_new_dacl.is_null() {
                LocalFree(p_new_dacl as HLOCAL);
            }
        }
    }
    if !p_sd.is_null() {
        LocalFree(p_sd as HLOCAL);
    }
    Ok(added)
}

/// Adds a deny ACE to prevent reads for the given SID on the target path.
///
/// `SetEntriesInAclW` places newly-created deny ACEs before allow ACEs, which
/// keeps the resulting DACL in the order Windows expects for denies to win.
/// The ACE is inheritable so a deny applied to a materialized directory also
/// covers files and directories later created underneath it.
///
/// # Safety
/// Caller must ensure `psid` points to a valid SID and `path` refers to an existing file or directory.
pub unsafe fn add_deny_read_ace(path: &Path, psid: *mut c_void) -> Result<bool> {
    add_deny_ace(path, psid, DenyAceKind::Read)
}

pub unsafe fn revoke_ace(path: &Path, psid: *mut c_void) {
    let mut p_sd: *mut c_void = std::ptr::null_mut();
    let mut p_dacl: *mut ACL = std::ptr::null_mut();
    let code = GetNamedSecurityInfoW(
        to_wide(path).as_ptr(),
        1,
        DACL_SECURITY_INFORMATION,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        &mut p_dacl,
        std::ptr::null_mut(),
        &mut p_sd,
    );
    if code != ERROR_SUCCESS {
        if !p_sd.is_null() {
            LocalFree(p_sd as HLOCAL);
        }
        return;
    }
    let trustee = TRUSTEE_W {
        pMultipleTrustee: std::ptr::null_mut(),
        MultipleTrusteeOperation: 0,
        TrusteeForm: TRUSTEE_IS_SID,
        TrusteeType: TRUSTEE_IS_UNKNOWN,
        ptstrName: psid as *mut u16,
    };
    let mut explicit: EXPLICIT_ACCESS_W = std::mem::zeroed();
    explicit.grfAccessPermissions = 0;
    explicit.grfAccessMode = 4; // REVOKE_ACCESS
    explicit.grfInheritance = CONTAINER_INHERIT_ACE | OBJECT_INHERIT_ACE;
    explicit.Trustee = trustee;
    let mut p_new_dacl: *mut ACL = std::ptr::null_mut();
    let code2 = SetEntriesInAclW(1, &explicit, p_dacl, &mut p_new_dacl);
    if code2 == ERROR_SUCCESS {
        let _ = SetNamedSecurityInfoW(
            to_wide(path).as_ptr() as *mut u16,
            1,
            DACL_SECURITY_INFORMATION,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            p_new_dacl,
            std::ptr::null_mut(),
        );
        if !p_new_dacl.is_null() {
            LocalFree(p_new_dacl as HLOCAL);
        }
    }
    if !p_sd.is_null() {
        LocalFree(p_sd as HLOCAL);
    }
}

/// Grants RX to the null device for the given SID to support stdout/stderr redirection.
///
/// # Safety
/// Caller must ensure `psid` is a valid SID pointer.
pub unsafe fn allow_null_device(psid: *mut c_void) {
    let desired = 0x00020000 | 0x00040000; // READ_CONTROL | WRITE_DAC
    let h = CreateFileW(
        to_wide(r"\\\\.\\NUL").as_ptr(),
        desired,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        std::ptr::null_mut(),
        OPEN_EXISTING,
        FILE_ATTRIBUTE_NORMAL,
        0,
    );
    if h == 0 || h == INVALID_HANDLE_VALUE {
        return;
    }
    let mut p_sd: *mut c_void = std::ptr::null_mut();
    let mut p_dacl: *mut ACL = std::ptr::null_mut();
    let code = GetSecurityInfo(
        h,
        SE_KERNEL_OBJECT as i32,
        DACL_SECURITY_INFORMATION,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        &mut p_dacl,
        std::ptr::null_mut(),
        &mut p_sd,
    );
    if code == ERROR_SUCCESS {
        let trustee = TRUSTEE_W {
            pMultipleTrustee: std::ptr::null_mut(),
            MultipleTrusteeOperation: 0,
            TrusteeForm: TRUSTEE_IS_SID,
            TrusteeType: TRUSTEE_IS_UNKNOWN,
            ptstrName: psid as *mut u16,
        };
        let mut explicit: EXPLICIT_ACCESS_W = std::mem::zeroed();
        explicit.grfAccessPermissions =
            FILE_GENERIC_READ | FILE_GENERIC_WRITE | FILE_GENERIC_EXECUTE;
        explicit.grfAccessMode = 2; // SET_ACCESS
        explicit.grfInheritance = 0;
        explicit.Trustee = trustee;
        let mut p_new_dacl: *mut ACL = std::ptr::null_mut();
        let code2 = SetEntriesInAclW(1, &explicit, p_dacl, &mut p_new_dacl);
        if code2 == ERROR_SUCCESS {
            let _ = SetSecurityInfo(
                h,
                SE_KERNEL_OBJECT as i32,
                DACL_SECURITY_INFORMATION,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                p_new_dacl,
                std::ptr::null_mut(),
            );
            if !p_new_dacl.is_null() {
                LocalFree(p_new_dacl as HLOCAL);
            }
        }
    }
    if !p_sd.is_null() {
        LocalFree(p_sd as HLOCAL);
    }
    CloseHandle(h);
}
const CONTAINER_INHERIT_ACE: u32 = 0x2;
const NO_PROPAGATE_INHERIT_ACE: u32 = 0x4;
const OBJECT_INHERIT_ACE: u32 = 0x1;

#[cfg(test)]
#[path = "acl_tests.rs"]
mod tests;
