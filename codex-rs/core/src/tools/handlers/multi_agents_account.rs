//! PF-84-S03 (decision D3): the spawn tool's `account` argument. A spawned
//! agent may run on another named account of its provider than its parent,
//! but only on an account that is already configured, and under the
//! Aggressive security level only after the human approves the switch. The
//! model sees account names, never credential values.
//!
//! "Aggressive" is Core's level in force or the level Corbanu Terminal
//! enforces at launch and shows (#428, Travis 2026-10-11, option 1), which
//! can be Aggressive while Core's level stays lower (no activation preflight).

use crate::config::Config;
use crate::config::configured_account_names;
use crate::config::selected_account_error;
use crate::config::stamp_provider_account;
use crate::function_tool::FunctionCallError;
use crate::security::protected_surface::ask_human;
use crate::session::session::Session;
use crate::session::turn_context::TurnContext;
use codex_features::Feature;
use codex_protocol::protocol::AskForApproval;
use codex_security_policy::SecurityLevel;

/// Applies `requested` to the child's resolved provider. Omitted keeps the
/// inherited account; `default` selects today's credentials.
pub(crate) async fn apply_spawn_agent_account(
    session: &Session,
    turn: &TurnContext,
    call_id: &str,
    config: &mut Config,
    requested: Option<&str>,
) -> Result<(), FunctionCallError> {
    let Some(requested) = requested.map(str::trim).filter(|name| !name.is_empty()) else {
        return Ok(());
    };
    let refuse = |message: String| Err(FunctionCallError::RespondToModel(message));
    if !config.features.enabled(Feature::NamedAccounts) {
        return refuse(
            "spawn_agent `account` needs the `named_accounts` feature; omit it to use the \
             parent's account."
                .to_string(),
        );
    }
    let provider_id = config.model_provider_id.clone();
    let mut provider = config.model_provider.clone();
    if let Err(error) = stamp_provider_account(&provider_id, &mut provider, requested) {
        let message = error.to_string();
        let prefix = format!("provider_accounts.{provider_id}: ");
        return refuse(format!(
            "spawn_agent account: {}",
            message.strip_prefix(&prefix).unwrap_or(&message)
        ));
    }
    if let Some(message) = selected_account_error(config.codex_home.as_path(), &provider) {
        let names = configured_account_names(config.codex_home.as_path(), &provider_id);
        let configured = if names.is_empty() {
            "none besides `default`".to_string()
        } else {
            format!("`default`, {}", names.join(", "))
        };
        return refuse(format!(
            "{message}. Configured accounts of `{provider_id}`: {configured}. Do not retry \
             on another account without the user's consent."
        ));
    }
    if provider.account == config.model_provider.account {
        return Ok(());
    }
    // The level in force, including a stricter one committed during the
    // session or inherited from the parent; an unreadable policy counts as
    // Aggressive.
    let level = session
        .services
        .agent_control
        .effective_security_policy()
        .snapshot_for_agent(session.thread_id)
        .map_or(SecurityLevel::Aggressive, |policy| {
            turn.config.security_level.max(policy.level)
        });
    if level == SecurityLevel::Aggressive || turn.config.permissions.launch_enforces_aggressive() {
        let shown = provider
            .account
            .as_ref()
            .map_or("default", |account| account.name.as_str());
        if turn.approval_policy.value() == AskForApproval::Never {
            return refuse(format!(
                "Running a spawned agent on account `{shown}` of `{provider_id}` needs the \
                 user's approval under the Aggressive security level, and approvals are off."
            ));
        }
        let question = format!(
            "Allow the spawned agent to run on account `{shown}` of `{provider_id}` instead \
             of the parent's account?"
        );
        if !ask_human(session, turn, call_id, question).await {
            return refuse(format!(
                "The user declined running the spawned agent on account `{shown}` of \
                 `{provider_id}`. Do not retry on another account without the user's consent."
            ));
        }
    }
    config.model_provider = provider;
    Ok(())
}

/// The account name shown with a spawn, when named accounts are on.
pub(crate) fn spawned_account_name(config: &Config) -> Option<String> {
    config.features.enabled(Feature::NamedAccounts).then(|| {
        config
            .model_provider
            .account
            .as_ref()
            .map_or_else(|| "default".to_string(), |account| account.name.clone())
    })
}
