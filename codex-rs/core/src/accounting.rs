//! Native sampling ownership glue. Replay, exact money and retention stay in state.

// Reject optimized release builds; the package entrypoint also rejects marked
// developer binaries, including debug profiles such as dev-small.
#[cfg(all(feature = "developer-accounting", not(debug_assertions)))]
compile_error!("developer-accounting is debug-only and must not be enabled in distribution builds");

use crate::config::AccountingMode;
use crate::config::PriceAuthority;
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
use sha2::Digest;
use sha2::Sha256;
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
        approved_query: canonical_query(provider),
        pricing: PriceAuthority::Unavailable,
    }
}

/// Shared admission policy. Authentication changes economics, not token attribution.
pub(super) fn route_refusal(
    provider: &codex_model_provider_info::ModelProviderInfo,
) -> Option<&'static str> {
    // Nothing is refused on provider shape any more.
    //
    // AWS signing changes how a request is authenticated, not where it goes or
    // which dialect it speaks. Configured query parameters are part of the route
    // and are pinned with it. A configured routing preference names the provider
    // that serves and bills the request. What remains unattributable is a
    // REQUEST-level routing key the configuration did not ask for, and that is
    // caught per request rather than per provider.
    let _ = provider;
    None
}

/// Bind developer collection to this turn, including provider switches and the
/// authentication-dependent default endpoint. Never read credentials here.
/// Canonical form of a request URL for route comparison.
///
/// `ApiProvider::url_for_path` appends configured query parameters by iterating a
/// `HashMap`, so their order is not stable and the string cannot be reconstructed
/// and compared byte for byte. Compare the path exactly and the parameters as a
/// sorted set instead, which is order-independent and still rejects any parameter
/// the configuration did not ask for.
pub(crate) fn canonical_route(url: &str) -> String {
    let (path, query) = match url.split_once('?') {
        Some((path, query)) => (path, Some(query)),
        None => (url, None),
    };
    let mut parameters: Vec<&str> = query
        .map(|query| query.split('&').filter(|item| !item.is_empty()).collect())
        .unwrap_or_default();
    parameters.sort_unstable();
    if parameters.is_empty() {
        path.to_string()
    } else {
        format!("{path}?{}", parameters.join("&"))
    }
}

/// Canonical `k=v&k=v` for a provider's configured query parameters.
/// A bounded turn label for a compaction of `sub_id`.
pub(crate) fn compaction_turn_label(sub_id: &str) -> String {
    scoped_turn_label("compact:", sub_id)
}

/// A bounded turn label for the startup prewarm of `sub_id`.
///
/// Prewarm is inference the operator paid for - it primes the model's cache, and
/// cache writes are charged - so it is recorded as its own turn rather than
/// escaping collection or being folded into the first real turn's identity.
pub(crate) fn prewarm_turn_label(sub_id: &str) -> String {
    scoped_turn_label("prewarm:", sub_id)
}

/// A bounded `prefix` + `sub_id` turn identity.
///
/// `Attempt::validate` caps a turn identity at 128 bytes, so a long submission id
/// would make the labelled turn unrecordable while the ordinary turn recorded
/// fine. Keep the prefix and as much of the id as fits.
fn scoped_turn_label(prefix: &str, sub_id: &str) -> String {
    const LIMIT: usize = 128;
    let room = LIMIT - prefix.len();
    if sub_id.len() <= room {
        return format!("{prefix}{sub_id}");
    }
    // Truncation alone would merge two submissions that share a long prefix into
    // one turn identity, so keep a digest of the whole id in the part that fits.
    let digest = format!("{:x}", Sha256::digest(sub_id.as_bytes()));
    let digest = &digest[..16];
    let mut end = room.saturating_sub(digest.len() + 1);
    while end > 0 && !sub_id.is_char_boundary(end) {
        end -= 1;
    }
    format!("{prefix}{}-{digest}", &sub_id[..end])
}

/// Guards that keep this turn's collectors attached to a client session.
///
/// Dropping this detaches them, so the session can be reused for work that is
/// not part of the turn.
pub(crate) struct TurnScopes {
    pub(crate) anthropic: Option<Arc<Sampling>>,
    pub(crate) responses: Option<Arc<responses::DeferredResponsesSampling>>,
    pub(crate) chat: Option<Arc<chat::DeferredChatSampling>>,
    _anthropic_scope: SamplingScope,
    _responses_scope: responses::Scope,
    _chat_scope: chat::Scope,
}

