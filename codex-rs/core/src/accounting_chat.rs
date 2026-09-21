//! Sampling-local lazy bootstrap shared by admitted Chat transport.
use super::FAILURE;
use super::Sampling;
use crate::config::AccountingMode;
use crate::session::session::Session;
use codex_login::CodexAuth;
use codex_model_provider_info::ModelProviderInfo;
use codex_model_provider_info::WireApi;
use codex_protocol::error::CodexErr;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use tokio::sync::OnceCell;
use uuid::Uuid;

pub(crate) type Slot = Arc<Mutex<Option<Arc<DeferredChatSampling>>>>;

pub(crate) struct DeferredChatSampling {
    request: Uuid,
    session: Arc<Session>,
    turn: String,
    mode: AccountingMode,
    sampling: OnceCell<Arc<Sampling>>,
    failed: AtomicBool,
}

pub(crate) fn read(slot: &Slot) -> Result<Option<Arc<DeferredChatSampling>>, CodexErr> {
    slot.lock().map(|value| value.clone()).map_err(|poison| {
        if let Some(value) = poison.into_inner().take() {
            value.reject();
        }
        CodexErr::Fatal(FAILURE.into())
    })
}

pub(crate) struct Scope(Slot);
impl Scope {
    pub(crate) fn attach(
        slot: Slot,
        value: Option<Arc<DeferredChatSampling>>,
    ) -> Result<Self, CodexErr> {
        *slot.lock().map_err(|_| CodexErr::Fatal(FAILURE.into()))? = value;
        Ok(Self(slot))
    }
}
impl Drop for Scope {
    fn drop(&mut self) {
        match self.0.lock() {
            Ok(mut value) => *value = None,
            Err(poison) => {
                if let Some(value) = poison.into_inner().take() {
                    value.reject();
                }
            }
        }
    }
}

struct Bootstrap<'a>(&'a DeferredChatSampling, bool);
impl Drop for Bootstrap<'_> {
    fn drop(&mut self) {
        if !self.1 {
            self.0.reject();
        }
    }
}

impl DeferredChatSampling {
    pub(crate) fn new(session: Arc<Session>, turn: String, mode: AccountingMode) -> Arc<Self> {
        Arc::new(Self {
            request: Uuid::new_v4(),
            session,
            turn,
            mode,
            sampling: OnceCell::new(),
            failed: AtomicBool::new(false),
        })
    }

    pub(crate) fn reject(&self) {
        self.failed.store(true, Ordering::Release);
        if let Some(sampling) = self.sampling.get() {
            sampling.reject();
        }
    }

    pub(crate) fn check(&self) -> Result<(), CodexErr> {
        if self.failed.load(Ordering::Acquire) {
            return Err(CodexErr::Fatal(FAILURE.into()));
        }
        self.sampling.get().map_or(Ok(()), |value| value.check())
    }

    pub(crate) fn exclude(&self) -> Result<(), CodexErr> {
        if self.sampling.initialized() {
            self.reject();
        }
        self.check()
    }

    pub(crate) async fn resolve(
        &self,
        provider: &ModelProviderInfo,
        auth: Option<&CodexAuth>,
        endpoint: &str,
        request: &codex_api::ChatCompletionsRequest,
    ) -> Result<Option<Arc<Sampling>>, CodexErr> {
        self.check()?;
        let admitted = if matches!(self.mode, AccountingMode::Provider { .. }) {
            eligible(provider, auth, request)
        } else {
            legacy_eligible(provider, auth, request)
        };
        if !admitted {
            if self.sampling.initialized() {
                self.reject();
                self.check()?;
            }
            return Ok(None);
        }
        let (AccountingMode::DirectOpenAiChat {
            approved_endpoint, ..
        }
        | AccountingMode::Provider {
            approved_endpoint,
            wire_api: WireApi::Chat,
            ..
        }) = &self.mode
        else {
            self.reject();
            return Err(CodexErr::Fatal(FAILURE.into()));
        };
        let pinned = super::pinned_route(
            approved_endpoint,
            super::canonical_query(provider).as_deref(),
            "chat/completions",
        );
        if super::canonical_route(endpoint) != super::canonical_route(&pinned) {
            self.reject();
            return Err(CodexErr::Fatal(FAILURE.into()));
        }
        let mode = if let AccountingMode::Provider { provider_id, .. } = &self.mode {
            super::turn_mode(
                &self.mode,
                provider_id,
                provider,
                auth.map(CodexAuth::auth_mode),
                approved_endpoint,
            )
        } else {
            self.mode.clone()
        };
        // A rebound mode that disagrees with the sampling already in flight would
        // record this turn's economics under the wrong authority.
        if let Some(existing) = self.sampling.get()
            && matches!(mode, AccountingMode::Provider { .. })
            && super::pricing_for(&mode) != existing.pricing
        {
            self.reject();
            return Err(CodexErr::Fatal(FAILURE.into()));
        }
        let result = self
            .sampling
            .get_or_try_init(|| async {
                let mut completion = Bootstrap(self, false);
                self.session
                    .try_ensure_rollout_materialized()
                    .await
                    .map_err(|_| CodexErr::Fatal(FAILURE.into()))?;
                let runtime = self
                    .session
                    .state_db()
                    .ok_or_else(|| CodexErr::Fatal(FAILURE.into()))?;
                let sampling = Sampling::start_request(
                    runtime,
                    self.session.thread_id,
                    self.turn.clone(),
                    &mode,
                    self.request,
                )
                .await?;
                completion.1 = true;
                Ok::<_, CodexErr>(sampling)
            })
            .await;
        match result {
            Ok(sampling) => {
                self.check()?;
                Ok(Some(Arc::clone(sampling)))
            }
            Err(_) => {
                self.reject();
                Err(CodexErr::Fatal(FAILURE.into()))
            }
        }
    }
}

