use super::*;
use crate::agent::role::DEFAULT_ROLE_NAME;
use crate::agent::types::SpawnAgentForkMode;
use crate::codex_thread::ThreadConfigSnapshot;
use crate::thread_spawn::ThreadSpawnError;
use crate::thread_spawn::ThreadSpawnRequest;
use crate::tools::handlers::multi_agents::collab_tool_call_status;
use crate::tools::handlers::multi_agents_account::apply_spawn_agent_account;
use crate::tools::handlers::multi_agents_account::spawned_account_name;
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

    fn handle<'a>(&'a self, invocation: ToolInvocation) -> codex_tools::ToolExecutorFuture<'a>
    where
        ToolInvocation: 'a,
    {
        Box::pin(async move {
            handle_with_analytics(invocation, CollaborationMessageEncoding::ProviderNative).await
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
            create_spawn_agent_tool_v2(self.options.clone(), /*description_override*/ None),
            PLAINTEXT_SPAWN_AGENT_TOOL,
            "spawn_agent",
        )
    }

    fn handle<'a>(&'a self, invocation: ToolInvocation) -> codex_tools::ToolExecutorFuture<'a>
    where
        ToolInvocation: 'a,
    {
        Box::pin(async move {
            handle_with_analytics(invocation, CollaborationMessageEncoding::PlaintextAdapter).await
        })
    }
}

/// Upstream's collab analytics around both the native tool and Corbanu's plaintext adapter.
async fn handle_with_analytics(
    invocation: ToolInvocation,
    encoding: CollaborationMessageEncoding,
) -> Result<Box<dyn ToolOutput>, FunctionCallError> {
    let analytics = invocation.session.services.analytics_events_client.clone();
    let sender_thread_id = invocation.session.thread_id;
    let turn_id = invocation.step_context.turn.sub_id.clone();
    let call_id = invocation.call_id.clone();
    let started_at_ms = now_unix_timestamp_ms();
    let result = handle_spawn_agent(invocation, encoding).await;
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
            reasoning_effort: agent_snapshot.and_then(|snapshot| snapshot.reasoning_effort.clone()),
            provider_account: None,
            agents_states,
        },
        started_at_ms,
        completed_at_ms,
    );

    result.map(|(output, _, _, _)| boxed_tool_output(output))
}

async fn handle_spawn_agent(
    invocation: ToolInvocation,
    encoding: CollaborationMessageEncoding,
) -> Result<
    (
        SpawnAgentResult,
        ThreadId,
        AgentStatus,
        ThreadConfigSnapshot,
    ),
    FunctionCallError,
> {
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

    // Corbanu (as main, #1016/#820): the codec follows the declared tool surface and the
    // sender's codec provider; the plaintext adapter always sends a plaintext task brief.
    let codec_provider_id =
        crate::tools::collaboration_wire::message_source_provider_id(turn.as_ref());
    let plaintext_source = crate::tools::context::ToolCallSource::DirectPlaintextMessage;
    let message_source = match encoding {
        CollaborationMessageEncoding::ProviderNative => &source,
        CollaborationMessageEncoding::PlaintextAdapter => &plaintext_source,
    };
    let (create_request, spawn_message) = ThreadSpawnRequest {
        task_name: args.task_name,
        message: agent_message_from_tool(message, message_source, codec_provider_id),
        fork_mode: fork_mode.clone(),
        agent_type: args.agent_type.clone(),
        model: args.model,
        reasoning_effort: args.reasoning_effort,
    }
    .into_parts()
    .map_err(spawn_preparation_error)?;
    // Corbanu: the task name is resolved before runtime policy (as main), and an explicit
    // provider from the plaintext adapter is checked against `agents.provider_allowlist`.
    let (mut settings, new_agent_path) = crate::thread_spawn::prepare_with_model_provider(
        &session,
        &step_context,
        &call_id,
        create_request,
        args.model_provider.as_deref(),
    )
    .await
    .map_err(spawn_preparation_error)?;
    let target_provider_id = settings.config.model_provider_id.clone();
    let ensure_encoding = |target: &codex_model_provider_info::ModelProviderInfo| {
        ensure_message_encoding_matches_target(
            ConfiguredRuntime::from_provider(&turn.config.model_provider_id, turn.provider.info()),
            codec_provider_id,
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
        &mut settings.config,
        args.account.as_deref(),
        &ensure_encoding,
    )
    .await?;
    let resolved_account = spawned_account_name(&settings.config);
    let resolved_model_provider = settings.config.model_provider_id.clone();
    let resolved_model = settings
        .config
        .model
        .clone()
        .unwrap_or_else(|| turn.model_info().slug.clone());
    let resolved_reasoning_effort = settings.config.model_reasoning_effort.clone();
    let resolved_service_tier = settings.config.service_tier.clone();
    ensure_encoding(&settings.config.model_provider)?;

    let spawned =
        crate::thread_spawn::spawn_prepared(&session, settings, spawn_message, new_agent_path)
            .await
            .map_err(|err| match err {
                ThreadSpawnError::Preparation(message) => {
                    FunctionCallError::RespondToModel(message)
                }
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
    if let Some(local) = session.services.agent_control.local() {
        local.note_native_agent_dispatch(session.thread_id);
    }
    let agent_status = spawned_agent.status;
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
            model_provider: resolved_model_provider,
            account: resolved_account,
            model: resolved_model,
            reasoning_effort: resolved_reasoning_effort,
            service_tier: resolved_service_tier,
        }
    };
    Ok((output, new_thread_id, agent_status, agent_snapshot))
}

fn spawn_preparation_error(err: ThreadSpawnError) -> FunctionCallError {
    match err {
        ThreadSpawnError::Preparation(message) => FunctionCallError::RespondToModel(message),
        ThreadSpawnError::Backend(err) => collab_spawn_error(err),
    }
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
