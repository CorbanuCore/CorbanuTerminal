//! Corbanu Terminal credential vault.
//!
//! This crate provides a labeled credential store built on top of [`codex_secrets::SecretsManager`].
//! Credentials are encrypted at rest using the existing age-encrypted secrets substrate (the same
//! one Codex uses for managed secrets and auth material), keyed by an OS keyring-stored passphrase.
//!
//! # Design
//!
//! The vault stores two kinds of records in the managed-secrets namespace:
//!
//! 1. A **metadata index** (`VAULT_INDEX`) — a JSON map of `label -> VaultCredentialMeta`. This
//!    contains no secret material and is used to list and show credentials without ever touching
//!    raw secrets.
//! 2. One **secret record** per label (`vault/<label>`) holding the raw secret value.
//!
//! Both records live inside the age-encrypted `secrets/local.age` file, so everything is encrypted
//! at rest. Raw secret material is only returned by explicit [`Vault::reveal`] / [`Vault::export`]
//! calls; listing and metadata access never touch it.
//!
//! # v0 scope
//!
//! This is a v0 slice: it covers encrypted labeled storage and a provider-key resolver/writer that
//! is migration-compatible with the legacy plaintext `provider_auth.json`. Lock/unlock is implicit
//! — the OS keyring passphrase gates decryption — so the vault is "unlocked" for the duration that
//! the keyring is available, with no repeated secret entry.

use std::collections::BTreeMap;
use std::fs::OpenOptions;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context;
use anyhow::Result;
use chrono::DateTime;
use chrono::Utc;
use codex_keyring_store::KeyringStore;
use codex_secrets::LocalSecretsNamespace;
use codex_secrets::SecretName;
use codex_secrets::SecretScope;
use codex_secrets::SecretsBackendKind;
use codex_secrets::SecretsManager;
use codex_secrets::local_keyring_fallback_path;
use codex_security_policy::SecurityLevel;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;
use sha2::Digest;
use sha2::Sha256;
use strum_macros::AsRefStr;
use strum_macros::Display;
use thiserror::Error;
use zeroize::Zeroizing;

mod capability;
mod claude_auth;
mod credential_panic;

pub use capability::ScopedCredentialCallbackError;
pub use capability::ScopedCredentialError;
pub use capability::VaultCredentialRef;
pub use claude_auth::CREDENTIALS_FILE_CLAUDE_AUTH_SOURCE_ID;
pub use claude_auth::ClaudeAuthHealth;
pub use claude_auth::ClaudeAuthResolution;
pub use claude_auth::ClaudeAuthSelection;
pub use claude_auth::ClaudeAuthSource;
pub use claude_auth::ClaudeAuthSourceMetadata;
pub use claude_auth::ClaudeAuthStoreKind;
pub use claude_auth::ClaudeSubscriptionTokenError;
pub use claude_auth::ENVIRONMENT_CLAUDE_AUTH_SOURCE_ID;
pub use claude_auth::MACOS_KEYCHAIN_CLAUDE_AUTH_SOURCE_ID;
pub use claude_auth::MANAGED_CLAUDE_AUTH_SOURCE_ID;
pub use claude_auth::MANAGED_CLAUDE_TOKEN_LABEL;
pub use claude_auth::ManagedClaudeTokenStatus;
pub use claude_auth::claude_code_macos_keychain_service;
pub use claude_auth::claude_login_authority_id;
pub use claude_auth::credentials_file_claude_auth_source_id;
pub use claude_auth::macos_keychain_claude_auth_source_id;
pub use claude_auth::resolve_claude_auth_source;
pub use credential_panic::scoped_credential_callback_active;

#[cfg(test)]
mod tests;

/// Stable secret name holding the vault metadata index.
const VAULT_INDEX_SECRET_NAME: &str = "VAULT_INDEX";

/// File used to serialize complete vault transactions across threads and processes.
const VAULT_LOCK_FILE_NAME: &str = ".vault.lock";

