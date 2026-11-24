//! #2407 item 2: Ctrl+T (transcript) and Ctrl+G (external editor) are handled
//! above ChatWidget, so they skipped the held-Enter cancellation every other
//! Ctrl/Alt key gets. Each case arms a held burst Enter on a controlled clock,
//! presses the real shortcut through `App::handle_key_event`, then asks the
//! composer whether a submission would still fire after the lookahead.

use super::*;
use crate::bottom_pane::InputResult;
use pretty_assertions::assert_eq;
use std::time::Duration;
use std::time::Instant;

/// Long enough to count as a single-line burst whose Enter is held.
const BURST: &str = "0123456789abcdef ask about the transcript";

fn ctrl(c: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL)
}

/// Well past any held Enter's deadline.
fn after_lookahead(now: Instant) -> Instant {
    now + Duration::from_secs(1)
}

#[tokio::test]
async fn held_burst_enter_submits_without_a_shortcut_control() {
    let mut app = make_test_app().await;
    let now = Instant::now();
    app.chat_widget.arm_burst_enter_for_tests(BURST, now);
    assert_eq!(
        app.chat_widget
            .take_burst_submission_for_tests(after_lookahead(now)),
        Some(InputResult::Submitted {
            text: BURST.to_string(),
            text_elements: Vec::new(),
        }),
        "control: an untouched held Enter submits at its deadline"
    );
}

#[tokio::test]
async fn ctrl_t_transcript_cancels_held_enter_and_keeps_the_draft() -> Result<()> {
    let mut app = make_test_app().await;
    let config = app.chat_widget.config_ref().clone();
    let mut app_server = Box::pin(crate::start_embedded_app_server_for_picker(&config)).await?;
    let mut tui = crate::tui::test_support::make_test_tui()?;
    assert!(app.keymap.app.open_transcript.is_pressed(ctrl('t')));

    let now = Instant::now();
    app.chat_widget.arm_burst_enter_for_tests(BURST, now);
    Box::pin(app.handle_key_event(&mut tui, &mut app_server, ctrl('t'))).await;

    assert!(app.overlay.is_some(), "Ctrl+T opened the transcript");
    assert!(
        !app.chat_widget.has_pending_burst_enter_for_tests(),
        "no submission may stay armed behind the overlay"
    );
    assert_eq!(app.chat_widget.composer_text_with_pending(), BURST);
    assert_eq!(
        app.chat_widget
            .take_burst_submission_for_tests(after_lookahead(now)),
        None,
        "closing the overlay later must not submit the draft"
    );
    Ok(())
}

#[tokio::test]
async fn ctrl_g_editor_seeds_from_held_burst_and_cancels_its_enter() -> Result<()> {
    let mut app = make_test_app().await;
    let config = app.chat_widget.config_ref().clone();
    let mut app_server = Box::pin(crate::start_embedded_app_server_for_picker(&config)).await?;
    let mut tui = crate::tui::test_support::make_test_tui()?;
    assert!(app.keymap.app.open_external_editor.is_pressed(ctrl('g')));

    let now = Instant::now();
    app.chat_widget.arm_burst_enter_for_tests(BURST, now);
    Box::pin(app.handle_key_event(&mut tui, &mut app_server, ctrl('g'))).await;

    assert_eq!(
        app.chat_widget.external_editor_state(),
        ExternalEditorState::Requested
    );
    // `launch_external_editor` seeds the editor from exactly this text.
    assert_eq!(app.chat_widget.composer_text_with_pending(), BURST);
    assert!(!app.chat_widget.has_pending_burst_enter_for_tests());
    assert_eq!(
        app.chat_widget
            .take_burst_submission_for_tests(after_lookahead(now)),
        None,
        "the editor round-trip must not race a delayed submission"
    );
    Ok(())
}

#[tokio::test]
async fn ctrl_t_with_ordinary_draft_leaves_it_unchanged() -> Result<()> {
    let mut app = make_test_app().await;
    let config = app.chat_widget.config_ref().clone();
    let mut app_server = Box::pin(crate::start_embedded_app_server_for_picker(&config)).await?;
    let mut tui = crate::tui::test_support::make_test_tui()?;

    app.chat_widget.insert_str("typed by hand");
    Box::pin(app.handle_key_event(&mut tui, &mut app_server, ctrl('t'))).await;

    assert!(app.overlay.is_some());
    assert_eq!(
        app.chat_widget.composer_text_with_pending(),
        "typed by hand"
    );
    assert!(!app.chat_widget.has_pending_burst_enter_for_tests());
    Ok(())
}
