use super::*;
use crate::agent::role::DEFAULT_ROLE_NAME;
<<<<<<< HEAD
use crate::agent_communication::AgentCommunicationContext;
use crate::agent_communication::AgentCommunicationKind;
use crate::tools::handlers::multi_agents_account::apply_spawn_agent_account;
use crate::tools::handlers::multi_agents_account::spawned_account_name;
||||||| 413492cd6c
use crate::agent_communication::AgentCommunicationContext;
use crate::agent_communication::AgentCommunicationKind;
=======
use crate::agent::types::SpawnAgentForkMode;
use crate::codex_thread::ThreadConfigSnapshot;
use crate::thread_spawn::ThreadSpawnError;
use crate::thread_spawn::ThreadSpawnRequest;
use crate::tools::handlers::multi_agents::collab_tool_call_status;
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
use crate::tools::handlers::multi_agents_spec::SpawnAgentToolOptions;
use crate::tools::handlers::multi_agents_spec::create_spawn_agent_tool_v2;
use crate::tools::handlers::multi_agents_v2::message_tool::message_content;
use crate::turn_timing::now_unix_timestamp_ms;
use codex_protocol::ThreadId;
use codex_protocol::protocol::MultiAgentVersion;
use codex_tools::ToolSpec;
use std::num::NonZeroUsize;

#[derive(Default)]
pub(crate) struct Handler {
    options: SpawnAgentToolOptions,
    description_override: Option<String>,
}

impl Handler {
    pub(crate) fn new(
        options: SpawnAgentToolOptions,
        description_override: Option<String>,
    ) -> Self {
        Self {
            options,
            description_override,
        }
    }
}

impl ToolExecutor<ToolInvocation> for Handler {
    fn tool_name(&self) -> ToolName {
        ToolName::plain("spawn_agent")
    }

    fn spec(&self) -> ToolSpec {
        create_spawn_agent_tool_v2(self.options.clone(), self.description_override.as_deref())
    }

<<<<<<< HEAD
    fn handle(&self, invocation: ToolInvocation) -> codex_tools::ToolExecutorFuture<'_> {
        Box::pin(async move {
            handle_spawn_agent(invocation, CollaborationMessageEncoding::ProviderNative)
                .await
                .map(boxed_tool_output)
        })
    }
}

#[derive(Default)]
pub(crate) struct PlaintextHandler {
    options: SpawnAgentToolOptions,
}

impl PlaintextHandler {
    pub(crate) fn new(options: SpawnAgentToolOptions) -> Self {
        Self { options }
    }
}

impl ToolExecutor<ToolInvocation> for PlaintextHandler {
    fn tool_name(&self) -> ToolName {
        ToolName::plain(PLAINTEXT_SPAWN_AGENT_TOOL)
    }

    fn spec(&self) -> ToolSpec {
        plaintext_adapter_spec(
            create_spawn_agent_tool_v2(self.options.clone()),
            PLAINTEXT_SPAWN_AGENT_TOOL,
            "spawn_agent",
        )
    }

