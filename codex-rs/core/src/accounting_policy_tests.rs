use super::*;
use anyhow::Result;
use codex_protocol::protocol::SessionSource;
use codex_state::SqliteConfig;
use codex_state::ThreadMetadataBuilder;
use codex_utils_absolute_path::AbsolutePathBuf;
use futures::poll;
use pretty_assertions::assert_eq;
use sqlx::Connection;
use std::panic::AssertUnwindSafe;
use std::panic::catch_unwind;
use std::time::Duration;

#[cfg(feature = "developer-accounting")]
#[tokio::test]
async fn accounting_admission_matrix_agrees_at_all_three_gates() -> Result<()> {
    use codex_login::CodexAuth;
    use codex_model_provider_info::{ModelProviderInfo, WireApi};
    let fixture = Fixture::new().await?;
    let api = CodexAuth::from_api_key("synthetic");
    let subscription =
        CodexAuth::from_external_chatgpt_tokens("header.e30.synthetic", "synthetic-account", None)?;
    let mut cells = 0;
    for id in [
        "anthropic",
        "openai",
        "claude-plan",
        "corbanu",
        "openrouter",
        "custom",
    ] {
        for wire in [WireApi::Responses, WireApi::Chat, WireApi::Anthropic] {
            for auth_kind in 0..5 {
                let auth = match auth_kind {
                    1 => Some(&api),
                    2 => Some(&subscription),
                    _ => None,
                };
                // `chat_completions_provider` is no longer an exclusion: a configured
                // routing preference names the provider that serves and bills the
                // request. It stays in the list as an ADMITTED shape so the cells
                // prove that, rather than silently dropping the case.
                for exclusion in [
                    None,
                    Some("aws"),
                    Some("chat_completions_provider"),
                    Some("query_params"),
                ] {
                    let mut provider = ModelProviderInfo::create_openai_provider(None);
                    provider.wire_api = wire;
                    if auth_kind == 3 {
                        provider.experimental_bearer_token = Some("synthetic-plan".into());
                    }
                    if auth_kind == 4 {
                        provider.auth = Some(serde_json::from_value(
                            serde_json::json!({"command":"never-executed-fixture"}),
                        )?);
                    }
                    if exclusion == Some("aws") {
                        provider.aws = Some(codex_model_provider_info::ModelProviderAwsAuthInfo {
                            profile: None,
                            region: None,
                        });
                    }
                    if exclusion == Some("chat_completions_provider") {
                        provider.chat_completions_provider = Some(serde_json::json!({}));
                    }
                    // The resolved URL would carry the query string, so it can never
                    // equal the pinned endpoint: the shape must be refused rather
                    // than admitted and then failed closed. Note that for this
                    // shape the three gates share `route_refusal`, so these cells
                    // pin that every gate consults it - not that three independent
                    // implementations agree.
                    if exclusion == Some("query_params") {
                        provider.query_params =
                            Some(std::collections::HashMap::from([(
                                "api-version".to_string(),
                                "2025-04-01-preview".to_string(),
                            )]));
                    }
                    let selected = developer_accounting_mode(id, &provider);
                    let endpoint = provider
                        .to_api_provider(auth.map(CodexAuth::auth_mode))?
                        .base_url;
                    let bound = turn_mode(
                        &selected,
                        id,
                        &provider,
                        auth.map(CodexAuth::auth_mode),
                        &endpoint,
                    );
                    let admitted =
                        exclusion.is_none() || exclusion == Some("chat_completions_provider");
                    let reason = route_refusal(&provider);
                    assert_eq!(reason.is_none(), admitted, "{id}/{wire}/{exclusion:?}");
                    assert_eq!(!matches!(selected, AccountingMode::Disabled), admitted);
                    assert_eq!(collects(&bound, id, &provider, wire), admitted);
                    // A configured routing preference is attributable only when the
                    // body actually carries exactly it, so build the request the way
                    // this configuration would send it.
                    let mut request = chat_body();
                    request.provider = provider.chat_completions_provider.clone();
                    let eligible = match wire {
                        WireApi::Responses => responses::eligible(&provider, auth),
                        WireApi::Chat => chat::eligible(&provider, auth, &request),
                        WireApi::Anthropic => route_refusal(&provider).is_none(),
                    };
                    assert_eq!(eligible, admitted, "{id}/{wire}/{exclusion:?}: {reason:?}");
                    // And a routing preference configuration did NOT ask for is never
                    // attributable, whatever the provider shape.
                    if wire == WireApi::Chat {
                        let mut foreign = chat_body();
                        foreign.provider = Some(serde_json::json!({"order": ["someone-else"]}));
                        assert!(
                            !chat::eligible(&provider, auth, &foreign),
                            "{id}/{exclusion:?}: an unconfigured routing preference must be refused"
                        );
                    }
                    if admitted {
                        let sample = Sampling::start(
                            fixture.db.clone(),
                            fixture.sampling.owner,
                            format!("matrix-{cells}"),
                            &bound,
                        )
                        .await?;
                        assert_eq!(sample.provider, id);
                        let path = match wire {
                            WireApi::Responses => "responses",
                            WireApi::Chat => "chat/completions",
                            WireApi::Anthropic => "messages",
                        };
                        assert_eq!(sample.endpoint, format!("{endpoint}/{path}"));
                        assert!(
                            sample
                                .admit("fixture", "http://wrong.invalid")
                                .await
                                .is_err()
                        );
                    }
                    cells += 1;
                }
            }
        }
    }
    assert_eq!(cells, 360);
    assert!(serde_json::from_value::<WireApi>(serde_json::json!("unknown")).is_err());
    Ok(())
}

