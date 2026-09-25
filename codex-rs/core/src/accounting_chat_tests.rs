use super::*;
use crate::accounting::transport::{AccountingTransport, ResponseEvidence};
use codex_api::{ChatTokenPresence, ChatUsageObserver, ChatUsagePatch};
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
    AccountingMode::DirectOpenAiChat {
        scope: Uuid::new_v4(),
        approved_endpoint: ENDPOINT.into(),
    }
}
fn provider() -> ModelProviderInfo {
    ModelProviderInfo {
        wire_api: WireApi::Chat,
        ..ModelProviderInfo::create_openai_provider(Some(ENDPOINT.into()))
    }
}
fn auth() -> CodexAuth {
    CodexAuth::from_api_key("synthetic-accounting-test")
}

struct Fixture {
    _home: tempfile::TempDir,
    deferred: Arc<DeferredChatSampling>,
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
            deferred: DeferredChatSampling::new(Arc::new(session), "turn".into(), mode()),
            db,
        })
    }
    async fn resolve(&self) -> anyhow::Result<Arc<Sampling>> {
        Ok(self
            .deferred
            .resolve(
                &provider(),
                Some(&auth()),
                &format!("{ENDPOINT}/chat/completions"),
                &body(),
            )
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
        let values = sqlx::query_scalar::<_, String>(query)
            .fetch_all(&pool)
            .await;
        pool.close().await;
        values?
            .into_iter()
            .map(|value| Ok(serde_json::from_str(&value)?))
            .collect()
    }
}
const ATTEMPTS: &str = "SELECT payload FROM draft_accounting_attempts ORDER BY rowid";
const OBSERVATIONS: &str = "SELECT payload FROM draft_accounting_observations ORDER BY rowid";

#[tokio::test]
async fn accounting_chat_bootstrap_is_lazy_once_and_mode_local() -> anyhow::Result<()> {
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
async fn accounting_chat_direct_auth_and_gateway_eligibility() -> anyhow::Result<()> {
    assert!(legacy_eligible(&provider(), Some(&auth()), &body()));
    assert!(!legacy_eligible(&provider(), None, &body()));
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
            3 => provider.wire_api = WireApi::Responses,
            _ => unreachable!(),
        }
        assert!(!legacy_eligible(&provider, Some(&auth()), &body()));
    }
    for index in 0..11 {
        let mut route = provider();
        let mut request = body();
        match index {
            0 => route = ModelProviderInfo::create_pfterminal_plan_provider(),
            1 => route.name = "compatible".into(),
            2 => route.requires_openai_auth = false,
            3 => route.chat_completions_provider = Some(serde_json::json!({})),
            4 => request.provider = Some(serde_json::json!({})),
            5 => request.provider_options = Some(serde_json::json!({})),
            6 => request.plugins = Some(vec![]),
            7 => route.env_key = Some("ANTHROPIC_API_KEY".into()),
            8 => {
                route.auth = Some(serde_json::from_value(
                    serde_json::json!({"command":"never-execute-fixture"}),
                )?)
            }
            9 => route.http_headers = Some([("API-KEY".into(), "unread".into())].into()),
            10 => {
                route.env_http_headers = Some([("Api-Key".into(), "UNREAD_FIXTURE".into())].into())
            }
            _ => unreachable!(),
        }
        route.base_url = Some(ENDPOINT.into());
        assert!(!legacy_eligible(&route, Some(&auth()), &request));
        let fixture = Fixture::new().await?;
        assert!(
            fixture
                .deferred
                .resolve(
                    &route,
                    Some(&auth()),
                    &format!("{ENDPOINT}/chat/completions"),
                    &request
                )
                .await?
                .is_none()
        );
        assert!(!fixture.deferred.sampling.initialized());
    }
    let fixture = Fixture::new().await?;
    fixture.resolve().await?;
    assert!(
        fixture
            .deferred
            .resolve(
                &provider(),
                None,
                &format!("{ENDPOINT}/chat/completions"),
                &body()
            )
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
        url: format!("{ENDPOINT}/chat/completions"),
        headers: Default::default(),
        body: None,
        compression: Default::default(),
        timeout: None,
    }
}