/// Version of the on-disk vault index schema. Bumped on breaking changes to
/// [`VaultIndex`] / [`VaultCredentialMeta`].
const VAULT_SCHEMA_VERSION: u32 = 1;

/// Canonical scope used for all vault records. Vault entries are global rather than
/// environment-scoped so credentials can be resolved from any working directory.
const VAULT_SCOPE: SecretScope = SecretScope::Global;

/// Errors returned by vault operations.
#[derive(Debug, Error)]
pub enum VaultError {
    /// A credential with the given label already exists.
    #[error("credential labeled {label:?} already exists")]
    CredentialExists { label: String },
    /// No credential with the given label was found.
    #[error("no credential labeled {label:?}")]
    NotFound { label: String },
    /// The supplied label failed validation.
    #[error("invalid credential label: {0}")]
    InvalidLabel(String),
    /// The raw secret value was empty.
    #[error("secret value must not be empty")]
    EmptySecret,
    /// A provider-managed credential is available only to its scoped integration.
    #[error("credential labeled {label:?} can only be used by its provider integration")]
    ProviderManagedCredential { label: String },
    /// The credential is intentionally excluded from generic automation.
    #[error(
        "credential labeled {label:?} has type {credential_type}, which cannot be used by vault auth-helper"
    )]
    ProgrammaticUseDenied {
        label: String,
        credential_type: CredentialType,
    },
    /// Raw programmatic export is unavailable outside Permissive.
    #[error(
        "vault auth-helper is unavailable under {level}; use a brokered credential label instead"
    )]
    ProgrammaticUseSecurityLevelDenied { level: SecurityLevel },
    /// An underlying storage/encryption error occurred.
    #[error(transparent)]
    Storage(#[from] anyhow::Error),
}

/// The supported credential types for the v0 vault.
///
/// These cover API keys, tokens, auth tuples, and keying material used across
/// model providers, exchanges, and deployment flows.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, Display, AsRefStr,
)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum CredentialType {
    /// A plain API key string (for example `sk-...`).
    ApiKey,
    /// A bearer token used in an `Authorization: Bearer` header.
    BearerToken,
    /// A username/password pair encoded as `username:password`.
    BasicAuth,
    /// OAuth client credentials (client id + secret + optional metadata).
    OauthClient,
    /// A PEM/DER-encoded private key.
    CryptoPrivateKey,
    /// A BIP-39 seed phrase (space- or newline-separated words).
    SeedPhrase,
    /// A JSON keystore blob (for example geth/web3 keystore files).
    KeystoreJson,
    /// An RPC endpoint key.
    RpcKey,
    /// An exchange API key (often key + secret + passphrase).
    ExchangeKey,
    /// A deployment key / CI secret.
    DeploymentKey,
    /// Any other manually-managed secret string.
    ManualSecret,
}

impl CredentialType {
    /// Human-readable description shown in vault listings.
    pub fn description(self) -> &'static str {
        match self {
            CredentialType::ApiKey => "API key",
            CredentialType::BearerToken => "bearer token",
            CredentialType::BasicAuth => "basic auth (username:password)",
            CredentialType::OauthClient => "OAuth client credentials",
            CredentialType::CryptoPrivateKey => "crypto private key",
            CredentialType::SeedPhrase => "seed phrase",
            CredentialType::KeystoreJson => "keystore JSON",
            CredentialType::RpcKey => "RPC key",
            CredentialType::ExchangeKey => "exchange key",
            CredentialType::DeploymentKey => "deployment key",
            CredentialType::ManualSecret => "manual secret",
        }
    }

    /// Whether this credential class may be resolved for use-time automation.
    ///
    /// Key custody material stays behind explicit user reveal/export flows. Other
    /// credentials may be passed directly to a command without requiring a
    /// provider-specific label or integration.
    pub fn permits_programmatic_use(self) -> bool {
        match self {
            CredentialType::ApiKey
            | CredentialType::BearerToken
            | CredentialType::BasicAuth
            | CredentialType::OauthClient
            | CredentialType::RpcKey
            | CredentialType::ExchangeKey
            | CredentialType::DeploymentKey
            | CredentialType::ManualSecret => true,
            CredentialType::CryptoPrivateKey
            | CredentialType::SeedPhrase
            | CredentialType::KeystoreJson => false,
        }
    }
}

