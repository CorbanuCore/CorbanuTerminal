//! Opt-in, numeric-only evidence captured before Responses display conversion.
use crate::error::ApiError;
use serde_json::Value;
use std::future::Future;
use std::pin::Pin;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ResponsesTokenPresence {
    #[default]
    Missing,
    Null,
    Number(i64),
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ResponsesUsagePatch {
    pub input_tokens: ResponsesTokenPresence,
    pub cached_tokens: ResponsesTokenPresence,
    pub cache_write_tokens: ResponsesTokenPresence,
    pub output_tokens: ResponsesTokenPresence,
    pub reasoning_tokens: ResponsesTokenPresence,
    pub total_tokens: ResponsesTokenPresence,
}

/// Invalid evidence, deliberately without provider body or credential data.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InvalidResponsesUsage;

/// Awaited numeric evidence sink bound to one physical HTTP response.
/// Implementations must retain source positions and reject invalid evidence.
/// Completion and terminal handling wait for successful persistence.
pub trait ResponsesUsageObserver: Send + Sync {
    fn observe(
        &self,
        position: i64,
        usage: Result<ResponsesUsagePatch, InvalidResponsesUsage>,
    ) -> Pin<Box<dyn Future<Output = Result<(), ApiError>> + Send + '_>>;
}

fn field(value: Option<&Value>) -> Result<ResponsesTokenPresence, InvalidResponsesUsage> {
    match value {
        None => Ok(ResponsesTokenPresence::Missing),
        Some(Value::Null) => Ok(ResponsesTokenPresence::Null),
        Some(value) => value
            .as_i64()
            .filter(|n| *n >= 0)
            .map(ResponsesTokenPresence::Number)
            .ok_or(InvalidResponsesUsage),
    }
}

fn detail(
    value: Option<&Value>,
    name: &str,
) -> Result<ResponsesTokenPresence, InvalidResponsesUsage> {
    match value {
        Some(Value::Object(fields)) => field(fields.get(name)),
        None | Some(Value::Null) => field(value),
        _ => Err(InvalidResponsesUsage),
    }
}

fn patch(value: Option<&Value>) -> Result<Option<ResponsesUsagePatch>, InvalidResponsesUsage> {
    let fields = match value {
        None | Some(Value::Null) => return Ok(None),
        Some(Value::Object(fields)) => fields,
        _ => return Err(InvalidResponsesUsage),
    };
    Ok(Some(ResponsesUsagePatch {
        input_tokens: field(fields.get("input_tokens"))?,
        cached_tokens: detail(fields.get("input_tokens_details"), "cached_tokens")?,
        cache_write_tokens: detail(fields.get("input_tokens_details"), "cache_write_tokens")?,
        output_tokens: field(fields.get("output_tokens"))?,
        reasoning_tokens: detail(fields.get("output_tokens_details"), "reasoning_tokens")?,
        total_tokens: field(fields.get("total_tokens"))?,
    }))
}

/// Numeric usage carried by a non-streaming Responses body, if it carries any.
///
/// The compaction endpoint answers with one JSON object rather than a stream.
/// A body with no `usage` is not an error: it means the provider stated no
/// numbers, which the ledger records as unknown rather than as zero.
pub fn body_usage(body: &[u8]) -> Result<Option<ResponsesUsagePatch>, InvalidResponsesUsage> {
    let value: Value = serde_json::from_slice(body).map_err(|_| InvalidResponsesUsage)?;
    patch(value.get("usage"))
}

pub(crate) fn decode(data: &str) -> Result<Option<ResponsesUsagePatch>, InvalidResponsesUsage> {
    let value: Value = serde_json::from_str(data).map_err(|_| InvalidResponsesUsage)?;
    match value.get("type").and_then(Value::as_str) {
        Some("response.completed" | "response.failed" | "response.incomplete") => {
            patch(value.get("response").and_then(|v| v.get("usage")))
        }
        Some("response.usage") => {
            let top = patch(value.get("usage"))?;
            let nested = patch(value.get("response").and_then(|v| v.get("usage")))?;
            match (top, nested) {
                (Some(a), Some(b)) if a != b => Err(InvalidResponsesUsage),
                (Some(a), _) | (_, Some(a)) => Ok(Some(a)),
                (None, None) => Ok(None),
            }
        }
        _ => Ok(None),
    }
}

#[cfg(test)]
#[path = "responses_accounting_tests.rs"]
mod tests;
