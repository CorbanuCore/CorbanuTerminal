use super::*;
use crate::bottom_pane::paste_burst::BURST_ENTER_LOOKAHEAD;
use pretty_assertions::assert_eq;

fn composer() -> ChatComposer {
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    ChatComposer::new(
        /*has_input_focus*/ true,
        AppEventSender::new(tx),
        /*enhanced_keys_supported*/ false,
        String::new(),
        /*disable_paste_burst*/ false,
    )
}

fn type_burst(composer: &mut ChatComposer, text: &str, now: &mut Instant) {
    for ch in text.chars() {
        let result = composer.handle_input_basic_with_time(
            KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE),
            *now,
        );
        assert_eq!(result.0, InputResult::None);
        *now += Duration::from_millis(1);
    }
}

fn submitted(text: &str) -> InputResult {
    InputResult::Submitted {
        text: text.to_string(),
        text_elements: Vec::new(),
    }
}

#[test]
fn continuation_before_deadline_preserves_newline_across_flush_ticks() {
    for delay in [1, 8, 16, 31] {
        let mut composer = composer();
        let mut now = Instant::now();
        type_burst(&mut composer, "0123456789abcdef", &mut now);
        assert_eq!(
            composer
                .handle_submission_with_time(/*should_queue*/ false, now)
                .0,
            InputResult::None
        );
        now += Duration::from_millis(delay);
        assert!(composer.take_burst_submission(now).is_none());
        assert!(!composer.handle_paste_burst_flush(now));
        type_burst(&mut composer, "日本語 second line", &mut now);
        now += Duration::from_millis(200);
        assert!(composer.take_burst_submission(now).is_none());
        assert!(composer.handle_paste_burst_flush(now));
        assert_eq!(
            composer.current_text(),
            "0123456789abcdef\n日本語 second line"
        );
        assert_eq!(
            composer
                .handle_submission_with_time(/*should_queue*/ false, now)
                .0,
            submitted("0123456789abcdef\n日本語 second line")
        );
    }
}

#[test]
fn quiet_enter_submits_once_at_deadline_and_late_input_starts_new_draft() {
    let mut composer = composer();
    let mut now = Instant::now();
    type_burst(&mut composer, "0123456789abcdef", &mut now);
    assert_eq!(
        composer
            .handle_submission_with_time(/*should_queue*/ false, now)
            .0,
        InputResult::None
    );
    assert!(!composer.handle_paste_burst_flush(now + BURST_ENTER_LOOKAHEAD));
    assert_eq!(
        composer.take_burst_submission(now + BURST_ENTER_LOOKAHEAD),
        Some(submitted("0123456789abcdef"))
    );
    assert!(
        composer
            .take_burst_submission(now + BURST_ENTER_LOOKAHEAD)
            .is_none()
    );
    now += BURST_ENTER_LOOKAHEAD + Duration::from_millis(1);
    type_burst(&mut composer, "next draft", &mut now);
    composer.handle_paste_burst_flush(now + Duration::from_millis(200));
    assert_eq!(composer.current_text(), "next draft");
}

#[test]
fn short_bursts_keep_existing_newline_rule() {
    let mut composer = composer();
    let mut now = Instant::now();
    type_burst(&mut composer, "0123456789abcde", &mut now);
    assert_eq!(
        composer
            .handle_submission_with_time(/*should_queue*/ false, now)
            .0,
        InputResult::None
    );
    assert!(
        composer
            .take_burst_submission(now + BURST_ENTER_LOOKAHEAD)
            .is_none()
    );
    composer.handle_paste_burst_flush(now + Duration::from_millis(200));
    assert_eq!(composer.current_text(), "0123456789abcde\n");
}