#[test]
fn accounting_chat_request_overrides_are_unattributable() -> Result<()> {
    use codex_model_provider_info::{ModelProviderInfo, WireApi};
    let mut provider = ModelProviderInfo::create_openai_provider(None);
    provider.wire_api = WireApi::Chat;
    for field in ["provider", "provider_options", "plugins"] {
        let mut value =
            serde_json::json!({"model":"fixture","messages":[],"stream":true,"tools":[]});
        value[field] = if field == "plugins" {
            serde_json::json!([])
        } else {
            serde_json::json!({})
        };
        let http =
            codex_http_client::Request::new(http::Method::POST, ENDPOINT.into()).with_json(&value);
        assert_eq!(transport::request_refusal(&http), Some(field));
        let mut request = chat_body();
        match field {
            "provider" => request.provider = Some(value[field].clone()),
            "provider_options" => request.provider_options = Some(value[field].clone()),
            "plugins" => request.plugins = Some(vec![]),
            _ => unreachable!(),
        }
        assert!(
            !chat::eligible(&provider, None, &request),
            "{field} changes serving attribution"
        );
    }
    Ok(())
}

/// The transport sees the body AFTER preparation, which for this client means
/// zstd-compressed bytes. Inspecting only plain JSON refused every real turn and
/// collected nothing, so cover both encodings and both answers here.
#[test]
fn accounting_prepared_bodies_are_inspected_not_refused() -> Result<()> {
    use codex_http_client::{Request, RequestCompression};
    let ordinary = serde_json::json!({"model":"fixture","input":[],"stream":true});
    for compression in [RequestCompression::None, RequestCompression::Zstd] {
        let prepared = Request::new(http::Method::POST, ENDPOINT.into())
            .with_json(&ordinary)
            .with_compression(compression)
            .into_prepared()
            .map_err(anyhow::Error::msg)?;
        assert_eq!(
            transport::request_refusal(&prepared),
            None,
            "an ordinary prepared body must remain collectable under {compression:?}"
        );
        for field in ["provider", "provider_options", "plugins"] {
            let mut value = ordinary.clone();
            value[field] = if field == "plugins" {
                serde_json::json!([])
            } else {
                serde_json::json!({})
            };
            let prepared = Request::new(http::Method::POST, ENDPOINT.into())
                .with_json(&value)
                .with_compression(compression)
                .into_prepared()
                .map_err(anyhow::Error::msg)?;
            assert_eq!(
                transport::request_refusal(&prepared),
                Some(field),
                "{field} must stay unattributable under {compression:?}"
            );
        }
    }
    // Serialize a real request rather than hand-writing the JSON: the wire name is
    // `providerOptions`, and a hand-built snake_case body would pass a check that
    // the actual traffic walks straight past.
    for compression in [RequestCompression::None, RequestCompression::Zstd] {
        let mut request = chat_body();
        request.provider_options = Some(serde_json::json!({"gateway":{"only":["zai"]}}));
        let prepared = Request::new(http::Method::POST, ENDPOINT.into())
            .with_json(&request)
            .with_compression(compression)
            .into_prepared()
            .map_err(anyhow::Error::msg)?;
        assert_eq!(
            transport::request_refusal(&prepared),
            Some("providerOptions"),
            "a serialized gateway pin must be refused under {compression:?}"
        );
        let prepared = Request::new(http::Method::POST, ENDPOINT.into())
            .with_json(&chat_body())
            .with_compression(compression)
            .into_prepared()
            .map_err(anyhow::Error::msg)?;
        assert_eq!(
            transport::request_refusal(&prepared),
            None,
            "an ordinary serialized request must remain collectable under {compression:?}"
        );
    }
    let opaque = Request::new(http::Method::POST, ENDPOINT.into())
        .with_raw_body(vec![0x00, 0x01, 0x02, 0x03]);
    assert_eq!(
        transport::request_refusal(&opaque),
        Some("uninspectable request body"),
        "a body whose routing keys cannot be read must still be refused"
    );
    Ok(())
}

