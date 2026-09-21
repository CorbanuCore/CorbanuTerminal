//! Native sampling ownership glue. Replay, exact money and retention stay in state.

// Reject optimized release builds; the package entrypoint also rejects marked
// developer binaries, including debug profiles such as dev-small.
#[cfg(all(feature = "developer-accounting", not(debug_assertions)))]
compile_error!("developer-accounting is debug-only and must not be enabled in distribution builds");

use crate::config::AccountingMode;
use codex_api::AnthropicTokenPresence;
use codex_api::AnthropicUsagePatch;
use codex_protocol::ThreadId;
use codex_protocol::error::CodexErr;
use codex_state::StateRuntime;
use codex_state::accounting::AccountingStore;
use codex_state::accounting::Attempt;
use codex_state::accounting::Dialect;
use codex_state::accounting::Observation;
use codex_state::accounting::Patch;
use codex_state::accounting::Presence;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use uuid::Uuid;

#[path = "accounting_chat.rs"]
pub(crate) mod chat;
#[cfg(test)]
#[path = "accounting_policy_tests.rs"]
mod policy_tests;
#[path = "accounting_prices.rs"]
mod prices;
#[path = "accounting_responses.rs"]
pub(crate) mod responses;
#[path = "accounting_transport.rs"]
pub(crate) mod transport;
#[path = "accounting_websocket.rs"]
pub(crate) mod websocket;

/// Build-time developer opt-in only; normal builds do not contain this selector.
/// Existing collectors still enforce endpoint and authentication eligibility.
#[cfg(feature = "developer-accounting")]
pub(crate) fn developer_accounting_mode(
    provider_id: &str,
    provider: &codex_model_provider_info::ModelProviderInfo,
) -> AccountingMode {
    use codex_model_provider_info::WireApi;
    // Retain a byte marker in executable data even when symbols are stripped.
    // scripts/build_codex_package.py refuses any input carrying this marker.
    std::hint::black_box(b"CORBANU_DEVELOPER_ACCOUNTING_NOT_FOR_DISTRIBUTION");
    let scope = Uuid::new_v4();
    let approved_endpoint = provider.base_url.clone().unwrap_or_else(|| {
        match provider.wire_api {
            WireApi::Anthropic => codex_model_provider_info::ANTHROPIC_BASE_URL,
            WireApi::Responses | WireApi::Chat => "https://api.openai.com/v1",
        }
        .into()
    });
    if route_refusal(provider).is_some() {
        return AccountingMode::Disabled;
    }
    AccountingMode::Provider {
        scope,
        provider_id: provider_id.into(),
        wire_api: provider.wire_api,
        approved_endpoint,
        api_key_pricing: false,
    }
}

/// Shared admission policy. Authentication changes economics, not token attribution.
pub(super) fn route_refusal(
    provider: &codex_model_provider_info::ModelProviderInfo,
) -> Option<&'static str> {
    if provider.aws.is_some() {
        Some("AWS signing does not establish the supported endpoint/dialect")
    } else if provider.chat_completions_provider.is_some() {
        Some("chat_completions_provider overrides the serving provider")
    } else if provider.query_params.is_some() {
        // The resolved request URL carries the query string, so it can never equal
        // the pinned `{base}/path`. Refusing the shape leaves such a provider
        // uncollected; admitting it made every turn fail closed instead.
        Some("query parameters are not part of the pinned endpoint")
    } else {
        None
    }
}

