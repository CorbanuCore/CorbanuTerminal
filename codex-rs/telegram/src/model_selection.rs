use std::collections::HashMap;

use codex_core::config::Config;
use codex_model_provider_info::CLAUDE_FABLE_5_PLAN_MODEL;
use codex_model_provider_info::CLAUDE_PLAN_MODEL;
use codex_model_provider_info::ModelProviderInfo;
use codex_model_provider_info::corrected_catalog_provider;
use sha2::Digest;
use sha2::Sha256;
use tracing::warn;

const GPT_5_5_MODEL: &str = "gpt-5.5";
const MODEL_CALLBACK_PREFIX: &str = "tgm";
const MODEL_FINGERPRINT_HEX_LEN: usize = 24;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ModelPickerCallback {
    Select { fingerprint: String },
    Page { page: usize },
}

impl ModelPickerCallback {
    pub(crate) fn select(model: &str) -> Self {
        Self::Select {
            fingerprint: model_fingerprint(model),
        }
    }

    pub(crate) fn encode(&self) -> String {
        match self {
            Self::Select { fingerprint } => {
                format!("{MODEL_CALLBACK_PREFIX}:s:{fingerprint}")
            }
            Self::Page { page } => format!("{MODEL_CALLBACK_PREFIX}:p:{page}"),
        }
    }

