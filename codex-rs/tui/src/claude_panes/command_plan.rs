//! Building Claude Code command plans, settings, and prompts.

use std::collections::BTreeMap;
use std::net::TcpListener as StdTcpListener;
use std::path::Path;
use std::path::PathBuf;

use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use codex_app_server_protocol::UserInput;
use codex_vault::Vault;
use serde_json::Value;
use uuid::Uuid;

use crate::app_command::AppCommand;
use crate::internal_cli_helper::internal_cli_helper_executable;
use crate::spawn_orchestration::SpawnRole;

use super::containment::ClaudeContainment;
use super::pane::ClaudeCommandMode;
use super::pane::ClaudePane;
use super::provider::ClaudeProviderProfile;
use super::provider::ClaudeProviderProfileKind;
use super::provider::ClaudeProviderTransport;
use super::turn_types::ClaudeBridgeKind;
use super::turn_types::ClaudeBridgePlan;
use super::turn_types::ClaudeCommandPlan;
use super::turn_types::DeferredClaudePlanAuth;
use super::turn_types::DeferredVaultSecret;
use super::turn_types::PaneDirectAccounting;

const ANTHROPIC_AUTH_ENV_KEYS: [&str; 2] = ["ANTHROPIC_API_KEY", "ANTHROPIC_AUTH_TOKEN"];
const CLAUDE_CREDENTIAL_ENV_KEYS: [&str; 4] = [
    "CLAUDE_CODE_OAUTH_TOKEN",
    "CLAUDE_CODE_OAUTH_TOKEN_FILE_DESCRIPTOR",
    "CLAUDE_CODE_OAUTH_REFRESH_TOKEN",
    "CLAUDE_CODE_API_KEY_FILE_DESCRIPTOR",
];
const CLAUDE_PLAN_ROUTING_ENV_KEYS: [&str; 12] = [
    "ANTHROPIC_API_KEY",
    "ANTHROPIC_AUTH_TOKEN",
    "ANTHROPIC_BASE_URL",
    "CLAUDE_CODE_OAUTH_TOKEN",
    "CLAUDE_CODE_OAUTH_TOKEN_FILE_DESCRIPTOR",
    "CLAUDE_CODE_OAUTH_REFRESH_TOKEN",
    "CLAUDE_CODE_OAUTH_SCOPES",
    "CLAUDE_CODE_API_KEY_FILE_DESCRIPTOR",
    "CLAUDE_CODE_USE_BEDROCK",
    "CLAUDE_CODE_USE_VERTEX",
    "CLAUDE_CODE_USE_FOUNDRY",
    "CLAUDE_CONFIG_DIR",
];

pub(crate) fn reveal_provider_secret(codex_home: &Path, label: &str) -> Result<String> {
    if !allowed_provider_vault_label(label) {
        return Err(anyhow!(
            "Vault label `{label}` is not allowed for Claude pane auth"
        ));
    }
    let vault = Vault::new(codex_home.to_path_buf());
    vault
        .reveal(label)
        .with_context(|| format!("failed to read vault credential `{label}`"))
}

pub(crate) fn allowed_provider_vault_label(label: &str) -> bool {
    matches!(
        label,
        "provider/zai_api_key"
            | "provider/anthropic_api_key"
            | "provider/ambient_api_key"
            | "provider/kimi_api_key"
            | "provider/baseten_api_key"
            | "provider/openrouter_api_key"
            | "provider/model_api_key"
            | "provider/ai_gateway_api_key"
    )
}

