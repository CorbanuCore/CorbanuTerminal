//! PF-27-S04 user-session credential broker process.
//!
//! With the `isolated_credential_broker` feature, provider credentials that
//! the network proxy would otherwise hold in Core are handed to a separate
//! broker process. Core keeps dummy values and opaque references only.

mod client;
pub(crate) mod protocol;
mod server;

pub use client::CODEX_CREDENTIAL_BROKER_ARG1;
pub(crate) use client::IsolatedBrokerClient;
pub(crate) use client::IsolatedBrokerError;
pub(crate) use client::IsolatedBrokerLauncher;
pub(crate) use client::IsolatedBrokerOptions;
pub(crate) use client::user_runtime_dir;
pub use server::run_credential_broker_main;

#[cfg(test)]
#[path = "isolated_tests.rs"]
mod tests;
