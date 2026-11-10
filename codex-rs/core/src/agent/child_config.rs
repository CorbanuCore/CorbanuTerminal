//! Prepares child configuration from captured step settings and requested overrides.
//!
//! Spawn and reload share live runtime policy; role and model precedence, full-history
//! inheritance, and validation messages remain the same for each multi-agent version.

use crate::agent::role::DEFAULT_ROLE_NAME;
use crate::agent::role::apply_role_to_config;
use crate::agent::role::apply_role_to_config_for_multi_agent_v2;
use crate::config::Config;
use crate::session::session::Session;
use crate::session::step_context::StepContext;
use crate::session::turn_context::TurnContext;
use codex_features::Feature;
use codex_models_manager::manager::RefreshStrategy;
use codex_protocol::config_types::SERVICE_TIER_DEFAULT_REQUEST_VALUE;
use codex_protocol::models::BaseInstructions;
use codex_protocol::openai_models::ModelPreset;
use codex_protocol::openai_models::ReasoningEffort;
use codex_protocol::openai_models::ReasoningEffortPreset;
use codex_protocol::protocol::MultiAgentVersion;

// Corbanu: more of the catalogue is listed when a requested model is unknown.
pub(crate) const MAX_SPAWN_AGENT_MODEL_OVERRIDES: usize = 32;

pub(crate) fn model_supports_multi_agent_backend(
    model: &ModelPreset,
    multi_agent_version: MultiAgentVersion,
) -> bool {
    multi_agent_version != MultiAgentVersion::V2
        || model.multi_agent_version != Some(MultiAgentVersion::Disabled)
}

/// Selects the existing spawn tool's role-inheritance rules.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum SpawnConfigVersion {
    V1,
    V2,
}

pub(crate) struct SpawnConfigOptions<'a> {
    pub(crate) version: SpawnConfigVersion,
    pub(crate) full_history_fork: bool,
    pub(crate) role_name: Option<&'a str>,
    /// Corbanu: an explicit provider for the child; needs `model` too.
    pub(crate) model_provider: Option<&'a str>,
    pub(crate) model: Option<&'a str>,
    pub(crate) reasoning_effort: Option<ReasoningEffort>,
}

pub(crate) struct PreparedSpawnConfig {
    pub(crate) config: Config,
    pub(crate) role_name: Option<String>,
}

/// Resolves child settings before starting the thread, retaining the invoking tool's precedence.
pub(crate) async fn prepare_agent_spawn_config(
    session: &Session,
    step_context: &StepContext,
    options: SpawnConfigOptions<'_>,
) -> Result<PreparedSpawnConfig, String> {
    let turn = step_context.turn.as_ref();
    let mut config =
        build_agent_spawn_config(&session.get_base_instructions().await, step_context)?;
    if options.version == SpawnConfigVersion::V1 && options.full_history_fork {
        reject_full_fork_agent_type_override(options.role_name)?;
    }
    // Corbanu: an explicit provider is ignored on full-history forks (as main), so the parent's
    // history only goes to a provider the parent already uses or a role selects.
    let model_provider = options
        .model_provider
        .map(str::trim)
        .filter(|provider| !provider.is_empty() && !options.full_history_fork);
    // Corbanu (as main's V2 handler): a session may still run a model that was later hidden or
    // disabled in the refreshed catalogue. A V2 child that inherits that already-authorized
    // runtime keeps the session contract; a runtime selected for the child still fails closed.
    let child_runtime_was_selected = options.version == SpawnConfigVersion::V1
        || options.model_provider.is_some()
        || options.model.is_some()
        || options.reasoning_effort.is_some()
        || options.role_name.is_some()
        || turn.config.agent_default_subagent_model.is_some()
        || turn
            .config
            .agent_default_subagent_reasoning_effort
            .is_some();
    if model_provider.is_none() {
        apply_requested_spawn_agent_runtime_overrides(
            session,
            step_context,
            &mut config,
            /*requested_provider*/ None,
            options.model,
            options.reasoning_effort.clone(),
        )
        .await?;
    }
    if !options.full_history_fork
        || (options.version == SpawnConfigVersion::V2 && options.role_name.is_some())
    {
        apply_spawn_agent_role(session, &mut config, options.role_name).await?;
        if options.version == SpawnConfigVersion::V2
            && options.full_history_fork
            && config.developer_instructions.is_none()
        {
            config
                .developer_instructions
                .clone_from(&turn.developer_instructions);
        }
    }
    // Corbanu (as main): an explicit runtime is applied after the role, so the role's
    // instructions survive while the requested provider and model win.
    if model_provider.is_some() {
        apply_requested_spawn_agent_runtime_overrides(
            session,
            step_context,
            &mut config,
            model_provider,
            options.model,
            options.reasoning_effort,
        )
        .await?;
    }
    apply_spawn_agent_service_tier(session, &mut config).await?;
    apply_spawn_agent_runtime_overrides(&mut config, turn)?;
    // Corbanu: the resolved runtime must clear the operator's spend policy and be usable.
    ensure_spawn_provider_authorized(&config, &config.model_provider_id)?;
    if child_runtime_was_selected {
        ensure_spawn_runtime_eligible(session, &config).await?;
    }

    // Remember an applied configured default so cold reload reapplies its restrictions.
    let role_name = options
        .role_name
        .or_else(|| {
            (options.version == SpawnConfigVersion::V2
                && !options.full_history_fork
                && config
                    .agent_roles
                    .get(DEFAULT_ROLE_NAME)
                    .is_some_and(|role| role.config_file.is_some()))
            .then_some(DEFAULT_ROLE_NAME)
        })
        .map(str::to_owned);
    Ok(PreparedSpawnConfig { config, role_name })
}