/// Bind collection for one turn on one client session.
///
/// Both the ordinary turn path and compaction use this: a compaction request is
/// inference the operator paid for, so it must be recorded like any other turn
/// rather than silently escaping collection.
pub(crate) async fn attach_turn(
    session: &Arc<crate::session::session::Session>,
    turn_context: &crate::session::turn_context::TurnContext,
    client_session: &crate::client::ModelClientSession,
    turn: String,
) -> Result<TurnScopes, CodexErr> {
    let mode = if matches!(
        turn_context.config.accounting,
        AccountingMode::Provider { .. }
    ) {
        let auth = turn_context.provider.auth().await;
        let api = turn_context.provider.api_provider().await?;
        turn_mode(
            &turn_context.config.accounting,
            &turn_context.config.model_provider_id,
            turn_context.provider.info(),
            auth.as_ref().map(codex_login::CodexAuth::auth_mode),
            &api.base_url,
        )
    } else {
        turn_context.config.accounting.clone()
    };
    let collects_wire = |wire| {
        collects(
            &mode,
            &turn_context.config.model_provider_id,
            turn_context.provider.info(),
            wire,
        )
    };
    let anthropic = if collects_wire(codex_model_provider_info::WireApi::Anthropic) {
        session
            .try_ensure_rollout_materialized()
            .await
            .map_err(|_| CodexErr::Fatal(FAILURE.into()))?;
        let runtime = session
            .state_db()
            .ok_or_else(|| CodexErr::Fatal(FAILURE.into()))?;
        Some(Sampling::start(runtime, session.thread_id, turn.clone(), &mode).await?)
    } else {
        None
    };
    let _anthropic_scope =
        SamplingScope::attach(Arc::clone(&client_session.accounting), anthropic.clone())?;
    let responses = collects_wire(codex_model_provider_info::WireApi::Responses).then(|| {
        responses::DeferredResponsesSampling::new(Arc::clone(session), turn.clone(), mode.clone())
    });
    let _responses_scope = responses::Scope::attach(
        Arc::clone(&client_session.responses_accounting),
        responses.clone(),
    )?;
    let chat = collects_wire(codex_model_provider_info::WireApi::Chat)
        .then(|| chat::DeferredChatSampling::new(Arc::clone(session), turn.clone(), mode.clone()));
    let _chat_scope =
        chat::Scope::attach(Arc::clone(&client_session.chat_accounting), chat.clone())?;
    Ok(TurnScopes {
        anthropic,
        responses,
        chat,
        _anthropic_scope,
        _responses_scope,
        _chat_scope,
    })
}

pub(crate) fn canonical_query(
    provider: &codex_model_provider_info::ModelProviderInfo,
) -> Option<String> {
    let parameters = provider.query_params.as_ref()?;
    if parameters.is_empty() {
        return None;
    }
    let mut items: Vec<String> = parameters
        .iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect();
    items.sort();
    Some(items.join("&"))
}

