use pretty_assertions::assert_eq;
use teloxide::types::ChatId;
use teloxide::types::MessageId;
use teloxide::types::ThreadId;

use super::ConversationKey;

#[test]
fn storage_key_round_trips_private_chat() {
    let key = ConversationKey::new(ChatId(42), None);

    assert_eq!(key.storage_key(), "42");
    assert_eq!(key.storage_key().parse::<ConversationKey>().unwrap(), key);
}

#[test]
fn storage_key_round_trips_group_topic() {
    let key = ConversationKey::new(ChatId(-100_200), Some(ThreadId(MessageId(17))));

    assert_eq!(key.storage_key(), "-100200:17");
    assert_eq!(key.storage_key().parse::<ConversationKey>().unwrap(), key);
}

#[test]
fn old_chat_only_storage_keys_remain_compatible() {
    assert_eq!(
        "-1002".parse::<ConversationKey>().unwrap(),
        ConversationKey::new(ChatId(-1002), None)
    );
}

#[test]
fn redacted_identity_is_stable_without_exposing_chat_id() {
    let key = ConversationKey::new(ChatId(42), None);
    assert_eq!(key.redacted_id(), key.redacted_id());
    assert_eq!(key.redacted_id().len(), 12);
    assert_ne!(key.redacted_id(), key.storage_key());
}
