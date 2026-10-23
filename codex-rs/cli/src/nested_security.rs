//! `corbanu exec` and `corbanu review` held to Aggressive when an agent
//! command under Aggressive started them and nested launches pass, and to
//! Moderate when an agent command under Moderate started them (#766).

use std::future::Future;
use std::path::Path;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::OnceLock;

use codex_config::TomlValue;
use codex_config::types::ShellEnvironmentPolicyToml;
use codex_core::config::Config;
use codex_core::config::ConfigOverrides;
use codex_exec::EnforcedSecurity;

use crate::AppServerCommand;
use crate::AppServerSubcommand;
use crate::DebugCommand;
use crate::DebugSubcommand;
use crate::Subcommand;

/// Subcommands that start, host or hand credentials to agents, with how a
/// nested launch under Aggressive treats them (see `codex_tui::nested_launch`).
pub(crate) fn nested_launch_kind(
    subcommand: Option<&Subcommand>,
) -> Option<(&'static str, codex_tui::NestedKind)> {
    use codex_tui::NestedKind::Agent;
    use codex_tui::NestedKind::Credentials;
    use codex_tui::NestedKind::Host;
    use codex_tui::NestedKind::Interactive;
    let Some(subcommand) = subcommand else {
        return Some(("", Interactive));
    };
    Some(match subcommand {
        Subcommand::Exec(_) => ("exec", Agent),
        Subcommand::Review(_) => ("review", Agent),
        Subcommand::Resume(_) => ("resume", Interactive),
        Subcommand::Fork(_) => ("fork", Interactive),
        Subcommand::McpServer(_) => ("mcp-server", Host),
        Subcommand::AppServer(AppServerCommand {
            subcommand:
                Some(
                    AppServerSubcommand::GenerateTs(_)
                    | AppServerSubcommand::GenerateJsonSchema(_)
                    | AppServerSubcommand::GenerateInternalJsonSchema(_),
                ),
            ..
        }) => return None,
        Subcommand::AppServer(_) => ("app-server", Host),
        Subcommand::Debug(DebugCommand {
            subcommand: DebugSubcommand::AppServer(_),
        }) => ("debug app-server", Host),
        Subcommand::RemoteControl(_) => ("remote-control", Host),
        #[cfg(any(target_os = "macos", target_os = "windows"))]
        Subcommand::App(_) => ("app", Host),
        Subcommand::ExecServer(_) => ("exec-server", Host),
        Subcommand::Cloud(_) => ("cloud", Host),
        Subcommand::Telegram(_) => ("telegram", Host),
        // They reach the person's sessions through an app server or daemon.
        Subcommand::Archive(_) => ("archive", Host),
        Subcommand::Delete(_) => ("delete", Host),
        Subcommand::Unarchive(_) => ("unarchive", Host),
        Subcommand::ClaudePaneSmoke(_) => ("claude-pane-smoke", Host),
        Subcommand::ClaudePaneWorkflowSuite(_) => ("claude-pane-workflow-suite", Host),
        // Reaches a running app server or daemon socket, as a host would.
        Subcommand::StdioToUds(_) => ("stdio-to-uds", Host),
        Subcommand::Vault(_) => ("vault", Credentials),
        Subcommand::Account(_) => ("account", Credentials),
        Subcommand::InternalClaudeOauthToken { .. } => (
            codex_login::auth::CLAUDE_PLAN_TOKEN_HELPER_SUBCOMMAND,
            Credentials,
        ),
        Subcommand::InternalGpuEndpointToken { .. } => (
            codex_login::auth::GPU_ENDPOINT_TOKEN_HELPER_SUBCOMMAND,
            Credentials,
        ),
        Subcommand::InternalGpuController(_) => ("internal-gpu-controller", Credentials),
        Subcommand::InternalClaudeLoginHealth { .. } => {
            ("internal-claude-login-health", Credentials)
        }
        Subcommand::Tasknode(_) => ("tasknode", Credentials),
        // Account, configuration and inspection commands start no agent.
        Subcommand::Login(_)
        | Subcommand::Logout(_)
        | Subcommand::Mcp(_)
        | Subcommand::Plugin(_)
        | Subcommand::Completion(_)
        | Subcommand::Update
        | Subcommand::Doctor(_)
        | Subcommand::Sandbox(_)
        | Subcommand::Debug(_)
        | Subcommand::Execpolicy(_)
        | Subcommand::Apply(_)
        | Subcommand::ResponsesApiProxy(_)
        | Subcommand::Features(_) => return None,
    })
}

