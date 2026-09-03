//! Metadata-only Claude subscription authentication selection and health contracts.

use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;
use sha2::Digest;
use sha2::Sha256;
use std::fmt;
use std::path::Path;
use std::path::PathBuf;
use thiserror::Error;
use unicode_normalization::UnicodeNormalization;
use uuid::Uuid;
use zeroize::Zeroizing;

use crate::CredentialType;
use crate::VAULT_SCOPE;
use crate::Vault;
use crate::VaultError;
use crate::index_secret_entry;
use crate::secret_name_for;
use codex_secrets::SecretName;
use codex_secrets::SecretScope;

const CLAUDE_AUTH_SELECTION_SECRET_NAME: &str = "CLAUDE_AUTH_SELECTION";
const CLAUDE_AUTH_SELECTION_VERSION: u32 = 1;
const MAX_SOURCE_ID_BYTES: usize = 256;
const CLAUDE_LOGIN_AUTHORITY_PREFIX: &str = "claude-login-authority:sha256:";
const CLAUDE_ENVIRONMENT_TOKEN_AUTHORITY_PREFIX: &str =
    "claude-environment-token-authority:sha256:";
const MAX_MANAGED_TOKEN_BYTES: usize = 16 * 1024;
const CLAUDE_AUTH_SELECTION_SENTINEL_FILE: &str = ".claude-auth-selection-present";
const CLAUDE_AUTH_SELECTION_REVISION_FILE: &str = ".claude-auth-selection-revision";

/// Non-secret sentinel distinguishing explicit selection from legacy discovery.
pub fn claude_auth_selection_sentinel_path(codex_home: &Path) -> PathBuf {
    codex_home.join(CLAUDE_AUTH_SELECTION_SENTINEL_FILE)
}

/// Non-secret revision marker used to invalidate command-backed bearer caches.
pub fn claude_auth_selection_revision_path(codex_home: &Path) -> PathBuf {
    codex_home.join(CLAUDE_AUTH_SELECTION_REVISION_FILE)
}

fn ensure_claude_auth_selection_sentinel(codex_home: &Path) -> Result<(), VaultError> {
    std::fs::create_dir_all(codex_home).map_err(|error| {
        VaultError::Storage(anyhow::anyhow!(
            "failed to create Claude auth selection directory: {error}"
        ))
    })?;
    let path = claude_auth_selection_sentinel_path(codex_home);
    match std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
    {
        Ok(file) => file.sync_all().map_err(|error| {
            VaultError::Storage(anyhow::anyhow!(
                "failed to persist Claude auth selection sentinel: {error}"
            ))
        }),
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            let metadata = std::fs::symlink_metadata(&path).map_err(|metadata_error| {
                VaultError::Storage(anyhow::anyhow!(
                    "failed to inspect Claude auth selection sentinel: {metadata_error}"
                ))
            })?;
            if metadata.file_type().is_file() {
                Ok(())
            } else {
                Err(VaultError::Storage(anyhow::anyhow!(
                    "Claude auth selection sentinel is not a regular file"
                )))
            }
        }
        Err(error) => Err(VaultError::Storage(anyhow::anyhow!(
            "failed to create Claude auth selection sentinel: {error}"
        ))),
    }
}

struct PreparedClaudeAuthRevision {
    path: PathBuf,
    temporary_path: PathBuf,
}

impl PreparedClaudeAuthRevision {
    fn new(codex_home: &Path) -> Result<Self, VaultError> {
        std::fs::create_dir_all(codex_home).map_err(|error| {
            VaultError::Storage(anyhow::anyhow!(
                "failed to create Claude auth revision directory: {error}"
            ))
        })?;
        let path = claude_auth_selection_revision_path(codex_home);
        let temporary_path = codex_home.join(format!(
            ".claude-auth-selection-revision.{}.tmp",
            Uuid::new_v4()
        ));
        std::fs::write(&temporary_path, Uuid::new_v4().to_string()).map_err(|error| {
            VaultError::Storage(anyhow::anyhow!(
                "failed to prepare Claude auth cache revision: {error}"
            ))
        })?;
        Ok(Self {
            path,
            temporary_path,
        })
    }

