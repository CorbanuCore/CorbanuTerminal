use super::analytics::ToolCallAnalytics;
use super::message_tool::FollowupTaskArgs;
use super::message_tool::handle_message_string_tool;
use super::*;
use crate::agent::types::MessageDeliveryMode;
use crate::tools::handlers::multi_agents_spec::create_followup_task_tool;
use codex_tools::ToolSpec;

pub(crate) struct Handler;

impl ToolExecutor<ToolInvocation> for Handler {
    fn tool_name(&self) -> ToolName {
        ToolName::plain("followup_task")
    }

    fn spec(&self) -> ToolSpec {
        create_followup_task_tool()
    }

    fn handle<'a>(&'a self, invocation: ToolInvocation) -> codex_tools::ToolExecutorFuture<'a>
    where
        ToolInvocation: 'a,
    {
        Box::pin(async move {
            let mut analytics = ToolCallAnalytics::new(&invocation, CollabAgentTool::FollowupTask);
            let result = self.handle_call(invocation, &mut analytics).await;
            analytics.finish(&result);
            result
        })
    }
}

impl Handler {
    async fn handle_call(
        &self,
        invocation: ToolInvocation,
        analytics: &mut ToolCallAnalytics,
    ) -> Result<Box<dyn crate::tools::context::ToolOutput>, FunctionCallError> {
        ensure_manager_tool_allowed(&invocation.turn, "followup_task")?;
        let arguments = function_arguments(invocation.payload.clone())?;
        let args: FollowupTaskArgs = parse_arguments(&arguments)?;
        handle_message_string_tool(
            invocation,
            MessageDeliveryMode::TriggerTurn,
            CollaborationMessageEncoding::ProviderNative,
            args.target,
            args.message,
            analytics,
        )
        .await
        .map(boxed_tool_output)
    }
}

pub(crate) struct PlaintextHandler;

impl ToolExecutor<ToolInvocation> for PlaintextHandler {
    fn tool_name(&self) -> ToolName {
        ToolName::plain(PLAINTEXT_FOLLOWUP_TASK_TOOL)
    }

    fn spec(&self) -> ToolSpec {
        plaintext_adapter_spec(
            create_followup_task_tool(),
            PLAINTEXT_FOLLOWUP_TASK_TOOL,
            "followup_task",
        )
    }

    fn handle<'a>(&'a self, invocation: ToolInvocation) -> codex_tools::ToolExecutorFuture<'a>
    where
        ToolInvocation: 'a,
    {
        Box::pin(async move {
            let mut analytics = ToolCallAnalytics::new(&invocation, CollabAgentTool::FollowupTask);
            let result = async {
                ensure_manager_tool_allowed(&invocation.turn, PLAINTEXT_FOLLOWUP_TASK_TOOL)?;
                let arguments = function_arguments(invocation.payload.clone())?;
                let args: FollowupTaskArgs = parse_arguments(&arguments)?;
                handle_message_string_tool(
                    invocation,
                    MessageDeliveryMode::TriggerTurn,
                    CollaborationMessageEncoding::PlaintextAdapter,
                    args.target,
                    args.message,
                    &mut analytics,
                )
                .await
                .map(boxed_tool_output)
            }
            .await;
            analytics.finish(&result);
            result
        })
    }
}

impl CoreToolRuntime for PlaintextHandler {
    fn matches_kind(&self, payload: &ToolPayload) -> bool {
        matches!(payload, ToolPayload::Function { .. })
    }
}

impl CoreToolRuntime for Handler {
    fn matches_kind(&self, payload: &ToolPayload) -> bool {
        matches!(payload, ToolPayload::Function { .. })
    }
}
