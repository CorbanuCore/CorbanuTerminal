//! PF-84-S06 (#658): the sign-in the process's own ChatGPT backend requests
//! use (plugin listings, usage, settings, the apps directory, analytics):
//! the account its user sessions run on, never the default login while a
//! named account is in use.
//!
//! Every user (root) session registers the OpenAI sign-in it runs on and
//! updates it on a switch; the registration ends with the session and does
//! not count while no client is attached to it. Kept per
//! home rather than per manager, so a manager built later from the same
//! configuration (the ChatGPT connector helpers build their own) sees it.
//!
//! [`AuthManager::for_session`] resolves the registrations of a home:
//! - none, or only sessions on the default login: the default login;
//! - sessions on one named account only: that account;
//! - sessions on different accounts (panes on `work` and on the default
//!   login, or on two named accounts): nothing. Such a request goes out
//!   without credentials or not at all; [`AuthManager::session_account_conflict`]
//!   says why.
//!
//! Sessions on providers without OpenAI sign-in (Z.AI, Bedrock, ...) do not
//! count. Before the first session of a launch whose account is known only
//! later (resume or fork with named accounts on) nothing is sent.

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::LazyLock;
use std::sync::Mutex;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

use super::AuthManager;
use super::CodexAuth;
use crate::auth::named_chatgpt::NamedChatgptAccount;

/// The OpenAI sign-in a user session runs on.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SessionSignIn {
    /// A provider without OpenAI sign-in: it does not count.
    NotOpenAi,
    /// The default login.
    Default,
    /// A named sign-in account.
    Named(NamedChatgptAccount),
    /// Not known yet: a resumed or forked thread's account is known only when
    /// it starts. Only for [`AuthManager::register_configured_sign_in`]; until
    /// then nothing is sent.
    Unknown,
}

/// Registration key 0 is the account the configuration starts the process on,
/// which counts only while no user session is registered.
const CONFIGURED: u64 = 0;

type Registrations = BTreeMap<u64, SessionSignIn>;

static SESSIONS: LazyLock<Mutex<HashMap<PathBuf, Registrations>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
static NEXT_ID: AtomicU64 = AtomicU64::new(CONFIGURED + 1);
/// Homes whose configured stand-in turns [`SessionSignIn::Unknown`] once a
/// session on a named account registers (#1085). Locked only inside
/// [`with_sessions`].
static HOLD_ONCE_NAMED: LazyLock<Mutex<HashSet<PathBuf>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));

fn with_sessions<T>(f: impl FnOnce(&mut HashMap<PathBuf, Registrations>) -> T) -> T {
    f(&mut SESSIONS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner))
}

/// Turns the configured stand-in of `home` [`SessionSignIn::Unknown`] for
/// good when it is held once named and `registrations` hold a named account.
/// Callers hold the sessions lock.
fn hold_if_named(home: &PathBuf, registrations: &mut Registrations) {
    if !registrations
        .values()
        .any(|sign_in| matches!(sign_in, SessionSignIn::Named(_)))
    {
        return;
    }
    let mut held = HOLD_ONCE_NAMED
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if held.remove(home) {
        registrations.insert(CONFIGURED, SessionSignIn::Unknown);
    }
}

fn set(home: &PathBuf, id: u64, sign_in: SessionSignIn) {
    with_sessions(|sessions| {
        let registrations = sessions.entry(home.clone()).or_default();
        match sign_in {
            SessionSignIn::NotOpenAi => {
                registrations.remove(&id);
            }
            sign_in => {
                registrations.insert(id, sign_in);
                hold_if_named(home, registrations);
            }
        }
        if registrations.is_empty() {
            sessions.remove(home);
        }
    });
}

/// What the registrations of a home resolve to.
enum Resolved {
    Default,
    Named(NamedChatgptAccount),
    Conflict(Vec<String>),
    Unknown,
}

fn resolve(home: &PathBuf) -> Resolved {
    with_sessions(|sessions| {
        let Some(registrations) = sessions.get(home) else {
            return Resolved::Default;
        };
        // The configured stand-in counts only while no session is registered.
        let sessions_only = registrations.keys().any(|id| *id != CONFIGURED);
        let sign_ins: Vec<&SessionSignIn> = registrations
            .iter()
            .filter(|(id, _)| !sessions_only || **id != CONFIGURED)
            .map(|(_, sign_in)| sign_in)
            .collect();
        if sign_ins.contains(&&SessionSignIn::Unknown) {
            return Resolved::Unknown;
        }
        let named: BTreeSet<&NamedChatgptAccount> = sign_ins
            .iter()
            .filter_map(|sign_in| match sign_in {
                SessionSignIn::Named(account) => Some(account),
                _ => None,
            })
            .collect();
        let default = sign_ins.contains(&&SessionSignIn::Default);
        let mut accounts = named.into_iter();
        match (accounts.next(), accounts.next(), default) {
            (None, _, _) => Resolved::Default,
            (Some(account), None, false) => Resolved::Named(account.clone()),
            (first, second, default) => Resolved::Conflict(
                first
                    .into_iter()
                    .chain(second)
                    .chain(accounts)
                    .map(ToString::to_string)
                    .chain(default.then(|| "the default login".to_string()))
                    .collect(),
            ),
        }
    })
}

