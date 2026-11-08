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
//! (query and duplicate only), and a reaper removes those entries once the
//! runner exits. The runner restricts that token, as it did its own, and
//! starts the command with it, which takes `SeAssignPrimaryTokenPrivilege`
//! (setup grants it to the runner's account, D392). The runner enables the
//! privilege only to start its one command and then removes it from its
//! token. A command's default DACL then gives full access only to its own
//! logon SID, its command SID and SYSTEM (`crate::token`).

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
use windows_sys::Win32::Foundation::CloseHandle;
use windows_sys::Win32::Foundation::DuplicateHandle;
use windows_sys::Win32::Foundation::GetLastError;
use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::Security::LOGON32_LOGON_INTERACTIVE;
use windows_sys::Win32::Security::LOGON32_PROVIDER_DEFAULT;
use windows_sys::Win32::Security::LogonUserW;
use windows_sys::Win32::Security::TOKEN_DUPLICATE;
use windows_sys::Win32::Security::TOKEN_QUERY;
use windows_sys::Win32::System::Threading::GetCurrentProcess;

/// The privilege the runner needs to start a command with a token it did
/// not derive from its own.
pub const ASSIGN_PRIMARY_TOKEN_PRIVILEGE: &str = "SeAssignPrimaryTokenPrivilege";

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
/// that access to a reaper of `runner` and the token (query and duplicate
/// only) to `runner`, Core's handle to the running runner.
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
            TOKEN_DUPLICATE | TOKEN_QUERY,
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

/// In the runner: enables [`ASSIGN_PRIMARY_TOKEN_PRIVILEGE`] on `own_token`
/// (its own token, opened before it hardened itself).
pub fn enable_assign_primary_token_privilege(own_token: HANDLE) -> Result<()> {
    // SAFETY: a live token handle opened for `TOKEN_ADJUST_PRIVILEGES`.
    unsafe {
        set_single_privilege(
            own_token,
            ASSIGN_PRIMARY_TOKEN_PRIVILEGE,
            SE_PRIVILEGE_ENABLED,
        )
    }
}

/// In the runner, once its command has started: removes
/// [`ASSIGN_PRIMARY_TOKEN_PRIVILEGE`] from `own_token` for good, so a runner
/// taken over later can't use it.
pub fn remove_assign_primary_token_privilege(own_token: HANDLE) -> Result<()> {
    // SAFETY: as above.
    unsafe {
        set_single_privilege(
            own_token,
            ASSIGN_PRIMARY_TOKEN_PRIVILEGE,
            SE_PRIVILEGE_REMOVED,
        )
    }
}
