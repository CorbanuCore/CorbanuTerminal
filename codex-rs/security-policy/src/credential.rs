use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;
use thiserror::Error;

use crate::ActionReceipt;
use crate::ActorChain;
use crate::AuthorizationDecision;
use crate::AuthorizationEffect;
use crate::AuthorizationError;
use crate::AuthorizationRequest;
use crate::BoundedGrant;
use crate::BoundedText;
use crate::BoundedTextError;
use crate::DecisionReason;
use crate::GrantValidationError;
use crate::PolicyAction;
use crate::PrincipalKind;
use crate::ResourceKind;
use crate::RevocationError;
use crate::RevocationState;
use crate::digest::canonical_sha256;

pub const CREDENTIAL_CAPABILITY_SCHEMA_VERSION: u32 = 1;
pub const CREDENTIAL_USAGE_SCHEMA_VERSION: u32 = 1;
pub const CREDENTIAL_USAGE_MAX_REQUESTS: u64 = 1_024;
pub const CAPABILITY_ID_HEX_LENGTH: usize = 64;
const CREDENTIAL_USAGE_DIMENSIONS: [&str; 4] = ["requests", "tokens", "bytes", "spend_microunits"];

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum CredentialHttpMethod {
    Delete,
    Get,
    Head,
    Patch,
    Post,
    Put,
}

impl CredentialHttpMethod {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Delete => "DELETE",
            Self::Get => "GET",
            Self::Head => "HEAD",
            Self::Patch => "PATCH",
            Self::Post => "POST",
            Self::Put => "PUT",
        }
    }
}

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum CredentialTransport {
    Https,
}

/// Canonical network authority. Host names are stored as lowercase DNS names;
/// IP literals, trailing dots, userinfo, paths, queries, and fragments cannot
/// enter this type.
#[derive(
    Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct CredentialDestination {
    pub transport: CredentialTransport,
    pub host: BoundedText,
    pub port: u16,
}

impl CredentialDestination {
    pub fn https(host: impl Into<String>, port: u16) -> Result<Self, CredentialCapabilityError> {
        let destination = Self {
            transport: CredentialTransport::Https,
            host: BoundedText::new(host.into().to_ascii_lowercase())?,
            port,
        };
        destination.validate()?;
        Ok(destination)
    }

    pub fn validate(&self) -> Result<(), CredentialCapabilityError> {
        if self.port == 0 {
            return Err(CredentialCapabilityError::InvalidDestinationPort);
        }
        validate_dns_host(self.host.as_str())?;
        if self
            .host
            .as_str()
            .bytes()
            .any(|byte| byte.is_ascii_uppercase())
        {
            return Err(CredentialCapabilityError::NonCanonicalDestinationHost);
        }
        Ok(())
    }

    pub fn authority(&self) -> Result<BoundedText, CredentialCapabilityError> {
        self.validate()?;
        let scheme = match self.transport {
            CredentialTransport::Https => "https",
        };
        Ok(BoundedText::new(format!(
            "{scheme}://{}:{}",
            self.host, self.port
        ))?)
    }
}

/// Secret-free vault reference. Both values are identifiers, not credential
/// payload fields, and use a deliberately narrow identifier alphabet.
#[derive(
    Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct CredentialReference {
    pub label: BoundedText,
    pub scope: BoundedText,
}

impl CredentialReference {
    pub fn new(
        label: impl Into<String>,
        scope: impl Into<String>,
    ) -> Result<Self, CredentialCapabilityError> {
        let reference = Self {
            label: BoundedText::new(label)?,
            scope: BoundedText::new(scope)?,
        };
        reference.validate()?;
        Ok(reference)
    }

    pub fn validate(&self) -> Result<(), CredentialCapabilityError> {
        validate_identifier("credential label", self.label.as_str())?;
        validate_identifier("credential scope", self.scope.as_str())
    }
}

