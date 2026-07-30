//! Implements the MultiAgentV2 collaboration tool surface.

use crate::agent::AgentStatus;
use crate::agent::agent_resolver::resolve_agent_target;
use crate::context::ContextualUserFragment;
use crate::context::InterAgentMessage;
use crate::context::InterAgentMessageType;
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
use codex_protocol::items::CollabAgentTool;
use codex_protocol::items::CollabAgentToolCallItem;
use codex_protocol::items::CollabAgentToolCallStatus;
use codex_protocol::items::SubAgentActivityItem;
use codex_protocol::items::TurnItem;
use codex_protocol::models::ResponseInputItem;
use codex_protocol::openai_models::ReasoningEffort;
use codex_protocol::protocol::InterAgentCommunication;
use codex_protocol::protocol::SubAgentActivityKind;
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

pub(crate) async fn emit_sub_agent_activity(
    session: &crate::session::session::Session,
    turn: &crate::session::turn_context::TurnContext,
    item: SubAgentActivityItem,
) {
    let item = TurnItem::SubAgentActivity(item);
    session.emit_turn_item_started(turn, &item).await;
    session.emit_turn_item_completed(turn, item).await;
}

fn communication_from_tool_message(
    author: AgentPath,
    recipient: AgentPath,
    message: String,
    source: &crate::tools::context::ToolCallSource,
    trigger_turn: bool,
) -> InterAgentCommunication {
    if !matches!(
        source,
        crate::tools::context::ToolCallSource::DirectPlaintextMessage
    ) {
        return InterAgentCommunication::new_encrypted(
            author,
            recipient,
            Vec::new(),
            message,
            trigger_turn,
        );
    }
    let message_type = if trigger_turn {
        InterAgentMessageType::NewTask
    } else {
        InterAgentMessageType::Message
    };
    let content =
        InterAgentMessage::new(message_type, recipient.clone(), author.clone(), message).render();
    InterAgentCommunication::new(author, recipient, Vec::new(), content, trigger_turn)
}

pub(super) fn ensure_manager_tool_allowed(
    _turn: &TurnContext,
    _tool_name: &str,
) -> Result<(), FunctionCallError> {
    // Role names are behavioral profiles and UI labels, not authorization tokens. Delegation
    // permissions belong to the crew policy at the PfTerminal control-plane boundary.
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
