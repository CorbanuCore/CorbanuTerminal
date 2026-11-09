use super::*;
use pretty_assertions::assert_eq;
use windows_sys::Win32::Security::ACCESS_ALLOWED_ACE;
use windows_sys::Win32::Security::EqualSid;
use windows_sys::Win32::Security::GetAce;
use windows_sys::Win32::Security::GetLengthSid;
use windows_sys::Win32::Security::TokenRestrictedSids;
use windows_sys::Win32::System::SystemServices::ACCESS_ALLOWED_ACE_TYPE;

unsafe fn token_has_restricting_sid(token: HANDLE, expected_sid: *mut c_void) -> Result<bool> {
    let mut needed = 0;
    GetTokenInformation(
        token,
        TokenRestrictedSids,
        std::ptr::null_mut(),
        0,
        &mut needed,
    );
    if needed == 0 {
        return Err(anyhow!(
            "GetTokenInformation(TokenRestrictedSids) size query failed: {}",
            GetLastError()
        ));
    }

    let mut buffer = vec![0_u8; needed as usize];
    if GetTokenInformation(
        token,
        TokenRestrictedSids,
        buffer.as_mut_ptr().cast(),
        needed,
        &mut needed,
    ) == 0
    {
        return Err(anyhow!(
            "GetTokenInformation(TokenRestrictedSids) failed: {}",
            GetLastError()
        ));
    }

    let group_count = std::ptr::read_unaligned(buffer.as_ptr().cast::<u32>()) as usize;
    let after_count = buffer.as_ptr().add(std::mem::size_of::<u32>()) as usize;
    let align = std::mem::align_of::<SID_AND_ATTRIBUTES>();
    let entries_addr = (after_count + (align - 1)) & !(align - 1);
    let restricting_sids =
        std::slice::from_raw_parts(entries_addr as *const SID_AND_ATTRIBUTES, group_count);
    Ok(restricting_sids
        .iter()
        .any(|entry| EqualSid(entry.Sid, expected_sid) != 0))
}

/// BUILTIN\Users, which the test process holds; it stands in for the sandbox
/// users' group, which it doesn't (#1795).
const TEST_GROUP_SID: &str = "S-1-5-32-545";

#[test]
fn elevated_token_includes_network_proxy_restricting_sid() -> Result<()> {
    let capability_sid = LocalSid::from_string("S-1-5-21-10-20-30-40")?;
    let network_proxy_sid = LocalSid::from_string("S-1-5-21-50-60-70-80")?;
    let group_sid = LocalSid::from_string(TEST_GROUP_SID)?;
    let base_token = unsafe { get_current_token_for_restriction()? };
    let restricted_token = unsafe {
        create_elevated_token_with_group(
            base_token,
            &[capability_sid.as_ptr()],
            &[network_proxy_sid.as_ptr()],
            group_sid.as_ptr(),
        )?
    };

    let has_network_proxy_sid =
        unsafe { token_has_restricting_sid(restricted_token, network_proxy_sid.as_ptr()) };
    unsafe {
        CloseHandle(restricted_token);
        CloseHandle(base_token);
    }

    assert!(has_network_proxy_sid?);
    Ok(())
}

