use pretty_assertions::assert_eq;
use teloxide::types::ChatId;
use teloxide::types::UserId;

use codex_telegram::auth::ChatAllowlist;

#[test]
fn allowlist_defaults_to_deny() {
    let allowlist = ChatAllowlist::new(Vec::new());

    assert_eq!(allowlist.is_authorized(ChatId(42)), false);
    assert_eq!(allowlist.is_empty(), true);
}

#[test]
fn allowlist_accepts_positive_and_supergroup_ids() {
    let allowlist = ChatAllowlist::new(vec![21_000_038, -1_001_941_234_987]);

    assert_eq!(allowlist.is_authorized(ChatId(21_000_038)), true);
    assert_eq!(allowlist.is_authorized(ChatId(-1_001_941_234_987)), true);
    assert_eq!(allowlist.is_authorized(ChatId(7)), false);
}

#[test]
fn group_chat_requires_an_explicit_authorized_actor() {
    let allowlist = ChatAllowlist::with_users(vec![-100_123], vec![7]);

    assert!(allowlist.reject_if_unauthorized_actor(ChatId(-100_123), Some(UserId(7)), false));
    assert!(!allowlist.reject_if_unauthorized_actor(ChatId(-100_123), Some(UserId(8)), false));
    assert!(!allowlist.reject_if_unauthorized_actor(ChatId(-100_123), None, false));
}

#[test]
fn private_chat_retains_chat_allowlist_behavior() {
    let allowlist = ChatAllowlist::new(vec![42]);

    assert!(allowlist.reject_if_unauthorized_actor(ChatId(42), Some(UserId(42)), true));
}