pub(crate) struct NestedAggressive {
    codex_home: PathBuf,
    origin: PathBuf,
    cli_overrides: Vec<(String, TomlValue)>,
}

impl NestedAggressive {
    pub(crate) fn new(origin: PathBuf) -> anyhow::Result<Self> {
        let codex_home = codex_core::config::find_codex_home()?.to_path_buf();
        let cli_overrides =
            codex_tui::prepare_nested_exec(&codex_home, &origin).map_err(anyhow::Error::msg)?;
        Ok(Self {
            codex_home,
            origin,
            cli_overrides,
        })
    }
}

impl EnforcedSecurity for NestedAggressive {
    fn notice(&self) -> String {
        format!(
            "Security level Aggressive is enforced because an agent command under Aggressive started this run ({}).",
            self.origin.display()
        )
    }

    fn cli_overrides(&self) -> Vec<(String, TomlValue)> {
        self.cli_overrides.clone()
    }

    fn env_overrides(&self, user_env: &ShellEnvironmentPolicyToml) -> Vec<(String, TomlValue)> {
        codex_tui::aggressive_env_overrides(user_env)
    }

    fn apply_overrides(&self, overrides: &mut ConfigOverrides) -> Vec<&'static str> {
        codex_tui::apply_aggressive_launch_overrides(overrides)
    }

    fn verify<'a>(
        &'a self,
        config: &'a Config,
    ) -> Pin<Box<dyn Future<Output = Result<(), String>> + 'a>> {
        Box::pin(codex_tui::verify_aggressive_config(
            &self.codex_home,
            &self.origin,
            config,
        ))
    }
}

/// #766: the Corbanu home this process was given, and the Moderate origin,
/// when [`prepare_moderate_home`] replaced that home with one of its own.
static PROTECTED_HOME: OnceLock<(PathBuf, PathBuf)> = OnceLock::new();

/// #766: the Moderate origin found before the home was replaced. Detection
/// after the change may no longer find it (it was found through the home
/// this process was given), so it stays held to Moderate.
pub(crate) fn relocated_origin() -> Option<PathBuf> {
    PROTECTED_HOME.get().map(|(_, origin)| origin.clone())
}

/// #766: before any thread starts, for `corbanu exec` or `review` held to
/// Moderate. Inside the sandbox of the agent command that started it, the
/// Corbanu home that command runs on is read-only and its config, sign-in,
/// security state and history are unreadable, so the run could not start
/// there. When the home it was given cannot be written, it gets an empty
/// Corbanu home of its own for this run (removed when the returned folder
/// is dropped): no config, sign-in or model keys come from the protected
/// home. A home it can write (one the agent command chose) is used as given.
pub(crate) fn prepare_moderate_home(origin: &Path) -> Result<Option<tempfile::TempDir>, String> {
    let Ok(home) = codex_core::config::find_codex_home() else {
        return Ok(None);
    };
    match tempfile::NamedTempFile::new_in(&home) {
        Ok(_) => return Ok(None),
        // Created at start, as for a first run.
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => {}
    }
    let own = tempfile::Builder::new()
        .prefix("corbanu-moderate-home-")
        .tempdir()
        .map_err(|err| {
            format!(
                "This run was started by an agent command while security level Moderate is enforced ({}). It cannot use the Corbanu home {} inside the sandbox and could not make one of its own in {}: {err}",
                origin.display(),
                home.display(),
                std::env::temp_dir().display()
            )
        })?;
    // SAFETY: called from `main` before the async runtime starts. No thread
    // of this program exists yet; the account lookup above may have started
    // system helper threads, which do not read the environment. The debug
    // home variables need no change: `configure_for_current_process` already
    // turned them into `CODEX_HOME`, which wins here.
    unsafe {
        std::env::remove_var("CORBANU_HOME");
        std::env::remove_var("PFTERMINAL_HOME");
        std::env::remove_var(codex_state::SQLITE_HOME_ENV);
        std::env::set_var("CODEX_HOME", own.path());
    }
    let _ = PROTECTED_HOME.set((home.to_path_buf(), origin.to_path_buf()));
    Ok(Some(own))
}

