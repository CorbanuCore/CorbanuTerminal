//! PF-84-S06: whether a TUI launch leaves the default OpenAI login in place
//! or enforces the login restrictions on it at startup.

use crate::legacy_core::config::Config;
use codex_app_server_client::RemoteAppServerClient;
use codex_login::enforce_login_restrictions;

use crate::AppServerTarget;

/// PF-84-S06: what a launch does with the default OpenAI login.
pub(crate) struct DefaultLoginAtLaunch {
    /// The app server the launch uses: a local daemon is replaced by the
    /// embedded one when the launch leaves the default login in place (#658).
    pub(crate) app_server_target: AppServerTarget,
    /// The default login was left in place; the caller records it with
    /// `codex_login::leave_default_login_unenforced` before any auth manager
    /// is built.
    pub(crate) left_in_place: bool,
    /// Why a running local daemon was not reused, for the startup warnings
    /// (logging is not set up yet at this point of the launch).
    pub(crate) daemon_not_reused: Option<String>,
}

const DAEMON_NOT_REUSED: &str = "This session did not reuse the running local app-server \
     daemon: it runs on a named account, or resumes or forks a conversation whose account is \
     known only later, and the daemon may already serve another session on another account. \
     It started its own app server instead.";

/// PF-84-S06 (#570, #595): a session on a named sign-in account checks the
/// restrictions against that account at thread start and never reads or
/// logs out the default login. A resumed or forked thread's account is known
/// only later, so those launches leave the default login in place too. The
/// app server then never uses a ruled-out default login and refuses a thread
/// on it with the reason, because this process records it. Such a launch
/// never reuses a running local daemon (#658: the daemon may serve another
/// session on another account); it starts its own app server. A remote
/// server decides for itself. Every other launch enforces at startup as
/// before.
pub(crate) async fn settle_default_login_at_launch(
    config: &Config,
    launches_resume_or_fork: bool,
    app_server_target: AppServerTarget,
) -> std::io::Result<DefaultLoginAtLaunch> {
    let leaves_default_login = !app_server_target.uses_remote_workspace()
        && (config.runs_on_named_sign_in_account()
            || (config.named_accounts_enabled() && launches_resume_or_fork));
    if !leaves_default_login {
        if !app_server_target.uses_remote_workspace() {
            enforce_login_restrictions(&config.default_login_restrictions()).await?;
        }
        return Ok(DefaultLoginAtLaunch {
            app_server_target,
            left_in_place: false,
            daemon_not_reused: None,
        });
    }
    // #658: a daemon may already serve another client's conversation on
    // another account; until this launch's own conversation starts, the
    // daemon's requests outside a conversation would use that account (the
    // default login, say). Such a launch starts its own app server instead.
    let (app_server_target, daemon_not_reused) = match app_server_target {
        AppServerTarget::LocalDaemon { .. } => (
            AppServerTarget::Embedded,
            Some(DAEMON_NOT_REUSED.to_string()),
        ),
        target => (target, None),
    };
    Ok(DefaultLoginAtLaunch {
        app_server_target,
        left_in_place: true,
        daemon_not_reused,
    })
}

/// PF-84-S06 (#595): the warning [`settle_default_login_at_launch`] added
/// to `config`, if any, so a reloaded config can carry it.
pub(crate) fn daemon_not_reused_warning(config: &Config) -> Option<String> {
    config
        .startup_warnings
        .iter()
        .find(|warning| warning.as_str() == DAEMON_NOT_REUSED)
        .cloned()
}

/// PF-84-S06 (#595): whether a connected local daemon leaves a ruled-out
/// default login in place and refuses to use it, under the same login
/// restrictions as this launch. A launch that leaves the login in place no
/// longer targets a local daemon (#658); this stays as a guard for any
/// connection made with that setting.
pub(crate) fn daemon_matches_default_login_policy(
    client: &RemoteAppServerClient,
    config: &Config,
) -> bool {
    client.ruled_out_default_login().is_some_and(|policy| {
        policy.forced_login_method == config.configured_forced_login_method
            && policy.forced_chatgpt_workspace_id == config.forced_chatgpt_workspace_id
    })
}

#[cfg(test)]
#[path = "default_login_launch_tests.rs"]
mod tests;