/// Upstream (#47361), for the shared default DACL (`DefaultDacl::Shared`).
/// Corbanu's elevated builders set D324's DACL instead and refuse this test
/// process (it isn't in the sandbox users' group, #1795), and legacy commands
/// get D259/D429's (tested below).
#[test]
fn default_objects_deny_other_logons_even_with_the_same_owner() -> Result<()> {
    use std::os::windows::io::AsRawHandle;
    use std::os::windows::io::FromRawHandle;
    use std::os::windows::io::OwnedHandle;
    use windows_sys::Win32::Security::ACCESS_ALLOWED_ACE;
    use windows_sys::Win32::Security::AccessCheck;
    use windows_sys::Win32::Security::DuplicateToken;
    use windows_sys::Win32::Security::GENERIC_MAPPING;
    use windows_sys::Win32::Security::GetAce;
    use windows_sys::Win32::Security::InitializeSecurityDescriptor;
    use windows_sys::Win32::Security::MapGenericMask;
    use windows_sys::Win32::Security::SECURITY_DESCRIPTOR;
    use windows_sys::Win32::Security::SecurityImpersonation;
    use windows_sys::Win32::Security::SetSecurityDescriptorDacl;
    use windows_sys::Win32::Security::SetSecurityDescriptorGroup;
    use windows_sys::Win32::Security::SetSecurityDescriptorOwner;
    use windows_sys::Win32::Storage::FileSystem::READ_CONTROL;
    use windows_sys::Win32::Storage::FileSystem::WRITE_DAC;
    use windows_sys::Win32::System::Threading::PROCESS_ALL_ACCESS;
    use windows_sys::Win32::System::Threading::PROCESS_CREATE_THREAD;
    use windows_sys::Win32::System::Threading::PROCESS_QUERY_INFORMATION;
    use windows_sys::Win32::System::Threading::PROCESS_TERMINATE;
    use windows_sys::Win32::System::Threading::PROCESS_VM_OPERATION;
    use windows_sys::Win32::System::Threading::PROCESS_VM_READ;
    use windows_sys::Win32::System::Threading::PROCESS_VM_WRITE;

    unsafe {
        let base = OwnedHandle::from_raw_handle(get_current_token_for_restriction()? as _);
        let base_raw = base.as_raw_handle() as HANDLE;
        let capability = LocalSid::from_string("S-1-5-21-10-20-30-40")?;
        let victim = OwnedHandle::from_raw_handle(create_workspace_write_token_with_caps_from(
            base_raw,
            &[capability.as_ptr()],
        )? as _);
        let mut needed = 0;
        GetTokenInformation(
            victim.as_raw_handle() as _,
            TokenDefaultDacl,
            std::ptr::null_mut(),
            0,
            &mut needed,
        );
        let mut buffer = vec![0u8; needed as usize];
        ensure!(
            GetTokenInformation(
                victim.as_raw_handle() as _,
                TokenDefaultDacl,
                buffer.as_mut_ptr().cast(),
                needed,
                &mut needed
            ) != 0,
            "GetTokenInformation(TokenDefaultDacl) failed: {}",
            GetLastError()
        );
        let default_dacl = std::ptr::read_unaligned(buffer.as_ptr().cast::<TokenDefaultDaclInfo>());
        let mapping = GENERIC_MAPPING {
            GenericRead: READ_CONTROL | PROCESS_VM_READ | PROCESS_QUERY_INFORMATION,
            GenericWrite: READ_CONTROL | PROCESS_VM_WRITE | PROCESS_VM_OPERATION,
            GenericExecute: READ_CONTROL,
            GenericAll: PROCESS_ALL_ACCESS,
        };
        // Object creation maps the token default DACL's generic ACE masks to
        // object-specific rights. AccessCheck expects that mapping already done.
        for index in 0..u32::from((*default_dacl.default_dacl).AceCount) {
            let mut ace = std::ptr::null_mut();
            ensure!(GetAce(default_dacl.default_dacl, index, &mut ace) != 0);
            MapGenericMask(&mut (*ace.cast::<ACCESS_ALLOWED_ACE>()).Mask, &mapping);
        }
        let mut owner = get_user_sid_bytes(base_raw)?;
        let mut descriptor: SECURITY_DESCRIPTOR = std::mem::zeroed();
        let sd = std::ptr::addr_of_mut!(descriptor).cast();
        ensure!(InitializeSecurityDescriptor(sd, 1) != 0);
        ensure!(SetSecurityDescriptorOwner(sd, owner.as_mut_ptr().cast(), 0) != 0);
        ensure!(SetSecurityDescriptorGroup(sd, owner.as_mut_ptr().cast(), 0) != 0);
        ensure!(SetSecurityDescriptorDacl(sd, 1, default_dacl.default_dacl, 0) != 0);

        // Model another logon of the same account without the victim's logon
        // SID. Build from the base token, retaining the shared user, Everyone,
        // and filesystem capability in the write-restricting SID set.
        let mut logon = get_logon_sid_bytes(base_raw)?;
        let mut everyone = world_sid()?;
        let disabled = SID_AND_ATTRIBUTES {
            Sid: logon.as_mut_ptr().cast(),
            Attributes: 0,
        };
        let restricting = [
            capability.as_ptr(),
            owner.as_mut_ptr().cast(),
            everyone.as_mut_ptr().cast(),
        ]
        .map(|sid| SID_AND_ATTRIBUTES {
            Sid: sid,
            Attributes: 0,
        });
        let mut other = std::ptr::null_mut();
        ensure!(
            CreateRestrictedToken(
                base_raw,
                DISABLE_MAX_PRIVILEGE | LUA_TOKEN | WRITE_RESTRICTED,
                1,
                &disabled,
                0,
                std::ptr::null(),
                restricting.len() as u32,
                restricting.as_ptr(),
                &mut other
            ) != 0,
            "CreateRestrictedToken(other logon) failed: {}",
            GetLastError()
        );
        let other = OwnedHandle::from_raw_handle(other as _);
        for (token, same_logon) in [(&victim, true), (&other, false)] {
            let mut impersonation = std::ptr::null_mut();
            ensure!(
                DuplicateToken(
                    token.as_raw_handle() as _,
                    SecurityImpersonation,
                    &mut impersonation
                ) != 0,
                "DuplicateToken failed: {}",
                GetLastError()
            );
            let impersonation = OwnedHandle::from_raw_handle(impersonation as _);
            for access in [
                PROCESS_VM_READ,
                PROCESS_VM_WRITE,
                PROCESS_VM_OPERATION,
                PROCESS_CREATE_THREAD,
                PROCESS_TERMINATE,
                PROCESS_QUERY_INFORMATION,
                WRITE_DAC,
                0x43a,
            ] {
                let mut privileges = [0u64; 128];
                let mut privilege_bytes = std::mem::size_of_val(&privileges) as u32;
                let mut granted = 0;
                let mut allowed = 0;
                ensure!(
                    AccessCheck(
                        sd,
                        impersonation.as_raw_handle() as _,
                        access,
                        &mapping,
                        privileges.as_mut_ptr().cast(),
                        &mut privilege_bytes,
                        &mut granted,
                        &mut allowed
                    ) != 0,
                    "AccessCheck failed: {}",
                    GetLastError()
                );
                assert_eq!(allowed != 0, same_logon, "access {access:#x}");
            }
        }
    }
    Ok(())
}

