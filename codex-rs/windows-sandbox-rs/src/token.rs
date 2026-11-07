use crate::winutil::to_wide;
use anyhow::Result;
use anyhow::anyhow;
use anyhow::ensure;
use std::ffi::c_void;
use windows_sys::Win32::Foundation::CloseHandle;
use windows_sys::Win32::Foundation::ERROR_SUCCESS;
use windows_sys::Win32::Foundation::GetLastError;
use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::Foundation::HLOCAL;
use windows_sys::Win32::Foundation::LUID;
use windows_sys::Win32::Foundation::LocalFree;
use windows_sys::Win32::Security::AdjustTokenPrivileges;
use windows_sys::Win32::Security::Authorization::EXPLICIT_ACCESS_W;
use windows_sys::Win32::Security::Authorization::GRANT_ACCESS;
use windows_sys::Win32::Security::Authorization::SetEntriesInAclW;
use windows_sys::Win32::Security::Authorization::TRUSTEE_IS_SID;
use windows_sys::Win32::Security::Authorization::TRUSTEE_IS_UNKNOWN;
use windows_sys::Win32::Security::Authorization::TRUSTEE_W;
use windows_sys::Win32::Security::CopySid;
use windows_sys::Win32::Security::CreateRestrictedToken;
use windows_sys::Win32::Security::CreateWellKnownSid;
use windows_sys::Win32::Security::GetTokenInformation;
use windows_sys::Win32::Security::IsValidSid;
use windows_sys::Win32::Security::LookupPrivilegeValueW;
use windows_sys::Win32::Security::SetTokenInformation;

use windows_sys::Win32::Security::ACL;
use windows_sys::Win32::Security::SID_AND_ATTRIBUTES;
use windows_sys::Win32::Security::TOKEN_ADJUST_DEFAULT;
use windows_sys::Win32::Security::TOKEN_ADJUST_PRIVILEGES;
use windows_sys::Win32::Security::TOKEN_ADJUST_SESSIONID;
use windows_sys::Win32::Security::TOKEN_ASSIGN_PRIMARY;
use windows_sys::Win32::Security::TOKEN_DUPLICATE;
use windows_sys::Win32::Security::TOKEN_GROUPS;
use windows_sys::Win32::Security::TOKEN_PRIVILEGES;
use windows_sys::Win32::Security::TOKEN_QUERY;
use windows_sys::Win32::Security::TokenDefaultDacl;
use windows_sys::Win32::Security::TokenGroups;
use windows_sys::Win32::System::Threading::GetCurrentProcess;

const DISABLE_MAX_PRIVILEGE: u32 = 0x01;
const LUA_TOKEN: u32 = 0x04;
const WRITE_RESTRICTED: u32 = 0x08;
const GENERIC_ALL: u32 = 0x1000_0000;
const GENERIC_EXECUTE: u32 = 0x2000_0000;
const GENERIC_READ: u32 = 0x8000_0000;
const WIN_WORLD_SID: i32 = 1;
const WIN_LOCAL_SYSTEM_SID: i32 = 22;
const SE_GROUP_LOGON_ID: u32 = 0xC0000000;

#[repr(C)]
struct TokenDefaultDaclInfo {
    default_dacl: *mut ACL,
}

<<<<<<< HEAD
/// Sets a permissive default DACL so sandboxed processes can create pipes/IPC objects
/// without hitting ACCESS_DENIED when PowerShell builds pipelines.
unsafe fn set_default_dacl(h_token: HANDLE, sids: &[*mut c_void]) -> Result<()> {
    let grants: Vec<(*mut c_void, u32)> = sids.iter().map(|sid| (*sid, GENERIC_ALL)).collect();
    set_default_dacl_grants(h_token, &grants)
}

