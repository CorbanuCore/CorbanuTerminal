//! PF-84-S04: named accounts in `/providers` and onboarding.
//!
//! Only names, kinds and salted 12-hex fingerprints are ever shown. A value goes
//! from masked entry straight to the vault; events carry it in a redacting
//! wrapper and nothing here logs or renders it.

use std::collections::BTreeMap;
use std::fmt;
use std::path::Path;
use std::path::PathBuf;

use codex_model_provider_info::CLAUDE_PLAN_PROVIDER_ID;
use codex_model_provider_info::ModelProviderInfo;
use codex_vault::DEFAULT_PROVIDER_ACCOUNT;
use codex_vault::ProviderAccountKind;
use codex_vault::ProviderAccountName;
use codex_vault::Vault;
use codex_vault::VaultKeyStorage;
use zeroize::Zeroizing;

use crate::legacy_core::config::Config;

/// One named account of one provider (runtime provider id).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct NamedAccountRow {
    pub(crate) provider_id: String,
    pub(crate) name: ProviderAccountName,
    pub(crate) kinds: Vec<ProviderAccountKind>,
    pub(crate) fingerprint: Option<String>,
}

/// What `/providers` knows about accounts: the named rows, the account this
/// session uses and the account new sessions use, per provider id. A provider
/// missing from a map uses its `default` account.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct AccountsView {
    pub(crate) rows: Vec<NamedAccountRow>,
    pub(crate) session: BTreeMap<String, String>,
    pub(crate) defaults: BTreeMap<String, String>,
}

impl AccountsView {
    pub(crate) fn new(
        rows: Vec<NamedAccountRow>,
        session_config: &Config,
        current_config: &Config,
    ) -> Self {
        let session = session_config
            .model_providers
            .iter()
            .filter_map(|(id, provider)| {
                provider
                    .account
                    .as_ref()
                    .map(|account| (id.clone(), account.name.clone()))
            })
            .collect();
        let defaults = persisted_defaults(current_config);
        Self {
            rows,
            session,
            defaults,
        }
    }

    pub(crate) fn session_account(&self, provider_id: &str) -> &str {
        self.session
            .get(provider_id)
            .map_or(DEFAULT_PROVIDER_ACCOUNT, String::as_str)
    }

    pub(crate) fn default_account(&self, provider_id: &str) -> &str {
        self.defaults
            .get(provider_id)
            .map_or(DEFAULT_PROVIDER_ACCOUNT, String::as_str)
    }

    /// Named accounts of any of `provider_ids`, in a stable order.
    pub(crate) fn rows_for<'a>(
        &'a self,
        provider_ids: &'a [String],
    ) -> impl Iterator<Item = &'a NamedAccountRow> + 'a {
        self.rows
            .iter()
            .filter(move |row| provider_ids.contains(&row.provider_id))
    }

    pub(crate) fn find(&self, provider_id: &str, name: &str) -> Option<&NamedAccountRow> {
        self.rows
            .iter()
            .find(|row| row.provider_id == provider_id && row.name.as_str() == name)
    }

    /// ` · this session · default` style markers for one account.
    pub(crate) fn markers(&self, provider_id: &str, name: &str) -> String {
        let mut markers = String::new();
        if self.session_account(provider_id) == name {
            markers.push_str(" · this session");
        }
        if self.default_account(provider_id) == name {
            markers.push_str(" · default for new sessions");
        }
        markers
    }

    /// Whether removing or renaming `name` would pull it from under this
    /// session or from new sessions.
    pub(crate) fn in_use(&self, provider_id: &str, name: &str) -> bool {
        self.session_account(provider_id) == name || self.default_account(provider_id) == name
    }
}

/// `[provider_accounts]` as new sessions see it (config files and `-c`).
pub(crate) fn persisted_defaults(config: &Config) -> BTreeMap<String, String> {
    let effective = config.config_layer_stack.effective_config();
    let Some(table) = effective
        .get("provider_accounts")
        .and_then(|value| value.as_table())
    else {
        return BTreeMap::new();
    };
    table
        .iter()
        .filter_map(|(provider_id, value)| {
            let name = value.as_str()?.trim();
            (!name.is_empty() && name != DEFAULT_PROVIDER_ACCOUNT)
                .then(|| (provider_id.clone(), name.to_string()))
        })
        .collect()
}