/// The route the client will actually request for `path`, including configured
/// query parameters. Compared through `canonical_route`.
pub(crate) fn pinned_route(approved_endpoint: &str, query: Option<&str>, path: &str) -> String {
    let route = format!("{}/{path}", approved_endpoint.trim_end_matches('/'));
    match query {
        Some(query) if !query.is_empty() => format!("{route}?{query}"),
        _ => route,
    }
}

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
    // Only credential-bearing headers disqualify a route from monetary rates. The
    // built-in providers always set benign originator and version headers, so
    // requiring the maps to be absent entirely would silence pricing on exactly
    // the metered API-key routes that can be priced. This keeps the predicate the
    // legacy eligibility used.
    let credential_header = |headers: &Option<std::collections::HashMap<String, String>>| {
        headers.as_ref().is_some_and(|headers| {
            headers.keys().any(|name| {
                name.eq_ignore_ascii_case("authorization") || name.eq_ignore_ascii_case("api-key")
            })
        })
    };
    // Any built-in provider, at its own default route, in its own dialect. The
    // catalogue states exact rates per provider row, so restricting money to two
    // of those providers left every other provider recording tokens with no
    // economics at all. What must hold is that the request really went to the
    // route those rates are quoted for. Both bases need this; nothing the
    // catalogue says about a route survives being pointed somewhere else.
    // Resolved under the turn's own authentication, because a provider's route
    // can differ by credential: a ChatGPT plan turn goes to the Codex route, not
    // to the API-key one, and both are that provider's own.
    let own_route = codex_model_provider_info::built_in_model_providers(None)
        .get(provider_id)
        .is_some_and(|built_in| {
            built_in.wire_api == provider.wire_api
                && built_in
                    .to_api_provider(auth_mode)
                    .is_ok_and(|api| api.base_url == resolved_endpoint)
        });
    // Which economics apply is decided by the authentication actually used, not
    // by whether per-token rates happen to be available. Treating "API key on a
    // route this client will not price" as subscription capacity would book
    // API-key spend as plan work.
    use codex_protocol::auth::AuthMode;
    // Named positively. Defining the plan side as "not an API key" would make a
    // turn with no visible credential, or one whose credential is supplied out of
    // band, into subscription capacity - the same substitution in the other
    // direction.
    let codex_subscription = matches!(
        auth_mode,
        Some(
            AuthMode::Chatgpt
                | AuthMode::ChatgptAuthTokens
                | AuthMode::Headers
                | AuthMode::AgentIdentity
                | AuthMode::PersonalAccessToken
        )
    );
    // A provider whose own credential is a plan login, as the built-in
    // `claude-plan` provider's command auth is. Such a turn carries no Codex auth
    // mode of its own, so keying only on `AuthMode` dropped it to no economics -
    // and gave it the plan side only when an unrelated ChatGPT login happened to
    // exist. The shape is named here rather than inferred from that accident.
    let provider_plan_login = provider.auth.is_some() && auth_mode != Some(AuthMode::ApiKey);
    let subscription = codex_subscription || provider_plan_login;
    let pricing = if !own_route {
        PriceAuthority::Unavailable
    } else if auth_mode == Some(AuthMode::ApiKey) {
        // Deliberately NOT gated on `api_key_header_name`: the built-in Anthropic
        // provider declares `x-api-key` as its own credential header, so requiring
        // it to be absent made the Anthropic pricing arm below dead code and left
        // metered Anthropic turns with no rate at all. What disqualifies per-token
        // rates is a credential this client cannot attribute to the account the
        // catalogue quotes.
        let attributable = provider.aws.is_none()
            && provider.auth.is_none()
            && provider.experimental_bearer_token.is_none()
            && !credential_header(&provider.http_headers)
            && !credential_header(&provider.env_http_headers);
        if attributable {
            PriceAuthority::ApiKeyRates
        } else {
            PriceAuthority::Unavailable
        }
    } else if subscription {
        PriceAuthority::PlanRate
    } else {
        // No credential this client can name: not an API key it can attribute,
        // not a Codex subscription, not a provider-held plan login. It cannot say
        // which side of the catalogue such a turn is charged on, so it says
        // nothing.
        PriceAuthority::Unavailable
    };
    AccountingMode::Provider {
        scope: *scope,
        provider_id: provider_id.into(),
        wire_api: provider.wire_api,
        approved_endpoint: resolved_endpoint.into(),
        approved_query: canonical_query(provider),
        pricing,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Pricing {
    Anthropic,
    Responses,
    Chat,
    /// Subscription capacity: record the plan rate and the API equivalent, not spend.
    Plan,
    /// Tokens only: this route and credential state no economics this client can use.
    Unavailable,
}

/// The economics a mode admits, shared by admission and by the guards that check
/// a rebound mode still agrees with the sampling already in flight.
fn pricing_for(mode: &AccountingMode) -> Pricing {
    match mode {
        AccountingMode::Provider {
            wire_api, pricing, ..
        } => match pricing {
            PriceAuthority::ApiKeyRates => match wire_api {
                codex_model_provider_info::WireApi::Anthropic => Pricing::Anthropic,
                codex_model_provider_info::WireApi::Responses => Pricing::Responses,
                codex_model_provider_info::WireApi::Chat => Pricing::Chat,
            },
            PriceAuthority::PlanRate => Pricing::Plan,
            PriceAuthority::Unavailable => Pricing::Unavailable,
        },
        AccountingMode::DirectAnthropic { .. } => Pricing::Anthropic,
        AccountingMode::DirectOpenAiChat { .. } => Pricing::Chat,
        AccountingMode::DirectOpenAiResponsesHttp { .. }
        | AccountingMode::DirectOpenAiResponses { .. } => Pricing::Responses,
        AccountingMode::Disabled => Pricing::Unavailable,
    }
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
        let mut approved_query = None;
        let (scope, approved_endpoint, provider, dialect, path) = match mode {
            AccountingMode::Provider {
                scope,
                provider_id,
                wire_api,
                approved_endpoint,
                approved_query: query,
                ..
            } => {
                use codex_model_provider_info::WireApi;
                let (dialect, path) = match wire_api {
                    WireApi::Anthropic => (Dialect::NativeAnthropic, "messages"),
                    WireApi::Responses => (Dialect::Inclusive, "responses"),
                    WireApi::Chat => (Dialect::Inclusive, "chat/completions"),
                };
                approved_query = query.clone();
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
            endpoint: pinned_route(approved_endpoint, approved_query.as_deref(), path),
            provider: provider.into(),
            dialect,
            pricing: pricing_for(mode),
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
        anyhow::ensure!(
            canonical_route(endpoint) == canonical_route(&self.endpoint),
            "accounting route mismatch"
        );
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
            Pricing::Plan => {
                prices::plan_original(model, &self.provider, self.scope, dispatched_at, tier)?
            }
            Pricing::Anthropic => {
                prices::anthropic_original(model, &self.provider, self.scope, dispatched_at)?
            }
            Pricing::Responses => {
                prices::responses_original(model, &self.provider, self.scope, dispatched_at, tier)?
            }
            Pricing::Chat => {
                prices::chat_original(model, &self.provider, self.scope, dispatched_at)?
            }
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
