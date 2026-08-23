//! Deterministic security-level policy primitives for Corbanu Terminal.
//!
//! This crate is deliberately independent of runtime tool implementations. It
//! defines the persisted security level and secret-free authorization contracts
//! that enforcement boundaries compose with their existing decisions.
//!
//! An `Allow` from this layer never overrides an existing deny. Callers must
//! combine decisions with [`compose_existing_decision`].

mod authorization;
mod bounded;
mod digest;
mod level;

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
pub use level::SECURITY_SETTINGS_VERSION;
pub use level::SecurityLevel;
pub use level::SecuritySettings;
pub use level::SecuritySettingsError;
