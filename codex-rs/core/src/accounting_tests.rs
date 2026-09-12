use super::*;
use crate::accounting::Sampling;
use crate::accounting::SamplingScope;
use crate::accounting::transport::AccountingTransport;
use crate::accounting::transport::ResponseEvidence;
use crate::config::AccountingMode;
use codex_api::AnthropicTokenPresence;
use codex_api::AnthropicUsageObserver;
use codex_api::AnthropicUsagePatch;
use codex_protocol::protocol::SessionSource;
use codex_state::SqliteConfig;
use codex_state::StateRuntime;
use codex_state::ThreadMetadataBuilder;
use codex_state::accounting::Attempt;
use codex_utils_absolute_path::AbsolutePathBuf;
use pretty_assertions::assert_eq;
use sqlx::Connection;
use sqlx::SqliteConnection;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use uuid::Uuid;

const ENDPOINT: &str = "http://127.0.0.1:1/v1";

struct DeniedAuth;

impl codex_api::AuthProvider for DeniedAuth {
    fn add_auth_headers(&self, _: &mut http::HeaderMap) {}
    fn apply_auth(&self, _: Request) -> codex_api::AuthProviderFuture<'_> {
        Box::pin(async {
            Err(codex_api::AuthError::Build(
                "synthetic preflight denial".into(),
            ))
        })
    }
}

#[tokio::test]
async fn accounting_awaited_auth_denial_creates_no_attempt_or_transport_send() -> anyhow::Result<()>
{
    let fixture = Fixture::new().await?;
    let sends = Arc::new(AtomicUsize::new(0));
    let evidence = ResponseEvidence::new(fixture.sampling.clone());
    let transport = StageOneGuardedTransport::new(
        AccountingTransport::new(
            Probe {
                sends: sends.clone(),
                fail_first: false,
            },
            Some(evidence.clone()),
            "claude-opus-5".into(),
        ),
        /*binding*/ None,
    );
    let provider = codex_api::Provider {
        name: "anthropic".into(),
        base_url: ENDPOINT.into(),
        query_params: None,
        headers: Default::default(),
        retry: codex_api::RetryConfig {
            max_attempts: 2,
            base_delay: std::time::Duration::ZERO,
            retry_429: true,
            retry_5xx: true,
            retry_transport: true,
        },
        stream_idle_timeout: std::time::Duration::from_secs(1),
    };
    let client = codex_api::AnthropicMessagesClient::new(transport, provider, Arc::new(DeniedAuth))
        .with_usage_observer(Some(evidence));
    let request = codex_api::AnthropicMessagesRequest {
        model: "claude-opus-5".into(),
        system: vec![],
        messages: vec![],
        tools: vec![],
        tool_choice: None,
        stream: true,
        max_tokens: 10,
        thinking: None,
        output_config: None,
        provider_options: None,
    };
    assert!(
        client
            .stream_request(request, Default::default())
            .await
            .is_err()
    );
    assert_eq!(fixture.attempts().await?, vec![]);
    assert_eq!(sends.load(Ordering::SeqCst), 0);
    Ok(())
}

#[tokio::test]
async fn accounting_role_reload_preserves_internal_binding_without_changing_off_route_behavior()
-> anyhow::Result<()> {
    for on in [false, true] {
        let mut config = crate::config::test_config().await;
        config.model_provider_id = "anthropic".into();
        config.model = Some("claude-opus-5".into());
        config.model_provider = ModelProviderInfo {
            base_url: Some(ENDPOINT.into()),
            env_key: None,
            experimental_bearer_token: Some("synthetic".into()),
            ..ModelProviderInfo::create_anthropic_provider()
        };
        if on {
            config.accounting = AccountingMode::DirectAnthropic {
                scope: Uuid::new_v4(),
                approved_endpoint: ENDPOINT.into(),
            };
        }
        let path = config.codex_home.join("accounting-role.toml");
        std::fs::create_dir_all(config.codex_home.as_path())?;
        std::fs::write(&path, "model_reasoning_effort = \"low\"")?;
        config.agent_roles.insert(
            "accounting-role".into(),
            crate::config::AgentRoleConfig {
                config_file: Some(path.to_path_buf()),
                description: Some("fixture".into()),
                nickname_candidates: None,
            },
        );
        let mode = config.accounting.clone();
        let provider = config.model_provider.clone();
        crate::agent::role::apply_role_to_config(&mut config, Some("accounting-role"))
            .await
            .map_err(anyhow::Error::msg)?;
        assert_eq!(config.accounting, mode);
        if on {
            assert_eq!(config.model_provider, provider);
        } else {
            assert_eq!(
                config.model_provider,
                ModelProviderInfo::create_anthropic_provider()
            );
        }
    }
    Ok(())
}

