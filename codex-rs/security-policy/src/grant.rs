use std::collections::BTreeMap;
use std::collections::BTreeSet;

use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;
use thiserror::Error;

use crate::ActorChain;
use crate::AuthorizationRequest;
use crate::BoundedText;
use crate::PolicyAction;
use crate::PolicyPrincipal;
use crate::PrincipalKind;
use crate::ProtectedResource;
use crate::digest::canonical_sha256;

pub const GRANT_SCHEMA_VERSION: u32 = 2;
pub const MAX_GRANT_ACTIONS: usize = 16;
pub const MAX_GRANT_LIMITS: usize = 16;

/// Exact request context to which a temporary grant applies.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GrantContext {
    pub session_id: BoundedText,
    pub task_id: BoundedText,
    pub purpose: BoundedText,
    pub operation: BoundedText,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<BoundedText>,
}

impl GrantContext {
    pub fn new(
        session_id: BoundedText,
        task_id: BoundedText,
        purpose: BoundedText,
        operation: BoundedText,
    ) -> Self {
        Self {
            session_id,
            task_id,
            purpose,
            operation,
            model: None,
        }
    }

    pub fn with_model(mut self, model: BoundedText) -> Self {
        self.model = Some(model);
        self
    }
}

/// Exact authority carried by a temporary grant.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GrantScope {
    pub resource: ProtectedResource,
    pub actions: BTreeSet<PolicyAction>,
    pub context: GrantContext,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub destination: Option<BoundedText>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub quantitative_limits: BTreeMap<BoundedText, u64>,
}

impl GrantScope {
    pub fn new(
        resource: ProtectedResource,
        actions: impl IntoIterator<Item = PolicyAction>,
        context: GrantContext,
        destination: Option<BoundedText>,
        quantitative_limits: BTreeMap<BoundedText, u64>,
    ) -> Result<Self, GrantValidationError> {
        let scope = Self {
            resource,
            actions: actions.into_iter().collect(),
            context,
            destination,
            quantitative_limits,
        };
        scope.validate()?;
        Ok(scope)
    }

    pub fn validate(&self) -> Result<(), GrantValidationError> {
        if self.actions.is_empty() {
            return Err(GrantValidationError::EmptyActions);
        }
        if self.actions.len() > MAX_GRANT_ACTIONS {
            return Err(GrantValidationError::TooManyActions {
                actual: self.actions.len(),
                maximum: MAX_GRANT_ACTIONS,
            });
        }
        if self.quantitative_limits.len() > MAX_GRANT_LIMITS {
            return Err(GrantValidationError::TooManyLimits {
                actual: self.quantitative_limits.len(),
                maximum: MAX_GRANT_LIMITS,
            });
        }
        if self.quantitative_limits.values().any(|limit| *limit == 0) {
            return Err(GrantValidationError::ZeroLimit);
        }
        Ok(())
    }

    /// Whether this scope grants no more authority than `parent`.
    pub fn is_narrower_or_equal(&self, parent: &Self) -> bool {
        if self.resource != parent.resource
            || !self.actions.is_subset(&parent.actions)
            || self.context != parent.context
        {
            return false;
        }
        if let Some(parent_destination) = &parent.destination
            && self.destination.as_ref() != Some(parent_destination)
        {
            return false;
        }
        if parent.quantitative_limits.is_empty() {
            return true;
        }
        !self.quantitative_limits.is_empty()
            && self.quantitative_limits.iter().all(|(asset, child_max)| {
                parent
                    .quantitative_limits
                    .get(asset)
                    .is_some_and(|parent_max| child_max <= parent_max)
            })
    }

    fn matches_request(&self, request: &AuthorizationRequest) -> bool {
        if self.resource != request.resource
            || !self.actions.contains(&request.action)
            || self.context.session_id != request.context.session_id
            || self.context.task_id != request.context.task_id
            || self.context.purpose != request.context.purpose
            || self.context.operation != request.context.operation
        {
            return false;
        }
        if let Some(destination) = &self.destination
            && request.context.destination.as_ref() != Some(destination)
        {
            return false;
        }
        if self.quantitative_limits.is_empty() {
            return true;
        }
        let Some(quantity) = &request.context.quantity else {
            return false;
        };
        self.quantitative_limits
            .get(&quantity.asset)
            .is_some_and(|max_units| quantity.max_units <= *max_units)
    }
}

