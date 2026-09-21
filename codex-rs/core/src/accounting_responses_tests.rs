use super::*;
use crate::accounting::transport::{AccountingTransport, ResponseEvidence};
use codex_api::{ResponsesTokenPresence, ResponsesUsageObserver, ResponsesUsagePatch};
use codex_http_client::{HttpTransport, Request, Response, StreamResponse, TransportError};
use codex_protocol::protocol::SessionSource;
use codex_state::accounting::{Attempt, Observation};
use codex_state::{SqliteConfig, StateRuntime, ThreadMetadataBuilder};
use codex_utils_absolute_path::AbsolutePathBuf;
use futures::{FutureExt, StreamExt, poll};
use pretty_assertions::assert_eq;
use std::sync::atomic::AtomicUsize;

const ENDPOINT: &str = "http://127.0.0.1:1/v1";
fn mode() -> AccountingMode {
    AccountingMode::DirectOpenAiResponsesHttp {
        scope: Uuid::new_v4(),
        approved_endpoint: ENDPOINT.into(),
    }
}
fn provider() -> ModelProviderInfo {
    ModelProviderInfo::create_openai_provider(Some(ENDPOINT.into()))
}
fn auth() -> CodexAuth {
    CodexAuth::from_api_key("synthetic-accounting-test")
}

struct Fixture {
    _home: tempfile::TempDir,
    deferred: Arc<DeferredResponsesSampling>,
    db: Arc<StateRuntime>,
}
impl Fixture {
    async fn new() -> anyhow::Result<Self> {
        let home = tempfile::tempdir()?;
        let db = StateRuntime::init(
            SqliteConfig::from_sqlite_home(AbsolutePathBuf::try_from(home.path())?),
            "openai".into(),
        )
        .await?;
        let (mut session, _) = crate::session::tests::make_session_and_context().await;
        session.services.state_db = Some(db.clone());
        db.upsert_thread(
            &ThreadMetadataBuilder::new(
                session.thread_id,
                home.path().join("fixture.jsonl"),
                chrono::Utc::now(),
                SessionSource::Cli,
            )
            .build("openai"),
        )
        .await?;
        Ok(Self {
            _home: home,
            deferred: DeferredResponsesSampling::new(Arc::new(session), "turn".into(), mode()),
            db,
        })
    }
    async fn resolve(&self) -> anyhow::Result<Arc<Sampling>> {
        Ok(self
            .deferred
            .resolve(&provider(), Some(&auth()), &format!("{ENDPOINT}/responses"))
            .await?
            .unwrap())
    }
    async fn rows<T: serde::de::DeserializeOwned>(
        &self,
        query: &'static str,
    ) -> anyhow::Result<Vec<T>> {
        let pool = self
            .db
            .sqlite()
            .open_read_only_pool(&self.db.sqlite().state_db_path())
            .await?;
        let values: Vec<String> = sqlx::query_scalar(query).fetch_all(&pool).await?;
        pool.close().await;
        values
            .into_iter()
            .map(|value| Ok(serde_json::from_str(&value)?))
            .collect()
    }
}
const ATTEMPTS: &str = "SELECT payload FROM draft_accounting_attempts ORDER BY rowid";
const OBSERVATIONS: &str = "SELECT payload FROM draft_accounting_observations ORDER BY rowid";

#[tokio::test]
async fn accounting_responses_bootstrap_is_lazy_and_once() -> anyhow::Result<()> {
    let fixture = Fixture::new().await?;
    assert!(!fixture.deferred.sampling.initialized());
    assert!(fixture.rows::<Attempt>(ATTEMPTS).await.is_err());
    let request = fixture.deferred.request;
    let first = fixture.resolve().await?;
    let second = fixture.resolve().await?;
    assert!(Arc::ptr_eq(&first, &second));
    assert_eq!(first.request, request);
    assert_eq!(fixture.rows::<Attempt>(ATTEMPTS).await?, vec![]);
    Ok(())
}

