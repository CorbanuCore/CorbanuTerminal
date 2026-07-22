use std::fmt;
use std::str::FromStr;

use anyhow::Context;
use serde::Deserialize;
use serde::Serialize;
use sha2::Digest;
use sha2::Sha256;
use teloxide::types::ChatId;
use teloxide::types::Message;
use teloxide::types::MessageId;
use teloxide::types::ThreadId;

/// Stable identity for one Telegram conversation.
///
/// Private chats and non-topic groups have no thread component. Forum topics
/// and reply threads include Telegram's `message_thread_id`, preventing state,
/// approvals, and output from crossing between topics in the same chat.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConversationKey {
    pub chat_id: ChatId,
    pub thread_id: Option<ThreadId>,
}

impl ConversationKey {
    pub fn new(chat_id: ChatId, thread_id: Option<ThreadId>) -> Self {
        Self { chat_id, thread_id }
    }

    pub fn from_message(message: &Message) -> Self {
        Self::new(message.chat.id, message.thread_id)
    }

    pub fn storage_key(self) -> String {
        match self.thread_id {
            Some(ThreadId(MessageId(thread_id))) => format!("{}:{thread_id}", self.chat_id.0),
            None => self.chat_id.0.to_string(),
        }
    }

    pub fn display_label(self) -> String {
        match self.thread_id {
            Some(ThreadId(MessageId(thread_id))) => {
                format!("chat {} · topic {thread_id}", self.chat_id.0)
            }
            None => format!("chat {}", self.chat_id.0),
        }
    }

    pub fn redacted_id(self) -> String {
        let digest = format!("{:x}", Sha256::digest(self.storage_key().as_bytes()));
        digest[..12].to_string()
    }
}

impl fmt::Display for ConversationKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.storage_key())
    }
}

impl From<ChatId> for ConversationKey {
    fn from(chat_id: ChatId) -> Self {
        Self::new(chat_id, None)
    }
}

impl FromStr for ConversationKey {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let (chat_id, thread_id) = match value.split_once(':') {
            Some((chat_id, thread_id)) => (
                chat_id,
                Some(
                    thread_id
                        .parse::<i32>()
                        .with_context(|| format!("invalid Telegram thread id in `{value}`"))?,
                ),
            ),
            None => (value, None),
        };
        let chat_id = chat_id
            .parse::<i64>()
            .with_context(|| format!("invalid Telegram chat id in `{value}`"))?;
        Ok(Self::new(
            ChatId(chat_id),
            thread_id.map(|thread_id| ThreadId(MessageId(thread_id))),
        ))
    }
}

#[cfg(test)]
#[path = "conversation_tests.rs"]
mod tests;
