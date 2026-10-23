//! #878: which brokered requests the model-key broker refused, per session.
//!
//! The inspector shows the model-key broker as degraded while the last
//! brokered request of the inspected session (or one of its agents), or of
//! this process outside any session (the model list, the wallet), was
//! refused. Every refusal counts: a provider URL the broker cannot bind, a
//! credential it cannot register (no key, a store it cannot read, a broker
//! that is gone or rejects it), a request it cannot sign, and a request the
//! broker itself denies. Only a later request of the same owner that the
//! broker serves clears it. A broker that died after a credential was
//! registered is not reported by the transport (its connection error could
//! also come mid-response); the next auth finds it gone and records that.
//! The broker passes a provider's error response on unchanged, so a provider
//! can make one look like a denial: that only shows the row degraded.

use codex_protocol::ThreadId;
use http::HeaderValue;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::future::Future;
use std::sync::Mutex;
use std::sync::PoisonError;

use super::BrokerModelAuthError;

tokio::task_local! {
    /// The session whose model client resolves auth on this task.
    static REQUEST_OWNER: ThreadId;
}

/// Runs `future` with brokered auth it resolves owned by `thread`.
pub(crate) async fn owned_by<F: Future>(thread: ThreadId, future: F) -> F::Output {
    REQUEST_OWNER.scope(thread, future).await
}

/// The session resolving auth on this task, or `None` outside one.
pub(crate) fn current_owner() -> Option<ThreadId> {
    REQUEST_OWNER.try_with(|thread| *thread).ok()
}

/// Runs `f` with brokered auth it resolves owned by `thread` (tests).
#[cfg(test)]
pub(crate) fn owned_by_sync<T>(thread: ThreadId, f: impl FnOnce() -> T) -> T {
    REQUEST_OWNER.sync_scope(thread, f)
}

/// Why the broker refused a request.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Refusal {
    /// Its provider URL cannot be brokered.
    Unbindable,
    /// The broker has no key for the provider, or cannot read the store.
    NoKey,
    /// The broker is not running, is gone, or does not run here.
    BrokerUnavailable,
    /// The broker refused to register the credential.
    Rejected,
    /// The request could not be signed for the broker.
    #[cfg_attr(
        not(any(unix, windows)),
        allow(dead_code, reason = "only the broker signs requests")
    )]
    Unsigned,
    /// The broker denied the signed request.
    Denied,
}

impl From<&BrokerModelAuthError> for Refusal {
    fn from(error: &BrokerModelAuthError) -> Self {
        match error {
            BrokerModelAuthError::Unbindable { .. } => Self::Unbindable,
            BrokerModelAuthError::MissingKey { .. }
            | BrokerModelAuthError::MissingAccountKey { .. }
            | BrokerModelAuthError::Store(_) => Self::NoKey,
            BrokerModelAuthError::Unavailable
            | BrokerModelAuthError::NotStarted { .. }
            | BrokerModelAuthError::Unsupported
            | BrokerModelAuthError::PipeSquatted => Self::BrokerUnavailable,
            BrokerModelAuthError::Rejected => Self::Rejected,
        }
    }
}

/// Whose request it was, for the inspector's text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Scope {
    Session,
    Process,
}

macro_rules! refusal_texts {
    ($($kind:ident => $why:literal,)*) => {
        impl Refusal {
            /// Fixed, secret-free text for the inspector.
            fn text(self, scope: Scope) -> &'static str {
                match (self, scope) {
                    $(
                        (Self::$kind, Scope::Session) => concat!(
                            "the last model request of this session or one of its agents was refused: ",
                            $why,
                        ),
                        (Self::$kind, Scope::Process) => concat!(
                            "the last brokered request outside a session (the model list or the wallet) was refused: ",
                            $why,
                        ),
                    )*
                }
            }
        }
    };
}

refusal_texts! {
    Unbindable => "its provider URL cannot be brokered (it needs HTTPS with a DNS name or IPv4 address); nothing was sent",
    NoKey => "the broker has no key for its provider or cannot read the stored key; nothing was sent",
    BrokerUnavailable => "the credential broker is not available; nothing was sent",
    Rejected => "the credential broker rejected its credential; nothing was sent",
    Unsigned => "it could not be signed for the credential broker; nothing was sent",
    Denied => "the credential broker denied it; it did not reach the provider",
}