/// Storage backend that backs a credential. v0 always uses the encrypted
/// secrets-managed store; the enum leaves room for future backends.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum StorageBackend {
    /// Encrypted age store managed by `codex_secrets`.
    #[default]
    EncryptedSecrets,
}

/// Non-secret metadata for a single stored credential.
///
/// This is safe to surface in listings and `/vault show` output. Raw secret
/// material is never included here.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct VaultCredentialMeta {
    /// Human-readable label, unique within the vault.
    pub label: String,
    /// Categorization of the credential.
    #[serde(rename = "type")]
    pub credential_type: CredentialType,
    /// Optional provider/service this credential is for (for example "ambient", "openrouter").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    /// Free-form notes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    /// Optional revocation / recovery guidance (for example "rotate at https://...").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revocation_notes: Option<String>,
    /// When the credential was first stored (Unix seconds, UTC).
    pub created_at: i64,
    /// When the credential was last updated (Unix seconds, UTC).
    pub updated_at: i64,
    /// Where the credential is physically stored.
    #[serde(default)]
    pub storage_backend: StorageBackend,
}

/// On-disk vault index: a versioned map of label -> metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct VaultIndex {
    version: u32,
    #[serde(default)]
    credentials: BTreeMap<String, VaultCredentialMeta>,
}

impl Default for VaultIndex {
    fn default() -> Self {
        Self {
            version: VAULT_SCHEMA_VERSION,
            credentials: BTreeMap::new(),
        }
    }
}

/// Request to add a credential to the vault.
#[derive(Debug, Clone)]
pub struct AddCredential {
    pub label: String,
    pub credential_type: CredentialType,
    pub provider: Option<String>,
    pub notes: Option<String>,
    pub revocation_notes: Option<String>,
    /// The raw secret value. This is only held in memory until stored.
    pub secret: String,
}

/// The Corbanu Terminal credential vault.
///
/// Wraps [`SecretsManager`] (managed-secrets namespace) to provide labeled,
/// encrypted-at-rest credential storage with explicit reveal/export controls.
#[derive(Clone)]
pub struct Vault {
    secrets: SecretsManager,
    codex_home: PathBuf,
    uses_default_keyring: bool,
}

/// Non-secret description of where the encrypted vault passphrase is kept.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VaultKeyStorage {
    NotInitialized,
    OperatingSystemKeyring,
    LocalFileFallback,
    ConfiguredKeyring,
}

impl std::fmt::Debug for Vault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Vault").finish_non_exhaustive()
    }
}

impl Vault {
    /// Create a vault backed by the default OS keyring store.
    pub fn new(codex_home: PathBuf) -> Self {
        let secrets = SecretsManager::new(codex_home.clone(), SecretsBackendKind::Local);
        Self {
            secrets,
            codex_home,
            uses_default_keyring: true,
        }
    }

    /// Create a vault backed by a custom keyring store (used by tests and
    /// environments without an OS keyring).
    pub fn new_with_keyring_store(
        codex_home: PathBuf,
        keyring_store: Arc<dyn KeyringStore>,
    ) -> Self {
        let secrets = SecretsManager::new_with_keyring_store_and_namespace(
            codex_home.clone(),
            SecretsBackendKind::Local,
            keyring_store,
            LocalSecretsNamespace::ManagedSecrets,
        );
        Self {
            secrets,
            codex_home,
            uses_default_keyring: false,
        }
    }

    /// Report the active passphrase storage mechanism without opening or
    /// exposing the passphrase itself.
    pub fn key_storage(&self) -> VaultKeyStorage {
        if !self.uses_default_keyring {
            return VaultKeyStorage::ConfiguredKeyring;
        }
        if local_keyring_fallback_path(&self.codex_home).is_file() {
            VaultKeyStorage::LocalFileFallback
        } else if self.codex_home.join("secrets").join("local.age").is_file() {
            VaultKeyStorage::OperatingSystemKeyring
        } else {
            VaultKeyStorage::NotInitialized
        }
    }

