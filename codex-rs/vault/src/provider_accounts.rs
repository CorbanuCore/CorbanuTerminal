//! PF-84 named accounts per provider.
//!
//! An account is a `(provider id, name)` pair. Today's credentials are the
//! implicit `default` account and keep their existing labels; nothing here
//! touches them. Named accounts live in the same per-home vault under
//! `provider/<provider id>/accounts/<name>/<kind>`, so they add no keychain
//! item: the vault key stays the home's single item. The set of these labels is
//! the encrypted account registry.

use std::collections::BTreeMap;
use std::fmt;

use chrono::Utc;
use zeroize::Zeroizing;

use crate::CredentialType;
use crate::StorageBackend;
use crate::VAULT_SCOPE;
use crate::Vault;
use crate::VaultCredentialMeta;
use crate::VaultError;
use crate::index_secret_entry;
use crate::normalize_label;
use crate::secret_name_for;

/// The implicit account that owns today's (unqualified) credentials.
pub const DEFAULT_PROVIDER_ACCOUNT: &str = "default";
const ACCOUNT_LABEL_PREFIX: &str = "provider/";
const ACCOUNTS_SEGMENT: &str = "accounts";
const MAX_ACCOUNT_NAME_BYTES: usize = 32;
const MAX_PROVIDER_ID_BYTES: usize = 48;

/// Why an account name or provider id was rejected.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ProviderAccountError {
    #[error(
        "invalid account name: use 1-32 lowercase letters, digits or '-', starting with a letter or digit"
    )]
    InvalidName,
    #[error("`default` names today's credentials; choose another account name")]
    ReservedName,
    #[error(
        "this provider id cannot hold named accounts: use lowercase letters, digits, '-', '_' or '.'"
    )]
    InvalidProviderId,
}

/// A validated, non-default account name: `[a-z0-9][a-z0-9-]{0,31}`.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProviderAccountName(String);

impl ProviderAccountName {
    pub fn parse(raw: &str) -> Result<Self, ProviderAccountError> {
        let name = raw.trim();
        if name == DEFAULT_PROVIDER_ACCOUNT {
            return Err(ProviderAccountError::ReservedName);
        }
        let mut bytes = name.bytes();
        let valid_first = bytes
            .next()
            .is_some_and(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit());
        let valid_rest =
            bytes.all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-');
        if !valid_first || !valid_rest || name.len() > MAX_ACCOUNT_NAME_BYTES {
            return Err(ProviderAccountError::InvalidName);
        }
        Ok(Self(name.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ProviderAccountName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl fmt::Debug for ProviderAccountName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, formatter)
    }
}

/// Parses an account selection: `default` (or empty) is `None`.
pub fn parse_provider_account_selection(
    raw: &str,
) -> Result<Option<ProviderAccountName>, ProviderAccountError> {
    let raw = raw.trim();
    if raw.is_empty() || raw == DEFAULT_PROVIDER_ACCOUNT {
        return Ok(None);
    }
    ProviderAccountName::parse(raw).map(Some)
}

/// Validates a provider id for use in account labels.
pub fn validate_provider_account_provider_id(
    provider_id: &str,
) -> Result<(), ProviderAccountError> {
    let valid = !provider_id.is_empty()
        && provider_id.len() <= MAX_PROVIDER_ID_BYTES
        && provider_id.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'_' | b'.')
        })
        && provider_id.as_bytes()[0].is_ascii_alphanumeric();
    if valid {
        Ok(())
    } else {
        Err(ProviderAccountError::InvalidProviderId)
    }
}

/// One kind of material a named account holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ProviderAccountKind {
    /// A provider API key (env-key providers).
    ApiKey,
    /// A Claude subscription OAuth token (`claude setup-token`).
    ClaudeOauthToken,
    /// The `CLAUDE_CONFIG_DIR` of a Claude Code login (not secret).
    ClaudeConfigDir,
    /// A ChatGPT sign-in (`auth.json` contents), kept only in the vault.
    ChatgptAuth,
    /// An AWS profile name (not secret).
    AwsProfile,
    /// Marks an account of an `auth.command` provider; the command receives
    /// the account name in `CORBANU_PROVIDER_ACCOUNT` (not secret).
    Command,
}

