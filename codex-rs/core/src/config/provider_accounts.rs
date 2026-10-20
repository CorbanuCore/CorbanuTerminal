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
            format!(
                "named accounts need the `named_accounts` feature (`--enable named_accounts`); \
                 `--account {provider_id}:default` (on a resume or fork) uses the default \
                 credentials instead"
            )
        } else {
            "named accounts need the `named_accounts` feature (`--enable named_accounts`); \
             `--account default` uses the default credentials"
                .to_string()
        };
        return Err(invalid(message));
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
fn explicit_account_error(config: &super::Config) -> Option<String> {
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

/// PF-84 (#555): applies the account a resumed thread recorded,
/// `<provider>:<name>`, when that provider is not the session's. The loader
/// applies an inherited account only to the session's own provider; a client
/// that resumes a thread on its recorded provider (the TUI, when no model or
/// provider override applies) calls this so the provider it switches to
/// carries the account the thread runs on. An explicit `--account` of that
/// provider still wins.
pub fn apply_recorded_provider_account(
    config: &mut super::Config,
    selection: &str,
) -> std::io::Result<()> {
    let (provider_id, _) = split_account_selection(selection, &config.model_provider_id);
    let explicit_for_provider = config
        .provider_account_override
        .as_deref()
        .and_then(|explicit| explicit.split_once(':'))
        .is_some_and(|(explicit_provider, _)| explicit_provider == provider_id);
    if provider_id == config.model_provider_id || explicit_for_provider {
        return Ok(());
    }
    let named_accounts_enabled = config
        .features
        .enabled(codex_features::Feature::NamedAccounts);
    let session_provider_id = config.model_provider_id.clone();
    apply_explicit_provider_account(
        &mut config.model_providers,
        &session_provider_id,
        selection,
        named_accounts_enabled,
        /*recorded*/ true,
    )
}

/// Why the session cannot start on the accounts it selects: the session
/// provider's account (from `--account` or `[provider_accounts]`), then an
/// explicit account of another provider.
pub fn session_account_error(config: &super::Config) -> Option<String> {
    selected_account_error(config.codex_home.as_path(), &config.model_provider)
        .or_else(|| sign_in_restriction_error(config))
        .or_else(|| other_provider_account_error(config))
}

/// Why a spawned agent cannot start on its provider's account: not
/// configured, or a sign-in the login restrictions rule out.
pub fn subagent_account_error(config: &super::Config) -> Option<String> {
    selected_account_error(config.codex_home.as_path(), &config.model_provider)
        .or_else(|| sign_in_restriction_error(config))
}

/// PF-84-S06: a named OpenAI sign-in of the kind or workspace the
/// configuration's login restrictions rule out is refused before the session
/// starts (its manager also refuses to load it). Only the configured
/// `forced_login_method` counts, not the API-key-only onboarding default a
/// third-party session provider implies (#571).
fn sign_in_restriction_error(config: &super::Config) -> Option<String> {
    let account = codex_model_provider::named_sign_in_account(&config.model_provider)?;
    codex_login::named_account_restriction_error(
        config.codex_home.as_path(),
        &account,
        config.configured_forced_login_method,
        config.forced_chatgpt_workspace_id.as_deref(),
    )
}

/// PF-84-S06 (#570): a thread on the default OpenAI login is refused while
/// the process's default-login manager hides that login because the login
/// restrictions it was built with rule it out (it is left unchanged). Only
/// in a process that left the default login in place at startup (see
/// `codex_login::leave_default_login_unenforced`): a new session on the
/// default account (`/providers` → *Use default account for this session*),
/// a spawned agent on it, or a resumed or forked default-login thread. A
/// mid-session provider switch to it is refused at the first request with
/// the same reason (the provider's `api_auth`).
pub fn default_login_restriction_error(
    config: &super::Config,
    auth_manager: &std::sync::Arc<codex_login::AuthManager>,
) -> Option<String> {
    let provider = &config.model_provider;
    if !config
        .features
        .enabled(codex_features::Feature::NamedAccounts)
        || !provider.requires_openai_auth
        || provider.auth.is_some()
        || provider.aws.is_some()
        || provider.env_key.is_some()
        || provider.account.is_some()
    {
        return None;
    }
    auth_manager
        .default_account_manager()
        .hidden_default_login_reason()
}

/// #689: why a spawned agent on `child` (runtime provider `child_id`) cannot
/// sign in, checked before it starts, as the TUI checks its own spawns (#665).
/// Named accounts are per connection: `zai` and `zai-anthropic` share
/// `ZAI_API_KEY`, but only the connection a named account is selected for runs
/// on it. When `child` runs on the default key of such a key variable while
/// another connection (or the parent, `parent`) runs on a named account, and
/// the default key is in neither the environment (nor held by the credential
/// broker) nor storage, the child would fail at its first request with a
/// sign-in error; it is refused instead. Without named accounts in play, or
/// when storage cannot be read, nothing is refused.
pub(crate) fn spawn_default_key_error(
    config: &super::Config,
    child_id: &str,
    child: &ModelProviderInfo,
    parent_id: &str,
    parent: &ModelProviderInfo,
) -> Option<String> {
    spawn_default_key_error_with(config, child_id, child, (parent_id, parent), |name| {
        std::env::var(name).is_ok_and(|value| !value.trim().is_empty())
            || crate::model_broker_auth::holds_environment_key(name)
    })
}

fn spawn_default_key_error_with(
    config: &super::Config,
    child_id: &str,
    child: &ModelProviderInfo,
    parent: (&str, &ModelProviderInfo),
    environment_holds: impl Fn(&str) -> bool,
) -> Option<String> {
    if child.account.is_some()
        || child.auth.is_some()
        || child.aws.is_some()
        || child.requires_openai_auth
        || child.is_claude_plan()
        || child.experimental_bearer_token.is_some()
    {
        return None;
    }
    let env_key = child.env_key.as_deref()?;
    let mut configured = config
        .model_providers
        .iter()
        .map(|(id, provider)| (id.as_str(), provider))
        .collect::<Vec<_>>();
    configured.sort_by_key(|(id, _)| *id);
    // The parent first, so the message names it when it is the one on a
    // named account.
    let (index, named_id, account) = std::iter::once(parent)
        .chain(configured)
        .enumerate()
        .find_map(|(index, (id, provider))| {
            let account = provider.account.as_ref()?;
            (provider.env_key.as_deref() == Some(env_key)).then_some((index, id, account))
        })?;
    let on_parent = index == 0;
    if child
        .api_key_env_vars()
        .iter()
        .any(|name| environment_holds(name))
    {
        return None;
    }
    match codex_login::provider_api_key_metadata_from_auth_storage(&config.codex_home, env_key) {
        Ok(
            codex_login::ProviderApiKeyStorageMetadata::Missing
            | codex_login::ProviderApiKeyStorageMetadata::Suppressed,
        ) => {}
        Ok(codex_login::ProviderApiKeyStorageMetadata::Stored { .. }) | Err(_) => return None,
    }
    let named = if on_parent {
        format!("the parent runs on `{named_id}` account `{}`", account.name)
    } else {
        format!(
            "account `{}` is selected for `{named_id}` only",
            account.name
        )
    };
    Some(format!(
        "spawn_agent: `{child_id}` would run on its default account, which has no credential \
         (`{env_key}` is not set and no default key is stored); accounts are selected per \
         connection, and {named}. Spawn without the `model_provider`, `account` or \
         `agent_type` that chose this connection and account to run on the parent's, or ask \
         the user to add a default `{env_key}` (`/providers`). Do not retry on another \
         account without the user's consent."
    ))
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
            // The Claude account helper serves the built-in Claude Plan only,
            // and named sign-ins (`corbanu login --account`) the built-in OpenAI.
            if !supports_named_accounts(provider)
                || (provider.is_claude_plan()
                    && provider_id != codex_model_provider_info::CLAUDE_PLAN_PROVIDER_ID)
                || (provider.requires_openai_auth
                    && provider_id != codex_model_provider_info::OPENAI_PROVIDER_ID)
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

/// API-key providers, `auth.command` providers (including Claude Plan),
/// Amazon Bedrock without command auth (an AWS profile per account) and
/// OpenAI sign-in providers (a ChatGPT or OpenAI API-key sign-in per account,
/// PF-84-S06). Selecting an account of any other provider fails instead of
/// silently using the default credential.
pub fn supports_named_accounts(provider: &ModelProviderInfo) -> bool {
    if provider.requires_openai_auth {
        return provider.env_key.is_none() && provider.auth.is_none() && provider.aws.is_none();
    }
    if provider.aws.is_some() {
        return provider.is_amazon_bedrock() && provider.auth.is_none();
    }
    provider.env_key.is_some() || provider.auth.is_some()
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
            ("plain".to_string(), ModelProviderInfo::default()),
        ])
    }

    fn accounts(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
        pairs
            .iter()
            .map(|(provider, name)| (provider.to_string(), name.to_string()))
            .collect()
    }

    /// #689: a spawned agent on the default key of a connection whose
    /// sibling (or parent) runs on a named account is refused only when the
    /// default key is nowhere: not in the environment, not held by the
    /// broker and not stored.
    #[tokio::test]
    async fn spawn_on_a_default_key_that_is_nowhere_is_refused() {
        let temp_dir = tempfile::tempdir().expect("tempdir");
        use core_test_support::PathBufExt;
        let mut config = crate::config::test_config().await;
        config.codex_home = temp_dir.path().join("codex-home").abs();
        std::fs::create_dir_all(&config.codex_home).expect("create codex home");
        let zai = ModelProviderInfo::create_zai_provider();
        let anthropic = ModelProviderInfo::create_zai_anthropic_provider();
        let mut work = zai.clone();
        stamp_provider_account("zai", &mut work, "work").expect("stamp");
        config.model_providers = HashMap::from([
            ("zai".to_string(), zai.clone()),
            ("zai-anthropic".to_string(), anthropic.clone()),
        ]);
        let nothing_held = |_: &str| false;
        let check = |config: &crate::config::Config,
                     child_id: &str,
                     child: &ModelProviderInfo,
                     parent: (&str, &ModelProviderInfo),
                     held: &dyn Fn(&str) -> bool| {
            spawn_default_key_error_with(config, child_id, child, parent, held)
        };
        let refusal = |named: &str| {
            format!(
                "spawn_agent: `zai-anthropic` would run on its default account, which has no \
                 credential (`ZAI_API_KEY` is not set and no default key is stored); accounts \
                 are selected per connection, and {named}. Spawn without the `model_provider`, \
                 `account` or `agent_type` that chose this connection and account to run on the \
                 parent's, or ask the user to add a default `ZAI_API_KEY` (`/providers`). Do not \
                 retry on another account without the user's consent."
            )
        };
        // The parent runs `zai` on `work`; no default key anywhere.
        assert_eq!(
            check(
                &config,
                "zai-anthropic",
                &anthropic,
                ("zai", &work),
                &nothing_held
            ),
            Some(refusal("the parent runs on `zai` account `work`"))
        );
        // The same, with `[provider_accounts] zai = "work"` and a parent on
        // another provider.
        let mut stamped = config.clone();
        stamped
            .model_providers
            .insert("zai".to_string(), work.clone());
        let kimi = ModelProviderInfo::create_kimi_code_provider();
        assert_eq!(
            check(
                &stamped,
                "zai-anthropic",
                &anthropic,
                ("kimi-code", &kimi),
                &nothing_held
            ),
            Some(refusal("account `work` is selected for `zai` only"))
        );
        // No named account in play: nothing changes.
        assert_eq!(
            check(
                &config,
                "zai-anthropic",
                &anthropic,
                ("zai", &zai),
                &nothing_held
            ),
            None
        );
        // The child on a named account is checked by its enrollment instead.
        let mut anthropic_work = anthropic.clone();
        stamp_provider_account("zai-anthropic", &mut anthropic_work, "work").expect("stamp");
        assert_eq!(
            check(
                &config,
                "zai-anthropic",
                &anthropic_work,
                ("zai", &work),
                &nothing_held
            ),
            None
        );
        // A default key in the environment, or held by the broker.
        assert_eq!(
            check(
                &config,
                "zai-anthropic",
                &anthropic,
                ("zai", &work),
                &|name: &str| name == "ZAI_API_KEY"
            ),
            None
        );
        // A stored default key.
        codex_login::login_with_provider_api_key(
            &config.codex_home,
            "ZAI_API_KEY",
            "canary-689-default",
            codex_login::AuthCredentialsStoreMode::File,
            codex_login::AuthKeyringBackendKind::default(),
        )
        .expect("store default key");
        assert_eq!(
            check(
                &config,
                "zai-anthropic",
                &anthropic,
                ("zai", &work),
                &nothing_held
            ),
            None
        );
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
            Some(&accounts(&[("plain", "work")])),
            /*named_accounts_enabled*/ true,
            &mut Vec::new(),
        )
        .expect_err("a provider without credentials cannot hold accounts");
        assert_eq!(
            error.to_string(),
            "provider_accounts.plain: provider `plain` does not support named accounts yet"
        );
    }

    /// PF-84-S06 slice B: OpenAI sign-in providers take named accounts.
    #[test]
    fn openai_sign_in_accounts_are_stamped() {
        let mut model_providers = providers();
        apply_provider_accounts(
            &mut model_providers,
            Some(&accounts(&[("openai", "work")])),
            /*named_accounts_enabled*/ true,
            &mut Vec::new(),
        )
        .expect("openai sign-in account");
        assert_eq!(
            model_providers["openai"].account,
            Some(NamedProviderAccount {
                provider_id: "openai".to_string(),
                name: "work".to_string(),
            })
        );
    }

    /// PF-84-S06: Bedrock takes AWS profile accounts; Bedrock behind an
    /// `auth.command` and other AWS-configured providers still fail closed.
    #[test]
    fn bedrock_takes_profile_accounts_but_other_aws_providers_fail_closed() {
        let mut command_bedrock =
            ModelProviderInfo::create_amazon_bedrock_provider(/*aws*/ None);
        command_bedrock.auth = Some(
            serde_json::from_value(serde_json::json!({ "command": "token-fetcher" }))
                .expect("auth"),
        );
        let mut custom_aws = ModelProviderInfo::create_zai_provider();
        custom_aws.aws = ModelProviderInfo::create_amazon_bedrock_provider(/*aws*/ None).aws;
        let mut model_providers = HashMap::from([
            (
                "amazon-bedrock".to_string(),
                ModelProviderInfo::create_amazon_bedrock_provider(/*aws*/ None),
            ),
            ("command-bedrock".to_string(), command_bedrock),
            ("custom-aws".to_string(), custom_aws),
        ]);
        apply_provider_accounts(
            &mut model_providers,
            Some(&accounts(&[("amazon-bedrock", "work")])),
            /*named_accounts_enabled*/ true,
            &mut Vec::new(),
        )
        .expect("bedrock profile accounts");
        assert_eq!(
            model_providers["amazon-bedrock"].account,
            Some(NamedProviderAccount {
                provider_id: "amazon-bedrock".to_string(),
                name: "work".to_string(),
            })
        );
        for provider_id in ["command-bedrock", "custom-aws"] {
            let error = apply_provider_accounts(
                &mut model_providers,
                Some(&accounts(&[(provider_id, "work")])),
                /*named_accounts_enabled*/ true,
                &mut Vec::new(),
            )
            .expect_err("not supported");
            assert_eq!(
                error.to_string(),
                format!(
                    "provider_accounts.{provider_id}: provider `{provider_id}` does not support \
                     named accounts yet"
                )
            );
        }
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
                error("plain:work", true, &mut model_providers),
            ],
            [
                "--account zai:work: named accounts need the `named_accounts` feature \
                 (`--enable named_accounts`); `--account default` uses the default credentials"
                    .to_string(),
                "--account gone:work: unknown provider `gone`".to_string(),
                "--account plain:work: provider `plain` does not support named accounts yet"
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
                unused_other_provider_account(
                    "zai:work", "zai", /*spawns_possible*/ false,
                    /*agent_provider_allowlist*/ None
                ),
                unused_other_provider_account(
                    "kimi-code:main",
                    "zai",
                    /*spawns_possible*/ true,
                    /*agent_provider_allowlist*/ None
                ),
                unused_other_provider_account(
                    "kimi-code:main",
                    "zai",
                    /*spawns_possible*/ true,
                    Some(&["kimi-code".to_string()]),
                ),
                unused_other_provider_account(
                    "kimi-code:default",
                    "zai",
                    /*spawns_possible*/ false,
                    /*agent_provider_allowlist*/ None
                ),
            ],
            [None, None, None, None]
        );
        assert_eq!(
            [
                unused_other_provider_account("kimi-code:main", "zai", /*spawns_possible*/ false, /*agent_provider_allowlist*/ None),
                unused_other_provider_account("kimi-code:main", "zai", /*spawns_possible*/ true, Some(&allow_zai)),
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
             feature (`--enable named_accounts`); `--account zai:default` (on a resume or fork) \
             uses the default credentials instead"
        );
    }
}
