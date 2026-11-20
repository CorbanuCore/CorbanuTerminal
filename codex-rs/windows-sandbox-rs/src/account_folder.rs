//! #2819: the LocalAppData folder of the account this process runs as.
//!
//! `SHGetKnownFolderPath(FOLDERID_LocalAppData)` expands the shell's
//! `%USERPROFILE%\AppData\Local` with this process's environment, even when
//! given the process token, so a changed `USERPROFILE` moved the planted-folder
//! records to an empty folder. This reads the same `User Shell Folders` value
//! the shell does and expands it with `ExpandEnvironmentStringsForUserW` on
//! the process token, which builds the account's own environment from its
//! profile and ignores this process's.

use std::path::PathBuf;
use windows_sys::Win32::Foundation::CloseHandle;
use windows_sys::Win32::Foundation::ERROR_SUCCESS;
use windows_sys::Win32::System::Environment::ExpandEnvironmentStringsForUserW;
use windows_sys::Win32::System::Registry::HKEY_CURRENT_USER;
use windows_sys::Win32::System::Registry::RRF_NOEXPAND;
use windows_sys::Win32::System::Registry::RRF_RT_REG_EXPAND_SZ;
use windows_sys::Win32::System::Registry::RRF_RT_REG_SZ;
use windows_sys::Win32::System::Registry::RegGetValueW;

const USER_SHELL_FOLDERS: &str =
    r"Software\Microsoft\Windows\CurrentVersion\Explorer\User Shell Folders";
const LOCAL_APP_DATA_VALUE: &str = "Local AppData";
/// The shell's default when the account has no `Local AppData` value.
const DEFAULT_LOCAL_APP_DATA: &str = r"%USERPROFILE%\AppData\Local";
const MAX_CHARS: usize = 32_768;

/// The account's LocalAppData, as the shell resolves it for an unchanged
/// environment. `None` if the token can't be read or expansion fails.
pub(crate) fn local_app_data() -> Option<PathBuf> {
    let template =
        registered_local_app_data().unwrap_or_else(|| DEFAULT_LOCAL_APP_DATA.to_string());
    expand_for_account(&template).map(PathBuf::from)
}

/// The unexpanded `Local AppData` value from the account's own hive.
fn registered_local_app_data() -> Option<String> {
    let subkey = crate::winutil::to_wide(USER_SHELL_FOLDERS);
    let value = crate::winutil::to_wide(LOCAL_APP_DATA_VALUE);
    let mut buffer = vec![0u16; MAX_CHARS];
    let mut bytes = (buffer.len() * std::mem::size_of::<u16>()) as u32;
    // SAFETY: the strings are NUL-terminated and `bytes` is `buffer`'s size.
    let status = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            subkey.as_ptr(),
            value.as_ptr(),
            RRF_RT_REG_SZ | RRF_RT_REG_EXPAND_SZ | RRF_NOEXPAND,
            std::ptr::null_mut(),
            buffer.as_mut_ptr().cast(),
            &mut bytes,
        )
    };
    if status != ERROR_SUCCESS {
        return None;
    }
    let text = from_wide_nul(&buffer[..(bytes as usize / 2).min(buffer.len())]);
    (!text.trim().is_empty()).then_some(text)
}

/// `template` with the account's environment variables, not this process's.
fn expand_for_account(template: &str) -> Option<String> {
    // SAFETY: the token is closed below.
    let token = unsafe { crate::token::get_current_token_for_query() }.ok()?;
    let source = crate::winutil::to_wide(template);
    let mut buffer = vec![0u16; MAX_CHARS];
    // SAFETY: `source` is NUL-terminated and `buffer` holds `MAX_CHARS`.
    let ok = unsafe {
        ExpandEnvironmentStringsForUserW(
            token,
            source.as_ptr(),
            buffer.as_mut_ptr(),
            MAX_CHARS as u32,
        )
    };
    // SAFETY: opened above.
    unsafe { CloseHandle(token) };
    let expanded = from_wide_nul(&buffer);
    (ok != 0 && !expanded.is_empty()).then_some(expanded)
}

fn from_wide_nul(wide: &[u16]) -> String {
    let len = wide.iter().position(|&c| c == 0).unwrap_or(wide.len());
    String::from_utf16_lossy(&wide[..len])
}

#[cfg(test)]
#[path = "account_folder_tests.rs"]
mod tests;
