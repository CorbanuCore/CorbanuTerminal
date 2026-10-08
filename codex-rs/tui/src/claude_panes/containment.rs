//! Contained Claude panes (issue #218, feature `contained_external_agents`).
//!
//! With the feature on, every Claude pane turn runs under the PF-27-S02
//! secretless launch contract instead of directly:
//! - a clean environment: the contract's allowlist, nothing proxy-related
//!   from the user, and only what the pane needs on top;
//! - every provider behind the per-turn loopback bridge, so Claude Code only
//!   ever holds the bridge token, never a provider key or `apiKeyHelper`;
//! - an OS sandbox around `claude` itself: writes only in the pane's folder
//!   and the pane's own Claude state folder (outside `CODEX_HOME`), the
//!   contract's protected reads denied, and network only to the bridge. The
//!   bridge is reached as `HTTP_PROXY`, which Seatbelt allows by port and the
//!   Linux sandbox carries into its network namespace over a Unix socket.

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use std::sync::OnceLock;

use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use codex_network_proxy::ManagedNetworkSandboxContext;
use codex_protocol::config_types::WindowsSandboxLevel;
use codex_protocol::models::PermissionProfile;
use codex_protocol::permissions::FileSystemAccessMode;
use codex_protocol::permissions::FileSystemPath;
use codex_protocol::permissions::FileSystemSandboxEntry;
use codex_protocol::permissions::FileSystemSandboxPolicy;
use codex_protocol::permissions::NetworkSandboxPolicy;
use codex_protocol::secretless_launch;
use codex_sandboxing::SandboxCommand;
use codex_sandboxing::SandboxManager;
use codex_sandboxing::SandboxTransformRequest;
use codex_sandboxing::get_platform_sandbox;
use codex_utils_absolute_path::AbsolutePathBuf;
use codex_utils_path_uri::PathUri;
use sha2::Digest;
use sha2::Sha256;

/// What the TUI's config says about contained panes, installed at startup.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ContainmentSettings {
    /// The `contained_external_agents` feature.
    pub(crate) enabled: bool,
    /// The Linux sandbox helper (`codex-linux-sandbox`), from the launch.
    pub(crate) linux_sandbox_exe: Option<PathBuf>,
    /// Where pane state folders go instead of the account's application
    /// state (tests).
    pub(crate) state_root: Option<PathBuf>,
}

static SETTINGS: OnceLock<ContainmentSettings> = OnceLock::new();

pub(crate) fn install(settings: ContainmentSettings) {
    let _ = SETTINGS.set(settings);
}

/// The installed settings when contained panes are on.
pub(crate) fn enabled() -> Option<ContainmentSettings> {
    #[cfg(test)]
    if let Some(settings) = test_settings::SETTINGS.with(|cell| cell.borrow().clone()) {
        return settings.enabled.then_some(settings);
    }
    SETTINGS.get().filter(|settings| settings.enabled).cloned()
}

/// Stand-in settings for tests, on the current thread only.
#[cfg(test)]
pub(crate) mod test_settings {
    use std::cell::Cell;
    use std::cell::RefCell;

    use super::ContainmentSettings;

    thread_local! {
        pub(super) static SETTINGS: RefCell<Option<ContainmentSettings>> =
            const { RefCell::new(None) };
        pub(super) static CONTRACT_ARMED: Cell<Option<bool>> = const { Cell::new(None) };
    }

    /// Stands in for whether the secretless launch contract is armed, until
    /// dropped.
    pub(crate) struct ArmedGuard;

    pub(crate) fn set_contract_armed(armed: bool) -> ArmedGuard {
        CONTRACT_ARMED.set(Some(armed));
        ArmedGuard
    }

    impl Drop for ArmedGuard {
        fn drop(&mut self) {
            CONTRACT_ARMED.set(None);
        }
    }

    /// Sets the settings until dropped.
    pub(crate) struct Guard;

    pub(crate) fn set(settings: ContainmentSettings) -> Guard {
        SETTINGS.with(|cell| *cell.borrow_mut() = Some(settings));
        Guard
    }

