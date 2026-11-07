//! Windows sandbox configuration, managed requirements, and executor selection for the TUI.

use codex_app_server_client::AppServerRequestHandle;
use codex_app_server_protocol::ClientRequest;
use codex_app_server_protocol::ConfigReadResponse;
use codex_app_server_protocol::ConfigRequirementsReadResponse;
use codex_app_server_protocol::RequestId;
use codex_app_server_protocol::WindowsSandboxImplementation;
use codex_app_server_protocol::WindowsSandboxSetupMode;
use codex_protocol::config_types::WindowsSandboxLevel;
use uuid::Uuid;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct WindowsSandboxConfig {
    pub(crate) mxc_selected: bool,
    pub(crate) mode: Option<WindowsSandboxSetupMode>,
    // None means policy has not been loaded; a loaded null list allows both modes.
    pub(crate) requirements: Option<Option<Vec<WindowsSandboxSetupMode>>>,
}

#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
impl WindowsSandboxConfig {
    pub(crate) fn from_responses(
        config: &ConfigReadResponse,
        requirements: ConfigRequirementsReadResponse,
    ) -> Self {
        let configured_sandbox = config
            .config
            .additional
            .get("windows")
            .and_then(|windows| windows.get("sandbox"));
        let mxc_selected = configured_sandbox
            .and_then(|implementation| serde_json::from_value(implementation.clone()).ok())
            == Some(WindowsSandboxImplementation::Mxc);
        let mut state = Self {
            mxc_selected,
            mode: configured_sandbox
                .and_then(|mode| serde_json::from_value(mode.clone()).ok())
                .or_else(|| {
                    if mxc_selected {
                        return None;
                    }
                    let features = config.config.additional.get("features")?;
                    [
                        (
                            "elevated_windows_sandbox",
                            WindowsSandboxSetupMode::Elevated,
                        ),
                        (
                            "experimental_windows_sandbox",
                            WindowsSandboxSetupMode::Unelevated,
                        ),
                        (
                            "enable_experimental_windows_sandbox",
                            WindowsSandboxSetupMode::Unelevated,
                        ),
                    ]
                    .into_iter()
                    .find_map(|(key, mode)| {
                        (features.get(key).and_then(serde_json::Value::as_bool) == Some(true))
                            .then_some(mode)
                    })
                }),
            requirements: Some(
                requirements
                    .requirements
                    .and_then(|requirements| requirements.allowed_windows_sandbox_implementations)
                    .map(|allowed| {
                        allowed
                            .into_iter()
                            .filter_map(|implementation| match implementation {
                                WindowsSandboxImplementation::Elevated => {
                                    Some(WindowsSandboxSetupMode::Elevated)
                                }
                                WindowsSandboxImplementation::Unelevated => {
                                    Some(WindowsSandboxSetupMode::Unelevated)
                                }
                                WindowsSandboxImplementation::Mxc => None,
                            })
                            .collect()
                    }),
            ),
        };
        if !state.mxc_selected
            && let Some(Some(allowed)) = &state.requirements
            && !state.mode.is_some_and(|mode| allowed.contains(&mode))
        {
            // Managed requirements prefer elevated when the configured value is disallowed.
            state.mode = [
                WindowsSandboxSetupMode::Elevated,
                WindowsSandboxSetupMode::Unelevated,
            ]
            .into_iter()
            .find(|mode| allowed.contains(mode));
        }
        state
    }

    pub(crate) fn is_enabled(&self) -> bool {
        self.mxc_selected || self.mode.is_some()
    }

    pub(crate) fn level(&self) -> WindowsSandboxLevel {
        match self.mode {
            Some(WindowsSandboxSetupMode::Elevated) => WindowsSandboxLevel::Elevated,
            Some(WindowsSandboxSetupMode::Unelevated) => WindowsSandboxLevel::RestrictedToken,
            None => WindowsSandboxLevel::Disabled,
        }
    }

    pub(crate) fn allows(&self, mode: WindowsSandboxSetupMode) -> bool {
        match &self.requirements {
            Some(Some(allowed)) => allowed.contains(&mode),
            Some(None) => true,
            None => false,
        }
    }

    pub(crate) fn requires_elevated(&self) -> bool {
        self.mode == Some(WindowsSandboxSetupMode::Elevated)
            && matches!(self.requirements, Some(Some(_)))
    }

    pub(crate) async fn read(
        handle: AppServerRequestHandle,
        cwd: String,
    ) -> color_eyre::Result<Self> {
        tokio::time::timeout(std::time::Duration::from_secs(5), async {
            let config = crate::config_update::read_effective_config(handle.clone(), cwd).await?;
            let requirements = handle
                .request_typed(ClientRequest::ConfigRequirementsRead {
                    request_id: RequestId::String(format!(
                        "tui-windows-sandbox-requirements-{}",
                        Uuid::new_v4()
                    )),
                    params: None,
                })
                .await?;
            Ok(Self::from_responses(&config, requirements))
        })
        .await?
    }
}

<<<<<<< HEAD
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
||||||| 413492cd6c
#[cfg(target_os = "windows")]
pub(crate) use codex_windows_sandbox::sandbox_setup_is_complete;

#[cfg(not(target_os = "windows"))]
pub(crate) fn sandbox_setup_is_complete(_codex_home: &Path) -> bool {
    false
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
=======
/// A server-local connection can still select remote executors. Missing selections are unknown.
pub(crate) fn host_from_environments(
    environments: Option<&[codex_app_server_protocol::ThreadEnvironment]>,
) -> crate::app::WindowsSandboxHost {
    use crate::app::WindowsSandboxHost;
    let Some(environments) = environments.filter(|environments| !environments.is_empty()) else {
        return WindowsSandboxHost::Unknown;
    };
    let local = environments
        .iter()
        .any(|environment| environment.environment_id == codex_exec_server::LOCAL_ENVIRONMENT_ID);
    let remote = environments
        .iter()
        .any(|environment| environment.environment_id != codex_exec_server::LOCAL_ENVIRONMENT_ID);
    match (local, remote) {
        (true, false) => WindowsSandboxHost::Local,
        (true, true) => WindowsSandboxHost::Mixed,
        (false, true) => WindowsSandboxHost::Remote,
        (false, false) => WindowsSandboxHost::Unknown,
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
    }
}

<<<<<<< HEAD
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
||||||| 413492cd6c
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
=======
#[cfg(test)]
#[path = "windows_sandbox_tests.rs"]
mod tests;
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
