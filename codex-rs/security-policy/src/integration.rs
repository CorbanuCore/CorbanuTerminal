//! Read-only integration facts, not policy decisions or human authority.
//!
//! Trusted host adapters produce these snapshots. Deserialization validates the
//! shape, not the sender: model/tool payloads must never become inspector truth.
//! The effective level is a policy floor, not a claim that its controls exist.

use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;
use thiserror::Error;

use crate::SecurityLevel;

pub const SECURITY_INSPECTOR_SCHEMA_VERSION: u32 = 1;

/// Fixed diagnostic categories keep backend errors, paths, URLs, and credentials
/// out of inspector/audit payloads. Detailed errors belong to a separate safe sink.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SecurityDegradationReason {
    BackendUnavailable,
    UnsupportedPlatform,
    PolicyMismatch,
    HealthCheckFailed,
    ResourceLimit,
}

/// Status of one stronger-mode control, independently of the selected level.
// Empty struct variants make Serde reject extra fields; internally tagged unit
// variants otherwise discard payload extensions even with deny_unknown_fields.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "state", rename_all = "snake_case", deny_unknown_fields)]
pub enum SecurityControlHealth {
    /// No implementation or no trustworthy observation. Never implies protection.
    Unavailable {},
    /// Implemented but not active for this policy context.
    Inactive {},
    /// The host has verified the control is active for this policy context.
    /// Process availability alone, or a content classifier verdict, is insufficient.
    Enforcing {},
    /// Protection cannot currently be claimed; consumers must not fall back to an
    /// unprotected path for an operation requiring this control.
    Degraded { reason: SecurityDegradationReason },
}

impl Default for SecurityControlHealth {
    fn default() -> Self {
        Self::Unavailable {}
    }
}

/// Independent facts: browser containment does not establish content trust, and
/// neither control implies confidentiality or protected-action enforcement.
/// Missing implementations start unavailable, including in stronger modes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SecurityControlHealthSnapshot {
    pub browser_isolation: SecurityControlHealth,
    pub content_firewall: SecurityControlHealth,
    pub confidentiality: SecurityControlHealth,
    pub protected_actions: SecurityControlHealth,
}

/// Secret-free inspector contract. No free-form payloads, grants, credential
/// references, source content, or authorization decisions are carried here.
///
/// `requested_level` is the committed human selection, not a pending UI preview.
/// `effective_level` includes inherited narrowing and must not be less strict.
/// Consumers display health independently: selecting Moderate/Aggressive does
/// not make unavailable controls enforcing. A snapshot never authorizes an action
/// and must not be used as an authority cache across policy changes or resume.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(try_from = "InspectorWire", deny_unknown_fields)]
pub struct SecurityInspectorSnapshot {
    #[schemars(range(min = 1, max = 1))]
    schema_version: u32,
    requested_level: SecurityLevel,
    effective_level: SecurityLevel,
    controls: SecurityControlHealthSnapshot,
}

impl SecurityInspectorSnapshot {
    pub fn new(
        requested_level: SecurityLevel,
        effective_level: SecurityLevel,
        controls: SecurityControlHealthSnapshot,
    ) -> Result<Self, SecurityInspectorError> {
        if effective_level < requested_level {
            return Err(SecurityInspectorError::WeakerEffectiveLevel);
        }
        // These are added stronger-mode controls, not existing Permissive policy.
        if effective_level == SecurityLevel::Permissive
            && [
                controls.browser_isolation,
                controls.content_firewall,
                controls.confidentiality,
                controls.protected_actions,
            ]
            .contains(&SecurityControlHealth::Enforcing {})
        {
            return Err(SecurityInspectorError::PermissiveEnforcementClaim);
        }
        Ok(Self {
            schema_version: SECURITY_INSPECTOR_SCHEMA_VERSION,
            requested_level,
            effective_level,
            controls,
        })
    }

    pub fn requested_level(&self) -> SecurityLevel {
        self.requested_level
    }

    pub fn effective_level(&self) -> SecurityLevel {
        self.effective_level
    }

    pub fn controls(&self) -> &SecurityControlHealthSnapshot {
        &self.controls
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct InspectorWire {
    schema_version: u32,
    requested_level: SecurityLevel,
    effective_level: SecurityLevel,
    controls: SecurityControlHealthSnapshot,
}

impl TryFrom<InspectorWire> for SecurityInspectorSnapshot {
    type Error = SecurityInspectorError;

    fn try_from(value: InspectorWire) -> Result<Self, Self::Error> {
        if value.schema_version != SECURITY_INSPECTOR_SCHEMA_VERSION {
            return Err(SecurityInspectorError::UnsupportedVersion);
        }
        Self::new(value.requested_level, value.effective_level, value.controls)
    }
}

#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum SecurityInspectorError {
    #[error("unsupported security inspector schema version")]
    UnsupportedVersion,
    #[error("effective security level cannot weaken the committed selection")]
    WeakerEffectiveLevel,
    #[error("Permissive cannot claim added stronger-mode enforcement")]
    PermissiveEnforcementClaim,
}