    impl Drop for Guard {
        fn drop(&mut self) {
            SETTINGS.with(|cell| *cell.borrow_mut() = None);
        }
    }
}

/// Whether Claude panes can run contained here: the feature is on, the
/// secretless launch contract is armed, and the platform has a sandbox.
pub(crate) fn contained_launch_ready() -> bool {
    #[cfg(test)]
    let armed = test_settings::CONTRACT_ARMED
        .get()
        .unwrap_or_else(crate::legacy_core::external_agent_contract_armed);
    #[cfg(not(test))]
    let armed = crate::legacy_core::external_agent_contract_armed();
    enabled().is_some()
        && armed
        && get_platform_sandbox(/*windows_sandbox_enabled*/ false).is_some()
}

/// How one contained turn is launched.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ClaudeContainment {
    /// The pane's own Claude state (`CLAUDE_CONFIG_DIR` and `TMPDIR`), the
    /// only writable place besides the pane's folder.
    pub(crate) state_dir: PathBuf,
    /// `CODEX_HOME/panes`: every pane's transcripts and audits, unreadable
    /// to a contained pane, like the contract's `sessions`.
    pub(crate) panes_dir: PathBuf,
    pub(crate) linux_sandbox_exe: Option<PathBuf>,
}

impl ClaudeContainment {
    /// Corbanu's settings for the turn, read-only to Claude Code.
    pub(crate) fn settings_path(&self) -> PathBuf {
        self.state_dir.join("settings.json")
    }

    pub(crate) fn config_dir(&self) -> PathBuf {
        self.state_dir.join("config")
    }

    pub(crate) fn tmp_dir(&self) -> PathBuf {
        self.state_dir.join("tmp")
    }
}

/// The pane's Claude state folder. It must sit outside `CODEX_HOME`, which
/// the contract never lets a launch write, so it lives in the account's
/// application state, keyed by the Corbanu home and the pane.
pub(crate) fn state_dir(
    codex_home: &Path,
    pane_id: &str,
    state_root: Option<&Path>,
) -> Result<PathBuf> {
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .filter(|home| home.is_absolute())
        .ok_or_else(|| anyhow!("contained Claude panes need an absolute HOME"))?;
    let base = if let Some(root) = state_root {
        root.to_path_buf()
    } else if cfg!(target_os = "macos") {
        home.join("Library/Application Support/Corbanu/claude-panes")
    } else {
        std::env::var_os("XDG_STATE_HOME")
            .map(PathBuf::from)
            .filter(|dir| dir.is_absolute())
            .unwrap_or_else(|| home.join(".local/state"))
            .join("corbanu/claude-panes")
    };
    let digest = Sha256::digest(codex_home.as_os_str().as_encoded_bytes());
    let home_key = digest
        .iter()
        .take(8)
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    Ok(base.join(home_key).join(pane_id))
}

/// The launch environment: the contract's allowlist applied to this process's
/// environment, without any proxy setting of the user's (the Linux sandbox
/// would carry a loopback proxy into the sandbox), then the pane's own
/// variables.
pub(crate) fn launch_env(
    inherited: impl IntoIterator<Item = (String, String)>,
    pane_env: &BTreeMap<String, String>,
) -> HashMap<String, String> {
    let mut env = inherited
        .into_iter()
        .filter(|(name, _)| !name.to_ascii_uppercase().contains("PROXY"))
        .collect::<HashMap<_, _>>();
    secretless_launch::retain_launch_env(&mut env, |_| false);
    env.extend(
        pane_env
            .iter()
            .map(|(name, value)| (name.clone(), value.clone())),
    );
    env
}

