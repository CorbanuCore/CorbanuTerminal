//! TUI-owned Windows sandbox helpers retained while setup still runs in the local client process.
//!
//! TODO: These helpers inspect and modify the TUI host, so they do not support
//! cross-platform remote app servers. Move readiness and setup to the existing
//! `windowsSandbox/*` RPCs while preserving the pending permission profile,
//! use the server platform reported during initialization, and add a remote
//! equivalent for read-root grants.

use crate::legacy_core::config::Config;
use codex_config::types::WindowsSandboxModeToml;
use codex_features::Feature;
use codex_protocol::config_types::WindowsSandboxLevel;
#[cfg(target_os = "windows")]
use codex_protocol::models::PermissionProfile;
#[cfg(target_os = "windows")]
use codex_utils_absolute_path::AbsolutePathBuf;
#[cfg(target_os = "windows")]
use std::collections::HashMap;
use std::path::Path;
#[cfg(target_os = "windows")]
use std::path::PathBuf;

pub(crate) fn level_from_config(config: &Config) -> WindowsSandboxLevel {
    match config.permissions.windows_sandbox_mode {
        Some(WindowsSandboxModeToml::Elevated) => WindowsSandboxLevel::Elevated,
        Some(WindowsSandboxModeToml::Unelevated) => WindowsSandboxLevel::RestrictedToken,
        None if config.features.enabled(Feature::WindowsSandboxElevated) => {
            WindowsSandboxLevel::Elevated
        }
        None if config.features.enabled(Feature::WindowsSandbox) => {
            WindowsSandboxLevel::RestrictedToken
        }
        None => WindowsSandboxLevel::Disabled,
    }
}

#[cfg(target_os = "windows")]
pub(crate) use codex_windows_sandbox::sandbox_setup_is_complete;

#[cfg(not(target_os = "windows"))]
pub(crate) fn sandbox_setup_is_complete(_codex_home: &Path) -> bool {
    false
}

/// #691 (Travis, 2026-10-14): on Windows, `/security` offers Aggressive only
/// once the default (elevated) Windows sandbox is set up. Without it,
/// credential files created later stay readable to agent commands and
/// workspace writes fall back to read-only; the non-admin sandbox cannot deny
/// reads of the vault and sign-in files. #783 (Travis, 2026-10-14): Moderate
/// is gated the same way until its protected-file rules are checked under the
/// non-admin sandbox. How to set the sandbox up (the picker words the note
/// around it); `None` once it is set up.
#[cfg(target_os = "windows")]
pub(crate) fn protected_level_setup_steps(config: &Config) -> Option<&'static str> {
    setup_steps(
        level_from_config(config),
        sandbox_setup_is_complete(config.codex_home.as_path()),
    )
}

/// `protected_level_setup_steps` for a Windows sandbox `level` whose elevated
/// setup is (or is not) `setup_complete`.
fn setup_steps(level: WindowsSandboxLevel, setup_complete: bool) -> Option<&'static str> {
    match level {
        WindowsSandboxLevel::Elevated if setup_complete => None,
        WindowsSandboxLevel::Elevated => Some(
            "The default Windows sandbox is selected, but its setup has not finished. It runs \
             (with an Administrator prompt) the first time an agent command runs in the \
             sandbox; let it finish",
        ),
        WindowsSandboxLevel::RestrictedToken => Some(
            "This session uses the non-admin Windows sandbox, which cannot keep agent commands \
             from reading the vault and sign-in files. Run /setup-default-sandbox (it needs \
             Administrator permission)",
        ),
        WindowsSandboxLevel::Disabled => Some(
            "The Windows sandbox is not set up. Open /permissions, choose Default and pick \
             \"Set up default sandbox\" (it needs Administrator permission)",
        ),
    }
}

#[cfg(target_os = "windows")]
pub(crate) fn run_elevated_setup(
    permission_profile: &PermissionProfile,
    workspace_roots: &[AbsolutePathBuf],
    command_cwd: &Path,
    env_map: &HashMap<String, String>,
    codex_home: &Path,
) -> anyhow::Result<()> {
    let permissions = codex_windows_sandbox::ResolvedWindowsSandboxPermissions::try_from_permission_profile_for_workspace_roots(
        permission_profile,
        workspace_roots,
    )?;
    codex_windows_sandbox::run_elevated_setup(
        codex_windows_sandbox::SandboxSetupRequest {
            permissions: &permissions,
            command_cwd,
            env_map,
            codex_home,
            proxy_enforced: false,
        },
        codex_windows_sandbox::SetupRootOverrides::default(),
    )
}

#[cfg(target_os = "windows")]
pub(crate) fn elevated_setup_failure_details(err: &anyhow::Error) -> Option<(String, String)> {
    let failure = codex_windows_sandbox::extract_setup_failure(err)?;
    Some((
        failure.code.as_str().to_string(),
        codex_windows_sandbox::sanitize_setup_metric_tag_value(&failure.message),
    ))
}

#[cfg(target_os = "windows")]
pub(crate) fn elevated_setup_failure_metric_name(err: &anyhow::Error) -> &'static str {
    if codex_windows_sandbox::extract_setup_failure(err).is_some_and(|failure| {
        matches!(
            failure.code,
            codex_windows_sandbox::SetupErrorCode::OrchestratorHelperLaunchCanceled
        )
    }) {
        "codex.windows_sandbox.elevated_setup_canceled"
    } else {
        "codex.windows_sandbox.elevated_setup_failure"
    }
}

#[cfg(target_os = "windows")]
pub(crate) fn grant_read_root_non_elevated(
    permission_profile: &PermissionProfile,
    workspace_roots: &[AbsolutePathBuf],
    command_cwd: &Path,
    env_map: &HashMap<String, String>,
    codex_home: &Path,
    read_root: &Path,
) -> anyhow::Result<PathBuf> {
    if !read_root.is_absolute() {
        anyhow::bail!("path must be absolute: {}", read_root.display());
    }
    if !read_root.exists() {
        anyhow::bail!("path does not exist: {}", read_root.display());
    }
    if !read_root.is_dir() {
        anyhow::bail!("path must be a directory: {}", read_root.display());
    }

    let canonical_root = dunce::canonicalize(read_root)?;
    codex_windows_sandbox::run_setup_refresh_with_extra_read_roots(
        permission_profile,
        workspace_roots,
        command_cwd,
        env_map,
        codex_home,
        vec![canonical_root.clone()],
        /*proxy_enforced*/ false,
    )?;
    Ok(canonical_root)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn issue_691_783_protected_levels_need_the_elevated_sandbox_set_up() {
        assert_eq!(
            setup_steps(WindowsSandboxLevel::Elevated, /*setup_complete*/ true),
            None
        );
        let not_set_up =
            setup_steps(WindowsSandboxLevel::Disabled, /*setup_complete*/ true).unwrap();
        assert!(not_set_up.contains("/permissions"), "{not_set_up}");
        let non_admin = setup_steps(
            WindowsSandboxLevel::RestrictedToken,
            /*setup_complete*/ true,
        )
        .unwrap();
        assert!(non_admin.contains("/setup-default-sandbox"), "{non_admin}");
        let unfinished =
            setup_steps(WindowsSandboxLevel::Elevated, /*setup_complete*/ false).unwrap();
        assert!(
            unfinished.contains("setup has not finished"),
            "{unfinished}"
        );
        for steps in [not_set_up, non_admin, unfinished] {
            assert!(!steps.ends_with('.'), "{steps}");
        }
    }
}
