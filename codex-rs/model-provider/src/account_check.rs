//! PF-84: whether a provider's selected named account can be used, checked
//! from vault metadata only (no secret is decrypted) before a session,
//! resumed thread or spawned worker starts on it.

use std::path::Path;

use codex_login::ProviderAccountKind;
use codex_login::provider_account_holds;
use codex_model_provider_info::CLAUDE_PLAN_PROVIDER_ID;
use codex_model_provider_info::ModelProviderInfo;
use codex_model_provider_info::NamedProviderAccount;

/// The account kinds that make a named account usable for `provider`.
fn usable_account_kinds(provider: &ModelProviderInfo) -> &'static [ProviderAccountKind] {
    let is_claude_plan = provider.is_claude_plan()
        && provider
            .account
            .as_ref()
            .is_some_and(|account| account.provider_id == CLAUDE_PLAN_PROVIDER_ID);
    if is_claude_plan {
        &[
            ProviderAccountKind::ClaudeOauthToken,
            ProviderAccountKind::ClaudeConfigDir,
        ]
    } else if provider.auth.is_some() {
        &[ProviderAccountKind::Command]
    } else if provider.requires_openai_auth {
        &[ProviderAccountKind::ChatgptAuth]
    } else if provider.is_amazon_bedrock() {
        &[ProviderAccountKind::AwsProfile]
    } else if provider.env_key.is_some() {
        &[ProviderAccountKind::ApiKey]
    } else {
        &[]
    }
}

/// Where the user recovers a missing named account.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccountRecovery {
    /// `corbanu account add`, or another account with `--account`.
    CommandLine,
    /// Inside the TUI: `/providers` adds the account or picks another one.
    Providers,
}

/// Why the provider's selected named account cannot be used, with recovery
/// text; `None` for the `default` account or a configured one.
pub fn selected_account_error(codex_home: &Path, provider: &ModelProviderInfo) -> Option<String> {
    selected_account_error_with_recovery(codex_home, provider, AccountRecovery::CommandLine)
}

/// [`selected_account_error`] with the recovery `recovery` names.
pub fn selected_account_error_with_recovery(
    codex_home: &Path,
    provider: &ModelProviderInfo,
    recovery: AccountRecovery,
) -> Option<String> {
    let NamedProviderAccount { provider_id, name } = provider.account.as_ref()?;
    let mut configured = Ok(false);
    for kind in usable_account_kinds(provider) {
        configured = provider_account_holds(codex_home, provider_id, name, *kind);
        if !matches!(configured, Ok(false)) {
            break;
        }
    }
    match configured {
        Ok(true) => None,
        Ok(false) if recovery == AccountRecovery::Providers => Some(format!(
            "account `{name}` of provider `{provider_id}` is not configured; add it or pick \
             another account in `/providers`"
        )),
        Ok(false) => {
            if usable_account_kinds(provider) == [ProviderAccountKind::ChatgptAuth] {
                return Some(format!(
                    "account `{name}` of provider `{provider_id}` is not signed in; sign in with \
                     `corbanu login --account {name}` or pick another with `--account` (see \
                     `corbanu account list`)"
                ));
            }
            let kind = if provider.is_amazon_bedrock() && provider.auth.is_none() {
                " --kind aws-profile --value <profile>"
            } else {
                ""
            };
            Some(format!(
                "account `{name}` of provider `{provider_id}` is not configured; add it with \
                 `corbanu account add {provider_id} {name}{kind}` or pick another with \
                 `--account` (see `corbanu account list`)"
            ))
        }
        Err(error) => Some(format!(
            "account `{name}` of provider `{provider_id}` is unavailable: {error}"
        )),
    }
}

#[cfg(test)]
#[path = "account_check_tests.rs"]
mod tests;
