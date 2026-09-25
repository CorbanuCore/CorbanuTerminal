//! Opt-in raw Chat evidence. The cache-write count is read from
//! `prompt_tokens_details.cache_write_tokens`, where OpenRouter reports it;
//! no vendor billing fields are supported.
use crate::error::ApiError;
use serde_json::Value;
use std::future::Future;
use std::pin::Pin;

/// Wire presence before the native display conversion supplies defaults.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ChatTokenPresence {
    #[default]
    Missing,
    Null,
    Number(i64),
}

/// The six supported cumulative counters; absent fields never mean zero.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ChatUsagePatch {
    pub input_tokens: ChatTokenPresence,
    pub cached_tokens: ChatTokenPresence,
    pub cache_write_tokens: ChatTokenPresence,
    pub output_tokens: ChatTokenPresence,
    pub reasoning_tokens: ChatTokenPresence,
    pub total_tokens: ChatTokenPresence,
}

/// Bounded error deliberately containing no response body.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InvalidChatUsage;

/// Awaited evidence sink bound to one physical response. Implementations must
/// preserve positions and reject invalid evidence before terminal handling.
pub trait ChatUsageObserver: Send + Sync {
    fn observe(
        &self,
        position: i64,
        usage: Result<ChatUsagePatch, InvalidChatUsage>,
    ) -> Pin<Box<dyn Future<Output = Result<(), ApiError>> + Send + '_>>;
}

fn field(value: Option<&Value>) -> Result<ChatTokenPresence, InvalidChatUsage> {
    match value {
        None => Ok(ChatTokenPresence::Missing),
        Some(Value::Null) => Ok(ChatTokenPresence::Null),
        Some(value) => value
            .as_i64()
            .filter(|n| *n >= 0)
            .map(ChatTokenPresence::Number)
            .ok_or(InvalidChatUsage),
    }
}

fn detail(value: Option<&Value>, name: &str) -> Result<ChatTokenPresence, InvalidChatUsage> {
    match value {
        Some(Value::Object(fields)) => field(fields.get(name)),
        None | Some(Value::Null) => field(value),
        _ => Err(InvalidChatUsage),
    }
}

/// Numeric usage carried by a non-streaming Chat Completions body, if it
/// carries any.
///
/// A client outside this process can send a chat request and report what the
/// provider answered; reading its numbers with the same parser the streaming
/// path uses keeps one definition of what a usage object means. A body with no
/// `usage` is not an error: the provider stated no numbers, which the ledger
/// records as unknown rather than as zero.
pub fn body_usage(body: &[u8]) -> Result<Option<ChatUsagePatch>, InvalidChatUsage> {
    let data = std::str::from_utf8(body).map_err(|_| InvalidChatUsage)?;
    decode(data)
}

pub(super) fn decode(data: &str) -> Result<Option<ChatUsagePatch>, InvalidChatUsage> {
    let value: Value = serde_json::from_str(data).map_err(|_| InvalidChatUsage)?;
    let object = value.as_object().ok_or(InvalidChatUsage)?;
    let fields = match object.get("usage") {
        None | Some(Value::Null) => return Ok(None),
        Some(Value::Object(fields)) => fields,
        _ => return Err(InvalidChatUsage),
    };
    Ok(Some(ChatUsagePatch {
        input_tokens: field(fields.get("prompt_tokens"))?,
        cached_tokens: detail(fields.get("prompt_tokens_details"), "cached_tokens")?,
        cache_write_tokens: detail(fields.get("prompt_tokens_details"), "cache_write_tokens")?,
        output_tokens: field(fields.get("completion_tokens"))?,
        reasoning_tokens: detail(fields.get("completion_tokens_details"), "reasoning_tokens")?,
        total_tokens: field(fields.get("total_tokens"))?,
    }))
}