/// Builds the base config snapshot for a newly spawned sub-agent.
///
/// The returned config starts from the parent's effective config and then refreshes the
/// model selection and reasoning settings captured for the invoking step, plus the turn's
/// runtime approval policy, sandbox, and cwd. Role-specific overrides are layered
/// after this step; skipping this helper and cloning stale config state directly can send the child
/// agent out with the wrong provider or runtime policy.
pub(crate) fn build_agent_spawn_config(
    base_instructions: &BaseInstructions,
    step_context: &StepContext,
) -> Result<Config, String> {
    let mut config = build_agent_shared_config(step_context.turn.as_ref())?;
    let settings = &step_context.settings;
    config.model = Some(settings.model_info.slug.clone());
    config.model_reasoning_effort = settings.effective_reasoning_effort();
    config.model_reasoning_summary = Some(settings.reasoning_summary);
    config.base_instructions = Some(base_instructions.text.clone());
    config.base_instructions_provenance = base_instructions.provenance.clone();
    Ok(config)
}

pub(crate) fn build_agent_resume_config(turn: &TurnContext) -> Result<Config, String> {
    let mut config = build_agent_shared_config(turn)?;
    // For resume, keep base instructions sourced from rollout/session metadata.
    config.base_instructions = None;
    config.base_instructions_provenance = None;
    Ok(config)
}

fn build_agent_shared_config(turn: &TurnContext) -> Result<Config, String> {
    let base_config = turn.config.clone();
    let mut config = (*base_config).clone();
    if config
        .features
        .enabled(Feature::SubagentDefaultContextLimits)
    {
        config.model_context_window = None;
        config.model_auto_compact_token_limit = None;
    }
    // Preserve activation for history forks without freezing the parent's model-owned prompts.
    // Fresh child startup restores configured preferences from the retained snapshot.
    config.token_budget = turn.configured_token_budget.clone();
    config.model = Some(turn.model_info().slug.clone());
    config.model_provider = turn.provider.info().clone();
    config.model_reasoning_effort = turn
        .reasoning_effort()
        .or(turn.model_info().default_reasoning_level.as_ref())
        .cloned();
    config.model_reasoning_summary = Some(turn.reasoning_summary());
    config.developer_instructions = turn.developer_instructions.clone();
    if turn.multi_agent_version == MultiAgentVersion::V2
        && let Some(developer_instructions) = turn
            .config
            .multi_agent_v2
            .subagent_developer_instructions
            .clone()
    {
        config.developer_instructions = Some(developer_instructions);
    }
    apply_spawn_agent_runtime_overrides(&mut config, turn)?;

    Ok(config)
}

fn reject_full_fork_agent_type_override(agent_type: Option<&str>) -> Result<(), String> {
    if agent_type.is_some() {
        return Err(
            "Full-history forked agents inherit the parent agent type; omit agent_type, or spawn without a full-history fork.".to_string());
    }
    Ok(())
}

