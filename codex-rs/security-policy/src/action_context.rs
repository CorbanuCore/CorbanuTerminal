//! Immutable post-read action binding. These contracts add checks, never replace
//! native permission decisions or establish authority from a deserialized grant.

use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;
use thiserror::Error;

use crate::AuthorizationRequest;
use crate::BoundedGrant;
use crate::RevocationState;
use crate::TaintContext;
use crate::digest::canonical_sha256;

/// A revision is meaningful only within one fresh host runtime incarnation.
/// Resume/restart must generate a new nonce, even when persisted revisions match.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(try_from = "EpochWire", deny_unknown_fields)]
pub struct AuthorityEpoch {
    runtime_nonce: [u8; 16],
    policy_revision: u64,
    revocation_generation: u64,
}

impl AuthorityEpoch {
    pub fn new(
        runtime_nonce: [u8; 16],
        policy_revision: u64,
        revocation_generation: u64,
    ) -> Result<Self, ActionContextError> {
        if runtime_nonce == [0; 16] {
            return Err(ActionContextError::InvalidEpoch);
        }
        Ok(Self {
            runtime_nonce,
            policy_revision,
            revocation_generation,
        })
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EpochWire {
    runtime_nonce: [u8; 16],
    policy_revision: u64,
    revocation_generation: u64,
}

impl TryFrom<EpochWire> for AuthorityEpoch {
    type Error = ActionContextError;

    fn try_from(value: EpochWire) -> Result<Self, Self::Error> {
        Self::new(
            value.runtime_nonce,
            value.policy_revision,
            value.revocation_generation,
        )
    }
}

/// Host-bound request plus every ancestor and the current authority revision.
/// The request cannot be edited after binding. A wire round-trip validates shape,
/// not origin: dispatch must compare against live host identity, taint, and epoch.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(try_from = "ActionWire", deny_unknown_fields)]
pub struct ActionContext {
    #[schemars(range(min = 1, max = 1))]
    schema_version: u32,
    request: AuthorizationRequest,
    taint: TaintContext,
    epoch: AuthorityEpoch,
}

impl ActionContext {
    pub fn new(
        request: AuthorizationRequest,
        taint: TaintContext,
        epoch: AuthorityEpoch,
    ) -> Result<Self, ActionContextError> {
        request
            .validate()
            .map_err(|_| ActionContextError::InvalidRequest)?;
        Ok(Self {
            schema_version: 1,
            request,
            taint,
            epoch,
        })
    }

    pub fn request(&self) -> &AuthorizationRequest {
        &self.request
    }

    pub fn epoch(&self) -> AuthorityEpoch {
        self.epoch
    }

    pub fn digest(&self) -> Result<String, ActionContextError> {
        canonical_sha256(self).map_err(|_| ActionContextError::InvalidRequest)
    }

    /// This check and dispatch must share the caller's authority/context guard.
    /// New reads invalidate pending approval even if they appear harmless.
    pub fn validate_current(
        &self,
        epoch: AuthorityEpoch,
        taint: &TaintContext,
    ) -> Result<(), ActionContextError> {
        if self.epoch != epoch {
            return Err(ActionContextError::StaleAuthority);
        }
        if &self.taint != taint {
            return Err(ActionContextError::StaleTaint);
        }
        if self.taint.has_unknown_origin() {
            return Err(ActionContextError::UnknownOrigin);
        }
        Ok(())
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ActionWire {
    schema_version: u32,
    request: AuthorizationRequest,
    taint: TaintContext,
    epoch: AuthorityEpoch,
}

impl TryFrom<ActionWire> for ActionContext {
    type Error = ActionContextError;

    fn try_from(value: ActionWire) -> Result<Self, Self::Error> {
        if value.schema_version != 1 {
            return Err(ActionContextError::UnsupportedVersion);
        }
        Self::new(value.request, value.taint, value.epoch)
    }
}

/// An existing grant bound by the host to its issuance epoch. This is deliberately
/// not deserializable: a payload cannot claim an old grant was freshly issued.
#[derive(Clone, Debug)]
pub struct EpochBoundGrant {
    grant: BoundedGrant,
    epoch: AuthorityEpoch,
}

impl EpochBoundGrant {
    /// Trusted issuers call this only at issuance, never to refresh a stored or
    /// wire-supplied grant into a new epoch. Shape validation is not issuance.
    pub fn bind(grant: BoundedGrant, epoch: AuthorityEpoch) -> Result<Self, ActionContextError> {
        grant
            .validate()
            .map_err(|_| ActionContextError::InvalidGrant)?;
        Ok(Self { grant, epoch })
    }

    /// Host time is checked separately from the immutable proposal timestamp.
    /// Success means only that this grant matches; existing denials still win.
    pub fn validate_at(
        &self,
        action: &ActionContext,
        epoch: AuthorityEpoch,
        taint: &TaintContext,
        revocations: &RevocationState,
        now_unix_seconds: i64,
    ) -> Result<(), ActionContextError> {
        action.validate_current(epoch, taint)?;
        revocations
            .validate()
            .map_err(|_| ActionContextError::InvalidRevocations)?;
        if self.epoch != epoch || revocations.generation != epoch.revocation_generation {
            return Err(ActionContextError::StaleAuthority);
        }
        if revocations.grant_is_revoked(&self.grant) {
            return Err(ActionContextError::Revoked);
        }
        if now_unix_seconds < action.request.context.now_unix_seconds
            || now_unix_seconds < self.grant.issued_at_unix_seconds
            || self.grant.is_expired_at(now_unix_seconds)
        {
            return Err(ActionContextError::ExpiredOrFuture);
        }
        if !self
            .grant
            .matches_request(&action.request)
            .map_err(|_| ActionContextError::InvalidGrant)?
        {
            return Err(ActionContextError::GrantMismatch);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum ActionContextError {
    #[error("unsupported action context schema version")]
    UnsupportedVersion,
    #[error("authority epoch requires a fresh nonzero runtime nonce")]
    InvalidEpoch,
    #[error("invalid authorization request")]
    InvalidRequest,
    #[error("invalid grant")]
    InvalidGrant,
    #[error("invalid revocation state")]
    InvalidRevocations,
    #[error("authority changed; obtain fresh authorization")]
    StaleAuthority,
    #[error("context changed after the action was proposed")]
    StaleTaint,
    #[error("unknown provenance cannot authorize protected use")]
    UnknownOrigin,
    #[error("grant is revoked")]
    Revoked,
    #[error("grant expired or timestamp is in the future")]
    ExpiredOrFuture,
    #[error("grant does not match the exact request")]
    GrantMismatch,
}