impl ProviderAccountKind {
    pub const ALL: [Self; 6] = [
        Self::ApiKey,
        Self::ClaudeOauthToken,
        Self::ClaudeConfigDir,
        Self::ChatgptAuth,
        Self::AwsProfile,
        Self::Command,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::ApiKey => "api_key",
            Self::ClaudeOauthToken => "claude_oauth_token",
            Self::ClaudeConfigDir => "claude_config_dir",
            Self::ChatgptAuth => "chatgpt_auth",
            Self::AwsProfile => "aws_profile",
            Self::Command => "command",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| kind.as_str() == raw)
    }

    /// Subscription material that only its provider integration may read;
    /// generic `/vault` reveal and `vault auth-helper` refuse it.
    pub fn is_provider_managed(self) -> bool {
        matches!(self, Self::ClaudeOauthToken | Self::ChatgptAuth)
    }

    fn credential_type(self) -> CredentialType {
        match self {
            Self::ApiKey => CredentialType::ApiKey,
            Self::ClaudeOauthToken | Self::ChatgptAuth => CredentialType::BearerToken,
            Self::ClaudeConfigDir | Self::AwsProfile | Self::Command => {
                CredentialType::ManualSecret
            }
        }
    }
}

/// The vault label for one kind of a named account.
pub fn provider_account_label(
    provider_id: &str,
    name: &ProviderAccountName,
    kind: ProviderAccountKind,
) -> Result<String, ProviderAccountError> {
    validate_provider_account_provider_id(provider_id)?;
    Ok(format!(
        "{ACCOUNT_LABEL_PREFIX}{provider_id}/{ACCOUNTS_SEGMENT}/{name}/{}",
        kind.as_str()
    ))
}

/// Splits a named-account label into its parts, or `None` for any other label.
pub fn parse_provider_account_label(
    label: &str,
) -> Option<(String, ProviderAccountName, ProviderAccountKind)> {
    let rest = label.strip_prefix(ACCOUNT_LABEL_PREFIX)?;
    let mut parts = rest.split('/');
    let (provider_id, segment, name, kind) =
        (parts.next()?, parts.next()?, parts.next()?, parts.next()?);
    if parts.next().is_some() || segment != ACCOUNTS_SEGMENT {
        return None;
    }
    validate_provider_account_provider_id(provider_id).ok()?;
    Some((
        provider_id.to_string(),
        ProviderAccountName::parse(name).ok()?,
        ProviderAccountKind::parse(kind)?,
    ))
}

/// Whether generic vault reveal/edit must refuse this label.
pub fn is_provider_managed_account_label(label: &str) -> bool {
    parse_provider_account_label(label).is_some_and(|(_, _, kind)| kind.is_provider_managed())
}

/// Metadata for one named account: no secret material.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProviderAccountMeta {
    pub provider_id: String,
    pub name: ProviderAccountName,
    pub kinds: Vec<ProviderAccountKind>,
}

impl Vault {
    /// Lists named accounts (all providers) from vault metadata only.
    pub fn list_provider_accounts(&self) -> Result<Vec<ProviderAccountMeta>, VaultError> {
        let mut accounts: BTreeMap<(String, ProviderAccountName), Vec<ProviderAccountKind>> =
            BTreeMap::new();
        for entry in self.list()? {
            if let Some((provider_id, name, kind)) = parse_provider_account_label(&entry.label) {
                accounts.entry((provider_id, name)).or_default().push(kind);
            }
        }
        Ok(accounts
            .into_iter()
            .map(|((provider_id, name), mut kinds)| {
                kinds.sort();
                ProviderAccountMeta {
                    provider_id,
                    name,
                    kinds,
                }
            })
            .collect())
    }