/// Copies runtime-only turn state onto a child config before it is handed to `AgentControl`.
///
/// These values are chosen by the live turn rather than persisted config, so leaving them stale can
/// make a child agent disagree with its parent about approval policy, cwd, or sandboxing.
fn apply_spawn_agent_runtime_overrides(
    config: &mut Config,
    turn: &TurnContext,
) -> Result<(), String> {
    // Corbanu: V2's agent paths and mailboxes belong to the running engine, not the selected
    // model, so the child keeps the parent's multi-agent runtime.
    if turn.multi_agent_version == MultiAgentVersion::V2 {
        config
            .features
            .enable(Feature::MultiAgentV2)
            .map_err(|err| format!("could not preserve the parent agent runtime: {err}"))?;
    }
    config
        .permissions
        .approval_policy
        .set(turn.approval_policy())
        .map_err(|err| format!("approval_policy is invalid: {err}"))?;
    config.approvals_reviewer = turn.config.approvals_reviewer;
    #[allow(deprecated)]
    let turn_cwd = turn.cwd.clone();
    config.cwd = turn_cwd;
    config
        .permissions
        .set_permission_profile_from_session_snapshot(
            turn.config
                .permissions
                .permission_profile_state()
                .snapshot(),
        )
        .map_err(|err| format!("permission_profile is invalid: {err}"))?;
    Ok(())
}

/// Corbanu: operator spend policy for model-driven agent creation.
///
/// `agents.provider_allowlist` is authorization, not preference. A model selects a runtime; only
/// the operator authorizes one. This is enforced in core because the model-facing spawn path never
/// crosses the TUI, and it is neutral to provider, model, and role name.
pub(crate) fn ensure_spawn_provider_authorized(
    config: &Config,
    provider_id: &str,
) -> Result<(), String> {
    let Some(allowlist) = config.agent_provider_allowlist.as_ref() else {
        return Ok(());
    };
    if allowlist.iter().any(|allowed| allowed == provider_id) {
        return Ok(());
    }
    Err(format!(
        "Provider `{provider_id}` is not authorized for spawned agents. \
Authorized providers: {}. This is operator policy set in `agents.provider_allowlist`; \
it cannot be changed from a task and must not be worked around by selecting a different \
model that routes to an unauthorized provider. Do not spawn a substitute runtime in this \
turn; report the refusal and obtain the user's explicit consent before trying a fallback.",
        allowlist.join(", ")
    ))
}

/// Corbanu: validates a fully resolved child runtime without inventing an operator policy.
///
/// Catalogue visibility and lifecycle metadata control picker presentation and automatic
/// recommendations. They must not become a hidden spawn blocklist: an exact runtime configured by
/// the operator remains usable even when it is hidden, custom, or represented by another
/// provider's catalogue row. Only an explicit `agents.provider_allowlist` is spend authorization.
pub(crate) async fn ensure_spawn_runtime_eligible(
    session: &Session,
    config: &Config,
) -> Result<(), String> {
    let model = config
        .model
        .as_deref()
        .ok_or_else(|| "spawn_agent could not resolve the child model".to_string())?;
    if !config
        .model_providers
        .contains_key(&config.model_provider_id)
    {
        return Err(format!(
            "Runtime provider `{}` is not configured for spawned agents.",
            config.model_provider_id
        ));
    }
    let model_info = crate::model_info_for_provider::model_info_for_config(
        &session.services.models_manager,
        model,
        config,
    )
    .await;
    if model_info
        .orchestration
        .as_ref()
        .is_some_and(|metadata| metadata.provider_id() == config.model_provider_id)
        && let Some(reasoning_effort) = config.model_reasoning_effort.as_ref()
        && !model_info.supported_reasoning_levels.is_empty()
    {
        validate_spawn_agent_reasoning_effort(
            model,
            &model_info.supported_reasoning_levels,
            reasoning_effort,
        )?;
    }
    Ok(())
}