/// Bind developer collection to this turn, including provider switches and the
/// authentication-dependent default endpoint. Never read credentials here.
pub(crate) fn turn_mode(
    mode: &AccountingMode,
    provider_id: &str,
    provider: &codex_model_provider_info::ModelProviderInfo,
    auth_mode: Option<codex_protocol::auth::AuthMode>,
    resolved_endpoint: &str,
) -> AccountingMode {
    let AccountingMode::Provider { scope, .. } = mode else {
        return mode.clone();
    };
    if route_refusal(provider).is_some() {
        return AccountingMode::Disabled;
    }
    // Only the existing native API-key authority can supply monetary rates.
    let api_key_pricing = auth_mode == Some(codex_protocol::auth::AuthMode::ApiKey)
        && provider.auth.is_none()
        && provider.experimental_bearer_token.is_none()
        && provider.http_headers.is_none()
        && provider.env_http_headers.is_none()
        && match (provider_id, provider.wire_api) {
            ("anthropic", codex_model_provider_info::WireApi::Anthropic) => {
                resolved_endpoint == codex_model_provider_info::ANTHROPIC_BASE_URL
            }
            (
                "openai",
                codex_model_provider_info::WireApi::Responses
                | codex_model_provider_info::WireApi::Chat,
            ) => resolved_endpoint == "https://api.openai.com/v1",
            _ => false,
        };
    AccountingMode::Provider {
        scope: *scope,
        provider_id: provider_id.into(),
        wire_api: provider.wire_api,
        approved_endpoint: resolved_endpoint.into(),
        api_key_pricing,
    }
}

pub(crate) fn collects(
    mode: &AccountingMode,
    provider_id: &str,
    provider: &codex_model_provider_info::ModelProviderInfo,
    wire: codex_model_provider_info::WireApi,
) -> bool {
    use codex_model_provider_info::WireApi;
    if provider.wire_api != wire || route_refusal(provider).is_some() {
        return false;
    }
    match mode {
        AccountingMode::Provider {
            provider_id: bound,
            wire_api,
            ..
        } => bound == provider_id && *wire_api == wire,
        AccountingMode::DirectAnthropic { .. } => {
            provider_id == "anthropic" && wire == WireApi::Anthropic
        }
        AccountingMode::DirectOpenAiChat { .. } => provider_id == "openai" && wire == WireApi::Chat,
        AccountingMode::DirectOpenAiResponses { .. }
        | AccountingMode::DirectOpenAiResponsesHttp { .. } => {
            provider_id == "openai" && wire == WireApi::Responses
        }
        AccountingMode::Disabled => false,
    }
}

pub(crate) const FAILURE: &str =
    "Native Anthropic accounting failed; request stopped without a repair send";
pub(crate) type Slot = Arc<Mutex<Option<Arc<Sampling>>>>;

// The facade accepts explicit as-of values. Serialize this process's collector
// transactions before sampling time so concurrent native children cannot commit
// a later checkpoint ahead of an older local operation. Never hold across HTTP;
// external writers still use the store's fail-visible validation contract.
static WRITES: tokio::sync::Semaphore = tokio::sync::Semaphore::const_new(1);

pub(crate) fn read_slot(slot: &Slot) -> Result<Option<Arc<Sampling>>, CodexErr> {
    match slot.lock() {
        Ok(value) => Ok(value.clone()),
        Err(poison) => {
            if let Some(stale) = poison.into_inner().take() {
                stale.reject();
            }
            Err(CodexErr::Fatal(FAILURE.into()))
        }
    }
}

pub(crate) struct SamplingScope(Slot);

impl SamplingScope {
    pub(crate) fn attach(slot: Slot, sampling: Option<Arc<Sampling>>) -> Result<Self, CodexErr> {
        {
            match slot.lock() {
                Ok(mut value) => *value = sampling,
                Err(poison) => {
                    if let Some(stale) = poison.into_inner().take() {
                        stale.reject();
                    }
                    if let Some(incoming) = sampling {
                        incoming.reject();
                    }
                    return Err(CodexErr::Fatal(FAILURE.into()));
                }
            }
        }
        Ok(Self(slot))
    }
}

impl Drop for SamplingScope {
    fn drop(&mut self) {
        match self.0.lock() {
            Ok(mut value) => *value = None,
            Err(poison) => {
                if let Some(stale) = poison.into_inner().take() {
                    stale.reject();
                }
            }
        }
    }
}

enum Pricing {
    Anthropic,
    Responses,
    Chat,
    Unavailable,
}

pub(crate) struct Sampling {
    runtime: Arc<StateRuntime>,
    owner: ThreadId,
    turn: String,
    request: Uuid,
    scope: Uuid,
    endpoint: String,
    provider: String,
    dialect: Dialect,
    pricing: Pricing,
    previous: Mutex<Option<Uuid>>,
    failed: AtomicBool,
}

