use std::path::PathBuf;

use crossterm::event::KeyEventState;
use insta::assert_snapshot;
use pretty_assertions::assert_eq;
use serde_json::json;
use tokio::sync::oneshot;

use super::*;
use crate::claude_panes::approval::ApprovalResponder;
use crate::claude_panes::approval::details;

fn request_for(input: serde_json::Value) -> (ClaudeApprovalRequest, oneshot::Receiver<bool>) {
    let (responder, rx) = ApprovalResponder::new();
    let request = ClaudeApprovalRequest {
        pane_id: "claude-1234abcd-5678-4000-8000-000000000000".to_string(),
        pane_title: "Claude Code - GLM 5.2 Z.AI".to_string(),
        cwd: PathBuf::from("/work/repo"),
        tool_name: "Bash".to_string(),
        tool_use_id: Some("toolu_01".to_string()),
        details: details("Bash", &input, str::to_string),
        responder,
    };
    (request, rx)
}

/// A view drawn once at width 80 (so short details count as seen), with the
/// time it first showed.
fn view_for(input: serde_json::Value) -> (ClaudeApprovalView, oneshot::Receiver<bool>, Instant) {
    let (request, rx) = request_for(input);
    let view = ClaudeApprovalView::new(request, /*typed_at*/ None);
    render(&view, 80);
    let shown_at = view.shown_at.get().expect("drawn");
    (view, rx, shown_at)
}

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn repeat(code: KeyCode) -> KeyEvent {
    KeyEvent {
        code,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Repeat,
        state: KeyEventState::NONE,
    }
}

