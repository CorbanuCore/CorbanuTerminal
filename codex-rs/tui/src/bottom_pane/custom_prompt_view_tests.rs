use super::*;
use crate::key_hint;
use crate::keymap::KeymapContext;
use crate::keymap::RuntimeKeymap;
use pretty_assertions::assert_eq;
use std::sync::mpsc::Receiver;

#[test]
fn picker_prompt_keeps_multiline_cursor_and_escape_hint_visible_after_resize() {
    let (mut view, submitted_rx) = custom_prompt_view();
    view.enable_vim_in_insert_mode();
    view.handle_paste("Check the parser and preserve 日本語.\n".repeat(/*n*/ 12));
    for (width, height) in [(80, 16), (40, 8), (80, 16), (12, 3)] {
        let area = Rect::new(/*x*/ 0, /*y*/ 0, width, height);
        let mut buf = Buffer::empty(area);
        view.render(area, &mut buf);
        let (x, y) = view.cursor_pos(area).expect("editable row remains visible");
        assert!(
            x >= 2 && x < width - 2 && y < height - 1,
            "cursor {x},{y} in {area:?}"
        );
        if width >= 40 {
            let rows = (0..height)
                .map(|y| {
                    (0..width)
                        .map(|x| buf[(x, y)].symbol())
                        .collect::<String>()
                        .trim_end()
                        .to_owned()
                })
                .collect::<Vec<_>>();
            assert!(rows.last().unwrap().contains("esc normal mode"));
            insta::assert_snapshot!(
                format!("picker_prompt_multiline_{width}x{height}"),
                rows.join("\n")
            );
        }
    }
    view.handle_key_event(KeyEvent::from(KeyCode::Esc));
    assert!(!view.is_complete(), "first Escape enters Vim normal mode");
    view.handle_key_event(KeyEvent::from(KeyCode::Esc));
    assert_eq!(view.completion(), Some(ViewCompletion::Cancelled));
    assert!(submitted_rx.try_recv().is_err());
}

#[test]
fn paste_burst_newline_does_not_submit_short_first_line() {
    let now = Instant::now();

    for (first_line, second_line) in [("x", "rest"), ("id", "body"), ("foo", "bar")] {
        let (mut view, submitted_rx) = custom_prompt_view();
        let mut ms = 0;

        for ch in first_line.chars() {
            view.handle_key_event_at(KeyEvent::from(KeyCode::Char(ch)), now + elapsed(ms));
            ms += 1;
        }
        view.handle_key_event_at(KeyEvent::from(KeyCode::Enter), now + elapsed(ms));
        ms += 1;
        for ch in second_line.chars() {
            view.handle_key_event_at(KeyEvent::from(KeyCode::Char(ch)), now + elapsed(ms));
            ms += 1;
        }

        assert!(submitted_rx.try_recv().is_err());
        assert!(!view.is_complete());

        view.handle_key_event_at(KeyEvent::from(KeyCode::Enter), now + elapsed(/*ms*/ 200));

        assert_eq!(
            submitted_rx.try_recv(),
            Ok(format!("{first_line}\n{second_line}"))
        );
        assert!(view.is_complete());
    }
}

#[test]
fn paste_burst_newline_after_tab_does_not_submit() {
    let (mut view, submitted_rx) = custom_prompt_view();
    let now = Instant::now();
    let mut ms = 0;

    view.handle_key_event_at(KeyEvent::from(KeyCode::Char('x')), now + elapsed(ms));
    ms += 1;
    view.handle_key_event_at(KeyEvent::from(KeyCode::Tab), now + elapsed(ms));
    ms += 1;
    view.handle_key_event_at(KeyEvent::from(KeyCode::Enter), now + elapsed(ms));
    ms += 1;
    for ch in "rest".chars() {
        view.handle_key_event_at(KeyEvent::from(KeyCode::Char(ch)), now + elapsed(ms));
        ms += 1;
    }

    assert!(submitted_rx.try_recv().is_err());
    assert!(!view.is_complete());

    view.handle_key_event_at(KeyEvent::from(KeyCode::Enter), now + elapsed(/*ms*/ 200));

    assert_eq!(submitted_rx.try_recv(), Ok("x\nrest".to_string()));
    assert!(view.is_complete());
}