pub(crate) fn build_claude_command_plan(
    pane: &ClaudePane,
    prompt: String,
    codex_home: &Path,
) -> Result<ClaudeCommandPlan> {
    let profile = pane.profile.profile();
    let turn_index = pane.next_turn_index;
    let settings_path = pane.artifact_dir.join("settings.json");
    let artifact_path = pane
        .artifact_dir
        .join(format!("turn-{turn_index:04}.jsonl"));
    let audit_path = pane
        .artifact_dir
        .join(format!("turn-{turn_index:04}.audit.json"));
    // #218: a contained pane reaches every provider through the bridge, so
    // Claude Code never holds a provider key.
    let containment = super::containment::enabled()
        .map(|settings| -> Result<ClaudeContainment> {
            Ok(ClaudeContainment {
                state_dir: super::containment::state_dir(
                    codex_home,
                    &pane.id,
                    settings.state_root.as_deref(),
                )?,
                panes_dir: codex_home.join("panes"),
                linux_sandbox_exe: settings.linux_sandbox_exe,
            })
        })
        .transpose()?;
    let mut bridge = None;
    let mut base_url_override = None;
    if containment.is_some()
        || matches!(profile.kind, ClaudeProviderProfileKind::ClaudePlan)
        || matches!(
            profile.transport,
            ClaudeProviderTransport::AmbientChatBridge
                | ClaudeProviderTransport::AnthropicPassthroughBridge
        )
    {
        let listener = StdTcpListener::bind("127.0.0.1:0")
            .context("failed to bind Claude bridge loopback listener")?;
        listener
            .set_nonblocking(true)
            .context("failed to set Claude bridge listener nonblocking")?;
        let bind_addr = listener
            .local_addr()
            .context("failed to read Claude bridge listener address")?;
        let (kind, upstream_base_url, upstream_model, deferred_vault_secret) = match profile.kind {
            ClaudeProviderProfileKind::ClaudePlan => (
                ClaudeBridgeKind::AnthropicOauthPassthrough,
                "https://api.anthropic.com".to_string(),
                profile.provider_model.to_string(),
                None,
            ),
            _ if matches!(
                profile.transport,
                ClaudeProviderTransport::AmbientChatBridge
            ) =>
            {
                (
                    ClaudeBridgeKind::AmbientChat,
                    "https://api.ambient.xyz/v1/chat/completions".to_string(),
                    profile.provider_model.to_string(),
                    Some(DeferredVaultSecret {
                        codex_home: codex_home.to_path_buf(),
                        label: profile
                            .vault_label
                            .ok_or_else(|| {
                                anyhow!("Claude bridge requires a provider vault label")
                            })?
                            .to_string(),
                    }),
                )
            }
            _ if matches!(
                profile.transport,
                ClaudeProviderTransport::AnthropicPassthroughBridge
            ) =>
            {
                (
                    ClaudeBridgeKind::AnthropicPassthrough,
                    profile
                        .base_url
                        .ok_or_else(|| anyhow!("Anthropic passthrough bridge requires base URL"))?
                        .trim_end_matches('/')
                        .to_string(),
                    profile.provider_model.to_string(),
                    Some(DeferredVaultSecret {
                        codex_home: codex_home.to_path_buf(),
                        label: profile
                            .vault_label
                            .ok_or_else(|| {
                                anyhow!("Claude bridge requires a provider vault label")
                            })?
                            .to_string(),
                    }),
                )
            }
            // Direct providers, bridged only when contained: the bridge sends
            // the key in both headers, as Claude Code's `apiKeyHelper` did.
            _ => (
                ClaudeBridgeKind::AnthropicApiKeyPassthrough,
                profile
                    .base_url
                    .ok_or_else(|| anyhow!("Claude bridge requires a provider base URL"))?
                    .trim_end_matches('/')
                    .to_string(),
                profile.provider_model.to_string(),
                Some(DeferredVaultSecret {
                    codex_home: codex_home.to_path_buf(),
                    label: profile
                        .vault_label
                        .ok_or_else(|| anyhow!("Claude bridge requires a provider vault label"))?
                        .to_string(),
                }),
            ),
        };
        base_url_override = Some(format!("http://{bind_addr}"));
        bridge = Some(ClaudeBridgePlan {
            kind,
            listener,
            bind_addr,
            client_auth_token: Uuid::new_v4().to_string(),
            upstream_base_url,
            upstream_api_key: None,
            deferred_vault_secret,
            upstream_model,
            accounting_provider_id: profile.accounting_provider_id.map(str::to_string),
        });
    }
    let mut settings = settings_json_with_base_url(
        profile,
        if bridge.is_some() {
            None
        } else {
            Some("corbanu")
        },
        base_url_override.as_deref(),
    );
    if containment.is_some() {
        add_contained_settings(&mut settings);
    }
    std::fs::write(&settings_path, settings.to_string()).with_context(|| {
        format!(
            "failed to write Claude pane settings `{}`",
            settings_path.display()
        )
    })?;
    // A contained pane cannot read `CODEX_HOME/panes`; it gets the same
    // settings from its state folder, read-only there.
    let settings_path = match containment.as_ref() {
        Some(containment) => {
            let path = containment.settings_path();
            std::fs::create_dir_all(&containment.state_dir).with_context(|| {
                format!(
                    "failed to create the Claude pane state folder `{}`",
                    containment.state_dir.display()
                )
            })?;
            // The folder is writable to the pane; replace the file rather
            // than write through whatever is there.
            let mut file = tempfile::NamedTempFile::new_in(&containment.state_dir)
                .context("failed to create Claude pane settings")?;
            std::io::Write::write_all(&mut file, settings.to_string().as_bytes())
                .context("failed to write Claude pane settings")?;
            file.persist(&path)
                .map_err(|err| err.error)
                .with_context(|| {
                    format!("failed to write Claude pane settings `{}`", path.display())
                })?;
            path
        }
        None => settings_path,
    };

    let mut env = BTreeMap::new();
    // Claude Code gives its own OAuth variables precedence over provider-specific
    // apiKeyHelper settings. Never let a subscription credential inherited from
    // the parent shell escape into any pane, including third-party providers.
    let mut env_remove = CLAUDE_CREDENTIAL_ENV_KEYS.map(ToString::to_string).to_vec();
    let deferred_claude_plan_auth = if matches!(profile.kind, ClaudeProviderProfileKind::ClaudePlan)
    {
        env_remove.extend(CLAUDE_PLAN_ROUTING_ENV_KEYS.map(ToString::to_string));
        Some(DeferredClaudePlanAuth {
            codex_home: codex_home.to_path_buf(),
            helper_executable: internal_cli_helper_executable()
                .context("failed to locate Corbanu for Claude Plan authentication")?,
            cwd: pane.cwd.clone(),
            claude_config_dir_override: absolute_claude_config_dir_override()?,
            account: None,
        })
    } else {
        None
    };
    if let Some(base_url) = base_url_override.as_deref().or(profile.base_url) {
        env.insert("ANTHROPIC_BASE_URL".to_string(), base_url.to_string());
    }
    if let Some(bridge) = bridge.as_ref() {
        env_remove.extend(ANTHROPIC_AUTH_ENV_KEYS.map(ToString::to_string));
        env.insert("ANTHROPIC_API_KEY".to_string(), String::new());
        env.insert(
            "ANTHROPIC_AUTH_TOKEN".to_string(),
            bridge.client_auth_token.clone(),
        );
    } else if profile.vault_label.is_some() {
        env_remove.extend(ANTHROPIC_AUTH_ENV_KEYS.map(ToString::to_string));
        env.insert("ANTHROPIC_API_KEY".to_string(), String::new());
    }
    if let Some(containment) = containment.as_ref() {
        // The sandbox lets Claude Code reach only the bridge, as its HTTP
        // proxy (see `containment`); everything else it might call fails.
        if let Some(base_url) = base_url_override.as_deref() {
            env.insert("HTTP_PROXY".to_string(), base_url.to_string());
        }
        env.insert(
            "CLAUDE_CONFIG_DIR".to_string(),
            containment.config_dir().to_string_lossy().into_owned(),
        );
        // Claude Code keeps its own temporary files (the Bash tool's working
        // folder) under `CLAUDE_CODE_TMPDIR`, not `TMPDIR`.
        for name in ["TMPDIR", "CLAUDE_CODE_TMPDIR"] {
            env.insert(
                name.to_string(),
                containment.tmp_dir().to_string_lossy().into_owned(),
            );
        }
        env.insert(
            "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC".to_string(),
            "1".to_string(),
        );
        env.insert("DISABLE_AUTOUPDATER".to_string(), "1".to_string());
    }
    if profile.uses_bare_mode {
        env.insert(
            "ANTHROPIC_MODEL".to_string(),
            profile.claude_model.to_string(),
        );
        env.insert(
            "ANTHROPIC_DEFAULT_OPUS_MODEL".to_string(),
            profile.provider_model.to_string(),
        );
        env.insert(
            "ANTHROPIC_DEFAULT_SONNET_MODEL".to_string(),
            profile.provider_model.to_string(),
        );
        env.insert(
            "ANTHROPIC_DEFAULT_HAIKU_MODEL".to_string(),
            profile.small_model.to_string(),
        );
        env.insert(
            "ANTHROPIC_SMALL_FAST_MODEL".to_string(),
            profile.small_model.to_string(),
        );
        env.insert(
            "CLAUDE_CODE_SUBAGENT_MODEL".to_string(),
            profile.provider_model.to_string(),
        );
        env.insert(
            "CLAUDE_CODE_AUTO_COMPACT_WINDOW".to_string(),
            "1000000".to_string(),
        );
        env.insert("API_TIMEOUT_MS".to_string(), "3000000".to_string());
        env.insert(
            "CLAUDE_CODE_DISABLE_EXPERIMENTAL_BETAS".to_string(),
            "1".to_string(),
        );
        env.insert(
            "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC".to_string(),
            "1".to_string(),
        );
        env.insert(
            "CLAUDE_CODE_DISABLE_NONSTREAMING_FALLBACK".to_string(),
            "1".to_string(),
        );
        env.insert("CLAUDECODE".to_string(), String::new());
    }

    let mut args = Vec::new();
    if profile.uses_bare_mode {
        args.push("--bare".to_string());
    }
    args.extend([
        "-p".to_string(),
        "--output-format".to_string(),
        "stream-json".to_string(),
        "--verbose".to_string(),
        "--settings".to_string(),
        settings_path.to_string_lossy().into_owned(),
        "--exclude-dynamic-system-prompt-sections".to_string(),
        "--model".to_string(),
        profile.claude_model.to_string(),
    ]);
    if containment.is_some() {
        // Claude Code asks before the tools in `CONTAINED_ASK_TOOLS` (the
        // settings' `ask` rules) through stdin/stdout, and Corbanu asks a
        // person (`super::approval`). The prompt then goes in on stdin too.
        // Only the tools in `CONTAINED_TOOLS` exist, so a newer Claude Code's
        // new tools are not available either; subagents, skills, slash
        // commands and tools that start or schedule agents are also denied
        // by name.
        let tools = CONTAINED_TOOLS.join(",");
        let disallowed = CONTAINED_DISALLOWED_TOOLS.join(",");
        args.extend(
            [
                "--permission-mode",
                "default",
                "--permission-prompt-tool",
                "stdio",
                "--input-format",
                "stream-json",
                "--tools",
                tools.as_str(),
                "--disallowedTools",
                disallowed.as_str(),
            ]
            .map(str::to_string),
        );
    } else {
        args.extend([
            "--permission-mode".to_string(),
            "bypassPermissions".to_string(),
        ]);
    }
    if matches!(profile.kind, ClaudeProviderProfileKind::ClaudePlan) {
        args.extend(["--effort".to_string(), "high".to_string()]);
    }
    if containment.is_some() {
        // Settings, hooks and MCP servers a pane could write into its own
        // folder must not run or allow anything without a person: only
        // Corbanu's settings file applies.
        // `--safe-mode` also leaves out CLAUDE.md, skills, plugins, custom
        // commands and agents; Corbanu's `--settings` still apply.
        args.extend(
            [
                "--setting-sources",
                "",
                "--strict-mcp-config",
                "--safe-mode",
            ]
            .map(str::to_string),
        );
    } else {
        args.extend(["--setting-sources".to_string(), "project".to_string()]);
    }
    let (command_mode, command_session_id) = if let Some(session_id) = &pane.claude_session_id {
        args.push("--resume".to_string());
        args.push(session_id.clone());
        (ClaudeCommandMode::Resume, session_id.clone())
    } else {
        let session_id = Uuid::new_v4().to_string();
        args.push("--session-id".to_string());
        args.push(session_id.clone());
        (ClaudeCommandMode::NewSession, session_id)
    };
    let stdin_prompt = if containment.is_some() {
        // A prompt starting with `/` would run a Claude Code command (built-in
        // skills such as `/batch` or `/loop` stay under `--safe-mode`); with a
        // leading space it is plain text for the model.
        Some(if prompt.starts_with('/') {
            format!(" {prompt}")
        } else {
            prompt
        })
    } else {
        args.push(prompt);
        None
    };

    // With a bridge, every send passes through this process and is reported
    // there. Without one, the pane talks to the provider itself and the only
    // account of what it cost is the one the pane gives back.
    let direct_accounting = match (
        bridge.is_none(),
        profile.accounting_provider_id,
        profile.base_url,
    ) {
        (true, Some(provider_id), Some(base_url)) => Some(PaneDirectAccounting {
            provider_id: provider_id.to_string(),
            base_url: format!("{}/v1", base_url.trim_end_matches('/')),
            model: profile.provider_model.to_string(),
        }),
        _ => None,
    };
    Ok(ClaudeCommandPlan {
        executable: "claude".to_string(),
        args,
        env,
        env_remove,
        cwd: pane.cwd.clone(),
        pane_id: pane.id.clone(),
        pane_title: pane.title.clone(),
        profile_title: profile.title.to_string(),
        provider_model: profile.provider_model.to_string(),
        turn_index,
        command_mode,
        command_session_id,
        max_turns: None,
        artifact_path,
        audit_path,
        timeout_ms: None,
        deferred_claude_plan_auth,
        bridge,
        direct_accounting,
        containment,
        stdin_prompt,
    })
}

