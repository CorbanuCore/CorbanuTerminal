//! PF-84: stamps each provider with the named account `[provider_accounts]`
//! selects, so every lookup of that provider (session start, model switch,
//! spawned agents) resolves the same account.

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::path::Path;

use codex_model_provider::selected_account_error;
use codex_model_provider_info::ModelProviderInfo;
use codex_model_provider_info::NamedProviderAccount;
use codex_vault::Vault;
use codex_vault::VaultKeyStorage;
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
        let Some(provider) = model_providers.get_mut(provider_id) else {
            // Still reject a malformed name for an unknown provider.
            parse_provider_account_selection(selection)
                .map_err(|error| invalid_selection(provider_id, error.to_string()))?;
            startup_warnings.push(format!(
                "provider_accounts.{provider_id} names an unknown provider and is ignored."
            ));
            continue;
        };
        stamp_provider_account(provider_id, provider, selection)?;
    }
    Ok(())
}

/// Applies an explicit `--account [<provider>:]<name>` selection; it beats
/// `[provider_accounts]`. Without a prefix it selects an account of
/// `session_provider_id`; with another provider's prefix it selects that
/// provider's account for whatever in the session runs on it (spawned agents,
/// workers, a switch to it), and [`other_provider_account_error`] checks it at
/// thread start. Unlike the config table it is an error when the
/// `named_accounts` feature is off, because ignoring it would run the session
/// on the default account. `recorded` marks the account a resumed thread
/// recorded (or a worker's parent runs on) rather than a flag the user
/// passed, so errors name its source.
pub(crate) fn apply_explicit_provider_account(
    model_providers: &mut HashMap<String, ModelProviderInfo>,
    session_provider_id: &str,
    selection: &str,
    named_accounts_enabled: bool,
    recorded: bool,
) -> std::io::Result<()> {
    let (provider_id, name) = split_account_selection(selection, session_provider_id);
    let source = if recorded {
        format!("the thread's recorded account {provider_id}:{name}")
    } else {
        format!("--account {provider_id}:{name}")
    };
    let invalid = |message: String| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("{source}: {message}"),
        )
    };
    if !named_accounts_enabled {
        if parse_provider_account_selection(name).is_ok_and(|name| name.is_none()) {
            return Ok(());
        }
        let message = if recorded {
            "named accounts need the `named_accounts` feature (`--enable named_accounts`); \
             resume with `--account default` to use the default credentials instead"
        } else {
            "named accounts need the `named_accounts` feature (`--enable named_accounts`); \
             `--account default` uses the default credentials"
        };
        return Err(invalid(message.to_string()));
    }
    let provider = model_providers
        .get_mut(provider_id)
        .ok_or_else(|| invalid(format!("unknown provider `{provider_id}`")))?;
    stamp_provider_account(provider_id, provider, name).map_err(|error| {
        let message = error.to_string();
        let prefix = format!("provider_accounts.{provider_id}: ");
        invalid(
            message
                .strip_prefix(&prefix)
                .unwrap_or(&message)
                .to_string(),
        )
    })
}

/// Why the account an explicit `--account` names cannot be used, whether it
/// belongs to the session's provider or another one.
pub fn explicit_account_error(config: &super::Config) -> Option<String> {
    let (provider_id, _) = config
        .provider_account_override
        .as_deref()?
        .split_once(':')?;
    let provider = if provider_id == config.model_provider_id {
        &config.model_provider
    } else {
        config.model_providers.get(provider_id)?
    };
    selected_account_error(config.codex_home.as_path(), provider)
}

/// Why an explicit `--account <provider>:<name>` for a provider other than
/// the session's cannot be used, with the same recovery text as the session
/// provider's account (#425). The session's own account is checked
/// separately, from `config.model_provider`.
pub fn other_provider_account_error(config: &super::Config) -> Option<String> {
    let (provider_id, _) = config
        .provider_account_override
        .as_deref()?
        .split_once(':')?;
    if provider_id == config.model_provider_id {
        return None;
    }
    explicit_account_error(config)
}

/// Why an explicit account of a provider other than the session's would be
/// ignored by a session that cannot switch providers (exec): such an account
/// applies only to spawned agents on that provider, so when no spawned agent
/// may run there it is refused rather than silently unused (#425).
pub fn unused_other_provider_account_error(config: &super::Config) -> Option<String> {
    let spawns_possible = config.multi_agent_version_override()
        != Some(codex_protocol::protocol::MultiAgentVersion::Disabled);
    unused_other_provider_account(
        config.provider_account_override.as_deref()?,
        &config.model_provider_id,
        spawns_possible,
        config.agent_provider_allowlist.as_deref(),
    )
}