    /// Add a new credential. Returns [`VaultError::CredentialExists`] if the label is taken.
    pub fn add(&self, entry: AddCredential) -> Result<(), VaultError> {
        let AddCredential {
            label: entry_label,
            credential_type,
            provider,
            notes,
            revocation_notes,
            secret,
        } = entry;
        let label = normalize_label(&entry_label)?;
        let secret = Zeroizing::new(secret);
        if secret.trim().is_empty() {
            return Err(VaultError::EmptySecret);
        }
        self.with_storage_lock(|| {
            let mut index = self.load_index()?;
            if index.credentials.contains_key(&label) {
                return Err(VaultError::CredentialExists { label });
            }

            let now = Utc::now().timestamp();
            let meta = VaultCredentialMeta {
                label: label.clone(),
                credential_type,
                provider,
                notes,
                revocation_notes,
                created_at: now,
                updated_at: now,
                storage_backend: StorageBackend::EncryptedSecrets,
            };

            self.write_secret(&label, secret.as_str())?;
            index.credentials.insert(label, meta);
            self.save_index(&index)?;
            Ok(())
        })
    }

    /// Update an existing credential's secret and optional metadata fields.
    /// Only fields that are `Some` are overwritten.
    pub fn update(
        &self,
        label: &str,
        secret: Option<String>,
        provider: Option<Option<String>>,
        notes: Option<Option<String>>,
        revocation_notes: Option<Option<String>>,
    ) -> Result<VaultCredentialMeta, VaultError> {
        let label = normalize_label(label)?;
        self.with_storage_lock(|| {
            let mut index = self.load_index()?;
            let meta = index
                .credentials
                .get_mut(&label)
                .ok_or_else(|| VaultError::NotFound {
                    label: label.clone(),
                })?;

            if let Some(secret) = secret {
                let secret = Zeroizing::new(secret);
                if secret.trim().is_empty() {
                    return Err(VaultError::EmptySecret);
                }
                self.write_secret(&label, secret.as_str())?;
            }
            if let Some(provider) = provider {
                meta.provider = provider;
            }
            if let Some(notes) = notes {
                meta.notes = notes;
            }
            if let Some(revocation_notes) = revocation_notes {
                meta.revocation_notes = revocation_notes;
            }
            meta.updated_at = Utc::now().timestamp();
            let updated = meta.clone();
            self.save_index(&index)?;
            Ok(updated)
        })
    }

    /// Delete a credential and its secret. Returns `true` if anything was removed.
    pub fn delete(&self, label: &str) -> Result<bool, VaultError> {
        let normalized = normalize_label(label)?;
        self.with_storage_lock(|| {
            let mut index = self.load_index()?;
            let removed_meta = index.credentials.remove(&normalized).is_some();
            if removed_meta {
                self.save_index(&index)?;
            }
            let removed_secret = self.delete_secret(&normalized)?;
            Ok(removed_meta || removed_secret)
        })
    }

    /// Delete several credentials in one encrypted-store transaction.
    pub fn delete_many(&self, labels: &[String]) -> Result<usize, VaultError> {
        let normalized = labels
            .iter()
            .map(|label| normalize_label(label))
            .collect::<Result<Vec<_>, _>>()?;
        self.with_storage_lock(|| {
            let mut index = self.load_index()?;
            let mut secret_entries = Vec::with_capacity(normalized.len());
            let mut removed_metadata = 0;
            for label in &normalized {
                removed_metadata += usize::from(index.credentials.remove(label).is_some());
                secret_entries.push((VAULT_SCOPE.clone(), secret_name_for(label)?));
            }
            if removed_metadata > 0 {
                self.save_index(&index)?;
            }
            let removed_secrets = self.secrets.delete_many(&secret_entries)?;
            Ok(removed_metadata.max(removed_secrets))
        })
    }

