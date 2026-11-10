//! Applies bounded agent-role overrides to an existing session config.
//!
//! Roles may customize the child or reduce its capabilities, but never replace the parent
//! session's authority. A projected layer keeps existing layer-based consumers in sync.
//!
//! Corbanu (D526 (a), provisional): besides upstream's fields a role may set base instructions
//! (the crew roles) and choose a provider, which keeps the parent's account and the developer
//! accounting wire. Permission settings may only make the child stricter: `allow_login_shell =
//! false` and upstream's feature/skill disables apply; the parent's sandbox, approvals, network,
//! raised Windows sandbox and depth limit are kept, and other keys are reported at startup.
//! A project's role files get the project-config denylist (D512).

use crate::config::AccountingMode;
use crate::config::AgentRoleConfig;
use crate::config::AgentRoleSource;
use crate::config::Config;
use crate::config::deserialize_config_toml_with_base;
use anyhow::anyhow;
use codex_agent_roles::parse_agent_role_file_contents;
use codex_agent_roles::project_role_ignored_keys_warning;
use codex_agent_roles::sanitize_role_config_layer;
use codex_agent_roles::unapplied_role_keys;
use codex_agent_roles::unapplied_role_keys_warning;
use codex_config::ConfigLayerEntry;
use codex_config::ConfigLayerSource;
use codex_config::ConfigLayerStack;
use codex_config::SkillsConfig;
use codex_config::loader::resolve_relative_paths_in_config_toml;
use codex_exec_server::read_sensitive_file_to_string;
use codex_features::Feature;
use codex_features::feature_for_key;
use codex_model_provider_info::ModelProviderInfo;
use codex_model_provider_info::WireApi;
use codex_protocol::config_types::Personality;
use codex_protocol::config_types::ReasoningSummary;
use codex_protocol::config_types::ServiceTier;
use codex_protocol::config_types::Verbosity;
use codex_protocol::models::BaseInstructionsProvenance;
use codex_protocol::openai_models::ReasoningEffort;
use codex_protocol::protocol::AskForApproval;
use codex_protocol::protocol::SandboxPolicy;
use serde::Serialize;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::path::Path;
use std::str::FromStr;
use std::sync::LazyLock;
use toml::Value as TomlValue;

/// The role name used when a caller omits `agent_type`.
pub const DEFAULT_ROLE_NAME: &str = "default";
const AGENT_TYPE_UNAVAILABLE_ERROR: &str = "agent type is currently not available";
const AGENT_NAMES: &str = include_str!("../../assets/agent/agent_names.txt");

fn default_agent_nickname_list() -> Vec<&'static str> {
    AGENT_NAMES
        .lines()
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .collect()
}

pub(crate) fn agent_nickname_candidates(config: &Config, role_name: Option<&str>) -> Vec<String> {
    let role_name = role_name.unwrap_or(DEFAULT_ROLE_NAME);
    if let Some(candidates) =
        resolve_role_config(config, role_name).and_then(|role| role.nickname_candidates.clone())
    {
        return candidates;
    }

    default_agent_nickname_list()
        .into_iter()
        .map(ToOwned::to_owned)
        .collect()
}

#[derive(Default, Serialize)]
struct AgentRoleOverrides {
    developer_instructions: Option<String>,
    /// Corbanu: base instructions (the crew roles' `base_instructions`).
    instructions: Option<String>,
    model: Option<String>,
    /// Corbanu: the provider this role selects.
    model_provider: Option<String>,
    model_reasoning_effort: Option<ReasoningEffort>,
    model_reasoning_summary: Option<ReasoningSummary>,
    model_verbosity: Option<Verbosity>,
    personality: Option<Personality>,
    service_tier: Option<String>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    features: BTreeMap<String, bool>,
    skills: Option<SkillsConfig>,
    /// Corbanu: only `false` is kept, so a role can't re-enable login shells.
    allow_login_shell: Option<bool>,
    /// Corbanu: the role's `[model_providers.<selected id>]` table. Not projected into the
    /// layer stack, which may be shown to the user and can hold credentials.
    #[serde(skip)]
    model_provider_overlay: Option<TomlValue>,
}

