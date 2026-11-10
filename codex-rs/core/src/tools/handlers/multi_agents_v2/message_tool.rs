//! Shared argument parsing and dispatch for the v2 agent messaging tools.
//!
//! `send_message` and `followup_task` share the same submission path and differ only in whether the
//! resulting `InterAgentCommunication` should wake the target immediately.

use super::analytics::ToolCallAnalytics;
use super::*;
use crate::TurnStartOptions;
use crate::agent::child_config::build_agent_resume_config;
use crate::agent::types::MessageDeliveryMode;
use crate::agent_communication::AgentCommunicationContext;
use crate::agent_communication::AgentCommunicationKind;
use crate::tools::context::FunctionToolOutput;
use codex_protocol::AgentPath;
use codex_protocol::error::CodexErr;
use codex_protocol::protocol::MAX_AGENT_MESSAGE_BYTES;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
/// Input for the MultiAgentV2 `send_message` tool.
pub(crate) struct SendMessageArgs {
    pub(crate) target: String,
    pub(crate) message: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
/// Input for the MultiAgentV2 `followup_task` tool.
pub(crate) struct FollowupTaskArgs {
    pub(crate) target: String,
    pub(crate) message: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct MessageToolResult {
    pub(crate) message_id: String,
    pub(crate) target_thread_id: String,
    pub(crate) agent_path: String,
    pub(crate) agent_nickname: Option<String>,
    pub(crate) agent_role: Option<String>,
    pub(crate) delivery: String,
    pub(crate) triggered_turn: bool,
}

pub(super) fn message_content(message: String) -> Result<String, FunctionCallError> {
    if message.trim().is_empty() {
        return Err(FunctionCallError::RespondToModel(
            "Empty message can't be sent to an agent".to_string(),
        ));
    }
    let message_len = message.len();
    if message_len > MAX_AGENT_MESSAGE_BYTES {
        return Err(FunctionCallError::RespondToModel(format!(
            "agent message is {message_len} bytes; maximum is {MAX_AGENT_MESSAGE_BYTES} bytes"
        )));
    }
    Ok(message)
}

/// Handles the shared MultiAgentV2 message flow for both `send_message` and `followup_task`.
pub(super) async fn handle_message_string_tool(
    invocation: ToolInvocation,
    mode: MessageDeliveryMode,
    encoding: CollaborationMessageEncoding,
    target: String,
    message: String,
    analytics: &mut ToolCallAnalytics,
) -> Result<FunctionToolOutput, FunctionCallError> {
    let message = message_content(message)?;
    let ToolInvocation {
        session,
        turn,
        call_id,
        source,
        ..
    } = invocation;
    if mode == MessageDeliveryMode::TriggerTurn {
        ensure_manager_tool_allowed(&turn, "followup_task")?;
    }
    let receiver_thread_id = resolve_agent_target(&session, &turn, &target).await?;
    analytics.set_receiver(receiver_thread_id);
    // Corbanu (D527 (a)): main's message flow runs on the shared handle's local backend (the
    // encoding check needs the recipient's loaded runtime); any other backend fails closed.
    let local_agent_control = session
        .services
        .agent_control
        .local_or_unsupported()
        .map_err(|err| collab_agent_error(receiver_thread_id, err))?;
    let receiver_agent = local_agent_control
        .get_agent_metadata(receiver_thread_id)
        .ok_or_else(|| {
            collab_agent_error(
                receiver_thread_id,
                CodexErr::ThreadNotFound(receiver_thread_id),
            )
        })?;
    if mode == MessageDeliveryMode::TriggerTurn
        && receiver_agent
            .agent_path
            .as_ref()
            .is_some_and(AgentPath::is_root)
    {
        return Err(FunctionCallError::RespondToModel(
            "Follow-up tasks can't target the root agent".to_string(),
        ));
    }
    let receiver_agent_path = receiver_agent.agent_path.clone().ok_or_else(|| {
        FunctionCallError::RespondToModel("target agent is missing an agent_path".to_string())
    })?;
    let author = turn
        .session_source
        .get_agent_path()
        .unwrap_or_else(AgentPath::root);
    // Persisted agents are represented in the control plane before their runtime is loaded.
    // Reopen that runtime first so mailbox admission can use its state database. Live agents
    // take the fast path in `ensure_v2_agent_loaded`.
    let resume_config =
        build_agent_resume_config(&turn).map_err(FunctionCallError::RespondToModel)?;
    let _residency_pin = local_agent_control
        .ensure_v2_agent_loaded(resume_config, receiver_thread_id, /*parent*/ None)
        .await
        .map_err(|err| collab_agent_error(receiver_thread_id, err))?;
    let receiver = local_agent_control
        .get_agent_config_snapshot(receiver_thread_id)
        .await
        .ok_or_else(|| {
            FunctionCallError::RespondToModel(format!(
                "target agent `{receiver_agent_path}` has no loaded provider configuration"
            ))
        })?;
    let (native_tool_name, plaintext_tool_name) = match mode {
        MessageDeliveryMode::QueueOnly => ("send_message", PLAINTEXT_SEND_MESSAGE_TOOL),
        MessageDeliveryMode::TriggerTurn => ("followup_task", PLAINTEXT_FOLLOWUP_TASK_TOOL),
    };
    let codec_provider_id =
        crate::tools::collaboration_wire::message_source_provider_id(turn.as_ref());
    ensure_message_encoding_matches_target(
        ConfiguredRuntime::from_provider(&turn.config.model_provider_id, turn.provider.info()),
        codec_provider_id,
        &source,
        ConfiguredRuntime::from_snapshot(
            &receiver.model_provider_id,
            receiver.provider_account.as_deref(),
        ),
        encoding,
        native_tool_name,
        plaintext_tool_name,
    )?;
    let source = match encoding {
        CollaborationMessageEncoding::ProviderNative => source,
        CollaborationMessageEncoding::PlaintextAdapter => {
            crate::tools::context::ToolCallSource::DirectPlaintextMessage
        }
    };
    let mut communication = agent_message_from_tool(message, &source, codec_provider_id)
        .into_communication(author, receiver_agent_path.clone(), mode);
    communication.kind = Some(match mode {
        MessageDeliveryMode::QueueOnly => codex_protocol::protocol::AgentMessageKind::Informational,
        MessageDeliveryMode::TriggerTurn => codex_protocol::protocol::AgentMessageKind::FollowUp,
    });
    let message_id = communication.ensure_message_identity().to_string();
    communication
        .metadata
        .get_or_insert_with(codex_protocol::models::ResponseItemMetadata::default)
        .source_call_id = Some(call_id.clone());
    let kind = match mode {
        MessageDeliveryMode::QueueOnly => AgentCommunicationKind::Message,
        MessageDeliveryMode::TriggerTurn => AgentCommunicationKind::Followup,
    };
    // The durable mailbox admits the message before the hand-off (#1999/#2033, D485). As main,
    // a follow-up is admitted even when every agent slot is busy; its turn waits for a slot.
    local_agent_control
        .send_inter_agent_communication_without_capacity_check(
            receiver_thread_id,
            communication,
            AgentCommunicationContext::new(kind, session.thread_id),
            TurnStartOptions {
                parent_turn_id: (mode == MessageDeliveryMode::TriggerTurn)
                    .then(|| turn.sub_id.clone()),
                root_turn_id: turn.turn_metadata_state.root_turn_id(),
                turn_trigger: turn.turn_metadata_state.current_turn_trigger(),
                cyber_access_program: turn.cyber_access_program,
                ..Default::default()
            },
        )
        .await
        .map_err(|err| collab_v2_agent_error(receiver_thread_id, err))?;
    local_agent_control.note_native_agent_dispatch(session.thread_id);
    emit_sub_agent_activity(
        &session,
        &turn,
        SubAgentActivityItem {
            model: None,
            reasoning_effort: None,
            id: call_id,
            agent_thread_id: receiver_thread_id,
            agent_path: receiver_agent_path.clone(),
            kind: SubAgentActivityKind::Interacted,
        },
    )
    .await;

    let result = MessageToolResult {
        message_id,
        target_thread_id: receiver_thread_id.to_string(),
        agent_path: receiver_agent_path.to_string(),
        agent_nickname: receiver_agent.agent_nickname,
        agent_role: receiver_agent.agent_role,
        delivery: match mode {
            MessageDeliveryMode::QueueOnly => "queued",
            MessageDeliveryMode::TriggerTurn => "followup_task_sent",
        }
        .to_string(),
        triggered_turn: mode == MessageDeliveryMode::TriggerTurn,
    };
    Ok(FunctionToolOutput::from_text(
        tool_output_json_text(&result, "agent_message"),
        Some(true),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_content_rejects_empty_and_oversized_messages() {
        assert!(message_content(" ".to_string()).is_err());
        assert!(message_content("x".repeat(MAX_AGENT_MESSAGE_BYTES)).is_ok());
        assert_eq!(
            message_content("x".repeat(MAX_AGENT_MESSAGE_BYTES + 1)),
            Err(FunctionCallError::RespondToModel(format!(
                "agent message is {} bytes; maximum is {MAX_AGENT_MESSAGE_BYTES} bytes",
                MAX_AGENT_MESSAGE_BYTES + 1
            )))
        );
    }
}
