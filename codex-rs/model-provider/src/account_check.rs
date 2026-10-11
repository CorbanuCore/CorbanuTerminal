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
    } else if provider.env_key.is_some() {
        &[ProviderAccountKind::ApiKey]
    } else {
        &[]
    }
}

/// Why the provider's selected named account cannot be used, with recovery
/// text; `None` for the `default` account or a configured one.
pub fn selected_account_error(codex_home: &Path, provider: &ModelProviderInfo) -> Option<String> {
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
        Ok(false) => Some(format!(
            "account `{name}` of provider `{provider_id}` is not configured; add it with \
             `corbanu account add {provider_id} {name}` or pick another with `--account` \
             (see `corbanu account list`)"
        )),
        Err(error) => Some(format!(
            "account `{name}` of provider `{provider_id}` is unavailable: {error}"
        )),
    }
}