/// Applies typed role overrides to the existing parent-derived configuration.
pub(crate) async fn apply_role_to_config(
    config: &mut Config,
    role_name: Option<&str>,
) -> Result<(), String> {
    let role_name = role_name.unwrap_or(DEFAULT_ROLE_NAME);

    let role = resolve_role_config(config, role_name)
        .cloned()
        .ok_or_else(|| format!("unknown agent_type '{role_name}'"))?;

    apply_role_to_config_inner(config, role_name, &role)
        .await
        .map_err(|err| {
            tracing::warn!("failed to apply role to config: {err}");
            AGENT_TYPE_UNAVAILABLE_ERROR.to_string()
        })
}

/// Applies a v2 role. Kept for its callers: the role is applied to a clone of the caller's
/// config, so instructions the caller selected survive unless the role sets its own.
pub(crate) async fn apply_role_to_config_for_multi_agent_v2(
    config: &mut Config,
    role_name: Option<&str>,
) -> Result<(), String> {
    apply_role_to_config(config, role_name).await
}

async fn apply_role_to_config_inner(
    config: &mut Config,
    role_name: &str,
    role: &AgentRoleConfig,
) -> anyhow::Result<()> {
    let is_built_in = !config.agent_roles.contains_key(role_name);
    let Some(config_file) = role.config_file.as_ref() else {
        return Ok(());
    };
    let role_layer_toml =
        load_role_layer_toml(config, config_file, is_built_in, role.source, role_name).await?;
    reject_stricter_unapplied_permissions(config, &role_layer_toml)?;
    let unapplied_keys =
        unapplied_role_keys(&role_layer_toml, Some(config.model_provider_id.as_str()));
    if !unapplied_keys.is_empty() {
        tracing::warn!(
            "{}",
            unapplied_role_keys_warning(config_file, &unapplied_keys)
        );
    }
    let selected_provider_id = role_layer_toml
        .get("model_provider")
        .and_then(TomlValue::as_str)
        .unwrap_or(&config.model_provider_id)
        .to_owned();
    let model_provider_overlay = role_layer_toml
        .get("model_providers")
        .and_then(|providers| providers.get(&selected_provider_id))
        .cloned();
    let role_config = deserialize_config_toml_with_base(role_layer_toml, &config.codex_home)?;
    let mut overrides = AgentRoleOverrides {
        developer_instructions: role_config.developer_instructions,
        instructions: role_config.instructions,
        model: role_config.model,
        model_provider: role_config.model_provider,
        model_reasoning_effort: role_config.model_reasoning_effort,
        model_reasoning_summary: role_config.model_reasoning_summary,
        model_verbosity: role_config.model_verbosity,
        personality: role_config.personality,
        service_tier: role_config.service_tier,
        allow_login_shell: role_config.allow_login_shell.filter(|allowed| !allowed),
        model_provider_overlay,
        ..Default::default()
    };

    if let Some(features) = role_config.features {
        for (key, enabled) in features.entries() {
            if !enabled
                && let Some(
                    feature @ (Feature::ShellTool
                    | Feature::Apps
                    | Feature::Plugins
                    | Feature::MemoryTool
                    | Feature::RequestPermissionsTool),
                ) = feature_for_key(&key)
            {
                overrides.features.insert(feature.key().to_string(), false);
            }
        }
    }
    if let Some(mut skills) = role_config.skills {
        skills.config.retain(|skill| !skill.enabled);
        skills.bundled = skills.bundled.filter(|bundled| !bundled.enabled);
        skills.include_instructions = skills.include_instructions.filter(|enabled| !enabled);
        skills.max_context_tokens = None;
        if !skills.config.is_empty()
            || skills.bundled.is_some()
            || skills.include_instructions.is_some()
        {
            overrides.skills = Some(skills);
        }
    }

    let role_layer_toml = TomlValue::try_from(&overrides)?;
    if role_layer_toml
        .as_table()
        .is_some_and(toml::map::Map::is_empty)
        && overrides.model_provider_overlay.is_none()
    {
        return Ok(());
    }
    let mut next_config = role_overrides::build_next_config(config, role_layer_toml, &overrides)?;
    // Corbanu: a built-in crew role that pins no reasoning effort takes the configured effort,
    // not the caller's runtime one, so an Orc spawned from an xhigh Troll doesn't silently
    // inherit xhigh. A user role keeps the caller's runtime effort.
    if is_built_in && overrides.model_reasoning_effort.is_none() {
        next_config.model_reasoning_effort = next_config
            .config_layer_stack
            .effective_config()
            .get("model_reasoning_effort")
            .and_then(TomlValue::as_str)
            .and_then(|value| ReasoningEffort::from_str(value).ok());
    }
    *config = next_config;
    Ok(())
}

