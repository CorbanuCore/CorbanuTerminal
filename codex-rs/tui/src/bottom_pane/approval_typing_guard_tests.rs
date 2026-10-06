use super::*;
use pretty_assertions::assert_eq;

#[test]
fn one_decision_key_arms_its_option_for_enter() {
    let mut guard = TypingGuard::default();

    assert_eq!(guard.on_text_key(Some(1)), Some(1));
    assert!(guard.allows_accept());
}

#[test]
fn second_character_is_typed_text_however_slowly_it_arrives() {
    // "permissions": `p` is bound, `e` is not. Timing plays no part.
    let mut guard = TypingGuard::default();

    assert_eq!(guard.on_text_key(Some(1)), Some(1));
    assert_eq!(guard.on_text_key(/*option*/ None), None);
    assert!(!guard.allows_accept());
    // Later decision keys stay text until the user navigates.
    assert_eq!(guard.on_text_key(Some(0)), None);
    assert!(guard.typed_text());

    guard.reset();
    assert_eq!(guard.on_text_key(Some(0)), Some(0));
}

#[test]
fn two_decision_keys_are_typed_text() {
    let mut guard = TypingGuard::default();

    guard.on_text_key(Some(0));
    assert_eq!(guard.on_text_key(Some(0)), None);

    assert!(guard.typed_text());
}

#[test]
fn unbound_first_character_is_typed_text() {
    let mut guard = TypingGuard::default();

    assert_eq!(guard.on_text_key(/*option*/ None), None);

    assert!(!guard.allows_accept());
}

#[test]
fn editing_input_after_a_decision_key_blocks_enter() {
    // `p` then Backspace, or a paste.
    let mut guard = TypingGuard::default();

    guard.on_text_key(Some(1));
    guard.on_typed_input();

    assert!(!guard.allows_accept());
}

#[test]
fn request_change_disarms_but_keeps_typed_text() {
    let mut armed = TypingGuard::default();
    armed.on_text_key(Some(0));
    armed.on_request_changed();
    assert!(!armed.allows_accept());

    let mut idle = TypingGuard::default();
    idle.on_request_changed();
    assert!(idle.is_idle());
}
