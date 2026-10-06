use super::*;
use pretty_assertions::assert_eq;

#[test]
fn enter_on_an_untouched_prompt_confirms_the_highlighted_row() {
    let mut guard = TypingGuard::default();

    assert_eq!(guard.on_confirm(), Confirm::Highlighted);
}

#[test]
fn one_decision_key_arms_its_option_for_enter() {
    let mut guard = TypingGuard::default();

    assert_eq!(guard.on_text_key(Some(1)), Some(1));
    assert_eq!(guard.on_confirm(), Confirm::Option(1));
}

#[test]
fn second_character_is_typed_text_however_slowly_it_arrives() {
    // "permissions": `p` is bound, `e` is not. Timing plays no part.
    let mut guard = TypingGuard::default();

    assert_eq!(guard.on_text_key(Some(1)), Some(1));
    assert_eq!(guard.on_text_key(/*option*/ None), None);
    assert_eq!(guard.on_confirm(), Confirm::Blocked);
    // Later decision keys stay text until the user navigates.
    assert_eq!(guard.on_text_key(Some(0)), None);
    assert_eq!(guard.notice(), Some(Notice::TypedText));

    guard.reset();
    assert_eq!(guard.on_text_key(Some(0)), Some(0));
}

#[test]
fn two_decision_keys_are_typed_text() {
    let mut guard = TypingGuard::default();

    guard.on_text_key(Some(0));

    assert_eq!(guard.on_text_key(Some(0)), None);
    assert_eq!(guard.on_confirm(), Confirm::Blocked);
}

#[test]
fn unbound_first_character_is_typed_text() {
    let mut guard = TypingGuard::default();

    assert_eq!(guard.on_text_key(/*option*/ None), None);

    assert_eq!(guard.on_confirm(), Confirm::Blocked);
}

#[test]
fn editing_input_after_a_decision_key_blocks_enter() {
    // `p` then Backspace, or a paste.
    let mut guard = TypingGuard::default();

    guard.on_text_key(Some(1));
    guard.on_typed_input();

    assert_eq!(guard.on_confirm(), Confirm::Blocked);
}

#[test]
fn next_request_needs_a_fresh_choice() {
    let mut guard = TypingGuard::default();
    guard.on_text_key(Some(0));
    assert_eq!(guard.on_confirm(), Confirm::Option(0));

    guard.on_request_changed();

    // A second Enter is explained, not confirmed.
    assert_eq!(guard.notice(), None);
    assert_eq!(guard.on_confirm(), Confirm::Blocked);
    assert_eq!(guard.notice(), Some(Notice::ChooseFirst));
    // A decision key or navigation makes Enter work again.
    assert_eq!(guard.on_text_key(Some(2)), Some(2));
    assert_eq!(guard.on_confirm(), Confirm::Option(2));
}

#[test]
fn request_change_keeps_typed_text() {
    let mut guard = TypingGuard::default();
    guard.on_typed_input();

    guard.on_request_changed();

    assert_eq!(guard.notice(), Some(Notice::TypedText));
    assert!(!guard.accepts_commands());
}