/// #766 (Travis, 2026-10-14): `corbanu exec` or `review` started by an agent
/// command under Moderate runs with Moderate enforced: Core's level is at
/// least Moderate and the untrusted-content labels are on, whatever the
/// home's config says; weakening flags are ignored; it runs headless, so
/// nobody approves its commands; and a run without a sandbox is refused.
pub(crate) struct NestedModerate {
    origin: PathBuf,
    codex_home: PathBuf,
}

impl NestedModerate {
    pub(crate) fn new(origin: PathBuf) -> anyhow::Result<Self> {
        let codex_home = codex_core::config::find_codex_home()?.to_path_buf();
        Ok(Self { origin, codex_home })
    }
}

impl EnforcedSecurity for NestedModerate {
    fn notice(&self) -> String {
        let mut notice = format!(
            "Security level Moderate is enforced because an agent command under Moderate started this run ({}).",
            self.origin.display()
        );
        if let Some((protected, _)) = PROTECTED_HOME.get() {
            let which = if *protected == self.origin {
                "that Corbanu home".to_string()
            } else {
                format!("the Corbanu home {}", protected.display())
            };
            notice.push_str(&format!(
                " It cannot write {which} inside the sandbox, so it runs in a Corbanu home of its own ({}), without that home's config, sign-in or model keys; give the model and provider with -m and -c.",
                self.codex_home.display()
            ));
        }
        notice
    }

    fn cli_overrides(&self) -> Vec<(String, TomlValue)> {
        vec![
            // Core keeps the stricter of every layer's level.
            ("security.version".to_string(), TomlValue::Integer(1)),
            (
                "security.level".to_string(),
                TomlValue::String("moderate".to_string()),
            ),
            // Without the labels Core refuses every model request under a
            // protected level.
            (
                "features.source_envelopes".to_string(),
                TomlValue::Boolean(true),
            ),
        ]
    }

    fn env_overrides(&self, _user_env: &ShellEnvironmentPolicyToml) -> Vec<(String, TomlValue)> {
        Vec::new()
    }

    fn apply_overrides(&self, overrides: &mut ConfigOverrides) -> Vec<&'static str> {
        let mut replaced = Vec::new();
        if overrides.sandbox_mode
            == Some(codex_protocol::config_types::SandboxMode::DangerFullAccess)
        {
            overrides.sandbox_mode =
                Some(codex_protocol::config_types::SandboxMode::WorkspaceWrite);
            replaced.push("--sandbox danger-full-access");
        }
        if overrides.bypass_hook_trust == Some(true) {
            overrides.bypass_hook_trust = None;
            replaced.push("--dangerously-bypass-hook-trust");
        }
        // Headless, as `corbanu exec` is without enforcement.
        overrides.approval_policy = Some(codex_protocol::protocol::AskForApproval::Never);
        replaced
    }

    fn verify<'a>(
        &'a self,
        config: &'a Config,
    ) -> Pin<Box<dyn Future<Output = Result<(), String>> + 'a>> {
        Box::pin(async move {
            let mut failures = Vec::new();
            if config.security_level < codex_security_policy::SecurityLevel::Moderate {
                failures.push(format!("Core's level is {:?}", config.security_level));
            }
            if !config
                .features
                .enabled(codex_features::Feature::SourceEnvelopes)
            {
                failures
                    .push("the untrusted-content labels (source_envelopes) are off".to_string());
            }
            if config.permissions.file_system_sandbox_policy().kind
                != codex_protocol::permissions::FileSystemSandboxKind::Restricted
            {
                failures.push(
                    "agent commands would run without Corbanu's sandbox (sandbox_mode or permissions in this Corbanu home's config); choose read-only or workspace-write".to_string(),
                );
            }
            if failures.is_empty() {
                return Ok(());
            }
            Err(format!(
                "This run was started by an agent command while security level Moderate is enforced ({}), and Moderate could not be applied to it:\n  {}",
                self.origin.display(),
                failures.join("\n  ")
            ))
        })
    }
}
