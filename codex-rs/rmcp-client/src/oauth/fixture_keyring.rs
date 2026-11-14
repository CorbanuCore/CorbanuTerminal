//! Unit-test dependency injection, never compiled into the production client.
//!
//! Isolated child tests install their own in-memory or fixture-file builder.
//! Do not install OS credential builders here. Without an explicit fixture,
//! calls retain DefaultKeyringStore's native-access denial.

use std::sync::OnceLock;

use codex_keyring_store::CredentialStoreError;
use codex_keyring_store::DefaultKeyringStore as NativeKeyringStore;
use codex_keyring_store::KeyringStore;
use keyring::credential::CredentialBuilder;

#[path = "fixture_keyring_tests.rs"]
mod tests;

static FIXTURE: OnceLock<Box<CredentialBuilder>> = OnceLock::new();

pub(crate) fn install(builder: Box<CredentialBuilder>) {
    // Native denial only applies in debug builds (codex-keyring-store).
    if !cfg!(debug_assertions) {
        panic!("fixture injection requires a debug test build");
    }
    assert_eq!(
        std::env::var("CORBANU_TEST_NO_NATIVE_KEYRING").as_deref(),
        Ok("1"),
        "fixture injection requires the isolated test lane",
    );
    assert!(
        FIXTURE.set(builder).is_ok(),
        "one fixture per child process"
    );
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct DefaultKeyringStore;

impl KeyringStore for DefaultKeyringStore {
    fn load(&self, service: &str, account: &str) -> Result<Option<String>, CredentialStoreError> {
        let Some(builder) = FIXTURE.get() else {
            return NativeKeyringStore.load(service, account);
        };
        let result = builder
            .build(/*target*/ None, service, account)
            .and_then(|entry| entry.get_password());
        match result {
            Ok(value) => Ok(Some(value)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(error) => Err(CredentialStoreError::new(error)),
        }
    }

    fn save(&self, service: &str, account: &str, value: &str) -> Result<(), CredentialStoreError> {
        let Some(builder) = FIXTURE.get() else {
            return NativeKeyringStore.save(service, account, value);
        };
        builder
            .build(/*target*/ None, service, account)
            .and_then(|entry| entry.set_password(value))
            .map_err(CredentialStoreError::new)
    }

    fn delete(&self, service: &str, account: &str) -> Result<bool, CredentialStoreError> {
        let Some(builder) = FIXTURE.get() else {
            return NativeKeyringStore.delete(service, account);
        };
        let result = builder
            .build(/*target*/ None, service, account)
            .and_then(|entry| entry.delete_credential());
        match result {
            Ok(()) => Ok(true),
            Err(keyring::Error::NoEntry) => Ok(false),
            Err(error) => Err(CredentialStoreError::new(error)),
        }
    }
}
