//! `corbanu exec` and `corbanu review` held to Aggressive when an agent
//! command under Aggressive started them and nested launches pass.

use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;

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
        Subcommand::App(_) => ("app", Host),
        Subcommand::ExecServer(_) => ("exec-server", Host),
        Subcommand::Cloud(_) => ("cloud", Host),
        Subcommand::Telegram(_) => ("telegram", Host),
        Subcommand::ClaudePaneSmoke(_) => ("claude-pane-smoke", Host),
        Subcommand::ClaudePaneWorkflowSuite(_) => ("claude-pane-workflow-suite", Host),
        Subcommand::Vault(_) => ("vault", Credentials),
        Subcommand::InternalClaudeOauthToken => ("internal-claude-oauth-token", Credentials),
        Subcommand::InternalGpuEndpointToken { .. } => ("internal-gpu-endpoint-token", Credentials),
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
        | Subcommand::Archive(_)
        | Subcommand::Delete(_)
        | Subcommand::Unarchive(_)
        | Subcommand::ResponsesApiProxy(_)
        | Subcommand::StdioToUds(_)
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
        let cli_overrides = codex_tui::aggressive_cli_overrides(&codex_home, &origin)
            .map_err(anyhow::Error::msg)?;
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