#[test]
fn delayed_enter_after_typing_submits() {
    let (mut view, submitted_rx) = custom_prompt_view();
    let now = Instant::now();

    for (idx, ch) in "foo".chars().enumerate() {
        view.handle_key_event_at(KeyEvent::from(KeyCode::Char(ch)), now + elapsed(idx * 20));
    }
    view.handle_key_event_at(KeyEvent::from(KeyCode::Enter), now + elapsed(/*ms*/ 80));

    assert_eq!(submitted_rx.try_recv(), Ok("foo".to_string()));
    assert!(view.is_complete());
}

#[test]
<<<<<<< HEAD
fn ctrl_d_submit_mode_uses_enter_for_newline() {
    let (mut view, submitted_rx) = custom_prompt_view_ctrl_d();
    let now = Instant::now();

    for (idx, ch) in "foo".chars().enumerate() {
        view.handle_key_event_at(KeyEvent::from(KeyCode::Char(ch)), now + elapsed(idx * 20));
    }
    view.handle_key_event_at(KeyEvent::from(KeyCode::Enter), now + elapsed(/*ms*/ 100));
    for (idx, ch) in "bar".chars().enumerate() {
        view.handle_key_event_at(
            KeyEvent::from(KeyCode::Char(ch)),
            now + elapsed(120 + idx * 20),
        );
    }

    assert!(submitted_rx.try_recv().is_err());
    assert!(!view.is_complete());

    view.handle_key_event_at(
        KeyEvent::new(KeyCode::Char('d'), KeyModifiers::CONTROL),
        now + elapsed(/*ms*/ 240),
    );

    assert_eq!(submitted_rx.try_recv(), Ok("foo\nbar".to_string()));
    assert!(view.is_complete());
}

/// #1289: in a one-line field, Enter right after fast typing submits; it is
/// never taken into the field as a newline.
#[test]
fn single_line_mode_enter_inside_paste_burst_window_submits() {
    let (mut view, submitted_rx, refused_rx) = single_line_view();
    let now = Instant::now();
    let mut ms = 0;

    for ch in "work".chars() {
        view.handle_key_event_at(KeyEvent::from(KeyCode::Char(ch)), now + elapsed(ms));
        ms += 1;
    }
    // Within the burst window, where the multi-line mode inserts a newline.
    view.handle_key_event_at(KeyEvent::from(KeyCode::Enter), now + elapsed(ms));
    assert_eq!(view.textarea.text(), "work");
    assert!(!view.pre_draw_tick(now + elapsed(ms) + SINGLE_LINE_SUBMIT_HOLD / 2));
    assert!(!view.is_complete());
    assert!(view.pre_draw_tick(now + elapsed(ms) + SINGLE_LINE_SUBMIT_HOLD));

    assert_eq!(submitted_rx.try_recv(), Ok("work".to_string()));
    assert!(refused_rx.try_recv().is_err());
    assert!(view.is_complete());
}

/// #1337 review: the editor's newline keys (Ctrl+J, Ctrl+M, a raw line
/// feed, Shift+Enter, Alt+Enter) submit a one-line field like Enter.
#[test]
fn single_line_mode_newline_keys_submit() {
    for key in [
        KeyEvent::new(KeyCode::Char('j'), KeyModifiers::CONTROL),
        KeyEvent::new(KeyCode::Char('m'), KeyModifiers::CONTROL),
        KeyEvent::new(KeyCode::Char('\n'), KeyModifiers::NONE),
        KeyEvent::new(KeyCode::Enter, KeyModifiers::SHIFT),
        KeyEvent::new(KeyCode::Enter, KeyModifiers::ALT),
    ] {
        let (mut view, submitted_rx, refused_rx) = single_line_view();
        let now = Instant::now();
        for (idx, ch) in "work".chars().enumerate() {
            view.handle_key_event_at(KeyEvent::from(KeyCode::Char(ch)), now + elapsed(idx));
        }
        view.handle_key_event_at(key, now + elapsed(/*ms*/ 4));
        assert_eq!(view.textarea.text(), "work", "{key:?}");
        view.pre_draw_tick(now + elapsed(/*ms*/ 4) + SINGLE_LINE_SUBMIT_HOLD);

        assert_eq!(submitted_rx.try_recv(), Ok("work".to_string()), "{key:?}");
        assert!(refused_rx.try_recv().is_err(), "{key:?}");
        assert!(view.is_complete(), "{key:?}");
    }
}