/// Corbanu (D526 (a), provisional): a role's permission settings are not applied to the child,
/// which keeps its parent's. A setting that would be looser is ignored (reported at startup); one
/// that would be stricter, or that can't be compared, fails the spawn so the child never runs
/// looser than its role asked for.
fn reject_stricter_unapplied_permissions(config: &Config, role: &TomlValue) -> anyhow::Result<()> {
    if let Some(mode) = role.get("sandbox_mode") {
        let rank = |mode: &str| match mode {
            "read-only" => Some(0),
            "workspace-write" => Some(1),
            "danger-full-access" => Some(2),
            _ => None,
        };
        let parent_rank = match config
            .permissions
            .legacy_sandbox_policy(config.cwd.as_path())
        {
            SandboxPolicy::ReadOnly { .. } => 0,
            SandboxPolicy::WorkspaceWrite { .. } => 1,
            SandboxPolicy::DangerFullAccess | SandboxPolicy::ExternalSandbox { .. } => 2,
        };
        anyhow::ensure!(
            mode.as_str()
                .and_then(rank)
                .is_some_and(|rank| rank >= parent_rank),
            "this role asks for a stricter sandbox_mode than its parent's, which spawned agents \
             don't apply yet; the spawn is refused rather than run with the parent's sandbox"
        );
    }
    if let Some(policy) = role.get("approval_policy") {
        let policy: AskForApproval = policy.clone().try_into()?;
        let parent = config.permissions.approval_policy.value();
        // How often each policy asks; granular policies can't be compared.
        let rank = |policy: &AskForApproval| match policy {
            AskForApproval::Never => Some(0),
            AskForApproval::OnRequest => Some(1),
            AskForApproval::UnlessTrusted => Some(2),
            AskForApproval::Granular(_) => None,
        };
        anyhow::ensure!(
            policy == parent
                || rank(&policy)
                    .zip(rank(&parent))
                    .is_some_and(|(role, parent)| role <= parent),
            "this role asks for a stricter approval_policy than its parent's, which spawned \
             agents don't apply yet; the spawn is refused rather than run with the parent's policy"
        );
    }
    let role_network_off = role
        .get("sandbox_workspace_write")
        .and_then(|table| table.get("network_access"))
        .and_then(TomlValue::as_bool)
        == Some(false);
    anyhow::ensure!(
        !role_network_off || !config.permissions.network_sandbox_policy().is_enabled(),
        "this role turns network access off, which spawned agents don't apply yet; the spawn is \
         refused rather than run with the parent's network access"
    );
    if let Some(profile) = role.get("default_permissions") {
        let parent = config.permissions.active_permission_profile();
        anyhow::ensure!(
            profile.as_str().is_some_and(|profile| {
                parent
                    .as_ref()
                    .is_some_and(|parent| parent.id.as_str() == profile)
            }),
            "this role selects a permission profile other than its parent's, which spawned \
             agents don't apply; the spawn is refused rather than run with the parent's profile"
        );
    }
    if let Some(mode) = role
        .get("windows")
        .and_then(|windows| windows.get("sandbox"))
        .and_then(TomlValue::as_str)
    {
        let parent = config
            .permissions
            .windows_sandbox_mode
            .and_then(|mode| TomlValue::try_from(mode).ok());
        anyhow::ensure!(
            mode == "unelevated" || parent.as_ref().and_then(TomlValue::as_str) == Some(mode),
            "this role asks for a different Windows sandbox than its parent's, which spawned \
             agents don't apply; the spawn is refused rather than run with the parent's sandbox"
        );
    }
    Ok(())
}

