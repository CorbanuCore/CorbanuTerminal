//! PF-27-S04 user-session credential broker process.
//!
//! With the `isolated_credential_broker` feature, provider credentials that
//! the network proxy would otherwise hold in Core are handed to a separate
//! broker process. Core keeps dummy values and opaque references only.

#[cfg(unix)]
mod client;
// PF-27-S06: the Windows transport; the broker is wired to it next.
#[cfg(windows)]
#[allow(dead_code)]
pub(crate) mod pipe;
#[cfg(unix)]
pub(crate) mod protocol;
#[cfg(unix)]
mod server;

#[cfg(unix)]
pub use client::CODEX_CREDENTIAL_BROKER_ARG1;
#[cfg(unix)]
pub(crate) use client::IsolatedBrokerClient;
#[cfg(unix)]
pub(crate) use client::IsolatedBrokerError;
#[cfg(unix)]
pub(crate) use client::IsolatedBrokerLauncher;
#[cfg(unix)]
pub(crate) use client::IsolatedBrokerOptions;
#[cfg(unix)]
pub(crate) use client::StoredRegistration;
#[cfg(unix)]
pub(crate) use client::user_runtime_dir;
#[cfg(unix)]
pub use server::StoredKeyResolver;
#[cfg(unix)]
pub use server::run_credential_broker_main;
#[cfg(unix)]
pub use server::run_credential_broker_main_with;

#[cfg(all(test, unix))]
#[path = "isolated_tests.rs"]
mod tests;
