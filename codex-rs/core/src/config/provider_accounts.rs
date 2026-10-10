//! PF-84: stamps each provider with the named account `[provider_accounts]`
//! selects, so every lookup of that provider (session start, model switch,
//! spawned agents) resolves the same account.

use std::collections::BTreeMap;
use std::collections::HashMap;

use codex_model_provider_info::ModelProviderInfo;
use codex_model_provider_info::NamedProviderAccount;
use codex_vault::parse_provider_account_selection;
use codex_vault::validate_provider_account_provider_id;

/// Applies `[provider_accounts]`. Ignored (with a warning) when the
/// `named_accounts` feature is off. An invalid name or a provider that cannot
/// hold accounts is an error: the session must not silently run on another
/// account.
pub(crate) fn apply_provider_accounts(
    model_providers: &mut HashMap<String, ModelProviderInfo>,
    provider_accounts: Option<&BTreeMap<String, String>>,
    named_accounts_enabled: bool,
    startup_warnings: &mut Vec<String>,
) -> std::io::Result<()> {
    let Some(provider_accounts) = provider_accounts.filter(|accounts| !accounts.is_empty()) else {
        return Ok(());
    };
    if !named_accounts_enabled {
        startup_warnings.push(
            "`[provider_accounts]` is ignored because the `named_accounts` feature is off; \
             every provider uses its default account."
                .to_string(),
        );
        return Ok(());
    }
    for (provider_id, selection) in provider_accounts {
        let invalid = |message: String| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("provider_accounts.{provider_id}: {message}"),
            )
        };
        let name = parse_provider_account_selection(selection)
            .map_err(|error| invalid(error.to_string()))?;
        let Some(provider) = model_providers.get_mut(provider_id) else {
            startup_warnings.push(format!(
                "provider_accounts.{provider_id} names an unknown provider and is ignored."
            ));
            continue;
        };
        provider.account = match name {
            Some(name) => {
                validate_provider_account_provider_id(provider_id)
                    .map_err(|error| invalid(error.to_string()))?;
                Some(NamedProviderAccount {
                    provider_id: provider_id.clone(),
                    name: name.as_str().to_string(),
                })
            }
            None => None,
        };
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn providers() -> HashMap<String, ModelProviderInfo> {
        ["zai", "kimi"]
            .into_iter()
            .map(|id| (id.to_string(), ModelProviderInfo::default()))
            .collect()
    }

    fn accounts(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
        pairs
            .iter()
            .map(|(provider, name)| (provider.to_string(), name.to_string()))
            .collect()
    }

    #[test]
    fn selected_accounts_are_stamped_on_their_providers_only() {
        let mut model_providers = providers();
        let mut warnings = Vec::new();
        apply_provider_accounts(
            &mut model_providers,
            Some(&accounts(&[
                ("zai", "work"),
                ("kimi", "default"),
                ("gone", "x"),
            ])),
            /*named_accounts_enabled*/ true,
            &mut warnings,
        )
        .expect("apply");
        assert_eq!(
            model_providers["zai"].account,
            Some(NamedProviderAccount {
                provider_id: "zai".to_string(),
                name: "work".to_string(),
            })
        );
        assert_eq!(model_providers["kimi"].account, None);
        assert_eq!(
            warnings,
            vec!["provider_accounts.gone names an unknown provider and is ignored.".to_string()]
        );
    }

    #[test]
    fn feature_off_keeps_every_default_account() {
        let mut model_providers = providers();
        let mut warnings = Vec::new();
        apply_provider_accounts(
            &mut model_providers,
            Some(&accounts(&[("zai", "work")])),
            /*named_accounts_enabled*/ false,
            &mut warnings,
        )
        .expect("apply");
        assert_eq!(model_providers["zai"].account, None);
        assert_eq!(warnings.len(), 1);
    }

    #[test]
    fn invalid_names_fail_closed() {
        let mut model_providers = providers();
        let error = apply_provider_accounts(
            &mut model_providers,
            Some(&accounts(&[("zai", "Work Account")])),
            /*named_accounts_enabled*/ true,
            &mut Vec::new(),
        )
        .expect_err("invalid name");
        assert!(
            error
                .to_string()
                .starts_with("provider_accounts.zai: invalid account name")
        );
    }
}