    /// List metadata for all stored credentials (no secret material).
    pub fn list(&self) -> Result<Vec<VaultCredentialMeta>, VaultError> {
        self.with_storage_lock(|| {
            let index = self.load_index()?;
            Ok(index.credentials.into_values().collect())
        })
    }

    /// Return metadata for a single credential (no secret material).
    pub fn show(&self, label: &str) -> Result<VaultCredentialMeta, VaultError> {
        let normalized = normalize_label(label)?;
        self.with_storage_lock(|| {
            let index = self.load_index()?;
            index
                .credentials
                .get(&normalized)
                .cloned()
                .ok_or_else(|| VaultError::NotFound {
                    label: normalized.clone(),
                })
        })
    }

    /// Reveal the raw secret value for a credential. Only call this from an
    /// explicit user action (`/vault credential reveal` / `/vault credential export`).
    pub fn reveal(&self, label: &str) -> Result<String, VaultError> {
        let normalized = normalize_label(label)?;
        if normalized == MANAGED_CLAUDE_TOKEN_LABEL {
            return Err(VaultError::ProviderManagedCredential { label: normalized });
        }
        self.with_storage_lock(|| {
            // Validate the label exists in the index before attempting decryption.
            let index = self.load_index()?;
            if !index.credentials.contains_key(&normalized) {
                return Err(VaultError::NotFound { label: normalized });
            }
            self.read_secret(&normalized)?
                .ok_or_else(|| VaultError::NotFound { label: normalized })
        })
    }

    /// Resolve an operational credential for immediate programmatic use.
    ///
    /// The caller must keep the returned value out of model context, logs, and
    /// persistent process state. Key custody material is rejected even when it
    /// exists in the vault.
    pub fn reveal_for_programmatic_use(
        &self,
        label: &str,
        security_level: SecurityLevel,
    ) -> Result<String, VaultError> {
        if security_level != SecurityLevel::Permissive {
            return Err(VaultError::ProgrammaticUseSecurityLevelDenied {
                level: security_level,
            });
        }
        let normalized = normalize_label(label)?;
        if normalized == MANAGED_CLAUDE_TOKEN_LABEL {
            return Err(VaultError::ProviderManagedCredential { label: normalized });
        }
        self.with_storage_lock(|| {
            let index = self.load_index()?;
            let meta = index
                .credentials
                .get(&normalized)
                .ok_or_else(|| VaultError::NotFound {
                    label: normalized.clone(),
                })?;
            if !meta.credential_type.permits_programmatic_use() {
                return Err(VaultError::ProgrammaticUseDenied {
                    label: normalized,
                    credential_type: meta.credential_type,
                });
            }
            self.read_secret(&normalized)?
                .ok_or_else(|| VaultError::NotFound { label: normalized })
        })
    }

    /// Whether a credential with the given label exists.
    pub fn exists(&self, label: &str) -> Result<bool, VaultError> {
        let normalized = normalize_label(label)?;
        self.with_storage_lock(|| {
            let index = self.load_index()?;
            Ok(index.credentials.contains_key(&normalized))
        })
    }

    // ---- index helpers ----

