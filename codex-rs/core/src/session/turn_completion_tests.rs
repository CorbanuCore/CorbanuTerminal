use super::*;
use pretty_assertions::assert_eq;

#[test]
fn completed_and_blocked_assessments_end_the_turn() {
    for assessment in [
        CompletionAssessment::Complete,
        CompletionAssessment::AwaitingUser,
        CompletionAssessment::Blocked,
    ] {
        let mut state = CompletionProgressState::default();
        assert_eq!(
            state.decide(assessment, "A terminal response."),
            CompletionAction::Accept
        );
    }
}

#[test]
fn incomplete_assessments_stall_after_two_no_progress_boundaries() {
    let mut state = CompletionProgressState::default();
    assert_eq!(
        state.decide(CompletionAssessment::Incomplete, "Checkpoint one."),
        CompletionAction::Continue
    );
    assert_eq!(
        state.decide(CompletionAssessment::Incomplete, "Checkpoint two."),
        CompletionAction::StopStalled
    );
    assert_eq!(state.consecutive_no_progress_count(), 2);
    assert_eq!(state.decision_reason(), "incomplete_without_progress_limit");
    assert_eq!(state.progress_reset_reason(), "none");
}

#[test]
fn tool_progress_allows_long_productive_turns() {
    let mut state = CompletionProgressState::default();
    for index in 0..10 {
        state.note_tool_progress();
        assert_eq!(
            state.decide(
                CompletionAssessment::Incomplete,
                &format!("Checkpoint {index}.")
            ),
            CompletionAction::Continue
        );
        assert_eq!(state.consecutive_no_progress_count(), 0);
        assert_eq!(state.progress_reset_reason(), "new_tool_progress");
    }
    assert_eq!(state.continuation_count(), 10);
}

#[test]
fn empty_stops_continue_once_then_stall() {
    let mut state = CompletionProgressState::default();
    assert_eq!(state.decide_empty_stop(), CompletionAction::Continue);
    assert_eq!(state.decide_empty_stop(), CompletionAction::StopStalled);
}

#[test]
fn failed_assessments_continue_while_fresh_tool_progress_is_observed() {
    let mut state = CompletionProgressState::default();
    for index in 0..5 {
        state.note_tool_progress();
        assert_eq!(
            state.fallback_after_failed_assessment(&format!("Checkpoint {index}.")),
            CompletionAction::Continue
        );
        assert_eq!(
            state.decision_reason(),
            "assessment_failed_with_tool_progress"
        );
    }
    assert_eq!(state.continuation_count(), 5);
}

#[test]
fn failed_assessment_without_fresh_tool_progress_warns_and_stops() {
    let mut state = CompletionProgressState::default();
    state.note_tool_progress();
    assert_eq!(
        state.fallback_after_failed_assessment("First checkpoint."),
        CompletionAction::Continue
    );
    assert_eq!(
        state.fallback_after_failed_assessment("Different words, no new work."),
        CompletionAction::AcceptWithWarning
    );
    assert_eq!(
        state.decision_reason(),
        "assessment_failed_without_new_tool_progress"
    );
}

#[test]
fn repeated_response_stalls_after_failed_assessment() {
    let mut state = CompletionProgressState::default();
    state.note_tool_progress();
    assert_eq!(
        state.fallback_after_failed_assessment("Same checkpoint."),
        CompletionAction::Continue
    );
    assert_eq!(
        state.fallback_after_failed_assessment("Same checkpoint."),
        CompletionAction::StopStalled
    );
}

#[test]
fn assessment_parser_requires_state_and_non_empty_reason() {
    assert_eq!(
        parse_assessment(r#"{"state":"incomplete","reason":"work remains"}"#).unwrap(),
        CompletionAssessment::Incomplete
    );
    assert!(parse_assessment(r#"{"state":"incomplete","reason":""}"#).is_err());
    assert!(parse_assessment(r#"{"decision":"incomplete"}"#).is_err());
    assert!(parse_assessment("incomplete").is_err());
}

#[test]
fn assessment_prompt_bounds_inputs() {
    let prompt = assessment_prompt(&"x".repeat(5_000), &"y".repeat(7_000));
    let ResponseItem::Message { content, .. } = &prompt.input[0] else {
        panic!("expected a message");
    };
    let ContentItem::InputText { text } = &content[0] else {
        panic!("expected input text");
    };
    assert!(text.chars().count() < 10_200);
    assert!(prompt.output_schema.is_some());
}