// A cancelled SQL future may already have committed. Never reuse that sampling
// with a guessed predecessor; release the permit normally and fail it closed.
struct Completion<'a> {
    sampling: &'a Sampling,
    complete: bool,
}

impl Drop for Completion<'_> {
    fn drop(&mut self) {
        if !self.complete {
            self.sampling.reject();
        }
    }
}

impl Sampling {
    pub(crate) async fn start(
        runtime: Arc<StateRuntime>,
        owner: ThreadId,
        turn: String,
        mode: &AccountingMode,
    ) -> Result<Arc<Self>, CodexErr> {
        Self::start_request(runtime, owner, turn, mode, Uuid::new_v4()).await
    }

    async fn start_request(
        runtime: Arc<StateRuntime>,
        owner: ThreadId,
        turn: String,
        mode: &AccountingMode,
        request: Uuid,
    ) -> Result<Arc<Self>, CodexErr> {
        let (scope, approved_endpoint, provider, dialect, path) = match mode {
            AccountingMode::Provider {
                scope,
                provider_id,
                wire_api,
                approved_endpoint,
                ..
            } => {
                use codex_model_provider_info::WireApi;
                let (dialect, path) = match wire_api {
                    WireApi::Anthropic => (Dialect::NativeAnthropic, "messages"),
                    WireApi::Responses => (Dialect::Inclusive, "responses"),
                    WireApi::Chat => (Dialect::Inclusive, "chat/completions"),
                };
                (
                    scope,
                    approved_endpoint,
                    provider_id.as_str(),
                    dialect,
                    path,
                )
            }
            AccountingMode::DirectAnthropic {
                scope,
                approved_endpoint,
            } => (
                scope,
                approved_endpoint,
                "anthropic",
                Dialect::NativeAnthropic,
                "messages",
            ),
            AccountingMode::DirectOpenAiResponsesHttp {
                scope,
                approved_endpoint,
            }
            | AccountingMode::DirectOpenAiResponses {
                scope,
                approved_endpoint,
            } => (
                scope,
                approved_endpoint,
                "openai",
                Dialect::Inclusive,
                "responses",
            ),
            AccountingMode::DirectOpenAiChat {
                scope,
                approved_endpoint,
            } => (
                scope,
                approved_endpoint,
                "openai",
                Dialect::Inclusive,
                "chat/completions",
            ),
            AccountingMode::Disabled => return Err(CodexErr::Fatal(FAILURE.into())),
        };
        let endpoint =
            url::Url::parse(approved_endpoint).map_err(|_| CodexErr::Fatal(FAILURE.into()))?;
        if !endpoint.username().is_empty()
            || endpoint.password().is_some()
            || endpoint.query().is_some()
            || endpoint.fragment().is_some()
            || !matches!(endpoint.scheme(), "http" | "https")
        {
            return Err(CodexErr::Fatal(FAILURE.into()));
        }
        let _write = WRITES
            .acquire()
            .await
            .map_err(|_| CodexErr::Fatal(FAILURE.into()))?;
        AccountingStore::open(&runtime, now())
            .await
            .map_err(|_| CodexErr::Fatal(FAILURE.into()))?;
        Ok(Arc::new(Self {
            runtime,
            owner,
            turn,
            request,
            scope: *scope,
            endpoint: format!("{}/{path}", approved_endpoint.trim_end_matches('/')),
            provider: provider.into(),
            dialect,
            pricing: match mode {
                AccountingMode::Provider {
                    wire_api,
                    api_key_pricing: true,
                    ..
                } => match wire_api {
                    codex_model_provider_info::WireApi::Anthropic => Pricing::Anthropic,
                    codex_model_provider_info::WireApi::Responses => Pricing::Responses,
                    codex_model_provider_info::WireApi::Chat => Pricing::Chat,
                },
                AccountingMode::Provider {
                    api_key_pricing: false,
                    ..
                } => Pricing::Unavailable,
                AccountingMode::DirectAnthropic { .. } => Pricing::Anthropic,
                AccountingMode::DirectOpenAiChat { .. } => Pricing::Chat,
                AccountingMode::DirectOpenAiResponsesHttp { .. }
                | AccountingMode::DirectOpenAiResponses { .. } => Pricing::Responses,
                AccountingMode::Disabled => unreachable!(),
            },
            previous: Mutex::new(None),
            failed: AtomicBool::new(false),
        }))
    }