#[tokio::test]
async fn accounting_responses_auth_route_eligibility() -> anyhow::Result<()> {
    assert!(legacy_eligible(&provider(), Some(&auth())));
    assert!(!legacy_eligible(&provider(), None));
    for kind in 0..4 {
        let mut provider = provider();
        match kind {
            0 => provider.experimental_bearer_token = Some("synthetic-override".into()),
            1 => {
                provider.http_headers = Some([("Authorization".into(), "synthetic".into())].into())
            }
            2 => {
                provider.env_http_headers =
                    Some([("authorization".into(), "UNREAD_FIXTURE".into())].into())
            }
            3 => provider.wire_api = WireApi::Chat,
            _ => unreachable!(),
        }
        assert!(!legacy_eligible(&provider, Some(&auth())));
    }
    let fixture = Fixture::new().await?;
    fixture.resolve().await?;
    assert!(
        fixture
            .deferred
            .resolve(&provider(), None, &format!("{ENDPOINT}/responses"))
            .await
            .is_err()
    );
    assert!(fixture.deferred.check().is_err());
    Ok(())
}

struct Probe(Arc<AtomicUsize>);
impl HttpTransport for Probe {
    async fn execute(&self, _: Request) -> Result<Response, TransportError> {
        unreachable!()
    }
    async fn stream(&self, _: Request) -> Result<StreamResponse, TransportError> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Ok(StreamResponse {
            status: http::StatusCode::OK,
            headers: Default::default(),
            bytes: futures::stream::empty().boxed(),
        })
    }
}
struct Denied;
impl codex_api::AuthProvider for Denied {
    fn add_auth_headers(&self, _: &mut http::HeaderMap) {}
    fn apply_auth(&self, _: Request) -> codex_api::AuthProviderFuture<'_> {
        Box::pin(async {
            tokio::task::yield_now().await;
            Err(codex_api::AuthError::Build("synthetic denial".into()))
        })
    }
}
fn request() -> Request {
    Request {
        method: http::Method::POST,
        url: format!("{ENDPOINT}/responses"),
        headers: Default::default(),
        body: None,
        compression: Default::default(),
        timeout: None,
    }
}

#[tokio::test]
async fn accounting_responses_auth_and_guard_precede_admission() -> anyhow::Result<()> {
    let fixture = Fixture::new().await?;
    let sends = Arc::new(AtomicUsize::new(0));
    let evidence = ResponseEvidence::new(fixture.resolve().await?);
    let transport = crate::memory_stage_one::StageOneGuardedTransport::new(
        AccountingTransport::new(
            Probe(sends.clone()),
            Some(evidence.clone()),
            "gpt-5.6-sol".into(),
        ),
        None,
    );
    let api_provider = provider().to_api_provider(Some(codex_protocol::auth::AuthMode::ApiKey))?;
    let client = codex_api::ResponsesClient::new(transport, api_provider, Arc::new(Denied))
        .with_usage_observer(Some(evidence));
    assert!(
        client
            .stream(
                serde_json::json!({"model":"gpt-5.6-sol"}),
                Default::default(),
                codex_api::Compression::None,
                None
            )
            .await
            .is_err()
    );
    assert_eq!(
        (
            sends.load(Ordering::SeqCst),
            fixture.rows::<Attempt>(ATTEMPTS).await?.len()
        ),
        (0, 0)
    );
    // Existing guard construction rejects a dead owner before any client exists.
    assert!(
        crate::memory_stage_one::StageOneMemoryClient::new(
            std::sync::Weak::new(),
            futures::future::pending().boxed().shared(),
            fixture.deferred.session.thread_id,
            &provider(),
        )
        .await
        .is_err()
    );
    assert_eq!(sends.load(Ordering::SeqCst), 0);
    Ok(())
}

#[tokio::test]
async fn accounting_responses_bootstrap_failure_and_cancellation() -> anyhow::Result<()> {
    let (mut session, _) = crate::session::tests::make_session_and_context().await;
    session.services.state_db = None;
    let deferred =
        DeferredResponsesSampling::new(Arc::new(session), "missing-state".into(), mode());
    assert!(
        deferred
            .resolve(&provider(), Some(&auth()), &format!("{ENDPOINT}/responses"))
            .await
            .is_err()
    );
    assert!(deferred.check().is_err());
    let fixture = Fixture::new().await?;
    let permit = crate::accounting::WRITES.acquire().await?;
    let mut pending = Box::pin(fixture.resolve());
    assert!(poll!(&mut pending).is_pending());
    drop(pending);
    drop(permit);
    assert!(fixture.deferred.check().is_err());
    assert!(!fixture.deferred.sampling.initialized());
    assert!(fixture.rows::<Attempt>(ATTEMPTS).await.is_err());
    Ok(())
}