/// Fully bound, secret-free authority requested before trusted credential
/// resolution. It reuses the common authorization, grant, revocation, and
/// receipt contracts rather than creating parallel policy primitives.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CredentialCapabilityRequest {
    pub schema_version: u32,
    pub authorization: AuthorizationRequest,
    pub grant: BoundedGrant,
    pub credential: CredentialReference,
    pub method: CredentialHttpMethod,
    pub destination: CredentialDestination,
    pub path: BoundedText,
    pub issued_at_unix_seconds: i64,
    pub expires_at_unix_seconds: i64,
    pub revocation_generation: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub triggering_receipt: Option<ActionReceipt>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage_schema_version: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<BoundedText>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub per_request_usage_limits: BTreeMap<BoundedText, u64>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub aggregate_usage_limits: BTreeMap<BoundedText, u64>,
}

impl CredentialCapabilityRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        authorization: AuthorizationRequest,
        grant: BoundedGrant,
        credential: CredentialReference,
        method: CredentialHttpMethod,
        destination: CredentialDestination,
        path: impl Into<String>,
        issued_at_unix_seconds: i64,
        expires_at_unix_seconds: i64,
        revocations: &RevocationState,
        triggering_receipt: Option<ActionReceipt>,
    ) -> Result<Self, CredentialCapabilityError> {
        let request = Self {
            schema_version: CREDENTIAL_CAPABILITY_SCHEMA_VERSION,
            authorization,
            grant,
            credential,
            method,
            destination,
            path: BoundedText::new(path)?,
            issued_at_unix_seconds,
            expires_at_unix_seconds,
            revocation_generation: revocations.generation,
            triggering_receipt,
            usage_schema_version: None,
            model: None,
            per_request_usage_limits: BTreeMap::new(),
            aggregate_usage_limits: BTreeMap::new(),
        };
        request.validate_at(issued_at_unix_seconds, revocations)?;
        Ok(request)
    }

    pub fn actor_chain(&self) -> &ActorChain {
        &self.authorization.subject
    }

    /// Construct the optional PF-13 usage contract without changing legacy
    /// one-shot capability behavior. Every limit and the exact model must
    /// already be integrity-bound by the existing `BoundedGrant`.
    #[allow(clippy::too_many_arguments)]
    pub fn new_with_usage_limits(
        authorization: AuthorizationRequest,
        grant: BoundedGrant,
        credential: CredentialReference,
        method: CredentialHttpMethod,
        destination: CredentialDestination,
        path: impl Into<String>,
        issued_at_unix_seconds: i64,
        expires_at_unix_seconds: i64,
        revocations: &RevocationState,
        triggering_receipt: Option<ActionReceipt>,
        model: impl Into<String>,
        per_request_usage_limits: BTreeMap<BoundedText, u64>,
        aggregate_usage_limits: BTreeMap<BoundedText, u64>,
    ) -> Result<Self, CredentialCapabilityError> {
        let request = Self {
            schema_version: CREDENTIAL_CAPABILITY_SCHEMA_VERSION,
            authorization,
            grant,
            credential,
            method,
            destination,
            path: BoundedText::new(path)?,
            issued_at_unix_seconds,
            expires_at_unix_seconds,
            revocation_generation: revocations.generation,
            triggering_receipt,
            usage_schema_version: Some(CREDENTIAL_USAGE_SCHEMA_VERSION),
            model: Some(BoundedText::new(model)?),
            per_request_usage_limits,
            aggregate_usage_limits,
        };
        request.validate_at(issued_at_unix_seconds, revocations)?;
        Ok(request)
    }

    pub fn has_usage_limits(&self) -> bool {
        self.usage_schema_version.is_some()
    }

    pub fn per_request_usage_limit(&self, dimension: &str) -> Option<u64> {
        usage_limit(&self.per_request_usage_limits, dimension)
    }

    pub fn aggregate_usage_limit(&self, dimension: &str) -> Option<u64> {
        usage_limit(&self.aggregate_usage_limits, dimension)
    }

    pub fn validate(&self) -> Result<(), CredentialCapabilityError> {
        if self.schema_version != CREDENTIAL_CAPABILITY_SCHEMA_VERSION {
            return Err(CredentialCapabilityError::UnsupportedSchemaVersion {
                found: self.schema_version,
                supported: CREDENTIAL_CAPABILITY_SCHEMA_VERSION,
            });
        }
        self.authorization.validate()?;
        self.grant.validate()?;
        self.credential.validate()?;
        self.destination.validate()?;
        validate_origin_path(self.path.as_str())?;
        if self.issued_at_unix_seconds < 0 || self.expires_at_unix_seconds < 0 {
            return Err(CredentialCapabilityError::NegativeTimestamp);
        }
        if self.expires_at_unix_seconds <= self.issued_at_unix_seconds {
            return Err(CredentialCapabilityError::InvalidExpiry);
        }
        if self.authorization.context.now_unix_seconds != self.issued_at_unix_seconds {
            return Err(CredentialCapabilityError::AuthorizationTimeMismatch);
        }
        if self.authorization.action != PolicyAction::Use
            || self.authorization.resource.kind != ResourceKind::VaultCredential
            || self.authorization.resource.id != self.credential.label
        {
            return Err(CredentialCapabilityError::CredentialAuthorityMismatch);
        }
        if self
            .authorization
            .subject
            .current_actor()
            .is_none_or(|actor| actor.kind != PrincipalKind::Agent)
        {
            return Err(CredentialCapabilityError::AgentActorRequired);
        }
        if self.authorization.context.destination.as_ref() != Some(&self.destination.authority()?) {
            return Err(CredentialCapabilityError::DestinationMismatch);
        }
        if self.authorization.context.operation != self.credential.scope {
            return Err(CredentialCapabilityError::CredentialScopeMismatch);
        }
        if self.issued_at_unix_seconds < self.grant.issued_at_unix_seconds
            || self.expires_at_unix_seconds > self.grant.expires_at_unix_seconds
            || !self.grant.matches_request(&self.authorization)?
        {
            return Err(CredentialCapabilityError::GrantMismatch);
        }
        if let Some(receipt) = &self.triggering_receipt {
            receipt.validate()?;
        }
        self.validate_usage_limits()?;
        Ok(())
    }

    fn validate_usage_limits(&self) -> Result<(), CredentialCapabilityError> {
        let grant_usage_limit_count = self
            .grant
            .scope
            .quantitative_limits
            .keys()
            .filter(|key| is_credential_usage_key(key.as_str()))
            .count();
        if self.usage_schema_version.is_none()
            && self.model.is_none()
            && self.per_request_usage_limits.is_empty()
            && self.aggregate_usage_limits.is_empty()
        {
            return if grant_usage_limit_count != 0 {
                Err(CredentialCapabilityError::GrantMismatch)
            } else {
                Ok(())
            };
        }
        if self.usage_schema_version != Some(CREDENTIAL_USAGE_SCHEMA_VERSION) {
            return Err(CredentialCapabilityError::UnsupportedSchemaVersion {
                found: self.usage_schema_version.unwrap_or(0),
                supported: CREDENTIAL_USAGE_SCHEMA_VERSION,
            });
        }
        let model = self
            .model
            .as_ref()
            .ok_or(CredentialCapabilityError::GrantMismatch)?;
        validate_identifier("credential model", model.as_str())?;
        if self.grant.scope.context.model.as_ref() != Some(model) {
            return Err(CredentialCapabilityError::GrantMismatch);
        }
        if self.per_request_usage_limits.len() != CREDENTIAL_USAGE_DIMENSIONS.len()
            || self.aggregate_usage_limits.len() != CREDENTIAL_USAGE_DIMENSIONS.len()
            || grant_usage_limit_count != CREDENTIAL_USAGE_DIMENSIONS.len() * 2
        {
            return Err(CredentialCapabilityError::GrantMismatch);
        }
        for dimension in CREDENTIAL_USAGE_DIMENSIONS {
            let per_request = self
                .per_request_usage_limit(dimension)
                .ok_or(CredentialCapabilityError::GrantMismatch)?;
            let aggregate = self
                .aggregate_usage_limit(dimension)
                .ok_or(CredentialCapabilityError::GrantMismatch)?;
            if per_request == 0 || aggregate == 0 {
                return Err(CredentialCapabilityError::GrantMismatch);
            }
            if per_request > aggregate {
                return Err(CredentialCapabilityError::GrantMismatch);
            }
            for (scope, expected) in [("per_request", per_request), ("aggregate", aggregate)] {
                let grant_key = format!("credential.{scope}.{dimension}");
                if usage_limit(&self.grant.scope.quantitative_limits, &grant_key) != Some(expected)
                {
                    return Err(CredentialCapabilityError::GrantMismatch);
                }
            }
        }
        if self.per_request_usage_limit("requests") != Some(1) {
            return Err(CredentialCapabilityError::GrantMismatch);
        }
        let expected_requests = self.aggregate_usage_limit("requests");
        if expected_requests.is_some_and(|requests| requests > CREDENTIAL_USAGE_MAX_REQUESTS) {
            return Err(CredentialCapabilityError::GrantMismatch);
        }
        if self
            .authorization
            .context
            .quantity
            .as_ref()
            .map(|quantity| (quantity.asset.as_str(), quantity.max_units))
            != expected_requests.map(|requests| ("credential.aggregate.requests", requests))
        {
            return Err(CredentialCapabilityError::GrantMismatch);
        }
        Ok(())
    }

    pub fn validate_at(
        &self,
        now_unix_seconds: i64,
        revocations: &RevocationState,
    ) -> Result<(), CredentialCapabilityError> {
        self.validate()?;
        revocations.validate()?;
        if now_unix_seconds < 0 {
            return Err(CredentialCapabilityError::NegativeTimestamp);
        }
        if now_unix_seconds < self.issued_at_unix_seconds
            || now_unix_seconds >= self.expires_at_unix_seconds
        {
            return Err(CredentialCapabilityError::ExpiredOrNotYetValid);
        }
        if self.revocation_generation != revocations.generation {
            return Err(CredentialCapabilityError::StaleRevocationGeneration);
        }
        if revocations.grant_is_revoked(&self.grant) {
            return Err(CredentialCapabilityError::Revoked);
        }
        Ok(())
    }

    pub fn decision(&self) -> Result<AuthorizationDecision, CredentialCapabilityError> {
        self.validate()?;
        Ok(AuthorizationDecision {
            effect: AuthorizationEffect::Allow,
            reason: DecisionReason::MatchingGrant,
            request_digest: BoundedText::new(self.authorization.digest()?)?,
            matched_grant_id: Some(self.grant.grant_id.clone()),
        })
    }

    pub fn digest(&self) -> Result<String, CredentialCapabilityError> {
        self.validate()?;
        canonical_sha256(self).map_err(CredentialCapabilityError::Serialization)
    }
}