/// Corbanu: an explicit `model_provider` selects an exact configured runtime; without one the
/// requested model (or the configured subagent default) applies on the parent's provider.
async fn apply_requested_spawn_agent_runtime_overrides(
    session: &Session,
    step_context: &StepContext,
    config: &mut Config,
    requested_provider: Option<&str>,
    requested_model: Option<&str>,
    requested_reasoning_effort: Option<ReasoningEffort>,
) -> Result<(), String> {
    let Some(requested_provider) = requested_provider
        .map(str::trim)
        .filter(|provider| !provider.is_empty())
    else {
        return apply_requested_spawn_agent_model_overrides(
            session,
            step_context,
            config,
            requested_model,
            requested_reasoning_effort,
        )
        .await;
    };
    let requested_model = requested_model
        .map(str::trim)
        .filter(|model| !model.is_empty())
        .ok_or_else(|| {
            "spawn_agent requires `model` when `model_provider` is set; set both fields or omit both to inherit the parent runtime.".to_string()
        })?;
    let provider = config
        .model_providers
        .get(requested_provider)
        .cloned()
        .ok_or_else(|| format!("Unknown model provider `{requested_provider}` for spawn_agent."))?;
    ensure_spawn_provider_authorized(config, requested_provider)?;
    let resolved_model = codex_model_provider_info::resolve_model_for_provider(
        Some(requested_model.to_string()),
        requested_provider,
    );
    if resolved_model.as_deref() != Some(requested_model) {
        return Err(format!(
            "Model `{requested_model}` is not valid for provider `{requested_provider}`."
        ));
    }

    config.model_provider_id = requested_provider.to_string();
    config.model_provider = provider;
    config.model = Some(requested_model.to_string());

    let selected_model_info = crate::model_info_for_provider::model_info_for_config(
        &session.services.models_manager,
        requested_model,
        config,
    )
    .await;
    if let Some(reasoning_effort) = requested_reasoning_effort {
        validate_spawn_agent_reasoning_effort(
            requested_model,
            &selected_model_info.supported_reasoning_levels,
            &reasoning_effort,
        )?;
        config.model_reasoning_effort = Some(reasoning_effort);
    } else {
        config.model_reasoning_effort = selected_model_info.default_reasoning_level.clone();
    }

    Ok(())
}

async fn apply_requested_spawn_agent_model_overrides(
    session: &Session,
    step_context: &StepContext,
    config: &mut Config,
    requested_model: Option<&str>,
    requested_reasoning_effort: Option<ReasoningEffort>,
) -> Result<(), String> {
    let turn = step_context.turn.as_ref();
    let parent_model = &step_context.settings.model_info;
    let requested_model = requested_model.or(turn.config.agent_default_subagent_model.as_deref());
    let requested_reasoning_effort = requested_reasoning_effort
        .or_else(|| turn.config.agent_default_subagent_reasoning_effort.clone());
    if requested_model.is_none() && requested_reasoning_effort.is_none() {
        return Ok(());
    }

    if let Some(requested_model) = requested_model {
        // Corbanu: the parent's own model needs no catalogue lookup or provider change.
        if requested_model == parent_model.slug {
            if let Some(reasoning_effort) = requested_reasoning_effort {
                validate_spawn_agent_reasoning_effort(
                    &parent_model.slug,
                    &parent_model.supported_reasoning_levels,
                    &reasoning_effort,
                )?;
                config.model_reasoning_effort = Some(reasoning_effort);
            }
            return Ok(());
        }
        reject_spawn_agent_model_switch_for_third_party_provider(
            step_context,
            config,
            requested_model,
        )?;
        let available_models = session
            .services
            .models_manager
            .list_models(RefreshStrategy::Offline, config.http_client_factory())
            .await;
        let selected_model_name = find_spawn_agent_model_name(
            &available_models,
            requested_model,
            turn.multi_agent_version,
        )?;

        config.model = Some(selected_model_name.clone());
        // Corbanu: a model switch must not silently keep the parent's provider when that
        // provider cannot serve the selected model; the child's first turn would fail with
        // "Unknown model".
        if let Some(corrected) = codex_model_provider_info::corrected_catalog_provider(
            &selected_model_name,
            &config.model_provider_id,
        ) && let Some(info) = config.model_providers.get(corrected)
        {
            // A re-route is still a provider selection and must clear the same operator policy
            // as an explicit one. Authorize before mutating, so a refused switch cannot leave the
            // child pointed at an unauthorized provider.
            ensure_spawn_provider_authorized(config, corrected)?;
            tracing::warn!(
                model = %selected_model_name,
                parent_provider = %config.model_provider_id,
                corrected_provider = corrected,
                "correcting inherited provider for spawn_agent model switch"
            );
            let info = info.clone().with_account_from(&config.model_provider)?;
            config.model_provider_id = corrected.to_string();
            config.model_provider = info;
        }
        // The child's provider is final now (#572: Bedrock metadata).
        let selected_model_info = crate::model_info_for_provider::model_info_for_config(
            &session.services.models_manager,
            &selected_model_name,
            config,
        )
        .await;
        if let Some(reasoning_effort) = requested_reasoning_effort {
            validate_spawn_agent_reasoning_effort(
                &selected_model_name,
                &selected_model_info.supported_reasoning_levels,
                &reasoning_effort,
            )?;
            config.model_reasoning_effort = Some(reasoning_effort);
        } else {
            config.model_reasoning_effort = selected_model_info.default_reasoning_level.clone();
        }

        return Ok(());
    }

    if let Some(reasoning_effort) = requested_reasoning_effort {
        validate_spawn_agent_reasoning_effort(
            &parent_model.slug,
            &parent_model.supported_reasoning_levels,
            &reasoning_effort,
        )?;
        config.model_reasoning_effort = Some(reasoning_effort);
    }

    Ok(())
}

