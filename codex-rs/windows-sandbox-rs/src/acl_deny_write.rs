//! Handle-based deny writes for the elevated helper (#1953).
use super::*;

/// Adds the normal inherited write deny to the object already opened by the
/// helper. Never resolves a path again. A failed ACL write is an error.
///
/// # Safety
/// `handle` must be a live file/directory handle opened with READ_CONTROL and
/// WRITE_DAC; `psid` must point to a valid SID for this call.
pub unsafe fn add_deny_write_ace_to_handle(handle: HANDLE, psid: *mut c_void) -> Result<bool> {
    let info = file_info(handle)?;
    let is_dir = info.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY != 0;
    let (dacl, descriptor) = security_info(handle)?;
    let result = (|| {
        let kind = DenyAceKind::Write;
        if kind.already_present(dacl, psid, is_dir) {
            return Ok(false);
        }
        let mut explicit: EXPLICIT_ACCESS_W = std::mem::zeroed();
        explicit.grfAccessPermissions = kind.mask();
        explicit.grfAccessMode = DENY_ACCESS;
        explicit.grfInheritance = kind.inheritance();
        explicit.Trustee = TRUSTEE_W {
            pMultipleTrustee: std::ptr::null_mut(),
            MultipleTrusteeOperation: 0,
            TrusteeForm: TRUSTEE_IS_SID,
            TrusteeType: TRUSTEE_IS_UNKNOWN,
            ptstrName: psid as *mut u16,
        };
        let mut new_dacl = std::ptr::null_mut();
        let code = SetEntriesInAclW(1, &explicit, dacl, &mut new_dacl);
        if code != ERROR_SUCCESS {
            return Err(anyhow!("SetEntriesInAclW failed: {code}"));
        }
        let protection = dacl_protection(descriptor);
        let result = protection.and_then(|protection| {
            let code = SetSecurityInfo(
                handle,
                /*objecttype*/ 1,
                DACL_SECURITY_INFORMATION | protection,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                new_dacl,
                std::ptr::null_mut(),
            );
            if code != ERROR_SUCCESS {
                return Err(anyhow!("SetSecurityInfo failed: {code}"));
            }
            Ok(true)
        });
        LocalFree(new_dacl as HLOCAL);
        result
    })();
    LocalFree(descriptor as HLOCAL);
    result
}
