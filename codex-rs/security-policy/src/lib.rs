//! Deterministic security-level policy primitives for Corbanu Terminal.
//!
//! This crate is deliberately independent of runtime tool implementations. It
//! defines the persisted security level and bounded identifiers used by later
//! authorization boundaries.

mod bounded;
mod level;

pub use bounded::BoundedText;
pub use bounded::BoundedTextError;
pub use bounded::MAX_POLICY_TEXT_BYTES;
pub use level::SECURITY_SETTINGS_VERSION;
pub use level::SecurityLevel;
pub use level::SecuritySettings;
pub use level::SecuritySettingsError;