/// The subscription integration test uses a wiremock endpoint, which already
/// forces `api_key_pricing` off. Pin the auth-mode half of the rule directly, at
/// the real default endpoint, so swapping the auth cannot silently keep prices.
#[test]
fn accounting_pricing_authority_follows_auth_mode_at_the_default_endpoint() {
    use codex_model_provider_info::{ModelProviderInfo, WireApi};
    use codex_protocol::auth::AuthMode;
    // Both built-in metered routes, because the Anthropic arm was dead code until
    // the predicate stopped rejecting a provider for declaring its own api-key
    // header, and nothing asserted the positive side of that arm.
    for (id, provider, endpoint) in [
        (
            "openai",
            ModelProviderInfo::create_openai_provider(None),
            "https://api.openai.com/v1",
        ),
        (
            "anthropic",
            ModelProviderInfo::create_anthropic_provider(),
            codex_model_provider_info::ANTHROPIC_BASE_URL,
        ),
    ] {
        let mode = crate::config::AccountingMode::Provider {
            scope: uuid::Uuid::new_v4(),
            provider_id: id.into(),
            wire_api: provider.wire_api,
            approved_endpoint: endpoint.into(),
            api_key_pricing: false,
        };
        let bound = super::turn_mode(
            &mode,
            id,
            &provider,
            Some(AuthMode::ApiKey),
            endpoint,
        );
        let crate::config::AccountingMode::Provider {
            api_key_pricing, ..
        } = bound
        else {
            panic!("{id} provider mode must survive rebinding");
        };
        assert!(
            api_key_pricing,
            "a metered API-key {id} route at its own default endpoint must be priced"
        );
    }
    let provider = ModelProviderInfo::create_openai_provider(None);
    assert_eq!(provider.wire_api, WireApi::Responses);
    let scope = uuid::Uuid::new_v4();
    let mode = crate::config::AccountingMode::Provider {
        scope,
        provider_id: "openai".into(),
        wire_api: WireApi::Responses,
        approved_endpoint: "https://api.openai.com/v1".into(),
        api_key_pricing: false,
    };
    for (auth, expected) in [
        (Some(AuthMode::ApiKey), true),
        (Some(AuthMode::Chatgpt), false),
        (Some(AuthMode::ChatgptAuthTokens), false),
        (None, false),
    ] {
        let bound = super::turn_mode(&mode, "openai", &provider, auth, "https://api.openai.com/v1");
        let crate::config::AccountingMode::Provider {
            api_key_pricing, ..
        } = bound
        else {
            panic!("provider mode must survive rebinding for {auth:?}");
        };
        assert_eq!(
            api_key_pricing, expected,
            "only API-key authority may supply monetary rates ({auth:?})"
        );
    }
}

fn chat_body() -> codex_api::ChatCompletionsRequest {
    codex_api::ChatCompletionsRequest {
        model: "fixture".into(),
        messages: vec![],
        stream: true,
        tools: vec![],
        stream_options: None,
        tool_choice: None,
        parallel_tool_calls: None,
        prompt_cache_key: None,
        response_format: None,
        emit_usage: None,
        enable_thinking: None,
        reasoning_effort: None,
        reasoning: None,
        provider: None,
        plugins: None,
        provider_options: None,
    }
}

