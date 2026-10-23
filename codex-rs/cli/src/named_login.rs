//! PF-84-S06 slice B: `corbanu login|logout --account <name>` signs a named
//! OpenAI account in or out. The sign-in goes to that account's vault store;
//! the default login is never cleared, replaced or read for credentials.

use codex_core::config::Config;
use codex_features::Feature;
use codex_login::CLIENT_ID;
use codex_login::NamedChatgptAccount;
use codex_login::ServerOptions;
use codex_login::load_auth_dot_json;
use codex_login::login_named_account_with_api_key;
use codex_login::logout_named_account_with_revoke;
use codex_login::named_account_summary;
use codex_login::run_device_code_login;
use codex_login::run_login_server;
use codex_login::same_user_sign_ins;
use codex_protocol::config_types::ForcedLoginMethod;
use codex_utils_cli::CliConfigOverrides;

/// Named sign-ins belong to the built-in OpenAI provider.
pub(crate) const OPENAI_PROVIDER_ID: &str = "openai";

/// How a named account signs in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NamedLoginMethod {
    Browser,
    DeviceCode,
    ApiKey(String),
}

fn exit_with(message: impl std::fmt::Display) -> ! {
    eprintln!("{message}");
    std::process::exit(1);
}

async fn config_and_account(
    cli_config_overrides: CliConfigOverrides,
    name: &str,
) -> (Config, NamedChatgptAccount) {
    let overrides = cli_config_overrides
        .parse_overrides()
        .unwrap_or_else(|error| exit_with(format!("Error parsing -c overrides: {error}")));
    let config = Config::load_with_cli_overrides(overrides)
        .await
        .unwrap_or_else(|error| exit_with(format!("Error loading configuration: {error}")));
    if !config.features.enabled(Feature::NamedAccounts) {
        exit_with(
            "named accounts are off; enable them with `[features] named_accounts = true` or \
             `--enable named_accounts`",
        );
    }
    let name = codex_vault::ProviderAccountName::parse(name)
        .unwrap_or_else(|error| exit_with(format!("--account {name}: {error}")));
    let supported = config
        .model_providers
        .get(OPENAI_PROVIDER_ID)
        .is_some_and(codex_core::config::supports_named_accounts);
    if !supported {
        exit_with("the OpenAI provider in this configuration cannot hold named accounts");
    }
    let account = NamedChatgptAccount {
        provider_id: OPENAI_PROVIDER_ID.to_string(),
        name: name.as_str().to_string(),
    };
    (config, account)
}