    fn with_storage_lock<T>(
        &self,
        operation: impl FnOnce() -> Result<T, VaultError>,
    ) -> Result<T, VaultError> {
        let secrets_dir = self.codex_home.join("secrets");
        std::fs::create_dir_all(&secrets_dir)
            .context("failed to create vault secrets directory")?;
        let lock_path = secrets_dir.join(VAULT_LOCK_FILE_NAME);
        let lock_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&lock_path)
            .with_context(|| format!("failed to open vault lock {}", lock_path.display()))?;
        lock_file
            .lock()
            .with_context(|| format!("failed to lock vault {}", lock_path.display()))?;
        operation()
    }

    fn load_index(&self) -> Result<VaultIndex, VaultError> {
        let name = index_secret_name()?;
        match self.secrets.get(&VAULT_SCOPE, &name)? {
            Some(serialized) => {
                let mut index: VaultIndex = serde_json::from_str(&serialized)
                    .context("failed to deserialize vault index")?;
                if index.version == 0 {
                    index.version = VAULT_SCHEMA_VERSION;
                }
                if index.version > VAULT_SCHEMA_VERSION {
                    return Err(VaultError::Storage(anyhow::anyhow!(
                        "vault index version {} is newer than supported version {}",
                        index.version,
                        VAULT_SCHEMA_VERSION
                    )));
                }
                Ok(index)
            }
            None => Ok(VaultIndex::default()),
        }
    }

    fn save_index(&self, index: &VaultIndex) -> Result<(), VaultError> {
        let name = index_secret_name()?;
        let serialized = serde_json::to_string(index).context("failed to serialize vault index")?;
        self.secrets.set(&VAULT_SCOPE, &name, &serialized)?;
        Ok(())
    }

    // ---- per-credential secret helpers ----

    fn write_secret(&self, label: &str, secret: &str) -> Result<(), VaultError> {
        let name = secret_name_for(label)?;
        self.secrets.set(&VAULT_SCOPE, &name, secret)?;
        Ok(())
    }

    fn read_secret(&self, label: &str) -> Result<Option<String>, VaultError> {
        let name = secret_name_for(label)?;
        Ok(self.secrets.get(&VAULT_SCOPE, &name)?)
    }

    fn delete_secret(&self, label: &str) -> Result<bool, VaultError> {
        let name = secret_name_for(label)?;
        Ok(self.secrets.delete(&VAULT_SCOPE, &name)?)
    }
}

/// Validate and canonicalize a credential label.
///
/// Labels must be 1..=128 chars and may contain ASCII alphanumerics, `-`, `_`,
/// `.`, and `/` (so provider-scoped labels like `ambient/api-key` work). They
/// are trimmed for stable lookups; case-distinct labels remain distinct.
fn normalize_label(raw: &str) -> Result<String, VaultError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(VaultError::InvalidLabel(
            "label must not be empty".to_string(),
        ));
    }
    if trimmed.len() > 128 {
        return Err(VaultError::InvalidLabel(
            "label must be at most 128 chars".to_string(),
        ));
    }
    if !trimmed
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' || c == '/')
    {
        return Err(VaultError::InvalidLabel(
            "label may only contain a-z, 0-9, '-', '_', '.', '/'".to_string(),
        ));
    }
    Ok(trimmed.to_string())
}

/// Convert a label into a collision-resistant `SecretName`.
///
/// The label is SHA-256 hashed and rendered as uppercase hex (A-F, 0-9), which satisfies the
/// `SecretName` charset (`A-Z`, `0-9`, `_`). Hashing (rather than uppercasing the raw label)
/// guarantees that distinct labels like `a/b`, `a.b`, `a_b`, `a-b`, `MyLabel`, and `mylabel` map
/// to distinct secret records, so adding one can never overwrite another's stored secret.
fn secret_name_for(label: &str) -> Result<SecretName, VaultError> {
    let mut hasher = Sha256::new();
    hasher.update(label.as_bytes());
    let digest = hasher.finalize();
    let hex = format!("{digest:X}");
    // 64 hex chars is collision-resistant; truncate to a still-ample 32 chars (128 bits).
    let fingerprint = hex.get(..32).unwrap_or(hex.as_str());
    let candidate = format!("VAULT_{fingerprint}");
    SecretName::new(&candidate).map_err(|err| VaultError::InvalidLabel(err.to_string()))
}

fn index_secret_name() -> Result<SecretName, VaultError> {
    SecretName::new(VAULT_INDEX_SECRET_NAME).map_err(|err| {
        VaultError::Storage(anyhow::anyhow!("invalid vault index secret name: {err}"))
    })
}

/// Format a Unix-seconds timestamp as an ISO-8601 string for display.
pub fn format_timestamp(seconds: i64) -> String {
    DateTime::<Utc>::from_timestamp(seconds, 0)
        .map(|dt| dt.to_rfc3339_opts(chrono::SecondsFormat::Secs, /*use_z*/ true))
        .unwrap_or_else(|| format!("unix {seconds}"))
}
