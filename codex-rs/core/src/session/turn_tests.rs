use super::*;
use codex_extension_api::ExtensionData;
use codex_extension_api::TurnItemContributor;
use codex_protocol::AgentPath;
use codex_protocol::ResponseItemId;
use codex_protocol::items::AgentMessageContent;
use codex_protocol::protocol::InterAgentCommunication;
use pretty_assertions::assert_eq;
use std::sync::Arc;
use tracing_subscriber::prelude::*;

struct RewriteAgentMessageContributor;

impl TurnItemContributor for RewriteAgentMessageContributor {
    fn contribute<'a>(
        &'a self,
        _thread_store: &'a ExtensionData,
        _turn_store: &'a ExtensionData,
        item: &'a mut TurnItem,
    ) -> codex_extension_api::ExtensionFuture<'a, Result<(), String>> {
        Box::pin(async move {
            if let TurnItem::AgentMessage(agent_message) = item {
                agent_message.content = vec![AgentMessageContent::Text {
                    text: "plan contributed assistant text".to_string(),
                }];
            }
            Ok(())
        })
    }
}

fn assistant_output_text(text: &str) -> ResponseItem {
    ResponseItem::Message {
        id: Some(ResponseItemId::with_suffix("msg", "1")),
        role: "assistant".to_string(),
        content: vec![ContentItem::OutputText {
            text: text.to_string(),
        }],
        phase: None,
        internal_chat_message_metadata_passthrough: None,
    }
}

fn collaboration_turn_input(kind: AgentMessageKind) -> TurnInput {
    TurnInput::InterAgentCommunication(
        InterAgentCommunication::new(
            AgentPath::try_from("/root/worker").expect("worker path"),
            AgentPath::root(),
            Vec::new(),
            "mailbox content".to_string(),
            /*trigger_turn*/ true,
        )
        .with_kind(kind),
    )
}

#[test]
fn native_auto_turn_classification_requires_terminal_result_only_input() {
    assert!(turn_inputs_are_terminal_result_only(
        [
            collaboration_turn_input(AgentMessageKind::TerminalResult),
            TurnInput::ResponseItem(assistant_output_text("retained context")),
        ]
        .iter()
    ));
    assert!(!turn_inputs_are_terminal_result_only(
        [
            collaboration_turn_input(AgentMessageKind::TerminalResult),
            collaboration_turn_input(AgentMessageKind::Informational),
        ]
        .iter()
    ));
    assert!(!turn_inputs_are_terminal_result_only(
        [
            collaboration_turn_input(AgentMessageKind::TerminalResult),
            TurnInput::UserInput {
                content: vec![UserInput::Text {
                    text: "operator steering".to_string(),
                    text_elements: Vec::new(),
                }],
                client_id: None,
            },
        ]
        .iter()
    ));
    assert!(!turn_inputs_are_terminal_result_only([].iter()));
}

#[test]
fn parent_completion_requires_triggering_collaboration_without_operator_input() {
    let triggering_collaboration = collaboration_turn_input(AgentMessageKind::FollowUp);
    assert!(turn_inputs_expect_parent_completion(
        [triggering_collaboration.clone()].iter()
    ));
    assert!(turn_inputs_expect_parent_completion(
        [
            triggering_collaboration.clone(),
            TurnInput::ResponseItem(assistant_output_text("retained context")),
        ]
        .iter()
    ));
    assert!(!turn_inputs_expect_parent_completion(
        [
            triggering_collaboration,
            TurnInput::UserInput {
                content: vec![UserInput::Text {
                    text: "operator owns this turn".to_string(),
                    text_elements: Vec::new(),
                }],
                client_id: None,
            },
        ]
        .iter()
    ));

    let queue_only = InterAgentCommunication::new(
        AgentPath::try_from("/root/worker").expect("worker path"),
        AgentPath::root(),
        Vec::new(),
        "informational update".to_string(),
        /*trigger_turn*/ false,
    )
    .with_kind(AgentMessageKind::Informational);
    assert!(!turn_inputs_expect_parent_completion(
        [TurnInput::InterAgentCommunication(queue_only)].iter()
    ));
    assert!(!turn_inputs_expect_parent_completion([].iter()));
}

#[test]
fn post_sampling_token_estimate_is_disabled_by_always_on_sinks() {
    let feedback = codex_feedback::CodexFeedback::new();
    let subscriber = tracing_subscriber::registry()
        .with(feedback.logger_layer())
        .with(tracing_subscriber::fmt::layer().with_filter(codex_state::log_db::default_filter()));

    tracing::subscriber::with_default(subscriber, || {
        assert!(!tracing::event_enabled!(
            target: POST_SAMPLING_TOKEN_ESTIMATE_TARGET,
            tracing::Level::TRACE,
            turn_id,
            estimated_token_count,
            message
        ));
    });
}

#[tokio::test]
async fn plan_mode_uses_contributed_turn_item_for_last_agent_message() {
    let (mut session, turn_context) = crate::session::tests::make_session_and_context().await;
    let mut builder = codex_extension_api::ExtensionRegistryBuilder::new();
    builder.turn_item_contributor(Arc::new(RewriteAgentMessageContributor));
    session.services.extensions = Arc::new(builder.build());
    let turn_store = ExtensionData::new(turn_context.sub_id.clone());
    let mut state = PlanModeStreamState::new(&turn_context.sub_id);
    let mut last_agent_message = None;
    let item = assistant_output_text("original assistant text");

    let handled = handle_assistant_item_done_in_plan_mode(
        &session,
        &turn_context,
        &turn_store,
        &item,
        &mut state,
        /*previously_active_item*/ None,
        &mut last_agent_message,
    )
    .await;

    assert!(handled);
    assert_eq!(
        last_agent_message.as_deref(),
        Some("plan contributed assistant text")
    );
}
