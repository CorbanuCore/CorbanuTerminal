//! Opt-in raw Chat evidence. The cache-write count is read from
//! `prompt_tokens_details.cache_write_tokens`, where OpenRouter reports it,
//! and the charge from `usage.cost` (see [`ChatUsagePatch::billed_usd`]).
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
    /// The charge stated with the response, in USD, as exact plain decimal
    /// text: `usage.cost`, which OpenRouter reports on every response. Absent
    /// when there is none, when it is not a nonnegative number, and when
    /// `usage.is_byok` is true - the provider then bills the upstream key
    /// separately and the figure is only its own fee. Never a reason to reject
    /// the usage report: the token counters stand on their own.
    pub billed_usd: Option<String>,
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

/// `usage.cost` as exact plain decimal text, per [`ChatUsagePatch::billed_usd`].
fn billed_usd(fields: &serde_json::Map<String, Value>) -> Option<String> {
    if fields.get("is_byok").and_then(Value::as_bool) == Some(true) {
        return None;
    }
    match fields.get("cost")? {
        Value::Number(number) => plain_decimal(&number.to_string()),
        _ => None,
    }
}

/// A JSON number's text ("0.0123", "1.23e-5", "7") as plain nonnegative decimal
/// text ("0.0123", "0.0000123", "7"), digit for digit. Anything else is None.
fn plain_decimal(text: &str) -> Option<String> {
    let (mantissa, exponent) = match text.find(['e', 'E']) {
        Some(at) => (&text[..at], text[at + 1..].parse::<i32>().ok()?),
        None => (text, 0),
    };
    let (whole, fraction) = mantissa.split_once('.').unwrap_or((mantissa, ""));
    if whole.is_empty()
        || !whole
            .bytes()
            .chain(fraction.bytes())
            .all(|b| b.is_ascii_digit())
    {
        return None;
    }
    let digits = format!("{whole}{fraction}");
    // Position of the decimal point within `digits` after applying the exponent.
    let point = i32::try_from(whole.len()).ok()?.checked_add(exponent)?;
    if !(-40..=40).contains(&point) {
        return None;
    }
    let (int_part, frac_part) = if point <= 0 {
        (
            "0".to_string(),
            format!("{}{digits}", "0".repeat(point.unsigned_abs() as usize)),
        )
    } else if point as usize >= digits.len() {
        (
            format!("{digits}{}", "0".repeat(point as usize - digits.len())),
            String::new(),
        )
    } else {
        (
            digits[..point as usize].to_string(),
            digits[point as usize..].to_string(),
        )
    };
    let int_part = int_part.trim_start_matches('0');
    let int_part = if int_part.is_empty() { "0" } else { int_part };
    let frac_part = frac_part.trim_end_matches('0');
    Some(if frac_part.is_empty() {
        int_part.to_string()
    } else {
        format!("{int_part}.{frac_part}")
    })
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
        billed_usd: billed_usd(fields),
    }))
}
