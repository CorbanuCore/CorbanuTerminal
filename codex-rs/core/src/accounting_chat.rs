//! Sampling-local lazy bootstrap shared by admitted Chat transport.
use super::FAILURE;
use super::Sampling;
use super::failure;
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
    /// The thread its attempts are recorded under.
    owner: codex_protocol::ThreadId,
    /// The ledger that thread lives in; the session's own when `None`.
    owner_db: Option<codex_rollout::state_db::StateDbHandle>,
    turn: String,
    mode: AccountingMode,
    sampling: OnceCell<Arc<Sampling>>,
    failed: AtomicBool,
}

/// The sampling attached to `slot`. A poisoned slot is closed and the request
/// goes unrecorded.
pub(crate) fn read(slot: &Slot) -> Result<Option<Arc<DeferredChatSampling>>, CodexErr> {
    Ok(slot
        .lock()
        .map(|value| value.clone())
        .unwrap_or_else(|poison| {
            if let Some(value) = poison.into_inner().take() {
                value.reject();
            }
            super::gap("read sampling slot", "slot lock poisoned");
            None
        }))
}

pub(crate) struct Scope(Slot);
impl Scope {
    pub(crate) fn attach(
        slot: Slot,
        value: Option<Arc<DeferredChatSampling>>,
    ) -> Result<Self, CodexErr> {
        match slot.lock() {
            Ok(mut slot) => *slot = value,
            Err(poison) => {
                // Leave the slot empty: this turn's requests go unrecorded.
                if let Some(stale) = poison.into_inner().take() {
                    stale.reject();
                }
                if let Some(value) = value {
                    value.reject();
                }
                super::gap("attach sampling", "slot lock poisoned");
            }
        }
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
    #[cfg(test)]
    pub(crate) fn new(session: Arc<Session>, turn: String, mode: AccountingMode) -> Arc<Self> {
        Arc::new(Self {
            request: Uuid::new_v4(),
            owner: session.thread_id,
            owner_db: None,
            session,
            turn,
            mode,
            sampling: OnceCell::new(),
            failed: AtomicBool::new(false),
        })
    }

    /// A sampling recorded under `owner`: the session's own thread, or the
    /// conversation an ephemeral review fork works for.
    pub(crate) fn new_for(
        session: Arc<Session>,
        owner: crate::accounting::AccountingOwner,
        turn: String,
        mode: AccountingMode,
    ) -> Arc<Self> {
        Arc::new(Self {
            request: Uuid::new_v4(),
            session,
            owner: owner.thread,
            owner_db: Some(owner.db),
            turn,
            mode,
            sampling: OnceCell::new(),
            failed: AtomicBool::new(false),
        })
    }

    #[track_caller]
    pub(crate) fn reject(&self) {
        if !self.failed.swap(true, Ordering::AcqRel) {
            tracing::warn!(
                target: "codex_core::accounting",
                at = %std::panic::Location::caller(),
                "accounting sampling closed; later requests in this turn are sent unrecorded"
            );
        }
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

    /// Whether this sampling has stopped recording.
    pub(crate) fn is_closed(&self) -> bool {
        self.check().is_err()
    }

    /// Whether this sampling stopped its turn (`Sampling::halt`).
    pub(crate) fn is_halted(&self) -> bool {
        self.sampling.get().is_some_and(|value| value.is_halted())
    }

    /// The sampling this request is recorded on, if any. Accounting never
    /// stops a request: when it fails, this sampling closes and the request,
    /// like every later one in the turn, is sent unrecorded.
    pub(crate) async fn resolve(
        &self,
        provider: &ModelProviderInfo,
        auth: Option<&CodexAuth>,
        endpoint: &str,
        request: &codex_api::ChatCompletionsRequest,
    ) -> Result<Option<Arc<Sampling>>, CodexErr> {
        if self.is_closed() {
            return Ok(None);
        }
        match self.try_resolve(provider, auth, endpoint, request).await {
            Ok(sampling) => Ok(sampling),
            Err(error) => {
                self.reject();
                super::gap("resolve sampling", error);
                Ok(None)
            }
        }
    }

    async fn try_resolve(
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
            return Err(CodexErr::Fatal(
                failure("resolve sampling", "turn mode does not match this wire").into(),
            ));
        };
        let pinned = super::pinned_route(
            approved_endpoint,
            super::canonical_query(provider).as_deref(),
            "chat/completions",
        );
        if super::canonical_route(endpoint) != super::canonical_route(&pinned) {
            self.reject();
            return Err(CodexErr::Fatal(
                failure(
                    "resolve sampling",
                    "request route differs from the approved route",
                )
                .into(),
            ));
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
            && (super::pricing_for(&mode) != existing.pricing
                || super::basis_source_for(&mode) != existing.basis_source
                || !existing.same_account(&mode))
        {
            self.reject();
            return Err(CodexErr::Fatal(
                failure("resolve sampling", "turn economics changed mid-sampling").into(),
            ));
        }
        let result = self
            .sampling
            .get_or_try_init(|| async {
                let mut completion = Bootstrap(self, false);
                self.session
                    .try_ensure_rollout_materialized()
                    .await
                    .map_err(|error| {
                        CodexErr::Fatal(failure("materialize rollout", error).into())
                    })?;
                let runtime = self
                    .owner_db
                    .clone()
                    .or_else(|| self.session.state_db())
                    .ok_or_else(|| {
                        CodexErr::Fatal(failure("open sampling", "no state database").into())
                    })?;
                let sampling = Sampling::start_request(
                    runtime,
                    self.owner,
                    self.turn.clone(),
                    &mode,
                    self.request,
                    /*path_override*/ None,
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
            Err(error) => {
                // `start_request` logged its own cause; this records the outcome.
                tracing::debug!(target: "codex_core::accounting", %error, "sampling did not open");
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

/// The ledger patch for one Chat usage report from `provider_id`.
///
/// OpenRouter returns `cache_write_tokens` "only for models with explicit
/// caching and cache write pricing" (openrouter.ai/docs, usage accounting), so
/// on that route an absent count is a model with no cache-write charge: its
/// prompt is exactly cache hits plus misses. Recording it as zero lets the
/// ledger derive the uncached input, which it otherwise leaves unknown.
///
/// OpenAI's own Chat Completions API reports `cache_write_tokens` as a subset of
/// `prompt_tokens` (a real `gpt-5.6-luna` response on 2026-10-09: 6,017 prompt,
/// 6,014 written, then 6,014 cached; #361), so it is kept on the `openai` route.
/// That route is identified by provider ID, as its prices are, so an `openai`
/// provider pointed at another base URL gets the same reading; a report whose
/// writes fall outside the prompt fails replay rather than being mispriced.
///
/// Moonshot's Kimi Open Platform documents `cache_write_tokens` as the part of
/// `prompt_tokens` written to the cache, apart from the cached part
/// (platform.kimi.ai/docs/api/chat, usage fields), so it is kept on that route
/// (`route`), whatever the user named the provider that reached it. Its schema
/// marks both counts required, but the API leaves out a count that is zero:
/// real responses on 2026-10-15 (#368) had `kimi-k3` report 11,776 written and
/// no `cached_tokens`, and `kimi-k2.7-code` send no `prompt_tokens_details` at
/// all. So on that route, with `prompt_tokens` stated, a missing read or write
/// count is zero; a read missing from the details is taken from
/// `usage.cached_tokens` first, where Moonshot also states it. Without this
/// the uncached input could not be derived and `/cost` showed only a lower
/// bound (K2: the output alone).
///
/// Every other route leaves the cache-write count unknown, reported or not: the
/// field's meaning is only established for OpenRouter, OpenAI and Moonshot (a
/// subset of the prompt). A price sheet that states writes free bills written
/// tokens as ordinary input, and a gateway reporting writes outside the prompt
/// count would fail replay.
///
/// #786: a reasoning count above the output count it is part of is not a count
/// this client can state (OpenRouter's MiniMax M3 reported 37 reasoning tokens
/// of 36 output). It is recorded as not reported, and the rest of the report,
/// the charge included, is kept; replay would otherwise refuse the whole record.
pub(super) fn patch(
    usage: codex_api::ChatUsagePatch,
    provider_id: &str,
    route: Option<crate::config::PricedRoute>,
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
    let output = presence(usage.output_tokens)?;
    let mut reasoning = presence(usage.reasoning_tokens)?;
    if let (Presence::Number(reasoning_count), Presence::Number(output_count)) = (reasoning, output)
        && i64::from(reasoning_count) > i64::from(output_count)
    {
        tracing::debug!(
            target: "codex_core::accounting",
            "reasoning count above the output count; recorded as not reported"
        );
        reasoning = Presence::Missing;
    }
    let moonshot = route == Some(crate::config::PricedRoute::MoonshotOpenPlatform);
    // On the Moonshot route a part of a stated prompt that is not reported is
    // zero (see above).
    let moonshot_part = |value| match value {
        ChatTokenPresence::Missing
            if moonshot && matches!(usage.input_tokens, ChatTokenPresence::Number(_)) =>
        {
            ChatTokenPresence::Number(0)
        }
        other => other,
    };
    let read = match usage.cached_tokens {
        ChatTokenPresence::Missing if moonshot => moonshot_part(usage.usage_cached_tokens),
        other => other,
    };
    Ok(codex_state::accounting::Patch {
        input: presence(usage.input_tokens)?,
        read: presence(read)?,
        // The route decides before the provider's name does.
        write: match (provider_id, route) {
            (_, Some(crate::config::PricedRoute::MoonshotOpenPlatform)) => {
                presence(moonshot_part(usage.cache_write_tokens))?
            }
            (codex_model_provider_info::OPENROUTER_PROVIDER_ID, _) => {
                match usage.cache_write_tokens {
                    ChatTokenPresence::Missing => Presence::Number(0.try_into()?),
                    other => presence(other)?,
                }
            }
            (codex_model_provider_info::OPENAI_PROVIDER_ID, _) => {
                presence(usage.cache_write_tokens)?
            }
            _ => Presence::Missing,
        },
        // OpenRouter and the Corbanu API state what they charged with every
        // response. Other routes' `cost` fields have no established meaning
        // here. A figure the exact decimal type cannot hold is dropped rather
        // than rounded.
        billed_usd: matches!(
            provider_id,
            codex_model_provider_info::OPENROUTER_PROVIDER_ID
                | codex_model_provider_info::PFTERMINAL_PLAN_PROVIDER_ID
        )
        .then_some(usage.billed_usd)
        .flatten()
        .and_then(|text| codex_state::accounting::Decimal::try_from(text).ok()),
        output,
        reasoning,
        total: presence(usage.total_tokens)?,
        refused_status: None,
        session_ended_at_ms: None,
    })
}

#[cfg(test)]
#[path = "accounting_chat_tests.rs"]
mod tests;