async fn load_role_layer_toml(
    config: &Config,
    config_file: &Path,
    is_built_in: bool,
    source: AgentRoleSource,
    role_name: &str,
) -> anyhow::Result<TomlValue> {
    let (role_config_toml, role_config_base) = if is_built_in {
        let role_config_contents = built_in::config_file_contents(config_file)
            .map(str::to_owned)
            .ok_or(anyhow!("No corresponding config content"))?;
        let role_config_toml: TomlValue = toml::from_str(&role_config_contents)?;
        (role_config_toml, config.codex_home.as_path())
    } else {
        let role_config_contents = read_sensitive_file_to_string(config_file).await?;
        let role_config_base = config_file
            .parent()
            .ok_or(anyhow!("No corresponding config content"))?;
        let mut role_config_toml = parse_agent_role_file_contents(
            &role_config_contents,
            config_file,
            role_config_base,
            Some(role_name),
        )?
        .config;
        // The file is read again at spawn, so the denylist is applied here, not
        // only when roles are discovered (D512, #1997).
        let ignored_keys = sanitize_role_config_layer(source, &mut role_config_toml);
        if !ignored_keys.is_empty() {
            tracing::warn!(
                "{}",
                project_role_ignored_keys_warning(config_file, &ignored_keys)
            );
        }
        (role_config_toml, role_config_base)
    };

    deserialize_config_toml_with_base(role_config_toml.clone(), role_config_base)?;
    Ok(resolve_relative_paths_in_config_toml(
        role_config_toml,
        role_config_base,
    )?)
}

pub(crate) fn resolve_role_config<'a>(
    config: &'a Config,
    role_name: &str,
) -> Option<&'a AgentRoleConfig> {
    config
        .agent_roles
        .get(role_name)
        .or_else(|| built_in::configs().get(role_name))
}

mod role_overrides {
    use super::*;

    pub(super) fn build_next_config(
        config: &Config,
        role_layer_toml: TomlValue,
        overrides: &AgentRoleOverrides,
    ) -> anyhow::Result<Config> {
        let mut next_config = config.clone();
        next_config.config_layer_stack = build_config_layer_stack(config, &role_layer_toml)?;
        if let Some(model) = &overrides.model {
            next_config.model = Some(model.clone());
        }
        if let Some(instructions) = &overrides.developer_instructions {
            next_config.developer_instructions = Some(instructions.clone());
        }
        if let Some(effort) = overrides.model_reasoning_effort.clone() {
            next_config.model_reasoning_effort = Some(effort);
        }
        if let Some(summary) = overrides.model_reasoning_summary {
            next_config.model_reasoning_summary = Some(summary);
        }
        if let Some(verbosity) = overrides.model_verbosity {
            next_config.model_verbosity = Some(verbosity);
        }
        if let Some(personality) = overrides.personality {
            next_config.personality = Some(personality);
        }
        if let Some(service_tier) = &overrides.service_tier {
            next_config.service_tier = match ServiceTier::from_request_value(service_tier) {
                Some(ServiceTier::Fast) => next_config
                    .features
                    .enabled(Feature::FastMode)
                    .then(|| ServiceTier::Fast.request_value().to_string()),
                Some(ServiceTier::Flex) => Some(ServiceTier::Flex.request_value().to_string()),
                None if service_tier == "ultrafast" => next_config
                    .features
                    .enabled(Feature::UltrafastMode)
                    .then(|| service_tier.clone()),
                None => Some(service_tier.clone()),
            };
        }
        for key in overrides.features.keys() {
            if let Some(feature) = feature_for_key(key) {
                next_config.features.disable(feature)?;
            }
        }
        if overrides
            .skills
            .as_ref()
            .is_some_and(|skills| skills.include_instructions == Some(false))
        {
            next_config.include_skill_instructions = false;
        }
        let strips_baked_personality =
            |config: &Config| config.personality == Some(Personality::None);
        if strips_baked_personality(config) != strips_baked_personality(&next_config)
            && matches!(
                config.base_instructions_provenance,
                Some(BaseInstructionsProvenance::Model { .. })
            )
        {
            next_config.base_instructions = None;
            next_config.base_instructions_provenance = None;
        }
        if let Some(instructions) = &overrides.instructions {
            next_config.base_instructions = Some(instructions.clone());
            next_config.base_instructions_provenance = Some(BaseInstructionsProvenance::Custom);
        }
        if overrides.allow_login_shell == Some(false) {
            next_config.permissions.allow_login_shell = false;
        }
        if overrides.model_provider.is_some() || overrides.model_provider_overlay.is_some() {
            apply_role_provider(config, &mut next_config, overrides)?;
        }
        Ok(next_config)
    }