    pub(crate) fn decode(raw: &str) -> Option<Self> {
        let mut parts = raw.split(':');
        if parts.next()? != MODEL_CALLBACK_PREFIX {
            return None;
        }
        let kind = parts.next()?;
        let value = parts.next()?;
        if parts.next().is_some() {
            return None;
        }
        match kind {
            "s" if value.len() == MODEL_FINGERPRINT_HEX_LEN
                && value.bytes().all(|byte| byte.is_ascii_hexdigit()) =>
            {
                Some(Self::Select {
                    fingerprint: value.to_ascii_lowercase(),
                })
            }
            "p" => Some(Self::Page {
                page: value.parse().ok()?,
            }),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ModelAlias {
    pub alias: &'static str,
    pub model: &'static str,
    pub display_name: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CatalogModel {
    pub id: String,
    pub model: String,
    pub display_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ModelResolutionSource {
    Alias,
    Catalog,
    PassThrough,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ModelResolution {
    pub model: String,
    pub source: ModelResolutionSource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AvailableModel {
    pub model: String,
    pub display_name: String,
    pub aliases: Vec<&'static str>,
    pub in_catalog: bool,
}

pub(crate) fn aliases() -> &'static [ModelAlias] {
    &[
        ModelAlias {
            alias: "fable",
            model: CLAUDE_FABLE_5_PLAN_MODEL,
            display_name: "Claude Fable 5 Plan",
        },
        ModelAlias {
            alias: "opus",
            model: CLAUDE_PLAN_MODEL,
            display_name: "Claude Opus 4.8 Plan",
        },
        ModelAlias {
            alias: "gpt",
            model: GPT_5_5_MODEL,
            display_name: "GPT-5.5",
        },
        ModelAlias {
            alias: "gpt-5.5",
            model: GPT_5_5_MODEL,
            display_name: "GPT-5.5",
        },
    ]
}

pub(crate) fn resolve_model(input: &str, catalog: &[CatalogModel]) -> ModelResolution {
    let trimmed = input.trim();
    if let Some(alias) = aliases()
        .iter()
        .find(|alias| alias.alias.eq_ignore_ascii_case(trimmed))
    {
        return ModelResolution {
            model: alias.model.to_string(),
            source: ModelResolutionSource::Alias,
        };
    }

    if let Some(model) = catalog.iter().find(|model| {
        model.id.eq_ignore_ascii_case(trimmed) || model.display_name.eq_ignore_ascii_case(trimmed)
    }) {
        return ModelResolution {
            model: model.model.clone(),
            source: ModelResolutionSource::Catalog,
        };
    }

    ModelResolution {
        model: trimmed.to_string(),
        source: ModelResolutionSource::PassThrough,
    }
}

/// The provider a `/model` switch will actually run on, and whether that differs
/// from the provider the chat was already using.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProviderChoice {
    pub provider: String,
    pub changed: bool,
}

/// Resolve the provider for `model`.
///
/// `corrected_catalog_provider` repairs *impossible* (model, provider) pairs; it
/// returns `None` for every model outside the families it knows. `model/list` does
/// not carry a provider, so when it returns `None` there is nothing to route by and
/// the chat keeps the provider it already had. That is a real limitation, not a
/// routing decision — callers must report `changed: false` rather than print a
/// provider next to the new model and imply a switch happened.
pub(crate) fn provider_for_model(model: &str, current_provider: &str) -> ProviderChoice {
    match corrected_catalog_provider(model, current_provider) {
        Some(provider) => ProviderChoice {
            provider: provider.to_string(),
            changed: provider != current_provider,
        },
        None => ProviderChoice {
            provider: current_provider.to_string(),
            changed: false,
        },
    }
}

pub(crate) fn provider_is_missing_openai_auth(
    provider_id: &str,
    providers: &HashMap<String, ModelProviderInfo>,
    openai_auth_present: bool,
) -> bool {
    !openai_auth_present
        && providers
            .get(provider_id)
            .is_some_and(|provider| provider.requires_openai_auth)
}

/// A credential a provider requires from the environment but does not have.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MissingProviderCredential {
    pub provider: String,
    pub env_key: String,
    pub instructions: Option<String>,
}

/// `Some` only when the provider is *certain* to fail auth: it declares an `env_key`,
/// has no other way to authenticate, and no key is reachable from either the process
/// environment or the stored provider-key vault.
///
/// Providers that authenticate by subscription login (`requires_openai_auth`), a token
/// command (`auth`), a static bearer, or AWS SigV4 are never reported — they hold no
/// env var and a missing one proves nothing about them.
pub(crate) fn missing_provider_credential(
    provider_id: &str,
    providers: &HashMap<String, ModelProviderInfo>,
    config: &Config,
) -> Option<MissingProviderCredential> {
    missing_provider_credential_with(provider_id, providers, |env_key| {
        provider_credential_present(config, env_key)
    })
}

/// Testable core of [`missing_provider_credential`]; `credential_present` stands in for
/// the environment and the vault so tests never touch global state or the filesystem.
pub(crate) fn missing_provider_credential_with(
    provider_id: &str,
    providers: &HashMap<String, ModelProviderInfo>,
    credential_present: impl Fn(&str) -> bool,
) -> Option<MissingProviderCredential> {
    let info = providers.get(provider_id)?;
    if info.requires_openai_auth
        || info.auth.is_some()
        || info.experimental_bearer_token.is_some()
        || info.aws.is_some()
    {
        return None;
    }

    let env_key = info.env_key.as_ref()?;
    if credential_present(env_key) {
        return None;
    }

    Some(MissingProviderCredential {
        provider: provider_id.to_string(),
        env_key: env_key.clone(),
        instructions: info.env_key_instructions.clone(),
    })
}

/// A provider key may live in the process environment *or* in the stored provider-key
/// vault (`/providers` writes it there), so an unset env var alone does not mean the
/// provider cannot authenticate. Mirrors what the model provider itself checks.
///
/// Fails open: if the store cannot be read we assume a key exists rather than block a
/// switch on an unrelated storage fault.
fn provider_credential_present(config: &Config, env_key: &str) -> bool {
    let from_env = std::env::var(env_key)
        .ok()
        .is_some_and(|value| !value.trim().is_empty());
    if from_env {
        return true;
    }

    match codex_login::auth::provider_api_key_from_auth_storage(
        &config.codex_home,
        env_key,
        config.cli_auth_credentials_store_mode,
        config.auth_keyring_backend_kind(),
    ) {
        Ok(Some(key)) => !key.trim().is_empty(),
        Ok(None) => false,
        Err(err) => {
            warn!("could not read stored provider key for {env_key}: {err}");
            true
        }
    }
}

pub(crate) fn available_models(catalog: &[CatalogModel]) -> Vec<AvailableModel> {
    let mut available = Vec::new();
    for model in catalog {
        let aliases = aliases_for_model(&model.model);
        available.push(AvailableModel {
            model: model.model.clone(),
            display_name: model.display_name.clone(),
            aliases,
            in_catalog: true,
        });
    }

    for alias in aliases() {
        if available.iter().any(|model| model.model == alias.model) {
            continue;
        }
        available.push(AvailableModel {
            model: alias.model.to_string(),
            display_name: alias.display_name.to_string(),
            aliases: aliases_for_model(alias.model),
            in_catalog: false,
        });
    }

    available
}

pub(crate) fn model_for_fingerprint(
    fingerprint: &str,
    models: &[AvailableModel],
) -> Option<String> {
    let mut matches = models
        .iter()
        .filter(|model| model_fingerprint(&model.model) == fingerprint)
        .map(|model| model.model.as_str());
    let first = matches.next()?;
    if matches.any(|candidate| candidate != first) {
        return None;
    }
    Some(first.to_string())
}

fn model_fingerprint(model: &str) -> String {
    let digest = format!("{:x}", Sha256::digest(model.as_bytes()));
    digest[..MODEL_FINGERPRINT_HEX_LEN].to_string()
}

pub(crate) fn known_model_source(model: &str, catalog: &[CatalogModel]) -> Option<&'static str> {
    if catalog.iter().any(|entry| entry.model == model) {
        return Some("catalog");
    }
    if aliases().iter().any(|alias| alias.model == model) {
        return Some("alias");
    }
    None
}

fn aliases_for_model(model: &str) -> Vec<&'static str> {
    aliases()
        .iter()
        .filter(|alias| alias.model == model)
        .map(|alias| alias.alias)
        .collect()
}

#[cfg(test)]
#[path = "model_selection_tests.rs"]
mod tests;
