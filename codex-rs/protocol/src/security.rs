//! Corbanu side-channel contracts. Do not embed these in provider tool schemas,
//! model messages, or native agent lifecycle payloads. Requests carry intent, not
//! human authentication; only the separately held Core controller can confirm.

use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;
use thiserror::Error;

pub use codex_security_policy::ActorChain;
pub use codex_security_policy::AuthorityEpoch;
pub use codex_security_policy::GrantScope;
pub use codex_security_policy::RevocationReason;
pub use codex_security_policy::RevocationTarget;
pub use codex_security_policy::SecurityInspectorSnapshot;
pub use codex_security_policy::SecurityLevel;

/// Observation from Core's current policy context. No command or authorization.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SecurityInspectorEvent {
    pub snapshot: SecurityInspectorSnapshot,
    pub epoch: AuthorityEpoch,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum SecurityControlAction {
    SetLevel {
        level: SecurityLevel,
    },
    CreateGrant {
        actor_chain: ActorChain,
        scope: GrantScope,
        expires_at_unix_seconds: i64,
    },
    Revoke {
        target: RevocationTarget,
        reason: RevocationReason,
    },
}

/// A proposal from the trusted UI's future confirmation flow. Wire callers cannot
/// supply an issuer, authenticated flag, or executable approval token. No existing
/// Op/EventMsg route accepts this request; PF-24/25 own interactive registration.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(try_from = "RequestWire", deny_unknown_fields)]
pub struct SecurityControlRequest {
    #[schemars(range(min = 1, max = 1))]
    schema_version: u32,
    expected_epoch: AuthorityEpoch,
    action: SecurityControlAction,
}

impl SecurityControlRequest {
    pub fn new(
        expected_epoch: AuthorityEpoch,
        action: SecurityControlAction,
    ) -> Result<Self, SecurityRequestError> {
        if let SecurityControlAction::CreateGrant {
            scope,
            expires_at_unix_seconds,
            ..
        } = &action
        {
            scope
                .validate()
                .map_err(|_| SecurityRequestError::InvalidGrant)?;
            if *expires_at_unix_seconds < 0 {
                return Err(SecurityRequestError::InvalidGrant);
            }
        }
        Ok(Self {
            schema_version: 1,
            expected_epoch,
            action,
        })
    }

    pub fn expected_epoch(&self) -> AuthorityEpoch {
        self.expected_epoch
    }

    pub fn action(&self) -> &SecurityControlAction {
        &self.action
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RequestWire {
    schema_version: u32,
    expected_epoch: AuthorityEpoch,
    action: SecurityControlAction,
}

impl TryFrom<RequestWire> for SecurityControlRequest {
    type Error = SecurityRequestError;

    fn try_from(value: RequestWire) -> Result<Self, Self::Error> {
        if value.schema_version != 1 {
            return Err(SecurityRequestError::UnsupportedVersion);
        }
        Self::new(value.expected_epoch, value.action)
    }
}

#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum SecurityRequestError {
    #[error("unsupported security request version")]
    UnsupportedVersion,
    #[error("invalid grant request")]
    InvalidGrant,
}

#[cfg(test)]
#[path = "security_tests.rs"]
mod tests;
