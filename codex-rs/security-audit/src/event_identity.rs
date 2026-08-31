use std::fmt;

use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use sha2::Digest as _;
use sha2::Sha256;
use thiserror::Error;

macro_rules! digest_id {
    ($name:ident) => {
        #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            pub fn as_str(&self) -> &str {
                &self.0
            }

            pub(crate) fn from_digest(value: String) -> Self {
                debug_assert!(is_lower_hex_sha256(&value));
                Self(value)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(formatter)
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let value = String::deserialize(deserializer)?;
                if !is_lower_hex_sha256(&value) {
                    return Err(serde::de::Error::custom("invalid audit digest identity"));
                }
                Ok(Self(value))
            }
        }
    };
}

digest_id!(SecurityEventId);
digest_id!(DecisionId);
digest_id!(ActionId);
digest_id!(ReservationId);

pub(crate) fn hash_value(value: &impl Serialize) -> Result<String, SecurityEventError> {
    let value = serde_json::to_value(value).map_err(SecurityEventError::Serialization)?;
    let canonical = canonicalize(value);
    let bytes = serde_json::to_vec(&canonical).map_err(SecurityEventError::Serialization)?;
    let digest = Sha256::digest(bytes);
    Ok(digest.iter().map(|byte| format!("{byte:02x}")).collect())
}

fn canonicalize(value: serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Array(values) => {
            serde_json::Value::Array(values.into_iter().map(canonicalize).collect())
        }
        serde_json::Value::Object(values) => {
            let mut entries = values.into_iter().collect::<Vec<_>>();
            entries.sort_unstable_by(|(left, _), (right, _)| left.cmp(right));
            let mut canonical = serde_json::Map::new();
            for (key, value) in entries {
                canonical.insert(key, canonicalize(value));
            }
            serde_json::Value::Object(canonical)
        }
        value => value,
    }
}

pub(crate) fn is_lower_hex_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[derive(Debug, Error)]
pub enum SecurityEventError {
    #[error("unsupported security event schema version {found}; supported version is {supported}")]
    UnsupportedVersion { found: u32, supported: u32 },
    #[error("run generation must be nonzero")]
    InvalidRunGeneration,
    #[error("security event timestamps must be non-negative")]
    NegativeTimestamp,
    #[error("authorization request is invalid")]
    InvalidRequest,
    #[error("grant or mandate authority is invalid")]
    InvalidAuthority,
    #[error("authorization decision is bound to a different request")]
    DecisionRequestMismatch,
    #[error("authorization decision identity is invalid")]
    DecisionIntegrityMismatch,
    #[error("dispatch action identity is invalid")]
    ActionIntegrityMismatch,
    #[error("dispatch deduplication identity is invalid")]
    DeduplicationIntegrityMismatch,
    #[error("dispatch reservation identity is invalid")]
    ReservationIntegrityMismatch,
    #[error("dispatch receipt timestamp does not match the resolution event")]
    ResolutionTimestampMismatch,
    #[error("revocation restriction is invalid")]
    InvalidRestriction,
    #[error("security event identity does not match its fields")]
    IntegrityMismatch,
    #[error(transparent)]
    BoundedText(#[from] codex_security_policy::BoundedTextError),
    #[error("failed to serialize canonical security event: {0}")]
    Serialization(serde_json::Error),
}
