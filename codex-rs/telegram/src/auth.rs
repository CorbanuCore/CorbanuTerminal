use teloxide::types::ChatId;
use tracing::warn;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatAllowlist {
    allowed_chat_ids: Vec<i64>,
}

impl ChatAllowlist {
    pub fn new(allowed_chat_ids: Vec<i64>) -> Self {
        Self { allowed_chat_ids }
    }

    pub fn is_authorized(&self, chat_id: ChatId) -> bool {
        self.allowed_chat_ids.contains(&chat_id.0)
    }

    pub fn reject_if_unauthorized(&self, chat_id: ChatId) -> bool {
        if self.is_authorized(chat_id) {
            true
        } else {
            warn!(chat_id = chat_id.0, "rejecting unauthorized Telegram chat");
            false
        }
    }

    pub fn is_empty(&self) -> bool {
        self.allowed_chat_ids.is_empty()
    }
}
