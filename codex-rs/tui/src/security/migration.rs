//! PF-29-S02: the vault behind `/security` credential migration, and the
//! debug-only failure hook the recovery demo uses.

use std::collections::BTreeSet;
use std::path::Path;

use codex_vault::AddCredential;
use codex_vault::CredentialType;
use codex_vault::Vault;
use zeroize::Zeroizing;

use crate::legacy_core::protected_preflight::migration::CredentialStore;
use crate::legacy_core::protected_preflight::migration::FailPoint;

/// Debug builds only: stop the next migration at this point, as if Corbanu
/// had crashed there (`prepared`, `stored`, `rewritten`, `committed`).
pub(crate) const FAIL_AT_ENV: &str = "CORBANU_TEST_MIGRATION_FAIL_AT";

pub(crate) fn injected_failure() -> Option<FailPoint> {
    if !cfg!(debug_assertions) {
        return None;
    }
    std::env::var(FAIL_AT_ENV)
        .ok()
        .and_then(|point| FailPoint::parse(&point))
}

pub(crate) struct VaultStore {
    vault: Vault,
}

impl VaultStore {
    pub(crate) fn new(codex_home: &Path) -> Self {
        Self {
            vault: Vault::new(codex_home.to_path_buf()),
        }
    }
}

impl CredentialStore for VaultStore {
    /// Labels already in use; a migration never replaces one.
    fn labels(&self) -> Result<BTreeSet<String>, String> {
        self.vault
            .list()
            .map(|credentials| credentials.into_iter().map(|meta| meta.label).collect())
            .map_err(|err| err.to_string())
    }

    fn put(&self, label: &str, name: &str, origin: &str, value: &str) -> Result<(), String> {
        self.vault
            .add(AddCredential {
                label: label.to_string(),
                credential_type: CredentialType::ManualSecret,
                provider: None,
                notes: Some(format!("{name}, moved from {origin} by /security")),
                revocation_notes: Some(
                    "It was stored in plain text where agents could read it; rotate it at its provider."
                        .to_string(),
                ),
                secret: value.to_string(),
            })
            .map_err(|err| err.to_string())
    }

    fn get(&self, label: &str) -> Result<Option<Zeroizing<String>>, String> {
        if !self.vault.exists(label).map_err(|err| err.to_string())? {
            return Ok(None);
        }
        self.vault
            .reveal(label)
            .map(|value| Some(Zeroizing::new(value)))
            .map_err(|err| err.to_string())
    }
}