/// A human-issued, expiring, integrity-bound grant.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BoundedGrant {
    pub schema_version: u32,
    pub grant_id: BoundedText,
    pub issuer: PolicyPrincipal,
    pub actor_chain: ActorChain,
    pub scope: GrantScope,
    pub issued_at_unix_seconds: i64,
    pub expires_at_unix_seconds: i64,
    pub nonce: BoundedText,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_grant_id: Option<BoundedText>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_scope_digest: Option<BoundedText>,
}

#[derive(Serialize)]
struct GrantBinding<'a> {
    schema_version: u32,
    issuer: &'a PolicyPrincipal,
    actor_chain: &'a ActorChain,
    scope: &'a GrantScope,
    issued_at_unix_seconds: i64,
    expires_at_unix_seconds: i64,
    nonce: &'a BoundedText,
    parent_grant_id: &'a Option<BoundedText>,
    parent_scope_digest: &'a Option<BoundedText>,
}

impl BoundedGrant {
    pub fn issue(
        issuer: PolicyPrincipal,
        actor_chain: ActorChain,
        scope: GrantScope,
        issued_at_unix_seconds: i64,
        expires_at_unix_seconds: i64,
        nonce: BoundedText,
    ) -> Result<Self, GrantValidationError> {
        Self::build(
            issuer,
            actor_chain,
            scope,
            issued_at_unix_seconds,
            expires_at_unix_seconds,
            nonce,
            None,
            None,
        )
    }