    /// The wire the parent's developer accounting is bound to, if any.
    fn accounting_wire(config: &Config) -> Option<WireApi> {
        match &config.accounting {
            AccountingMode::Provider { .. } => Some(config.model_provider.wire_api),
            AccountingMode::DirectAnthropic { .. } if config.model_provider_id == "anthropic" => {
                Some(WireApi::Anthropic)
            }
            AccountingMode::DirectOpenAiResponsesHttp { .. }
            | AccountingMode::DirectOpenAiResponses { .. }
                if config.model_provider_id == "openai" =>
            {
                Some(WireApi::Responses)
            }
            AccountingMode::DirectOpenAiChat { .. } if config.model_provider_id == "openai" => {
                Some(WireApi::Chat)
            }
            _ => None,
        }
    }

    /// Corbanu (D526 (a)): the role's provider choice, limited to providers the parent has
    /// configured and to what managed requirements allow. The parent's live provider is the
    /// base for its own id and the parent's configured entry for another id; the role's table
    /// for that id is overlaid, including nested settings. The account always comes from that
    /// base, never from the role (PF-84), and on the parent's own provider the developer
    /// accounting binding is kept: a role can't change the wire it was approved for. Endpoint
    /// changes remain visible to the existing pre-send rejection.
    fn apply_role_provider(
        config: &Config,
        next_config: &mut Config,
        overrides: &AgentRoleOverrides,
    ) -> anyhow::Result<()> {
        let provider_id = overrides
            .model_provider
            .clone()
            .unwrap_or_else(|| config.model_provider_id.clone());
        let same_provider = provider_id == config.model_provider_id;
        // Managed requirements pin the provider and its definition, as they did when a role
        // rebuilt the config from layers.
        if let Some(required) = config.config_layer_stack.required_model_provider() {
            anyhow::ensure!(
                required == provider_id,
                "managed requirements select model provider `{required}`; this role can't select \
                 `{provider_id}`"
            );
        }
        anyhow::ensure!(
            overrides.model_provider_overlay.is_none()
                || !config
                    .config_layer_stack
                    .requirements_toml()
                    .model_providers
                    .as_ref()
                    .is_some_and(|providers| providers.contains_key(&provider_id)),
            "managed requirements define model provider `{provider_id}`; this role can't change it"
        );
        anyhow::ensure!(
            same_provider || overrides.model.is_some(),
            "this role selects model provider `{provider_id}` without a model; set `model` too"
        );
        // Only providers the parent has configured: a child rebuilt from its layers must be
        // able to resolve the provider again.
        let base = if same_provider {
            config.model_provider.clone()
        } else {
            config
                .model_providers
                .get(&provider_id)
                .cloned()
                .ok_or_else(|| {
                    anyhow!(
                        "this role selects model provider `{provider_id}`, which is not configured"
                    )
                })?
        };
        let mut provider = TomlValue::try_from(&base)?;
        if let Some(overlay) = &overrides.model_provider_overlay {
            codex_config::merge_toml_values(&mut provider, overlay);
        }
        let mut provider: ModelProviderInfo = provider.try_into().map_err(|err| {
            anyhow!("this role's model provider `{provider_id}` is not usable: {err}")
        })?;
        provider.validate().map_err(anyhow::Error::msg)?;
        // As on main: the binding follows the parent's provider; another provider is
        // rebound per turn by the accounting route.
        if same_provider && let Some(accounting_wire) = accounting_wire(config) {
            anyhow::ensure!(
                provider.wire_api == accounting_wire,
                "this role changes the provider's wire_api, which developer accounting cannot follow; \
                 remove wire_api from the role's [model_providers] overlay"
            );
        }
        // Runtime-only fields are not serialized, so they come from the base.
        provider.account = base.account;
        provider.include_internal_metadata = base.include_internal_metadata;
        next_config
            .model_providers
            .insert(provider_id.clone(), provider.clone());
        next_config.model_provider_id = provider_id;
        next_config.model_provider = provider;
        Ok(())
    }