fn render_in(view: &ClaudeApprovalView, width: u16, height: u16) -> String {
    let area = Rect::new(0, 0, width, height);
    let mut buf = Buffer::empty(area);
    view.render(area, &mut buf);
    (0..height)
        .map(|y| {
            (0..width)
                .map(|x| buf[(x, y)].symbol())
                .collect::<String>()
                .trim_end()
                .to_string()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn render(view: &ClaudeApprovalView, width: u16) -> String {
    render_in(view, width, view.desired_height(width))
}

/// Review finding 1: an Enter right after the popup opens does nothing, and
/// once keys work Enter on the untouched popup denies.
#[test]
fn enter_on_a_fresh_popup_never_allows() {
    let (mut view, mut rx, shown_at) = view_for(json!({ "command": "touch x" }));
    view.handle_key_at(key(KeyCode::Enter), shown_at + INPUT_GUARD / 2);
    assert!(!view.is_complete());
    assert!(rx.try_recv().is_err());

    // The ignored Enter restarted the guard.
    let quiet = shown_at + INPUT_GUARD / 2 + INPUT_GUARD;
    view.handle_key_at(key(KeyCode::Enter), quiet);
    assert!(view.is_complete());
    assert_eq!(rx.try_recv(), Ok(false));
}

#[test]
fn a_view_never_drawn_ignores_keys() {
    let (request, mut rx) = request_for(json!({ "command": "touch x" }));
    let mut view = ClaudeApprovalView::new(request, /*typed_at*/ None);
    view.handle_key_at(key(KeyCode::Enter), Instant::now() + INPUT_GUARD * 10);
    assert!(!view.is_complete());
    assert!(rx.try_recv().is_err());
}

/// Review round 2, finding 1: keys typed while the popup is guarded keep it
/// guarded, the composer's last keystroke counts, and Tab never chooses.
#[test]
fn continued_typing_never_allows() {
    let (mut view, mut rx, shown_at) = view_for(json!({ "command": "touch x" }));
    // Typing Right/Enter steadily, 400 ms apart, across the guard boundary.
    let mut now = shown_at;
    for _ in 0..10 {
        view.handle_key_at(key(KeyCode::Right), now);
        view.handle_key_at(key(KeyCode::Enter), now);
        now += Duration::from_millis(400);
    }
    assert!(!view.is_complete());
    assert!(rx.try_recv().is_err());

    let (mut view, _rx, shown_at) = view_for(json!({ "command": "touch x" }));
    let later = shown_at + INPUT_GUARD;
    view.handle_key_at(key(KeyCode::Tab), later);
    assert_eq!(view.choice, Choice::Deny);

    // Typed in the composer just before the popup: guarded until quiet.
    let (request, _rx) = request_for(json!({ "command": "touch x" }));
    let typed_at = Instant::now() + INPUT_GUARD * 2;
    let mut view = ClaudeApprovalView::new(request, Some(typed_at));
    render(&view, 80);
    view.handle_key_at(key(KeyCode::Right), typed_at + INPUT_GUARD / 2);
    assert_eq!(view.choice, Choice::Deny);
}

#[test]
fn allowing_takes_a_choice_then_enter() {
    let (mut view, mut rx, shown_at) = view_for(json!({ "command": "touch x" }));
    let later = shown_at + INPUT_GUARD;
    view.handle_key_at(key(KeyCode::Right), later);
    // A held (repeated) Enter never confirms.
    view.handle_key_at(repeat(KeyCode::Enter), later);
    assert!(!view.is_complete());
    view.handle_key_at(key(KeyCode::Enter), later);
    assert!(view.is_complete());
    assert_eq!(rx.try_recv(), Ok(true));
}

#[test]
fn escape_and_ctrl_c_deny_even_during_the_guard() {
    let (mut view, mut rx, shown_at) = view_for(json!({ "command": "touch x" }));
    view.handle_key_at(key(KeyCode::Esc), shown_at);
    assert_eq!(rx.try_recv(), Ok(false));

    let (mut view, mut rx, _) = view_for(json!({ "command": "touch x" }));
    assert_eq!(view.on_ctrl_c(), CancellationEvent::Handled);
    assert!(view.is_complete());
    assert_eq!(rx.try_recv(), Ok(false));
}

#[tokio::test]
async fn a_dropped_popup_denies() {
    let (view, rx, _) = view_for(json!({ "command": "touch x" }));
    drop(view);
    // The waiting turn reads a closed channel as a denial.
    assert!(rx.await.is_err());
}

/// Review round 2, finding 2: requests queue instead of covering each other,
/// and each starts on Deny behind a fresh guard.
#[test]
fn requests_queue_and_each_starts_on_deny_with_a_fresh_guard() {
    let (mut view, mut first_rx, shown_at) = view_for(json!({ "command": "touch first" }));
    let (second, mut second_rx) = request_for(json!({ "command": "touch second" }));
    assert!(view.try_consume_claude_approval(second).is_none());
    assert!(render(&view, 80).contains("1 more requests waiting"));

    let later = shown_at + INPUT_GUARD;
    view.handle_key_at(key(KeyCode::Right), later);
    view.handle_key_at(key(KeyCode::Enter), later);
    assert_eq!(first_rx.try_recv(), Ok(true));
    assert!(!view.is_complete());
    // A quick second Enter lands on a request not drawn yet: ignored.
    view.handle_key_at(key(KeyCode::Enter), later);
    assert!(second_rx.try_recv().is_err());
    assert_eq!(view.choice, Choice::Deny);
    let screen = render(&view, 80);
    assert!(screen.contains("touch second"), "{screen}");
    let shown_again = view.shown_at.get().expect("drawn");
    view.handle_key_at(key(KeyCode::Enter), shown_again + INPUT_GUARD);
    assert_eq!(second_rx.try_recv(), Ok(false));
    assert!(view.is_complete());
}

#[test]
fn coming_back_from_under_another_view_starts_over() {
    let (mut view, _rx, shown_at) = view_for(json!({ "command": "touch x" }));
    view.handle_key_at(key(KeyCode::Right), shown_at + INPUT_GUARD);
    assert_eq!(view.choice, Choice::Allow);
    view.on_uncovered();
    assert_eq!(view.choice, Choice::Deny);
    assert_eq!(view.shown_at.get(), None);
}

#[test]
fn settled_requests_leave_the_queue() {
    let (mut view, rx, shown_at) = view_for(json!({ "command": "touch first" }));
    let (second, mut second_rx) = request_for(json!({ "command": "touch second" }));
    view.enqueue(second);
    assert!(!view.remove_settled_requests());
    drop(rx);
    // The front request stopped waiting: the next one shows, afresh.
    assert!(!view.remove_settled_requests());
    assert!(render(&view, 80).contains("touch second"));
    let shown_again = view.shown_at.get().expect("drawn");
    assert!(shown_again >= shown_at);
    assert!(second_rx.try_recv().is_err());
    let (mut view, rx, _) = view_for(json!({ "command": "touch x" }));
    drop(rx);
    assert!(view.remove_settled_requests());
    // Keys do nothing on a request nobody waits for.
    view.handle_key_at(key(KeyCode::Enter), Instant::now() + INPUT_GUARD * 10);
    assert!(!view.is_complete());
}

/// Review finding 2: a long multi-line command is shown in full, wrapped,
/// with its line breaks visible, and scrolls to its end; Allow works only
/// once the end was on screen.
#[test]
fn long_commands_are_fully_visible_and_allow_needs_the_end_seen() {
    let long_line = format!("echo {} && rm -rf ~/important", "a".repeat(150));
    let (view, _rx, _) = view_for(json!({
        "command": format!("ls\n{long_line}"),
        "run_in_background": true,
    }));
    let screen = render(&view, 60);
    let joined: String = screen.split_whitespace().collect();
    assert!(joined.contains("ls⏎"), "{screen}");
    assert!(joined.contains("rm-rf~/important"), "{screen}");
    assert!(joined.contains("run_in_background:true"), "{screen}");

    let lines = (0..40)
        .map(|n| format!("step {n}"))
        .collect::<Vec<_>>()
        .join("\n");
    let (mut view, mut rx, shown_at) = view_for(json!({ "command": lines }));
    let later = shown_at + INPUT_GUARD;
    let screen = render(&view, 60);
    assert!(
        screen.contains("step 0") && !screen.contains("step 39"),
        "{screen}"
    );
    assert!(screen.contains("lines 1-16 of 41"), "{screen}");
    assert!(screen.contains("Scroll to the end to allow it"), "{screen}");
    view.handle_key_at(key(KeyCode::Right), later);
    assert_eq!(view.choice, Choice::Deny);
    view.handle_key_at(key(KeyCode::End), later);
    let screen = render(&view, 60);
    assert!(
        screen.contains("step 39") && !screen.contains("step 0⏎"),
        "{screen}"
    );
    view.handle_key_at(key(KeyCode::Right), later);
    view.handle_key_at(key(KeyCode::Enter), later);
    assert_eq!(rx.try_recv(), Ok(true));
}

#[test]
fn a_popup_with_no_room_for_details_cannot_allow() {
    let (request, mut rx) = request_for(json!({ "command": "touch x" }));
    let mut view = ClaudeApprovalView::new(request, /*typed_at*/ None);
    // Only the header and options fit.
    render_in(&view, 80, 9);
    assert!(!view.seen_end.get());
    let later = view.shown_at.get().expect("drawn") + INPUT_GUARD;
    view.handle_key_at(key(KeyCode::Right), later);
    view.handle_key_at(key(KeyCode::Enter), later);
    assert_eq!(rx.try_recv(), Ok(false));
}

#[test]
fn a_request_too_long_to_show_can_only_be_denied() {
    let command = "x".repeat(crate::claude_panes::approval::DETAIL_MAX_CHARS + 10);
    let (mut view, mut rx, shown_at) = view_for(json!({ "command": command }));
    let screen = render(&view, 80);
    assert!(screen.contains("more characters are not shown"), "{screen}");
    assert!(!screen.contains("Allow once"), "{screen}");
    let later = shown_at + INPUT_GUARD;
    view.handle_key_at(key(KeyCode::End), later);
    render(&view, 80);
    view.handle_key_at(key(KeyCode::Right), later);
    view.handle_key_at(key(KeyCode::Enter), later);
    assert_eq!(rx.try_recv(), Ok(false));
}

#[test]
fn popup_snapshot() {
    let (view, _rx, _) = view_for(json!({
        "command": "ls \u{202e}gpj.sh\nrm -rf .",
        "description": "List files",
        "dangerouslyDisableSandbox": true,
    }));
    // The guard is over: the hint shows how to answer.
    view.shown_at.set(Some(Instant::now() - INPUT_GUARD));
    assert_snapshot!(render(&view, 72));
}