    fn commit(mut self) -> Result<(), VaultError> {
        match std::fs::remove_file(&self.path) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                self.invalidate();
                return Err(VaultError::Storage(anyhow::anyhow!(
                    "failed to replace Claude auth cache revision: {error}"
                )));
            }
        }
        if let Err(error) = std::fs::rename(&self.temporary_path, &self.path) {
            self.invalidate();
            return Err(VaultError::Storage(anyhow::anyhow!(
                "failed to publish Claude auth cache revision: {error}"
            )));
        }
        self.temporary_path = PathBuf::new();
        Ok(())
    }

    fn invalidate(&self) {
        let _ = std::fs::remove_file(&self.path);
        if !self.temporary_path.as_os_str().is_empty() {
            let _ = std::fs::remove_file(&self.temporary_path);
        }
    }
}

impl Drop for PreparedClaudeAuthRevision {
    fn drop(&mut self) {
        if !self.temporary_path.as_os_str().is_empty() {
            let _ = std::fs::remove_file(&self.temporary_path);
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EnrollmentFailurePoint {
    None,
    #[cfg(test)]
    AfterTokenWrite,
    #[cfg(test)]
    AfterIndexWrite,
}

/// Stable encrypted-vault label for Corbanu's managed Claude subscription token.
pub const MANAGED_CLAUDE_TOKEN_LABEL: &str = "provider/claude-code-oauth-token";
pub const MANAGED_CLAUDE_AUTH_SOURCE_ID: &str = "corbanu-vault:claude-plan";
pub const ENVIRONMENT_CLAUDE_AUTH_SOURCE_ID: &str = "environment:CLAUDE_CODE_OAUTH_TOKEN";
pub const MACOS_KEYCHAIN_CLAUDE_AUTH_SOURCE_ID: &str = "claude-login:macos-keychain";
pub const CREDENTIALS_FILE_CLAUDE_AUTH_SOURCE_ID: &str = "claude-login:credentials-file";

/// Match Claude Code's macOS Keychain service identity for one configuration profile.
pub fn claude_code_macos_keychain_service(
    config_dir: &Path,
    config_dir_overridden: bool,
    custom_oauth: bool,
) -> String {
    let oauth_suffix = if custom_oauth { "-custom-oauth" } else { "" };
    let config_suffix = if config_dir_overridden {
        let normalized = config_dir.to_string_lossy().nfc().collect::<String>();
        let digest = format!("{:x}", Sha256::digest(normalized.as_bytes()));
        format!("-{}", &digest[..8])
    } else {
        String::new()
    };
    format!("Claude Code{oauth_suffix}-credentials{config_suffix}")
}

/// Persist the exact Keychain service, so a later profile change cannot drift accounts.
pub fn macos_keychain_claude_auth_source_id(service: &str) -> String {
    format!("{MACOS_KEYCHAIN_CLAUDE_AUTH_SOURCE_ID}:{service}")
}

/// Persist the exact credentials-file profile without exposing its filesystem path.
///
/// Making relative paths absolute is important because Claude Code resolves a relative
/// `CLAUDE_CONFIG_DIR` against the process working directory. NFC normalization keeps
/// equivalent Unicode spellings aligned across persistence and provider resolution.
pub fn credentials_file_claude_auth_source_id(config_dir: &Path) -> std::io::Result<String> {
    credentials_file_claude_auth_source_id_against(config_dir, &std::env::current_dir()?)
}

/// Bind a Claude-owned login slot to the account identity reported by Claude Code.
///
/// The returned digest is persisted only inside the encrypted selection record. The raw email,
/// optional organization identifier, and optional subscription name are not persisted in that
/// selection record. Claude Code versions do not all report the optional fields, so a normalized
/// email is the minimum stable identity. When both optional fields are present, the original v1
/// digest is retained for compatibility with already-persisted selections.
pub fn claude_login_authority_id(
    email: &str,
    organization_id: Option<&str>,
    subscription_type: Option<&str>,
) -> Result<String, String> {
    let email = email.trim().nfc().collect::<String>().to_lowercase();
    let organization_id = organization_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.nfc().collect::<String>().to_lowercase());
    let subscription_type = subscription_type
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.nfc().collect::<String>().to_lowercase());
    if email.is_empty() {
        return Err("Claude login account identity is incomplete".to_string());
    }
    let mut digest = Sha256::new();
    if let (Some(organization_id), Some(subscription_type)) =
        (organization_id.as_deref(), subscription_type.as_deref())
    {
        digest.update(b"corbanu:claude-login-authority:v1\0");
        digest.update(organization_id.as_bytes());
        digest.update(b"\0");
        digest.update(email.as_bytes());
        digest.update(b"\0");
        digest.update(subscription_type.as_bytes());
    } else {
        digest.update(b"corbanu:claude-login-authority:v2\0");
        digest.update(email.as_bytes());
        digest.update(b"\0organization\0");
        digest.update(organization_id.as_deref().unwrap_or_default().as_bytes());
        digest.update(b"\0subscription\0");
        digest.update(subscription_type.as_deref().unwrap_or_default().as_bytes());
    }
    Ok(format!(
        "{CLAUDE_LOGIN_AUTHORITY_PREFIX}{:x}",
        digest.finalize()
    ))
}

/// Bind an explicitly selected environment token without persisting its value.
///
/// The digest makes a later shell/profile token replacement fail closed while
/// leaving selection-less legacy environment resolution unchanged.
pub fn claude_environment_token_authority_id(token: &str) -> String {
    let token = token.trim();
    let mut digest = Sha256::new();
    digest.update(b"corbanu:claude-environment-token-authority:v1\0");
    digest.update(token.as_bytes());
    format!(
        "{CLAUDE_ENVIRONMENT_TOKEN_AUTHORITY_PREFIX}{:x}",
        digest.finalize()
    )
}

fn credentials_file_claude_auth_source_id_against(
    config_dir: &Path,
    base_dir: &Path,
) -> std::io::Result<String> {
    let path = if config_dir.is_absolute() {
        config_dir.to_path_buf()
    } else {
        base_dir.join(config_dir)
    };
    let absolute = std::path::absolute(path)?;
    let normalized = absolute.to_string_lossy().nfc().collect::<String>();
    let digest = format!("{:x}", Sha256::digest(normalized.as_bytes()));
    Ok(format!("{CREDENTIALS_FILE_CLAUDE_AUTH_SOURCE_ID}:{digest}"))
}

/// Metadata-only status for the managed long-lived token.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManagedClaudeTokenStatus {
    Missing,
    Stored { updated_at: i64 },
}

