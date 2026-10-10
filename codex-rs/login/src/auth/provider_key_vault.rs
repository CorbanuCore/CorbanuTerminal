//! Vault-backed resolver/writer for provider API keys (Ambient, Z.AI, OpenRouter, etc.).
//!
//! This is the migration-compatible bridge between the legacy plaintext `provider_auth.json`
//! and the new encrypted [`codex_vault::Vault`] substrate. New provider keys are written to the
//! vault (encrypted at rest via the age-encrypted secrets store keyed by the configured keyring).
//! Reads check the vault first and fall back to `provider_auth.json` so existing installations
//! keep working until the key is re-saved.
//!
//! Provider keys are stored as vault credentials labeled `provider/<provider_key_id>`, where
//! `provider_key_id` is the provider's env-key name (for example `AMBIENT_API_KEY`).
//!
//! # Fallback policy
//!
//! The production vault uses the OS keyring when available. On headless Linux hosts without a
//! FreeDesktop Secret Service, the secrets layer stores only the vault encryption passphrase in a
//! local `0600` keyring-fallback file so provider keys can still live in the encrypted vault and
//! appear under `/vault`. Legacy plaintext `provider_auth.json` is used only if the vault itself
//! fails for some other reason. Those read/write fallbacks emit `tracing::warn!` so they are never
//! silent, and the legacy file retains its `0600` permissions.

use std::collections::HashSet;
use std::path::Path;
#[cfg(test)]
use std::sync::Arc;

#[cfg(test)]
use codex_keyring_store::KeyringStore;
use codex_vault::AddCredential;
use codex_vault::CredentialType;
use codex_vault::Vault;
use codex_vault::VaultError;
use codex_vault::VaultKeyStorage;

use super::manager::ProviderApiKeyStorageMetadata;
use super::manager::ProviderApiKeyStorageSource;

/// Prefix used for provider-key vault labels so they are namespaced apart from user-added
/// credentials.
pub(crate) const PROVIDER_LABEL_PREFIX: &str = "provider/";

/// PF-84: the API key of one named account, from the encrypted vault only.
/// There is no legacy-file or environment fallback, and a missing account is
/// `Ok(None)`, never another account's key.
pub fn provider_account_api_key(
    codex_home: &Path,
    provider_id: &str,
    account_name: &str,
) -> std::io::Result<Option<String>> {
    read_provider_account_kind(
        &Vault::new(codex_home.to_path_buf()),
        provider_id,
        account_name,
        codex_vault::ProviderAccountKind::ApiKey,
    )
}

/// PF-84: the AWS profile name a named account uses (not secret).
pub fn provider_account_aws_profile(
    codex_home: &Path,
    provider_id: &str,
    account_name: &str,
) -> std::io::Result<Option<String>> {
    read_provider_account_kind(
        &Vault::new(codex_home.to_path_buf()),
        provider_id,
        account_name,
        codex_vault::ProviderAccountKind::AwsProfile,
    )
}

fn read_provider_account_kind(
    vault: &Vault,
    provider_id: &str,
    account_name: &str,
    kind: codex_vault::ProviderAccountKind,
) -> std::io::Result<Option<String>> {
    let name = codex_vault::ProviderAccountName::parse(account_name)
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidInput, error))?;
    if vault.key_storage() == VaultKeyStorage::NotInitialized {
        return Ok(None);
    }
    vault
        .read_provider_account(provider_id, &name, kind)
        .map(|value| value.map(|value| value.to_string()))
        .map_err(std::io::Error::other)
}

/// Resolve a provider API key, preferring the encrypted vault over the legacy plaintext store.
///
/// Returns `Ok(None)` when the key is absent from both stores.
pub(crate) fn read_provider_key(
    codex_home: &Path,
    provider_key_id: &str,
) -> std::io::Result<Option<String>> {
    let vault = Vault::new(codex_home.to_path_buf());
    if vault.key_storage() == VaultKeyStorage::NotInitialized {
        return super::manager::legacy_provider_key(codex_home, provider_key_id);
    }
    read_provider_key_with_vault(codex_home, provider_key_id, vault)
}

