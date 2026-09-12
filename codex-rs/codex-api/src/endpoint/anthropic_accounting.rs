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