/// #1337 review: input sent together with the submit key (a key after the
/// name) is never shown and never passed on: the field is refused once the
/// input is quiet, and the prompt stays open until then so nothing reaches a
/// view or the composer under it.
#[test]
fn single_line_mode_input_after_submit_key_is_refused_unshown() {
    for submit in [
        KeyEvent::from(KeyCode::Enter),
        KeyEvent::new(KeyCode::Char('j'), KeyModifiers::CONTROL),
    ] {
        let (mut view, submitted_rx, refused_rx) = single_line_view();
        let now = Instant::now();
        let mut ms = 0;
        for ch in "work".chars() {
            view.handle_key_event_at(KeyEvent::from(KeyCode::Char(ch)), now + elapsed(ms));
            ms += 1;
        }
        view.handle_key_event_at(submit, now + elapsed(ms));
        for ch in "sk-fake-0000".chars() {
            ms += 1;
            view.handle_key_event_at(KeyEvent::from(KeyCode::Char(ch)), now + elapsed(ms));
        }
        // The hold runs from the last key, not from the submit key.
        assert!(!view.pre_draw_tick(now + elapsed(/*ms*/ 4) + SINGLE_LINE_SUBMIT_HOLD));
        assert!(!view.is_complete());
        assert_eq!(view.textarea.text(), "work");
        assert!(!render_view(&view).contains("sk-fake"));

        assert!(view.pre_draw_tick(now + elapsed(ms) + SINGLE_LINE_SUBMIT_HOLD));
        assert!(submitted_rx.try_recv().is_err(), "{submit:?}");
        assert_eq!(refused_rx.try_recv(), Ok(()), "{submit:?}");
        assert!(view.is_complete());
    }
}

/// #1337 review (optional item): a paste with more than one line keeps only
/// its first line, so a key-like second line is never shown; the field is
/// then refused. A single trailing line break is just dropped.
#[test]
fn single_line_mode_multi_line_paste_keeps_first_line() {
    let (mut view, submitted_rx, refused_rx) = single_line_view();
    assert!(view.handle_paste("work\nsk-fake-0000".to_string()));
    assert_eq!(view.textarea.text(), "work");
    assert!(!render_view(&view).contains("sk-fake"));
    let now = Instant::now();
    view.handle_key_event_at(KeyEvent::from(KeyCode::Enter), now);
    view.pre_draw_tick(now + SINGLE_LINE_SUBMIT_HOLD);
    assert!(submitted_rx.try_recv().is_err());
    assert_eq!(refused_rx.try_recv(), Ok(()));

    let (mut view, submitted_rx, refused_rx) = single_line_view();
    assert!(view.handle_paste("work\n".to_string()));
    assert_eq!(view.textarea.text(), "work");
    view.handle_key_event_at(KeyEvent::from(KeyCode::Enter), now);
    view.pre_draw_tick(now + SINGLE_LINE_SUBMIT_HOLD);
    assert_eq!(submitted_rx.try_recv(), Ok("work".to_string()));
    assert!(refused_rx.try_recv().is_err());
}

fn single_line_view() -> (CustomPromptView, Receiver<String>, Receiver<()>) {
    let (submitted, submitted_rx) = std::sync::mpsc::channel();
    let (refused, refused_rx) = std::sync::mpsc::channel();
    let view = CustomPromptView::new(
        "Add a Z.AI account".to_string(),
        "Account name".to_string(),
        String::new(),
        /*context_label*/ None,
        Box::new(move |text| {
            submitted.send(text).expect("send submitted text");
        }),
    )
    .with_submit_mode(CustomPromptSubmitMode::EnterSingleLine)
    .with_on_refused(Box::new(move || {
        refused.send(()).expect("send refusal");
    }));
    (view, submitted_rx, refused_rx)
}