    pub fn derive_child(
        parent: &Self,
        actor_chain: ActorChain,
        scope: GrantScope,
        issued_at_unix_seconds: i64,
        expires_at_unix_seconds: i64,
        nonce: BoundedText,
    ) -> Result<Self, GrantValidationError> {
        parent.validate()?;
        if !actor_chain.extends(&parent.actor_chain) {
            return Err(GrantValidationError::ActorChainNotDescendant);
        }
        if !scope.is_narrower_or_equal(&parent.scope) {
            return Err(GrantValidationError::ScopeNotNarrower);
        }
        if issued_at_unix_seconds < parent.issued_at_unix_seconds {
            return Err(GrantValidationError::IssuedBeforeParent);
        }
        if expires_at_unix_seconds > parent.expires_at_unix_seconds {
            return Err(GrantValidationError::ExpiryNotNarrower);
        }
        let scope_digest = BoundedText::new(canonical_sha256(&parent.scope)?)?;
        Self::build(
            parent.issuer.clone(),
            actor_chain,
            scope,
            issued_at_unix_seconds,
            expires_at_unix_seconds,
            nonce,
            Some(parent.grant_id.clone()),
            Some(scope_digest),
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn build(
        issuer: PolicyPrincipal,
        actor_chain: ActorChain,
        scope: GrantScope,
        issued_at_unix_seconds: i64,
        expires_at_unix_seconds: i64,
        nonce: BoundedText,
        parent_grant_id: Option<BoundedText>,
        parent_scope_digest: Option<BoundedText>,
    ) -> Result<Self, GrantValidationError> {
        let mut grant = Self {
            schema_version: GRANT_SCHEMA_VERSION,
            grant_id: BoundedText::new("pending")?,
            issuer,
            actor_chain,
            scope,
            issued_at_unix_seconds,
            expires_at_unix_seconds,
            nonce,
            parent_grant_id,
            parent_scope_digest,
        };
        grant.validate_fields()?;
        grant.grant_id = BoundedText::new(grant.expected_id()?)?;
        Ok(grant)
    }

    pub fn validate(&self) -> Result<(), GrantValidationError> {
        self.validate_fields()?;
        if self.grant_id.as_str() != self.expected_id()? {
            return Err(GrantValidationError::IntegrityMismatch);
        }
        Ok(())
    }

    fn validate_fields(&self) -> Result<(), GrantValidationError> {
        if self.schema_version != GRANT_SCHEMA_VERSION {
            return Err(GrantValidationError::UnsupportedSchemaVersion {
                found: self.schema_version,
                supported: GRANT_SCHEMA_VERSION,
            });
        }
        if self.issuer.kind != PrincipalKind::Human {
            return Err(GrantValidationError::IssuerMustBeHuman);
        }
        if self.actor_chain.as_slice().first() != Some(&self.issuer) {
            return Err(GrantValidationError::IssuerDoesNotOwnActorChain);
        }
        if self.issued_at_unix_seconds < 0 || self.expires_at_unix_seconds < 0 {
            return Err(GrantValidationError::NegativeTimestamp);
        }
        if self.expires_at_unix_seconds <= self.issued_at_unix_seconds {
            return Err(GrantValidationError::InvalidExpiry);
        }
        if self.parent_grant_id.is_some() != self.parent_scope_digest.is_some() {
            return Err(GrantValidationError::IncompleteParentBinding);
        }
        self.scope.validate()
    }

    pub fn is_expired_at(&self, now_unix_seconds: i64) -> bool {
        now_unix_seconds >= self.expires_at_unix_seconds
    }

    pub fn matches_request(
        &self,
        request: &AuthorizationRequest,
    ) -> Result<bool, GrantValidationError> {
        self.validate()?;
        request
            .validate()
            .map_err(GrantValidationError::Authorization)?;
        if request.context.now_unix_seconds < self.issued_at_unix_seconds
            || self.is_expired_at(request.context.now_unix_seconds)
        {
            return Ok(false);
        }
        if self.actor_chain != request.subject || !self.scope.matches_request(request) {
            return Ok(false);
        }
        if let Some(requested_grant_id) = &request.context.grant_id
            && requested_grant_id != &self.grant_id
        {
            return Ok(false);
        }
        Ok(true)
    }

    fn expected_id(&self) -> Result<String, GrantValidationError> {
        canonical_sha256(&GrantBinding {
            schema_version: self.schema_version,
            issuer: &self.issuer,
            actor_chain: &self.actor_chain,
            scope: &self.scope,
            issued_at_unix_seconds: self.issued_at_unix_seconds,
            expires_at_unix_seconds: self.expires_at_unix_seconds,
            nonce: &self.nonce,
            parent_grant_id: &self.parent_grant_id,
            parent_scope_digest: &self.parent_scope_digest,
        })
        .map_err(GrantValidationError::Serialization)
    }
}

#[derive(Debug, Error)]
pub enum GrantValidationError {
    #[error("grant must contain at least one action")]
    EmptyActions,
    #[error("grant has {actual} actions; maximum is {maximum}")]
    TooManyActions { actual: usize, maximum: usize },
    #[error("grant has {actual} quantitative limits; maximum is {maximum}")]
    TooManyLimits { actual: usize, maximum: usize },
    #[error("grant quantitative limits must be greater than zero")]
    ZeroLimit,
    #[error("grant issuer must be a human")]
    IssuerMustBeHuman,
    #[error("grant issuer must be the first actor in the actor chain")]
    IssuerDoesNotOwnActorChain,
    #[error("grant timestamps must be non-negative")]
    NegativeTimestamp,
    #[error("grant expiry must be later than issuance")]
    InvalidExpiry,
    #[error("derived grant actor chain must extend the parent actor chain")]
    ActorChainNotDescendant,
    #[error("derived grant scope must be equal to or narrower than its parent")]
    ScopeNotNarrower,
    #[error("derived grant cannot be issued before its parent")]
    IssuedBeforeParent,
    #[error("derived grant cannot expire later than its parent")]
    ExpiryNotNarrower,
    #[error("parent grant id and parent scope digest must both be present or absent")]
    IncompleteParentBinding,
    #[error("grant integrity digest does not match its bound fields")]
    IntegrityMismatch,
    #[error("unsupported grant schema version {found}; supported version is {supported}")]
    UnsupportedSchemaVersion { found: u32, supported: u32 },
    #[error(transparent)]
    BoundedText(#[from] crate::BoundedTextError),
    #[error(transparent)]
    Authorization(#[from] crate::AuthorizationError),
    #[error("failed to serialize canonical grant: {0}")]
    Serialization(#[from] serde_json::Error),
}
