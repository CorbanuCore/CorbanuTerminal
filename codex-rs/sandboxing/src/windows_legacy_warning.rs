//! User-facing warning for sessions whose sandboxed commands run under the
//! legacy (non-admin, restricted-token) Windows sandbox backend.
//!
//! That backend cannot stop a sandboxed command from deleting or moving files
//! outside its writable roots when the parent folder grants the user
//! `FILE_DELETE_CHILD`, which covers most of the user profile (issue #158).
//! The elevated backend is not affected, so the warning tells users how to
//! switch to it. It never changes how commands are sandboxed.

use codex_protocol::config_types::WindowsSandboxLevel;
use codex_protocol::models::PermissionProfile;

use crate::permission_profile_supports_windows_restricted_token_sandbox;

/// Warning shown once per session when commands run under the legacy backend.
/// The link points at the shipped guidance in `docs/sandbox.md`.
pub const LEGACY_WINDOWS_SANDBOX_WARNING: &str = concat!(
    "Corbanu Terminal is using the non-admin Windows sandbox. ",
    "In this mode, sandboxed commands can delete or move files outside your workspace ",
    "in folders you own, such as most of your user profile. ",
    "Switch to the default sandbox (one-time Administrator setup) by running ",
    "/setup-default-sandbox, or set `sandbox = \"elevated\"` under `[windows]` in config.toml. ",
    "Learn more: ",
    "https://github.com/CorbanuCore/CorbanuTerminal/blob/main/docs/sandbox.md#windows-non-admin-sandbox",
);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HostPlatform {
    Windows,
    Other,
}

impl HostPlatform {
    fn current() -> Self {
        if cfg!(target_os = "windows") {
            Self::Windows
        } else {
            Self::Other
        }
    }
}

/// Returns the legacy-backend warning when this host will run sandboxed
/// commands for `permission_profile` under the legacy Windows backend.
///
/// `windows_sandbox_level` is the level that selects the backend:
/// `RestrictedToken` is the legacy backend, `Elevated` is not affected, and
/// `Disabled` means sessions do not use a Windows sandbox at all.
pub fn legacy_windows_sandbox_warning(
    windows_sandbox_level: WindowsSandboxLevel,
    permission_profile: &PermissionProfile,
) -> Option<String> {
    legacy_windows_sandbox_warning_for_host(
        HostPlatform::current(),
        windows_sandbox_level,
        permission_profile,
    )
}

fn legacy_windows_sandbox_warning_for_host(
    host: HostPlatform,
    windows_sandbox_level: WindowsSandboxLevel,
    permission_profile: &PermissionProfile,
) -> Option<String> {
    let uses_legacy_backend = match windows_sandbox_level {
        WindowsSandboxLevel::RestrictedToken => true,
        WindowsSandboxLevel::Elevated | WindowsSandboxLevel::Disabled => false,
    };
    (host == HostPlatform::Windows
        && uses_legacy_backend
        && permission_profile_supports_windows_restricted_token_sandbox(permission_profile))
    .then(|| LEGACY_WINDOWS_SANDBOX_WARNING.to_string())
}

#[cfg(test)]
#[path = "windows_legacy_warning_tests.rs"]
mod tests;