fn render_view(view: &CustomPromptView) -> String {
    let width = 60;
    let area = Rect::new(0, 0, width, view.desired_height(width));
    let mut buf = Buffer::empty(area);
    view.render(area, &mut buf);
    (0..area.height)
        .map(|y| {
            (0..area.width)
                .map(|x| buf[(x, y)].symbol().to_string())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
||||||| 413492cd6c
=======
fn vim_insert_escape_clears_paste_burst_before_enter() {
    let (mut view, submitted_rx) = custom_prompt_view();
    let now = Instant::now();
    view.enable_vim_in_insert_mode();

    for (idx, ch) in "abc".chars().enumerate() {
        view.handle_key_event_at(KeyEvent::from(KeyCode::Char(ch)), now + elapsed(idx));
    }
    view.handle_key_event_at(KeyEvent::from(KeyCode::Esc), now + elapsed(/*ms*/ 3));
    view.handle_key_event_at(KeyEvent::from(KeyCode::Enter), now + elapsed(/*ms*/ 4));

    assert_eq!(submitted_rx.try_recv(), Ok("abc".to_string()));
    assert_eq!(view.completion(), Some(ViewCompletion::Accepted));
}

#[test]
fn vim_prompt_hint_tracks_escape_behavior() {
    let (mut view, _submitted_rx) = custom_prompt_view();
    let rendered_hint = |view: &CustomPromptView, width: u16| {
        let area = Rect::new(/*x*/ 0, /*y*/ 0, width, /*height*/ 5);
        let mut buf = Buffer::empty(area);
        view.render(area, &mut buf);
        (0..area.width)
            .map(|x| buf[(x, area.height - 1)].symbol())
            .collect::<String>()
            .trim_end()
            .to_string()
    };
    let vim_color = |view: &CustomPromptView| {
        let area = Rect::new(
            /*x*/ 0, /*y*/ 0, /*width*/ 80, /*height*/ 5,
        );
        let mut buf = Buffer::empty(area);
        view.render(area, &mut buf);
        buf[(67, 4)].style().fg
    };

    let mut hints = vec![rendered_hint(&view, /*width*/ 80)];
    view.enable_vim_in_insert_mode();
    hints.extend([80, 60, 14].map(|width| rendered_hint(&view, width)));
    assert_eq!(vim_color(&view), Some(ratatui::style::Color::Green));

    view.handle_key_event(KeyEvent::from(KeyCode::Esc));
    hints.push(rendered_hint(&view, /*width*/ 80));
    assert_eq!(vim_color(&view), Some(ratatui::style::Color::Magenta));
    view.handle_key_event(KeyEvent::from(KeyCode::Char('R')));
    hints.push(rendered_hint(&view, /*width*/ 80));
    insta::assert_snapshot!(hints.join("\n"));
}

#[test]
fn vim_insert_cursor_tracks_mode_and_normal_mode_commands() {
    let (mut view, _submitted_rx) = custom_prompt_view();
    let area = Rect::new(0, 0, 80, 10);
    view.textarea.set_text_clearing_elements("rename");
    view.textarea.set_cursor("rename".len());
    let mut config = codex_config::types::TuiKeymap::default();
    config.editor.move_left = Some(codex_config::types::KeybindingsSpec::One(
        codex_config::types::KeybindingSpec("f2".to_string()),
    ));
    let mut keymap = RuntimeKeymap::from_config(&config).expect("valid editor remap");
    keymap.vim_normal.delete_char = vec![key_hint::plain(KeyCode::F(3))];
    view.set_keymap_bindings(&keymap);
    view.handle_key_event(KeyEvent::from(KeyCode::F(2)));
    assert_eq!(view.textarea.cursor(), "rename".len() - 1);
    view.textarea.set_cursor("rename".len());

    assert!(matches!(
        view.cursor_style(area),
        crossterm::cursor::SetCursorStyle::DefaultUserShape
    ));

    view.enable_vim_in_insert_mode();
    assert!(view.prefer_esc_to_handle_key_event());
    assert!(view.keymap_contexts().contains(KeymapContext::Editor));
    assert!(matches!(
        view.cursor_style(area),
        crossterm::cursor::SetCursorStyle::SteadyBar
    ));

    view.handle_key_event(KeyEvent::from(KeyCode::Esc));
    assert!(!view.is_complete());
    assert!(!view.prefer_esc_to_handle_key_event());
    assert!(view.keymap_contexts().contains(KeymapContext::VimNormal));
    assert!(matches!(
        view.cursor_style(area),
        crossterm::cursor::SetCursorStyle::DefaultUserShape
    ));

    for key_code in [KeyCode::Enter, KeyCode::Esc] {
        view.handle_key_event(KeyEvent::from(KeyCode::Char('d')));
        assert!(view.prefer_esc_to_handle_key_event());
        assert!(view.keymap_contexts().contains(KeymapContext::VimOperator));
        view.handle_key_event(KeyEvent::from(key_code));
        assert!(!view.is_complete());
        assert!(!view.prefer_esc_to_handle_key_event());
        assert_eq!(view.textarea.text(), "rename");
    }

    view.handle_key_event(KeyEvent::from(KeyCode::Char('d')));
    view.handle_key_event(KeyEvent::from(KeyCode::Char('i')));
    assert!(
        view.keymap_contexts()
            .contains(KeymapContext::VimTextObject)
    );
    view.handle_key_event(KeyEvent::from(KeyCode::Esc));

    view.handle_key_event(KeyEvent::from(KeyCode::Char('h')));
    view.handle_key_event(KeyEvent::from(KeyCode::F(3)));
    assert_eq!(view.textarea.text(), "renae");

    view.handle_key_event(KeyEvent::from(KeyCode::Char('i')));
    assert!(matches!(
        view.cursor_style(area),
        crossterm::cursor::SetCursorStyle::SteadyBar
    ));
    view.handle_key_event(KeyEvent::from(KeyCode::Char('m')));
    assert_eq!(view.textarea.text(), "rename");

    view.handle_key_event(KeyEvent::from(KeyCode::Esc));
    view.handle_key_event(KeyEvent::from(KeyCode::Char('R')));
    assert!(view.prefer_esc_to_handle_key_event());
    view.handle_key_event(KeyEvent::from(KeyCode::Char('x')));
    assert_eq!(view.textarea.text(), "renaxe");
    view.handle_key_event(KeyEvent::from(KeyCode::Esc));
    assert!(!view.is_complete());
    assert!(!view.prefer_esc_to_handle_key_event());
    assert_eq!(view.textarea.text(), "renaxe");
    view.handle_key_event(KeyEvent::from(KeyCode::Esc));
    assert!(view.is_complete());
    assert_eq!(view.completion(), Some(ViewCompletion::Cancelled));
}

#[test]
fn escape_without_vim_cancels_prompt() {
    let (mut view, _submitted_rx) = custom_prompt_view();

    view.handle_key_event(KeyEvent::from(KeyCode::Esc));

    assert!(view.is_complete());
    assert_eq!(view.completion(), Some(ViewCompletion::Cancelled));
}

#[test]
fn background_prefill_requires_a_matching_unedited_prompt() {
    let request_id = Uuid::new_v4();
    let (view, submitted_rx) = custom_prompt_view();
    let mut view = view.with_text_suggestion(request_id, "Loading".into(), "Ready".into());

    assert!(!view.apply_text_suggestion(Uuid::new_v4(), Some("Stale")));
    assert!(view.apply_text_suggestion(request_id, Some("Suggested text")));
    assert_eq!(view.context_label.as_deref(), Some("Ready"));

    view.handle_key_event_at(KeyEvent::from(KeyCode::Enter), Instant::now());

    assert_eq!(submitted_rx.try_recv(), Ok("Suggested text".to_string()));
    assert!(!view.apply_text_suggestion(request_id, Some("Late")));
}

#[test]
fn background_prefill_failure_clears_loading_without_changing_text() {
    let request_id = Uuid::new_v4();
    let (view, _submitted_rx) = custom_prompt_view();
    let mut view = view.with_text_suggestion(request_id, "Loading".into(), "Ready".into());
    view.textarea.set_text_clearing_elements("Existing title");

    assert!(view.apply_text_suggestion(request_id, /*suggestion*/ None));
    assert_eq!(
        (view.textarea.text(), view.context_label.as_deref()),
        ("Existing title", None)
    );
}

#[test]
fn background_prefill_preserves_typing_even_after_the_text_is_deleted() {
    let request_id = Uuid::new_v4();
    let (view, _submitted_rx) = custom_prompt_view();
    let mut view = view.with_text_suggestion(request_id, "Loading".into(), "Ready".into());

    view.handle_key_event_at(KeyEvent::from(KeyCode::Char('x')), Instant::now());
    view.handle_key_event_at(KeyEvent::from(KeyCode::Backspace), Instant::now());

    assert!(view.apply_text_suggestion(request_id, Some("Generated")));
    assert_eq!(
        (view.textarea.text(), view.context_label.as_deref()),
        ("", None)
    );
}

#[test]
fn background_prefill_preserves_pasted_text() {
    let request_id = Uuid::new_v4();
    let (view, _submitted_rx) = custom_prompt_view();
    let mut view = view.with_text_suggestion(request_id, "Loading".into(), "Ready".into());

    assert!(view.handle_paste("Manual title".to_string()));
    assert!(view.apply_text_suggestion(request_id, Some("Generated")));
    assert_eq!(
        (view.textarea.text(), view.context_label.as_deref()),
        ("Manual title", None)
    );
}

#[test]
fn background_prefill_survives_cursor_movement() {
    let request_id = Uuid::new_v4();
    let (view, _submitted_rx) = custom_prompt_view();
    let mut view = view.with_text_suggestion(request_id, "Loading".into(), "Ready".into());
    view.textarea.set_text_clearing_elements("Existing");
    view.textarea.set_cursor("Existing".len());

    view.handle_key_event_at(KeyEvent::from(KeyCode::Left), Instant::now());

    assert!(view.apply_text_suggestion(request_id, Some("Generated")));
    assert_eq!(
        (view.textarea.text(), view.context_label.as_deref()),
        ("Generated", Some("Ready"))
    );
}

#[test]
fn background_prefill_preserves_replace_backspace_recovery() {
    let request_id = Uuid::new_v4();
    let (view, _submitted_rx) = custom_prompt_view();
    let mut view = view.with_text_suggestion(request_id, "Loading".into(), "Ready".into());
    view.enable_vim_in_insert_mode();
    for code in [KeyCode::Esc, KeyCode::Char('R')] {
        view.handle_key_event(KeyEvent::from(code));
    }
    assert!(view.apply_text_suggestion(request_id, Some("abc")));
    for code in [KeyCode::Left, KeyCode::Char('X'), KeyCode::Backspace] {
        view.handle_key_event(KeyEvent::from(code));
    }
    assert_eq!(
        (view.textarea.text(), view.textarea.vim_mode_label()),
        ("abc", Some("Replace"))
    );
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
}

fn custom_prompt_view() -> (CustomPromptView, Receiver<String>) {
    let (submitted, submitted_rx) = std::sync::mpsc::channel();
    let view = CustomPromptView::new(
        "Edit goal".to_string(),
        "Type a goal objective and press Enter".to_string(),
        String::new(),
        /*context_label*/ None,
        Box::new(move |text| {
            submitted.send(text).expect("send submitted text");
        }),
    );
    (view, submitted_rx)
}

fn custom_prompt_view_ctrl_d() -> (CustomPromptView, Receiver<String>) {
    let (submitted, submitted_rx) = std::sync::mpsc::channel();
    let view = CustomPromptView::new(
        "Edit context".to_string(),
        "Type context".to_string(),
        String::new(),
        /*context_label*/ None,
        Box::new(move |text| {
            submitted.send(text).expect("send submitted text");
        }),
    )
    .with_submit_mode(CustomPromptSubmitMode::CtrlD);
    (view, submitted_rx)
}

fn elapsed(ms: usize) -> std::time::Duration {
    std::time::Duration::from_millis(ms as u64)
}
