//! PF-27-S04 user-session credential broker process (Unix sockets; named
//! pipes on Windows since PF-27-S06).
//!
//! With the `isolated_credential_broker` feature, provider credentials that
//! the network proxy would otherwise hold in Core are handed to a separate
//! broker process. Core keeps dummy values and opaque references only.

mod client;
/// PF-27-S06: the Windows transport (named pipes).
#[cfg(windows)]
pub(crate) mod pipe;
pub(crate) mod protocol;
mod server;

pub use client::CODEX_CREDENTIAL_BROKER_ARG1;
pub(crate) use client::IsolatedBrokerClient;
pub(crate) use client::IsolatedBrokerError;
pub(crate) use client::IsolatedBrokerLauncher;
pub(crate) use client::IsolatedBrokerOptions;
pub(crate) use client::StoredRegistration;
pub(crate) use client::user_runtime_dir;
pub use protocol::StoredKeyAccount;
pub use server::StoredKeyResolver;
#[cfg(windows)]
pub(crate) use server::prepare_vault_lock;
pub use server::run_credential_broker_main;
pub use server::run_credential_broker_main_with;

#[cfg(test)]
#[path = "isolated_tests.rs"]
mod tests;