fn unused_other_provider_account(
    selection: &str,
    session_provider_id: &str,
    spawns_possible: bool,
    agent_provider_allowlist: Option<&[String]>,
) -> Option<String> {
    let (provider_id, name) = selection.split_once(':')?;
    // `default` asks for today's credentials, so it is never refused.
    if provider_id == session_provider_id
        || parse_provider_account_selection(name).is_ok_and(|name| name.is_none())
    {
        return None;
    }
    let reason = if !spawns_possible {
        "`agents.enabled` is false, so no agent can be spawned"
    } else if agent_provider_allowlist
        .is_some_and(|allowed| !allowed.iter().any(|allowed| allowed == provider_id))
    {
        "`agents.provider_allowlist` does not allow it for spawned agents"
    } else {
        return None;
    };
    Some(format!(
        "--account {selection}: this session runs on `{session_provider_id}` and nothing in it \
         can run on `{provider_id}` ({reason}), so the account would be ignored; start the \
         session on `{provider_id}` (`-c model_provider=\"{provider_id}\"`) or drop the \
         provider prefix"
    ))
}

/// Why the session cannot start on the accounts it selects: the session
/// provider's account (from `--account` or `[provider_accounts]`), then an
/// explicit account of another provider.
pub fn session_account_error(config: &super::Config) -> Option<String> {
    selected_account_error(config.codex_home.as_path(), &config.model_provider)
        .or_else(|| other_provider_account_error(config))
}

/// Splits `[<provider>:]<name>`; the provider defaults to the session's.
pub fn split_account_selection<'a>(
    selection: &'a str,
    session_provider_id: &'a str,
) -> (&'a str, &'a str) {
    match selection.split_once(':') {
        Some((provider_id, name)) => (provider_id.trim(), name.trim()),
        None => (session_provider_id, selection.trim()),
    }
}

fn invalid_selection(provider_id: &str, message: String) -> std::io::Error {
    std::io::Error::new(
        std::io::ErrorKind::InvalidInput,
        format!("provider_accounts.{provider_id}: {message}"),
    )
}

/// Validates `selection` for `provider` and records it (`default` clears it).
pub(crate) fn stamp_provider_account(
    provider_id: &str,
    provider: &mut ModelProviderInfo,
    selection: &str,
) -> std::io::Result<()> {
    let name = parse_provider_account_selection(selection)
        .map_err(|error| invalid_selection(provider_id, error.to_string()))?;
    provider.account = match name {
        Some(name) => {
            validate_provider_account_provider_id(provider_id)
                .map_err(|error| invalid_selection(provider_id, error.to_string()))?;
            // The Claude account helper serves the built-in Claude Plan only.
            if !supports_named_accounts(provider)
                || (provider.is_claude_plan()
                    && provider_id != codex_model_provider_info::CLAUDE_PLAN_PROVIDER_ID)
            {
                return Err(invalid_selection(
                    provider_id,
                    format!("provider `{provider_id}` does not support named accounts yet"),
                ));
            }
            Some(NamedProviderAccount {
                provider_id: provider_id.to_string(),
                name: name.as_str().to_string(),
            })
        }
        None => None,
    };
    Ok(())
}

/// The named accounts configured for `provider_id`, from vault metadata only.
pub(crate) fn configured_account_names(codex_home: &Path, provider_id: &str) -> Vec<String> {
    let vault = Vault::new(codex_home.to_path_buf());
    if vault.key_storage() == VaultKeyStorage::NotInitialized {
        return Vec::new();
    }
    vault
        .list_provider_accounts()
        .unwrap_or_default()
        .into_iter()
        .filter(|account| account.provider_id == provider_id)
        .map(|account| account.name.as_str().to_string())
        .collect()
}