pub(super) fn eligible(
    provider: &ModelProviderInfo,
    _auth: Option<&CodexAuth>,
    request: &codex_api::ChatCompletionsRequest,
) -> bool {
    provider.wire_api == WireApi::Chat
        && super::route_refusal(provider).is_none()
        // A configured routing preference is part of who serves and bills this
        // request, so it stays attributable to the selected provider. A
        // request-level preference configuration did not ask for is not.
        && request.provider == provider.chat_completions_provider
        // The other two routing fields are emitted by this client from provider
        // configuration as well: `providerOptions` when the selected provider is
        // the Vercel gateway, `plugins` when it is OpenRouter and the session has
        // web search. Requiring them to be absent excluded those sessions from
        // collection entirely. Their exact values are still checked at the
        // transport against what the client constructed.
        && (request.provider_options.is_none() || provider.is_vercel_gateway())
        && (request.plugins.is_none() || provider.is_openrouter())
}

pub(super) fn legacy_eligible(
    provider: &ModelProviderInfo,
    auth: Option<&CodexAuth>,
    request: &codex_api::ChatCompletionsRequest,
) -> bool {
    provider.wire_api == WireApi::Chat
        && provider.is_openai()
        && provider.requires_openai_auth
        && !provider.is_pfterminal_plan()
        && provider.chat_completions_provider.is_none()
        && request.provider.is_none()
        && request.provider_options.is_none()
        && request.plugins.is_none()
        && matches!(auth, Some(CodexAuth::ApiKey(_)))
        && provider.aws.is_none()
        && provider.auth.is_none()
        && provider.experimental_bearer_token.is_none()
        && provider.api_key_header_name().is_none()
        && !provider.http_headers.as_ref().is_some_and(|headers| {
            headers.keys().any(|name| {
                name.eq_ignore_ascii_case("authorization") || name.eq_ignore_ascii_case("api-key")
            })
        })
        && !provider.env_http_headers.as_ref().is_some_and(|headers| {
            headers.keys().any(|name| {
                name.eq_ignore_ascii_case("authorization") || name.eq_ignore_ascii_case("api-key")
            })
        })
}

pub(super) fn patch(
    usage: codex_api::ChatUsagePatch,
) -> anyhow::Result<codex_state::accounting::Patch> {
    use codex_api::ChatTokenPresence;
    use codex_state::accounting::Presence;
    let presence = |value| {
        Ok::<_, anyhow::Error>(match value {
            ChatTokenPresence::Missing => Presence::Missing,
            ChatTokenPresence::Null => Presence::Null,
            ChatTokenPresence::Number(value) => Presence::Number(value.try_into()?),
        })
    };
    Ok(codex_state::accounting::Patch {
        input: presence(usage.input_tokens)?,
        read: presence(usage.cached_tokens)?,
        write: Presence::Missing,
        output: presence(usage.output_tokens)?,
        reasoning: presence(usage.reasoning_tokens)?,
        total: presence(usage.total_tokens)?,
    })
}

#[cfg(test)]
#[path = "accounting_chat_tests.rs"]
mod tests;