    fn handle(&self, invocation: ToolInvocation) -> codex_tools::ToolExecutorFuture<'_> {
        Box::pin(async move {
            handle_spawn_agent(invocation, CollaborationMessageEncoding::PlaintextAdapter)
                .await
                .map(boxed_tool_output)
||||||| 413492cd6c
    fn handle(&self, invocation: ToolInvocation) -> codex_tools::ToolExecutorFuture<'_> {
        Box::pin(async move { handle_spawn_agent(invocation).await.map(boxed_tool_output) })
=======
    fn handle<'a>(&'a self, invocation: ToolInvocation) -> codex_tools::ToolExecutorFuture<'a>
    where
        ToolInvocation: 'a,
    {
        Box::pin(async move {
            let analytics = invocation.session.services.analytics_events_client.clone();
            let sender_thread_id = invocation.session.thread_id;
            let turn_id = invocation.step_context.turn.sub_id.clone();
            let call_id = invocation.call_id.clone();
            let started_at_ms = now_unix_timestamp_ms();
            let result = handle_spawn_agent(invocation).await;
            let completed_at_ms = now_unix_timestamp_ms();
            let (status, receiver_thread_ids, agents_states) = match &result {
                Ok((_, thread_id, agent_status, _)) => (
                    collab_tool_call_status(agent_status, Some(*thread_id)),
                    vec![*thread_id],
                    [(*thread_id, agent_status.clone())].into_iter().collect(),
                ),
                Err(_) => (
                    CollabAgentToolCallStatus::Failed,
                    Vec::new(),
                    Default::default(),
                ),
            };
            let agent_snapshot = result.as_ref().ok().map(|(_, _, _, snapshot)| snapshot);

            analytics.track_collab_tool_call(
                turn_id,
                CollabAgentToolCallItem {
                    id: call_id,
                    tool: CollabAgentTool::SpawnAgent,
                    status,
                    sender_thread_id,
                    receiver_thread_ids,
                    receiver_agents: Vec::new(),
                    prompt: None,
                    model: agent_snapshot.map(|snapshot| snapshot.model.clone()),
                    reasoning_effort: agent_snapshot
                        .and_then(|snapshot| snapshot.reasoning_effort.clone()),
                    agents_states,
                },
                started_at_ms,
                completed_at_ms,
            );

            result.map(|(output, _, _, _)| boxed_tool_output(output))
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        })
    }
}

async fn handle_spawn_agent(
    invocation: ToolInvocation,
<<<<<<< HEAD
    encoding: CollaborationMessageEncoding,
) -> Result<SpawnAgentResult, FunctionCallError> {
||||||| 413492cd6c
) -> Result<SpawnAgentResult, FunctionCallError> {
=======
) -> Result<
    (
        SpawnAgentResult,
        ThreadId,
        AgentStatus,
        ThreadConfigSnapshot,
    ),
    FunctionCallError,
> {
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
    let ToolInvocation {
        session,
        step_context,
        payload,
        call_id,
        source,
        ..
    } = invocation;
    let turn = &step_context.turn;
    ensure_manager_tool_allowed(turn, "spawn_agent")?;
    let arguments = function_arguments(payload)?;
    let args: SpawnAgentArgs = parse_arguments(&arguments)?;
    let fork_mode = args.fork_mode()?;
    let message = message_content(args.message)?;
    let role_name = args
        .agent_type
        .as_deref()
        .map(str::trim)
        .filter(|role| !role.is_empty());

<<<<<<< HEAD
    let session_source = turn.session_source.clone();
    let child_depth = next_thread_spawn_depth(&session_source);
    // Resolve the canonical path before runtime policy so malformed task names produce the
    // actionable routing error regardless of the parent runtime's current catalogue lifecycle.
    let spawn_source = thread_spawn_source(
        session.thread_id,
        &turn.session_source,
        child_depth,
        role_name,
        Some(args.task_name.clone()),
        Some(call_id.clone()),
    )?;
    let new_agent_path = spawn_source.get_agent_path().ok_or_else(|| {
        FunctionCallError::RespondToModel(
            "spawned agent is missing a canonical task name".to_string(),
        )
    })?;
    let child_runtime_was_selected = args.model_provider.is_some()
        || args.model.is_some()
        || args.reasoning_effort.is_some()
        || role_name.is_some()
        || turn.config.agent_default_subagent_model.is_some()
        || turn
            .config
            .agent_default_subagent_reasoning_effort
            .is_some();
    let mut config =
        build_agent_spawn_config(&session.get_base_instructions().await, turn.as_ref())?;
    if let Some(service_tier) = args.service_tier.as_ref() {
        config.service_tier = Some(service_tier.clone());
    }
    let is_full_history_fork = matches!(fork_mode, Some(SpawnAgentForkMode::FullHistory));
    if is_full_history_fork {
        reject_full_fork_agent_type_override(role_name)?;
    }
    if is_full_history_fork || args.model_provider.is_none() {
        apply_requested_spawn_agent_model_overrides(
            &session,
            turn.as_ref(),
            &mut config,
            args.model.as_deref(),
            args.reasoning_effort.clone(),
        )
        .await?;
        if !is_full_history_fork {
            // Legacy same-provider model selection retains the established role precedence.
            apply_spawn_agent_role(&session, &mut config, role_name).await?;
        }
    } else {
        // An explicit provider/model pair is a complete runtime selection. Apply the role first
        // so its instruction and sandbox layers survive while the requested runtime wins.
        apply_spawn_agent_role(&session, &mut config, role_name).await?;
        apply_requested_spawn_agent_runtime_overrides(
            &session,
            turn.as_ref(),
            &mut config,
            args.model_provider.as_deref(),
            args.model.as_deref(),
            args.reasoning_effort.clone(),
        )
        .await?;
    }
    apply_spawn_agent_service_tier(
        &session,
        &mut config,
        turn.config.service_tier.as_deref(),
        args.service_tier.as_deref(),
    )
    .await?;
    apply_spawn_agent_runtime_overrides(&mut config, turn.as_ref())?;
    ensure_spawn_provider_authorized(&config, &config.model_provider_id)?;
    // Existing sessions may legitimately run a model that has since been hidden or disabled in
    // the refreshed catalogue. Inheriting that already-authorized runtime preserves the active
    // session contract; selections made specifically for the child still fail closed.
    if child_runtime_was_selected {
        ensure_spawn_runtime_eligible(&session, &config).await?;
    }
    let target_provider_id = config.model_provider_id.clone();
    let ensure_encoding = |target: &codex_model_provider_info::ModelProviderInfo| {
        ensure_message_encoding_matches_target(
            ConfiguredRuntime::from_provider(&turn.config.model_provider_id, turn.provider.info()),
            crate::tools::collaboration_wire::message_source_provider_id(turn.as_ref()),
            &source,
            ConfiguredRuntime::from_provider(&target_provider_id, target),
            encoding,
            "spawn_agent",
            PLAINTEXT_SPAWN_AGENT_TOOL,
        )
    };
    // #1144: an account the encrypted payload cannot follow is refused
    // before the Aggressive approval is asked.
    apply_spawn_agent_account(
        &session,
        turn.as_ref(),
        &call_id,
        &mut config,
        args.account.as_deref(),
        &ensure_encoding,
    )
    .await?;
    let resolved_account = spawned_account_name(&config);
    let resolved_model_provider = config.model_provider_id.clone();
    let resolved_model = config
        .model
        .clone()
        .unwrap_or_else(|| turn.model_info.slug.clone());
    let resolved_reasoning_effort = config.model_reasoning_effort.clone();
    let resolved_service_tier = config.service_tier.clone();

    ensure_encoding(&config.model_provider)?;
    let source = match encoding {
        CollaborationMessageEncoding::ProviderNative => source,
        CollaborationMessageEncoding::PlaintextAdapter => {
            crate::tools::context::ToolCallSource::DirectPlaintextMessage
        }
    };

    let author = turn
        .session_source
        .get_agent_path()
        .unwrap_or_else(AgentPath::root);
    let communication = communication_from_tool_message(
        author,
        new_agent_path.clone(),
        message,
        &source,
        crate::tools::collaboration_wire::message_source_provider_id(turn.as_ref()),
        /*trigger_turn*/ true,
    );
    let context = AgentCommunicationContext::new(AgentCommunicationKind::Spawn, session.thread_id);
    let spawned_agent = Box::pin(
        session
            .services
            .agent_control
            .spawn_agent_with_communication(
                config,
                communication,
                context,
                Some(spawn_source),
                SpawnAgentOptions {
                    fork_parent_spawn_call_id: fork_mode.as_ref().map(|_| call_id.clone()),
                    fork_mode,
                    parent_thread_id: Some(session.thread_id),
                    parent_turn_id: Some(turn.sub_id.clone()),
                    environments: Some(step_context.environments.to_selections()),
                },
            ),
||||||| 413492cd6c
    let session_source = turn.session_source.clone();
    let child_depth = next_thread_spawn_depth(&session_source);
    let mut config =
        build_agent_spawn_config(&session.get_base_instructions().await, turn.as_ref())?;
    if let Some(service_tier) = args.service_tier.as_ref() {
        config.service_tier = Some(service_tier.clone());
    }
    let is_full_history_fork = matches!(fork_mode, Some(SpawnAgentForkMode::FullHistory));
    if is_full_history_fork {
        reject_full_fork_agent_type_override(role_name)?;
    }
    apply_requested_spawn_agent_model_overrides(
        &session,
        turn.as_ref(),
        &mut config,
        args.model.as_deref(),
        args.reasoning_effort.clone(),
    )
    .await?;
    if !is_full_history_fork {
        apply_spawn_agent_role(&session, &mut config, role_name).await?;
    }
    apply_spawn_agent_service_tier(
        &session,
        &mut config,
        turn.config.service_tier.as_deref(),
        args.service_tier.as_deref(),
    )
    .await?;
    apply_spawn_agent_runtime_overrides(&mut config, turn.as_ref())?;

    let spawn_source = thread_spawn_source(
        session.thread_id,
        &turn.session_source,
        child_depth,
        role_name,
        Some(args.task_name.clone()),
    )?;
    let new_agent_path = spawn_source.get_agent_path().ok_or_else(|| {
        FunctionCallError::RespondToModel(
            "spawned agent is missing a canonical task name".to_string(),
        )
    })?;
    let author = turn
        .session_source
        .get_agent_path()
        .unwrap_or_else(AgentPath::root);
    let communication = communication_from_tool_message(
        author,
        new_agent_path.clone(),
        message,
        &source,
        /*trigger_turn*/ true,
    );
    let context = AgentCommunicationContext::new(AgentCommunicationKind::Spawn, session.thread_id);
    let spawned_agent = Box::pin(
        session
            .services
            .agent_control
            .spawn_agent_with_communication(
                config,
                communication,
                context,
                Some(spawn_source),
                SpawnAgentOptions {
                    fork_parent_spawn_call_id: fork_mode.as_ref().map(|_| call_id.clone()),
                    fork_mode,
                    parent_thread_id: Some(session.thread_id),
                    parent_turn_id: Some(turn.sub_id.clone()),
                    environments: Some(step_context.environments.to_selections()),
                },
            ),
=======
    let spawned = crate::thread_spawn::spawn(
        &session,
        &step_context,
        &call_id,
        ThreadSpawnRequest {
            task_name: args.task_name,
            message: agent_message_from_tool(message, &source),
            fork_mode: fork_mode.clone(),
            agent_type: args.agent_type.clone(),
            model: args.model,
            reasoning_effort: args.reasoning_effort,
        },
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
    )
    .await
    .map_err(|err| match err {
        ThreadSpawnError::Preparation(message) => FunctionCallError::RespondToModel(message),
        ThreadSpawnError::Backend(err) => {
            record_collab_spawn_failure(
                &turn.session_telemetry,
                turn.config.apps_mcp_product_sku.as_deref(),
                &err,
                &call_id,
                &turn.sub_id,
                fork_mode.as_ref(),
                MultiAgentVersion::V2,
            );
            collab_spawn_error(err)
        }
    })?;
    let spawned_agent = spawned.agent;
    let agent_snapshot = spawned.config;
    let new_agent_path = spawned.task_name;
    let new_thread_id = spawned_agent.thread_id;
<<<<<<< HEAD
    session
        .services
        .agent_control
        .note_native_agent_dispatch(session.thread_id);
    let agent_snapshot = session
        .services
        .agent_control
        .get_agent_config_snapshot(new_thread_id)
        .await;
||||||| 413492cd6c
    let agent_snapshot = session
        .services
        .agent_control
        .get_agent_config_snapshot(new_thread_id)
        .await;
=======
    let agent_status = spawned_agent.status;
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
    let nickname = agent_snapshot
        .session_source
        .get_nickname()
        .or(spawned_agent.metadata.agent_nickname);
    emit_sub_agent_activity(
        &session,
        turn,
        SubAgentActivityItem {
            model: Some(agent_snapshot.model.clone()),
            reasoning_effort: agent_snapshot.reasoning_effort.clone(),
            id: call_id,
            agent_thread_id: new_thread_id,
            agent_path: new_agent_path.clone(),
            kind: SubAgentActivityKind::Started,
        },
    )
    .await;
    let role_tag = role_name.unwrap_or(DEFAULT_ROLE_NAME);
    turn.session_telemetry.counter(
        "codex.multi_agent.spawn",
        /*inc*/ 1,
        &[("role", role_tag), ("version", "v2")],
    );
    let task_name = String::from(new_agent_path);

    let hide_agent_metadata = turn.config.multi_agent_v2.hide_spawn_agent_metadata;
    let output = if hide_agent_metadata {
        SpawnAgentResult::HiddenMetadata { task_name }
    } else {
        SpawnAgentResult::WithNickname {
            task_name,
            nickname,
<<<<<<< HEAD
            model_provider: resolved_model_provider,
            account: resolved_account,
            model: resolved_model,
            reasoning_effort: resolved_reasoning_effort,
            service_tier: resolved_service_tier,
        })
    }
||||||| 413492cd6c
        })
    }