/// API-key providers and `auth.command` providers (including Claude Plan).
/// OpenAI sign-in and AWS providers are not supported yet, so selecting an
/// account for them fails instead of silently using the default credential.
fn supports_named_accounts(provider: &ModelProviderInfo) -> bool {
    provider.aws.is_none()
        && !provider.requires_openai_auth
        && (provider.env_key.is_some() || provider.auth.is_some())
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn providers() -> HashMap<String, ModelProviderInfo> {
        HashMap::from([
            ("zai".to_string(), ModelProviderInfo::create_zai_provider()),
            ("kimi".to_string(), ModelProviderInfo::create_zai_provider()),
            (
                "openai".to_string(),
                ModelProviderInfo::create_openai_provider(/*base_url*/ None),
            ),
        ])
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

    #[test]
    fn providers_without_account_support_fail_closed() {
        let mut model_providers = providers();
        let error = apply_provider_accounts(
            &mut model_providers,
            Some(&accounts(&[("openai", "work")])),
            /*named_accounts_enabled*/ true,
            &mut Vec::new(),
        )
        .expect_err("openai sign-in accounts are not supported yet");
        assert_eq!(
            error.to_string(),
            "provider_accounts.openai: provider `openai` does not support named accounts yet"
        );
    }

    #[test]
    fn aws_providers_fail_closed() {
        let mut model_providers = HashMap::from([(
            "amazon-bedrock".to_string(),
            ModelProviderInfo::create_amazon_bedrock_provider(/*aws*/ None),
        )]);
        let error = apply_provider_accounts(
            &mut model_providers,
            Some(&accounts(&[("amazon-bedrock", "work")])),
            /*named_accounts_enabled*/ true,
            &mut Vec::new(),
        )
        .expect_err("aws accounts are not supported yet");
        assert!(
            error
                .to_string()
                .ends_with("does not support named accounts yet")
        );
    }

    #[test]
    fn explicit_account_selects_the_session_provider_or_a_prefixed_one() {
        let mut model_providers = providers();
        apply_explicit_provider_account(
            &mut model_providers,
            "zai",
            "work",
            /*named_accounts_enabled*/ true,
            /*recorded*/ false,
        )
        .expect("session provider");
        apply_explicit_provider_account(
            &mut model_providers,
            "zai",
            "kimi:alt",
            /*named_accounts_enabled*/ true,
            /*recorded*/ false,
        )
        .expect("prefixed provider");
        assert_eq!(
            (
                model_providers["zai"].account.clone(),
                model_providers["kimi"].account.clone()
            ),
            (
                Some(NamedProviderAccount {
                    provider_id: "zai".to_string(),
                    name: "work".to_string(),
                }),
                Some(NamedProviderAccount {
                    provider_id: "kimi".to_string(),
                    name: "alt".to_string(),
                }),
            )
        );
        apply_explicit_provider_account(
            &mut model_providers,
            "zai",
            "default",
            /*named_accounts_enabled*/ true,
            /*recorded*/ false,
        )
        .expect("default clears");
        assert_eq!(model_providers["zai"].account, None);
    }

    #[test]
    fn explicit_account_fails_closed_when_it_cannot_be_honoured() {
        let mut model_providers = providers();
        let error = |selection: &str, enabled: bool, providers: &mut HashMap<_, _>| {
            apply_explicit_provider_account(
                providers, "zai", selection, enabled, /*recorded*/ false,
            )
            .expect_err(selection)
            .to_string()
        };
        assert_eq!(
            [
                error("work", false, &mut model_providers),
                error("gone:work", true, &mut model_providers),
                error("openai:work", true, &mut model_providers),
            ],
            [
                "--account zai:work: named accounts need the `named_accounts` feature \
                 (`--enable named_accounts`); `--account default` uses the default credentials"
                    .to_string(),
                "--account gone:work: unknown provider `gone`".to_string(),
                "--account openai:work: provider `openai` does not support named accounts yet"
                    .to_string(),
            ]
        );
        // `default` is today's behaviour, so it is accepted with the feature off.
        apply_explicit_provider_account(
            &mut model_providers,
            "zai",
            "default",
            /*named_accounts_enabled*/ false,
            /*recorded*/ false,
        )
        .expect("default without the feature");
        assert_eq!(model_providers["zai"].account, None);
    }

    /// #425: exec cannot switch providers, so another provider's account
    /// that no spawned agent can use is refused, not silently ignored.
    #[test]
    fn other_provider_account_is_refused_when_nothing_can_use_it() {
        let allow_zai = ["zai".to_string()];
        assert_eq!(
            [
                unused_other_provider_account("zai:work", "zai", false, None),
                unused_other_provider_account("kimi-code:main", "zai", true, None),
                unused_other_provider_account(
                    "kimi-code:main",
                    "zai",
                    true,
                    Some(&["kimi-code".to_string()]),
                ),
                unused_other_provider_account("kimi-code:default", "zai", false, None),
            ],
            [None, None, None, None]
        );
        assert_eq!(
            [
                unused_other_provider_account("kimi-code:main", "zai", false, None),
                unused_other_provider_account("kimi-code:main", "zai", true, Some(&allow_zai)),
            ],
            [
                Some(
                    "--account kimi-code:main: this session runs on `zai` and nothing in it can \
                     run on `kimi-code` (`agents.enabled` is false, so no agent can be spawned), so \
                     the account would be ignored; start the session on `kimi-code` \
                     (`-c model_provider=\"kimi-code\"`) or drop the provider prefix"
                        .to_string()
                ),
                Some(
                    "--account kimi-code:main: this session runs on `zai` and nothing in it can \
                     run on `kimi-code` (`agents.provider_allowlist` does not allow it for \
                     spawned agents), so the account would be ignored; start the session on \
                     `kimi-code` (`-c model_provider=\"kimi-code\"`) or drop the provider prefix"
                        .to_string()
                ),
            ]
        );
    }

    /// #427: a flag-off resume names the thread's recorded account, not a
    /// `--account` flag the user did not pass.
    #[test]
    fn recorded_account_errors_name_the_recorded_account() {
        let error = apply_explicit_provider_account(
            &mut providers(),
            "zai",
            "zai:main",
            /*named_accounts_enabled*/ false,
            /*recorded*/ true,
        )
        .expect_err("named account with the feature off");
        assert_eq!(
            error.to_string(),
            "the thread's recorded account zai:main: named accounts need the `named_accounts` \
             feature (`--enable named_accounts`); resume with `--account default` to use the \
             default credentials instead"
        );
    }
}