/// Corbanu: third-party gateways serve only their own catalogue, so a child of one may switch
/// models only to one a configured provider serves.
fn reject_spawn_agent_model_switch_for_third_party_provider(
    step_context: &StepContext,
    child_config: &Config,
    requested_model: &str,
) -> Result<(), String> {
    let turn = step_context.turn.as_ref();
    let parent_model = &step_context.settings.model_info.slug;
    let provider_info = turn.provider.info();
    if !(provider_info.is_ambient()
        || provider_info.is_kimi_code()
        || provider_info.is_zai()
        || provider_info.is_openrouter()
        || provider_info.is_baseten()
        || provider_info.is_vercel())
    {
        return Ok(());
    }
    if let Some(corrected) = codex_model_provider_info::corrected_catalog_provider(
        requested_model,
        &turn.config.model_provider_id,
    ) && child_config.model_providers.contains_key(corrected)
    {
        return Ok(());
    }

    Err(format!(
        "spawn_agent cannot switch from provider `{}` model `{parent_model}` to model `{requested_model}`. Subagents inherit the parent provider; omit model to inherit `{parent_model}`, or switch the parent session provider/model before spawning.",
        turn.config.model_provider_id
    ))
}

pub(crate) async fn apply_spawn_agent_service_tier(
    session: &Session,
    config: &mut Config,
) -> Result<(), String> {
    let Some(service_tier) = session.services.agent_control.service_tier() else {
        config.service_tier = None;
        return Ok(());
    };
    if service_tier == SERVICE_TIER_DEFAULT_REQUEST_VALUE {
        config.service_tier = Some(service_tier);
        return Ok(());
    }

    let model = config.model.clone().ok_or_else(|| {
        "spawn_agent could not resolve the child model for service tier validation".to_string()
    })?;
    let model_info = crate::model_info_for_provider::model_info_for_config(
        &session.services.models_manager,
        model.as_str(),
        config,
    )
    .await;

    config.service_tier = model_info
        .supports_service_tier(service_tier.as_str())
        .then_some(service_tier);
    Ok(())
}

async fn apply_spawn_agent_role(
    session: &Session,
    config: &mut Config,
    role_name: Option<&str>,
) -> Result<(), String> {
    // Corbanu: the role's model and effort are validated with the final runtime in
    // `ensure_spawn_runtime_eligible`, against the child's own provider.
    if session.multi_agent_version() == Some(MultiAgentVersion::V2) {
        apply_role_to_config_for_multi_agent_v2(config, role_name).await
    } else {
        apply_role_to_config(config, role_name).await
    }
}

fn find_spawn_agent_model_name(
    available_models: &[ModelPreset],
    requested_model: &str,
    multi_agent_version: MultiAgentVersion,
) -> Result<String, String> {
    available_models
        .iter()
        .find(|model| {
            model.model == requested_model
                && model_supports_multi_agent_backend(model, multi_agent_version)
        })
        .map(|model| model.model.clone())
        .ok_or_else(|| {
            let available = available_models
                .iter()
                .filter(|model| model.show_in_picker)
                .filter(|model| model_supports_multi_agent_backend(model, multi_agent_version))
                .take(MAX_SPAWN_AGENT_MODEL_OVERRIDES)
                .map(|model| model.model.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            format!(
                "Unknown model `{requested_model}` for spawn_agent. Available models: {available}"
            )
        })
}

fn validate_spawn_agent_reasoning_effort(
    model: &str,
    supported_reasoning_levels: &[ReasoningEffortPreset],
    requested_reasoning_effort: &ReasoningEffort,
) -> Result<(), String> {
    if supported_reasoning_levels
        .iter()
        .any(|preset| &preset.effort == requested_reasoning_effort)
    {
        return Ok(());
    }

    let supported = supported_reasoning_levels
        .iter()
        .map(|preset| preset.effort.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    Err(format!(
        "Reasoning effort `{requested_reasoning_effort}` is not supported for model `{model}`. Supported reasoning efforts: {supported}"
    ))
}
