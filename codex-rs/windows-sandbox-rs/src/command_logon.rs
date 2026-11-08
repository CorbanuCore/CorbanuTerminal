//! #1721 (a′) and #1560 (`pl`): each elevated command runs on a logon of its
//! own, made by Core, and the command runner runs as an account no command
//! ever holds ([`crate::setup::RUNNER_USERNAME`]).
//!
//! Before (a′) the runner ran as the command's sandbox account and derived
//! the command's token from its own, so while a runner started (before it
//! restricted itself) every elevated command held what its process, threads
//! and token granted (#1721), and the runners Core started from one logon
//! session shared their logon SID, so no SID named one command (#1560).
//!
//! Now Core logs the command's account (offline or online) on for each
//! command (`LogonUserW`), gives that logon what a command needs to start on
//! this window station, desktop and session namespace
//! ([`WindowAccess::grant_command_logon`]), hands the runner the token
//! (the access it had to its own token, [`RUNNER_TOKEN_ACCESS`]), and a reaper removes those entries once the
//! runner exits. The runner restricts that token, as it did its own, and
//! starts the command with it, which takes `SeAssignPrimaryTokenPrivilege`
//! (setup grants it to the runner's account, D392). The runner enables the
//! privilege only to start its one command and then removes it from its
//! token, and from the token of the ConPTY console host it starts (a copy of
//! its own, made before the command starts), failing if either still holds
//! it. A command's default DACL then gives full access only to its own logon
//! SID, its command SID and SYSTEM (`crate::token`).

use crate::identity::SandboxCreds;
use crate::logon_launch::LogonError;
use crate::logon_launch::WindowAccessReaper;
use crate::token::SE_PRIVILEGE_ENABLED;
use crate::token::SE_PRIVILEGE_REMOVED;
use crate::token::get_logon_sid_bytes;
use crate::token::set_single_privilege;
use crate::window_station::WindowAccess;
use crate::winutil::to_wide;
use anyhow::Context;
use anyhow::Result;
use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use windows_sys::Win32::Foundation::CloseHandle;
use windows_sys::Win32::Foundation::DuplicateHandle;
use windows_sys::Win32::Foundation::GetLastError;
use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::Security::LOGON32_LOGON_INTERACTIVE;
use windows_sys::Win32::Security::LOGON32_PROVIDER_DEFAULT;
use windows_sys::Win32::Security::LogonUserW;
use windows_sys::Win32::Security::TOKEN_ADJUST_DEFAULT;
use windows_sys::Win32::Security::TOKEN_ADJUST_PRIVILEGES;
use windows_sys::Win32::Security::TOKEN_ADJUST_SESSIONID;
use windows_sys::Win32::Security::TOKEN_ASSIGN_PRIMARY;
use windows_sys::Win32::Security::TOKEN_DUPLICATE;
use windows_sys::Win32::Security::TOKEN_QUERY;
use windows_sys::Win32::System::Threading::GetCurrentProcess;

/// The privilege the runner needs to start a command with a token it did
/// not derive from its own.
pub const ASSIGN_PRIMARY_TOKEN_PRIVILEGE: &str = "SeAssignPrimaryTokenPrivilege";

/// The access the runner gets to the command's logon token: what it had to
/// its own token for the same job (`get_current_token_for_restriction`).
/// `CreateRestrictedToken` gives the restricted token the same access, and
/// the runner sets its default DACL and privileges and starts the command
/// with it. Not `TOKEN_IMPERSONATE`, `TOKEN_ADJUST_GROUPS` or `WRITE_DAC`.
const RUNNER_TOKEN_ACCESS: u32 = TOKEN_DUPLICATE
    | TOKEN_QUERY
    | TOKEN_ASSIGN_PRIMARY
    | TOKEN_ADJUST_DEFAULT
    | TOKEN_ADJUST_SESSIONID
    | TOKEN_ADJUST_PRIVILEGES;

