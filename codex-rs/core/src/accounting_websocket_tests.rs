use super::*;
use crate::config::AccountingMode;
use codex_protocol::protocol::SessionSource;
use codex_state::SqliteConfig;
use codex_state::StateRuntime;
use codex_state::ThreadMetadataBuilder;
use codex_state::accounting::Attempt;
use codex_state::accounting::Observation;
use codex_state::accounting::Snapshot;
use codex_utils_absolute_path::AbsolutePathBuf;
use futures::poll;
use pretty_assertions::assert_eq;
use uuid::Uuid;

const BASE: &str = "http://127.0.0.1:12345/v1";
fn mode() -> AccountingMode {
    AccountingMode::DirectOpenAiResponses {
        scope: Uuid::new_v4(),
        approved_endpoint: BASE.into(),
    }
}
fn provider() -> ModelProviderInfo {
    ModelProviderInfo::create_openai_provider(Some(BASE.into()))
}
fn auth() -> CodexAuth {
    CodexAuth::from_api_key("synthetic-accounting-ws")
}
fn provenance() -> Provenance {
    Provenance::capture(
        &provider(),
        Some(&auth()),
        &provider()
            .to_api_provider(Some(codex_protocol::auth::AuthMode::ApiKey))
            .unwrap(),
        false,
    )
}
struct Fixture {
    _home: tempfile::TempDir,
    deferred: Arc<DeferredResponsesSampling>,
    db: Arc<StateRuntime>,
}
impl Fixture {
    async fn new(mode: AccountingMode) -> anyhow::Result<Self> {
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
            db,
            deferred: DeferredResponsesSampling::new(Arc::new(session), "fixture".into(), mode),
        })
    }
    async fn resolve(&self) -> anyhow::Result<Arc<Sampling>> {
        Ok(self
            .deferred
            .resolve(&provider(), Some(&auth()), &format!("{BASE}/responses"))
            .await?
            .unwrap())
    }
    async fn rows<T: serde::de::DeserializeOwned>(&self, table: &str) -> anyhow::Result<Vec<T>> {
        let pool = self
            .db
            .sqlite()
            .open_read_only_pool(&self.db.sqlite().state_db_path())
            .await?;
        let rows: Vec<String> = sqlx::query_scalar(match table {
            "draft_accounting_attempts" => {
                "SELECT payload FROM draft_accounting_attempts ORDER BY rowid"
            }
            "draft_accounting_observations" => {
                "SELECT payload FROM draft_accounting_observations ORDER BY rowid"
            }
            "draft_accounting_price_snapshots" => {
                "SELECT payload FROM draft_accounting_price_snapshots ORDER BY rowid"
            }
            _ => panic!("unknown fixture table"),
        })
        .fetch_all(&pool)
        .await?;
        pool.close().await;
        rows.iter()
            .map(|row| Ok(serde_json::from_str(row)?))
            .collect()
    }
    async fn admission(&self) -> anyhow::Result<Arc<dyn ResponsesWebsocketAdmission>> {
        Ok(Admission::new(
            self.resolve().await?,
            provenance(),
            endpoint(BASE)?,
        ))
    }
}
#[tokio::test]
async fn accounting_responses_ws_mode_scope_and_lazy_bootstrap() -> anyhow::Result<()> {
    for mode in [
        AccountingMode::Disabled,
        AccountingMode::DirectOpenAiResponsesHttp {
            scope: Uuid::new_v4(),
            approved_endpoint: BASE.into(),
        },
        mode(),
    ] {
        let combined = matches!(mode, AccountingMode::DirectOpenAiResponses { .. });
        let fixture = Fixture::new(mode).await?;
        assert_eq!(fixture.deferred.websocket_endpoint()?.is_some(), combined);
        assert!(
            fixture
                .rows::<Attempt>("draft_accounting_attempts")
                .await
                .is_err()
        );
        if combined {
            let first = fixture.resolve().await?;
            assert!(Arc::ptr_eq(&first, &fixture.resolve().await?));
            assert!(
                fixture
                    .rows::<Attempt>("draft_accounting_attempts")
                    .await?
                    .is_empty()
            );
        }
    }
    Ok(())
}
#[tokio::test]
async fn accounting_responses_ws_auth_route_eligibility() -> anyhow::Result<()> {
    let fixture = Fixture::new(mode()).await?;
    assert!(provenance().validate(&fixture.deferred, None)?);
    let api = provider().to_api_provider(Some(codex_protocol::auth::AuthMode::ApiKey))?;
    for variant in 0..9 {
        let mut provider = provider();
        match variant {
            8 => {
                provider.auth = Some(serde_json::from_value(
                    serde_json::json!({"command":"never-run-fixture"}),
                )?)
            }
            0 => provider.experimental_bearer_token = Some("synthetic".into()),
            1 => {
                provider.http_headers = Some([("Authorization".into(), "synthetic".into())].into())
            }
            2 => {
                provider.env_http_headers =
                    Some([("api-key".into(), "UNREAD_FIXTURE".into())].into())
            }
            3 => provider.wire_api = codex_model_provider_info::WireApi::Chat,
            4 => provider.http_headers = Some([("api-key".into(), "synthetic".into())].into()),
            _ => {}
        }
        let auth = if variant == 7 {
            CodexAuth::from_external_chatgpt_tokens(
                "header.e30.synthetic",
                "synthetic-account",
                None,
            )?
        } else {
            auth()
        };
        let current = Provenance::capture(
            &provider,
            if variant == 5 { None } else { Some(&auth) },
            &api,
            variant == 6,
        );
        assert!(!current.validate(&fixture.deferred, None)?);
    }
    fixture.resolve().await?;
    let mut current = provenance();
    current.api_key = false;
    assert!(current.validate(&fixture.deferred, None).is_err());
    assert!(fixture.deferred.check().is_err());
    Ok(())
}
#[tokio::test]
async fn accounting_responses_ws_exact_endpoint_binding() -> anyhow::Result<()> {
    for (http, ws) in [
        (BASE, "ws://127.0.0.1:12345/v1/responses"),
        (
            "https://api.openai.com/v1",
            "wss://api.openai.com/v1/responses",
        ),
    ] {
        assert_eq!(endpoint(http)?, ws);
    }
    for bad in [
        "http://user@127.0.0.1/v1",
        "http://127.0.0.1/v1?q=1",
        "http://127.0.0.1/v1#fragment",
        "ftp://127.0.0.1/v1",
    ] {
        assert!(endpoint(bad).is_err());
    }
    for bad in [
        "ws://127.0.0.1:12346/v1/responses",
        "wss://127.0.0.1:12345/v1/responses",
        "ws://localhost:12345/v1/responses",
        "ws://127.0.0.1:12345/v2/responses",
    ] {
        let fixture = Fixture::new(mode()).await?;
        let mut current = provenance();
        current.endpoint = Some(bad.into());
        assert!(current.validate(&fixture.deferred, None).is_err());
        assert!(
            fixture
                .rows::<Attempt>("draft_accounting_attempts")
                .await
                .is_err()
        );
    }
    Ok(())
}
#[tokio::test]
async fn accounting_responses_ws_cached_connection_provenance() -> anyhow::Result<()> {
    for variant in 0..3 {
        let fixture = Fixture::new(mode()).await?;
        let current = provenance();
        let mut cached = current.clone();
        if variant == 1 {
            cached.api_key = false;
        }
        if variant == 2 {
            cached.endpoint = None;
        }
        assert_eq!(
            current.validate(&fixture.deferred, Some(&cached)).is_ok(),
            variant == 0
        );
    }
    Ok(())
}
#[tokio::test]
async fn accounting_responses_ws_fallback_keeps_request_and_predecessor() -> anyhow::Result<()> {
    let fixture = Fixture::new(mode()).await?;
    let ws = fixture.admission().await?;
    let old = ws.admit("gpt-5.6-sol".into(), None).await?;
    let sampling = fixture.resolve().await?;
    let http = sampling
        .admit_with_tier("gpt-5.6-sol", &format!("{BASE}/responses"), None)
        .await?;
    old.observe(
        2,
        Ok(codex_api::ResponsesUsagePatch {
            input_tokens: codex_api::ResponsesTokenPresence::Number(7),
            ..Default::default()
        }),
    )
    .await?;
    let records = fixture.rows::<Attempt>("draft_accounting_attempts").await?;
    assert_eq!(records.len(), 2);
    assert_eq!(http.retry_of, Some(records[0].attempt_id));
    assert_eq!(http.request_id, records[0].request_id);
    assert_eq!(
        fixture
            .rows::<Observation>("draft_accounting_observations")
            .await?
            .len(),
        1
    );
    Ok(())
}
#[tokio::test]
async fn accounting_responses_ws_original_price_binding() -> anyhow::Result<()> {
    let fixture = Fixture::new(mode()).await?;
    let admission = fixture.admission().await?;
    for (model, tier) in [
        ("gpt-5.6-sol", None),
        ("gpt-6-astra", None),
        ("gpt-5.6-sol", Some("priority")),
        ("alias", None),
        ("remote-only", None),
    ] {
        admission
            .admit(model.into(), tier.map(str::to_string))
            .await?;
    }
    let snapshots = fixture
        .rows::<Snapshot>("draft_accounting_price_snapshots")
        .await?;
    assert_eq!(snapshots.len(), 1);
    let source = serde_json::to_vec(&(
        "openai-responses-api-key-bundled-v1",
        "openai",
        "gpt-5.6-sol",
        "api_key",
        "default",
        "USD/million",
        5000u32,
        30000u32,
        Some(500u32),
    ))?;
    assert_eq!(
        snapshots[0].source_reference,
        Uuid::new_v5(&Uuid::NAMESPACE_OID, &source)
    );
    assert_eq!(
        serde_json::to_value(&snapshots[0].rates)?,
        serde_json::json!({"noncached":"5","output":"30","read":"0.5","write":null})
    );
    assert_eq!(
        fixture
            .rows::<Attempt>("draft_accounting_attempts")
            .await?
            .len(),
        5
    );
    admission.admit("gpt-5.6-sol".into(), None).await?;
    let later = fixture
        .rows::<Snapshot>("draft_accounting_price_snapshots")
        .await?;
    assert!(later.contains(&snapshots[0]));
    Ok(())
}
#[tokio::test]
async fn accounting_responses_ws_failure_and_cancellation_latch() -> anyhow::Result<()> {
    let fixture = Fixture::new(mode()).await?;
    let admission = fixture.admission().await?;
    let permit = crate::accounting::WRITES.acquire().await?;
    let mut pending = admission.admit("gpt-5.6-sol".into(), None);
    assert!(poll!(&mut pending).is_pending());
    drop(pending);
    drop(permit);
    // Cancellation before the write permit cannot have committed an intent.
    assert!(fixture.deferred.check().is_ok());
    let observer = admission.admit("gpt-5.6-sol".into(), None).await?;
    assert!(
        observer
            .observe(1, Err(codex_api::InvalidResponsesUsage))
            .await
            .is_err()
    );
    assert!(fixture.deferred.check().is_err());
    assert!(admission.admit("gpt-5.6-sol".into(), None).await.is_err());
    let slot = super::super::responses::Slot::default();
    let scope =
        super::super::responses::Scope::attach(slot.clone(), Some(fixture.deferred.clone()))?;
    drop(scope);
    assert!(super::super::responses::read(&slot)?.is_none());

    let fresh = Fixture::new(mode()).await?;
    let permit = crate::accounting::WRITES.acquire().await?;
    let mut bootstrap = Box::pin(fresh.resolve());
    assert!(poll!(&mut bootstrap).is_pending());
    drop(bootstrap);
    drop(permit);
    assert!(fresh.deferred.check().is_err());
    assert!(
        fresh
            .rows::<Attempt>("draft_accounting_attempts")
            .await
            .is_err()
    );
    let (mut session, _) = crate::session::tests::make_session_and_context().await;
    session.services.state_db = None;
    let failed = DeferredResponsesSampling::new(Arc::new(session), "missing-state".into(), mode());
    assert!(
        failed
            .resolve(&provider(), Some(&auth()), &format!("{BASE}/responses"))
            .await
            .is_err()
    );
    assert!(failed.check().is_err());
    Ok(())
}
#[tokio::test]
async fn accounting_responses_ws_role_inheritance_preserves_reserved_provider_rule()
-> anyhow::Result<()> {
    let mut config = crate::config::test_config().await;
    config.model_provider_id = "openai".into();
    config.model_provider = provider();
    config.accounting = mode();
    let original = config.accounting.clone();
    let role = config.codex_home.join("ws-role.toml");
    std::fs::create_dir_all(config.codex_home.as_path())?;
    std::fs::write(&role, "developer_instructions = \"fixture role\"\n")?;
    config.agent_roles.insert(
        "fixture".into(),
        crate::config::AgentRoleConfig {
            config_file: Some(role.to_path_buf()),
            description: None,
            nickname_candidates: None,
        },
    );
    crate::agent::role::apply_role_to_config(&mut config, Some("fixture"))
        .await
        .map_err(anyhow::Error::msg)?;
    assert_eq!(config.accounting, original);
    assert_eq!(
        config.developer_instructions.as_deref(),
        Some("fixture role")
    );
    std::fs::write(
        &role,
        "[model_providers.openai]\nname = \"blocked override\"\n",
    )?;
    assert!(
        crate::agent::role::apply_role_to_config(&mut config, Some("fixture"))
            .await
            .is_err()
    );
    assert_eq!(config.accounting, original);
    Ok(())
}
