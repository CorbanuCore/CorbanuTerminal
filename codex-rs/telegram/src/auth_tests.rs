use pretty_assertions::assert_eq;
use teloxide::types::ChatId;

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
