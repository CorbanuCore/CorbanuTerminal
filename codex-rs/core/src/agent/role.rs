//! Applies bounded agent-role overrides to an existing session config.
//!
//! Roles may customize the child or reduce its capabilities, but never replace the parent
//! session's authority. A projected layer keeps existing layer-based consumers in sync.

use crate::config::AgentRoleConfig;
use crate::config::Config;
use crate::config::deserialize_config_toml_with_base;
use anyhow::anyhow;
use codex_agent_roles::parse_agent_role_file_contents;
use codex_config::ConfigLayerEntry;
use codex_config::ConfigLayerSource;
use codex_config::ConfigLayerStack;
use codex_config::SkillsConfig;
use codex_config::loader::resolve_relative_paths_in_config_toml;
<<<<<<< HEAD
use codex_exec_server::LOCAL_FS;
use codex_protocol::openai_models::ReasoningEffort;
||||||| 413492cd6c
use codex_exec_server::LOCAL_FS;
=======
use codex_exec_server::read_sensitive_file_to_string;
use codex_features::Feature;
use codex_features::feature_for_key;
use codex_protocol::config_types::Personality;
use codex_protocol::config_types::ReasoningSummary;
use codex_protocol::config_types::ServiceTier;
use codex_protocol::config_types::Verbosity;
use codex_protocol::models::BaseInstructionsProvenance;
use codex_protocol::openai_models::ReasoningEffort;
use serde::Serialize;
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::path::Path;
use std::str::FromStr;
use std::sync::LazyLock;
use toml::Value as TomlValue;

/// The role name used when a caller omits `agent_type`.
pub const DEFAULT_ROLE_NAME: &str = "default";
const AGENT_TYPE_UNAVAILABLE_ERROR: &str = "agent type is currently not available";
const AGENT_NAMES: &str = include_str!("agent_names.txt");

<<<<<<< HEAD
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

/// Applies a named role layer to `config` while preserving caller-owned runtime settings.
///
/// The role layer is inserted at session-flag precedence so it can override persisted config, but
/// the caller's current `model`, `model_provider`, and `service_tier` remain sticky runtime
/// choices unless the role explicitly sets the corresponding top-level config key. Rebuilding the
/// config without those overrides would make a spawned agent silently fall back to default settings.
||||||| 413492cd6c
/// Applies a named role layer to `config` while preserving caller-owned provider settings.
///
/// The role layer is inserted at session-flag precedence so it can override persisted config, but
/// the caller's current `model_provider` and `service_tier` remain sticky runtime choices unless
/// the role explicitly sets the corresponding top-level config key. Rebuilding the config without
/// those overrides would make a spawned agent silently fall back to default settings.
=======
#[derive(Default, Serialize)]
struct AgentRoleOverrides {
    developer_instructions: Option<String>,
    model: Option<String>,
    model_reasoning_effort: Option<ReasoningEffort>,
    model_reasoning_summary: Option<ReasoningSummary>,
    model_verbosity: Option<Verbosity>,
    personality: Option<Personality>,
    service_tier: Option<String>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    features: BTreeMap<String, bool>,
    skills: Option<SkillsConfig>,
}

/// Applies typed role overrides to the existing parent-derived configuration.
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
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

