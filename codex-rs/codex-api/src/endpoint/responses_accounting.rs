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
    /// The charge the Vercel AI Gateway stated for this response, in USD, as
    /// exact plain decimal text: `provider_metadata.gateway.gatewayCost`, which
    /// is inference plus the gateway's own surcharges. Absent when there is
    /// none, when it is not a nonnegative number, and when an upstream attempt
    /// used the caller's own provider key (that upstream bills separately).
    /// Never a reason to reject the usage report.
    pub billed_usd: Option<String>,
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
        billed_usd: None,
    }))
}

/// [`ResponsesUsagePatch::billed_usd`] from a response object.
fn gateway_billed_usd(response: Option<&Value>) -> Option<String> {
    let gateway = response?.get("provider_metadata")?.get("gateway")?;
    let own_key = gateway
        .pointer("/routing/modelAttempts")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|attempt| attempt.get("providerAttempts")?.as_array())
        .flatten()
        .any(|attempt| {
            attempt
                .get("credentialType")
                .and_then(Value::as_str)
                .is_some_and(|kind| kind != "system")
        });
    if own_key {
        return None;
    }
    match gateway.get("gatewayCost")? {
        Value::String(text) => crate::endpoint::chat_completions::accounting::plain_decimal(text),
        Value::Number(number) => {
            crate::endpoint::chat_completions::accounting::plain_decimal(&number.to_string())
        }
        _ => None,
    }
}

/// Usage with the stated charge of the response object that carries it.
fn response_patch(
    response: Option<&Value>,
) -> Result<Option<ResponsesUsagePatch>, InvalidResponsesUsage> {
    Ok(
        patch(response.and_then(|v| v.get("usage")))?.map(|usage| ResponsesUsagePatch {
            billed_usd: gateway_billed_usd(response),
            ..usage
        }),
    )
}

/// Numeric usage carried by a non-streaming Responses body, if it carries any.
///
/// The compaction endpoint answers with one JSON object rather than a stream.
/// A body with no `usage` is not an error: it means the provider stated no
/// numbers, which the ledger records as unknown rather than as zero.
pub fn body_usage(body: &[u8]) -> Result<Option<ResponsesUsagePatch>, InvalidResponsesUsage> {
    let value: Value = serde_json::from_slice(body).map_err(|_| InvalidResponsesUsage)?;
    response_patch(Some(&value))
}

pub(crate) fn decode(data: &str) -> Result<Option<ResponsesUsagePatch>, InvalidResponsesUsage> {
    let value: Value = serde_json::from_str(data).map_err(|_| InvalidResponsesUsage)?;
    match value.get("type").and_then(Value::as_str) {
        Some("response.completed" | "response.failed" | "response.incomplete") => {
            response_patch(value.get("response"))
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
