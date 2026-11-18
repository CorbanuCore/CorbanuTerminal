//! Explicit credential injection for isolated integration-test child processes.
//! This opt-in feature never enables the platform or global keyring builder.

use std::sync::OnceLock;

use keyring::Entry;
use keyring::credential::CredentialBuilder;

use crate::CredentialStoreError;

#[cfg(test)]
#[path = "test_support_tests.rs"]
mod tests;

static FIXTURE: OnceLock<Box<CredentialBuilder>> = OnceLock::new();

/// Install an in-memory or fixture-file builder, once in an isolated child process.
/// Never pass an OS credential builder. Release builds cannot install fixtures.
pub fn install(builder: Box<CredentialBuilder>) {
    assert!(
        require_isolated_debug_lane().is_ok(),
        "credential fixture requires the isolated debug lane"
    );
    assert!(
        FIXTURE.set(builder).is_ok(),
        "one credential fixture per process"
    );
}

pub(crate) fn is_installed() -> bool {
    FIXTURE.get().is_some()
}

pub(crate) fn require_isolated_debug_lane() -> Result<(), CredentialStoreError> {
    if cfg!(debug_assertions)
        && std::env::var("CORBANU_TEST_NO_NATIVE_KEYRING").as_deref() == Ok("1")
    {
        Ok(())
    } else {
        Err(CredentialStoreError::from_message(
            "Credential fixtures require an isolated debug test process".to_string(),
        ))
    }
}

pub(crate) fn entry(service: &str, account: &str) -> Option<Result<Entry, CredentialStoreError>> {
    let builder = FIXTURE.get()?;
    Some(require_isolated_debug_lane().and_then(|()| {
        builder
            .build(/*target*/ None, service, account)
            .map(Entry::new_with_credential)
            .map_err(CredentialStoreError::new)
    }))
}
