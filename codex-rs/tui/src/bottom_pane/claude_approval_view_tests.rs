use std::path::PathBuf;

use crossterm::event::KeyEventState;
use insta::assert_snapshot;
use pretty_assertions::assert_eq;
use serde_json::json;
use tokio::sync::oneshot;

use super::*;
use crate::claude_panes::approval::ApprovalResponder;
use crate::claude_panes::approval::details;

fn view_for(input: serde_json::Value) -> (ClaudeApprovalView, oneshot::Receiver<bool>, Instant) {
    let (responder, rx) = ApprovalResponder::new();
    let view = ClaudeApprovalView::new(ClaudeApprovalRequest {
        pane_id: "claude-1234abcd-5678-4000-8000-000000000000".to_string(),
        pane_title: "Claude Code - GLM 5.2 Z.AI".to_string(),
        cwd: PathBuf::from("/work/repo"),
        tool_name: "Bash".to_string(),
        tool_use_id: Some("toolu_01".to_string()),
        details: details("Bash", &input, str::to_string),
        responder,
    });
    let shown_at = Instant::now();
    view.shown_at.set(Some(shown_at));
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

fn render(view: &ClaudeApprovalView, width: u16) -> String {
    let height = view.desired_height(width);
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

/// Review finding 1: an Enter right after the popup opens does nothing, and
/// once keys work Enter on the untouched popup denies.
#[test]
fn enter_on_a_fresh_popup_never_allows() {
    let (mut view, mut rx, shown_at) = view_for(json!({ "command": "touch x" }));
    view.handle_key_at(key(KeyCode::Right), shown_at);
    view.handle_key_at(key(KeyCode::Enter), shown_at + INPUT_GUARD / 2);
    assert!(!view.is_complete());
    assert!(rx.try_recv().is_err());

    view.handle_key_at(key(KeyCode::Enter), shown_at + INPUT_GUARD);
    assert_eq!(view.completion(), Some(ViewCompletion::Cancelled));
    assert_eq!(rx.try_recv(), Ok(false));
}

#[test]
fn a_view_never_drawn_ignores_keys() {
    let (mut view, mut rx, _) = view_for(json!({ "command": "touch x" }));
    view.shown_at.set(None);
    view.handle_key_at(key(KeyCode::Enter), Instant::now() + INPUT_GUARD * 10);
    assert!(!view.is_complete());
    assert!(rx.try_recv().is_err());
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
    assert_eq!(view.completion(), Some(ViewCompletion::Accepted));
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

#[test]
fn a_request_settled_elsewhere_is_removed_and_ignores_keys() {
    let (mut view, rx, shown_at) = view_for(json!({ "command": "touch x" }));
    assert!(!view.is_settled_elsewhere());
    drop(rx);
    assert!(view.is_settled_elsewhere());
    view.handle_key_at(key(KeyCode::Right), shown_at + INPUT_GUARD);
    view.handle_key_at(key(KeyCode::Enter), shown_at + INPUT_GUARD);
    assert!(!view.is_complete());
}

/// Review finding 2: a long multi-line command is shown in full, wrapped,
/// with its line breaks visible, and scrolls to its end.
#[test]
fn long_multi_line_commands_are_fully_visible() {
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
    let (mut view, _rx, shown_at) = view_for(json!({ "command": lines }));
    let screen = render(&view, 60);
    assert!(
        screen.contains("step 0") && !screen.contains("step 39"),
        "{screen}"
    );
    assert!(screen.contains("lines 1-16 of 41"), "{screen}");
    view.handle_key_at(key(KeyCode::End), shown_at + INPUT_GUARD);
    let screen = render(&view, 60);
    assert!(
        screen.contains("step 39") && !screen.contains("step 0⏎"),
        "{screen}"
    );
}

#[test]
fn a_request_too_long_to_show_can_only_be_denied() {
    let command = "x".repeat(crate::claude_panes::approval::DETAIL_MAX_CHARS + 10);
    let (mut view, mut rx, shown_at) = view_for(json!({ "command": command }));
    let screen = render(&view, 80);
    assert!(
        screen.contains("10 more characters are not shown"),
        "{screen}"
    );
    assert!(!screen.contains("Allow once"), "{screen}");
    let later = shown_at + INPUT_GUARD;
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