const MODEL: &str = "claude-opus-5";
const ENDPOINT: &str = "http://127.0.0.1:1/v1/messages";

/// A body that can re-route the serving provider must not be attributed, must not
/// kill the turn, and must not leave partial evidence. Nothing covered this
/// before, which is how an earlier version that rejected the turn's sampling and
/// then sent the request anyway passed every lane.
#[tokio::test]
async fn accounting_unattributable_request_is_served_without_evidence() -> Result<()> {
    use codex_http_client::HttpTransport;
    use codex_http_client::Request;
    use codex_http_client::RequestCompression;
    use std::sync::atomic::AtomicUsize;
    use std::sync::atomic::Ordering;

    #[derive(Default, Clone)]
    struct Counting(Arc<AtomicUsize>);
    impl HttpTransport for Counting {
        async fn execute(
            &self,
            _req: Request,
        ) -> std::result::Result<codex_http_client::Response, codex_http_client::TransportError>
        {
            self.0.fetch_add(1, Ordering::SeqCst);
            Err(codex_http_client::TransportError::Build("stub".into()))
        }
        async fn stream(
            &self,
            _req: Request,
        ) -> std::result::Result<
            codex_http_client::StreamResponse,
            codex_http_client::TransportError,
        > {
            self.0.fetch_add(1, Ordering::SeqCst);
            Err(codex_http_client::TransportError::Build("stub".into()))
        }
    }

    let fixture = Fixture::new().await?;
    let evidence = transport::ResponseEvidence::new(Arc::clone(&fixture.sampling));
    let inner = Counting::default();
    let sent = Arc::clone(&inner.0);
    let wrapper =
        transport::AccountingTransport::new(inner, Some(Arc::clone(&evidence)), MODEL.into());
    let mut body = serde_json::json!({"model": MODEL, "input": [], "stream": true});
    body["providerOptions"] = serde_json::json!({"gateway": {"only": ["zai"]}});
    let request = Request::new(http::Method::POST, ENDPOINT.into())
        .with_json(&body)
        .with_compression(RequestCompression::Zstd)
        .into_prepared()
        .map_err(anyhow::Error::msg)?;
    // The request reaches the real transport rather than being short-circuited.
    assert!(wrapper.stream(request).await.is_err(), "stub inner transport");
    assert_eq!(sent.load(Ordering::SeqCst), 1, "the turn's request was sent");
    // The turn survives: rejecting the sampling here would abort it at the next check.
    fixture.sampling.check()?;
    // And nothing was recorded for a request we could not attribute.
    assert!(fixture.attempts().await?.is_empty());
    // A usage event on an excluded request records nothing instead of failing the
    // stream - on every dialect, because the three observers are three different
    // code shapes and a future edit could drop the guard from one of them.
    codex_api::ResponsesUsageObserver::observe(
        &*evidence,
        0,
        Ok(codex_api::ResponsesUsagePatch::default()),
    )
    .await
    .map_err(|error| anyhow::anyhow!("excluded responses usage must not fail: {error}"))?;
    codex_api::ChatUsageObserver::observe(&*evidence, 0, Ok(codex_api::ChatUsagePatch::default()))
        .await
        .map_err(|error| anyhow::anyhow!("excluded chat usage must not fail: {error}"))?;
    codex_api::AnthropicUsageObserver::observe(
        &*evidence,
        0,
        Ok(codex_api::AnthropicUsagePatch::default()),
    )
    .await
    .map_err(|error| anyhow::anyhow!("excluded anthropic usage must not fail: {error}"))?;
    fixture.sampling.check()?;
    assert!(fixture.attempts().await?.is_empty());
    Ok(())
}

struct Fixture {
    home: tempfile::TempDir,
    db: Arc<StateRuntime>,
    sampling: Arc<Sampling>,
}

impl Fixture {
    async fn new() -> Result<Self> {
        let home = tempfile::tempdir()?;
        let db = StateRuntime::init(
            SqliteConfig::from_sqlite_home(AbsolutePathBuf::try_from(home.path())?),
            "anthropic".into(),
        )
        .await?;
        let owner = ThreadId::new();
        db.upsert_thread(
            &ThreadMetadataBuilder::new(
                owner,
                home.path().join("policy.jsonl"),
                chrono::Utc::now(),
                SessionSource::Cli,
            )
            .build("anthropic"),
        )
        .await?;
        let sampling = Sampling::start(db.clone(), owner, "policy-turn".into(), &mode()).await?;
        Ok(Self { home, db, sampling })
    }

