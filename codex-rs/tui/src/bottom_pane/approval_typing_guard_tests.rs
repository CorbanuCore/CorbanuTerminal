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

/// #817: the next request waits for nothing but the bottom pane's guard
/// (#787), which drops a double or held Enter; after it, Enter confirms.
#[test]
fn next_request_takes_a_later_enter() {
    let mut guard = TypingGuard::default();
    guard.on_text_key(Some(0));
    assert_eq!(guard.on_confirm(), Confirm::Option(0));

    guard.on_request_changed();

    assert_eq!(guard.notice(), None);
    assert_eq!(guard.on_confirm(), Confirm::Highlighted);
    assert_eq!(guard.notice(), None);
}

/// A key armed or pressed for the previous request does not carry over:
/// Enter on the new one confirms its own highlighted row. Keys that reach
/// the new request as it appears are the bottom pane's to drop (#787).
#[test]
fn a_key_for_the_previous_request_does_not_carry_over() {
    for first in [Some(1), None] {
        let mut guard = TypingGuard::default();
        match first {
            Some(idx) => {
                guard.on_text_key(Some(idx));
            }
            None => guard.on_persistent_key(),
        }

        guard.on_request_changed();

        assert_eq!(guard.notice(), None, "{first:?}");
        assert_eq!(guard.on_confirm(), Confirm::Highlighted, "{first:?}");
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

/// #523: right after a character command key such as open thread, the
/// prompt says to navigate, takes a key as typed text, and waits for the
/// arrow keys.
#[test]
fn deliberate_choice_says_to_navigate_first() {
    let mut guard = TypingGuard::default();
    guard.on_command_key();
    assert!(guard.awaits_navigation());
    assert_eq!(guard.on_confirm(), Confirm::Blocked);
    assert_eq!(guard.notice(), Some(Notice::NavigateFirst));
    assert_eq!(guard.on_text_key(Some(0)), None);
    assert_eq!(guard.notice(), Some(Notice::TypedText));
    assert_eq!(guard.on_confirm(), Confirm::Blocked);
    guard.reset();
    assert!(!guard.awaits_navigation());
    assert_eq!(guard.on_confirm(), Confirm::Highlighted);
}

/// #787: after input was dropped as the request appeared, Enter and a
/// decision key don't answer it; the prompt says why until the user
/// navigates, and a new request starts without the notice.
#[test]
fn early_input_needs_navigation_before_enter() {
    let mut guard = TypingGuard::default();

    guard.on_early_input();
    assert_eq!(guard.notice(), Some(Notice::EarlyKeys));
    assert_eq!(guard.on_confirm(), Confirm::Blocked);
    assert_eq!(guard.on_text_key(Some(0)), None);
    assert_eq!(guard.on_confirm(), Confirm::Blocked);
    assert_eq!(guard.notice(), Some(Notice::EarlyKeys));

    guard.reset();
    assert_eq!(guard.notice(), None);
    assert_eq!(guard.on_confirm(), Confirm::Highlighted);

    // #817: keys dropped for one request do not hold the next.
    guard.on_early_input();
    guard.on_request_changed();
    assert_eq!(guard.notice(), None);
    assert_eq!(guard.on_confirm(), Confirm::Highlighted);
}

/// #817 (review r1): text typed into one request does not block the next
/// request of the same prompt; on the same request shown again it stays.
#[test]
fn typed_text_stays_with_its_request() {
    let mut guard = TypingGuard::default();
    guard.on_typed_input();
    guard.on_new_request();
    assert_eq!(guard.notice(), None);
    assert_eq!(guard.on_confirm(), Confirm::Highlighted);

    let mut guard = TypingGuard::default();
    guard.on_early_input();
    guard.on_new_request();
    assert_eq!(guard.notice(), None);
    assert_eq!(guard.on_confirm(), Confirm::Highlighted);
}