#[tokio::test]
async fn accounting_role_overlay_keeps_explicit_provider_fields_and_validates_conflicts()
-> anyhow::Result<()> {
    let mut config = crate::config::test_config().await;
    config.model_provider_id = "anthropic".into();
    config.model = Some("claude-opus-5".into());
    config.model_provider = ModelProviderInfo {
        base_url: Some(ENDPOINT.into()),
        env_key: None,
        experimental_bearer_token: Some("synthetic-parent".into()),
        http_headers: Some([("x-parent".into(), "preserved".into())].into()),
        ..ModelProviderInfo::create_anthropic_provider()
    };
    config.accounting = AccountingMode::DirectAnthropic {
        scope: Uuid::new_v4(),
        approved_endpoint: ENDPOINT.into(),
    };
    let role = config.codex_home.join("accounting-overlay.toml");
    std::fs::create_dir_all(config.codex_home.as_path())?;
    std::fs::write(
        &role,
        r#"
model_provider = "anthropic"
[model_providers.anthropic]
name = "explicit role"
base_url = "http://127.0.0.1:2/v1"
experimental_bearer_token = "synthetic-child"
wire_api = "anthropic"
request_max_retries = 0
stream_max_retries = 0
stream_idle_timeout_ms = 10000
stream_actionable_timeout_ms = 5000
stream_long_failure_retry_threshold_ms = 2000
stream_long_failure_max_retries = 0
websocket_connect_timeout_ms = 3000
[model_providers.anthropic.http_headers]
x-child = "explicit"
[model_providers.anthropic.runtime_policy]
request_body_max_bytes = 30000000
retry_request_body_max_bytes = 15000000
web_search_max_uses = 2
"#,
    )?;
    config.agent_roles.insert(
        "overlay".into(),
        crate::config::AgentRoleConfig {
            config_file: Some(role.to_path_buf()),
            description: None,
            nickname_candidates: None,
        },
    );
    let mode = config.accounting.clone();
    let mut expected = ModelProviderInfo {
        name: "explicit role".into(),
        base_url: Some("http://127.0.0.1:2/v1".into()),
        experimental_bearer_token: Some("synthetic-child".into()),
        request_max_retries: Some(0),
        stream_max_retries: Some(0),
        stream_idle_timeout_ms: Some(10000),
        stream_actionable_timeout_ms: Some(5000),
        stream_long_failure_retry_threshold_ms: Some(2000),
        stream_long_failure_max_retries: Some(0),
        websocket_connect_timeout_ms: Some(3000),
        http_headers: Some(
            [
                ("x-parent".into(), "preserved".into()),
                ("x-child".into(), "explicit".into()),
            ]
            .into(),
        ),
        ..config.model_provider.clone()
    };
    expected.runtime_policy.web_search_max_uses = Some(2);
    crate::agent::role::apply_role_to_config(&mut config, Some("overlay"))
        .await
        .map_err(anyhow::Error::msg)?;
    assert_eq!(config.model_provider, expected);
    assert_eq!(config.model_providers["anthropic"], expected);
    assert_eq!(config.accounting, mode);
    // A wire bypass or conflicting command credential is rejected, not silently
    // dropped or used to authorize a different route/credential source.
    for overlay in [
        "[model_providers.anthropic]\nname = \"wire override\"\nwire_api = \"responses\"",
        "[model_providers.anthropic]\nname = \"auth override\"\n[model_providers.anthropic.auth]\ncommand = \"never-executed-fixture\"",
    ] {
        std::fs::write(&role, overlay)?;
        assert!(
            crate::agent::role::apply_role_to_config(&mut config, Some("overlay"))
                .await
                .is_err()
        );
        assert_eq!(config.model_provider, expected);
        assert_eq!(config.accounting, mode);
    }
    Ok(())
}

