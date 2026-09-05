#![forbid(unsafe_code)]

//! Trusted service composition, not an installer or worker-facing bootstrap
//! protocol. The launcher must supply independently validated platform authority,
//! a recovered PF-41 journal with protected integrity root, live Vault grants,
//! and a connected socket whose expected peer came from trusted process launch.
//! None of these objects can be deserialized from a worker request.

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::BrokerService;
#[cfg(target_os = "linux")]
pub use linux::TrustedSession;
