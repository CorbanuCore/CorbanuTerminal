use std::fmt;

use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;
use thiserror::Error;

pub const SECURITY_SETTINGS_VERSION: u32 = 1;

/// User-facing Corbanu security posture.
///
/// Serialized spellings are a persistence contract.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    JsonSchema,
)]
#[serde(rename_all = "lowercase")]
pub enum SecurityLevel {
    /// Preserve all existing policy decisions exactly.
    #[default]
    Permissive,
    /// Add deterministic protections around untrusted content and protected actions.
    Moderate,
    /// Deny sensitive operations until covered by narrow human authority.
    Aggressive,
}

impl SecurityLevel {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Permissive => "permissive",
            Self::Moderate => "moderate",
            Self::Aggressive => "aggressive",
        }
    }
}

impl fmt::Display for SecurityLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Versioned persisted settings for the security posture.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SecuritySettings {
    pub version: u32,
    pub level: SecurityLevel,
}

impl SecuritySettings {
    pub const fn new(level: SecurityLevel) -> Self {
        Self {
            version: SECURITY_SETTINGS_VERSION,
            level,
        }
    }

    pub fn validate(&self) -> Result<(), SecuritySettingsError> {
        if self.version != SECURITY_SETTINGS_VERSION {
            return Err(SecuritySettingsError::UnsupportedVersion {
                found: self.version,
                supported: SECURITY_SETTINGS_VERSION,
            });
        }
        Ok(())
    }
}

impl Default for SecuritySettings {
    fn default() -> Self {
        Self::new(SecurityLevel::Permissive)
    }
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum SecuritySettingsError {
    #[error("unsupported security settings version {found}; supported version is {supported}")]
    UnsupportedVersion { found: u32, supported: u32 },
}