/// D324: the public elevated token builders refuse a base token outside the
/// sandbox users' group, as the test process is.
#[test]
fn elevated_token_builders_refuse_a_caller_outside_the_sandbox_users_group() -> Result<()> {
    let capability_sid = LocalSid::from_string("S-1-5-21-10-20-30-40")?;
    let network_proxy_sid = LocalSid::from_string("S-1-5-21-50-60-70-80")?;
    let base_token = unsafe { get_current_token_for_restriction()? };
    let results = unsafe {
        [
            create_readonly_token_with_caps_and_user_from(
                base_token,
                &[capability_sid.as_ptr()],
                &[network_proxy_sid.as_ptr()],
            ),
            create_workspace_write_token_with_caps_and_user_from(
                base_token,
                &[capability_sid.as_ptr()],
                &[network_proxy_sid.as_ptr()],
            ),
        ]
    };
    let mut refusals = Vec::new();
    for result in results {
        match result {
            Ok(token) => {
                unsafe { CloseHandle(token) };
                refusals.push(None);
            }
            Err(err) => refusals.push(Some(format!("{err:#}"))),
        }
    }
    unsafe { CloseHandle(base_token) };

    for refusal in refusals {
        let refusal = refusal.expect("a token for a caller outside CodexSandboxUsers");
        assert!(refusal.contains(SANDBOX_USERS_GROUP), "{refusal}");
    }
    Ok(())
}