    pub(crate) fn check(&self) -> Result<(), CodexErr> {
        if self.failed.load(Ordering::Acquire) {
            Err(CodexErr::Fatal(FAILURE.into()))
        } else {
            Ok(())
        }
    }

    pub(crate) fn reject(&self) {
        self.failed.store(true, Ordering::Release);
    }

    async fn admit(&self, model: &str, endpoint: &str) -> anyhow::Result<Attempt> {
        self.admit_with_tier(model, endpoint, None).await
    }

    async fn admit_with_tier(
        &self,
        model: &str,
        endpoint: &str,
        tier: Option<&str>,
    ) -> anyhow::Result<Attempt> {
        self.check()?;
        anyhow::ensure!(endpoint == self.endpoint, "accounting route mismatch");
        let _write = WRITES.acquire().await?;
        self.check()?;
        let mut completion = Completion {
            sampling: self,
            complete: false,
        };
        let previous = *self.previous.lock().map_err(|_| {
            self.reject();
            anyhow::anyhow!(FAILURE)
        })?;
        // The facade borrows its runtime. Reopening validates/maintains through
        // its public contract; never fabricate an attached or Active handle.
        let store = AccountingStore::open(&self.runtime, now()).await?;
        let dispatched_at = now();
        let attempt = Attempt {
            attempt_id: Uuid::new_v4(),
            request_id: self.request,
            thread_id: self.owner,
            turn: self.turn.clone(),
            retry_of: previous,
            provider: self.provider.clone(),
            model: model.into(),
            scope: self.scope,
            dialect: self.dialect,
            dispatched_at_ms: dispatched_at.try_into()?,
        };
        let prices = match self.pricing {
            Pricing::Unavailable => Vec::new(),
            Pricing::Anthropic => prices::original(model, self.scope, dispatched_at)?,
            Pricing::Responses => {
                prices::responses_original(model, self.scope, dispatched_at, tier)?
            }
            Pricing::Chat => prices::chat_original(model, self.scope, dispatched_at)?,
        };
        store.admit(self.owner, &attempt, &prices, now()).await?;
        *self.previous.lock().map_err(|_| {
            self.reject();
            anyhow::anyhow!(FAILURE)
        })? = Some(attempt.attempt_id);
        completion.complete = true;
        Ok(attempt)
    }

    async fn observe(
        &self,
        attempt: &Attempt,
        source: Uuid,
        position: i64,
        usage: AnthropicUsagePatch,
    ) -> anyhow::Result<()> {
        self.check()?;
        let patch = Patch {
            input: presence(usage.input_tokens)?,
            read: presence(usage.cache_read_input_tokens)?,
            write: presence(usage.cache_creation_input_tokens)?,
            output: presence(usage.output_tokens)?,
            ..Patch::default()
        };
        self.observe_patch(attempt, source, position, patch).await
    }

    async fn observe_patch(
        &self,
        attempt: &Attempt,
        source: Uuid,
        position: i64,
        patch: Patch,
    ) -> anyhow::Result<()> {
        let position = position.try_into()?;
        let observation = Observation {
            source,
            revision: position,
            sequence: position,
            patch,
        };
        let _write = WRITES.acquire().await?;
        self.check()?;
        let mut completion = Completion {
            sampling: self,
            complete: false,
        };
        let store = AccountingStore::open(&self.runtime, now()).await?;
        store
            .observe(self.owner, attempt, &[observation], now())
            .await?;
        completion.complete = true;
        Ok(())
    }
}

fn presence(value: AnthropicTokenPresence) -> anyhow::Result<Presence> {
    Ok(match value {
        AnthropicTokenPresence::Missing => Presence::Missing,
        AnthropicTokenPresence::Null => Presence::Null,
        AnthropicTokenPresence::Number(value) => Presence::Number(value.try_into()?),
    })
}

fn now() -> i64 {
    chrono::Utc::now().timestamp_millis()
}