/// Frames awaiting the broker's answer, at most this many.
const MAX_PENDING: usize = 64;
/// Owners with a refusal on record, at most this many (the oldest refusal
/// goes first): sessions end, the process may not.
const MAX_REFUSED: usize = 256;

/// One broker's refusals, by owner (`None`: outside any session).
#[derive(Default)]
pub(crate) struct Refusals {
    state: Mutex<State>,
}

#[derive(Default)]
struct State {
    /// Orders refusals and signed requests: an answer to a request signed
    /// before a refusal does not clear it.
    clock: u64,
    refused: HashMap<Option<ThreadId>, (Refusal, u64)>,
    /// The owner of each signed frame whose answer has not arrived yet, and
    /// when it was signed. A served request removes the owner's older ones,
    /// so a late answer to them cannot bring a refusal back.
    pending: VecDeque<(HeaderValue, Option<ThreadId>, u64)>,
}

impl State {
    fn tick(&mut self) -> u64 {
        self.clock += 1;
        self.clock
    }

    /// Records `refusal` at `at` unless a later refusal is recorded.
    fn refuse(&mut self, owner: Option<ThreadId>, refusal: Refusal, at: u64) {
        if self
            .refused
            .get(&owner)
            .is_some_and(|(_, refused)| *refused > at)
        {
            return;
        }
        if !self.refused.contains_key(&owner)
            && self.refused.len() >= MAX_REFUSED
            && let Some(oldest) = self
                .refused
                .iter()
                .min_by_key(|(_, (_, refused))| *refused)
                .map(|(oldest, _)| *oldest)
        {
            self.refused.remove(&oldest);
        }
        self.refused.insert(owner, (refusal, at));
    }

    /// The request `owner` signed at `at` was served: refusals before it are
    /// over, and so are its requests signed before it (their answers are
    /// ignored).
    fn serve(&mut self, owner: Option<ThreadId>, at: u64) {
        if self
            .refused
            .get(&owner)
            .is_some_and(|(_, refused)| *refused < at)
        {
            self.refused.remove(&owner);
        }
        self.pending
            .retain(|(_, pending_owner, signed)| *pending_owner != owner || *signed > at);
    }
}

impl Refusals {
    fn state(&self) -> std::sync::MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// `owner`'s request was refused.
    pub(crate) fn refused(&self, owner: Option<ThreadId>, refusal: Refusal) {
        let mut state = self.state();
        let at = state.tick();
        state.refuse(owner, refusal, at);
    }

    /// `owner` sends a request signed with `frame`; the broker's answer to it
    /// decides.
    pub(crate) fn signed(&self, owner: Option<ThreadId>, frame: HeaderValue) {
        let mut state = self.state();
        let at = state.tick();
        if state.pending.len() == MAX_PENDING {
            state.pending.pop_front();
        }
        state.pending.push_back((frame, owner, at));
    }

    /// What became of the request signed with `frame`. A frame this broker
    /// did not sign (or no longer remembers) is ignored.
    pub(crate) fn answered(
        &self,
        frame: &HeaderValue,
        outcome: codex_http_client::ModelBrokerOutcome<'_>,
    ) {
        use codex_http_client::ModelBrokerOutcome;
        let mut state = self.state();
        let Some(index) = state
            .pending
            .iter()
            .position(|(signed, _, _)| signed == frame)
        else {
            return;
        };
        let Some((_, owner, at)) = state.pending.remove(index) else {
            return;
        };
        match outcome {
            ModelBrokerOutcome::Served => state.serve(owner, at),
            ModelBrokerOutcome::Denied(_) => state.refuse(owner, Refusal::Denied, at),
            ModelBrokerOutcome::NotRunning => {
                state.refuse(owner, Refusal::BrokerUnavailable, at);
            }
        }
    }

    /// What the inspector of a session whose agents are `threads` shows:
    /// their refusal first, else one outside any session.
    pub(crate) fn reason(&self, threads: &[ThreadId]) -> Option<&'static str> {
        let state = self.state();
        threads
            .iter()
            .find_map(|thread| state.refused.get(&Some(*thread)))
            .map(|(refusal, _)| refusal.text(Scope::Session))
            .or_else(|| {
                state
                    .refused
                    .get(&None)
                    .map(|(refusal, _)| refusal.text(Scope::Process))
            })
    }
}

#[cfg(test)]
#[path = "model_broker_refusals_tests.rs"]
mod tests;
