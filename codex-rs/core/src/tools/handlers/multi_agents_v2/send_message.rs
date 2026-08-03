use super::message_tool::MessageDeliveryMode;
use super::message_tool::SendMessageArgs;
use super::message_tool::handle_message_string_tool;
use super::*;
use crate::tools::handlers::multi_agents_spec::create_send_message_tool;
use codex_tools::ToolSpec;

pub(crate) struct Handler;

impl ToolExecutor<ToolInvocation> for Handler {
    fn tool_name(&self) -> ToolName {
        ToolName::plain("send_message")
    }

    fn spec(&self) -> ToolSpec {
        create_send_message_tool()
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
        let arguments = function_arguments(invocation.payload.clone())?;
        let args: SendMessageArgs = parse_arguments(&arguments)?;
        handle_message_string_tool(
            invocation,
            MessageDeliveryMode::QueueOnly,
            CollaborationMessageEncoding::ProviderNative,
            args.target,
            args.message,
        )
        .await
        .map(boxed_tool_output)
    }
}

pub(crate) struct PlaintextHandler;

impl ToolExecutor<ToolInvocation> for PlaintextHandler {
    fn tool_name(&self) -> ToolName {
        ToolName::plain(PLAINTEXT_SEND_MESSAGE_TOOL)
    }

    fn spec(&self) -> ToolSpec {
        plaintext_adapter_spec(
            create_send_message_tool(),
            PLAINTEXT_SEND_MESSAGE_TOOL,
            "send_message",
        )
    }

    fn handle(&self, invocation: ToolInvocation) -> codex_tools::ToolExecutorFuture<'_> {
        Box::pin(async move {
            let arguments = function_arguments(invocation.payload.clone())?;
            let args: SendMessageArgs = parse_arguments(&arguments)?;
            handle_message_string_tool(
                invocation,
                MessageDeliveryMode::QueueOnly,
                CollaborationMessageEncoding::PlaintextAdapter,
                args.target,
                args.message,
            )
            .await
            .map(boxed_tool_output)
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