/// Secret-free failures from the managed token lifecycle.
#[derive(Debug, Error)]
pub enum ClaudeSubscriptionTokenError {
    #[error("Claude subscription token is empty")]
    Empty,
    #[error("Claude subscription token must be one bounded line")]
    InvalidFormat,
    #[error(transparent)]
    Vault(#[from] VaultError),
}

/// A credential source the user can explicitly select for Claude Plan requests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ClaudeAuthSource {
    /// A one-year subscription token generated by `claude setup-token` and held by Corbanu.
    ManagedSubscriptionToken,
    /// The token supplied explicitly through `CLAUDE_CODE_OAUTH_TOKEN`.
    EnvironmentToken,
    /// Claude Code's platform-owned `/login` credential.
    ClaudeCodeLogin,
}

/// Non-secret storage location metadata for a discovered source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ClaudeAuthStoreKind {
    CorbanuVault,
    Environment,
    MacosKeychain,
    CredentialsFile,
    LegacyCredentialsFile,
}

/// Health of one discovered source without including credential material.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ClaudeAuthHealth {
    Healthy,
    NeedsEnrollment,
    NeedsReauthorization,
    Missing,
    Malformed,
    Unavailable,
}

/// Persisted user choice. `source_id` names a store/account slot, never a secret.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaudeAuthSelection {
    version: u32,
    pub source: ClaudeAuthSource,
    pub source_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authority_id: Option<String>,
    pub selected_at: i64,
}