#[tokio::test]
async fn accounting_responses_scope_cleanup_and_failure_latch() -> anyhow::Result<()> {
    let fixture = Fixture::new().await?;
    let slot = Slot::default();
    let scope = Scope::attach(slot.clone(), Some(fixture.deferred.clone()))?;
    assert!(read(&slot)?.is_some());
    let evidence = ResponseEvidence::new(fixture.resolve().await?);
    assert!(
        evidence
            .observe(1, Err(codex_api::InvalidResponsesUsage))
            .await
            .is_err()
    );
    assert!(fixture.deferred.check().is_err());
    drop(scope);
    assert!(read(&slot)?.is_none());
    Ok(())
}

#[tokio::test]
async fn accounting_responses_response_local_identity() -> anyhow::Result<()> {
    let fixture = Fixture::new().await?;
    let sampling = fixture.resolve().await?;
    let sends = Arc::new(AtomicUsize::new(0));
    let old = ResponseEvidence::new(sampling.clone());
    let new = ResponseEvidence::new(sampling);
    for evidence in [&old, &new] {
        AccountingTransport::new(
            Probe(sends.clone()),
            Some(evidence.clone()),
            "gpt-5.6-sol".into(),
        )
        .stream(request())
        .await?;
    }
    for (evidence, count) in [(&new, 7), (&old, 3)] {
        evidence
            .observe(
                1,
                Ok(ResponsesUsagePatch {
                    input_tokens: ResponsesTokenPresence::Number(count),
                    ..Default::default()
                }),
            )
            .await?;
    }
    let attempts = fixture.rows::<Attempt>(ATTEMPTS).await?;
    let observations = fixture.rows::<Observation>(OBSERVATIONS).await?;
    assert_eq!(attempts.len(), 2);
    assert_eq!(attempts[1].retry_of, Some(attempts[0].attempt_id));
    assert_eq!(attempts[0].request_id, fixture.deferred.request);
    assert_eq!(attempts[1].request_id, fixture.deferred.request);
    assert_ne!(observations[0].source, observations[1].source);
    assert_eq!(sends.load(Ordering::SeqCst), 2);
    Ok(())
}

#[tokio::test]
async fn accounting_responses_role_overlay_preserves_binding() -> anyhow::Result<()> {
    let fixture = Fixture::new().await?;
    let mut config = crate::config::test_config().await;
    config.model_provider_id = "openai".into();
    config.model = Some("gpt-5.6-sol".into());
    config.model_provider = provider();
    config.accounting = fixture.deferred.mode.clone();
    let role = config.codex_home.join("responses-role.toml");
    std::fs::create_dir_all(config.codex_home.as_path())?;
    std::fs::write(
        &role,
        "[model_providers.openai]\nname = \"fixture role\"\nrequest_max_retries = 0\nstream_idle_timeout_ms = 9876\nsupports_websockets = true\n",
    )?;
    config.agent_roles.insert(
        "fixture".into(),
        crate::config::AgentRoleConfig {
            config_file: Some(role.to_path_buf()),
            description: None,
            nickname_candidates: None,
        },
    );
    let parse_error =
        toml::from_str::<codex_config::config_toml::ConfigToml>(&std::fs::read_to_string(&role)?)
            .unwrap_err()
            .to_string();
    assert!(parse_error.contains("reserved built-in provider IDs: `openai`"));
    assert!(
        crate::agent::role::apply_role_to_config(&mut config, Some("fixture"))
            .await
            .is_err()
    );
    assert_eq!(config.model_provider, provider());
    assert_eq!(config.model_provider_id, "openai");
    assert_eq!(config.accounting, fixture.deferred.mode);
    assert!(legacy_eligible(&config.model_provider, Some(&auth())));
    let endpoint = format!("{ENDPOINT}/responses");
    let sampling = fixture
        .deferred
        .resolve(&config.model_provider, Some(&auth()), &endpoint)
        .await?
        .expect("built-in API-key Responses binding");
    assert_eq!(sampling.endpoint, endpoint);
    assert_eq!(sampling.provider, "openai");
    assert_eq!(
        sampling.dialect,
        codex_state::accounting::Dialect::Inclusive
    );
    assert_eq!(sampling.request, fixture.deferred.request);
    assert_eq!(sampling.owner, fixture.deferred.session.thread_id);
    let AccountingMode::DirectOpenAiResponsesHttp { scope, .. } = config.accounting else {
        unreachable!("fixture Responses mode")
    };
    assert_eq!(sampling.scope, scope);
    Ok(())
}
