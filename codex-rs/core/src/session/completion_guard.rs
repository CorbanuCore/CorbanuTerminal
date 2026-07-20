use crate::client_common::Prompt;
use crate::session::TurnInput;
use codex_protocol::models::BaseInstructions;
use codex_protocol::models::ContentItem;
use codex_protocol::models::ResponseItem;
use codex_protocol::user_input::UserInput;
use serde::Deserialize;
use serde_json::json;

pub(super) const MAX_COMPLETION_ASSESSMENT_ATTEMPTS: u64 = 3;
const MAX_OBJECTIVE_CHARS: usize = 6_000;
const MAX_ASSISTANT_RESPONSE_CHARS: usize = 8_000;

const ASSESSMENT_INSTRUCTIONS: &str = "You are a task-completion classifier. Decide whether the assistant's latest response genuinely ends the user's requested action turn. Return only the required JSON. Classify complete when the response reports completed work, gives a final answer, or explicitly reports a blocker or need for user input. Classify incomplete when it announces a next action, describes partial progress, or leaves requested work unresolved. Use uncertain when the evidence does not support either conclusion. Do not infer completion merely from confident tone.";

pub(super) const CONTINUE_INSTRUCTION: &str = "A completion check found that the preceding response did not finish the requested work. Continue the work now, using tools when needed. Do not merely announce a next action. If progress is genuinely blocked or user input is required, give a final response that states the blocker explicitly.";

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub(super) enum CompletionAssessment {
    Complete,
    Incomplete,
    Uncertain,
}

#[derive(Debug, Deserialize)]
struct CompletionAssessmentOutput {
    decision: CompletionAssessment,
}

#[derive(Debug, Default)]
pub(super) struct CompletionGuardState {
    unsuccessful_assessments: u64,
}

impl CompletionGuardState {
    pub(super) fn should_assess(
        &self,
        provider_requires_guard: bool,
        needs_follow_up: bool,
    ) -> bool {
        // These providers have demonstrated premature `stop` responses both before their first
        // tool call and after tool work. A provider stop is therefore only a transport signal;
        // the structured assessment decides whether the user-visible action turn is complete.
        provider_requires_guard && !needs_follow_up
    }

    pub(super) fn record_assessment(
        &mut self,
        assessment: CompletionAssessment,
    ) -> CompletionGuardAction {
        match assessment {
            CompletionAssessment::Complete => {
                self.unsuccessful_assessments = 0;
                CompletionGuardAction::Accept
            }
            CompletionAssessment::Incomplete | CompletionAssessment::Uncertain => {
                self.unsuccessful_assessments += 1;
                if self.unsuccessful_assessments >= MAX_COMPLETION_ASSESSMENT_ATTEMPTS {
                    CompletionGuardAction::AcceptAfterLimit
                } else {
                    CompletionGuardAction::Continue
                }
            }
        }
    }

    pub(super) fn unsuccessful_assessments(&self) -> u64 {
        self.unsuccessful_assessments
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum CompletionGuardAction {
    Accept,
    Continue,
    AcceptAfterLimit,
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

pub(super) fn assessment_prompt(objective: &str, assistant_response: &str) -> Prompt {
    let objective = if objective.trim().is_empty() {
        "(No plain-text user objective was available; judge only whether the response itself claims unfinished work.)".to_string()
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
                    "USER OBJECTIVE:\n{objective}\n\nASSISTANT RESPONSE TO ASSESS:\n{assistant_response}"
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
                "decision": {
                    "type": "string",
                    "enum": ["complete", "incomplete", "uncertain"]
                }
            },
            "required": ["decision"],
            "additionalProperties": false
        })),
        output_schema_strict: true,
        ..Prompt::default()
    }
}