    fn build_config_layer_stack(
        config: &Config,
        role_layer_toml: &TomlValue,
    ) -> anyhow::Result<ConfigLayerStack> {
        let mut layers: Vec<_> = config
            .config_layer_stack
            .all_layers_low_to_high()
            .cloned()
            .collect();
        let role_layer =
            ConfigLayerEntry::new(ConfigLayerSource::SessionFlags, role_layer_toml.clone());
        let insertion_index = layers.partition_point(|layer| layer.name <= role_layer.name);
        layers.insert(insertion_index, role_layer);
        Ok(ConfigLayerStack::new(
            layers,
            config.config_layer_stack.requirements().clone(),
            config.config_layer_stack.requirements_toml().clone(),
        )?
        .with_cloud_config_binding(config.config_layer_stack.cloud_config_binding().cloned())
        .with_mcp_ema_cancellation(config.config_layer_stack.mcp_ema_cancellation().clone())
        .with_applied_config_server(config.config_layer_stack.applied_config_server().cloned())
        .with_user_and_project_exec_policy_rules_ignored(
            config
                .config_layer_stack
                .ignore_user_and_project_exec_policy_rules(),
        ))
    }
}

pub(crate) mod spawn_tool_spec {
    use super::*;

    /// Builds the spawn-agent tool description text from built-in and configured roles.
    pub(crate) fn build(user_defined_agent_roles: &BTreeMap<String, AgentRoleConfig>) -> String {
        let built_in_roles = built_in::configs();
        build_from_configs(built_in_roles, user_defined_agent_roles)
    }

    // This function is not inlined for testing purpose.
    fn build_from_configs(
        built_in_roles: &BTreeMap<String, AgentRoleConfig>,
        user_defined_roles: &BTreeMap<String, AgentRoleConfig>,
    ) -> String {
        let mut seen = BTreeSet::new();
        let mut formatted_roles = Vec::new();
        for (name, declaration) in user_defined_roles {
            if seen.insert(name.as_str()) {
                formatted_roles.push(format_role(name, declaration));
            }
        }
        for (name, declaration) in built_in_roles {
            if seen.insert(name.as_str()) {
                formatted_roles.push(format_role(name, declaration));
            }
        }

        format!("Available roles:\n{}", formatted_roles.join("\n"))
    }

    fn format_role(name: &str, declaration: &AgentRoleConfig) -> String {
        if let Some(description) = &declaration.description {
            let locked_settings_note = declaration
                .config_file
                .as_ref()
                .and_then(|config_file| {
                    built_in::config_file_contents(config_file)
                        .map(str::to_owned)
                        .or_else(|| std::fs::read_to_string(config_file).ok())
                })
                .and_then(|contents| toml::from_str::<TomlValue>(&contents).ok())
                .map(|role_toml| {
                    let model = role_toml
                        .get("model")
                        .and_then(TomlValue::as_str);
                    let reasoning_effort = role_toml
                        .get("model_reasoning_effort")
                        .and_then(TomlValue::as_str);
                    match (model, reasoning_effort) {
                        (Some(model), Some(reasoning_effort)) => format!(
                            "\n- This role's model is set to `{model}` and its reasoning effort is set to `{reasoning_effort}`. These settings cannot be changed."
                        ),
                        (Some(model), None) => {
                            format!(
                                "\n- This role's model is set to `{model}` and cannot be changed."
                            )
                        }
                        (None, Some(reasoning_effort)) => {
                            format!(
                                "\n- This role's reasoning effort is set to `{reasoning_effort}` and cannot be changed."
                            )
                        }
                        (None, None) => String::new(),
                    }
                })
                .unwrap_or_default();
            format!("{name}: {{\n{description}{locked_settings_note}\n}}")
        } else {
            format!("{name}: no description")
        }
    }
}

