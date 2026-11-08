use super::*;
use windows_sys::Win32::Security::ACCESS_ALLOWED_ACE;
use windows_sys::Win32::Security::EqualSid;
use windows_sys::Win32::Security::GetAce;
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
            ElevatedBaseLogon::OwnLogon,
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
                ElevatedBaseLogon::OwnLogon,
                &[capability_sid.as_ptr()],
                &[network_proxy_sid.as_ptr()],
            ),
            create_workspace_write_token_with_caps_and_user_from(
                base_token,
                ElevatedBaseLogon::RunnerLogon,
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

/// D392 (b): an elevated command's default DACL follows whose logon its base
/// token is. On a logon of its own (#1721 (a′)), the `pl` DACL: full access
/// for SYSTEM, that logon SID and the command SID only. On the runner's own
/// logon, which every runner Core starts shares (a standard user's setup,
/// no runner account), the D324 DACL as before #1800: full access for
/// SYSTEM, the sandbox users' group and the command SID, read and execute
/// for the shared SIDs. The `pl` DACL there would give every elevated
/// command full access to every other's objects through the shared logon
/// SID.
#[test]
fn d392_elevated_default_dacl_follows_whose_logon_the_base_token_is() -> Result<()> {
    let capability = "S-1-5-21-10-20-30-40";
    let capability_sid = LocalSid::from_string(capability)?;
    let group_sid = LocalSid::from_string(TEST_GROUP_SID)?;
    let base_token = unsafe { get_current_token_for_restriction()? };
    let sid_string = |sid: Result<Vec<u8>>| {
        sid.and_then(|sid| crate::winutil::string_from_sid_bytes(&sid).map_err(|err| anyhow!(err)))
    };
    let user = sid_string(unsafe { get_user_sid_bytes(base_token) });
    let logon = sid_string(unsafe { get_logon_sid_bytes(base_token) });
    let dacl = |base_logon: ElevatedBaseLogon| -> Result<Vec<(String, u32)>> {
        let token = unsafe {
            create_elevated_token_with_group(
                base_token,
                base_logon,
                &[capability_sid.as_ptr()],
                &[],
                group_sid.as_ptr(),
            )
        }?;
        let entries = unsafe { default_dacl_entries(token) };
        unsafe { CloseHandle(token) };
        entries
    };
    let own = dacl(ElevatedBaseLogon::OwnLogon);
    let runner = dacl(ElevatedBaseLogon::RunnerLogon);
    unsafe { CloseHandle(base_token) };
    let (user, logon) = (user?, logon?);
    // The command SID is new for each token: name it.
    let known = [
        user.as_str(),
        logon.as_str(),
        capability,
        TEST_GROUP_SID,
        "S-1-1-0",
        "S-1-3-4",
        "S-1-5-18",
    ];
    let named = |entries: Vec<(String, u32)>| {
        let mut entries: Vec<(String, u32)> = entries
            .into_iter()
            .map(|(sid, mask)| {
                if known.contains(&sid.as_str()) {
                    (sid, mask)
                } else {
                    ("command".to_string(), mask)
                }
            })
            .collect();
        entries.sort();
        entries
    };
    let sorted = |mut entries: Vec<(String, u32)>| {
        entries.sort();
        entries
    };
    let read_execute = GENERIC_READ | GENERIC_EXECUTE;
    pretty_assertions::assert_eq!(
        (named(own?), named(runner?)),
        (
            sorted(vec![
                ("S-1-5-18".to_string(), GENERIC_ALL),
                (logon.clone(), GENERIC_ALL),
                ("command".to_string(), GENERIC_ALL),
                (TEST_GROUP_SID.to_string(), ELEVATED_PEER_ACCESS),
                ("S-1-1-0".to_string(), ELEVATED_PEER_ACCESS),
                ("S-1-3-4".to_string(), READ_CONTROL),
            ]),
            sorted(vec![
                ("S-1-5-18".to_string(), GENERIC_ALL),
                (TEST_GROUP_SID.to_string(), GENERIC_ALL),
                ("command".to_string(), GENERIC_ALL),
                (logon, read_execute),
                ("S-1-3-4".to_string(), read_execute),
                (user, read_execute),
                ("S-1-1-0".to_string(), read_execute),
                (capability.to_string(), read_execute),
            ]),
        )
    );
    Ok(())
}

/// D392: a runner makes the command's token from the command's own logon
/// when Core sent one, and from its own token only when it doesn't hold
/// `SeAssignPrimaryTokenPrivilege` (a runner running as the runner account
/// refuses a request without a command logon).
#[test]
fn d392_runner_uses_its_own_token_only_without_the_privilege() {
    let decide = |logon, privilege: bool| {
        ElevatedBaseLogon::for_request(logon, || Ok(privilege)).map_err(|err| format!("{err:#}"))
    };
    pretty_assertions::assert_eq!(
        [
            decide(true, true),
            decide(true, false),
            decide(false, false),
            decide(false, true),
        ],
        [
            Ok(ElevatedBaseLogon::OwnLogon),
            Ok(ElevatedBaseLogon::OwnLogon),
            Ok(ElevatedBaseLogon::RunnerLogon),
            Err("runner: Core sent no logon token for the command (#1721)".to_string()),
        ]
    );
    // With a command logon, the runner's own token isn't queried at all.
    let unasked = ElevatedBaseLogon::for_request(true, || anyhow::bail!("queried"));
    pretty_assertions::assert_eq!(
        unasked.map_err(|err| format!("{err:#}")),
        Ok(ElevatedBaseLogon::OwnLogon)
    );
}