/// Public identifier safe for decisions, receipts, logs, and UI. This digest is
/// not the bearer capability and cannot authorize credential use by itself.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, JsonSchema)]
#[serde(transparent)]
#[schemars(transparent)]
pub struct CapabilityId(BoundedText);

impl CapabilityId {
    pub fn from_sha256_hex(value: impl Into<String>) -> Result<Self, CredentialCapabilityError> {
        let value = value.into();
        if value.len() != CAPABILITY_ID_HEX_LENGTH
            || !value
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(CredentialCapabilityError::InvalidCapabilityId);
        }
        Ok(Self(BoundedText::new(value)?))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl<'de> Deserialize<'de> for CapabilityId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::from_sha256_hex(value).map_err(serde::de::Error::custom)
    }
}

fn validate_dns_host(host: &str) -> Result<(), CredentialCapabilityError> {
    if host.len() > 253
        || host.starts_with('.')
        || host.ends_with('.')
        || host.parse::<std::net::IpAddr>().is_ok()
    {
        return Err(CredentialCapabilityError::InvalidDestinationHost);
    }
    for label in host.split('.') {
        if label.is_empty()
            || label.len() > 63
            || !label
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
            || !label
                .as_bytes()
                .first()
                .is_some_and(u8::is_ascii_alphanumeric)
            || !label
                .as_bytes()
                .last()
                .is_some_and(u8::is_ascii_alphanumeric)
        {
            return Err(CredentialCapabilityError::InvalidDestinationHost);
        }
    }
    Ok(())
}

