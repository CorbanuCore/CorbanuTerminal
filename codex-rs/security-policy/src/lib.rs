//! Deterministic security-level policy primitives for Corbanu Terminal.
//!
//! This crate is deliberately independent of runtime tool implementations. It
//! defines the persisted security level and secret-free authorization,
//! delegation, mandate, receipt, and revocation contracts that enforcement
//! boundaries compose with their existing decisions.
//!
//! An `Allow` from this layer never overrides an existing deny. Callers must
//! combine decisions with [`compose_existing_decision`].

mod action_context;
mod authorization;
mod bounded;
mod credential;
mod digest;
mod grant;
mod integration;
mod level;
mod mandate;
mod provenance;
mod revocation;

pub use action_context::ActionContext;
pub use action_context::ActionContextError;
pub use action_context::AuthorityEpoch;
pub use action_context::EpochBoundGrant;

pub use authorization::ActorChain;
pub use authorization::AuthorizationContext;
pub use authorization::AuthorizationDecision;
pub use authorization::AuthorizationEffect;
pub use authorization::AuthorizationError;
pub use authorization::AuthorizationRequest;
pub use authorization::DecisionReason;
pub use authorization::PolicyAction;
pub use authorization::PolicyPrincipal;
pub use authorization::PrincipalKind;
pub use authorization::ProtectedResource;
pub use authorization::QuantitativeLimit;
pub use authorization::ResourceKind;
pub use authorization::compose_existing_decision;
pub use authorization::permissive_decision;
pub use bounded::BoundedText;
pub use bounded::BoundedTextError;
pub use bounded::MAX_POLICY_TEXT_BYTES;
pub use credential::CAPABILITY_ID_HEX_LENGTH;
pub use credential::CREDENTIAL_CAPABILITY_SCHEMA_VERSION;
pub use credential::CapabilityId;
pub use credential::CredentialCapabilityError;
pub use credential::CredentialCapabilityRequest;
pub use credential::CredentialDestination;
pub use credential::CredentialHttpMethod;
pub use credential::CredentialReference;
pub use credential::CredentialTransport;
pub use grant::BoundedGrant;
pub use grant::GrantContext;
pub use grant::GrantScope;
pub use grant::GrantValidationError;
pub use integration::SECURITY_INSPECTOR_SCHEMA_VERSION;
pub use integration::SecurityControlHealth;
pub use integration::SecurityControlHealthSnapshot;
pub use integration::SecurityDegradationReason;
pub use integration::SecurityInspectorError;
pub use integration::SecurityInspectorSnapshot;
pub use level::SECURITY_SETTINGS_VERSION;
pub use level::SecurityLevel;
pub use level::SecuritySettings;
pub use level::SecuritySettingsError;
pub use mandate::ActionReceipt;
pub use mandate::CredentialUseReceiptMetadata;
pub use mandate::MandateError;
pub use mandate::MandateOutcome;
pub use mandate::ProtectedActionMandate;
pub use mandate::ProtectedActionPreview;
pub use mandate::ReplayLedger;
pub use provenance::MAX_TAINT_SOURCES;
pub use provenance::ProvenanceError;
pub use provenance::SourceEnvelope;
pub use provenance::SourceId;
pub use provenance::SourceKind;
pub use provenance::TaintContext;
pub use revocation::DISPATCH_FENCE_SCHEMA_VERSION;
pub use revocation::DispatchAuthorityKind;
pub use revocation::DispatchFence;
pub use revocation::DispatchPhase;
pub use revocation::ProtectedDispatchStep;
pub use revocation::RestrictionApplication;
pub use revocation::RestrictionAuditStatus;
pub use revocation::RevocationError;
pub use revocation::RevocationEvent;
pub use revocation::RevocationReason;
pub use revocation::RevocationState;
pub use revocation::RevocationTarget;

#[cfg(test)]
#[path = "provenance_tests.rs"]
mod provenance_tests;

#[cfg(test)]
#[path = "action_context_tests.rs"]
mod action_context_tests;

#[cfg(test)]
#[path = "credential_tests.rs"]
mod credential_tests;

#[cfg(test)]
#[path = "integration_tests.rs"]
mod integration_tests;

#[cfg(test)]
#[path = "security_policy_tests.rs"]
mod tests;