struct Fixture {
    _home: tempfile::TempDir,
    owner: ThreadId,
    db: Arc<StateRuntime>,
    sampling: Arc<Sampling>,
}

impl Fixture {
    async fn new() -> anyhow::Result<Self> {
        let home = tempfile::tempdir()?;
        let db = StateRuntime::init(
            SqliteConfig::from_sqlite_home(AbsolutePathBuf::try_from(home.path().to_path_buf())?),
            "anthropic".into(),
        )
        .await?;
        let owner = ThreadId::new();
        let metadata = ThreadMetadataBuilder::new(
            owner,
            home.path().join("fixture.jsonl"),
            chrono::Utc::now(),
            SessionSource::Cli,
        );
        db.upsert_thread(&metadata.build("anthropic")).await?;
        let sampling = Sampling::start(
            db.clone(),
            owner,
            "turn".into(),
            &AccountingMode::DirectAnthropic {
                scope: Uuid::new_v4(),
                approved_endpoint: ENDPOINT.into(),
            },
        )
        .await?;
        Ok(Self {
            _home: home,
            owner,
            db,
            sampling,
        })
    }

    async fn connection(&self) -> anyhow::Result<SqliteConnection> {
        Ok(SqliteConnection::connect_with(
            &sqlx::sqlite::SqliteConnectOptions::new()
                .filename(self.db.sqlite().state_db_path())
                .foreign_keys(true),
        )
        .await?)
    }

    async fn attempts(&self) -> anyhow::Result<Vec<Attempt>> {
        let values: Vec<String> =
            sqlx::query_scalar("SELECT payload FROM draft_accounting_attempts ORDER BY rowid")
                .fetch_all(&mut self.connection().await?)
                .await?;
        values
            .into_iter()
            .map(|value| serde_json::from_str(&value).map_err(Into::into))
            .collect()
    }
}

struct Probe {
    sends: Arc<AtomicUsize>,
    fail_first: bool,
}

impl HttpTransport for Probe {
    async fn execute(&self, _: Request) -> Result<Response, TransportError> {
        unreachable!("streaming fixture")
    }
    async fn stream(&self, _: Request) -> Result<StreamResponse, TransportError> {
        let number = self.sends.fetch_add(1, Ordering::SeqCst);
        if self.fail_first && number == 0 {
            return Err(TransportError::Timeout);
        }
        Ok(StreamResponse {
            status: http::StatusCode::OK,
            headers: Default::default(),
            bytes: futures::stream::empty().boxed(),
        })
    }
}

fn request() -> Request {
    Request {
        method: http::Method::POST,
        url: format!("{ENDPOINT}/messages"),
        headers: Default::default(),
        body: None,
        compression: Default::default(),
        timeout: None,
    }
}

fn usage(value: i64) -> AnthropicUsagePatch {
    AnthropicUsagePatch {
        input_tokens: AnthropicTokenPresence::Number(value),
        ..Default::default()
    }
}

