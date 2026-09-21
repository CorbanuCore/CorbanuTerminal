//! Lossless, opt-in numeric evidence before the display usage accumulator.
use crate::error::ApiError;
use serde::Deserialize;
use serde_json::Value;
use std::future::Future;
use std::pin::Pin;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AnthropicTokenPresence {
    #[default]
    Missing,
    Null,
    Number(i64),
}

impl<'de> Deserialize<'de> for AnthropicTokenPresence {
    fn deserialize<D: serde::Deserializer<'de>>(decoder: D) -> Result<Self, D::Error> {
        match Option::<i64>::deserialize(decoder)? {
            None => Ok(Self::Null),
            Some(value) if value >= 0 => Ok(Self::Number(value)),
            Some(_) => Err(serde::de::Error::custom("negative token count")),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct AnthropicUsagePatch {
    pub input_tokens: AnthropicTokenPresence,
    pub cache_read_input_tokens: AnthropicTokenPresence,
    pub cache_creation_input_tokens: AnthropicTokenPresence,
    pub output_tokens: AnthropicTokenPresence,
}

/// Invalid numeric evidence; deliberately contains no provider response data.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InvalidAnthropicUsage;

/// Awaited numeric-only sink bound to one physical successful response.
/// Implementations must preserve source positions and fail visibly, including on
/// invalid evidence. Completion is not delivered until this future succeeds.
pub trait AnthropicUsageObserver: Send + Sync {
    fn observe(
        &self,
        position: i64,
        usage: Result<AnthropicUsagePatch, InvalidAnthropicUsage>,
    ) -> Pin<Box<dyn Future<Output = Result<(), ApiError>> + Send + '_>>;
}

/// Numeric usage carried by a non-streaming Messages body, if it carries any.
///
/// The streaming shape puts usage inside events; a single response puts it at
/// the top level. A body with no `usage` is not an error: the provider stated
/// no numbers, which the ledger records as unknown rather than as zero.
pub fn body_usage(body: &[u8]) -> Result<Option<AnthropicUsagePatch>, InvalidAnthropicUsage> {
    let value: Value = serde_json::from_slice(body).map_err(|_| InvalidAnthropicUsage)?;
    match value.get("usage") {
        None | Some(Value::Null) => Ok(None),
        Some(usage) => serde_json::from_value(usage.clone())
            .map(Some)
            .map_err(|_| InvalidAnthropicUsage),
    }
}

/// The usage an Anthropic event stream stated, as one object in the shape the
/// provider used to state it.
///
/// A streamed Messages response puts its numbers in events rather than in a
/// body: the input and cache counts arrive with `message_start` and the output
/// count arrives cumulatively with `message_delta`. A caller holding the whole
/// stream - a proxy that buffered it, for instance - has everything the
/// provider said, and this is where that is read, beside the event shapes it
/// depends on, rather than restated by every caller that needs it.
///
/// The last statement of a field wins, because `message_delta` restates the
/// running total. `None` means the stream stated no usage at all, which is
/// unknown rather than zero.
pub fn stream_usage(body: &[u8]) -> Option<Value> {
    let mut usage = serde_json::Map::new();
    for line in std::str::from_utf8(body).ok()?.lines() {
        let Some(data) = line.strip_prefix("data:") else {
            continue;
        };
        let Ok(value) = serde_json::from_str::<Value>(data.trim()) else {
            continue;
        };
        let stated = match value.get("type").and_then(Value::as_str) {
            Some("message_start") => value
                .get("message")
                .and_then(|message| message.get("usage")),
            Some("message_delta") => value.get("usage"),
            _ => None,
        };
        if let Some(Value::Object(stated)) = stated {
            for (field, value) in stated {
                usage.insert(field.clone(), value.clone());
            }
        }
    }
    (!usage.is_empty()).then_some(Value::Object(usage))
}

pub(super) fn decode(data: &str) -> Result<Option<AnthropicUsagePatch>, InvalidAnthropicUsage> {
    let value: Value = serde_json::from_str(data).map_err(|_| InvalidAnthropicUsage)?;
    let usage = match value.get("type").and_then(Value::as_str) {
        Some("message_start") => value
            .get("message")
            .and_then(|message| message.get("usage")),
        Some("message_delta") => value.get("usage"),
        _ => return Ok(None),
    };
    match usage {
        None | Some(Value::Null) => Ok(None),
        Some(usage) => serde_json::from_value(usage.clone())
            .map(Some)
            .map_err(|_| InvalidAnthropicUsage),
    }
}

#[cfg(test)]
#[path = "anthropic_accounting_tests.rs"]
mod tests;
