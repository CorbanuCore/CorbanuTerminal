use std::time::Duration;

use crate::client::ModelClientSession;
use crate::client_common::Prompt;
use crate::client_common::ResponseEvent;
use crate::responses_metadata::CodexResponsesMetadata;
use crate::session::TurnInput;
use crate::session::session::Session;
use crate::session::turn_context::TurnContext;
use crate::stream_events_utils::raw_assistant_output_text_from_item;
use codex_async_utils::OrCancelExt;
use codex_protocol::config_types::ReasoningSummary as ReasoningSummaryConfig;
use codex_protocol::error::CodexErr;
use codex_protocol::error::Result as CodexResult;
use codex_protocol::models::BaseInstructions;
use codex_protocol::models::ContentItem;
use codex_protocol::models::ResponseItem;
use codex_protocol::openai_models::ReasoningEffort as ReasoningEffortConfig;
use codex_protocol::user_input::UserInput;
use codex_rollout_trace::InferenceTraceContext;
use futures::StreamExt;
use serde::Deserialize;
use serde_json::json;
use sha2::Digest;
use sha2::Sha256;
use tokio_util::sync::CancellationToken;

// This is an end-to-end network deadline, not a model-compute target. Live Kimi
// traffic has occasionally taken 7–8 seconds merely to return HTTP 200, so a
// six-second deadline misclassified normal provider latency as a semantic
// failure. Keep this bounded but above observed tail latency.
const ASSESSMENT_TIMEOUT: Duration = Duration::from_secs(15);
const MAX_OBJECTIVE_CHARS: usize = 4_000;
const MAX_ASSISTANT_RESPONSE_CHARS: usize = 6_000;

const ASSESSMENT_INSTRUCTIONS: &str = concat!(
    "Decide whether the assistant's latest response completes the user's current request. ",
    "Return only the required JSON. Use complete when the requested answer or action was ",
    "delivered. Use incomplete when the response is a progress checkpoint, announces work ",
    "that was not performed, or omits a requested deliverable. Use awaiting_user when a user ",
    "choice, credential, authorization, or missing input is required. Use blocked when an ",
    "external condition prevents useful progress and the response clearly reports it. Use ",
    "uncertain only when the evidence cannot distinguish these states. Do not treat confident ",
    "tone or a provider stop signal as evidence of completion."
);

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub(super) enum CompletionAssessment {
    Complete,
    Incomplete,
    AwaitingUser,
    Blocked,
    Uncertain,
}

#[derive(Debug, Deserialize)]
struct CompletionAssessmentOutput {
    state: CompletionAssessment,
    reason: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum CompletionAction {
    Accept,
    Continue,
    AcceptWithWarning,
    StopStalled,
}

#[derive(Debug, Default)]
pub(super) struct CompletionProgressState {
    tool_progress_epoch: u64,
    last_decision_tool_epoch: u64,
    consecutive_no_progress: u8,
    continuation_count: u64,
    last_response_digest: Option<[u8; 32]>,
    empty_stop_seen: bool,
    last_decision_reason: Option<&'static str>,
    last_progress_reset_reason: Option<&'static str>,
}

impl CompletionProgressState {
    pub(super) fn note_tool_progress(&mut self) {
        self.tool_progress_epoch = self.tool_progress_epoch.saturating_add(1);
    }

    pub(super) fn continuation_count(&self) -> u64 {
        self.continuation_count
    }

    pub(super) fn consecutive_no_progress_count(&self) -> u8 {
        self.consecutive_no_progress
    }

    pub(super) fn decision_reason(&self) -> &'static str {
        self.last_decision_reason.unwrap_or("not_evaluated")
    }