/// Writes only in the pane's folder and its Claude state folder, network off
/// except the bridge. The contract adds its protected-read denials.
pub(crate) fn base_profile(containment: &ClaudeContainment) -> Result<PermissionProfile> {
    let absolute = |path: &Path| {
        AbsolutePathBuf::from_absolute_path(path)
            .with_context(|| format!("`{}` is not absolute", path.display()))
    };
    let mut file_system = FileSystemSandboxPolicy::workspace_write(
        &[absolute(&containment.state_dir)?],
        /*exclude_tmpdir_env_var*/ true,
        /*exclude_slash_tmp*/ true,
    );
    file_system.entries.extend([
        FileSystemSandboxEntry::new(
            FileSystemPath::Path {
                path: absolute(&containment.settings_path())?,
            },
            FileSystemAccessMode::Read,
        ),
        FileSystemSandboxEntry::new(
            FileSystemPath::Path {
                path: absolute(&containment.panes_dir)?,
            },
            FileSystemAccessMode::Deny,
        ),
    ]);
    // Other panes' Claude state (their session transcripts) sits next to
    // this pane's. Each one is denied; the folder holding them is not,
    // because Claude Code examines every component of its own paths.
    if let Some(parent) = containment.state_dir.parent()
        && let Ok(siblings) = std::fs::read_dir(parent)
    {
        for sibling in siblings.flatten() {
            let path = sibling.path();
            if path != containment.state_dir {
                file_system.entries.push(FileSystemSandboxEntry::new(
                    FileSystemPath::Path {
                        path: absolute(&path)?,
                    },
                    FileSystemAccessMode::Deny,
                ));
            }
        }
    }
    Ok(PermissionProfile::from_runtime_permissions(
        &file_system,
        NetworkSandboxPolicy::Restricted,
    ))
}

/// A contained launch, ready to spawn.
#[derive(Debug)]
pub(crate) struct ContainedCommand {
    pub(crate) argv: Vec<String>,
    #[cfg_attr(
        not(unix),
        allow(dead_code, reason = "argv[0] is only overridden on Unix")
    )]
    pub(crate) arg0: Option<String>,
    pub(crate) env: HashMap<String, String>,
}

/// Wrap `executable args` in the platform sandbox under the armed contract.
/// `bridge_port` is the only network destination allowed.
pub(crate) fn contain(
    containment: &ClaudeContainment,
    executable: &str,
    args: &[String],
    pane_env: &BTreeMap<String, String>,
    cwd: &Path,
    bridge_port: u16,
) -> Result<ContainedCommand> {
    for dir in [containment.config_dir(), containment.tmp_dir()] {
        std::fs::create_dir_all(&dir)
            .with_context(|| format!("failed to create `{}`", dir.display()))?;
    }
    let sandbox = get_platform_sandbox(/*windows_sandbox_enabled*/ false)
        .ok_or_else(|| anyhow!("contained Claude panes need macOS or Linux"))?;
    let inherited = std::env::vars_os()
        .filter_map(|(name, value)| Some((name.into_string().ok()?, value.into_string().ok()?)));
    let mut env = launch_env(inherited, pane_env);
    let mut argv = vec![executable.to_string()];
    argv.extend(args.iter().cloned());
    let profile = crate::legacy_core::protect_external_agent_launch(
        sandbox,
        &argv,
        &mut env,
        &base_profile(containment)?,
        cwd,
    )
    .map_err(|err| anyhow!(err))?;
    let cwd = AbsolutePathBuf::from_absolute_path(cwd).context("pane folder is not absolute")?;
    let cwd = PathUri::from_abs_path(&cwd);
    let request = SandboxManager::new()
        .transform(SandboxTransformRequest {
            command: SandboxCommand {
                program: executable.into(),
                args: args.to_vec(),
                cwd: cwd.clone(),
                env,
                managed_network: Some(ManagedNetworkSandboxContext {
                    loopback_ports: vec![bridge_port],
                    allow_local_binding: false,
                }),
                additional_permissions: None,
            },
            permissions: &profile,
            sandbox,
            enforce_managed_network: true,
            environment_id: None,
            network: None,
            sandbox_policy_cwd: &cwd,
            codex_linux_sandbox_exe: containment.linux_sandbox_exe.as_deref(),
            use_legacy_landlock: false,
            windows_sandbox_level: WindowsSandboxLevel::Disabled,
            windows_sandbox_private_desktop: false,
        })
        .map_err(|err| anyhow!("could not sandbox Claude Code: {err}"))?;
    Ok(ContainedCommand {
        argv: request.command,
        arg0: request.arg0,
        env: request.env,
    })
}

#[cfg(test)]
#[path = "containment_tests.rs"]
mod tests;
