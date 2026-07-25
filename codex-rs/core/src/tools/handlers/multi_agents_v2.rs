//! Implements the MultiAgentV2 collaboration tool surface.

use crate::agent::AgentStatus;
use crate::agent::agent_resolver::resolve_agent_target;
use crate::function_tool::FunctionCallError;
use crate::session::turn_context::TurnContext;
use crate::tools::context::ToolInvocation;
use crate::tools::context::ToolOutput;
use crate::tools::context::ToolPayload;
use crate::tools::context::boxed_tool_output;
use crate::tools::handlers::multi_agents_common::*;
use crate::tools::handlers::parse_arguments;
use crate::tools::registry::CoreToolRuntime;
use crate::tools::registry::ToolExecutor;
#[cfg(test)]
use codex_model_provider_info::OPENAI_PROVIDER_ID;
use codex_protocol::AgentPath;
use codex_protocol::models::ResponseInputItem;
use codex_protocol::openai_models::ReasoningEffort;
use codex_protocol::protocol::CollabWaitingBeginEvent;
use codex_protocol::protocol::CollabWaitingEndEvent;
use codex_protocol::protocol::InterAgentCommunication;
use codex_protocol::protocol::SubAgentActivityEvent;
use codex_protocol::protocol::SubAgentActivityKind;
use codex_protocol::user_input::UserInput;
use codex_tools::ToolName;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value as JsonValue;

pub(crate) use followup_task::Handler as FollowupTaskHandler;
pub(crate) use interrupt_agent::Handler as InterruptAgentHandler;
pub(crate) use list_agents::Handler as ListAgentsHandler;
pub(crate) use send_message::Handler as SendMessageHandler;
pub(crate) use spawn::Handler as SpawnAgentHandler;
pub(crate) use wait::Handler as WaitAgentHandler;

mod followup_task;
mod interrupt_agent;
mod list_agents;
mod message_tool;
mod send_message;
mod spawn;
pub(crate) mod wait;

pub(super) fn communication_from_model_tool_message(
    author: AgentPath,
    recipient: AgentPath,
    message: String,
    _provider_id: &str,
) -> InterAgentCommunication {
    // A durable heterogeneous mailbox must be able to deliver the same message to any provider.
    // OpenAI's encrypted-string tool extension produces an opaque value that non-OpenAI
    // recipients cannot consume, so the canonical V2 bus intentionally carries bounded
    // plaintext and keeps it out of previews and logs.
    InterAgentCommunication::new(
        author,
        recipient,
        Vec::new(),
        message,
        /*trigger_turn*/ true,
    )
}

pub(super) fn ensure_manager_tool_allowed(
    turn: &TurnContext,
    tool_name: &str,
) -> Result<(), FunctionCallError> {
    if turn
        .session_source
        .get_agent_role()
        .as_deref()
        .is_some_and(|role| role.eq_ignore_ascii_case("orc"))
    {
        return Err(FunctionCallError::RespondToModel(format!(
            "{tool_name} rejected by the runtime: caller role orc has no manager tools; return your report to your parent Troll instead"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn openai_collaboration_tool_payload_is_provider_neutral_plaintext() {
        let communication = communication_from_model_tool_message(
            AgentPath::root(),
            AgentPath::try_from("/root/worker").expect("agent path"),
            "opaque-ciphertext".to_string(),
            OPENAI_PROVIDER_ID,
        );

        assert_eq!(communication.content, "opaque-ciphertext");
        assert_eq!(communication.encrypted_content, None);
    }

    #[test]
    fn non_openai_collaboration_tool_payload_is_plaintext() {
        let communication = communication_from_model_tool_message(
            AgentPath::try_from("/root/worker").expect("agent path"),
            AgentPath::root(),
            "ordinary provider report".to_string(),
            "zai",
        );

        assert_eq!(communication.content, "ordinary provider report");
        assert_eq!(communication.encrypted_content, None);
        assert!(matches!(
            communication.to_model_input_item(),
            codex_protocol::models::ResponseItem::AgentMessage { content, .. }
                if matches!(content.as_slice(), [codex_protocol::models::AgentMessageInputContent::InputText { text }] if text == "ordinary provider report")
        ));
    }
}