fn absolute_claude_config_dir_override() -> Result<Option<PathBuf>> {
    let configured = std::env::var_os("CLAUDE_CONFIG_DIR")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from);
    absolute_claude_config_dir_override_against(configured, &std::env::current_dir()?)
}

pub(crate) fn absolute_claude_config_dir_override_against(
    configured: Option<PathBuf>,
    current_dir: &Path,
) -> Result<Option<PathBuf>> {
    configured
        .map(|config_dir| {
            let path = if config_dir.is_absolute() {
                config_dir
            } else {
                current_dir.join(config_dir)
            };
            std::path::absolute(path)
                .context("failed to resolve the exact Claude Code configuration profile")
        })
        .transpose()
}

/// Tools a contained pane always asks a person about, even when Claude Code
/// would consider the call read-only (its read-only Bash classifier has had
/// bypasses). Reading and searching files inside the sandbox stays automatic.
/// Kept in step with `claude_code_regression.py` (qa/security-levels).
pub(crate) const CONTAINED_ASK_TOOLS: [&str; 7] = [
    "Bash",
    "Edit",
    "Write",
    "MultiEdit",
    "NotebookEdit",
    "WebFetch",
    "WebSearch",
];

/// The only tools a contained pane has (`--tools`); names a Claude Code
/// version does not have are ignored.
pub(crate) const CONTAINED_TOOLS: [&str; 11] = [
    "Bash",
    "Read",
    "Edit",
    "Write",
    "MultiEdit",
    "NotebookEdit",
    "Glob",
    "Grep",
    "WebFetch",
    "WebSearch",
    "TodoWrite",
];