#[tokio::test]
async fn accounting_actual_transport_retries_and_old_response_keep_distinct_owned_identity()
-> anyhow::Result<()> {
    let fixture = Fixture::new().await?;
    let sends = Arc::new(AtomicUsize::new(0));
    let first = ResponseEvidence::new(fixture.sampling.clone());
    let transport = AccountingTransport::new(
        Probe {
            sends: sends.clone(),
            fail_first: true,
        },
        Some(first.clone()),
        "claude-opus-5".into(),
    );
    assert!(transport.stream(request()).await.is_err());
    transport.stream(request()).await?;
    first.observe(1, Ok(usage(7))).await?;
    let second = ResponseEvidence::new(fixture.sampling.clone());
    AccountingTransport::new(
        Probe {
            sends: sends.clone(),
            fail_first: false,
        },
        Some(second.clone()),
        "claude-opus-5".into(),
    )
    .stream(request())
    .await?;
    second.observe(1, Ok(usage(11))).await?;
    first.observe(2, Ok(usage(8))).await?;
    let attempts = fixture.attempts().await?;
    assert_eq!(attempts.len(), 3);
    assert_eq!(
        attempts
            .iter()
            .map(|attempt| (
                attempt.request_id,
                attempt.thread_id,
                attempt.turn.clone(),
                attempt.retry_of
            ))
            .collect::<Vec<_>>(),
        vec![
            (
                attempts[0].request_id,
                attempts[0].thread_id,
                "turn".into(),
                None
            ),
            (
                attempts[0].request_id,
                attempts[0].thread_id,
                "turn".into(),
                Some(attempts[0].attempt_id)
            ),
            (
                attempts[0].request_id,
                attempts[0].thread_id,
                "turn".into(),
                Some(attempts[1].attempt_id)
            )
        ]
    );
    let rows: Vec<(String, i64, String)> = sqlx::query_as("SELECT attempt_id, revision, json_extract(payload, '$.patch') FROM draft_accounting_observations ORDER BY rowid")
        .fetch_all(&mut fixture.connection().await?).await?;
    assert_eq!(
        rows,
        vec![
            (
                attempts[1].attempt_id.to_string(),
                1,
                r#"{"input":7}"#.into()
            ),
            (
                attempts[2].attempt_id.to_string(),
                1,
                r#"{"input":11}"#.into()
            ),
            (
                attempts[1].attempt_id.to_string(),
                2,
                r#"{"input":8}"#.into()
            ),
        ]
    );
    assert_eq!(sends.load(Ordering::SeqCst), 3);
    Ok(())
}

#[tokio::test]
async fn accounting_writer_barrier_prevents_send_until_durable_admission() -> anyhow::Result<()> {
    let fixture = Fixture::new().await?;
    let mut conn = fixture.connection().await?;
    let transaction = conn.begin_with("BEGIN IMMEDIATE").await?;
    let sends = Arc::new(AtomicUsize::new(0));
    let transport = AccountingTransport::new(
        Probe {
            sends: sends.clone(),
            fail_first: false,
        },
        Some(ResponseEvidence::new(fixture.sampling.clone())),
        "claude-opus-5".into(),
    );
    let task = tokio::spawn(async move { transport.stream(request()).await });
    tokio::time::sleep(std::time::Duration::from_millis(40)).await;
    assert!(!task.is_finished());
    assert_eq!(sends.load(Ordering::SeqCst), 0);
    assert_eq!(fixture.attempts().await?, vec![]);
    transaction.commit().await?;
    task.await??;
    assert_eq!(
        (
            fixture.attempts().await?.len(),
            sends.load(Ordering::SeqCst)
        ),
        (1, 1)
    );
    Ok(())
}