/// What Core gave one command's logon, held until the runner exits.
pub(crate) enum CommandLogonAccess {
    /// A reaper removes it when the runner exits, even after Core.
    Reaped,
    /// No reaper could take it: Core removes it when the runner exits,
    /// unless Core exits first.
    Held(WindowAccess),
}

/// One command's logon, handed to its runner.
pub(crate) struct CommandLogon {
    /// The token's handle value in the runner's handle table.
    pub(crate) runner_token: u64,
    pub(crate) access: CommandLogonAccess,
}

/// Logs `creds` on, gives that logon what a command needs to start here
/// (on this process's desktop too when `commands_use_this_desktop`), hands
/// that access to a reaper of `runner` and the token
/// ([`RUNNER_TOKEN_ACCESS`]) to `runner`, Core's handle to the running runner.
pub(crate) fn hand_command_logon_to_runner(
    creds: &SandboxCreds,
    runner: HANDLE,
    runner_exe: &Path,
    commands_use_this_desktop: bool,
    log_dir: Option<&Path>,
) -> Result<CommandLogon> {
    let token = OwnedToken(logon(creds)?);
    // SAFETY: a live token from `LogonUserW`.
    let logon_sid =
        unsafe { get_logon_sid_bytes(token.0) }.context("read the command's logon SID")?;
    let mut access =
        WindowAccess::grant_command_logon(&logon_sid, commands_use_this_desktop, log_dir)
            .context("give the command's logon this window station")?;
    access.set_log_dir(log_dir);
    let access = match WindowAccessReaper::start(access, runner, runner_exe) {
        Ok(_reaper) => CommandLogonAccess::Reaped,
        Err((access, err)) => {
            crate::logging::log_note(
                &format!(
                    "no reaper for the command logon's window access ({err:#}); Core removes it, unless it exits first"
                ),
                log_dir,
            );
            CommandLogonAccess::Held(access)
        }
    };
    let mut remote: HANDLE = 0;
    // SAFETY: both process handles are live; `remote` names a handle in the
    // runner, which owns it from here on.
    let ok = unsafe {
        DuplicateHandle(
            GetCurrentProcess(),
            token.0,
            runner,
            &mut remote,
            RUNNER_TOKEN_ACCESS,
            /*binherithandle*/ 0,
            /*dwoptions*/ 0,
        )
    };
    if ok == 0 {
        return Err(std::io::Error::last_os_error())
            .context("hand the command's token to the runner");
    }
    Ok(CommandLogon {
        runner_token: remote as u64,
        access,
    })
}

/// `LogonUserW(creds)`: a new interactive logon, with a logon SID of its own.
fn logon(creds: &SandboxCreds) -> Result<HANDLE> {
    let user = to_wide(&creds.username);
    let domain = to_wide(".");
    let password = to_wide(&creds.password);
    let mut token: HANDLE = 0;
    // SAFETY: NUL-terminated buffers that outlive the call.
    let ok = unsafe {
        LogonUserW(
            user.as_ptr(),
            domain.as_ptr(),
            password.as_ptr(),
            LOGON32_LOGON_INTERACTIVE,
            LOGON32_PROVIDER_DEFAULT,
            &mut token,
        )
    };
    if ok == 0 {
        // SAFETY: reads this thread's last error.
        let code = unsafe { GetLastError() };
        // A `LogonError` like the runner's own: a stale password reruns the
        // setup (`is_refreshable_sandbox_creds_error`).
        return Err(anyhow::Error::new(LogonError { code })).with_context(|| {
            format!(
                "LogonUserW: log the command's account {} on",
                creds.username
            )
        });
    }
    Ok(token)
}

struct OwnedToken(HANDLE);

impl Drop for OwnedToken {
    fn drop(&mut self) {
        // SAFETY: a handle this value owns.
        unsafe { CloseHandle(self.0) };
    }
}

/// Set in the runner once it has enabled [`ASSIGN_PRIMARY_TOKEN_PRIVILEGE`]:
/// the console hosts it starts from then on copy its token.
static RUNNER_HOLDS_PRIVILEGE: AtomicBool = AtomicBool::new(false);

