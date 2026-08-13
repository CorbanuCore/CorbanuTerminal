use codex_app_server_protocol::ThreadTokenUsage;
use codex_app_server_protocol::TokenUsageBreakdown;

use super::RuntimeState;
use super::runtime_state;
use super::thousands;
use super::token_usage_text;

fn breakdown(total: i64, input: i64, output: i64) -> TokenUsageBreakdown {
    TokenUsageBreakdown {
        total_tokens: total,
        input_tokens: input,
        cached_input_tokens: 0,
        cache_write_input_tokens: 0,
        output_tokens: output,
        reasoning_output_tokens: 0,
    }
}

#[test]
fn status_priority_preserves_actionable_state() {
    assert_eq!(
        runtime_state(
            /*approvals*/ 1, /*active_turn*/ true, /*queued*/ 1,
            /*has_error*/ true
        ),
        RuntimeState::AwaitingApproval
    );
    assert_eq!(
        runtime_state(
            /*approvals*/ 0, /*active_turn*/ true, /*queued*/ 1,
            /*has_error*/ true
        ),
        RuntimeState::WorkingQueued
    );
    assert_eq!(
        runtime_state(
            /*approvals*/ 0, /*active_turn*/ true, /*queued*/ 0,
            /*has_error*/ true
        ),
        RuntimeState::Working
    );
    assert_eq!(
        runtime_state(
            /*approvals*/ 0, /*active_turn*/ false, /*queued*/ 1,
            /*has_error*/ true
        ),
        RuntimeState::Blocked
    );
    assert_eq!(
        runtime_state(
            /*approvals*/ 0, /*active_turn*/ false, /*queued*/ 1,
            /*has_error*/ false
        ),
        RuntimeState::Recovering
    );
    assert_eq!(
        runtime_state(
            /*approvals*/ 0, /*active_turn*/ false, /*queued*/ 0,
            /*has_error*/ true
        ),
        RuntimeState::Blocked
    );
    assert_eq!(
        runtime_state(
            /*approvals*/ 0, /*active_turn*/ false, /*queued*/ 0,
            /*has_error*/ false
        ),
        RuntimeState::Idle
    );
}

#[test]
fn every_status_has_a_concrete_next_action() {
    for state in [
        RuntimeState::Idle,
        RuntimeState::Working,
        RuntimeState::WorkingQueued,
        RuntimeState::AwaitingApproval,
        RuntimeState::Recovering,
        RuntimeState::Blocked,
    ] {
        assert!(!state.next_action().is_empty());
    }
}

#[test]
fn thousands_groups_every_magnitude() {
    assert_eq!(thousands(/*value*/ 0), "0");
    assert_eq!(thousands(/*value*/ 999), "999");
    assert_eq!(thousands(/*value*/ 1_000), "1,000");
    assert_eq!(thousands(/*value*/ 178_010), "178,010");
    assert_eq!(thousands(/*value*/ 683_203_904), "683,203,904");
}

/// The reported thread outgrew every signal the chat surfaced: it ran for two
/// weeks on a single thread and its cost was invisible because `/status` showed
/// state and model only. Both numbers must appear, and the cumulative one is the
/// point -- window occupancy alone stays flat while the bill does not.
#[test]
fn status_reports_window_occupancy_and_the_unbounded_thread_total() {
    let usage = ThreadTokenUsage {
        total: breakdown(
            /*total*/ 684_885_374,
            /*input*/ 683_203_904,
            /*output*/ 1_681_470,
        ),
        last: breakdown(
            /*total*/ 178_010, /*input*/ 177_000, /*output*/ 1_010,
        ),
        model_context_window: Some(353_400),
    };

    let text = token_usage_text(&usage);

    assert!(
        text.contains("Context: 178,010 of 353,400 tokens (50%)"),
        "{text}"
    );
    assert!(
        text.contains("Thread total: 684,885,374 tokens (683,203,904 in, 1,681,470 out)"),
        "{text}"
    );
    assert!(text.contains("/new"), "{text}");
}

#[test]
fn status_omits_the_percentage_when_the_window_is_unknown() {
    let usage = ThreadTokenUsage {
        total: breakdown(
            /*total*/ 1_500, /*input*/ 1_200, /*output*/ 300,
        ),
        last: breakdown(
            /*total*/ 1_500, /*input*/ 1_200, /*output*/ 300,
        ),
        model_context_window: None,
    };

    let text = token_usage_text(&usage);

    assert!(text.contains("Context: 1,500 tokens"), "{text}");
    assert!(!text.contains('%'), "{text}");
}

/// A window reported smaller than current usage must not render above 100%.
#[test]
fn status_clamps_an_overfull_window() {
    let usage = ThreadTokenUsage {
        total: breakdown(
            /*total*/ 400_000, /*input*/ 399_000, /*output*/ 1_000,
        ),
        last: breakdown(
            /*total*/ 400_000, /*input*/ 399_000, /*output*/ 1_000,
        ),
        model_context_window: Some(353_400),
    };

    assert!(token_usage_text(&usage).contains("(100%)"));
}
