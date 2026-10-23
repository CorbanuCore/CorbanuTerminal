use std::sync::Arc;
use std::sync::OnceLock;

use super::trim_function_call_history_to_fit_context_window;
use crate::Prompt;
use crate::client::CompactConversationRequestSettings;
use crate::compact::CompactionAnalyticsDetails;
use crate::provider_throttle::AdmittedProviderRequest;
use crate::provider_throttle::AuxiliaryRequest;
use crate::provider_throttle::record_auxiliary_outcome;
use crate::responses_metadata::CodexResponsesRequestKind;
use crate::responses_metadata::CompactionTurnMetadata;
use crate::session::session::Session;
use crate::session::step_context::StepContext;
use codex_protocol::auth::AuthMode;
use codex_protocol::error::Result as CodexResult;
use codex_protocol::models::ResponseItem;
use codex_rollout_trace::CompactionTraceContext;
use tracing::info;

pub(super) struct RemoteCompactAttempt {
    pub(super) new_history: Vec<ResponseItem>,
    pub(super) trace_input_history: Option<Vec<ResponseItem>>,
}

pub(super) async fn run_remote_compact_attempt(
    sess: &Arc<Session>,
    step_context: &Arc<StepContext>,
    client_session: &crate::client::ModelClientSession,
    turn_state: Option<Arc<OnceLock<String>>>,
    compaction_trace: &CompactionTraceContext,
    compaction_metadata: CompactionTurnMetadata,
    analytics_details: &mut CompactionAnalyticsDetails,
) -> CodexResult<RemoteCompactAttempt> {
    let turn_context = &step_context.turn;
    let mut history = sess.clone_history().await;
    let base_instructions = sess.get_base_instructions().await;
    let (rewritten_outputs, estimated_deleted_tokens) =
        trim_function_call_history_to_fit_context_window(
            &mut history,
            turn_context.as_ref(),
            &base_instructions,
        );
    if rewritten_outputs > 0 {
        info!(
            turn_id = %turn_context.sub_id,
            rewritten_outputs,
            "rewrote history outputs before remote compaction"
        );
    }
    if estimated_deleted_tokens > 0 {
        let max_local_deleted_tokens = sess
            .estimated_tokens_after_last_model_generated_item()
            .await;
        analytics_details.active_context_tokens_before = analytics_details
            .active_context_tokens_before
            .map(|active_context_tokens_before| {
                active_context_tokens_before
                    .saturating_sub(estimated_deleted_tokens.min(max_local_deleted_tokens))
            });
    }
    let trace_input_history = compaction_trace
        .is_enabled()
        .then(|| history.raw_items().to_vec());
    let prompt_input = history.for_turn_prompt(turn_context);
    let tool_router = &step_context.tool_router;
    let prompt = Prompt {
        input: prompt_input,
        tools: tool_router.model_visible_specs(),
        parallel_tool_calls: turn_context.model_info.supports_parallel_tool_calls,
        base_instructions,
        output_schema: None,
        output_schema_strict: true,
        producer: Some(turn_context.output_producer()),
        flat_tool_names: tool_router.flat_tool_names().clone(),
    };
    let window_id = sess.current_window_id().await;
    let responses_metadata = turn_context.turn_metadata_state.to_responses_metadata(
        sess.installation_id.clone(),
        window_id,
        CodexResponsesRequestKind::Compaction(compaction_metadata),
    );
    // #509: remote compaction is a model request on the turn's key. An empty
    // history sends nothing.
    let admitted = if prompt.input.is_empty() {
        None
    } else {
        AdmittedProviderRequest::admit_for_turn(
            AuxiliaryRequest::Compaction,
            sess,
            turn_context,
            &prompt,
        )
        .await?
    };
    let mut unresolvable_item_retries = 0;
    let new_history = loop {
        let result = sess
            .services
            .model_client()
            .compact_conversation_history(
                &prompt,
                &turn_context.model_info,
                turn_state.clone(),
                CompactConversationRequestSettings {
                    effort: turn_context.reasoning_effort.clone(),
                    summary: turn_context.reasoning_summary,
                    service_tier: if sess.services.auth_manager().auth_mode()
                        == Some(AuthMode::ApiKey)
                    {
                        None
                    } else {
                        turn_context.config.service_tier.clone()
                    },
                },
                &turn_context.session_telemetry,
                compaction_trace,
                &responses_metadata,
                &client_session.responses_accounting,
            )
            .await;
        match result {
            // #578: the provider could not find an item this request
            // referenced; the retry is part of the same admitted request.
            Err(err)
                if crate::session::turn::retry_without_unresolvable_item(
                    client_session,
                    &err,
                    &mut unresolvable_item_retries,
                ) => {}
            result => {
                record_auxiliary_outcome(admitted.as_ref(), &result).await;
                break result?;
            }
        }
    };
    Ok(RemoteCompactAttempt {
        new_history,
        trace_input_history,
    })
}

#[cfg(test)]
mod throttle_tests {
    use super::*;
    use crate::provider_throttle::tests::RateLimitedSession;
    use crate::provider_throttle::tests::assert_rate_limited_then_blocked;
    use codex_analytics::CompactionImplementation;
    use codex_analytics::CompactionPhase;
    use codex_analytics::CompactionReason;
    use codex_analytics::CompactionTrigger;

    /// #509: a remote compaction respects the key's cooldown and records its
    /// own 429.
    #[tokio::test]
    async fn remote_compaction_respects_and_records_the_provider_request_throttle() {
        let RateLimitedSession {
            server,
            session,
            turn_context,
            home: _home,
            cwd: _cwd,
        } = RateLimitedSession::start().await;
        session
            .record_conversation_items(
                &turn_context,
                &[ResponseItem::Message {
                    id: None,
                    role: "user".to_string(),
                    content: vec![codex_protocol::models::ContentItem::InputText {
                        text: "compact this".to_string(),
                    }],
                    phase: None,
                    internal_chat_message_metadata_passthrough: None,
                }],
            )
            .await;
        let step_context = StepContext::for_test(Arc::new(turn_context));
        let session = Arc::new(session);
        let client_session = session.services.model_client().new_session();
        let attempt = async || {
            let mut analytics = CompactionAnalyticsDetails::default();
            match run_remote_compact_attempt(
                &session,
                &step_context,
                &client_session,
                /*turn_state*/ None,
                &CompactionTraceContext::disabled(),
                CompactionTurnMetadata::new(
                    CompactionTrigger::Manual,
                    CompactionReason::UserRequested,
                    CompactionImplementation::ResponsesCompact,
                    CompactionPhase::StandaloneTurn,
                ),
                &mut analytics,
            )
            .await
            {
                Ok(_) => panic!("the provider refuses every request"),
                Err(error) => error,
            }
        };
        let first = attempt().await;
        let second = attempt().await;
        assert_rate_limited_then_blocked(&first, &second);
        assert_eq!(
            server.received_requests().await.unwrap_or_default().len(),
            1
        );
    }
}
