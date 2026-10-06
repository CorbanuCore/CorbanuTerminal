use super::*;
use pretty_assertions::assert_eq;

const SETTLE: Duration = SHORTCUT_SETTLE_DELAY;

#[test]
fn lone_decision_key_applies_after_quiet_period() {
    let now = Instant::now();
    let mut guard = TypingGuard::default();

    guard.on_text_key(Some(ShortcutTarget::Option(0)), now);

    assert_eq!(guard.next_delay(now), Some(SETTLE));
    assert_eq!(
        guard.take_due(now + SETTLE - Duration::from_millis(1)),
        None
    );
    assert_eq!(
        guard.take_due(now + SETTLE),
        Some(ShortcutTarget::Option(0))
    );
    assert!(!guard.is_engaged());
}

#[test]
fn second_text_key_turns_a_held_key_into_typed_text() {
    let now = Instant::now();
    let mut guard = TypingGuard::default();

    // "permissions": `p` is bound, `e` is not.
    guard.on_text_key(Some(ShortcutTarget::Option(1)), now);
    guard.on_text_key(None, now + Duration::from_millis(5));

    assert_eq!(guard.take_due(now + SETTLE * 4), None);
    assert!(guard.typed_text());
    // Later decision keys stay text until the user navigates.
    guard.on_text_key(Some(ShortcutTarget::Option(0)), now + SETTLE * 5);
    assert_eq!(guard.take_due(now + SETTLE * 10), None);
}

#[test]
fn two_bound_keys_in_a_burst_are_typed_text() {
    let now = Instant::now();
    let mut guard = TypingGuard::default();

    guard.on_text_key(Some(ShortcutTarget::Option(0)), now);
    guard.on_text_key(
        Some(ShortcutTarget::Option(0)),
        now + Duration::from_millis(1),
    );

    assert_eq!(guard.take_due(now + SETTLE * 2), None);
    assert!(guard.typed_text());
}

#[test]
fn unbound_first_character_is_typed_text() {
    let now = Instant::now();
    let mut guard = TypingGuard::default();

    guard.on_text_key(/*target*/ None, now);

    assert!(guard.typed_text());
}

#[test]
fn enter_after_typed_text_is_blocked_until_reset() {
    let now = Instant::now();
    let mut guard = TypingGuard::default();

    assert!(!guard.blocks_accept());
    guard.on_text_key(Some(ShortcutTarget::Option(0)), now);
    // Enter right after a held key cancels it rather than confirming anything.
    assert!(guard.blocks_accept());
    assert_eq!(guard.take_due(now + SETTLE), None);

    guard.reset();
    assert!(!guard.blocks_accept());
}

#[test]
fn paste_is_typed_text() {
    let now = Instant::now();
    let mut guard = TypingGuard::default();

    guard.on_text_key(Some(ShortcutTarget::Cancel), now);
    guard.on_paste();

    assert_eq!(guard.take_due(now + SETTLE), None);
    assert!(guard.blocks_accept());
}