impl fmt::Debug for ClaudeAuthSelection {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ClaudeAuthSelection")
            .field("version", &self.version)
            .field("source", &self.source)
            .field("source_id", &self.source_id)
            .field("authority_bound", &self.authority_id.is_some())
            .field("selected_at", &self.selected_at)
            .finish()
    }
}

impl ClaudeAuthSelection {
    pub fn new(source: ClaudeAuthSource, source_id: impl Into<String>) -> Result<Self, String> {
        Self::new_at(source, source_id, Utc::now().timestamp())
    }

    pub fn new_at(
        source: ClaudeAuthSource,
        source_id: impl Into<String>,
        selected_at: i64,
    ) -> Result<Self, String> {
        let selection = Self {
            version: CLAUDE_AUTH_SELECTION_VERSION,
            source,
            source_id: source_id.into(),
            authority_id: None,
            selected_at,
        };
        selection.validate()?;
        Ok(selection)
    }

    pub fn new_claude_code_login(
        source_id: impl Into<String>,
        authority_id: impl Into<String>,
    ) -> Result<Self, String> {
        let mut selection = Self::new(ClaudeAuthSource::ClaudeCodeLogin, source_id)?;
        selection.authority_id = Some(authority_id.into());
        selection.validate()?;
        Ok(selection)
    }

    pub fn new_environment_token(token: &str) -> Result<Self, String> {
        if token.trim().is_empty() {
            return Err("Claude environment token is empty".to_string());
        }
        let mut selection = Self::new(
            ClaudeAuthSource::EnvironmentToken,
            ENVIRONMENT_CLAUDE_AUTH_SOURCE_ID,
        )?;
        selection.authority_id = Some(claude_environment_token_authority_id(token));
        selection.validate()?;
        Ok(selection)
    }

    fn validate(&self) -> Result<(), String> {
        if self.version != CLAUDE_AUTH_SELECTION_VERSION {
            return Err(format!(
                "Claude authentication selection version {} is unsupported",
                self.version
            ));
        }
        let id = self.source_id.trim();
        if id.is_empty() || id.len() > MAX_SOURCE_ID_BYTES {
            return Err("Claude authentication source id is invalid".to_string());
        }
        if id.chars().any(char::is_control) {
            return Err("Claude authentication source id contains control characters".to_string());
        }
        if let Some(authority_id) = self.authority_id.as_deref() {
            let valid = match self.source {
                ClaudeAuthSource::ClaudeCodeLogin => authority_id
                    .strip_prefix(CLAUDE_LOGIN_AUTHORITY_PREFIX)
                    .is_some_and(valid_claude_authority_digest),
                ClaudeAuthSource::EnvironmentToken => authority_id
                    .strip_prefix(CLAUDE_ENVIRONMENT_TOKEN_AUTHORITY_PREFIX)
                    .is_some_and(valid_claude_authority_digest),
                ClaudeAuthSource::ManagedSubscriptionToken => false,
            };
            if !valid {
                return Err("Claude authentication authority id is invalid".to_string());
            }
        }
        Ok(())
    }
}

fn valid_claude_authority_digest(digest: &str) -> bool {
    digest.len() == 64 && digest.bytes().all(|byte| byte.is_ascii_hexdigit())
}

/// Safe status for a source. Account hints are display metadata such as an email address.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaudeAuthSourceMetadata {
    pub source: ClaudeAuthSource,
    pub source_id: String,
    pub store: ClaudeAuthStoreKind,
    pub health: ClaudeAuthHealth,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub account_hint: Option<String>,
}