/// In the runner: enables [`ASSIGN_PRIMARY_TOKEN_PRIVILEGE`] on `own_token`
/// (its own token, opened before it hardened itself).
pub fn enable_assign_primary_token_privilege(own_token: HANDLE) -> Result<()> {
    RUNNER_HOLDS_PRIVILEGE.store(true, Ordering::SeqCst);
    // SAFETY: a live token handle opened for `TOKEN_ADJUST_PRIVILEGES`.
    unsafe {
        set_single_privilege(
            own_token,
            ASSIGN_PRIMARY_TOKEN_PRIVILEGE,
            SE_PRIVILEGE_ENABLED,
        )
    }
}

/// In the runner, once its command has started (or failed to): removes
/// [`ASSIGN_PRIMARY_TOKEN_PRIVILEGE`] from `own_token` for good, so a runner
/// taken over later can't use it, and checks that it is gone.
pub fn remove_assign_primary_token_privilege(own_token: HANDLE) -> Result<()> {
    remove_privilege(own_token).context("remove SeAssignPrimaryTokenPrivilege from the runner")
}

/// Removes [`ASSIGN_PRIMARY_TOKEN_PRIVILEGE`] from `token` (opened for
/// `TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY`) and checks that it is gone. A
/// token without it is left as it is.
fn remove_privilege(token: HANDLE) -> Result<()> {
    // SAFETY: a live token handle with the access above.
    if !unsafe { token_holds_privilege(token, ASSIGN_PRIMARY_TOKEN_PRIVILEGE) }? {
        return Ok(());
    }
    // SAFETY: as above.
    unsafe { set_single_privilege(token, ASSIGN_PRIMARY_TOKEN_PRIVILEGE, SE_PRIVILEGE_REMOVED) }?;
    // SAFETY: as above.
    if unsafe { token_holds_privilege(token, ASSIGN_PRIMARY_TOKEN_PRIVILEGE) }? {
        anyhow::bail!("the token still holds {ASSIGN_PRIMARY_TOKEN_PRIVILEGE}");
    }
    Ok(())
}

/// In the runner, right after it created a pseudo console (before any
/// command can write to it): removes [`ASSIGN_PRIMARY_TOKEN_PRIVILEGE`] from
/// the token of each console host this process started. Those run as the
/// runner's account, read what the command writes to its console, and copied
/// the runner's token with the privilege in it. Fails unless at least one
/// was found and none still holds it. Does nothing outside such a runner.
pub(crate) fn strip_privilege_from_console_hosts() -> Result<()> {
    if !RUNNER_HOLDS_PRIVILEGE.load(Ordering::SeqCst) {
        return Ok(());
    }
    let hosts = console_hosts_of_this_process();
    if hosts.is_empty() {
        anyhow::bail!("found no console host to remove {ASSIGN_PRIMARY_TOKEN_PRIVILEGE} from");
    }
    for host in hosts {
        strip_process(host).with_context(|| {
            format!("remove {ASSIGN_PRIMARY_TOKEN_PRIVILEGE} from console host {host}")
        })?;
    }
    Ok(())
}

fn strip_process(pid: u32) -> Result<()> {
    use windows_sys::Win32::Security::TOKEN_ADJUST_PRIVILEGES;
    use windows_sys::Win32::System::Threading::OpenProcess;
    use windows_sys::Win32::System::Threading::OpenProcessToken;
    use windows_sys::Win32::System::Threading::PROCESS_QUERY_LIMITED_INFORMATION;
    // SAFETY: the handles are owned below and closed on drop.
    let process = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) };
    if process == 0 {
        return Err(std::io::Error::last_os_error()).context("open the process");
    }
    let process = OwnedToken(process);
    let mut token: HANDLE = 0;
    // SAFETY: a live process handle.
    if unsafe { OpenProcessToken(process.0, TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY, &mut token) }
        == 0
    {
        return Err(std::io::Error::last_os_error()).context("open its token");
    }
    let token = OwnedToken(token);
    remove_privilege(token.0)
}

