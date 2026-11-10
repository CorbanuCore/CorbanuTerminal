use super::*;
use crate::agent::child_config::build_agent_resume_config;
use crate::agent::next_thread_spawn_depth;
use crate::config::resume_default_key_error;
use crate::tools::handlers::multi_agents_spec::create_resume_agent_tool;
use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::WarningEvent;
use codex_tools::ToolSpec;

pub(crate) struct Handler;

impl ToolExecutor<ToolInvocation> for Handler {
    fn tool_name(&self) -> ToolName {
        ToolName::namespaced(MULTI_AGENT_V1_NAMESPACE, "resume_agent")
    }

    fn spec(&self) -> ToolSpec {
        create_resume_agent_tool()
    }

    fn search_info(&self) -> Option<ToolSearchInfo> {
        multi_agent_tool_search_info(
            "resume_agent resume reopen closed agent subagent thread id target",
            self.spec(),
        )
    }

    fn handle<'a>(&'a self, invocation: ToolInvocation) -> codex_tools::ToolExecutorFuture<'a>
    where
        ToolInvocation: 'a,
    {
        Box::pin(async move { handle_resume_agent(invocation).await.map(boxed_tool_output) })
    }
}

async fn handle_resume_agent(
    invocation: ToolInvocation,
) -> Result<ResumeAgentResult, FunctionCallError> {
    let ToolInvocation {
        session,
        turn,
        payload,
        call_id,
        ..
    } = invocation;
    let arguments = function_arguments(payload)?;
    let args: ResumeAgentArgs = parse_arguments(&arguments)?;
    let receiver_thread_id = ThreadId::from_string(&args.id).map_err(|err| {
        FunctionCallError::RespondToModel(format!("invalid agent id {}: {err:?}", args.id))
    })?;
    // Corbanu (D527 (a)): the shared handle's local backend; any other backend fails closed.
    let local_agent_control = session
        .services
        .agent_control
        .local_or_unsupported()
        .map_err(|err| collab_agent_error(receiver_thread_id, err))?;
    let receiver_agent = local_agent_control
        .get_agent_metadata(receiver_thread_id)
        .unwrap_or_default();
    let child_depth = next_thread_spawn_depth(&turn.session_source);
    let max_depth = turn.config.agent_max_depth;
    if exceeds_thread_spawn_depth_limit(child_depth, max_depth) {
        return Err(FunctionCallError::RespondToModel(
            "Agent depth limit reached. Solve the task yourself.".to_string(),
        ));
    }

    session
        .emit_turn_item_started(
            &turn,
            &TurnItem::CollabAgentToolCall(CollabAgentToolCallItem {
                id: call_id.clone(),
                tool: CollabAgentTool::ResumeAgent,
                status: CollabAgentToolCallStatus::InProgress,
                sender_thread_id: session.thread_id,
                receiver_thread_ids: vec![receiver_thread_id],
                receiver_agents: vec![CollabAgentRef {
                    thread_id: receiver_thread_id,
                    agent_nickname: receiver_agent.agent_nickname.clone(),
                    agent_role: receiver_agent.agent_role.clone(),
                }],
                prompt: None,
                model: None,
                reasoning_effort: None,
                provider_account: None,
                agents_states: Default::default(),
            }),
        )
        .await;

    let result = async {
        let config = build_agent_resume_config(&turn).map_err(FunctionCallError::RespondToModel)?;
        // #1074: an agent that must be started again (not loaded, whatever its listed status) is
        // refused before it starts when its connection has no credential of its own (#689).
        let start_refusal = resume_default_key_error(
            &config,
            &config.model_provider_id,
            &config.model_provider,
            &turn.config.model_provider_id,
            turn.provider.info(),
        );
        let source = thread_spawn_source(
            session.thread_id(),
            &turn.session_source,
            child_depth,
            /*agent_role*/ None,
            /*task_name*/ None,
            Some(format!("resume:{receiver_thread_id}")),
        )?;
        local_agent_control
            .resume_agent(config, receiver_thread_id, source, start_refusal)
            .await
            .map_err(|err| collab_v2_agent_error(receiver_thread_id, err))
    }
    .await;
    // #1144: open sub-agents that stayed closed are reported with why.
    let (status, receiver_agent, error, descendants_not_resumed) = match result {
        Ok((agent, _, refused)) => (
            agent.status,
            agent.metadata,
            None,
            refused
                .into_iter()
                .map(|(id, error)| DescendantNotResumed {
                    id: id.to_string(),
                    error,
                })
                .collect(),
        ),
        Err(err) => (
            local_agent_control.get_status(receiver_thread_id).await,
            receiver_agent,
            Some(err),
            Vec::new(),
        ),
    };
    session
        .emit_turn_item_completed(
            &turn,
            TurnItem::CollabAgentToolCall(CollabAgentToolCallItem {
                id: call_id,
                tool: CollabAgentTool::ResumeAgent,
                status: collab_tool_call_status(&status, Some(receiver_thread_id)),
                sender_thread_id: session.thread_id(),
                receiver_thread_ids: vec![receiver_thread_id],
                receiver_agents: vec![CollabAgentRef {
                    thread_id: receiver_thread_id,
                    agent_nickname: receiver_agent.agent_nickname,
                    agent_role: receiver_agent.agent_role,
                }],
                prompt: None,
                model: None,
                reasoning_effort: None,
                provider_account: None,
                agents_states: [(receiver_thread_id, status.clone())].into_iter().collect(),
            }),
        )
        .await;

    if let Some(err) = error {
        return Err(err);
    }
    // #1144: a descendant that stayed closed is reported, not only logged.
    for refused in &descendants_not_resumed {
        session
            .send_event(
                &turn,
                EventMsg::Warning(WarningEvent {
                    message: format!(
                        "Agent {} was resumed, but its sub-agent {} was not: {}",
                        receiver_thread_id, refused.id, refused.error
                    ),
                }),
            )
            .await;
    }
    turn.session_telemetry
        .counter("codex.multi_agent.resume", /*inc*/ 1, &[]);

    Ok(ResumeAgentResult {
        status,
        descendants_not_resumed,
    })
}

impl CoreToolRuntime for Handler {
    fn matches_kind(&self, payload: &ToolPayload) -> bool {
        matches!(payload, ToolPayload::Function { .. })
    }
}

#[derive(Debug, Deserialize)]
struct ResumeAgentArgs {
    id: String,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct ResumeAgentResult {
    pub(crate) status: AgentStatus,
    /// Open sub-agents of the resumed agent that stayed closed, with why
    /// (#1144).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) descendants_not_resumed: Vec<DescendantNotResumed>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct DescendantNotResumed {
    pub(crate) id: String,
    pub(crate) error: String,
}

impl ToolOutput for ResumeAgentResult {
    fn log_output(&self) -> String {
        tool_output_json_text(self, "resume_agent")
    }

    fn success_for_logging(&self) -> bool {
        true
    }

    fn to_response_item(&self, call_id: &str, payload: &ToolPayload) -> ResponseInputItem {
        tool_output_response_item(call_id, payload, self, Some(true), "resume_agent")
    }

    fn code_mode_result(&self, _payload: &ToolPayload) -> JsonValue {
        tool_output_code_mode_result(self, "resume_agent")
    }
}