/// Deterministic outcome of resolving an explicit selection against discovered metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClaudeAuthResolution {
    SelectionRequired {
        available: Vec<ClaudeAuthSourceMetadata>,
    },
    Selected(ClaudeAuthSourceMetadata),
    MissingSelected(ClaudeAuthSelection),
    UnhealthySelected(ClaudeAuthSourceMetadata),
    Conflict {
        selection: ClaudeAuthSelection,
        matches: Vec<ClaudeAuthSourceMetadata>,
    },
}

/// Resolve only the exact selected source. This function never falls through to another source.
pub fn resolve_claude_auth_source(
    selection: Option<&ClaudeAuthSelection>,
    discovered: &[ClaudeAuthSourceMetadata],
) -> ClaudeAuthResolution {
    let mut discovered = discovered.to_vec();
    discovered.sort_by(|left, right| {
        (&left.source, &left.source_id, &left.store, &left.health).cmp(&(
            &right.source,
            &right.source_id,
            &right.store,
            &right.health,
        ))
    });
    let Some(selection) = selection else {
        return ClaudeAuthResolution::SelectionRequired {
            available: discovered,
        };
    };
    let matches = discovered
        .into_iter()
        .filter(|candidate| {
            candidate.source == selection.source && candidate.source_id == selection.source_id
        })
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [] => ClaudeAuthResolution::MissingSelected(selection.clone()),
        [selected] if selected.health == ClaudeAuthHealth::Healthy => {
            ClaudeAuthResolution::Selected(selected.clone())
        }
        [selected] => ClaudeAuthResolution::UnhealthySelected(selected.clone()),
        _ => ClaudeAuthResolution::Conflict {
            selection: selection.clone(),
            matches,
        },
    }
}

impl Vault {
    /// Load the selected Claude source. Absence preserves legacy installations unchanged.
    pub fn load_claude_auth_selection(&self) -> Result<Option<ClaudeAuthSelection>, VaultError> {
        self.with_storage_lock(|| {
            let name = claude_auth_selection_secret_name()?;
            let Some(serialized) = self.secrets.get(&SecretScope::Global, &name)? else {
                return Ok(None);
            };
            let selection: ClaudeAuthSelection = serde_json::from_str(&serialized)
                .map_err(|error| VaultError::Storage(error.into()))?;
            selection
                .validate()
                .map_err(|error| VaultError::Storage(anyhow::anyhow!(error)))?;
            Ok(Some(selection))
        })
    }

    /// Atomically persist a metadata-only source choice in the encrypted state substrate.
    pub fn save_claude_auth_selection(
        &self,
        selection: &ClaudeAuthSelection,
    ) -> Result<(), VaultError> {
        selection
            .validate()
            .map_err(|error| VaultError::Storage(anyhow::anyhow!(error)))?;
        // Publish the non-secret opt-in boundary before touching encrypted
        // selection state. If the later write fails, resolution remains
        // fail-closed instead of silently returning to legacy env-first auth.
        ensure_claude_auth_selection_sentinel(&self.codex_home)?;
        let revision = PreparedClaudeAuthRevision::new(&self.codex_home)?;
        let result = self.with_storage_lock(|| {
            let name = claude_auth_selection_secret_name()?;
            let serialized = serde_json::to_string(selection)
                .map_err(|error| VaultError::Storage(error.into()))?;
            self.secrets.set(&SecretScope::Global, &name, &serialized)?;
            Ok(())
        });
        match result {
            Ok(()) => revision.commit(),
            Err(error) => {
                revision.invalidate();
                Err(error)
            }
        }
    }