/// The SID string and access mask of each entry of `token`'s default DACL.
unsafe fn default_dacl_entries(token: HANDLE) -> Result<Vec<(String, u32)>> {
    let mut needed = 0;
    GetTokenInformation(
        token,
        TokenDefaultDacl,
        std::ptr::null_mut(),
        0,
        &mut needed,
    );
    let mut buffer = vec![0_usize; (needed as usize).div_ceil(std::mem::size_of::<usize>())];
    if needed == 0
        || GetTokenInformation(
            token,
            TokenDefaultDacl,
            buffer.as_mut_ptr().cast(),
            needed,
            &mut needed,
        ) == 0
    {
        return Err(anyhow!(
            "GetTokenInformation(TokenDefaultDacl) failed: {}",
            GetLastError()
        ));
    }
    let acl = (*buffer.as_ptr().cast::<TokenDefaultDaclInfo>()).default_dacl;
    let mut entries = Vec::new();
    for index in 0..u32::from((*acl).AceCount) {
        let mut ace: *mut c_void = std::ptr::null_mut();
        if GetAce(acl, index, &mut ace) == 0 {
            return Err(anyhow!("GetAce failed: {}", GetLastError()));
        }
        let ace = ace.cast::<ACCESS_ALLOWED_ACE>();
        if u32::from((*ace).Header.AceType) != ACCESS_ALLOWED_ACE_TYPE {
            return Err(anyhow!("unexpected entry type {}", (*ace).Header.AceType));
        }
        let sid = std::ptr::addr_of_mut!((*ace).SidStart).cast::<c_void>();
        let sid = std::slice::from_raw_parts(sid.cast::<u8>(), GetLengthSid(sid) as usize);
        let sid = crate::winutil::string_from_sid_bytes(sid).map_err(|err| anyhow!(err))?;
        entries.push((sid, (*ace).Mask));
    }
    Ok(entries)
}

/// D429 (#1803): a legacy command's default DACL gives Everyone, the logon
/// SID and the capability SIDs `SYNCHRONIZE` alone (before, read and
/// execute: `PROCESS_VM_READ` and `THREAD_GET_CONTEXT` among them), and full
/// access to the user, SYSTEM and the command's own SID.
#[test]
fn d429_legacy_default_dacl_gives_shared_sids_only_synchronize() -> Result<()> {
    let capability = "S-1-5-21-10-20-30-40";
    let command = "S-1-5-21-50-60-70-80";
    let capability_sid = LocalSid::from_string(capability)?;
    let command_sid = LocalSid::from_string(command)?;
    let base_token = unsafe { get_current_token_for_restriction()? };
    let user = unsafe { get_user_sid_bytes(base_token) }
        .and_then(|sid| crate::winutil::string_from_sid_bytes(&sid).map_err(|err| anyhow!(err)));
    let logon = unsafe { get_logon_sid_bytes(base_token) }
        .and_then(|sid| crate::winutil::string_from_sid_bytes(&sid).map_err(|err| anyhow!(err)));
    let token = unsafe {
        create_legacy_token_with_caps_from(
            base_token,
            &[capability_sid.as_ptr()],
            command_sid.as_ptr(),
        )
    };
    let entries = token
        .as_ref()
        .map_err(|err| anyhow!("{err:#}"))
        .and_then(|token| unsafe { default_dacl_entries(*token) });
    unsafe {
        if let Ok(token) = token {
            CloseHandle(token);
        }
        CloseHandle(base_token);
    }
    let mut entries = entries?;
    let mut expected = vec![
        (user?, GENERIC_ALL),
        ("S-1-5-18".to_string(), GENERIC_ALL),
        (command.to_string(), GENERIC_ALL),
        (logon?, SYNCHRONIZE),
        ("S-1-1-0".to_string(), SYNCHRONIZE),
        (capability.to_string(), SYNCHRONIZE),
    ];
    entries.sort();
    expected.sort();
    pretty_assertions::assert_eq!(entries, expected);
    Ok(())
}