/// Lists named accounts with their fingerprints. Blocking; vault metadata
/// and a salted hash only.
pub(crate) fn load_rows(codex_home: &Path) -> Result<Vec<NamedAccountRow>, String> {
    let vault = Vault::new(codex_home.to_path_buf());
    if vault.key_storage() == VaultKeyStorage::NotInitialized {
        return Ok(Vec::new());
    }
    let accounts = vault
        .list_provider_accounts()
        .map_err(|error| format!("Could not read named accounts: {error}"))?;
    Ok(accounts
        .into_iter()
        .map(|account| {
            let fingerprint = vault
                .provider_account_fingerprint(&account.provider_id, &account.name)
                .ok()
                .flatten();
            NamedAccountRow {
                provider_id: account.provider_id,
                name: account.name,
                kinds: account.kinds,
                fingerprint,
            }
        })
        .collect())
}

/// How a new named account of a provider is entered.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AddAccountMethod {
    ApiKey,
    ClaudeToken,
    ClaudeConfigDir,
    Command,
}

impl AddAccountMethod {
    pub(crate) fn kind(self) -> ProviderAccountKind {
        match self {
            Self::ApiKey => ProviderAccountKind::ApiKey,
            Self::ClaudeToken => ProviderAccountKind::ClaudeOauthToken,
            Self::ClaudeConfigDir => ProviderAccountKind::ClaudeConfigDir,
            Self::Command => ProviderAccountKind::Command,
        }
    }

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::ApiKey => "API key",
            Self::ClaudeToken => "Claude subscription token (claude setup-token)",
            Self::ClaudeConfigDir => "Claude Code login (its CLAUDE_CONFIG_DIR)",
            Self::Command => "External command account",
        }
    }

    /// Masked entry; otherwise the value is a path or nothing at all.
    pub(crate) fn is_secret(self) -> bool {
        matches!(self, Self::ApiKey | Self::ClaudeToken)
    }
}

/// The ways a provider can take a named account; empty when it cannot hold
/// one (OpenAI sign-in and AWS are a pending product decision).
pub(crate) fn add_methods(
    provider_id: &str,
    provider: &ModelProviderInfo,
) -> Vec<AddAccountMethod> {
    if !crate::legacy_core::config::supports_named_accounts(provider)
        || codex_vault::validate_provider_account_provider_id(provider_id).is_err()
    {
        return Vec::new();
    }
    if provider_id == CLAUDE_PLAN_PROVIDER_ID {
        vec![
            AddAccountMethod::ClaudeToken,
            AddAccountMethod::ClaudeConfigDir,
        ]
    } else if provider.is_claude_plan() {
        Vec::new()
    } else if provider.env_key.is_some() {
        vec![AddAccountMethod::ApiKey]
    } else if provider.auth.is_some() {
        vec![AddAccountMethod::Command]
    } else {
        Vec::new()
    }
}

/// The first provider id of a catalog entry that can hold named accounts.
pub(crate) fn account_provider_id<'a>(
    config: &Config,
    runtime_provider_ids: impl IntoIterator<Item = &'a str>,
) -> Option<String> {
    runtime_provider_ids.into_iter().find_map(|id| {
        config
            .model_providers
            .get(id)
            .filter(|provider| !add_methods(id, provider).is_empty())
            .map(|_| id.to_string())
    })
}

pub(crate) fn kinds_label(kinds: &[ProviderAccountKind]) -> String {
    kinds
        .iter()
        .map(|kind| match kind {
            ProviderAccountKind::ApiKey => "API key",
            ProviderAccountKind::ClaudeOauthToken => "subscription token",
            ProviderAccountKind::ClaudeConfigDir => "Claude Code login",
            ProviderAccountKind::ChatgptAuth => "ChatGPT sign-in",
            ProviderAccountKind::AwsProfile => "AWS profile",
            ProviderAccountKind::Command => "external command",
        })
        .collect::<Vec<_>>()
        .join(" + ")
}

