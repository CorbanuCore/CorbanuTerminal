//! Fail-closed external-content screening contracts for Corbanu Terminal.

mod contract;

pub use contract::ClassifierVerdict;
pub use contract::ContentAuthority;
pub use contract::ContentBinding;
pub use contract::ContentDigest;
pub use contract::ContentTaint;
pub use contract::ContractError;
pub use contract::ContractId;
pub use contract::DiagnosticCode;
pub use contract::MAX_SCREENED_CONTENT_BYTES;
pub use contract::MAX_SCREENING_ELAPSED_MS;
pub use contract::MAX_SCREENING_SEGMENTS;
pub use contract::MAX_VERDICT_AGE_MS;
pub use contract::ModelIdentity;
pub use contract::SCREENING_CONTRACT_VERSION;
pub use contract::SCREENING_FIXTURE_SCHEMA_VERSION;
pub use contract::ScreenedContent;
pub use contract::ScreeningBudget;
pub use contract::ScreeningDecision;
pub use contract::ScreeningProgress;
pub use contract::ScreeningSession;
pub use contract::ScreeningTarget;
pub use contract::SegmentEnvelope;
pub use contract::SourceBinding;
pub use contract::ThresholdIdentity;
pub use contract::TransformationBinding;
pub use contract::UnavailableReason;
pub use contract::UntrustedBytes;
pub use contract::VerdictIdentity;
pub use contract::VerdictKind;
pub use contract::WithheldContent;
