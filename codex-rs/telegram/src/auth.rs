use teloxide::types::ChatId;
use teloxide::types::UserId;
use tracing::warn;

use crate::conversation::ConversationKey;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatAllowlist {
    allowed_chat_ids: Vec<i64>,
    allowed_user_ids: Vec<u64>,
}

impl ChatAllowlist {
    pub fn new(allowed_chat_ids: Vec<i64>) -> Self {
        Self {
            allowed_chat_ids,
            allowed_user_ids: Vec::new(),
        }
    }

    pub fn with_users(allowed_chat_ids: Vec<i64>, allowed_user_ids: Vec<u64>) -> Self {
        Self {
            allowed_chat_ids,
            allowed_user_ids,
        }
    }

    pub fn is_authorized(&self, chat_id: ChatId) -> bool {
        self.allowed_chat_ids.contains(&chat_id.0)
    }

    pub fn reject_if_unauthorized(&self, chat_id: ChatId) -> bool {
        if self.is_authorized(chat_id) {
            true
        } else {
            warn!(
                conversation = %ConversationKey::from(chat_id).redacted_id(),
                "rejecting unauthorized Telegram chat"
            );
            false
        }
    }

    /// Authorize both the conversation and the human actor. Private chats keep
    /// the legacy chat-id allowlist behavior. Group and supergroup messages
    /// additionally require an explicit user id so membership alone never
    /// grants command execution or approval authority.
    pub fn reject_if_unauthorized_actor(
        &self,
        chat_id: ChatId,
        user_id: Option<UserId>,
        is_private: bool,
    ) -> bool {
        let allowed = self.is_authorized(chat_id)
            && (is_private
                || user_id.is_some_and(|user_id| self.allowed_user_ids.contains(&user_id.0)));
        if !allowed {
            warn!(
                conversation = %ConversationKey::from(chat_id).redacted_id(),
                actor_present = user_id.is_some(),
                "rejecting unauthorized Telegram actor"
            );
        }
        allowed
    }

    pub fn is_empty(&self) -> bool {
        self.allowed_chat_ids.is_empty()
    }
}
