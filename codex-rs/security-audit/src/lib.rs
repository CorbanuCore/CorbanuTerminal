//! Durable, secret-free security event contracts.
//!
//! This crate is a foundation only. It does not register a producer, activate a
//! protected profile, or grant authority. Consumers must obtain a durable
//! dispatch permit before an external effect and must resolve it as completed or
//! explicitly unknown. Integrity roots are owned by a PF-20 controller-backed
//! [`IntegrityRootStore`]; the local hash chain is not a host-compromise boundary.

mod event;
mod journal;
mod journal_debug;
#[cfg(test)]
mod journal_faults;
mod journal_support;
mod journal_types;
mod recovery;
mod storage;

pub use event::ActionId;
pub use event::AuthorityIdentity;
pub use event::DecisionId;
pub use event::DispatchResolution;
pub use event::EventContext;
pub use event::RequestIdentity;
pub use event::ReservationId;
pub use event::SecurityEvent;
pub use event::SecurityEventError;
pub use event::SecurityEventId;
pub use event::SecurityEventKind;
pub use event::UnknownOutcomeReason;
pub use journal::ReferenceJournal;
pub use journal_types::AppendAcknowledgement;
pub use journal_types::DispatchPermit;
pub use journal_types::IntegrityCheckpoint;
pub use journal_types::IntegrityRootError;
pub use journal_types::IntegrityRootStore;
pub use journal_types::JournalConfig;
pub use journal_types::JournalError;
pub use journal_types::JournalOwner;
pub use recovery::AuditGapReason;
pub use recovery::EmergencyRestrictionResult;
pub use recovery::PendingDispatch;
pub use recovery::RecoveryBlocker;
pub use recovery::RecoveryReport;
pub use recovery::RecoveryState;
pub use recovery::apply_emergency_restriction;

pub const SECURITY_AUDIT_SCHEMA_VERSION: u32 = 1;

#[cfg(test)]
#[path = "event_tests.rs"]
mod event_tests;

#[cfg(test)]
#[path = "journal_tests.rs"]
mod journal_tests;
