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
fn character_before_a_request_change_does_not_let_the_next_one_arm() {
    // "py" split across a request swap: `y` must not arm "yes" on the new one.
    for first in [Some(1), None] {
        let mut guard = TypingGuard::default();
        match first {
            Some(idx) => {
                guard.on_text_key(Some(idx));
            }
            None => guard.on_persistent_key(),
        }

        guard.on_request_changed();

        assert_eq!(guard.on_text_key(Some(0)), None, "{first:?}");
        assert_eq!(guard.on_confirm(), Confirm::Blocked, "{first:?}");
    }
}

#[test]
fn request_change_keeps_typed_text() {
    let mut guard = TypingGuard::default();
    guard.on_typed_input();

    guard.on_request_changed();

    assert_eq!(guard.notice(), Some(Notice::TypedText));
    assert!(!guard.accepts_commands());
}

#[test]
fn character_after_a_command_key_is_typed_text() {
    // "op": `o` opened the source thread; `p` is the rest of a word.
    let mut guard = TypingGuard::default();

    guard.on_command_key();

    assert_eq!(guard.on_text_key(Some(1)), None);
    assert_eq!(guard.on_confirm(), Confirm::Blocked);
}

#[test]
fn persistent_option_key_chooses_nothing_and_blocks_enter() {
    let mut guard = TypingGuard::default();

    guard.on_persistent_key();

    assert_eq!(guard.notice(), Some(Notice::PersistentNeedsArrows));
    assert_eq!(guard.on_confirm(), Confirm::Blocked);
    assert!(!guard.accepts_commands());
    // Navigation is the way to choose it.
    guard.reset();
    assert_eq!(guard.on_confirm(), Confirm::Highlighted);
}

#[test]
fn persistent_option_key_inside_a_word_is_typed_text() {
    let mut guard = TypingGuard::default();

    guard.on_text_key(Some(0));
    guard.on_persistent_key();
    assert_eq!(guard.notice(), Some(Notice::TypedText));

    let mut guard = TypingGuard::default();
    guard.on_persistent_key();
    assert_eq!(guard.on_text_key(Some(0)), None);
    assert_eq!(guard.notice(), Some(Notice::TypedText));
    assert_eq!(guard.on_confirm(), Confirm::Blocked);
}
