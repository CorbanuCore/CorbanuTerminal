use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;
use std::collections::BTreeMap;

/// Declarative description of an environment-backed credential family.
#[derive(Clone, Debug, Default, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct CredentialProviderConfig {
    pub env: Vec<String>,
    pub patterns: Vec<String>,
    /// URL prefixes authorized for injection. Bare loopback hosts imply HTTP; other bare hosts
    /// imply HTTPS.
    pub url_prefixes: Vec<String>,
    /// Environment variable containing an additional URL prefix or hostname.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url_prefix_from_env: Option<String>,
    pub auth: Vec<CredentialAuthMethod>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,
}

impl CredentialProviderConfig {
    /// Validate a complete provider definition using the broker's compilation rules.
    pub fn validate(&self, id: &str) -> anyhow::Result<()> {
        super::configured::ConfiguredCredentialProvider::compile(id, self)?;
        Ok(())
    }

    /// Environment sources Corbanu's credential engine does not broker (D612, #2629).
    ///
    /// Every proxy Corbanu builds runs the hardened engine, which brokers only
    /// the built-in providers' credential variables. A provider's other sources
    /// would only be withheld from commands at run time
    /// (`virtualize_withholding_configured_sources`).
    pub fn unbrokered_sources(&self) -> impl Iterator<Item = &str> {
        self.env.iter().map(String::as_str).filter(|key| {
            !super::providers::credential_env_keys()
                .any(|builtin| super::env_key_matches(key, builtin))
        })
    }
}

/// Refuses configured credential providers that name a source Corbanu's
/// credential engine does not broker (D612, #2629), so loading the config fails
/// with an actionable error instead of the value being withheld at run time.
/// Providers whose sources are all built-in credential variables load as before.
pub fn reject_unbrokered_credential_providers(
    providers: &BTreeMap<String, CredentialProviderConfig>,
) -> Result<(), String> {
    let unbrokered = providers
        .iter()
        .filter_map(|(id, provider)| {
            let sources = provider.unbrokered_sources().collect::<Vec<_>>();
            (!sources.is_empty()).then(|| format!("`{id}` ({})", sources.join(", ")))
        })
        .collect::<Vec<_>>();
    if unbrokered.is_empty() {
        return Ok(());
    }
    let built_in = super::providers::credential_env_keys().collect::<Vec<_>>();
    Err(format!(
        "unsupported network proxy credential provider {}: Corbanu's credential engine does not \
         broker these environment variables. It brokers only its built-in credential variables \
         ({}), so these values would be withheld from every command. Remove them from \
         [features.network_proxy.credentials] in config.toml or the selected profile's config \
         file; to keep a variable away from commands, add it to \
         shell_environment_policy.exclude instead.",
        unbrokered.join(", "),
        built_in.join(", ")
    ))
}

/// Authentication formats supported by declarative credential providers.
#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CredentialAuthMethod {
    Bearer,
    Token,
    Basic,
    Header,
}

#[cfg(test)]
#[path = "provider_config_tests.rs"]
mod tests;