    pub(super) fn progress_reset_reason(&self) -> &'static str {
        self.last_progress_reset_reason.unwrap_or("none")
    }

    pub(super) fn decide(
        &mut self,
        assessment: CompletionAssessment,
        assistant_response: &str,
    ) -> CompletionAction {
        match assessment {
            CompletionAssessment::Complete => {
                self.reset_after_terminal_decision("complete");
                CompletionAction::Accept
            }
            CompletionAssessment::AwaitingUser => {
                self.reset_after_terminal_decision("awaiting_user");
                CompletionAction::Accept
            }
            CompletionAssessment::Blocked => {
                self.reset_after_terminal_decision("blocked");
                CompletionAction::Accept
            }
            CompletionAssessment::Incomplete => self.record_incomplete_response(assistant_response),
            CompletionAssessment::Uncertain => {
                self.fallback_after_failed_assessment(assistant_response)
            }
        }
    }

    pub(super) fn decide_empty_stop(&mut self) -> CompletionAction {
        if self.empty_stop_seen {
            self.last_decision_reason = Some("repeated_empty_stop");
            return CompletionAction::StopStalled;
        }
        self.empty_stop_seen = true;
        self.continuation_count = self.continuation_count.saturating_add(1);
        self.last_decision_reason = Some("first_empty_stop");
        CompletionAction::Continue
    }

    pub(super) fn fallback_after_failed_assessment(
        &mut self,
        assistant_response: &str,
    ) -> CompletionAction {
        let digest = response_digest(assistant_response);
        let repeated = self.last_response_digest.is_some_and(|last| last == digest);
        let has_new_tool_progress = self.tool_progress_epoch > self.last_decision_tool_epoch;
        self.last_decision_tool_epoch = self.tool_progress_epoch;
        self.last_response_digest = Some(digest);
        self.last_progress_reset_reason = has_new_tool_progress.then_some("new_tool_progress");

        if repeated {
            self.last_decision_reason = Some("assessment_failed_repeated_response");
            CompletionAction::StopStalled
        } else if has_new_tool_progress {
            self.continuation_count = self.continuation_count.saturating_add(1);
            self.last_decision_reason = Some("assessment_failed_with_tool_progress");
            CompletionAction::Continue
        } else {
            self.last_decision_reason = Some("assessment_failed_without_new_tool_progress");
            CompletionAction::AcceptWithWarning
        }
    }

    fn record_incomplete_response(&mut self, assistant_response: &str) -> CompletionAction {
        let has_new_tool_progress = self.tool_progress_epoch > self.last_decision_tool_epoch;
        if has_new_tool_progress {
            self.consecutive_no_progress = 0;
            self.last_progress_reset_reason = Some("new_tool_progress");
        } else {
            self.consecutive_no_progress = self.consecutive_no_progress.saturating_add(1);
            self.last_progress_reset_reason = None;
        }

        self.last_decision_tool_epoch = self.tool_progress_epoch;
        self.last_response_digest = Some(response_digest(assistant_response));
        if self.consecutive_no_progress >= 2 {
            self.last_decision_reason = Some("incomplete_without_progress_limit");
            CompletionAction::StopStalled
        } else {
            self.continuation_count = self.continuation_count.saturating_add(1);
            self.last_decision_reason = Some(if has_new_tool_progress {
                "incomplete_after_tool_progress"
            } else {
                "incomplete_without_progress"
            });
            CompletionAction::Continue
        }
    }

    fn reset_after_terminal_decision(&mut self, reason: &'static str) {
        self.last_decision_tool_epoch = self.tool_progress_epoch;
        self.consecutive_no_progress = 0;
        self.empty_stop_seen = false;
        self.last_decision_reason = Some(reason);
        self.last_progress_reset_reason = Some("terminal_decision");
    }
}

