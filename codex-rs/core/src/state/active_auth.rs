//! PF-84: the sign-in a session's account-bound services use (MCP
//! connectors, plugin installs, accounting, the model list), following the
//! session's current provider: its named ChatGPT/OpenAI sign-in account's
//! manager, or the default login's.
//!
//! A session used to bind this once at start, so after a mid-session switch
//! from a named account to the default account (or between named accounts)
//! model requests used the new account while everything else stayed on the
//! old one.

use std::sync::Arc;

use arc_swap::ArcSwap;
use codex_login::AuthManager;
use codex_login::SessionSignInRegistration;
use codex_model_provider_info::ModelProviderInfo;
use tokio::sync::watch;

pub(crate) struct ActiveAuthManager {
    current: ArcSwap<AuthManager>,
    /// Bumped on every switch, so watchers of the old manager's auth changes
    /// can move to the new one.
    switches: watch::Sender<u64>,
    /// #658: a user session's registration of the sign-in it runs on, which
    /// the process's own backend requests follow (`AuthManager::for_session`).
    user_session: Option<SessionSignInRegistration>,
}

impl ActiveAuthManager {
    pub(crate) fn new(auth_manager: Arc<AuthManager>) -> Self {
        let (switches, _) = watch::channel(0);
        Self {
            current: ArcSwap::new(auth_manager),
            switches,
            user_session: None,
        }
    }

    /// For a user (root) session: registers the sign-in `provider` uses
    /// until the session ends; a switch updates it ([`Self::follow_provider`]).
    pub(crate) fn for_user_session(
        mut self,
        user_session: bool,
        provider: &ModelProviderInfo,
    ) -> Self {
        if user_session {
            self.user_session = Some(
                self.get()
                    .register_session_sign_in(codex_model_provider::session_sign_in(provider)),
            );
        }
        self
    }

    pub(crate) fn get(&self) -> Arc<AuthManager> {
        self.current.load_full()
    }

    /// The manager `provider` signs in with, from the same default login.
    pub(crate) fn for_provider(&self, provider: &ModelProviderInfo) -> Arc<AuthManager> {
        let base = self.get().default_account_manager();
        match codex_model_provider::named_sign_in_account(provider) {
            Some(account) => base.for_named_account(account),
            None => base,
        }
    }

    /// Makes `provider`'s sign-in the active one. Returns whether it changed.
    /// Runs under the session state lock; a named account's ChatGPT or
    /// API-key sign-in is read from the vault here (other kinds on first use).
    pub(crate) fn follow_provider(&self, provider: &ModelProviderInfo) -> bool {
        if let Some(user_session) = &self.user_session {
            user_session.update(codex_model_provider::session_sign_in(provider));
        }
        let next = self.for_provider(provider);
        if Arc::ptr_eq(&next, &self.get()) {
            return false;
        }
        self.current.store(next);
        self.switches.send_modify(|switches| *switches += 1);
        true
    }

    /// #658: whether a client is attached to this user session; a session
    /// the user left does not count for the process's own backend requests.
    pub(crate) fn set_user_attached(&self, attached: bool) {
        if let Some(user_session) = &self.user_session {
            user_session.set_attached(attached);
        }
    }

    pub(crate) fn subscribe_switches(&self) -> watch::Receiver<u64> {
        self.switches.subscribe()
    }

    #[cfg(test)]
    pub(crate) fn replace_for_testing(&self, auth_manager: Arc<AuthManager>) {
        self.current.store(auth_manager);
    }
}
