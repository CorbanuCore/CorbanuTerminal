use super::*;
use crate::agent::control::ListedAgent;
use crate::tools::handlers::multi_agents_common::spawn_billing_class;
use crate::tools::handlers::multi_agents_spec::create_list_agents_tool;
use codex_models_manager::manager::RefreshStrategy;
use codex_protocol::ThreadId;
use codex_protocol::openai_models::InputModality;
use codex_protocol::openai_models::ReasoningEffort;
use codex_tools::ToolSpec;

pub(crate) struct Handler;

impl ToolExecutor<ToolInvocation> for Handler {
    fn tool_name(&self) -> ToolName {
        ToolName::plain("list_agents")
    }

    fn spec(&self) -> ToolSpec {
        create_list_agents_tool()
    }

    fn handle(&self, invocation: ToolInvocation) -> codex_tools::ToolExecutorFuture<'_> {
        Box::pin(self.handle_call(invocation))
    }
}

impl Handler {
    async fn handle_call(
        &self,
        invocation: ToolInvocation,
    ) -> Result<Box<dyn crate::tools::context::ToolOutput>, FunctionCallError> {
        let ToolInvocation {
            session,
            turn,
            payload,
            ..
        } = invocation;
        let arguments = function_arguments(payload)?;
        let args: ListAgentsArgs = parse_arguments(&arguments)?;
        session
            .services
            .agent_control
            .register_session_root(session.thread_id, turn.parent_thread_id);
        let listed_agents = session
            .services
            .agent_control
            .list_agents(&turn.session_source, args.path_prefix.as_deref())
            .await
            .map_err(collab_spawn_error)?;
        let available_models = session
            .services
            .models_manager
            .list_models(
                RefreshStrategy::Offline,
                turn.config.http_client_factory(),
            )
            .await;
        let mut agents = Vec::with_capacity(listed_agents.len());
        for listed in listed_agents {
            agents.push(ListedAgentReport::from_catalogue(listed, &available_models));
        }

        Ok(boxed_tool_output(ListAgentsResult { agents }))
    }
}

impl CoreToolRuntime for Handler {
    fn matches_kind(&self, payload: &ToolPayload) -> bool {
        matches!(payload, ToolPayload::Function { .. })
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ListAgentsArgs {
    path_prefix: Option<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct ListAgentsResult {
    agents: Vec<ListedAgentReport>,
}

#[derive(Debug, Serialize)]
struct ListedAgentReport {
    agent_name: String,
    agent_thread_id: ThreadId,
    agent_status: AgentStatus,
    model_provider: String,
    model: String,
    reasoning_effort: Option<ReasoningEffort>,
    service_tier: Option<String>,
    capability: String,
    billing: String,
    vision: bool,
}

impl ListedAgentReport {
    fn from_catalogue(
        listed: ListedAgent,
        available_models: &[codex_protocol::openai_models::ModelPreset],
    ) -> Self {
        let preset = available_models.iter().find(|preset| {
                preset.model == listed.model
                    && preset.orchestration.as_ref().is_some_and(|metadata| {
                        metadata.provider_id() == listed.model_provider
                    })
            });
        let metadata = preset.and_then(|preset| preset.orchestration.as_ref());
        Self {
            agent_name: listed.agent_name,
            agent_thread_id: listed.agent_thread_id,
            agent_status: listed.agent_status,
            model_provider: listed.model_provider,
            model: listed.model,
            reasoning_effort: listed.reasoning_effort,
            service_tier: listed.service_tier,
            capability: metadata
                .map(|metadata| metadata.capability().to_string())
                .unwrap_or_else(|| "unavailable".to_string()),
            billing: metadata
                .map(spawn_billing_class)
                .unwrap_or("unavailable")
                .to_string(),
            vision: preset.is_some_and(|preset| {
                preset.input_modalities.contains(&InputModality::Image)
            }),
        }
    }
}

impl ToolOutput for ListAgentsResult {
    fn log_preview(&self) -> String {
        tool_output_json_text(self, "list_agents")
    }

    fn success_for_logging(&self) -> bool {
        true
    }

    fn to_response_item(&self, call_id: &str, payload: &ToolPayload) -> ResponseInputItem {
        tool_output_response_item(call_id, payload, self, Some(true), "list_agents")
    }

    fn code_mode_result(&self, _payload: &ToolPayload) -> JsonValue {
        tool_output_code_mode_result(self, "list_agents")
    }
}