fn validate_identifier(kind: &'static str, value: &str) -> Result<(), CredentialCapabilityError> {
    if value.len() > 128
        || !value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':' | b'/')
        })
    {
        return Err(CredentialCapabilityError::InvalidIdentifier(kind));
    }
    Ok(())
}

fn validate_origin_path(path: &str) -> Result<(), CredentialCapabilityError> {
    if !path.starts_with('/')
        || path.starts_with("//")
        || path.contains(['?', '#', '\\'])
        || path.split('/').any(|segment| matches!(segment, "." | ".."))
    {
        return Err(CredentialCapabilityError::InvalidOriginPath);
    }
    Ok(())
}

fn usage_limit(limits: &BTreeMap<BoundedText, u64>, dimension: &str) -> Option<u64> {
    limits
        .iter()
        .find_map(|(key, value)| (key.as_str() == dimension).then_some(*value))
}

fn is_credential_usage_key(key: &str) -> bool {
    ["credential.per_request.", "credential.aggregate."]
        .iter()
        .any(|prefix| key.starts_with(prefix))
}

#[derive(Debug, Error)]
pub enum CredentialCapabilityError {
    #[error(
        "unsupported credential capability schema version {found}; supported version is {supported}"
    )]
    UnsupportedSchemaVersion { found: u32, supported: u32 },
    #[error("credential capability timestamps must be non-negative")]
    NegativeTimestamp,
    #[error("credential capability expiry must be later than issuance")]
    InvalidExpiry,
    #[error("credential capability is expired or not yet valid")]
    ExpiredOrNotYetValid,
    #[error("credential capability authorization time does not match issuance")]
    AuthorizationTimeMismatch,
    #[error("credential capability requires an agent as its current actor")]
    AgentActorRequired,
    #[error("credential authorization does not match the referenced label")]
    CredentialAuthorityMismatch,
    #[error("credential scope does not match the requested operation")]
    CredentialScopeMismatch,
    #[error("credential destination does not match the authorization request")]
    DestinationMismatch,
    #[error("credential grant does not match the requested authority")]
    GrantMismatch,
    #[error("credential capability revocation generation is stale")]
    StaleRevocationGeneration,
    #[error("credential capability has been revoked")]
    Revoked,
    #[error("credential destination host is invalid")]
    InvalidDestinationHost,
    #[error("credential destination host must already be canonical lowercase")]
    NonCanonicalDestinationHost,
    #[error("credential destination port is invalid")]
    InvalidDestinationPort,
    #[error("credential request path is not canonical origin form")]
    InvalidOriginPath,
    #[error("{0} is not a valid bounded identifier")]
    InvalidIdentifier(&'static str),
    #[error("capability id is not a lowercase SHA-256 digest")]
    InvalidCapabilityId,
    #[error(transparent)]
    Authorization(#[from] AuthorizationError),
    #[error(transparent)]
    Grant(#[from] GrantValidationError),
    #[error(transparent)]
    Revocation(#[from] RevocationError),
    #[error(transparent)]
    Mandate(#[from] crate::MandateError),
    #[error(transparent)]
    BoundedText(#[from] BoundedTextError),
    #[error("failed to serialize canonical credential capability: {0}")]
    Serialization(serde_json::Error),
}