    async fn connection(&self) -> Result<sqlx::SqliteConnection> {
        let pool = self
            .db
            .sqlite()
            .open_read_write_pool(&self.db.sqlite().state_db_path())
            .await?;
        let connection = pool.acquire().await?.detach();
        pool.close().await;
        Ok(connection)
    }

    async fn attempts(&self) -> Result<Vec<Attempt>> {
        let mut connection = self.connection().await?;
        let rows: Vec<String> =
            sqlx::query_scalar("SELECT payload FROM draft_accounting_attempts ORDER BY rowid")
                .fetch_all(&mut connection)
                .await?;
        connection.close().await?;
        rows.into_iter()
            .map(|row| Ok(serde_json::from_str(&row)?))
            .collect()
    }

    async fn two_reopens(self, expected: Vec<Attempt>) -> Result<()> {
        // Release the sampling owner before shutting down its SQLite workers.
        let Self { home, db, sampling } = self;
        drop(sampling);
        db.close().await;
        drop(db);
        for _ in 0..2 {
            let db = StateRuntime::init(
                SqliteConfig::from_sqlite_home(AbsolutePathBuf::try_from(home.path())?),
                "anthropic".into(),
            )
            .await?;
            AccountingStore::open(&db, now()).await?;
            let pool = db
                .sqlite()
                .open_read_only_pool(&db.sqlite().state_db_path())
                .await?;
            let rows = sqlx::query_scalar::<_, String>(
                "SELECT payload FROM draft_accounting_attempts ORDER BY rowid",
            )
            .fetch_all(&pool)
            .await;
            pool.close().await;
            let actual: Vec<Attempt> = rows?
                .into_iter()
                .map(|row| serde_json::from_str(&row))
                .collect::<Result<_, _>>()?;
            assert_eq!(actual, expected);
            db.close().await;
            drop(db);
        }
        Ok(())
    }
}

fn mode() -> AccountingMode {
    AccountingMode::DirectAnthropic {
        scope: Uuid::new_v4(),
        approved_endpoint: "http://127.0.0.1:1/v1".into(),
    }
}

fn poison<T>(mutex: &Mutex<T>) {
    assert!(
        catch_unwind(AssertUnwindSafe(|| {
            let _guard = mutex.lock().unwrap();
            panic!("deliberate policy fixture poison");
        }))
        .is_err()
    );
}

#[tokio::test]
async fn accounting_policy_wait_cancellation_and_post_wait_failure_have_no_effect() -> Result<()> {
    let fixture = Fixture::new().await?;
    let permit = WRITES.acquire().await?;
    let mut waiting = Box::pin(fixture.sampling.admit(MODEL, ENDPOINT));
    assert!(poll!(&mut waiting).is_pending());
    drop(waiting);
    fixture.sampling.check()?;
    let mut start = Box::pin(Sampling::start(
        fixture.db.clone(),
        fixture.sampling.owner,
        "cancelled-start".into(),
        &AccountingMode::Disabled,
    ));
    // OFF fails before the gate; a valid start waits and can be dropped without a Slot.
    assert!(start.as_mut().await.is_err());
    drop(start);
    let enabled = mode();
    let mut start = Box::pin(Sampling::start(
        fixture.db.clone(),
        fixture.sampling.owner,
        "cancelled-start".into(),
        &enabled,
    ));
    assert!(poll!(&mut start).is_pending());
    drop(start);
    let mut rejected = Box::pin(fixture.sampling.admit(MODEL, ENDPOINT));
    assert!(poll!(&mut rejected).is_pending());
    fixture.sampling.reject();
    drop(permit);
    assert!(rejected.await.is_err());
    assert!(fixture.attempts().await?.is_empty());
    let fresh = Sampling::start(
        fixture.db.clone(),
        fixture.sampling.owner,
        "fresh".into(),
        &enabled,
    )
    .await?;
    let attempt = fresh.admit(MODEL, ENDPOINT).await?;
    assert_eq!(attempt.retry_of, None);
    drop(fresh);
    fixture.two_reopens(vec![attempt]).await
}