/// Signs `name` in and reports what was saved, warning when the same
/// ChatGPT user is already signed in under another account of this home.
pub async fn run_named_account_login(
    cli_config_overrides: CliConfigOverrides,
    name: &str,
    method: NamedLoginMethod,
    issuer_base_url: Option<String>,
) -> ! {
    let (config, account) = config_and_account(cli_config_overrides, name).await;
    // #573: only the configured restriction counts. The API-key-only default
    // a third-party session provider implies is about the default login's
    // onboarding, not a named OpenAI account.
    let api_key_login = matches!(method, NamedLoginMethod::ApiKey(_));
    match (config.configured_forced_login_method, api_key_login) {
        (Some(ForcedLoginMethod::Api), false) => exit_with(format!(
            "ChatGPT sign-in is disabled by `forced_login_method = \"api\"`; sign account \
             `{name}` in with an OpenAI API key: `corbanu login --account {name} --with-api-key`."
        )),
        (Some(ForcedLoginMethod::Chatgpt), true) => exit_with(format!(
            "API key sign-in is disabled by `forced_login_method = \"chatgpt\"`; sign account \
             `{name}` in with ChatGPT: `corbanu login --account {name}` (add `--device-auth` \
             for a device code)."
        )),
        _ => {}
    }
    let auth_route_config = config.auth_route_config();
    // The earlier sign-in stays until the new one is saved; then only its
    // tokens are revoked. A cancelled sign-in leaves the account as it was.
    let previous = codex_login::current_named_sign_in(&config.codex_home, &account);
    let result = match method {
        NamedLoginMethod::ApiKey(api_key) => {
            login_named_account_with_api_key(&config.codex_home, &account, &api_key)
        }
        NamedLoginMethod::Browser | NamedLoginMethod::DeviceCode => {
            let mut opts = ServerOptions::new(
                config.codex_home.to_path_buf(),
                CLIENT_ID.to_string(),
                config.forced_chatgpt_workspace_id.clone(),
                config.cli_auth_credentials_store_mode,
                config.auth_keyring_backend_kind(),
                auth_route_config.clone(),
            );
            opts.named_account = Some(account.clone());
            if let Some(issuer) = issuer_base_url {
                opts.issuer = issuer;
            }
            if method == NamedLoginMethod::DeviceCode {
                run_device_code_login(opts).await
            } else {
                match run_login_server(opts) {
                    Ok(server) => {
                        eprintln!(
                            "Signing in OpenAI account `{name}`. If your browser did not open, \
                             navigate to this URL:\n\n{}\n",
                            server.auth_url
                        );
                        server.block_until_done().await
                    }
                    Err(error) => Err(error),
                }
            }
        }
    };
    if let Err(error) = result {
        exit_with(format!("Error signing in account `{name}`: {error}"));
    }
    if let Some(previous) = previous {
        codex_login::revoke_replaced_sign_in(
            &config.codex_home,
            &account,
            previous,
            &auth_route_config,
        )
        .await;
    }
    match named_account_summary(&config.codex_home, &account) {
        Ok(Some(summary)) => eprintln!(
            "Signed in OpenAI account `{name}` ({}).",
            describe(&summary)
        ),
        Ok(None) | Err(_) => eprintln!("Signed in OpenAI account `{name}`."),
    }
    warn_same_user(&config, &account);
    std::process::exit(0);
}

fn describe(summary: &codex_login::NamedSignInSummary) -> String {
    [
        Some(summary.kind.to_string()),
        summary.email_hint.clone(),
        summary.plan.clone(),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join(" · ")
}

/// Travis, 2026-10-11: the same ChatGPT user under two accounts is allowed,
/// with a warning.
fn warn_same_user(config: &Config, account: &NamedChatgptAccount) {
    let default_auth = load_auth_dot_json(
        &config.codex_home,
        config.cli_auth_credentials_store_mode,
        config.auth_keyring_backend_kind(),
    )
    .ok()
    .flatten();
    let same =
        same_user_sign_ins(&config.codex_home, account, default_auth.as_ref()).unwrap_or_default();
    if !same.is_empty() {
        eprintln!(
            "Warning: this is the same ChatGPT user as account(s) {}. They share that user's \
             usage limits, and signing out of one can end the other's session.",
            same.iter()
                .map(|name| format!("`{name}`"))
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
}

/// `corbanu login status --account <name>`.
pub async fn run_named_account_login_status(
    cli_config_overrides: CliConfigOverrides,
    name: &str,
) -> ! {
    let (config, account) = config_and_account(cli_config_overrides, name).await;
    match named_account_summary(&config.codex_home, &account) {
        Ok(Some(summary)) => {
            eprintln!("OpenAI account `{name}`: {}", describe(&summary));
            std::process::exit(0);
        }
        Ok(None) => exit_with(format!("OpenAI account `{name}` is not signed in")),
        Err(error) => exit_with(format!("Error checking account `{name}`: {error}")),
    }
}

/// `corbanu logout --account <name>`: revokes and clears that account only.
pub async fn run_named_account_logout(cli_config_overrides: CliConfigOverrides, name: &str) -> ! {
    let (config, account) = config_and_account(cli_config_overrides, name).await;
    match logout_named_account_with_revoke(
        &config.codex_home,
        &account,
        &config.auth_route_config(),
    )
    .await
    {
        Ok(true) => {
            eprintln!("Signed out OpenAI account `{name}`");
            std::process::exit(0);
        }
        Ok(false) => {
            eprintln!("OpenAI account `{name}` is not signed in");
            std::process::exit(0);
        }
        Err(error) => exit_with(format!("Error signing out account `{name}`: {error}")),
    }
}