/// `token · fp 1a2b3c4d5e6f` for one row.
pub(crate) fn row_detail(row: &NamedAccountRow) -> String {
    match row.fingerprint.as_deref() {
        Some(fingerprint) => format!("{} · fp {fingerprint}", kinds_label(&row.kinds)),
        None => kinds_label(&row.kinds),
    }
}

/// A value on its way from masked entry to the vault.
pub(crate) struct AccountValue(Zeroizing<String>);

impl AccountValue {
    pub(crate) fn new(value: String) -> Self {
        Self(Zeroizing::new(value))
    }
}

impl fmt::Debug for AccountValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<redacted account value>")
    }
}

/// Saves a new account; an existing name is refused. Blocking.
pub(crate) fn save_account(
    codex_home: PathBuf,
    provider_id: &str,
    name: &ProviderAccountName,
    method: AddAccountMethod,
    value: AccountValue,
) -> Result<String, String> {
    let mut cleaned = Zeroizing::new(value.0.trim().to_string());
    if method == AddAccountMethod::ClaudeToken {
        cleaned.retain(|character| !matches!(character, '\r' | '\n'));
    }
    let cleaned = match method {
        AddAccountMethod::Command => Zeroizing::new("1".to_string()),
        _ if cleaned.is_empty() => return Err("The value cannot be empty.".to_string()),
        _ => cleaned,
    };
    // Adding never replaces an existing account's credential.
    Vault::new(codex_home)
        .create_provider_account(provider_id, name, method.kind(), &cleaned)
        .map_err(|error| format!("Could not save {provider_id} account `{name}`: {error}"))?;
    Ok(format!("Saved {provider_id} account `{name}`."))
}

/// Renames one account. Blocking.
pub(crate) fn rename_account(
    codex_home: PathBuf,
    provider_id: &str,
    from: &ProviderAccountName,
    to: &ProviderAccountName,
) -> Result<String, String> {
    Vault::new(codex_home)
        .rename_provider_account(provider_id, from, to)
        .map_err(|error| format!("Could not rename {provider_id} account `{from}`: {error}"))?;
    Ok(format!("Renamed {provider_id} account `{from}` to `{to}`."))
}

/// Removes one account and only its labels. Blocking.
pub(crate) fn remove_account(
    codex_home: PathBuf,
    provider_id: &str,
    name: &ProviderAccountName,
) -> Result<String, String> {
    match Vault::new(codex_home).remove_provider_account(provider_id, name) {
        Ok(true) => Ok(format!("Removed {provider_id} account `{name}`.")),
        Ok(false) => Err(format!("{provider_id} has no account `{name}`.")),
        Err(error) => Err(format!(
            "Could not remove {provider_id} account `{name}`: {error}"
        )),
    }
}

/// `/providers` account actions; routed by the app.
#[derive(Debug)]
pub(crate) enum ProviderAccountEvent {
    OpenActions {
        provider_id: String,
        name: ProviderAccountName,
    },
    AddStart {
        provider_id: String,
    },
    OpenMethodChoice {
        provider_id: String,
        name: ProviderAccountName,
        methods: Vec<AddAccountMethod>,
    },
    AddNamed {
        provider_id: String,
        name: ProviderAccountName,
        method: AddAccountMethod,
    },
    Save {
        provider_id: String,
        name: ProviderAccountName,
        method: AddAccountMethod,
        value: AccountValue,
    },
    UseForSession {
        provider_id: String,
        name: Option<ProviderAccountName>,
    },
    MakeDefault {
        provider_id: String,
        name: Option<ProviderAccountName>,
    },
    RenameStart {
        provider_id: String,
        name: ProviderAccountName,
    },
    Rename {
        provider_id: String,
        from: ProviderAccountName,
        to: ProviderAccountName,
    },
    RemoveStart {
        provider_id: String,
        name: ProviderAccountName,
    },
    /// `replacement` is the account (None = default) that takes over this
    /// session and new sessions first, when the removed one is in use.
    Remove {
        provider_id: String,
        name: ProviderAccountName,
        replacement: Option<Option<ProviderAccountName>>,
    },
    Finished {
        result: Result<String, String>,
    },
}

#[cfg(test)]
#[path = "provider_named_accounts_tests.rs"]
mod tests;