    /// The kinds a named account holds; empty when the account does not exist.
    pub fn provider_account_kinds(
        &self,
        provider_id: &str,
        name: &ProviderAccountName,
    ) -> Result<Vec<ProviderAccountKind>, VaultError> {
        Ok(self
            .list_provider_accounts()?
            .into_iter()
            .find(|account| account.provider_id == provider_id && &account.name == name)
            .map(|account| account.kinds)
            .unwrap_or_default())
    }

    /// Reads one kind of a named account for its provider integration.
    /// `Ok(None)` when the account does not hold that kind.
    pub fn read_provider_account(
        &self,
        provider_id: &str,
        name: &ProviderAccountName,
        kind: ProviderAccountKind,
    ) -> Result<Option<Zeroizing<String>>, VaultError> {
        let label = account_label(provider_id, name, kind)?;
        self.with_storage_lock(|| {
            // An entry of another type under this label (for example one added
            // by hand through `/vault`) is not this account's material.
            let index = self.load_index()?;
            if index
                .credentials
                .get(&label)
                .is_none_or(|meta| meta.credential_type != kind.credential_type())
            {
                return Ok(None);
            }
            Ok(self
                .read_secret(&label)?
                .filter(|value| !value.trim().is_empty())
                .map(Zeroizing::new))
        })
    }

    /// Stores (or replaces) one kind of a named account.
    pub fn write_provider_account(
        &self,
        provider_id: &str,
        name: &ProviderAccountName,
        kind: ProviderAccountKind,
        value: &str,
    ) -> Result<(), VaultError> {
        let label = account_label(provider_id, name, kind)?;
        if value.trim().is_empty() {
            return Err(VaultError::EmptySecret);
        }
        self.with_storage_lock(|| {
            let mut index = self.load_index()?;
            let now = Utc::now().timestamp();
            let created_at = index
                .credentials
                .get(&label)
                .map_or(now, |meta| meta.created_at);
            index.credentials.insert(
                label.clone(),
                VaultCredentialMeta {
                    label: label.clone(),
                    credential_type: kind.credential_type(),
                    provider: Some(provider_id.to_string()),
                    notes: Some(format!("{provider_id} account {name}")),
                    revocation_notes: None,
                    created_at,
                    updated_at: now,
                    storage_backend: StorageBackend::EncryptedSecrets,
                },
            );
            let updates = vec![
                (
                    VAULT_SCOPE.clone(),
                    secret_name_for(&label)?,
                    value.to_string(),
                ),
                index_secret_entry(&index)?,
            ];
            self.secrets.apply_batch(&updates, &[])?;
            Ok(())
        })
    }

    /// Removes every kind of a named account in one transaction. Returns
    /// whether anything was removed.
    pub fn remove_provider_account(
        &self,
        provider_id: &str,
        name: &ProviderAccountName,
    ) -> Result<bool, VaultError> {
        let labels = ProviderAccountKind::ALL
            .into_iter()
            .map(|kind| account_label(provider_id, name, kind))
            .collect::<Result<Vec<_>, _>>()?;
        self.with_storage_lock(|| {
            let mut index = self.load_index()?;
            let mut removed = false;
            let mut deletes = Vec::with_capacity(labels.len());
            for label in &labels {
                removed |= index.credentials.remove(label).is_some();
                deletes.push((VAULT_SCOPE.clone(), secret_name_for(label)?));
            }
            if !removed {
                return Ok(false);
            }
            let updates = vec![index_secret_entry(&index)?];
            self.secrets.apply_batch(&updates, &deletes)?;
            Ok(true)
        })
    }
}

fn account_label(
    provider_id: &str,
    name: &ProviderAccountName,
    kind: ProviderAccountKind,
) -> Result<String, VaultError> {
    let label = provider_account_label(provider_id, name, kind)
        .map_err(|error| VaultError::InvalidLabel(error.to_string()))?;
    normalize_label(&label)
}

#[cfg(test)]
#[path = "provider_accounts_tests.rs"]
mod tests;
