use crate::sandboxing::SandboxPermissions;
use crate::shell::Shell;
use crate::shell::ShellType;
use crate::shell::get_shell_by_model_provided_path;
use crate::tools::context::ToolInvocation;
use crate::tools::context::ToolOutput;
use crate::tools::context::ToolPayload;
use crate::tools::hook_names::HookToolName;
use crate::tools::registry::PostToolUsePayload;
use codex_exec_server::Environment;
use codex_protocol::models::AdditionalPermissionProfile;
use codex_tools::UnifiedExecShellMode;
use serde::Deserialize;
use serde::Deserializer;
use serde::de::Error as _;
use serde_json::Number;
use std::path::PathBuf;
use std::sync::Arc;

#[cfg(test)]
use crate::tools::handlers::parse_arguments;

mod exec_command;
mod write_stdin;

pub use exec_command::ExecCommandHandler;
pub(crate) use exec_command::ExecCommandHandlerOptions;
pub use write_stdin::WriteStdinHandler;

#[derive(Debug, Deserialize)]
pub(crate) struct ExecCommandArgs {
    pub(crate) cmd: String,
    #[serde(default)]
    shell: Option<String>,
    #[serde(default)]
    login: Option<bool>,
    #[serde(default = "default_tty")]
    tty: bool,
    #[serde(default = "default_exec_yield_time_ms")]
    #[serde(deserialize_with = "deserialize_integral_u64")]
    yield_time_ms: u64,
    #[serde(default, deserialize_with = "deserialize_optional_integral_usize")]
    max_output_tokens: Option<usize>,
    #[serde(default)]
    sandbox_permissions: SandboxPermissions,
    #[serde(default)]
    additional_permissions: Option<AdditionalPermissionProfile>,
    #[serde(default)]
    justification: Option<String>,
    #[serde(default)]
    prefix_rule: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct ExecCommandEnvironmentArgs {
    #[serde(default)]
    environment_id: Option<String>,
    // Keep this raw until after environment selection; relative paths must be
    // resolved against the selected environment cwd, not the process cwd.
    #[serde(default)]
    workdir: Option<String>,
}

fn default_exec_yield_time_ms() -> u64 {
    10_000
}

fn default_write_stdin_yield_time_ms() -> u64 {
    250
}

fn default_tty() -> bool {
    false
}

const MAX_EXACT_INTEGER_IN_F64: f64 = 9_007_199_254_740_991.0;

fn integral_f64<E>(number: &Number, field: &str) -> Result<Option<f64>, E>
where
    E: serde::de::Error,
{
    let Some(value) = number.as_f64() else {
        return Ok(None);
    };
    if !value.is_finite() || value.fract() != 0.0 || value.abs() > MAX_EXACT_INTEGER_IN_F64 {
        return Err(E::custom(format!(
            "`{field}` must be an exactly represented integer"
        )));
    }
    Ok(Some(value))
}

fn deserialize_integral_i32<'de, D>(deserializer: D) -> Result<i32, D::Error>
where
    D: Deserializer<'de>,
{
    let number = Number::deserialize(deserializer)?;
    if let Some(value) = number.as_i64() {
        return i32::try_from(value).map_err(D::Error::custom);
    }
    let value = integral_f64::<D::Error>(&number, "session_id")?
        .ok_or_else(|| D::Error::custom("`session_id` must be an integer"))?;
    if value < f64::from(i32::MIN) || value > f64::from(i32::MAX) {
        return Err(D::Error::custom("`session_id` is outside the i32 range"));
    }
    Ok(value as i32)
}

fn deserialize_integral_u64<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    let number = Number::deserialize(deserializer)?;
    if let Some(value) = number.as_u64() {
        return Ok(value);
    }
    let value = integral_f64::<D::Error>(&number, "yield_time_ms")?
        .ok_or_else(|| D::Error::custom("`yield_time_ms` must be a non-negative integer"))?;
    if value < 0.0 {
        return Err(D::Error::custom(
            "`yield_time_ms` must be a non-negative integer",
        ));
    }
    Ok(value as u64)
}

fn deserialize_optional_integral_usize<'de, D>(deserializer: D) -> Result<Option<usize>, D::Error>
where
    D: Deserializer<'de>,
{
    let Some(number) = Option::<Number>::deserialize(deserializer)? else {
        return Ok(None);
    };
    if let Some(value) = number.as_u64() {
        return usize::try_from(value).map(Some).map_err(D::Error::custom);
    }
    let value = integral_f64::<D::Error>(&number, "max_output_tokens")?
        .ok_or_else(|| D::Error::custom("`max_output_tokens` must be a non-negative integer"))?;
    if value < 0.0 || value > usize::MAX as f64 {
        return Err(D::Error::custom("`max_output_tokens` must fit in usize"));
    }
    Ok(Some(value as usize))
}

#[derive(Debug)]
pub(crate) struct ResolvedCommand {
    pub(crate) command: Vec<String>,
    pub(crate) shell_type: ShellType,
}

fn post_unified_exec_tool_use_payload(
    invocation: &ToolInvocation,
    result: &dyn ToolOutput,
) -> Option<PostToolUsePayload> {
    let ToolPayload::Function { .. } = &invocation.payload else {
        return None;
    };

    let tool_input = result.post_tool_use_input(&invocation.payload)?;
    let tool_use_id = result.post_tool_use_id(&invocation.call_id);
    let tool_response = result.post_tool_use_response(&tool_use_id, &invocation.payload)?;
    Some(PostToolUsePayload {
        tool_name: HookToolName::bash(),
        tool_use_id,
        tool_input,
        tool_response,
    })
}

pub(crate) fn get_command(
    args: &ExecCommandArgs,
    session_shell: Arc<Shell>,
    shell_mode: &UnifiedExecShellMode,
    allow_login_shell: bool,
) -> Result<ResolvedCommand, String> {
    let use_login_shell = match args.login {
        Some(true) if !allow_login_shell => {
            return Err(
                "login shell is disabled by config; omit `login` or set it to false.".to_string(),
            );
        }
        Some(use_login_shell) => use_login_shell,
        None => allow_login_shell,
    };

    match shell_mode {
        UnifiedExecShellMode::Direct => {
            let model_shell = args
                .shell
                .as_ref()
                .map(|shell_str| get_shell_by_model_provided_path(&PathBuf::from(shell_str)));
            let shell = model_shell.as_ref().unwrap_or(session_shell.as_ref());
            Ok(ResolvedCommand {
                command: shell.derive_exec_args(&args.cmd, use_login_shell),
                shell_type: shell.shell_type,
            })
        }
        UnifiedExecShellMode::ZshFork(zsh_fork_config) => {
            if args.shell.is_some() {
                return Err(
                    "`shell` is not supported for local zsh-fork exec; omit `shell` to use zsh-fork, or target a remote environment where `shell` is supported.".to_string(),
                );
            }

            Ok(ResolvedCommand {
                command: vec![
                    zsh_fork_config.shell_zsh_path.to_string_lossy().to_string(),
                    if use_login_shell { "-lc" } else { "-c" }.to_string(),
                    args.cmd.clone(),
                ],
                shell_type: ShellType::Zsh,
            })
        }
    }
}

pub(crate) fn shell_mode_for_environment(
    turn_shell_mode: &UnifiedExecShellMode,
    environment: &Environment,
) -> UnifiedExecShellMode {
    if environment.is_remote() {
        UnifiedExecShellMode::Direct
    } else {
        turn_shell_mode.clone()
    }
}

#[cfg(test)]
#[path = "unified_exec_tests.rs"]
mod tests;