/// Sets the token's default DACL to exactly `grants` (SID, access mask).
unsafe fn set_default_dacl_grants(h_token: HANDLE, grants: &[(*mut c_void, u32)]) -> Result<()> {
    if grants.is_empty() {
        return Ok(());
    }
    let entries: Vec<EXPLICIT_ACCESS_W> = grants
        .iter()
        .map(|(sid, access)| EXPLICIT_ACCESS_W {
            grfAccessPermissions: *access,
            grfAccessMode: GRANT_ACCESS,
            grfInheritance: 0,
            Trustee: TRUSTEE_W {
                pMultipleTrustee: std::ptr::null_mut(),
                MultipleTrusteeOperation: 0,
                TrusteeForm: TRUSTEE_IS_SID,
                TrusteeType: TRUSTEE_IS_UNKNOWN,
                ptstrName: *sid as *mut u16,
            },
        })
        .collect();
||||||| 413492cd6c
/// Sets a permissive default DACL so sandboxed processes can create pipes/IPC objects
/// without hitting ACCESS_DENIED when PowerShell builds pipelines.
unsafe fn set_default_dacl(h_token: HANDLE, sids: &[*mut c_void]) -> Result<()> {
    if sids.is_empty() {
        return Ok(());
    }
    let entries: Vec<EXPLICIT_ACCESS_W> = sids
        .iter()
        .map(|sid| EXPLICIT_ACCESS_W {
            grfAccessPermissions: GENERIC_ALL,
            grfAccessMode: GRANT_ACCESS,
            grfInheritance: 0,
            Trustee: TRUSTEE_W {
                pMultipleTrustee: std::ptr::null_mut(),
                MultipleTrusteeOperation: 0,
                TrusteeForm: TRUSTEE_IS_SID,
                TrusteeType: TRUSTEE_IS_UNKNOWN,
                ptstrName: *sid as *mut u16,
            },
        })
        .collect();
=======
/// Keep child-process and IPC access within the runner's logon session.
/// Elevated runners have distinct logon SIDs even when they use the same account.
unsafe fn set_default_dacl(h_token: HANDLE, logon_sid: *mut c_void) -> Result<()> {
    let owner_rights = LocalSid::from_string("S-1-3-4")?;
    // The shared account also owns these objects. An OWNER RIGHTS ACE suppresses
    // its implicit WRITE_DAC grant, which would otherwise let a different logon
    // rewrite this DACL. The creating logon retains full access explicitly.
    let entries = [
        (logon_sid, GENERIC_ALL),
        (
            owner_rights.as_ptr(),
            windows_sys::Win32::Storage::FileSystem::READ_CONTROL,
        ),
    ]
    .map(|(sid, access)| EXPLICIT_ACCESS_W {
        grfAccessPermissions: access,
        grfAccessMode: GRANT_ACCESS,
        grfInheritance: 0,
        Trustee: TRUSTEE_W {
            pMultipleTrustee: std::ptr::null_mut(),
            MultipleTrusteeOperation: 0,
            TrusteeForm: TRUSTEE_IS_SID,
            TrusteeType: TRUSTEE_IS_UNKNOWN,
            ptstrName: sid as *mut u16,
        },
    });
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
    let mut p_new_dacl: *mut ACL = std::ptr::null_mut();
    let res = SetEntriesInAclW(
        entries.len() as u32,
        entries.as_ptr(),
        std::ptr::null_mut(),
        &mut p_new_dacl,
    );
    if res != ERROR_SUCCESS {
        return Err(anyhow!("SetEntriesInAclW failed: {res}"));
    }
    let mut info = TokenDefaultDaclInfo {
        default_dacl: p_new_dacl,
    };
    let ok = SetTokenInformation(
        h_token,
        TokenDefaultDacl,
        &mut info as *mut _ as *mut c_void,
        std::mem::size_of::<TokenDefaultDaclInfo>() as u32,
    );
    if ok == 0 {
        let err = GetLastError();
        if !p_new_dacl.is_null() {
            LocalFree(p_new_dacl as HLOCAL);
        }
        return Err(anyhow!(
            "SetTokenInformation(TokenDefaultDacl) failed: {err}",
        ));
    }
    if !p_new_dacl.is_null() {
        LocalFree(p_new_dacl as HLOCAL);
    }
    Ok(())
}

pub unsafe fn world_sid() -> Result<Vec<u8>> {
    well_known_sid(WIN_WORLD_SID)
}

unsafe fn well_known_sid(kind: i32) -> Result<Vec<u8>> {
    let mut size: u32 = 0;
    CreateWellKnownSid(kind, std::ptr::null_mut(), std::ptr::null_mut(), &mut size);
    let mut buf: Vec<u8> = vec![0u8; size as usize];
    let ok = CreateWellKnownSid(
        kind,
        std::ptr::null_mut(),
        buf.as_mut_ptr() as *mut c_void,
        &mut size,
    );
    if ok == 0 {
        return Err(anyhow!("CreateWellKnownSid failed: {}", GetLastError()));
    }
    Ok(buf)
}

/// # Safety
/// Caller is responsible for freeing the returned SID with `LocalFree`.
pub unsafe fn convert_string_sid_to_sid(s: &str) -> Option<*mut c_void> {
    #[link(name = "advapi32")]
    unsafe extern "system" {
        fn ConvertStringSidToSidW(StringSid: *const u16, Sid: *mut *mut c_void) -> i32;
    }
    let mut psid: *mut c_void = std::ptr::null_mut();
    let ok = unsafe { ConvertStringSidToSidW(to_wide(s).as_ptr(), &mut psid) };
    if ok != 0 { Some(psid) } else { None }
}

/// Owns a SID allocated by `ConvertStringSidToSidW` and releases it with `LocalFree`.
pub struct LocalSid {
    psid: *mut c_void,
}

impl LocalSid {
    pub fn from_string(sid: &str) -> Result<Self> {
        let psid = unsafe { convert_string_sid_to_sid(sid) }
            .ok_or_else(|| anyhow!("invalid SID string: {sid}"))?;
        Ok(Self { psid })
    }

    pub fn as_ptr(&self) -> *mut c_void {
        self.psid
    }
}

impl Drop for LocalSid {
    fn drop(&mut self) {
        if !self.psid.is_null() {
            unsafe {
                LocalFree(self.psid as HLOCAL);
            }
        }
    }
}

/// # Safety
/// Caller must close the returned token handle.
pub unsafe fn get_current_token_for_restriction() -> Result<HANDLE> {
    let desired = TOKEN_DUPLICATE
        | TOKEN_QUERY
        | TOKEN_ASSIGN_PRIMARY
        | TOKEN_ADJUST_DEFAULT
        | TOKEN_ADJUST_SESSIONID
        | TOKEN_ADJUST_PRIVILEGES;
    let mut h: HANDLE = std::ptr::null_mut();
    #[link(name = "advapi32")]
    unsafe extern "system" {
        fn OpenProcessToken(
            ProcessHandle: HANDLE,
            DesiredAccess: u32,
            TokenHandle: *mut HANDLE,
        ) -> i32;
    }
    let ok = unsafe { OpenProcessToken(GetCurrentProcess(), desired, &mut h) };
    if ok == 0 {
        return Err(anyhow!("OpenProcessToken failed: {}", GetLastError()));
    }
    Ok(h)
}

/// An owned token group, including attributes such as enabled and deny-only.
#[derive(Debug, PartialEq, Eq)]
pub struct TokenGroup {
    pub sid: Vec<u8>,
    pub attributes: u32,
}

/// Queries groups without filtering membership or retaining pointers into Windows' buffer.
///
/// # Safety
/// `token` must remain a valid token handle with `TOKEN_QUERY` access during this call.
pub unsafe fn token_groups(token: HANDLE, max_bytes: u32) -> Result<Vec<TokenGroup>> {
    let mut needed = 0;
    GetTokenInformation(token, TokenGroups, std::ptr::null_mut(), 0, &mut needed);
    ensure!(
        needed > 0 && needed <= max_bytes,
        "invalid token group size"
    );
    let mut buffer = vec![0u8; needed as usize];
    if GetTokenInformation(
        token,
        TokenGroups,
        buffer.as_mut_ptr().cast(),
        needed,
        &mut needed,
    ) == 0
    {
        return Err(anyhow!(
            "GetTokenInformation(TokenGroups) failed: {}",
            GetLastError()
        ));
    }
    ensure!(
        needed as usize <= buffer.len(),
        "invalid token group result size"
    );
    decode_token_groups(&buffer[..needed as usize])
}

fn decode_token_groups(buffer: &[u8]) -> Result<Vec<TokenGroup>> {
    let offset = std::mem::offset_of!(TOKEN_GROUPS, Groups);
    let stride = std::mem::size_of::<SID_AND_ATTRIBUTES>();
    ensure!(buffer.len() >= offset, "truncated token group header");
    let count = unsafe { std::ptr::read_unaligned(buffer.as_ptr().cast::<u32>()) } as usize;
    ensure!(
        count <= (buffer.len() - offset) / stride,
        "truncated token groups"
    );
    let mut groups = Vec::with_capacity(count);
    for index in 0..count {
        let entry = unsafe {
            std::ptr::read_unaligned(
                buffer
                    .as_ptr()
                    .add(offset + index * stride)
                    .cast::<SID_AND_ATTRIBUTES>(),
            )
        };
        // Bound the header and every subauthority before calling a SID API.
        let sid_offset = (entry.Sid as usize).wrapping_sub(buffer.as_ptr() as usize);
        ensure!(
            sid_offset <= buffer.len().saturating_sub(8),
            "invalid token group SID pointer"
        );
        ensure!(buffer[sid_offset] == 1, "invalid token group SID revision");
        let sid_len = 8 + usize::from(buffer[sid_offset + 1]) * 4;
        ensure!(
            sid_len <= 68 && sid_len <= buffer.len() - sid_offset,
            "invalid token group SID size"
        );
        ensure!(
            unsafe { IsValidSid(entry.Sid) } != 0,
            "invalid token group SID"
        );
        let mut sid = vec![0u8; sid_len];
        ensure!(
            unsafe { CopySid(sid_len as u32, sid.as_mut_ptr().cast(), entry.Sid) } != 0,
            "invalid token group SID"
        );
        groups.push(TokenGroup {
            sid,
            attributes: entry.Attributes,
        });
    }
    Ok(groups)
}

pub unsafe fn get_logon_sid_bytes(h_token: HANDLE) -> Result<Vec<u8>> {
    unsafe fn scan_token_groups_for_logon(h: HANDLE) -> Option<Vec<u8>> {
        token_groups(h, u32::MAX)
            .ok()?
            .into_iter()
            .find(|group| group.attributes & SE_GROUP_LOGON_ID == SE_GROUP_LOGON_ID)
            .map(|group| group.sid)
    }

    if let Some(v) = scan_token_groups_for_logon(h_token) {
        return Ok(v);
    }

    #[repr(C)]
    struct TOKEN_LINKED_TOKEN {
        linked_token: HANDLE,
    }
    const TOKEN_LINKED_TOKEN_CLASS: i32 = 19; // TokenLinkedToken
    let mut ln_needed: u32 = 0;
    GetTokenInformation(
        h_token,
        TOKEN_LINKED_TOKEN_CLASS,
        std::ptr::null_mut(),
        0,
        &mut ln_needed,
    );
    if ln_needed >= std::mem::size_of::<TOKEN_LINKED_TOKEN>() as u32 {
        let mut ln_buf: Vec<u8> = vec![0u8; ln_needed as usize];
        let ok = GetTokenInformation(
            h_token,
            TOKEN_LINKED_TOKEN_CLASS,
            ln_buf.as_mut_ptr() as *mut c_void,
            ln_needed,
            &mut ln_needed,
        );
        if ok != 0 {
            let lt: TOKEN_LINKED_TOKEN =
                std::ptr::read_unaligned(ln_buf.as_ptr() as *const TOKEN_LINKED_TOKEN);
            if !lt.linked_token.is_null() {
                let res = scan_token_groups_for_logon(lt.linked_token);
                CloseHandle(lt.linked_token);
                if let Some(v) = res {
                    return Ok(v);
                }
            }
        }
    }

    Err(anyhow!("Logon SID not present on token"))
}

<<<<<<< HEAD
pub(crate) unsafe fn get_user_sid_bytes(h_token: HANDLE) -> Result<Vec<u8>> {
    let mut needed: u32 = 0;
    GetTokenInformation(h_token, TokenUser, std::ptr::null_mut(), 0, &mut needed);
    if needed == 0 {
        return Err(anyhow!("TokenUser size query returned 0"));
    }
    let mut user_buf: Vec<u8> = vec![0u8; needed as usize];
    let ok = GetTokenInformation(
        h_token,
        TokenUser,
        user_buf.as_mut_ptr() as *mut c_void,
        needed,
        &mut needed,
    );
    if ok == 0 || (needed as usize) < std::mem::size_of::<TOKEN_USER>() {
        return Err(anyhow!(
            "GetTokenInformation(TokenUser) failed: {}",
            GetLastError()
        ));
    }
    let token_user: TOKEN_USER = std::ptr::read_unaligned(user_buf.as_ptr() as *const TOKEN_USER);
    let sid_len = GetLengthSid(token_user.User.Sid);
    if sid_len == 0 {
        return Err(anyhow!(
            "GetLengthSid(TokenUser) failed: {}",
            GetLastError()
        ));
    }
    let mut user_sid_bytes = vec![0u8; sid_len as usize];
    if CopySid(
        sid_len,
        user_sid_bytes.as_mut_ptr() as *mut c_void,
        token_user.User.Sid,
    ) == 0
    {
        return Err(anyhow!("CopySid(TokenUser) failed: {}", GetLastError()));
    }
    Ok(user_sid_bytes)
}
||||||| 413492cd6c
unsafe fn get_user_sid_bytes(h_token: HANDLE) -> Result<Vec<u8>> {
    let mut needed: u32 = 0;
    GetTokenInformation(h_token, TokenUser, std::ptr::null_mut(), 0, &mut needed);
    if needed == 0 {
        return Err(anyhow!("TokenUser size query returned 0"));
    }
    let mut user_buf: Vec<u8> = vec![0u8; needed as usize];
    let ok = GetTokenInformation(
        h_token,
        TokenUser,
        user_buf.as_mut_ptr() as *mut c_void,
        needed,
        &mut needed,
    );
    if ok == 0 || (needed as usize) < std::mem::size_of::<TOKEN_USER>() {
        return Err(anyhow!(
            "GetTokenInformation(TokenUser) failed: {}",
            GetLastError()
        ));
    }
    let token_user: TOKEN_USER = std::ptr::read_unaligned(user_buf.as_ptr() as *const TOKEN_USER);
    let sid_len = GetLengthSid(token_user.User.Sid);
    if sid_len == 0 {
        return Err(anyhow!(
            "GetLengthSid(TokenUser) failed: {}",
            GetLastError()
        ));
    }
    let mut user_sid_bytes = vec![0u8; sid_len as usize];
    if CopySid(
        sid_len,
        user_sid_bytes.as_mut_ptr() as *mut c_void,
        token_user.User.Sid,
    ) == 0
    {
        return Err(anyhow!("CopySid(TokenUser) failed: {}", GetLastError()));
    }
    Ok(user_sid_bytes)
}
=======
pub(crate) use crate::token_user::get_user_sid_bytes;
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a

unsafe fn enable_single_privilege(h_token: HANDLE, name: &str) -> Result<()> {
    let mut luid = LUID {
        LowPart: 0,
        HighPart: 0,
    };
    let ok = LookupPrivilegeValueW(std::ptr::null(), to_wide(name).as_ptr(), &mut luid);
    if ok == 0 {
        return Err(anyhow!("LookupPrivilegeValueW failed: {}", GetLastError()));
    }
    let mut tp: TOKEN_PRIVILEGES = std::mem::zeroed();
    tp.PrivilegeCount = 1;
    tp.Privileges[0].Luid = luid;
    tp.Privileges[0].Attributes = 0x00000002; // SE_PRIVILEGE_ENABLED
    let ok2 = AdjustTokenPrivileges(
        h_token,
        0,
        &tp,
        0,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
    );
    if ok2 == 0 {
        return Err(anyhow!("AdjustTokenPrivileges failed: {}", GetLastError()));
    }
    let err = GetLastError();
    if err != 0 {
        return Err(anyhow!("AdjustTokenPrivileges error {err}"));
    }
    Ok(())
}

/// A read-only restricted token with the pre-D259 default DACL (no
/// per-command identity): legacy commands use
/// [`create_legacy_token_with_caps_from`] instead. Kept for tests that probe
/// Core's or the broker's objects from a restricted token.
///
/// # Safety
/// Caller must close the returned token handle; base_token must be a valid primary token.
pub unsafe fn create_readonly_token_with_cap_from(
    base_token: HANDLE,
    psid_capability: *mut c_void,
) -> Result<(HANDLE, *mut c_void)> {
    let new_token = create_token_with_caps_from(base_token, &[psid_capability], &[])?;
    Ok((new_token, psid_capability))
}

/// Create a restricted token that includes all provided capability SIDs.
///
/// # Safety
/// Caller must close the returned token handle; base_token must be a valid primary token.
pub unsafe fn create_workspace_write_token_with_caps_from(
    base_token: HANDLE,
    psid_capabilities: &[*mut c_void],
) -> Result<HANDLE> {
    create_token_with_caps_from(base_token, psid_capabilities, &[])
}

/// D259 (#1507): a legacy (non-elevated) command's token: the capability
/// SIDs plus `psid_command`, a SID unique to this command, as restricting
/// SIDs. Its default DACL, which the objects its processes create without
/// explicit security get (their processes, threads, tokens, pipes, events,
/// sections), grants full access only to the user, SYSTEM and
/// `psid_command`; Everyone, the logon SID and the capability SIDs get read
/// and execute. A write-restricted token needs a restricting SID granted
/// each write right, so one legacy command can still write to its own
/// objects, but not to another command's: it can't open them with
/// `PROCESS_CREATE_PROCESS` (to start a child in that command's job),
/// `WRITE_DAC`, `PROCESS_DUP_HANDLE` or memory writes. Before D259 the default
/// DACL granted Everyone and the logon SID, both restricting SIDs of every
/// legacy token, `GENERIC_ALL`. Not closed: `TOKEN_DUPLICATE` isn't a write
/// right, so another legacy command can still impersonate this command's
/// token, and with it (it carries `psid_command`) write to this command's
/// processes and writable roots (#1560). Read/execute still includes
/// `PROCESS_TERMINATE`.
///
/// # Safety
/// Caller must close the returned token handle; base_token must be a valid primary token,
/// and every SID pointer must be valid.
pub(crate) unsafe fn create_legacy_token_with_caps_from(
    base_token: HANDLE,
    psid_capabilities: &[*mut c_void],
    psid_command: *mut c_void,
) -> Result<HANDLE> {
    create_token(base_token, psid_capabilities, &[], Some(psid_command))
}

/// Create a restricted token that includes all provided capability SIDs, the token user SID, and
/// any additional restricting SIDs.
///
/// This is intended for the elevated sandbox backend, where the token user is the dedicated
/// sandbox account rather than the real signed-in user.
///
/// # Safety
/// Caller must close the returned token handle; base_token must be a valid primary token.
pub unsafe fn create_workspace_write_token_with_caps_and_user_from(
    base_token: HANDLE,
    psid_capabilities: &[*mut c_void],
    additional_restricting_sids: &[*mut c_void],
) -> Result<HANDLE> {
    create_token_with_caps_user_and_additional_restrictions_from(
        base_token,
        psid_capabilities,
        additional_restricting_sids,
    )
}

/// Create a restricted token that includes all provided capability SIDs.
///
/// # Safety
/// Caller must close the returned token handle; base_token must be a valid primary token.
pub unsafe fn create_readonly_token_with_caps_from(
    base_token: HANDLE,
    psid_capabilities: &[*mut c_void],
) -> Result<HANDLE> {
    create_token_with_caps_from(base_token, psid_capabilities, &[])
}

/// Create a restricted token that includes all provided capability SIDs, the token user SID, and
/// any additional restricting SIDs.
///
/// This is intended for the elevated sandbox backend, where the token user is the dedicated
/// sandbox account rather than the real signed-in user.
///
/// # Safety
/// Caller must close the returned token handle; base_token must be a valid primary token.
pub unsafe fn create_readonly_token_with_caps_and_user_from(
    base_token: HANDLE,
    psid_capabilities: &[*mut c_void],
    additional_restricting_sids: &[*mut c_void],
) -> Result<HANDLE> {
    create_token_with_caps_user_and_additional_restrictions_from(
        base_token,
        psid_capabilities,
        additional_restricting_sids,
    )
}

unsafe fn create_token_with_caps_user_and_additional_restrictions_from(
    base_token: HANDLE,
    psid_capabilities: &[*mut c_void],
    additional_restricting_sids: &[*mut c_void],
) -> Result<HANDLE> {
    let mut user_sid_bytes = get_user_sid_bytes(base_token)?;
    let psid_user = user_sid_bytes.as_mut_ptr() as *mut c_void;
    let mut extra_restricting_sids = Vec::with_capacity(additional_restricting_sids.len() + 1);
    extra_restricting_sids.push(psid_user);
    extra_restricting_sids.extend_from_slice(additional_restricting_sids);
    create_token_with_caps_from(base_token, psid_capabilities, &extra_restricting_sids)
}

unsafe fn create_token_with_caps_from(
    base_token: HANDLE,
    psid_capabilities: &[*mut c_void],
    extra_restricting_sids: &[*mut c_void],
) -> Result<HANDLE> {
    create_token(
        base_token,
        psid_capabilities,
        extra_restricting_sids,
        /*command_identity*/ None,
    )
}

unsafe fn create_token(
    base_token: HANDLE,
    psid_capabilities: &[*mut c_void],
    extra_restricting_sids: &[*mut c_void],
    command_identity: Option<*mut c_void>,
) -> Result<HANDLE> {
    if psid_capabilities.is_empty() {
        return Err(anyhow!("no capability SIDs provided"));
    }
    let mut logon_sid_bytes = get_logon_sid_bytes(base_token)?;
    let psid_logon = logon_sid_bytes.as_mut_ptr() as *mut c_void;
    let mut everyone = world_sid()?;
    let psid_everyone = everyone.as_mut_ptr() as *mut c_void;

    let mut extra_restricting_sids = extra_restricting_sids.to_vec();
    extra_restricting_sids.extend(command_identity);

    // Exact order: Capabilities..., ExtraRestricting..., Logon, Everyone
    let mut entries: Vec<SID_AND_ATTRIBUTES> =
        vec![std::mem::zeroed(); psid_capabilities.len() + extra_restricting_sids.len() + 2];
    for (i, psid) in psid_capabilities.iter().enumerate() {
        entries[i].Sid = *psid;
        entries[i].Attributes = 0;
    }
    let extras_idx = psid_capabilities.len();
    for (i, psid) in extra_restricting_sids.iter().enumerate() {
        entries[extras_idx + i].Sid = *psid;
        entries[extras_idx + i].Attributes = 0;
    }
    let logon_idx = extras_idx + extra_restricting_sids.len();
    entries[logon_idx].Sid = psid_logon;
    entries[logon_idx].Attributes = 0;
    entries[logon_idx + 1].Sid = psid_everyone;
    entries[logon_idx + 1].Attributes = 0;

<<<<<<< HEAD
    let mut new_token: HANDLE = 0;
    // WRITE_RESTRICTED: restricting SIDs are only consulted for write accesses.
    // Observed on Windows (#158): generic-write rights, DELETE, and WRITE_DAC are
    // restricted, but FILE_DELETE_CHILD is not. NTFS lets a caller delete or move
    // an object it lacks DELETE on when the parent grants FILE_DELETE_CHILD, so a
    // parent that grants the user full control lets the sandbox delete or move
    // children that capability-SID ACEs protect. Legacy sessions close that route
    // with Low integrity outside writable roots (`integrity.rs`) and with
    // Everyone deny ACEs for protected paths inside them (`delete_child_route_denies`).
||||||| 413492cd6c
    let mut new_token: HANDLE = 0;
=======
    let mut new_token: HANDLE = std::ptr::null_mut();
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
    let flags = DISABLE_MAX_PRIVILEGE | LUA_TOKEN | WRITE_RESTRICTED;
    let ok = CreateRestrictedToken(
        base_token,
        flags,
        0,
        std::ptr::null(),
        0,
        std::ptr::null(),
        entries.len() as u32,
        entries.as_mut_ptr(),
        &mut new_token,
    );
    if ok == 0 {
        return Err(anyhow!("CreateRestrictedToken failed: {}", GetLastError()));
    }

<<<<<<< HEAD
    let dacl = if let Some(psid_command) = command_identity {
        set_legacy_default_dacl(
            new_token,
            base_token,
            psid_command,
            psid_logon,
            psid_everyone,
            psid_capabilities,
        )
    } else {
        // Additional restricting SIDs are identity markers, not capabilities. Deliberately
        // exclude them from the default DACL so possessing a route identity cannot grant
        // object access.
        let mut dacl_sids: Vec<*mut c_void> = Vec::with_capacity(psid_capabilities.len() + 2);
        dacl_sids.push(psid_logon);
        dacl_sids.push(psid_everyone);
        dacl_sids.extend_from_slice(psid_capabilities);
        set_default_dacl(new_token, &dacl_sids)
    };
    if let Err(err) = dacl {
        CloseHandle(new_token);
        return Err(err);
    }

    if let Err(err) = enable_single_privilege(new_token, "SeChangeNotifyPrivilege") {
        CloseHandle(new_token);
        return Err(err);
||||||| 413492cd6c
    // Additional restricting SIDs are identity markers, not capabilities. Deliberately exclude
    // them from the default DACL so possessing a route identity cannot grant object access.
    let mut dacl_sids: Vec<*mut c_void> = Vec::with_capacity(psid_capabilities.len() + 2);
    dacl_sids.push(psid_logon);
    dacl_sids.push(psid_everyone);
    dacl_sids.extend_from_slice(psid_capabilities);
    set_default_dacl(new_token, &dacl_sids)?;

    enable_single_privilege(new_token, "SeChangeNotifyPrivilege")?;
=======
    // Filesystem and route capabilities may be shared between launches. They
    // must not grant access to another launch's processes, threads, or IPC.
    if let Err(error) = set_default_dacl(new_token, psid_logon)
        .and_then(|()| enable_single_privilege(new_token, "SeChangeNotifyPrivilege"))
    {
        CloseHandle(new_token);
        return Err(error);
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
    }
    Ok(new_token)
}

/// D259: sets a legacy command's default DACL (see
/// [`create_legacy_token_with_caps_from`]) on `token`; the user SID comes
/// from `base_token`.
unsafe fn set_legacy_default_dacl(
    token: HANDLE,
    base_token: HANDLE,
    psid_command: *mut c_void,
    psid_logon: *mut c_void,
    psid_everyone: *mut c_void,
    psid_capabilities: &[*mut c_void],
) -> Result<()> {
    let mut user = get_user_sid_bytes(base_token)?;
    let mut system = well_known_sid(WIN_LOCAL_SYSTEM_SID)?;
    let read_execute = GENERIC_READ | GENERIC_EXECUTE;
    let mut grants = vec![
        (user.as_mut_ptr() as *mut c_void, GENERIC_ALL),
        (system.as_mut_ptr() as *mut c_void, GENERIC_ALL),
        (psid_command, GENERIC_ALL),
        (psid_logon, read_execute),
        (psid_everyone, read_execute),
    ];
    grants.extend(psid_capabilities.iter().map(|sid| (*sid, read_execute)));
    set_default_dacl_grants(token, &grants)
}

#[cfg(test)]
#[path = "token_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "token_groups_tests.rs"]
mod token_groups_tests;
