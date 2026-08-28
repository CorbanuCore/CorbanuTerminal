//! PF-30's host-owned, networkless public-web acquisition boundary.
//! No model tool or user-facing facade is registered by this crate.

use thiserror::Error;

mod broker;
mod command;
mod container;
mod engine;
mod image;
mod quarantine;
mod runtime;

pub use quarantine::PromotedArtifact;
pub use quarantine::PromotionRequest;
pub use quarantine::QuarantinedArtifact;
pub use runtime::AcquiredPage;
pub use runtime::BrowserRuntime;
pub use runtime::LiveBrowserAuthority;

pub use engine::ContainerEngine;
pub use engine::EngineKind;
pub use engine::EnginePreference;

/// Secret-free failures safe for the inspector; raw engine/HTTP stderr is never
/// part of this contract. Missing protection never requests a host fallback.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Error)]
pub enum BrowserError {
    #[error("browser isolation is inactive for Permissive")]
    Inactive,
    #[error("container runtime installation is required")]
    RuntimeMissing,
    #[error("selected container runtime is unavailable")]
    RuntimeUnavailable,
    #[error("container runtime or platform is unsupported")]
    UnsupportedRuntime,
    #[error("pinned browser image setup failed")]
    ImageUnavailable,
    #[error("browser container ownership or configuration mismatch")]
    ContainerMismatch,
    #[error("browser containment health check failed")]
    HealthCheckFailed,
    #[error("browser acquisition denied by network policy")]
    DestinationDenied,
    #[error("public-web request failed")]
    FetchFailed,
    #[error("browser acquisition resource limit exceeded")]
    ResourceLimit,
    #[error("invalid isolated worker response")]
    InvalidWorkerResponse,
    #[error("browser acquisition cancelled")]
    Cancelled,
    #[error("browser authority changed; prepare again")]
    StaleAuthority,
    #[error("browser artifact promotion was not approved")]
    PromotionDenied,
}