    /// Store or replace the managed token without exposing it through metadata.
    pub fn store_managed_claude_subscription_token(
        &self,
        token: String,
    ) -> Result<ManagedClaudeTokenStatus, ClaudeSubscriptionTokenError> {
        let token = Zeroizing::new(token);
        validate_managed_token(token.as_str())?;
        // A missing marker denotes selection-less legacy behavior and must stay
        // uncached. Generic token storage only invalidates an already-persisted
        // selection; enrollment below creates the first marker atomically with
        // that selection.
        let revision = claude_auth_selection_revision_path(&self.codex_home)
            .is_file()
            .then(|| PreparedClaudeAuthRevision::new(&self.codex_home))
            .transpose()?;
        let result = self.with_storage_lock(|| {
            let mut index = self.load_index()?;
            let now = Utc::now().timestamp();
            upsert_managed_token_metadata(&mut index, now)?;
            let updates = vec![
                (
                    VAULT_SCOPE.clone(),
                    secret_name_for(MANAGED_CLAUDE_TOKEN_LABEL)?,
                    token.to_string(),
                ),
                index_secret_entry(&index)?,
            ];
            self.secrets.apply_batch(&updates, &[])?;
            Ok(ManagedClaudeTokenStatus::Stored { updated_at: now })
        });
        match result {
            Ok(status) => {
                if let Some(revision) = revision {
                    revision.commit()?;
                }
                Ok(status)
            }
            Err(error) => {
                if let Some(revision) = revision {
                    revision.invalidate();
                }
                Err(error.into())
            }
        }
    }

    /// Atomically enroll a managed token and select it for Claude Plan requests.
    ///
    /// If any encrypted-store write fails, the previous token, metadata index,
    /// and source selection are restored before the error is returned.
    pub fn enroll_managed_claude_subscription_token(
        &self,
        token: String,
    ) -> Result<ClaudeAuthSelection, ClaudeSubscriptionTokenError> {
        self.enroll_managed_claude_subscription_token_at(
            token,
            Utc::now().timestamp(),
            EnrollmentFailurePoint::None,
        )
    }

    fn enroll_managed_claude_subscription_token_at(
        &self,
        token: String,
        selected_at: i64,
        failure_point: EnrollmentFailurePoint,
    ) -> Result<ClaudeAuthSelection, ClaudeSubscriptionTokenError> {
        #[cfg(not(test))]
        let _ = failure_point;
        let token = Zeroizing::new(token);
        validate_managed_token(token.as_str())?;
        let selection = ClaudeAuthSelection::new_at(
            ClaudeAuthSource::ManagedSubscriptionToken,
            MANAGED_CLAUDE_AUTH_SOURCE_ID,
            selected_at,
        )
        .map_err(|error| VaultError::Storage(anyhow::anyhow!(error)))?;

        ensure_claude_auth_selection_sentinel(&self.codex_home)?;
        let revision = PreparedClaudeAuthRevision::new(&self.codex_home)?;

        let result = self.with_storage_lock(|| {
            let selection_name = claude_auth_selection_secret_name()?;
            let mut next_index = self.load_index()?;
            let now = Utc::now().timestamp();
            upsert_managed_token_metadata(&mut next_index, now)?;
            let serialized_selection = serde_json::to_string(&selection)
                .map_err(|error| VaultError::Storage(error.into()))?;

            #[cfg(test)]
            match failure_point {
                EnrollmentFailurePoint::AfterTokenWrite => {
                    return Err(VaultError::Storage(anyhow::anyhow!(
                        "injected managed-token enrollment failure before atomic commit (token checkpoint)"
                    )));
                }
                EnrollmentFailurePoint::AfterIndexWrite => {
                    return Err(VaultError::Storage(anyhow::anyhow!(
                        "injected managed-token enrollment failure before atomic commit (index checkpoint)"
                    )));
                }
                EnrollmentFailurePoint::None => {}
            }

            let updates = vec![
                (
                    VAULT_SCOPE.clone(),
                    secret_name_for(MANAGED_CLAUDE_TOKEN_LABEL)?,
                    token.to_string(),
                ),
                index_secret_entry(&next_index)?,
                (VAULT_SCOPE.clone(), selection_name, serialized_selection),
            ];
            self.secrets.apply_batch(&updates, &[])?;

            Ok(selection.clone())
        });
        match result {
            Ok(selection) => {
                revision.commit()?;
                Ok(selection)
            }
            Err(error) => {
                revision.invalidate();
                Err(error.into())
            }
        }
    }