#[tokio::test]
async fn accounting_guard_denial_precedes_admission_and_never_sends() -> anyhow::Result<()> {
    let fixture = Fixture::new().await?;
    let sends = Arc::new(AtomicUsize::new(0));
    let binding = Arc::new(StageOneMemoryBinding {
        owner: Weak::new(),
        termination: futures::future::pending().boxed().shared(),
        owner_id: ThreadId::new(),
        provider: ModelProviderInfo::create_anthropic_provider(),
        floor: SecurityLevel::Permissive,
        runtime_nonce: [0; 16],
        session_id: "fixture".into(),
        denial: Mutex::new(None),
    });
    let transport = StageOneGuardedTransport::new(
        AccountingTransport::new(
            Probe {
                sends: sends.clone(),
                fail_first: false,
            },
            Some(ResponseEvidence::new(fixture.sampling.clone())),
            "claude-opus-5".into(),
        ),
        Some(binding),
    );
    assert!(transport.stream(request()).await.is_err());
    assert_eq!(
        (
            fixture.attempts().await?.len(),
            sends.load(Ordering::SeqCst)
        ),
        (0, 0)
    );
    assert!(fixture.sampling.check().is_ok());
    Ok(())
}

#[tokio::test]
async fn accounting_admission_failure_and_route_mismatch_are_sticky_without_send()
-> anyhow::Result<()> {
    for fault in ["admission", "route", "missing_owner"] {
        let fixture = Fixture::new().await?;
        if fault == "admission" {
            sqlx::query("CREATE TRIGGER reject_admission BEFORE INSERT ON draft_accounting_attempts BEGIN SELECT RAISE(ABORT, 'fixture-admission'); END")
                .execute(&mut fixture.connection().await?).await?;
        }
        if fault == "missing_owner" {
            fixture.db.delete_thread(fixture.owner).await?;
        }
        let sends = Arc::new(AtomicUsize::new(0));
        let transport = AccountingTransport::new(
            Probe {
                sends: sends.clone(),
                fail_first: false,
            },
            Some(ResponseEvidence::new(fixture.sampling.clone())),
            "claude-opus-5".into(),
        );
        let mut outbound = request();
        if fault == "route" {
            outbound.url = "http://127.0.0.1:2/v1/messages".into();
        }
        assert!(transport.stream(outbound).await.is_err());
        assert!(transport.stream(request()).await.is_err());
        assert_eq!(
            (
                fixture.attempts().await?.len(),
                sends.load(Ordering::SeqCst)
            ),
            (0, 0)
        );
        let error = fixture.sampling.check().unwrap_err();
        assert!(!error.is_retryable());
        assert_eq!(
            error.to_string(),
            format!("Fatal error: {}", crate::accounting::FAILURE)
        );
    }
    Ok(())
}

#[tokio::test]
async fn accounting_scope_drop_clears_cancelled_sampling_and_observation_failure_is_fatal()
-> anyhow::Result<()> {
    let fixture = Fixture::new().await?;
    let slot = Default::default();
    let scope = SamplingScope::attach(Arc::clone(&slot), Some(fixture.sampling.clone()));
    assert!(slot.lock().unwrap().is_some());
    drop(scope);
    assert!(slot.lock().unwrap().is_none());
    let evidence = ResponseEvidence::new(fixture.sampling.clone());
    let sends = Arc::new(AtomicUsize::new(0));
    AccountingTransport::new(
        Probe {
            sends: sends.clone(),
            fail_first: false,
        },
        Some(evidence.clone()),
        "claude-opus-5".into(),
    )
    .stream(request())
    .await?;
    evidence.observe(1, Ok(usage(7))).await?;
    sqlx::query("CREATE TRIGGER reject_observation BEFORE INSERT ON draft_accounting_observations BEGIN SELECT RAISE(ABORT, 'fixture-observation'); END")
        .execute(&mut fixture.connection().await?).await?;
    assert!(evidence.observe(2, Ok(usage(9))).await.is_err());
    assert!(!fixture.sampling.check().unwrap_err().is_retryable());
    let rows: Vec<String> = sqlx::query_scalar(
        "SELECT json_extract(payload, '$.patch') FROM draft_accounting_observations",
    )
    .fetch_all(&mut fixture.connection().await?)
    .await?;
    assert_eq!(rows, vec![r#"{"input":7}"#]);
    assert_eq!(sends.load(Ordering::SeqCst), 1);
    Ok(())
}