/// The console hosts (`conhost.exe`, `OpenConsole.exe`) this process started.
fn console_hosts_of_this_process() -> Vec<u32> {
    use windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE;
    use windows_sys::Win32::System::Diagnostics::ToolHelp::CreateToolhelp32Snapshot;
    use windows_sys::Win32::System::Diagnostics::ToolHelp::PROCESSENTRY32W;
    use windows_sys::Win32::System::Diagnostics::ToolHelp::Process32FirstW;
    use windows_sys::Win32::System::Diagnostics::ToolHelp::Process32NextW;
    use windows_sys::Win32::System::Diagnostics::ToolHelp::TH32CS_SNAPPROCESS;
    let own = std::process::id();
    let mut hosts = Vec::new();
    // SAFETY: the snapshot handle is checked and closed here; the entry is
    // sized as the API requires.
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot == INVALID_HANDLE_VALUE {
            return hosts;
        }
        let mut entry: PROCESSENTRY32W = std::mem::zeroed();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
        let mut more = Process32FirstW(snapshot, &mut entry) != 0;
        while more {
            let len = entry
                .szExeFile
                .iter()
                .position(|&c| c == 0)
                .unwrap_or(entry.szExeFile.len());
            let image = String::from_utf16_lossy(&entry.szExeFile[..len]).to_lowercase();
            if entry.th32ParentProcessID == own
                && (image == "conhost.exe" || image == "openconsole.exe")
            {
                hosts.push(entry.th32ProcessID);
            }
            more = Process32NextW(snapshot, &mut entry) != 0;
        }
        CloseHandle(snapshot);
    }
    hosts
}

/// Whether `token` (opened for `TOKEN_QUERY`) holds the privilege `name`,
/// enabled or not.
pub(crate) unsafe fn token_holds_privilege(token: HANDLE, name: &str) -> Result<bool> {
    use windows_sys::Win32::Foundation::LUID;
    use windows_sys::Win32::Security::GetTokenInformation;
    use windows_sys::Win32::Security::LUID_AND_ATTRIBUTES;
    use windows_sys::Win32::Security::LookupPrivilegeValueW;
    use windows_sys::Win32::Security::TokenPrivileges;
    let mut luid = LUID {
        LowPart: 0,
        HighPart: 0,
    };
    let wide = to_wide(name);
    // SAFETY: a NUL-terminated name and a LUID to fill.
    if unsafe { LookupPrivilegeValueW(std::ptr::null(), wide.as_ptr(), &mut luid) } == 0 {
        return Err(std::io::Error::last_os_error()).context("look the privilege up");
    }
    let mut needed = 0u32;
    // SAFETY: a size query.
    unsafe { GetTokenInformation(token, TokenPrivileges, std::ptr::null_mut(), 0, &mut needed) };
    // `u64`s keep the buffer aligned for `TOKEN_PRIVILEGES`.
    let mut buffer = vec![0u64; (needed as usize).div_ceil(8).max(1)];
    // SAFETY: a buffer of at least `needed` bytes.
    if unsafe {
        GetTokenInformation(
            token,
            TokenPrivileges,
            buffer.as_mut_ptr().cast(),
            (buffer.len() * 8) as u32,
            &mut needed,
        )
    } == 0
    {
        return Err(std::io::Error::last_os_error()).context("read the token's privileges");
    }
    // SAFETY: `TOKEN_PRIVILEGES`: a count, then that many entries (4-byte aligned).
    let held = unsafe {
        let base = buffer.as_ptr().cast::<u8>();
        let count = std::ptr::read_unaligned(base.cast::<u32>()) as usize;
        let entries = base.add(4).cast::<LUID_AND_ATTRIBUTES>();
        (0..count).any(|index| {
            let entry = std::ptr::read_unaligned(entries.add(index));
            entry.Luid.LowPart == luid.LowPart && entry.Luid.HighPart == luid.HighPart
        })
    };
    Ok(held)
}