#[tokio::test]
async fn accounting_chat_auth_and_guard_before_admission() -> anyhow::Result<()> {
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
    let client = codex_api::ChatCompletionsClient::new(transport, api_provider, Arc::new(Denied))
        .with_usage_observer(Some(evidence.clone()));
    assert!(
        client
            .stream_request(body(), Default::default())
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
    let (mut owner, _) = crate::session::tests::make_session_and_context().await;
    owner.services.agent_control = owner
        .services
        .agent_control
        .clone()
        .with_effective_security_policy(
            codex_security_policy::SecurityLevel::Permissive,
            owner.thread_id,
            false,
        )?;
    let owner = Arc::new(owner);
    let memory = crate::memory_stage_one::StageOneMemoryClient::new(
        Arc::downgrade(&owner),
        futures::future::pending().boxed().shared(),
        owner.thread_id,
        &owner.provider().await,
    )
    .await?;
    let guard = crate::memory_stage_one::StageOneGuardedTransport::new(
        AccountingTransport::new(Probe(sends.clone()), Some(evidence), "gpt-5.6-sol".into()),
        Some(memory.binding_for_fixture(owner.thread_id)?),
    );
    // The actual guard checks the live owner before invoking admitted stream().
    let controller = owner
        .services
        .agent_control
        .trusted_security_controller()
        .unwrap();
    let change = controller.confirm_level_change(
        codex_security_policy::SecurityLevel::Moderate,
        codex_security_policy::RevocationState::new(),
    )?;
    controller.apply_confirmed_change(change)?;
    assert!(guard.stream(request()).await.is_err());
    assert_eq!(
        (
            sends.load(Ordering::SeqCst),
            fixture.rows::<Attempt>(ATTEMPTS).await?.len()
        ),
        (0, 0)
    );
    Ok(())
}

#[tokio::test]
async fn accounting_chat_borrowed_binding_denies_existing_and_future_clones() -> anyhow::Result<()>
{
    let (mut owner, _) = crate::session::tests::make_session_and_context().await;
    owner.services.agent_control = owner
        .services
        .agent_control
        .clone()
        .with_effective_security_policy(
            codex_security_policy::SecurityLevel::Permissive,
            owner.thread_id,
            /*inherits_from_spawn_parent*/ false,
        )?;
    let owner = Arc::new(owner);
    let memory = crate::memory_stage_one::StageOneMemoryClient::new(
        Arc::downgrade(&owner),
        futures::future::pending().boxed().shared(),
        owner.thread_id,
        &owner.provider().await,
    )
    .await?;
    let client = owner.services.model_client();
    let existing = client.as_ref().clone();
    let borrowed = client.as_ref();
    borrowed.with_stage_one_memory_binding(memory.binding_for_fixture(owner.thread_id)?)?;
    let future = borrowed.clone();
    let controller = owner
        .services
        .agent_control
        .trusted_security_controller()
        .unwrap();
    let change = controller.confirm_level_change(
        codex_security_policy::SecurityLevel::Moderate,
        codex_security_policy::RevocationState::new(),
    )?;
    controller.apply_confirmed_change(change)?;
    let sends = Arc::new(AtomicUsize::new(0));
    for clone in [&existing, &future] {
        let transport = crate::memory_stage_one::StageOneGuardedTransport::new(
            Probe(sends.clone()),
            clone.stage_one_memory_binding.get().cloned(),
        );
        assert!(
            transport.stream(request()).await.is_err(),
            "binding installed through a borrowed client must deny dispatch on every clone"
        );
    }
    assert_eq!(sends.load(Ordering::SeqCst), 0);
    assert!(
        borrowed
            .with_stage_one_memory_binding(memory.binding_for_fixture(owner.thread_id)?)
            .is_err()
    );
    Ok(())
}

#[tokio::test]
async fn accounting_chat_frame_guard_denies_provider_change_before_client_publication()
-> anyhow::Result<()> {
    use crate::memory_stage_one::{
        StageOneMemoryBinding, StageOneMemoryClient, StageOneMemoryDenial,
    };
    use crate::session::session::{Session, SessionSettingsUpdate};
    use codex_security_policy::SecurityLevel;
    use std::sync::{Mutex, OnceLock, Weak};

    type Observation = (bool, Result<(), StageOneMemoryDenial>);
    #[derive(Default)]
    struct FrameAtConfigChange {
        owner: OnceLock<Weak<Session>>,
        binding: OnceLock<Arc<StageOneMemoryBinding>>,
        observations: Mutex<Vec<Observation>>,
    }
    impl codex_extension_api::ConfigContributor<crate::config::Config> for FrameAtConfigChange {
        fn on_config_changed(
            &self,
            _: &codex_extension_api::ExtensionData,
            _: &codex_extension_api::ExtensionData,
            previous: &crate::config::Config,
            next: &crate::config::Config,
        ) {
            assert_ne!(previous.model_provider, next.model_provider);
            let owner = self.owner.get().unwrap().upgrade().unwrap();
            let configured = owner.provider().now_or_never().unwrap();
            // This real callback runs after configuration publication, before the old
            // implementation replaces its ModelClient. Keep state locked during the
            // frame check, as a concurrent session operation can do in that window.
            let _locked = owner
                .lock_state_for_accounting_fixture()
                .now_or_never()
                .expect("configuration callback must run outside the state lock");
            self.observations.lock().unwrap().push((
                configured == next.model_provider,
                self.binding.get().unwrap().check_stream(),
            ));
        }
    }

    let mut observed = Vec::new();
    for path in ["settings", "turn"] {
        let (mut owner, _) = crate::session::tests::make_session_and_context().await;
        owner.services.agent_control = owner
            .services
            .agent_control
            .clone()
            .with_effective_security_policy(
                SecurityLevel::Permissive,
                owner.thread_id,
                /*inherits_from_spawn_parent*/ false,
            )?;
        let probe = Arc::new(FrameAtConfigChange::default());
        let mut extensions = codex_extension_api::ExtensionRegistryBuilder::new();
        extensions.config_contributor(probe.clone());
        owner.services.extensions = Arc::new(extensions.build());
        let owner = Arc::new(owner);
        probe.owner.set(Arc::downgrade(&owner)).unwrap();
        let memory = StageOneMemoryClient::new(
            Arc::downgrade(&owner),
            futures::future::pending().boxed().shared(),
            owner.thread_id,
            &owner.provider().await,
        )
        .await?;
        probe
            .binding
            .set(memory.binding_for_fixture(owner.thread_id)?)
            .unwrap();
        probe.binding.get().unwrap().check_stream()?;
        let updates = SessionSettingsUpdate {
            model_provider: Some("anthropic".into()),
            ..Default::default()
        };
        match path {
            "settings" => owner.update_settings(updates).await?,
            "turn" => {
                owner
                    .new_turn_with_sub_id("provider-change".into(), updates)
                    .await?;
            }
            _ => unreachable!(),
        }
        observed.push((path, probe.observations.lock().unwrap().clone()));
    }
    assert_eq!(
        observed,
        vec![
            (
                "settings",
                vec![(true, Err(StageOneMemoryDenial::ProviderChanged))]
            ),
            (
                "turn",
                vec![(true, Err(StageOneMemoryDenial::ProviderChanged))]
            ),
        ],
        "each frame must reject the prior provider as soon as changed configuration is observable"
    );
    Ok(())
}

#[tokio::test]
async fn accounting_chat_bootstrap_cancel_scope_and_latch() -> anyhow::Result<()> {
    let (mut session, _) = crate::session::tests::make_session_and_context().await;
    session.services.state_db = None;
    let deferred = DeferredChatSampling::new(Arc::new(session), "missing-state".into(), mode());
    assert!(
        deferred
            .resolve(
                &provider(),
                Some(&auth()),
                &format!("{ENDPOINT}/chat/completions"),
                &body()
            )
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
    fixture.db.close().await;
    let fixture = Fixture::new().await?;
    let slot = Slot::default();
    let scope = Scope::attach(slot.clone(), Some(fixture.deferred.clone()))?;
    assert!(read(&slot)?.is_some());
    let evidence = ResponseEvidence::new(fixture.resolve().await?);
    assert!(
        evidence
            .observe(1, Err(codex_api::InvalidChatUsage))
            .await
            .is_err()
    );
    assert!(fixture.deferred.check().is_err());
    drop(scope);
    assert!(read(&slot)?.is_none());
    fixture.db.close().await;
    Ok(())
}

#[tokio::test]
async fn accounting_chat_response_local_attempt_identity() -> anyhow::Result<()> {
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
                Ok(ChatUsagePatch {
                    input_tokens: ChatTokenPresence::Number(count),
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
    let pool = fixture
        .db
        .sqlite()
        .open_read_only_pool(&fixture.db.sqlite().state_db_path())
        .await?;
    let bound: Vec<String> =
        sqlx::query_scalar("SELECT attempt_id FROM draft_accounting_observations ORDER BY rowid")
            .fetch_all(&pool)
            .await?;
    pool.close().await;
    assert_eq!(
        bound,
        vec![
            attempts[1].attempt_id.to_string(),
            attempts[0].attempt_id.to_string()
        ]
    );
    assert_eq!(
        observations[0].patch.input,
        codex_state::accounting::Presence::Number(7.try_into()?)
    );
    assert_eq!(
        observations[1].patch.input,
        codex_state::accounting::Presence::Number(3.try_into()?)
    );
    Ok(())
}

#[tokio::test]
async fn accounting_chat_role_inheritance_reserved_id_parity() -> anyhow::Result<()> {
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
    std::fs::write(
        &role,
        "developer_instructions = \"synthetic instruction role\"\n",
    )?;
    crate::agent::role::apply_role_to_config(&mut config, Some("fixture"))
        .await
        .map_err(anyhow::Error::msg)?;
    assert_eq!(config.model_provider, provider());
    assert_eq!(config.accounting, fixture.deferred.mode);
    std::fs::write(&role, "[model_providers.openai]\nname = \"fixture role\"\n")?;
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
    assert!(legacy_eligible(
        &config.model_provider,
        Some(&auth()),
        &body()
    ));
    let endpoint = format!("{ENDPOINT}/chat/completions");
    let sampling = fixture
        .deferred
        .resolve(&config.model_provider, Some(&auth()), &endpoint, &body())
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
    let AccountingMode::DirectOpenAiChat { scope, .. } = config.accounting else {
        unreachable!("fixture Responses mode")
    };
    assert_eq!(sampling.scope, scope);
    Ok(())
}

fn body() -> codex_api::ChatCompletionsRequest {
    codex_api::ChatCompletionsRequest {
        model: "gpt-5.6-sol".into(),
        messages: vec![],
        stream: true,
        stream_options: None,
        tools: vec![],
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

struct Mutate(&'static str);
impl codex_api::AuthProvider for Mutate {
    fn add_auth_headers(&self, _: &mut http::HeaderMap) {}
    fn apply_auth(&self, mut request: Request) -> codex_api::AuthProviderFuture<'_> {
        Box::pin(async move {
            tokio::task::yield_now().await;
            request.url = self.0.into();
            Ok(request)
        })
    }
}

#[tokio::test]
async fn accounting_chat_exact_final_endpoint_binding() -> anyhow::Result<()> {
    for endpoint in [
        "https://127.0.0.1:1/v1/chat/completions",
        "http://localhost:1/v1/chat/completions",
        "http://127.0.0.1:2/v1/chat/completions",
        "http://127.0.0.1:1/v1/responses",
        "http://127.0.0.1:1/v1/chat/completions?q=1",
        "http://user@127.0.0.1:1/v1/chat/completions",
        "http://127.0.0.1:1/v1/chat/completions#fragment",
    ] {
        let fixture = Fixture::new().await?;
        let sends = Arc::new(AtomicUsize::new(0));
        let transport = AccountingTransport::new(
            Probe(sends.clone()),
            Some(ResponseEvidence::new(fixture.resolve().await?)),
            "gpt-5.6-sol".into(),
        );
        let api = codex_api::ChatCompletionsClient::new(
            transport,
            provider().to_api_provider(Some(codex_protocol::auth::AuthMode::ApiKey))?,
            Arc::new(Mutate(endpoint)),
        );
        assert!(
            api.stream_request(body(), Default::default())
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
        assert!(fixture.deferred.check().is_err());
    }
    Ok(())
}

#[test]
fn accounting_chat_cache_writes_openrouter_absent_is_zero_elsewhere_unknown() -> anyhow::Result<()>
{
    use codex_state::accounting::Presence;
    let usage = |write| ChatUsagePatch {
        input_tokens: ChatTokenPresence::Number(8499),
        cached_tokens: ChatTokenPresence::Number(1152),
        cache_write_tokens: write,
        output_tokens: ChatTokenPresence::Number(29),
        ..Default::default()
    };
    // OpenRouter omits the count for models with no cache-write charge.
    assert_eq!(
        patch(usage(ChatTokenPresence::Missing), "openrouter")?.write,
        Presence::Number(0.try_into()?)
    );
    // A reported count is kept, on every route.
    for provider in ["openrouter", "vercel", "deepseek"] {
        assert_eq!(
            patch(usage(ChatTokenPresence::Number(300)), provider)?.write,
            Presence::Number(300.try_into()?)
        );
    }
    // Elsewhere an absent count stays unknown, and an explicit null stays null.
    assert_eq!(
        patch(usage(ChatTokenPresence::Missing), "vercel")?.write,
        Presence::Missing
    );
    assert_eq!(
        patch(usage(ChatTokenPresence::Null), "openrouter")?.write,
        Presence::Null
    );
    Ok(())
}