#[tokio::test]
async fn accounting_policy_serial_retry_identity_and_time_sample_after_gate() -> Result<()> {
    let fixture = Fixture::new().await?;
    let permit = WRITES.acquire().await?;
    let mut first = Box::pin(fixture.sampling.admit(MODEL, ENDPOINT));
    let mut second = Box::pin(fixture.sampling.admit(MODEL, ENDPOINT));
    assert!(poll!(&mut first).is_pending());
    assert!(poll!(&mut second).is_pending());
    let queued_at = now();
    tokio::time::timeout(Duration::from_secs(1), async {
        while now() <= queued_at {
            tokio::task::yield_now().await;
        }
    })
    .await?;
    let barrier_time = now();
    drop(permit);
    let (first, second) = tokio::try_join!(first, second)?;
    assert_eq!(first.retry_of, None);
    assert_eq!(second.retry_of, Some(first.attempt_id));
    assert_ne!(first.attempt_id, second.attempt_id);
    for attempt in [&first, &second] {
        assert_eq!(attempt.request_id, fixture.sampling.request);
        assert_eq!(attempt.thread_id, fixture.sampling.owner);
        assert_eq!(attempt.turn, "policy-turn");
        assert!(i64::from(attempt.dispatched_at_ms) >= barrier_time);
    }
    assert!(i64::from(second.dispatched_at_ms) >= i64::from(first.dispatched_at_ms));
    fixture.two_reopens(vec![first, second]).await
}

#[tokio::test]
async fn accounting_policy_observe_wait_cancel_and_start_cancel_publish_nothing() -> Result<()> {
    let fixture = Fixture::new().await?;
    let attempt = fixture.sampling.admit(MODEL, ENDPOINT).await?;
    let permit = WRITES.acquire().await?;
    let mut waiting = Box::pin(fixture.sampling.observe(
        &attempt,
        Uuid::new_v4(),
        0,
        AnthropicUsagePatch::default(),
    ));
    assert!(poll!(&mut waiting).is_pending());
    drop(waiting);
    fixture.sampling.check()?;
    drop(permit);
    let mut connection = fixture.connection().await?;
    sqlx::query("BEGIN IMMEDIATE")
        .execute(&mut connection)
        .await?;
    let slot = Slot::default();
    let enabled = mode();
    let mut start = Box::pin(async {
        let sampling = Sampling::start(
            fixture.db.clone(),
            fixture.sampling.owner,
            "cancelled".into(),
            &enabled,
        )
        .await?;
        SamplingScope::attach(slot.clone(), Some(sampling))
    });
    assert!(poll!(&mut start).is_pending());
    assert_eq!(WRITES.available_permits(), 0);
    drop(start);
    assert_eq!(WRITES.available_permits(), 1);
    assert!(read_slot(&slot)?.is_none());
    sqlx::query("ROLLBACK").execute(&mut connection).await?;
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM draft_accounting_observations")
        .fetch_one(&mut connection)
        .await?;
    assert_eq!(count, 0);
    connection.close().await?;
    fixture.two_reopens(vec![attempt]).await
}

#[tokio::test]
async fn accounting_policy_cancel_in_operation_rejects_and_releases_capacity() -> Result<()> {
    for operation in ["admit", "observe"] {
        let fixture = Fixture::new().await?;
        let attempt = fixture.sampling.admit(MODEL, ENDPOINT).await?;
        let mut connection = fixture.connection().await?;
        sqlx::query("BEGIN IMMEDIATE")
            .execute(&mut connection)
            .await?;
        let sampling = &fixture.sampling;
        let mut pending = Box::pin(async {
            if operation == "admit" {
                sampling.admit(MODEL, ENDPOINT).await.map(|_| ())
            } else {
                sampling
                    .observe(
                        &attempt,
                        Uuid::new_v4(),
                        0,
                        AnthropicUsagePatch {
                            input_tokens: AnthropicTokenPresence::Number(7),
                            ..Default::default()
                        },
                    )
                    .await
            }
        });
        assert!(poll!(&mut pending).is_pending());
        assert_eq!(
            WRITES.available_permits(),
            0,
            "inside the operation, not queued"
        );
        drop(pending);
        assert!(fixture.sampling.check().is_err());
        assert_eq!(WRITES.available_permits(), 1);
        sqlx::query("ROLLBACK").execute(&mut connection).await?;
        connection.close().await?;
        assert!(fixture.sampling.admit(MODEL, ENDPOINT).await.is_err());
        let mut connection = fixture.connection().await?;
        let count: i64 = sqlx::query_scalar("SELECT count(*) FROM draft_accounting_observations")
            .fetch_one(&mut connection)
            .await?;
        assert_eq!(count, 0);
        connection.close().await?;
        let fresh = tokio::time::timeout(
            Duration::from_secs(10),
            Sampling::start(
                fixture.db.clone(),
                fixture.sampling.owner,
                "fresh".into(),
                &mode(),
            ),
        )
        .await??;
        let next = fresh.admit(MODEL, ENDPOINT).await?;
        assert_eq!(next.retry_of, None);
        assert_ne!(next.request_id, attempt.request_id);
        // This barrier blocked the first DB operation. It does not establish
        // rollback for arbitrary cancellation after SQLite has committed.
        fixture.two_reopens(vec![attempt, next]).await?;
    }
    Ok(())
}

