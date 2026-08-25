use std::collections::BTreeSet;

use schemars::JsonSchema;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use thiserror::Error;

use crate::BoundedText;
use crate::digest::canonical_sha256;

pub const AUTHORIZATION_SCHEMA_VERSION: u32 = 1;
pub const MAX_ACTOR_CHAIN_LENGTH: usize = 16;

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum PrincipalKind {
    Human,
    Agent,
    Tool,
    Service,
}

/// A policy identity, never a credential or bearer token.
#[derive(
    Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct PolicyPrincipal {
    pub kind: PrincipalKind,
    pub id: BoundedText,
}

impl PolicyPrincipal {
    pub fn new(
        kind: PrincipalKind,
        id: impl Into<String>,
    ) -> Result<Self, crate::BoundedTextError> {
        Ok(Self {
            kind,
            id: BoundedText::new(id)?,
        })
    }
}

/// Ordered delegation path from the human authority to the current actor.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(transparent)]
#[schemars(transparent)]
pub struct ActorChain(Vec<PolicyPrincipal>);

impl ActorChain {
    pub fn new(actors: Vec<PolicyPrincipal>) -> Result<Self, AuthorizationError> {
        if actors.is_empty() {
            return Err(AuthorizationError::EmptyActorChain);
        }
        if actors.len() > MAX_ACTOR_CHAIN_LENGTH {
            return Err(AuthorizationError::ActorChainTooLong {
                actual: actors.len(),
                maximum: MAX_ACTOR_CHAIN_LENGTH,
            });
        }
        let unique = actors
            .iter()
            .map(|actor| &actor.id)
            .collect::<BTreeSet<_>>();
        if unique.len() != actors.len() {
            return Err(AuthorizationError::ActorChainCycle);
        }
        if actors
            .first()
            .is_none_or(|actor| actor.kind != PrincipalKind::Human)
        {
            return Err(AuthorizationError::ActorChainMustStartWithHuman);
        }
        Ok(Self(actors))
    }

    pub fn as_slice(&self) -> &[PolicyPrincipal] {
        &self.0
    }

    pub fn current_actor(&self) -> Option<&PolicyPrincipal> {
        self.0.last()
    }

    pub fn extends(&self, parent: &Self) -> bool {
        self.0.starts_with(&parent.0)
    }
}

impl<'de> Deserialize<'de> for ActorChain {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let actors = Vec::<PolicyPrincipal>::deserialize(deserializer)?;
        Self::new(actors).map_err(serde::de::Error::custom)
    }
}

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum ResourceKind {
    UntrustedContent,
    VaultCredential,
    ProtectedData,
    Tool,
    Account,
    NetworkDestination,
    Clipboard,
    Export,
    FinancialAction,
    SecurityPolicy,
    AgentSpawn,
}

#[derive(
    Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct ProtectedResource {
    pub kind: ResourceKind,
    pub id: BoundedText,
}

impl ProtectedResource {
    pub fn new(kind: ResourceKind, id: impl Into<String>) -> Result<Self, crate::BoundedTextError> {
        Ok(Self {
            kind,
            id: BoundedText::new(id)?,
        })
    }
}

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum PolicyAction {
    Inspect,
    Use,
    Reveal,
    Export,
    Execute,
    Connect,
    Spawn,
    ChangeSecurityLevel,
    CreateGrant,
    Revoke,
    Sign,
    Broadcast,
}

#[derive(
    Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct QuantitativeLimit {
    pub asset: BoundedText,
    pub max_units: u64,
}

impl QuantitativeLimit {
    pub fn new(asset: impl Into<String>, max_units: u64) -> Result<Self, AuthorizationError> {
        if max_units == 0 {
            return Err(AuthorizationError::ZeroQuantity);
        }
        Ok(Self {
            asset: BoundedText::new(asset)?,
            max_units,
        })
    }

    pub fn validate(&self) -> Result<(), AuthorizationError> {
        if self.max_units == 0 {
            return Err(AuthorizationError::ZeroQuantity);
        }
        Ok(())
    }
}