#[test]
fn cancellation_and_replacement_disarm_pending_enter() {
    for key in [KeyCode::Esc, KeyCode::Left, KeyCode::Backspace] {
        let mut composer = composer();
        let mut now = Instant::now();
        type_burst(&mut composer, "0123456789abcdef", &mut now);
        composer.handle_submission_with_time(/*should_queue*/ false, now);
        assert!(
            !composer.is_empty(),
            "held Enter must not look like an empty draft"
        );
        composer.handle_key_event(KeyEvent::new(key, KeyModifiers::NONE));
        assert!(
            composer
                .take_burst_submission(now + Duration::from_secs(1))
                .is_none()
        );
        // Escape can leave the burst buffered until the next UI tick.
        composer.handle_paste_burst_flush(now + Duration::from_secs(1));
        assert_eq!(
            composer.current_text(),
            if key == KeyCode::Backspace {
                "0123456789abcde"
            } else {
                "0123456789abcdef"
            }
        );
    }

    let mut composer = composer();
    let mut now = Instant::now();
    type_burst(&mut composer, "0123456789abcdef", &mut now);
    composer.handle_submission_with_time(/*should_queue*/ false, now);
    composer.set_text_content("replacement".to_string(), Vec::new(), Vec::new());
    assert!(
        composer
            .take_burst_submission(now + Duration::from_secs(1))
            .is_none()
    );
    assert_eq!(composer.current_text(), "replacement");
}

#[test]
fn explicit_paste_and_disabled_detection_submit_without_lookahead() {
    let mut composer = composer();
    composer.handle_paste("0123456789abcdef\r\nsecond line".to_string());
    assert_eq!(
        composer
            .handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE))
            .0,
        submitted("0123456789abcdef\nsecond line")
    );

    composer.set_disable_paste_burst(/*disabled*/ true);
    let mut now = Instant::now();
    type_burst(&mut composer, "0123456789abcdef", &mut now);
    assert_eq!(
        composer
            .handle_submission_with_time(/*should_queue*/ false, now)
            .0,
        submitted("0123456789abcdef")
    );
}

/// Sync: main's `tab_keeps_immediate_long_burst_submission`. Upstream keeps a
/// Tab during a burst as pasted text (`handle_paste_tab`), so on this branch it
/// neither submits nor arms a delayed submission.
#[test]
fn tab_during_long_burst_arms_no_delayed_submission() {
    let mut composer = composer();
    let mut now = Instant::now();
    type_burst(&mut composer, "0123456789abcdef", &mut now);
    assert_eq!(
        composer
            .handle_key_event(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE))
            .0,
        InputResult::None
    );
    assert!(
        composer
            .take_burst_submission(now + Duration::from_secs(1))
            .is_none()
    );
}

#[test]
fn bracketed_paste_during_lookahead_preserves_held_text() {
    let mut composer = composer();
    let mut now = Instant::now();
    type_burst(&mut composer, "0123456789abcdef", &mut now);
    composer.handle_submission_with_time(/*should_queue*/ false, now);
    composer.handle_paste("second line\r\nlast line".to_string());
    assert_eq!(
        composer.current_text(),
        "0123456789abcdef\nsecond line\nlast line"
    );
    assert!(
        composer
            .take_burst_submission(now + Duration::from_secs(1))
            .is_none()
    );
}

#[test]
fn multiline_keystroke_paste_snapshot() {
    super::tests::snapshot_composer_state_with_width(
        "multiline_keystroke_paste",
        /*width*/ 60,
        /*enhanced_keys_supported*/ false,
        |composer| {
            let mut now = Instant::now();
            type_burst(
                composer,
                "A first line with at least sixteen characters",
                &mut now,
            );
            composer.handle_submission_with_time(/*should_queue*/ false, now);
            now += Duration::from_millis(31);
            type_burst(composer, "second line 日本語", &mut now);
            composer.handle_paste_burst_flush(now + Duration::from_millis(200));
        },
    );
}

#[test]
fn long_multiline_keystroke_paste_stays_one_draft() {
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    let mut composer = ChatComposer::new(
        /*has_input_focus*/ true,
        AppEventSender::new(tx),
        /*enhanced_keys_supported*/ false,
        String::new(),
        /*disable_paste_burst*/ false,
    );
    let mut now = Instant::now();
    let payload = "A first line with at least sixteen characters\n\nsecond line 日本語\nlast line";
    for ch in payload.chars() {
        let result = if ch == '\n' {
            composer.handle_submission_with_time(/*should_queue*/ false, now)
        } else {
            composer.handle_input_basic_with_time(
                KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE),
                now,
            )
        };
        assert_eq!(
            result.0,
            InputResult::None,
            "paste must not submit a partial line"
        );
        now += Duration::from_millis(1);
    }
    composer.handle_paste_burst_flush(now + Duration::from_millis(200));
    assert_eq!(composer.current_text(), payload);
}