async fn apply_role_to_config_inner(
    config: &mut Config,
    role_name: &str,
    role: &AgentRoleConfig,
) -> anyhow::Result<()> {
    let is_built_in = !config.agent_roles.contains_key(role_name);
    let Some(config_file) = role.config_file.as_ref() else {
        return Ok(());
    };
    let role_layer_toml = load_role_layer_toml(config, config_file, is_built_in, role_name).await?;
    let role_config = deserialize_config_toml_with_base(role_layer_toml, &config.codex_home)?;
    let mut overrides = AgentRoleOverrides {
        developer_instructions: role_config.developer_instructions,
        model: role_config.model,
        model_reasoning_effort: role_config.model_reasoning_effort,
        model_reasoning_summary: role_config.model_reasoning_summary,
        model_verbosity: role_config.model_verbosity,
        personality: role_config.personality,
        service_tier: role_config.service_tier,
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
    {
        return Ok(());
    }
<<<<<<< HEAD
    let role_sets_model_runtime =
        role_layer_toml.get("model").is_some() || role_layer_toml.get("model_provider").is_some();
    let explicit_role_model = role_layer_toml
        .get("model")
        .and_then(TomlValue::as_str)
        .map(ToOwned::to_owned);
    let preserve_current_model = !role_sets_model_runtime;
    let preserved_current_model = preserve_current_model
        .then(|| config.model.clone())
        .flatten();
    let preserve_current_provider = role_layer_toml.get("model_provider").is_none();
    let preserve_current_service_tier = role_layer_toml.get("service_tier").is_none();
    // Runtime permission changes (CLI flags, `/permissions`, or a restored thread snapshot) are
    // held on `Config` and are not necessarily represented in its persisted layer stack. Rebuilding
    // that stack for an unrelated role layer must not silently reset a spawned child to defaults.
    // A role that explicitly declares permission-related configuration still owns that policy.
    let role_sets_permissions = [
        "approval_policy",
        "approvals_reviewer",
        "sandbox_mode",
        "default_permissions",
        "permissions",
        "network",
        "shell_environment_policy",
        "allow_login_shell",
        "windows",
    ]
    .into_iter()
    .any(|key| role_layer_toml.get(key).is_some());
    let preserved_runtime_permissions =
        (!role_sets_permissions).then(|| config.permissions.clone());
    let preserved_runtime_approvals_reviewer =
        (!role_sets_permissions).then_some(config.approvals_reviewer);
    let role_reasoning_effort = role_layer_toml
        .get("model_reasoning_effort")
        .and_then(TomlValue::as_str)
        .and_then(|value| ReasoningEffort::from_str(value).ok());
    let preserved_runtime_reasoning_effort = (!is_built_in && role_reasoning_effort.is_none())
        .then_some(config.model_reasoning_effort.clone())
        .flatten();

    let mut next_config = reload::build_next_config(
        config,
        role_layer_toml,
        developer_instructions,
        preserve_current_provider,
        preserve_current_service_tier,
    )
    .await?;
    if let Some(model) = explicit_role_model {
        next_config.model = Some(model);
    } else if let Some(model) = preserved_current_model {
        next_config.model = Some(model);
    }
    if let Some(reasoning_effort) = role_reasoning_effort {
        next_config.model_reasoning_effort = Some(reasoning_effort);
    } else if let Some(reasoning_effort) = preserved_runtime_reasoning_effort {
        next_config.model_reasoning_effort = Some(reasoning_effort);
    }
    if let Some(permissions) = preserved_runtime_permissions {
        next_config.permissions = permissions;
    }
    if let Some(approvals_reviewer) = preserved_runtime_approvals_reviewer {
        next_config.approvals_reviewer = approvals_reviewer;
    }
    // The caller may raise the depth at runtime for a PF crew topology (for example,
    // root -> Troll -> Orc). Rebuilding from persisted layers must not collapse that effective
    // runtime limit back to the upstream default when a role file is applied.
    next_config.agent_max_depth = config.agent_max_depth;
    // PF-84: the rebuilt layers only know `[provider_accounts]`; a child that
    // stays on its parent's provider keeps the parent's live account (from
    // `--account`, resume or a spawn `account`).
    if next_config.model_provider_id == config.model_provider_id {
        let account = config.model_provider.account.clone();
        if let Some(provider) = next_config
            .model_providers
            .get_mut(&next_config.model_provider_id)
        {
            provider.account = account.clone();
        }
        next_config.model_provider.account = account;
    }
    *config = next_config;
||||||| 413492cd6c
    let preserve_current_provider = role_layer_toml.get("model_provider").is_none();
    let preserve_current_service_tier = role_layer_toml.get("service_tier").is_none();

    *config = reload::build_next_config(
        config,
        role_layer_toml,
        developer_instructions,
        preserve_current_provider,
        preserve_current_service_tier,
    )
    .await?;
=======
    *config = role_overrides::build_next_config(config, role_layer_toml, &overrides)?;
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
    Ok(())
}

async fn load_role_layer_toml(
    config: &Config,
    config_file: &Path,
    is_built_in: bool,
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
        let role_config_toml = parse_agent_role_file_contents(
            &role_config_contents,
            config_file,
            role_config_base,
            Some(role_name),
        )?
        .config;
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
<<<<<<< HEAD
        let preserve_current_model = role_layer_toml.get("model").is_none();
        let mut overrides = reload_overrides(
            config,
            preserve_current_model,
            preserve_current_provider,
            preserve_current_service_tier,
        );
        if let (RoleDeveloperInstructions::PreserveCallerInstructions, Some(_), None) = (
            developer_instructions,
            &config.multi_agent_v2.subagent_developer_instructions,
            role_layer_toml.get("developer_instructions"),
        ) {
            overrides
                .developer_instructions
                .clone_from(&config.developer_instructions);
||||||| 413492cd6c
        let preserve_current_model = role_layer_toml.get("model").is_none();
        let preserve_current_reasoning_effort =
            role_layer_toml.get("model_reasoning_effort").is_none();
        let mut overrides = reload_overrides(
            config,
            preserve_current_model,
            preserve_current_provider,
            preserve_current_service_tier,
        );
        if let (RoleDeveloperInstructions::PreserveCallerInstructions, Some(_), None) = (
            developer_instructions,
            &config.multi_agent_v2.subagent_developer_instructions,
            role_layer_toml.get("developer_instructions"),
        ) {
            overrides
                .developer_instructions
                .clone_from(&config.developer_instructions);
=======
        let mut next_config = config.clone();
        next_config.config_layer_stack = build_config_layer_stack(config, &role_layer_toml)?;
        if let Some(model) = &overrides.model {
            next_config.model = Some(model.clone());
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        }
<<<<<<< HEAD
        let config_layer_stack = build_config_layer_stack(config, &role_layer_toml)?;
        let merged_config = deserialize_effective_config(config, &config_layer_stack)?;

        let mut next_config = Config::load_config_with_layer_stack(
            LOCAL_FS.as_ref(),
            merged_config,
            overrides,
            config.codex_home.clone(),
            config_layer_stack,
        )
        .await?;
        next_config.accounting.clone_from(&config.accounting);
        // Overlay explicit role fields onto the embedding's runtime provider,
        // including nested settings. A whole-provider copy loses role overrides;
        // copying selected fields would lose the next supported provider setting.
        let accounting_wire = match &config.accounting {
            crate::config::AccountingMode::Provider { .. } => Some(config.model_provider.wire_api),
            crate::config::AccountingMode::DirectAnthropic { .. }
                if config.model_provider_id == "anthropic" =>
            {
                Some(codex_model_provider_info::WireApi::Anthropic)
            }
            crate::config::AccountingMode::DirectOpenAiResponsesHttp { .. }
            | crate::config::AccountingMode::DirectOpenAiResponses { .. }
                if config.model_provider_id == "openai" =>
            {
                Some(codex_model_provider_info::WireApi::Responses)
            }
            crate::config::AccountingMode::DirectOpenAiChat { .. }
                if config.model_provider_id == "openai" =>
            {
                Some(codex_model_provider_info::WireApi::Chat)
            }
            _ => None,
        };
        if next_config.model_provider_id == config.model_provider_id
            && let Some(accounting_wire) = accounting_wire
        {
            let mut provider = TomlValue::try_from(&config.model_provider)?;
            if let Some(overlay) = role_layer_toml
                .get("model_providers")
                .and_then(|providers| providers.get(&config.model_provider_id))
            {
                codex_config::merge_toml_values(&mut provider, overlay);
            }
            let provider: codex_model_provider_info::ModelProviderInfo = provider.try_into()?;
            provider.validate().map_err(anyhow::Error::msg)?;
            // An explicit wire change cannot bypass the bound Messages adapter.
            // Endpoint changes remain visible to its existing pre-send rejection.
            anyhow::ensure!(
                provider.wire_api == accounting_wire,
                "this role changes the provider's wire_api, which developer accounting cannot follow; \
                 remove wire_api from the role's [model_providers] overlay"
            );
            next_config
                .model_providers
                .insert(config.model_provider_id.clone(), provider.clone());
            next_config.model_provider = provider;
||||||| 413492cd6c
        let config_layer_stack = build_config_layer_stack(config, &role_layer_toml)?;
        let merged_config = deserialize_effective_config(config, &config_layer_stack)?;

        let mut next_config = Config::load_config_with_layer_stack(
            LOCAL_FS.as_ref(),
            merged_config,
            overrides,
            config.codex_home.clone(),
            config_layer_stack,
        )
        .await?;
        if preserve_current_reasoning_effort {
            next_config
                .model_reasoning_effort
                .clone_from(&config.model_reasoning_effort);
=======
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
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        }
        Ok(next_config)
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
<<<<<<< HEAD
        )?)
    }

    fn deserialize_effective_config(
        config: &Config,
        config_layer_stack: &ConfigLayerStack,
    ) -> anyhow::Result<ConfigToml> {
        Ok(deserialize_config_toml_with_base(
            config_layer_stack.effective_config(),
            &config.codex_home,
        )?)
    }

    fn existing_layers(config: &Config) -> Vec<ConfigLayerEntry> {
        config
            .config_layer_stack
            .get_layers(
                ConfigLayerStackOrdering::LowestPrecedenceFirst,
                /*include_disabled*/ true,
            )
            .into_iter()
            .cloned()
            .collect()
    }

    fn insert_layer(layers: &mut Vec<ConfigLayerEntry>, layer: ConfigLayerEntry) {
        let insertion_index =
            layers.partition_point(|existing_layer| existing_layer.name <= layer.name);
        layers.insert(insertion_index, layer);
    }

    fn role_layer(role_layer_toml: TomlValue) -> ConfigLayerEntry {
        ConfigLayerEntry::new(ConfigLayerSource::SessionFlags, role_layer_toml)
    }

    fn reload_overrides(
        config: &Config,
        preserve_current_model: bool,
        preserve_current_provider: bool,
        preserve_current_service_tier: bool,
    ) -> ConfigOverrides {
        ConfigOverrides {
            model: preserve_current_model
                .then(|| config.model.clone())
                .flatten(),
            cwd: Some(config.cwd.to_path_buf()),
            model_provider: preserve_current_provider.then(|| config.model_provider_id.clone()),
            service_tier: preserve_current_service_tier.then(|| config.service_tier.clone()),
            codex_linux_sandbox_exe: config.codex_linux_sandbox_exe.clone(),
            main_execve_wrapper_exe: config.main_execve_wrapper_exe.clone(),
            workspace_roots: Some(config.workspace_roots.clone()),
            ..Default::default()
        }
||||||| 413492cd6c
        )?)
    }

    fn deserialize_effective_config(
        config: &Config,
        config_layer_stack: &ConfigLayerStack,
    ) -> anyhow::Result<ConfigToml> {
        Ok(deserialize_config_toml_with_base(
            config_layer_stack.effective_config(),
            &config.codex_home,
        )?)
    }

    fn existing_layers(config: &Config) -> Vec<ConfigLayerEntry> {
        config
            .config_layer_stack
            .get_layers(
                ConfigLayerStackOrdering::LowestPrecedenceFirst,
                /*include_disabled*/ true,
            )
            .into_iter()
            .cloned()
            .collect()
    }

    fn insert_layer(layers: &mut Vec<ConfigLayerEntry>, layer: ConfigLayerEntry) {
        let insertion_index =
            layers.partition_point(|existing_layer| existing_layer.name <= layer.name);
        layers.insert(insertion_index, layer);
    }

    fn role_layer(role_layer_toml: TomlValue) -> ConfigLayerEntry {
        ConfigLayerEntry::new(ConfigLayerSource::SessionFlags, role_layer_toml)
    }

    fn reload_overrides(
        config: &Config,
        preserve_current_model: bool,
        preserve_current_provider: bool,
        preserve_current_service_tier: bool,
    ) -> ConfigOverrides {
        ConfigOverrides {
            cwd: Some(config.cwd.to_path_buf()),
            model: preserve_current_model
                .then(|| config.model.clone())
                .flatten(),
            model_provider: preserve_current_provider.then(|| config.model_provider_id.clone()),
            service_tier: preserve_current_service_tier.then(|| config.service_tier.clone()),
            codex_linux_sandbox_exe: config.codex_linux_sandbox_exe.clone(),
            main_execve_wrapper_exe: config.main_execve_wrapper_exe.clone(),
            ..Default::default()
        }
=======
        )?
        .with_cloud_config_binding(config.config_layer_stack.cloud_config_binding().cloned())
        .with_mcp_ema_cancellation(config.config_layer_stack.mcp_ema_cancellation().clone())
        .with_applied_config_server(config.config_layer_stack.applied_config_server().cloned())
        .with_user_and_project_exec_policy_rules_ignored(
            config
                .config_layer_stack
                .ignore_user_and_project_exec_policy_rules(),
        ))
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
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
<<<<<<< HEAD
        const EXPLORER: &str = include_str!("builtins/explorer.toml");
        const AWAITER: &str = include_str!("builtins/awaiter.toml");
        const NAZGUL: &str = concat!(
            include_str!("builtins/nazgul.toml"),
            "\nbase_instructions = '''",
            include_str!("builtins/nazgul_base.md"),
            "'''\n"
        );
        const TROLL: &str = concat!(
            include_str!("builtins/troll.toml"),
            "\nbase_instructions = '''",
            include_str!("builtins/troll_base.md"),
            "'''\n"
        );
        const ORC: &str = concat!(
            include_str!("builtins/orc.toml"),
            "\nbase_instructions = '''",
            include_str!("builtins/orc_base.md"),
            "'''\n"
        );
||||||| 413492cd6c
        const EXPLORER: &str = include_str!("builtins/explorer.toml");
        const AWAITER: &str = include_str!("builtins/awaiter.toml");
=======
        const EXPLORER: &str = include_str!("../../assets/agent/builtins/explorer.toml");
        const AWAITER: &str = include_str!("../../assets/agent/builtins/awaiter.toml");
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
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