=======
        }
    };
    Ok((output, new_thread_id, agent_status, agent_snapshot))
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
}

impl CoreToolRuntime for Handler {
    fn matches_kind(&self, payload: &ToolPayload) -> bool {
        matches!(payload, ToolPayload::Function { .. })
    }
}

impl CoreToolRuntime for PlaintextHandler {
    fn matches_kind(&self, payload: &ToolPayload) -> bool {
        matches!(payload, ToolPayload::Function { .. })
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SpawnAgentArgs {
    message: String,
    task_name: String,
    agent_type: Option<String>,
    model_provider: Option<String>,
    account: Option<String>,
    model: Option<String>,
    reasoning_effort: Option<ReasoningEffort>,
    fork_turns: Option<String>,
    fork_context: Option<bool>,
}

impl SpawnAgentArgs {
    fn fork_mode(&self) -> Result<Option<SpawnAgentForkMode>, FunctionCallError> {
        if self.fork_context.is_some() {
            return Err(FunctionCallError::RespondToModel(
                "fork_context is not supported in MultiAgentV2; use fork_turns instead".to_string(),
            ));
        }

        let fork_turns = self
            .fork_turns
            .as_deref()
            .map(str::trim)
            .filter(|fork_turns| !fork_turns.is_empty())
            .unwrap_or("all");

        if fork_turns.eq_ignore_ascii_case("none") {
            return Ok(None);
        }
        // Accept legacy turn counts without limiting the inherited history.
        if fork_turns.eq_ignore_ascii_case("all") || fork_turns.parse::<NonZeroUsize>().is_ok() {
            return Ok(Some(SpawnAgentForkMode::FullHistory));
        }

        Err(FunctionCallError::RespondToModel(
            "fork_turns must be `none` or `all`".to_string(),
        ))
    }
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub(crate) enum SpawnAgentResult {
    WithNickname {
        task_name: String,
        nickname: Option<String>,
        model_provider: String,
        /// PF-84: the child's named account, when named accounts are on.
        #[serde(skip_serializing_if = "Option::is_none")]
        account: Option<String>,
        model: String,
        reasoning_effort: Option<ReasoningEffort>,
        service_tier: Option<String>,
    },
    HiddenMetadata {
        task_name: String,
    },
}

impl ToolOutput for SpawnAgentResult {
    fn log_output(&self) -> String {
        tool_output_json_text(self, "spawn_agent")
    }

    fn success_for_logging(&self) -> bool {
        true
    }

    fn to_response_item(&self, call_id: &str, payload: &ToolPayload) -> ResponseInputItem {
        tool_output_response_item(call_id, payload, self, Some(true), "spawn_agent")
    }

    fn code_mode_result(&self, _payload: &ToolPayload) -> JsonValue {
        tool_output_code_mode_result(self, "spawn_agent")
    }
}