    /// Inspect only managed-token metadata.
    pub fn managed_claude_subscription_token_status(
        &self,
    ) -> Result<ManagedClaudeTokenStatus, VaultError> {
        match self.show(MANAGED_CLAUDE_TOKEN_LABEL) {
            Ok(metadata) => Ok(ManagedClaudeTokenStatus::Stored {
                updated_at: metadata.updated_at,
            }),
            Err(VaultError::NotFound { .. }) => Ok(ManagedClaudeTokenStatus::Missing),
            Err(error) => Err(error),
        }
    }

    /// Load the managed token into zeroizing owned memory for trusted provider use.
    pub fn load_managed_claude_subscription_token(&self) -> Result<Zeroizing<String>, VaultError> {
        self.with_storage_lock(|| {
            let token = self
                .read_secret(MANAGED_CLAUDE_TOKEN_LABEL)?
                .ok_or_else(|| VaultError::NotFound {
                    label: MANAGED_CLAUDE_TOKEN_LABEL.to_string(),
                })?;
            Ok(Zeroizing::new(token))
        })
    }

    /// Remove only Corbanu's managed copy. This does not claim server-side revocation.
    pub fn remove_managed_claude_subscription_token(&self) -> Result<bool, VaultError> {
        let revision = claude_auth_selection_revision_path(&self.codex_home)
            .is_file()
            .then(|| PreparedClaudeAuthRevision::new(&self.codex_home))
            .transpose()?;
        match self.delete_normalized(MANAGED_CLAUDE_TOKEN_LABEL) {
            Ok(removed) => {
                if let Some(revision) = revision {
                    revision.commit()?;
                }
                Ok(removed)
            }
            Err(error) => {
                if let Some(revision) = revision {
                    revision.invalidate();
                }
                Err(error)
            }
        }
    }
}

fn upsert_managed_token_metadata(
    index: &mut crate::VaultIndex,
    now: i64,
) -> Result<(), VaultError> {
    if let Some(metadata) = index.credentials.get_mut(MANAGED_CLAUDE_TOKEN_LABEL) {
        metadata.credential_type = CredentialType::BearerToken;
        metadata.provider = Some("claude-plan".to_string());
        metadata.notes = Some("Managed long-lived Claude subscription token".to_string());
        metadata.revocation_notes = Some(
            "Local removal does not revoke the token; generate a replacement with `claude setup-token`"
                .to_string(),
        );
        metadata.updated_at = now;
    } else {
        let label = crate::normalize_label(MANAGED_CLAUDE_TOKEN_LABEL)?;
        let metadata = crate::VaultCredentialMeta {
            label: label.clone(),
            credential_type: CredentialType::BearerToken,
            provider: Some("claude-plan".to_string()),
            notes: Some("Managed long-lived Claude subscription token".to_string()),
            revocation_notes: Some(
                "Local removal does not revoke the token; generate a replacement with `claude setup-token`"
                    .to_string(),
            ),
            created_at: now,
            updated_at: now,
            storage_backend: crate::StorageBackend::EncryptedSecrets,
        };
        index.credentials.insert(label, metadata);
    }
    Ok(())
}

fn validate_managed_token(token: &str) -> Result<(), ClaudeSubscriptionTokenError> {
    let trimmed = token.trim();
    if trimmed.is_empty() {
        return Err(ClaudeSubscriptionTokenError::Empty);
    }
    if trimmed != token
        || token.len() > MAX_MANAGED_TOKEN_BYTES
        || token.chars().any(char::is_whitespace)
        || token.chars().any(char::is_control)
    {
        return Err(ClaudeSubscriptionTokenError::InvalidFormat);
    }
    Ok(())
}

fn claude_auth_selection_secret_name() -> Result<SecretName, VaultError> {
    SecretName::new(CLAUDE_AUTH_SELECTION_SECRET_NAME).map_err(|error| {
        VaultError::Storage(anyhow::anyhow!(
            "invalid Claude authentication selection secret name: {error}"
        ))
    })
}

#[cfg(test)]
#[path = "claude_auth_tests.rs"]
mod tests;