pub(super) fn parse_assessment(text: &str) -> Result<CompletionAssessment, serde_json::Error> {
    let value = serde_json::from_str::<serde_json::Value>(text.trim())?;
    if let Ok(output) = serde_json::from_value::<CompletionAssessmentOutput>(value.clone()) {
        return Ok(output.decision);
    }

    // Some chat-compatible providers ignore the requested property shape, returning either a
    // renamed enum (for example {"status":"incomplete"}) or a completion boolean. Accept only
    // one-field objects with a recognized enum, or boolean fields whose names explicitly mean
    // completion. Multiple fields, prose, unrelated booleans, and unknown values stay invalid.
    if let serde_json::Value::Object(object) = &value
        && object.len() == 1
        && let Some((key, decision)) = object.iter().next()
    {
        if let serde_json::Value::String(decision) = decision {
            return serde_json::from_value::<CompletionAssessment>(serde_json::Value::String(
                decision.clone(),
            ));
        }
        if let serde_json::Value::Bool(complete) = decision
            && matches!(
                key.as_str(),
                "complete" | "completed" | "is_complete" | "is_completed" | "done"
            )
        {
            return Ok(if *complete {
                CompletionAssessment::Complete
            } else {
                CompletionAssessment::Incomplete
            });
        }
    }

    serde_json::from_value::<CompletionAssessmentOutput>(value).map(|output| output.decision)
}

pub(super) fn continuation_message() -> ResponseItem {
    ResponseItem::Message {
        id: None,
        role: "developer".to_string(),
        content: vec![ContentItem::InputText {
            text: CONTINUE_INSTRUCTION.to_string(),
        }],
        phase: None,
        metadata: None,
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guard_is_provider_scoped_and_activates_before_tool_work() {
        let state = CompletionGuardState::default();
        assert!(!state.should_assess(false, false));
        assert!(!state.should_assess(true, true));
        assert!(state.should_assess(true, false));
    }

    #[test]
    fn incomplete_and_uncertain_assessments_request_bounded_continuation() {
        let mut state = CompletionGuardState::default();
        assert_eq!(
            state.record_assessment(CompletionAssessment::Incomplete),
            CompletionGuardAction::Continue
        );
        assert_eq!(
            state.record_assessment(CompletionAssessment::Uncertain),
            CompletionGuardAction::Continue
        );
        assert_eq!(state.unsuccessful_assessments(), 2);
        assert_eq!(
            state.record_assessment(CompletionAssessment::Incomplete),
            CompletionGuardAction::AcceptAfterLimit
        );
        assert_eq!(state.unsuccessful_assessments(), 3);
    }

    #[test]
    fn parser_accepts_only_structured_decisions() {
        assert_eq!(
            parse_assessment(r#"{"decision":"complete"}"#).unwrap(),
            CompletionAssessment::Complete
        );
        assert!(parse_assessment("Now I will continue.").is_err());
        assert!(parse_assessment(r#"{"decision":"yes"}"#).is_err());
        assert_eq!(
            parse_assessment(r#"{"status":"incomplete"}"#).unwrap(),
            CompletionAssessment::Incomplete
        );
        assert!(parse_assessment(r#"{"status":"incomplete","confidence":1}"#).is_err());
        assert_eq!(
            parse_assessment(r#"{"completed":false}"#).unwrap(),
            CompletionAssessment::Incomplete
        );
        assert_eq!(
            parse_assessment(r#"{"done":true}"#).unwrap(),
            CompletionAssessment::Complete
        );
        assert!(parse_assessment(r#"{"ready":false}"#).is_err());
    }

    #[test]
    fn assessment_prompt_is_bounded_and_has_no_completion_marker_protocol() {
        let prompt = assessment_prompt(&"x".repeat(7_000), &"y".repeat(9_000));
        let ResponseItem::Message { content, .. } = &prompt.input[0] else {
            panic!("expected message");
        };
        let ContentItem::InputText { text } = &content[0] else {
            panic!("expected input text");
        };
        assert!(text.len() < 15_000);
        assert!(!text.contains("pfterminal-task-complete"));
        assert!(prompt.output_schema.is_some());
    }
}