pub(super) fn objective_from_turn_input(input: &[TurnInput]) -> String {
    let text = input
        .iter()
        .filter_map(|item| match item {
            TurnInput::UserInput { content, .. } => Some(content),
            TurnInput::ResponseItem(_) | TurnInput::InterAgentCommunication(_) => None,
        })
        .flatten()
        .filter_map(|item| match item {
            UserInput::Text { text, .. } => Some(text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n");
    truncate_chars(&text, MAX_OBJECTIVE_CHARS)
}

pub(super) async fn assess_turn_completion(
    sess: &Session,
    turn_context: &TurnContext,
    client_session: &mut ModelClientSession,
    responses_metadata: &CodexResponsesMetadata,
    objective: &str,
    assistant_response: &str,
    cancellation_token: &CancellationToken,
) -> CodexResult<CompletionAssessment> {
    let assessment = assess_turn_completion_inner(
        sess,
        turn_context,
        client_session,
        responses_metadata,
        objective,
        assistant_response,
        cancellation_token,
    );
    tokio::time::timeout(ASSESSMENT_TIMEOUT, assessment)
        .await
        .map_err(|_| {
            CodexErr::Stream(
                format!(
                    "completion assessment timed out after {} ms",
                    ASSESSMENT_TIMEOUT.as_millis()
                ),
                None,
            )
        })?
}

async fn assess_turn_completion_inner(
    sess: &Session,
    turn_context: &TurnContext,
    client_session: &mut ModelClientSession,
    responses_metadata: &CodexResponsesMetadata,
    objective: &str,
    assistant_response: &str,
    cancellation_token: &CancellationToken,
) -> CodexResult<CompletionAssessment> {
    let prompt = assessment_prompt(objective, assistant_response);
    let stream_result = client_session
        .stream(
            &prompt,
            &turn_context.model_info,
            &turn_context.session_telemetry,
            Some(ReasoningEffortConfig::Low),
            ReasoningSummaryConfig::None,
            turn_context.config.service_tier.clone(),
            responses_metadata,
            &InferenceTraceContext::disabled(),
        )
        .or_cancel(cancellation_token)
        .await;
    let mut stream = match stream_result {
        Ok(result) => result?,
        Err(_) => return Err(CodexErr::TurnAborted),
    };
    let mut output = None;
    loop {
        let event = match stream.next().or_cancel(cancellation_token).await {
            Ok(Some(event)) => event?,
            Ok(None) => {
                return Err(CodexErr::Stream(
                    "completion assessment stream closed before response.completed".into(),
                    None,
                ));
            }
            Err(_) => return Err(CodexErr::TurnAborted),
        };
        match event {
            ResponseEvent::OutputItemDone(item) => {
                if let Some(text) = raw_assistant_output_text_from_item(&item)
                    && !text.trim().is_empty()
                {
                    output = Some(text);
                }
            }
            ResponseEvent::Completed {
                token_usage,
                finish_reason,
                ..
            } => {
                sess.update_token_usage_info(turn_context, token_usage.as_ref())
                    .await?;
                if let Some(codex_api::CompletionFinishReason::ProviderError(reason)) =
                    finish_reason.as_ref()
                {
                    return Err(CodexErr::Stream(
                        format!(
                            "the model provider ended completion assessment with retryable finish \
                             reason `{reason}`"
                        ),
                        None,
                    ));
                }
                if !matches!(
                    finish_reason,
                    None | Some(codex_api::CompletionFinishReason::Stop)
                ) {
                    return Err(CodexErr::InvalidRequest(format!(
                        "completion assessment ended with finish reason `{}`",
                        finish_reason
                            .as_ref()
                            .map_or("missing", codex_api::CompletionFinishReason::as_str)
                    )));
                }
                let output = output.ok_or_else(|| {
                    CodexErr::InvalidRequest(
                        "completion assessment returned no structured output".to_string(),
                    )
                })?;
                return parse_assessment(&output);
            }
            _ => {}
        }
    }
}

fn assessment_prompt(objective: &str, assistant_response: &str) -> Prompt {
    let objective = if objective.trim().is_empty() {
        "(No plain-text user objective was available.)".to_string()
    } else {
        truncate_chars(objective, MAX_OBJECTIVE_CHARS)
    };
    let assistant_response = truncate_chars(assistant_response, MAX_ASSISTANT_RESPONSE_CHARS);
    Prompt {
        input: vec![ResponseItem::Message {
            id: None,
            role: "user".to_string(),
            content: vec![ContentItem::InputText {
                text: format!(
                    "USER OBJECTIVE:\n{objective}\n\nASSISTANT RESPONSE:\n{assistant_response}"
                ),
            }],
            phase: None,
            metadata: None,
        }],
        base_instructions: BaseInstructions {
            text: ASSESSMENT_INSTRUCTIONS.to_string(),
        },
        output_schema: Some(json!({
            "type": "object",
            "properties": {
                "state": {
                    "type": "string",
                    "enum": [
                        "complete",
                        "incomplete",
                        "awaiting_user",
                        "blocked",
                        "uncertain"
                    ]
                },
                "reason": {
                    "type": "string",
                    "maxLength": 160
                }
            },
            "required": ["state", "reason"],
            "additionalProperties": false
        })),
        output_schema_strict: true,
        ..Prompt::default()
    }
}

fn parse_assessment(text: &str) -> CodexResult<CompletionAssessment> {
    let output =
        serde_json::from_str::<CompletionAssessmentOutput>(text.trim()).map_err(|err| {
            CodexErr::InvalidRequest(format!(
                "completion assessment returned invalid structured output: {err}"
            ))
        })?;
    if output.reason.trim().is_empty() {
        return Err(CodexErr::InvalidRequest(
            "completion assessment returned an empty reason".to_string(),
        ));
    }
    Ok(output.state)
}

fn truncate_chars(text: &str, max_chars: usize) -> String {
    let count = text.chars().count();
    if count <= max_chars {
        return text.to_string();
    }
    let omitted = count - max_chars;
    let tail = text.chars().skip(omitted).collect::<String>();
    format!("[earlier content omitted: {omitted} characters]\n{tail}")
}

fn response_digest(response: &str) -> [u8; 32] {
    Sha256::digest(response.trim().as_bytes()).into()
}

#[cfg(test)]
#[path = "turn_completion_tests.rs"]
mod tests;
