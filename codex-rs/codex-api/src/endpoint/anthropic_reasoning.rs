//! Optional compatible-provider display breakdown, not an extra billing bucket.
//! Adapted from goodalexander's #28 decoder; aliases are never added together.
use serde::Deserialize;
use serde_json::Value;

pub(super) fn decode<'de, D: serde::Deserializer<'de>>(
    decoder: D,
) -> Result<Option<i64>, D::Error> {
    let details = Value::deserialize(decoder)?;
    Ok(["thinking_tokens", "reasoning_tokens"]
        .into_iter()
        .filter_map(|field| details.get(field).and_then(Value::as_i64))
        .filter(|value| *value >= 0)
        .max())
}