/// Tools a contained pane does not get: subagents (whose definitions can ask
/// for other permission modes, hooks and MCP servers), skills and slash
/// commands (`allowed-tools`), and tools that start or schedule more agents.
pub(crate) const CONTAINED_DISALLOWED_TOOLS: [&str; 9] = [
    "Agent",
    "Task",
    "Skill",
    "SlashCommand",
    "Workflow",
    "CronCreate",
    "ScheduleWakeup",
    "SendMessage",
    "EnterWorktree",
];

/// Corbanu's settings for a contained pane: no hooks at all, the ask and deny
/// rules above, and no way into `bypassPermissions`. Checked against the real
/// Claude Code by `claude_code_regression.py`.
fn add_contained_settings(settings: &mut Value) {
    if let Some(settings) = settings.as_object_mut() {
        settings.insert("disableAllHooks".to_string(), Value::Bool(true));
        settings.insert(
            "permissions".to_string(),
            serde_json::json!({
                "ask": CONTAINED_ASK_TOOLS,
                "deny": CONTAINED_DISALLOWED_TOOLS,
                "disableBypassPermissionsMode": "disable",
            }),
        );
    }
}

pub(crate) fn settings_json_with_base_url(
    profile: ClaudeProviderProfile,
    helper_program: Option<&str>,
    base_url_override: Option<&str>,
) -> Value {
    let mut env = serde_json::Map::new();
    if matches!(profile.kind, ClaudeProviderProfileKind::ClaudePlan) {
        env.insert(
            "ANTHROPIC_BASE_URL".to_string(),
            Value::String(
                base_url_override
                    .unwrap_or("https://api.anthropic.com")
                    .to_string(),
            ),
        );
    }
    if profile.uses_bare_mode {
        if let Some(base_url) = base_url_override.or(profile.base_url) {
            env.insert(
                "ANTHROPIC_BASE_URL".to_string(),
                Value::String(base_url.to_string()),
            );
        }
        env.insert(
            "ANTHROPIC_API_KEY".to_string(),
            Value::String(String::new()),
        );
        env.insert(
            "ANTHROPIC_MODEL".to_string(),
            Value::String(profile.claude_model.to_string()),
        );
        env.insert(
            "ANTHROPIC_DEFAULT_OPUS_MODEL".to_string(),
            Value::String(profile.provider_model.to_string()),
        );
        env.insert(
            "ANTHROPIC_DEFAULT_SONNET_MODEL".to_string(),
            Value::String(profile.provider_model.to_string()),
        );
        env.insert(
            "ANTHROPIC_DEFAULT_HAIKU_MODEL".to_string(),
            Value::String(profile.small_model.to_string()),
        );
        env.insert(
            "ANTHROPIC_SMALL_FAST_MODEL".to_string(),
            Value::String(profile.small_model.to_string()),
        );
        env.insert(
            "CLAUDE_CODE_SUBAGENT_MODEL".to_string(),
            Value::String(profile.provider_model.to_string()),
        );
        env.insert(
            "CLAUDE_CODE_AUTO_COMPACT_WINDOW".to_string(),
            Value::String("1000000".to_string()),
        );
        env.insert(
            "API_TIMEOUT_MS".to_string(),
            Value::String("3000000".to_string()),
        );
        env.insert(
            "CLAUDE_CODE_DISABLE_EXPERIMENTAL_BETAS".to_string(),
            Value::String("1".to_string()),
        );
        env.insert(
            "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC".to_string(),
            Value::String("1".to_string()),
        );
        env.insert(
            "CLAUDE_CODE_DISABLE_NONSTREAMING_FALLBACK".to_string(),
            Value::String("1".to_string()),
        );
    }

    let mut settings = serde_json::Map::new();
    settings.insert("env".to_string(), Value::Object(env));
    if profile.uses_bare_mode
        && let (Some(helper_program), Some(label)) = (helper_program, profile.vault_label)
    {
        settings.insert(
            "apiKeyHelper".to_string(),
            Value::String(format!("{helper_program} vault auth-helper {label}")),
        );
    }
    Value::Object(settings)
}