#[tokio::test]
async fn accounting_policy_completion_guard_never_repairs_possible_committed_rows() -> Result<()> {
    let fixture = Fixture::new().await?;
    let attempt = fixture.sampling.admit(MODEL, ENDPOINT).await?;
    // Directly exercise the uncertain-completion guard with durable data.
    // This is not a claim of a process kill at an unobservable SQL instruction.
    drop(Completion {
        sampling: &fixture.sampling,
        complete: false,
    });
    assert!(fixture.sampling.check().is_err());
    assert!(fixture.sampling.admit(MODEL, ENDPOINT).await.is_err());
    fixture.two_reopens(vec![attempt]).await
}

#[tokio::test]
async fn accounting_policy_previous_poison_before_and_during_admission_is_sticky() -> Result<()> {
    for phase in ["before", "during"] {
        let fixture = Fixture::new().await?;
        let mut connection = fixture.connection().await?;
        if phase == "during" {
            sqlx::query("BEGIN IMMEDIATE")
                .execute(&mut connection)
                .await?;
        } else {
            poison(&fixture.sampling.previous);
        }
        let mut pending = Box::pin(fixture.sampling.admit(MODEL, ENDPOINT));
        if phase == "during" {
            assert!(poll!(&mut pending).is_pending());
            assert_eq!(WRITES.available_permits(), 0);
            poison(&fixture.sampling.previous);
            sqlx::query("ROLLBACK").execute(&mut connection).await?;
        }
        assert!(pending.await.is_err());
        assert!(fixture.sampling.check().is_err());
        assert!(fixture.sampling.admit(MODEL, ENDPOINT).await.is_err());
        assert_eq!(WRITES.available_permits(), 1);
        connection.close().await?;
        let rows = fixture.attempts().await?;
        assert_eq!(rows.len(), usize::from(phase == "during"));
        if let Some(attempt) = rows.first() {
            assert_eq!(attempt.retry_of, None);
            assert_eq!(attempt.request_id, fixture.sampling.request);
        }
        fixture.two_reopens(rows).await?;
    }
    Ok(())
}

#[tokio::test]
async fn accounting_policy_slot_poison_rejects_stale_and_incoming_without_disabling() -> Result<()>
{
    for operation in ["read", "attach", "drop"] {
        let fixture = Fixture::new().await?;
        let incoming = Sampling::start(
            fixture.db.clone(),
            fixture.sampling.owner,
            "incoming".into(),
            &mode(),
        )
        .await?;
        let slot = Slot::default();
        let scope = SamplingScope::attach(slot.clone(), Some(fixture.sampling.clone()))?;
        poison(&slot);
        match operation {
            "read" => assert!(read_slot(&slot).is_err()),
            "attach" => {
                assert!(SamplingScope::attach(slot.clone(), Some(incoming.clone())).is_err());
                assert!(incoming.check().is_err());
            }
            _ => {}
        }
        drop(scope);
        assert!(fixture.sampling.check().is_err());
        assert!(slot.is_poisoned());
        assert!(slot.lock().err().unwrap().into_inner().is_none());
        assert!(read_slot(&slot).is_err(), "empty poison must not look OFF");
        assert!(SamplingScope::attach(slot.clone(), None).is_err());
        assert!(fixture.attempts().await?.is_empty());
        fixture.db.close().await;
    }
    Ok(())
}