pub(crate) fn provider_key_metadata(
    codex_home: &Path,
    provider_key_id: &str,
) -> std::io::Result<ProviderApiKeyStorageMetadata> {
    let vault = Vault::new(codex_home.to_path_buf());
    if vault.key_storage() != VaultKeyStorage::NotInitialized {
        match vault.exists(&provider_label(provider_key_id)) {
            Ok(true) => {
                return Ok(ProviderApiKeyStorageMetadata::Stored {
                    source: ProviderApiKeyStorageSource::EncryptedVault,
                });
            }
            Ok(false) => {}
            Err(err) => tracing::warn!(
                ?err,
                provider_key_id,
                "provider key vault metadata unavailable; checking legacy provider storage"
            ),
        }
    }
    if super::manager::legacy_provider_key_is_present(codex_home, provider_key_id)? {
        Ok(ProviderApiKeyStorageMetadata::Stored {
            source: ProviderApiKeyStorageSource::LegacyPlaintext,
        })
    } else {
        Ok(ProviderApiKeyStorageMetadata::Missing)
    }
}

/// List stored provider-key identities with one metadata-only vault read.
///
/// An unavailable vault degrades to an empty set so callers can preserve the existing legacy
/// fallback behavior. Secret values are neither decrypted nor returned.
pub(crate) fn stored_provider_key_ids(codex_home: &Path) -> HashSet<String> {
    stored_provider_key_ids_with_vault(&Vault::new(codex_home.to_path_buf()))
}

