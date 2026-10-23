//! Realtime session ends on standalone WebSocket or existing WebRTC attempts.
//!
//! A WebSocket session is admitted when it opens; WebRTC retains the attempt
//! already admitted for HTTP creation. The end states only its timestamp. Its
//! tokens are never reported to this client - the realtime events it parses
//! carry none - so they stay unknown, never zero.
use super::Sampling;
use super::responses::DeferredResponsesSampling;
use codex_login::CodexAuth;
use codex_model_provider_info::ModelProviderInfo;
use codex_state::accounting::Attempt;
use codex_state::accounting::Patch;
use std::sync::Arc;
use uuid::Uuid;

pub(crate) struct RealtimeSessionRecord {
    sampling: Arc<Sampling>,
    attempt: Attempt,
    source: Uuid,
}

impl RealtimeSessionRecord {
    /// Admit the session that opened at `url` (`route` under the approved
    /// endpoint) on `deferred`, signed in with `auth`. `None` when it goes
    /// unrecorded: a route or credential the accounting does not approve, or
    /// a ledger failure, each logged as a gap. Never refuses the session.
    pub(crate) async fn begin(
        deferred: &DeferredResponsesSampling,
        provider: &ModelProviderInfo,
        auth: Option<&CodexAuth>,
        url: &str,
        route: &str,
        model: &str,
    ) -> Option<Self> {
        let sampling = deferred
            .resolve_path(provider, auth, url, route)
            .await
            .ok()
            .flatten()?;
        match sampling.admit_with_tier(model, url, /*tier*/ None).await {
            Ok(attempt) => Some(Self::for_attempt(sampling, attempt)),
            Err(error) => {
                sampling.reject();
                super::gap("admit realtime session", format_args!("{error:#}"));
                None
            }
        }
    }

    /// Observe a session on an already admitted HTTP call, without counting
    /// another request. A separate source keeps its end distinct from any
    /// observation made while creating that call.
    pub(super) fn for_attempt(sampling: Arc<Sampling>, attempt: Attempt) -> Self {
        Self {
            sampling,
            attempt,
            source: Uuid::new_v4(),
        }
    }

    /// Record that the session ended at `ended_at_ms` (Unix ms). Its tokens
    /// stay unknown. The write waits only while the state DB lock is held by
    /// another connection; a busy DB defers it and the close goes ahead.
    pub(crate) async fn end(self, ended_at_ms: i64) {
        let ended = match ended_at_ms.try_into() {
            Ok(ended) => ended,
            Err(error) => {
                super::gap("record realtime session end", error);
                return;
            }
        };
        let patch = Patch {
            session_ended_at_ms: Some(ended),
            ..Patch::default()
        };
        if let Err(error) = self
            .sampling
            .observe_patch(&self.attempt, self.source, /*position*/ 1, patch)
            .await
        {
            self.sampling.reject();
            super::gap("record realtime session end", format_args!("{error:#}"));
        }
    }
}