mod built_in {
    use super::*;

    /// Returns the cached built-in role declarations defined in this module.
    pub(super) fn configs() -> &'static BTreeMap<String, AgentRoleConfig> {
        static CONFIG: LazyLock<BTreeMap<String, AgentRoleConfig>> = LazyLock::new(|| {
            BTreeMap::from([
                (
                    DEFAULT_ROLE_NAME.to_string(),
                    AgentRoleConfig {
                        description: Some("Default agent.".to_string()),
                        config_file: None,
                        nickname_candidates: None,
                        source: AgentRoleSource::BuiltIn,
                    }
                ),
                (
                    "explorer".to_string(),
                    AgentRoleConfig {
                        description: Some(r#"Use `explorer` for specific codebase questions.
Explorers are fast and authoritative.
They must be used to ask specific, well-scoped questions on the codebase.
Rules:
- In order to avoid redundant work, you should avoid exploring the same problem that explorers have already covered. Typically, you should trust the explorer results without additional verification. You are still allowed to inspect the code yourself to gain the needed context!
- You are encouraged to spawn up multiple explorers in parallel when you have multiple distinct questions to ask about the codebase that can be answered independently. This allows you to get more information faster without waiting for one question to finish before asking the next. While waiting for the explorer results, you can continue working on other local tasks that do not depend on those results. This parallelism is a key advantage of delegation, so use it whenever you have multiple questions to ask.
- Reuse existing explorers for related questions."#.to_string()),
                        config_file: Some("explorer.toml".to_string().parse().unwrap_or_default()),
                        nickname_candidates: None,
                        source: AgentRoleSource::BuiltIn,
                    }
                ),
                (
                    "worker".to_string(),
                    AgentRoleConfig {
                        description: Some(r#"Use for execution and production work.
Typical tasks:
- Implement part of a feature
- Fix tests or bugs
- Split large refactors into independent chunks
Rules:
- Explicitly assign **ownership** of the task (files / responsibility). When the subtask involves code changes, you should clearly specify which files or modules the worker is responsible for. This helps avoid merge conflicts and ensures accountability. For example, you can say "Worker 1 is responsible for updating the authentication module, while Worker 2 will handle the database layer." By defining clear ownership, you can delegate more effectively and reduce coordination overhead.
- Always tell workers they are **not alone in the codebase**, and they should not revert the edits made by others, and they should adjust their implementation to accommodate the changes made by others. This is important because there may be multiple workers making changes in parallel, and they need to be aware of each other's work to avoid conflicts and ensure a cohesive final product."#.to_string()),
                        config_file: None,
                        nickname_candidates: None,
                        source: AgentRoleSource::BuiltIn,
                    }
                ),
                (
                    "nazgul".to_string(),
                    AgentRoleConfig {
                        description: Some(r#"Use `nazgul` for CTO-level architecture, delegation, security review, and orchestration.
Nazgul report to Sauron/the human, translate the user's vision into implementation blueprints, delegate execution to Trolls rather than Orc ICs, and do not act as individual contributors."#.to_string()),
                        config_file: Some("nazgul.toml".to_string().parse().unwrap_or_default()),
                        nickname_candidates: Some(vec![
                            "Angmar".to_string(),
                            "Khamul".to_string(),
                            "Morgul".to_string(),
                            "DolGuldur".to_string(),
                            "MinasMorgul".to_string(),
                            "Guldur".to_string(),
                            "Nurn".to_string(),
                            "Cirith".to_string(),
                            "Barad".to_string(),
                        ]),
                        source: AgentRoleSource::BuiltIn,
                    }
                ),
                (
                    "troll".to_string(),
                    AgentRoleConfig {
                        description: Some(r#"Use `troll` for engineering-manager supervision, review, coordination, and enforcement.
Trolls report to the Nazgul CTO, manage Orc IC executors, prefer delegation over implementation, must wait for Orcs to finish before claiming completion, and must critically review Orc output before reporting final results."#.to_string()),
                        config_file: Some("troll.toml".to_string().parse().unwrap_or_default()),
                        nickname_candidates: Some(vec![
                            "Burzum".to_string(),
                            "Durbat".to_string(),
                            "Lugburz".to_string(),
                            "Ashdurb".to_string(),
                            "Ghashlug".to_string(),
                            "Krimpat".to_string(),
                            "Durburz".to_string(),
                            "Ugburz".to_string(),
                            "Burzglob".to_string(),
                            "Skailug".to_string(),
                        ]),
                        source: AgentRoleSource::BuiltIn,
                    }
                ),
                (
                    "orc".to_string(),
                    AgentRoleConfig {
                        description: Some(r#"Use `orc` for direct IC execution work under a supervising Troll.
Orcs follow the Troll's assignment exactly, do not expand scope, produce concrete evidence such as changed files, tests, benchmark output, or findings, and must not spawn child agents."#.to_string()),
                        config_file: Some("orc.toml".to_string().parse().unwrap_or_default()),
                        nickname_candidates: Some(vec![
                            "Snaga".to_string(),
                            "Ghash".to_string(),
                            "Krimp".to_string(),
                            "Lug".to_string(),
                            "Ruk".to_string(),
                            "Uzg".to_string(),
                            "Mauh".to_string(),
                            "Thrak".to_string(),
                            "Narg".to_string(),
                            "Glob".to_string(),
                        ]),
                        source: AgentRoleSource::BuiltIn,
                    }
                ),
                // Awaiter is temp removed
//                 (
//                     "awaiter".to_string(),
//                     AgentRoleConfig {
//                         description: Some(r#"Use an `awaiter` agent EVERY TIME you must run a command that will take some very long time.
// This includes, but not only:
// * testing
// * monitoring of a long running process
// * explicit ask to wait for something
//
// Rules:
// - When an awaiter is running, you can work on something else. If you need to wait for its completion, use the largest possible timeout.
// - Be patient with the `awaiter`.
// - Do not use an awaiter for every compilation/test if it won't take time. Only use if for long running commands.
// - Close the awaiter when you're done with it."#.to_string()),
//                         config_file: Some("awaiter.toml".to_string().parse().unwrap_or_default()),
//                     }
//                 )
            ])
        });
        &CONFIG
    }

    /// Resolves a built-in role `config_file` path to embedded content.
    pub(super) fn config_file_contents(path: &Path) -> Option<&'static str> {
        const EXPLORER: &str = include_str!("../../assets/agent/builtins/explorer.toml");
        const AWAITER: &str = include_str!("../../assets/agent/builtins/awaiter.toml");
        const NAZGUL: &str = concat!(
            include_str!("../../assets/agent/builtins/nazgul.toml"),
            "\nbase_instructions = '''",
            include_str!("../../assets/agent/builtins/nazgul_base.md"),
            "'''\n"
        );
        const TROLL: &str = concat!(
            include_str!("../../assets/agent/builtins/troll.toml"),
            "\nbase_instructions = '''",
            include_str!("../../assets/agent/builtins/troll_base.md"),
            "'''\n"
        );
        const ORC: &str = concat!(
            include_str!("../../assets/agent/builtins/orc.toml"),
            "\nbase_instructions = '''",
            include_str!("../../assets/agent/builtins/orc_base.md"),
            "'''\n"
        );
        match path.to_str()? {
            "explorer.toml" => Some(EXPLORER),
            "awaiter.toml" => Some(AWAITER),
            "nazgul.toml" => Some(NAZGUL),
            "troll.toml" => Some(TROLL),
            "orc.toml" => Some(ORC),
            _ => None,
        }
    }
}

#[cfg(test)]
#[path = "role_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "role_project_tests.rs"]
mod project_tests;
