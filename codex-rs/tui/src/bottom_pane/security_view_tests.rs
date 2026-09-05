use super::*;
use crate::keymap::RuntimeKeymap;
use crossterm::event::KeyModifiers;
use pretty_assertions::assert_eq;

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn snapshot(view: &SecurityView, width: u16) -> String {
    let area = Rect::new(0, 0, width, view.desired_height(width));
    let mut buffer = Buffer::empty(area);
    view.render(area, &mut buffer);
    (0..area.height).map(|y| {
        (0..area.width).map(|x| buffer[(x, y)].symbol()).collect::<String>()
            .trim_end().to_string()
    }).collect::<Vec<_>>().join("\n")
}

#[test]
fn security_view_profiles_never_claim_healthy_protection() {
    for level in PROFILES {
        let view = SecurityView::new(Some(level), RuntimeKeymap::defaults().list);
        insta::assert_snapshot!(format!("security_view_{}", profile_name(level).to_lowercase()), snapshot(&view, 80));
    }
}

#[test]
fn security_view_narrow_and_unknown_state() {
    let mut view = SecurityView::new(None, RuntimeKeymap::defaults().list);
    view.handle_key_event(key(KeyCode::Down));
    view.handle_key_event(key(KeyCode::Enter));
    insta::assert_snapshot!("security_view_unknown_narrow", snapshot(&view, 40));
}

#[test]
fn security_view_navigation_enter_and_cancel_do_not_change_request() {
    let mut view = SecurityView::new(Some(SecurityLevel::Moderate), RuntimeKeymap::defaults().list);
    view.handle_key_event(key(KeyCode::Down));
    view.handle_key_event(key(KeyCode::Enter));
    assert_eq!((view.requested, view.selected, view.inspected, view.completion()),
        (Some(SecurityLevel::Moderate), 2, true, None));
    view.handle_key_event(key(KeyCode::Down));
    assert_eq!((view.selected, view.inspected), (0, false));
    view.handle_key_event(key(KeyCode::Up));
    assert_eq!(view.selected, 2);
    view.handle_key_event(key(KeyCode::Esc));
    assert_eq!((view.requested, view.completion()),
        (Some(SecurityLevel::Moderate), Some(ViewCompletion::Cancelled)));
}

#[test]
fn security_view_uses_configured_navigation_and_cancellation() {
    let mut keymap = RuntimeKeymap::defaults().list;
    keymap.move_down = vec![key_hint::plain(KeyCode::Char('j'))];
    keymap.accept = vec![key_hint::plain(KeyCode::Char('i'))];
    keymap.cancel = vec![key_hint::plain(KeyCode::Char('q'))];
    let mut view = SecurityView::new(Some(SecurityLevel::Permissive), keymap);
    view.handle_key_event(key(KeyCode::Char('j')));
    view.handle_key_event(key(KeyCode::Char('i')));
    assert_eq!((view.selected, view.inspected), (1, true));
    view.handle_key_event(key(KeyCode::Char('q')));
    assert_eq!(view.completion(), Some(ViewCompletion::Cancelled));
}

#[test]
fn security_view_short_terminal_keeps_escape_visible() {
    let view = SecurityView::new(None, RuntimeKeymap::defaults().list);
    let area = Rect::new(0, 0, 40, 8);
    let mut buffer = Buffer::empty(area);
    view.render(area, &mut buffer);
    let text = buffer.content.iter().map(|cell| cell.symbol()).collect::<String>();
    assert!(text.contains("esc close"));
}