/// A user session's registration; it ends when this is dropped. While no
/// client is attached to the session (a conversation the user left, kept
/// loaded for a while) it is suspended and does not count.
#[derive(Debug)]
pub struct SessionSignInRegistration {
    home: PathBuf,
    id: u64,
    /// The sign-in, and whether a client is attached.
    state: Mutex<(SessionSignIn, bool)>,
}

impl SessionSignInRegistration {
    /// The session now runs on `sign_in` (a mid-session switch).
    pub fn update(&self, sign_in: SessionSignIn) {
        let sign_in = match sign_in {
            SessionSignIn::Unknown => SessionSignIn::NotOpenAi,
            sign_in => sign_in,
        };
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.0 = sign_in.clone();
        if state.1 {
            set(&self.home, self.id, sign_in);
        }
    }

    /// Whether a client is attached to the session; a detached session does
    /// not count until one attaches again.
    pub fn set_attached(&self, attached: bool) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if state.1 == attached {
            return;
        }
        state.1 = attached;
        let sign_in = if attached {
            state.0.clone()
        } else {
            SessionSignIn::NotOpenAi
        };
        set(&self.home, self.id, sign_in);
    }
}

impl Drop for SessionSignInRegistration {
    fn drop(&mut self) {
        set(&self.home, self.id, SessionSignIn::NotOpenAi);
    }
}

impl AuthManager {
    /// Registers a user session that runs on `sign_in` (#658), attached.
    /// While any session is registered the configured stand-in does not count.
    pub fn register_session_sign_in(&self, sign_in: SessionSignIn) -> SessionSignInRegistration {
        let sign_in = match sign_in {
            SessionSignIn::Unknown => SessionSignIn::NotOpenAi,
            sign_in => sign_in,
        };
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        set(&self.codex_home, id, sign_in.clone());
        SessionSignInRegistration {
            home: self.codex_home.clone(),
            id,
            state: Mutex::new((sign_in, true)),
        }
    }

    /// The account the configuration starts the process on (#658), which
    /// counts while no user session of this home is registered: an app
    /// server's requests before any thread exists.
    pub fn register_configured_sign_in(&self, sign_in: SessionSignIn) {
        set(&self.codex_home, CONFIGURED, sign_in);
    }

    /// #1085: a server whose clients could choose a named account it did not
    /// know of at startup (one enrolled while it runs). Once a session on a
    /// named account registers in this home, the configured stand-in becomes
    /// [`SessionSignIn::Unknown`] for the rest of the process, so requests
    /// outside a conversation never fall back to the default login.
    pub fn hold_configured_sign_in_once_named(&self) {
        with_sessions(|sessions| {
            HOLD_ONCE_NAMED
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .insert(self.codex_home.clone());
            if let Some(registrations) = sessions.get_mut(&self.codex_home) {
                hold_if_named(&self.codex_home, registrations);
            }
        });
    }

    /// The manager of the sign-in this process's user sessions run on
    /// (module docs), or `None` when they run on different accounts. A named
    /// account without a usable sign-in is returned as is (it yields no auth),
    /// never replaced by the default login.
    pub fn for_session(self: &Arc<Self>) -> Option<Arc<AuthManager>> {
        let base = self.default_account_manager();
        match resolve(&base.codex_home) {
            Resolved::Default => Some(base),
            Resolved::Named(account) => Some(base.for_named_account(account)),
            Resolved::Conflict(_) | Resolved::Unknown => None,
        }
    }

    /// The auth of [`AuthManager::for_session`].
    pub async fn session_auth(self: &Arc<Self>) -> Option<CodexAuth> {
        match self.for_session() {
            Some(manager) => manager.auth().await,
            None => None,
        }
    }

    /// The cached auth of [`AuthManager::for_session`].
    pub fn session_auth_cached(self: &Arc<Self>) -> Option<CodexAuth> {
        self.for_session()?.auth_cached()
    }

    /// Whether the session's account is not known yet (a resumed or forked
    /// thread before it starts). The default login's model list is then
    /// served from its cache only.
    pub fn session_account_unknown(&self) -> bool {
        matches!(resolve(&self.codex_home), Resolved::Unknown)
    }

    /// Why [`AuthManager::for_session`] has no sign-in: the user sessions run
    /// on different accounts. `None` otherwise.
    pub fn session_account_conflict(&self) -> Option<String> {
        match resolve(&self.codex_home) {
            Resolved::Conflict(accounts) => Some(format!(
                "sessions on different accounts are open ({}); requests outside a \
                 conversation are not sent with any of them",
                accounts.join(", ")
            )),
            Resolved::Unknown => {
                Some("the session's account is not known until its conversation starts".to_string())
            }
            Resolved::Default | Resolved::Named(_) => None,
        }
    }
}

#[cfg(test)]
#[path = "manager_session_account_tests.rs"]
mod tests;
