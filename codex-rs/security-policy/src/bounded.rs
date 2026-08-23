use std::fmt;

use schemars::JsonSchema;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use thiserror::Error;

/// Maximum encoded size of any descriptive value carried by a policy object.
///
/// Policy objects intentionally cannot carry arbitrary payloads, tool output,
/// credentials, or financial records.
pub const MAX_POLICY_TEXT_BYTES: usize = 256;

/// A short, validated policy identifier or destination.
///
/// Leading or trailing whitespace and control characters are rejected so the
/// same displayed value always has the same serialized identity.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, JsonSchema)]
#[serde(transparent)]
#[schemars(transparent)]
pub struct BoundedText(String);

impl BoundedText {
    pub fn new(value: impl Into<String>) -> Result<Self, BoundedTextError> {
        let value = value.into();
        if value.is_empty() {
            return Err(BoundedTextError::Empty);
        }
        if value.len() > MAX_POLICY_TEXT_BYTES {
            return Err(BoundedTextError::TooLong {
                actual: value.len(),
                maximum: MAX_POLICY_TEXT_BYTES,
            });
        }
        if value.trim() != value {
            return Err(BoundedTextError::SurroundingWhitespace);
        }
        if value.chars().any(char::is_control) {
            return Err(BoundedTextError::ControlCharacter);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for BoundedText {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for BoundedText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<'de> Deserialize<'de> for BoundedText {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(serde::de::Error::custom)
    }
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum BoundedTextError {
    #[error("policy text must not be empty")]
    Empty,
    #[error("policy text is {actual} bytes; maximum is {maximum}")]
    TooLong { actual: usize, maximum: usize },
    #[error("policy text must not have leading or trailing whitespace")]
    SurroundingWhitespace,
    #[error("policy text must not contain control characters")]
    ControlCharacter,
}