pub(crate) fn prompt_from_user_turn(op: &AppCommand) -> Result<Option<String>> {
    let AppCommand::UserTurn { items, .. } = op else {
        return Ok(None);
    };
    let mut chunks = Vec::new();
    for item in items {
        match item {
            UserInput::Text { text, .. } => chunks.push(text.clone()),
            UserInput::Skill { name, path } => {
                chunks.push(format!("[Selected skill: {name} at {}]", path.display()))
            }
            UserInput::Mention { name, path } => {
                chunks.push(format!("[Mention: {name} at {path}]"));
            }
            UserInput::Image { .. }
            | UserInput::LocalImage { .. }
            | UserInput::Audio { .. }
            | UserInput::LocalAudio { .. } => {
                return Err(anyhow!(
                    "Claude panes currently accept text, skills, and mentions only; media input is not supported yet."
                ));
            }
        }
    }
    Ok(Some(chunks.join("\n\n")))
}

pub(crate) fn compose_claude_pane_prompt(prompt: String, spawn_context: Option<&str>) -> String {
    let Some(spawn_context) = spawn_context
        .map(str::trim)
        .filter(|context| !context.is_empty())
    else {
        return prompt;
    };
    format!("{spawn_context}\n\nUser message:\n{prompt}")
}

pub(crate) fn claude_pane_title(
    profile: ClaudeProviderProfileKind,
    spawn_role: Option<SpawnRole>,
    spawn_nickname: Option<&str>,
) -> String {
    match (spawn_role, spawn_nickname) {
        (Some(role), Some(nickname)) => format!(
            "Claude Code {} [{}] - {}",
            nickname,
            role.agent_type().unwrap_or_else(|| role.label()),
            profile.status_model_label()
        ),
        (Some(role), None) => format!(
            "Claude Code {} - {}",
            role.label(),
            profile.status_model_label()
        ),
        (None, _) => profile.profile().title.to_string(),
    }
}