fn stored_provider_key_ids_with_vault(vault: &Vault) -> HashSet<String> {
    if vault.key_storage() == VaultKeyStorage::NotInitialized {
        return HashSet::new();
    }
    match vault.list() {
        Ok(entries) => entries
            .into_iter()
            .filter_map(|entry| {
                let provider_key_id = entry.label.strip_prefix(PROVIDER_LABEL_PREFIX)?;
                (!provider_key_id.is_empty()
                    && provider_key_id
                        .bytes()
                        .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_'))
                .then(|| provider_key_id.to_ascii_uppercase())
            })
            .collect(),
        Err(err) => {
            tracing::warn!(
                ?err,
                "provider key vault metadata unavailable; checking legacy provider storage"
            );
            HashSet::new()
        }
    }
}

#[cfg(test)]
fn read_provider_key_with_store(
    codex_home: &Path,
    provider_key_id: &str,
    keyring_store: Arc<dyn KeyringStore>,
) -> std::io::Result<Option<String>> {
    let vault = Vault::new_with_keyring_store(codex_home.to_path_buf(), keyring_store);
    read_provider_key_with_vault(codex_home, provider_key_id, vault)
}

fn read_provider_key_with_vault(
    codex_home: &Path,
    provider_key_id: &str,
    vault: Vault,
) -> std::io::Result<Option<String>> {
    let label = provider_label(provider_key_id);
    match vault.reveal(&label) {
        Ok(value) if !value.trim().is_empty() => return Ok(Some(value)),
        Ok(_) => {} // empty value: fall through to legacy store
        Err(VaultError::NotFound { .. }) => {
            tracing::debug!(
                provider_key_id,
                "provider key absent from encrypted vault; checking legacy plaintext provider_auth.json"
            );
        }
        Err(err) => {
            // Vault unavailable (e.g. keyring missing) or decrypt failure: fall back to legacy.
            tracing::warn!(
                ?err,
                "provider key vault read unavailable; reading legacy plaintext provider_auth.json (keyring-less degradation mode)"
            );
        }
    }
    super::manager::legacy_provider_key(codex_home, provider_key_id)
}

/// Write a provider API key to the encrypted vault.
///
/// Falls back to the legacy plaintext store if the vault/keyring is unavailable so callers keep
/// working in keyring-less environments.
pub(crate) fn write_provider_key(
    codex_home: &Path,
    provider_key_id: &str,
    api_key: &str,
) -> std::io::Result<()> {
    let vault = Vault::new(codex_home.to_path_buf());
    write_provider_key_with_vault(codex_home, provider_key_id, api_key, vault)
}

/// Remove one provider key from the encrypted vault.
///
/// Legacy plaintext cleanup is owned by the manager so callers can remove both copies as one
/// operation.
pub(crate) fn delete_provider_key(
    codex_home: &Path,
    provider_key_id: &str,
) -> std::io::Result<bool> {
    let vault = Vault::new(codex_home.to_path_buf());
    vault
        .delete(&provider_label(provider_key_id))
        .map_err(std::io::Error::other)
}

#[cfg(test)]
fn write_provider_key_with_store(
    codex_home: &Path,
    provider_key_id: &str,
    api_key: &str,
    keyring_store: Arc<dyn KeyringStore>,
) -> std::io::Result<()> {
    let vault = Vault::new_with_keyring_store(codex_home.to_path_buf(), keyring_store);
    write_provider_key_with_vault(codex_home, provider_key_id, api_key, vault)
}

fn write_provider_key_with_vault(
    codex_home: &Path,
    provider_key_id: &str,
    api_key: &str,
    vault: Vault,
) -> std::io::Result<()> {
    let label = provider_label(provider_key_id);
    let entry = AddCredential {
        label: label.clone(),
        credential_type: CredentialType::ApiKey,
        provider: Some(provider_key_id.to_string()),
        notes: Some(format!("provider API key for {provider_key_id}")),
        revocation_notes: None,
        secret: api_key.to_string(),
    };

    // Overwrite if a credential for this provider already exists (key rotation); otherwise add.
    let stored = match vault.update(
        &label,
        Some(api_key.to_string()),
        /*provider*/ None,
        /*notes*/ None,
        /*revocation_notes*/ None,
    ) {
        Ok(_) => true,
        Err(codex_vault::VaultError::NotFound { .. }) => match vault.add(entry) {
            Ok(()) => true,
            Err(err) => {
                tracing::warn!(
                    ?err,
                    "provider key vault write failed; storing provider key in legacy plaintext provider_auth.json (keyring-less degradation mode)"
                );
                return super::manager::legacy_save_provider_key(
                    codex_home,
                    provider_key_id,
                    api_key,
                );
            }
        },
        Err(err) => {
            tracing::warn!(
                ?err,
                "provider key vault update failed; storing provider key in legacy plaintext provider_auth.json (keyring-less degradation mode)"
            );
            return super::manager::legacy_save_provider_key(codex_home, provider_key_id, api_key);
        }
    };

    if stored {
        // Migrate the old plaintext key away now that the encrypted copy is durable. Best-effort:
        // a failure here does not undo the successful vault write; it just leaves a stale copy that
        // a future logout or re-save will clean up.
        match super::manager::legacy_delete_provider_key(codex_home, provider_key_id) {
            Ok(true) => {
                tracing::info!(
                    provider_key_id,
                    "migrated provider key off legacy plaintext storage into the encrypted vault"
                );
            }
            Ok(false) => {}
            Err(err) => tracing::warn!(
                ?err,
                provider_key_id,
                "vault write succeeded but failed to remove the legacy plaintext provider key"
            ),
        }
    }
    Ok(())
}

/// Remove all provider keys from the vault (used during logout).
///
/// Returns `true` if any vault provider credential was removed. Best-effort: a vault failure
/// (for example an unavailable keyring) simply returns `Ok(false)`.
pub(crate) fn delete_all_provider_keys(codex_home: &Path) -> std::io::Result<bool> {
    let vault = Vault::new(codex_home.to_path_buf());
    delete_all_provider_keys_with_vault(vault)
}

#[cfg(test)]
fn delete_all_provider_keys_with_store(
    codex_home: &Path,
    keyring_store: Arc<dyn KeyringStore>,
) -> std::io::Result<bool> {
    let vault = Vault::new_with_keyring_store(codex_home.to_path_buf(), keyring_store);
    delete_all_provider_keys_with_vault(vault)
}

fn delete_all_provider_keys_with_vault(vault: Vault) -> std::io::Result<bool> {
    let listing = match vault.list() {
        Ok(listing) => listing,
        Err(err) => {
            tracing::debug!(?err, "provider key vault list unavailable during logout");
            return Ok(false);
        }
    };
    let labels = listing
        .into_iter()
        .filter(|meta| meta.label.starts_with(PROVIDER_LABEL_PREFIX))
        .map(|meta| meta.label)
        .collect::<Vec<_>>();
    if labels.is_empty() {
        return Ok(false);
    }
    match vault.delete_many(&labels) {
        Ok(removed) => Ok(removed > 0),
        Err(err) => {
            tracing::debug!(?err, "vault provider-key batch delete failed during logout");
            Ok(false)
        }
    }
}

fn provider_label(provider_key_id: &str) -> String {
    // Lowercase the env-key id for a stable, readable vault label.
    format!(
        "{PROVIDER_LABEL_PREFIX}{}",
        provider_key_id.trim().to_ascii_lowercase()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use codex_keyring_store::tests::MockKeyringStore;
    use pretty_assertions::assert_eq;

    #[test]
    fn provider_label_is_namespaced_and_lowercase() {
        assert_eq!(
            provider_label("AMBIENT_API_KEY"),
            "provider/ambient_api_key"
        );
        assert_eq!(provider_label("  ZAI_API_KEY  "), "provider/zai_api_key");
    }

    #[test]
    fn reading_an_empty_home_does_not_initialize_the_vault() {
        let parent = tempfile::tempdir().expect("tempdir");
        let codex_home = parent.path().join("new-home");

        assert_eq!(
            read_provider_key(&codex_home, "AMBIENT_API_KEY").expect("read empty vault"),
            None
        );
        assert!(!codex_home.exists());
    }

    /// Writes a provider key through the encrypted vault and reads it back via the vault resolver
    /// using the SAME keyring store (mirrors a real OS keyring that persists across calls).
    /// This proves new keys land in encrypted storage (not the plaintext provider_auth.json).
    #[test]
    fn write_then_read_round_trips_through_encrypted_vault() {
        let codex_home = tempfile::tempdir().expect("tempdir");
        let keyring = Arc::new(MockKeyringStore::default());
        write_provider_key_with_store(
            codex_home.path(),
            "AMBIENT_API_KEY",
            "ambient-secret",
            keyring.clone(),
        )
        .expect("write should succeed via vault");

        // No plaintext provider_auth.json should have been created: the key lives in the
        // age-encrypted secrets store keyed by the keyring passphrase.
        let legacy_path = codex_home.path().join("provider_auth.json");
        assert!(
            !legacy_path.exists(),
            "vault write must not create a plaintext provider_auth.json"
        );

        let read = read_provider_key_with_store(codex_home.path(), "AMBIENT_API_KEY", keyring)
            .expect("read with same keyring");
        assert_eq!(read.as_deref(), Some("ambient-secret"));
    }

    /// When the vault is empty, the resolver falls back to the legacy plaintext store.
    #[test]
    fn read_falls_back_to_legacy_provider_auth_when_vault_empty() {
        let codex_home = tempfile::tempdir().expect("tempdir");
        // Seed the legacy store directly (no vault involved).
        super::super::manager::legacy_save_provider_key(
            codex_home.path(),
            "OPENROUTER_API_KEY",
            "or-legacy-key",
        )
        .expect("legacy write");

        // Vault is empty (different keyring), so read must come from the legacy store.
        let value = read_provider_key_with_store(
            codex_home.path(),
            "OPENROUTER_API_KEY",
            Arc::new(MockKeyringStore::default()),
        )
        .expect("read should fall back");
        assert_eq!(value.as_deref(), Some("or-legacy-key"));
    }

    /// A successful vault write migrates the old plaintext key off `provider_auth.json`.
    #[test]
    fn vault_write_migrates_legacy_plaintext_key_away() {
        let codex_home = tempfile::tempdir().expect("tempdir");
        let keyring = Arc::new(MockKeyringStore::default());

        // Seed the legacy plaintext store with a pre-existing key.
        super::super::manager::legacy_save_provider_key(
            codex_home.path(),
            "AMBIENT_API_KEY",
            "legacy-plaintext",
        )
        .expect("seed legacy key");
        assert!(codex_home.path().join("provider_auth.json").exists());

        // Re-save via the vault (encrypted). The old plaintext copy must be removed.
        write_provider_key_with_store(
            codex_home.path(),
            "AMBIENT_API_KEY",
            "new-encrypted",
            keyring.clone(),
        )
        .expect("vault write");

        // The plaintext file should be gone (it held only this key, so it is deleted when empty).
        assert!(
            !codex_home.path().join("provider_auth.json").exists(),
            "legacy plaintext provider key must be migrated away after a successful vault write"
        );

        // The key now resolves from the encrypted vault.
        let value = read_provider_key_with_store(codex_home.path(), "AMBIENT_API_KEY", keyring)
            .expect("read");
        assert_eq!(value.as_deref(), Some("new-encrypted"));
    }

    /// A successful vault write leaves unrelated legacy keys in `provider_auth.json` intact.
    #[test]
    fn vault_write_preserves_unrelated_legacy_keys() {
        let codex_home = tempfile::tempdir().expect("tempdir");
        let keyring = Arc::new(MockKeyringStore::default());
        super::super::manager::legacy_save_provider_key(
            codex_home.path(),
            "AMBIENT_API_KEY",
            "ambient-legacy",
        )
        .expect("seed ambient");
        super::super::manager::legacy_save_provider_key(
            codex_home.path(),
            "OPENROUTER_API_KEY",
            "openrouter-legacy",
        )
        .expect("seed openrouter");

        write_provider_key_with_store(
            codex_home.path(),
            "AMBIENT_API_KEY",
            "ambient-encrypted",
            keyring.clone(),
        )
        .expect("vault write ambient");

        // The file survives because it still holds the unrelated OpenRouter key.
        assert!(codex_home.path().join("provider_auth.json").exists());
        // Ambient is now in the vault...
        assert_eq!(
            read_provider_key_with_store(codex_home.path(), "AMBIENT_API_KEY", keyring)
                .expect("ambient read")
                .as_deref(),
            Some("ambient-encrypted")
        );
        // ...and OpenRouter remains readable from the (now smaller) legacy file.
        assert_eq!(
            super::super::manager::legacy_provider_key(codex_home.path(), "OPENROUTER_API_KEY")
                .expect("openrouter legacy read")
                .as_deref(),
            Some("openrouter-legacy")
        );
    }

    /// Re-writing a provider key rotates the stored secret in place (update, not duplicate).
    #[test]
    fn rewriting_a_provider_key_rotates_in_place() {
        let codex_home = tempfile::tempdir().expect("tempdir");
        let keyring = Arc::new(MockKeyringStore::default());
        write_provider_key_with_store(codex_home.path(), "ZAI_API_KEY", "old", keyring.clone())
            .expect("first write");
        write_provider_key_with_store(codex_home.path(), "ZAI_API_KEY", "new", keyring.clone())
            .expect("rotation write");

        let value =
            read_provider_key_with_store(codex_home.path(), "ZAI_API_KEY", keyring).expect("read");
        assert_eq!(value.as_deref(), Some("new"));
    }

    /// `delete_all_provider_keys` removes provider-scoped vault credentials only.
    #[test]
    fn delete_all_provider_keys_clears_provider_entries() {
        let codex_home = tempfile::tempdir().expect("tempdir");
        let keyring = Arc::new(MockKeyringStore::default());
        write_provider_key_with_store(codex_home.path(), "AMBIENT_API_KEY", "a", keyring.clone())
            .expect("write ambient");
        write_provider_key_with_store(codex_home.path(), "ZAI_API_KEY", "z", keyring.clone())
            .expect("write zai");

        // Add a non-provider user credential that must survive the provider wipe.
        let vault = Vault::new_with_keyring_store(codex_home.path().to_path_buf(), keyring.clone());
        vault
            .add(AddCredential {
                label: "personal-token".to_string(),
                credential_type: CredentialType::BearerToken,
                provider: None,
                notes: None,
                revocation_notes: None,
                secret: "keep-me".to_string(),
            })
            .expect("user credential");

        let removed =
            super::delete_all_provider_keys_with_store(codex_home.path(), keyring.clone())
                .expect("delete");
        assert!(removed, "provider keys should have been removed");

        // Provider keys are gone...
        assert_eq!(
            read_provider_key_with_store(codex_home.path(), "AMBIENT_API_KEY", keyring)
                .expect("ambient read"),
            None
        );
        // ...but the user credential is untouched.
        assert_eq!(
            vault.reveal("personal-token").expect("user reveal"),
            "keep-me"
        );
    }

    /// PF-84: each named account resolves only its own key; a missing account
    /// never yields the default key, and the default key never yields an account's.
    #[test]
    fn named_account_keys_are_isolated_from_each_other_and_from_default() {
        let codex_home = tempfile::tempdir().expect("tempdir");
        let keyring = Arc::new(MockKeyringStore::default());
        write_provider_key_with_store(
            codex_home.path(),
            "ZAI_API_KEY",
            "canary-default",
            keyring.clone(),
        )
        .expect("default key");
        let vault = Vault::new_with_keyring_store(codex_home.path().to_path_buf(), keyring.clone());
        for (name, value) in [("a", "canary-a"), ("b", "canary-b")] {
            vault
                .write_provider_account(
                    "zai",
                    &codex_vault::ProviderAccountName::parse(name).expect("name"),
                    codex_vault::ProviderAccountKind::ApiKey,
                    value,
                )
                .expect("account key");
        }
        let account = |name: &str| {
            read_provider_account_kind(
                &vault,
                "zai",
                name,
                codex_vault::ProviderAccountKind::ApiKey,
            )
            .expect("read account")
        };
        assert_eq!(account("a").as_deref(), Some("canary-a"));
        assert_eq!(account("b").as_deref(), Some("canary-b"));
        assert_eq!(account("missing"), None);
        assert!(
            read_provider_account_kind(
                &vault,
                "zai",
                "default",
                codex_vault::ProviderAccountKind::ApiKey,
            )
            .is_err(),
            "`default` is not a named account"
        );
        assert_eq!(
            read_provider_key_with_store(codex_home.path(), "ZAI_API_KEY", keyring)
                .expect("default read")
                .as_deref(),
            Some("canary-default")
        );
        // Named labels never surface as default provider-key identities.
        assert_eq!(
            stored_provider_key_ids_with_vault(&vault),
            HashSet::from(["ZAI_API_KEY".to_string()])
        );
    }

    /// PF-84 migration: a single-account home resolves identically and its
    /// vault bytes stay unchanged until the first named-account write.
    #[test]
    fn single_account_home_is_unchanged_until_a_named_account_is_added() {
        let codex_home = tempfile::tempdir().expect("tempdir");
        let keyring = Arc::new(MockKeyringStore::default());
        write_provider_key_with_store(codex_home.path(), "ZAI_API_KEY", "canary", keyring.clone())
            .expect("default key");
        let vault_file = codex_home.path().join("secrets").join("local.age");
        let before = std::fs::read(&vault_file).expect("vault bytes");
        let vault = Vault::new_with_keyring_store(codex_home.path().to_path_buf(), keyring.clone());
        assert_eq!(vault.list_provider_accounts().expect("list"), Vec::new());
        assert_eq!(
            read_provider_account_kind(
                &vault,
                "zai",
                "work",
                codex_vault::ProviderAccountKind::ApiKey,
            )
            .expect("read"),
            None
        );
        assert_eq!(
            read_provider_key_with_store(codex_home.path(), "ZAI_API_KEY", keyring.clone())
                .expect("default read")
                .as_deref(),
            Some("canary")
        );
        assert_eq!(std::fs::read(&vault_file).expect("vault bytes"), before);

        vault
            .write_provider_account(
                "zai",
                &codex_vault::ProviderAccountName::parse("work").expect("name"),
                codex_vault::ProviderAccountKind::ApiKey,
                "canary-work",
            )
            .expect("named write");
        assert_ne!(std::fs::read(&vault_file).expect("vault bytes"), before);
        assert_eq!(
            read_provider_key_with_store(codex_home.path(), "ZAI_API_KEY", keyring)
                .expect("default read")
                .as_deref(),
            Some("canary")
        );
    }
}