/// Explicit, bounded request context. It cannot carry arbitrary metadata.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AuthorizationContext {
    pub now_unix_seconds: i64,
    pub session_id: BoundedText,
    pub task_id: BoundedText,
    pub purpose: BoundedText,
    pub operation: BoundedText,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub destination: Option<BoundedText>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quantity: Option<QuantitativeLimit>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grant_id: Option<BoundedText>,
}

impl AuthorizationContext {
    pub fn validate(&self) -> Result<(), AuthorizationError> {
        if self.now_unix_seconds < 0 {
            return Err(AuthorizationError::NegativeTimestamp);
        }
        if let Some(quantity) = &self.quantity {
            quantity.validate()?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AuthorizationRequest {
    pub schema_version: u32,
    pub subject: ActorChain,
    pub resource: ProtectedResource,
    pub action: PolicyAction,
    pub context: AuthorizationContext,
}

impl AuthorizationRequest {
    pub fn new(
        subject: ActorChain,
        resource: ProtectedResource,
        action: PolicyAction,
        context: AuthorizationContext,
    ) -> Result<Self, AuthorizationError> {
        let request = Self {
            schema_version: AUTHORIZATION_SCHEMA_VERSION,
            subject,
            resource,
            action,
            context,
        };
        request.validate()?;
        Ok(request)
    }

    pub fn validate(&self) -> Result<(), AuthorizationError> {
        if self.schema_version != AUTHORIZATION_SCHEMA_VERSION {
            return Err(AuthorizationError::UnsupportedSchemaVersion {
                found: self.schema_version,
                supported: AUTHORIZATION_SCHEMA_VERSION,
            });
        }
        self.context.validate()
    }

    pub fn digest(&self) -> Result<String, AuthorizationError> {
        self.validate()?;
        canonical_sha256(self).map_err(AuthorizationError::Serialization)
    }
}

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum AuthorizationEffect {
    Allow,
    Deny,
}

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum DecisionReason {
    PermissiveCompatibility,
    MatchingGrant,
    MissingGrant,
    ExpiredGrant,
    ScopeMismatch,
    Revoked,
    KillSwitch,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AuthorizationDecision {
    pub effect: AuthorizationEffect,
    pub reason: DecisionReason,
    pub request_digest: BoundedText,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub matched_grant_id: Option<BoundedText>,
}

impl AuthorizationDecision {
    pub fn allows(&self) -> bool {
        self.effect == AuthorizationEffect::Allow
    }
}

/// Return the security-layer decision for Permissive.
///
/// The result means "add no new denial" and must still be composed with the
/// existing policy decision.
pub fn permissive_decision(
    request: &AuthorizationRequest,
) -> Result<AuthorizationDecision, AuthorizationError> {
    Ok(AuthorizationDecision {
        effect: AuthorizationEffect::Allow,
        reason: DecisionReason::PermissiveCompatibility,
        request_digest: BoundedText::new(request.digest()?)?,
        matched_grant_id: None,
    })
}

/// Compose this additional layer without allowing it to bypass an existing deny.
pub fn compose_existing_decision(
    existing_allow: bool,
    security_decision: &AuthorizationDecision,
) -> bool {
    existing_allow && security_decision.allows()
}

#[derive(Debug, Error)]
pub enum AuthorizationError {
    #[error("actor chain must not be empty")]
    EmptyActorChain,
    #[error("actor chain has {actual} actors; maximum is {maximum}")]
    ActorChainTooLong { actual: usize, maximum: usize },
    #[error("actor chain contains a cycle or duplicate actor")]
    ActorChainCycle,
    #[error("actor chain must begin with a human authority")]
    ActorChainMustStartWithHuman,
    #[error("timestamps must be non-negative")]
    NegativeTimestamp,
    #[error("quantitative limits must be greater than zero")]
    ZeroQuantity,
    #[error("unsupported authorization schema version {found}; supported version is {supported}")]
    UnsupportedSchemaVersion { found: u32, supported: u32 },
    #[error(transparent)]
    BoundedText(#[from] crate::BoundedTextError),
    #[error("failed to serialize canonical policy object: {0}")]
    Serialization(serde_json::Error),
}
