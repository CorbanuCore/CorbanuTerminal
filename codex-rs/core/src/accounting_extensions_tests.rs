//! The seam extensions use to record their own model requests.
use super::*;
use codex_features::Feature;
use codex_state::SqliteConfig;
use codex_state::StateRuntime;
use codex_state::ThreadMetadataBuilder;
use codex_utils_absolute_path::AbsolutePathBuf;
use pretty_assertions::assert_eq;

struct FixtureAuth;
impl codex_api::AuthProvider for FixtureAuth {
    fn add_auth_headers(&self, _: &mut http::HeaderMap) {}
}

/// Image generation is billed model inference that ships on by default, and it
/// reached no ledger: the extension builds its own client, so there is no turn
/// to attach to. This is that request, recorded.
#[tokio::test]
async fn accounting_extension_client_records_its_own_request() -> anyhow::Result<()> {
    let server = wiremock::MockServer::start().await;
    let endpoint = format!("{}/v1", server.uri());
    wiremock::Mock::given(wiremock::matchers::method("POST"))
        .and(wiremock::matchers::path("/v1/images/generations"))
        .respond_with(
            wiremock::ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "created": 0,
                "data": [{"b64_json": "aW1hZ2U="}],
                "usage": {"input_tokens": 100, "input_tokens_details": {"cached_tokens": 20},
                    "output_tokens": 40, "total_tokens": 140}
            })),
        )
        .mount(&server)
        .await;

    let home = tempfile::tempdir()?;
    let mut config = crate::session::tests::build_test_config(home.path()).await;
    config.model_provider = codex_model_provider_info::ModelProviderInfo {
        request_max_retries: Some(0),
        stream_max_retries: Some(0),
        supports_websockets: false,
        ..codex_model_provider_info::ModelProviderInfo::create_openai_provider(Some(
            endpoint.clone(),
        ))
    };
    config.model_provider_id = "openai".into();
    config.accounting = crate::config::AccountingMode::Provider {
        scope: Uuid::new_v4(),
        provider_id: "openai".into(),
        wire_api: codex_model_provider_info::WireApi::Responses,
        approved_endpoint: endpoint.clone(),
        approved_query: None,
        pricing: crate::config::PriceAuthority::Unavailable,
    };
    config.features.enable(Feature::Sqlite)?;
    let provider = config.model_provider.clone();

    let (mut session, _context) =
        crate::session::tests::make_session_and_context_for_config(config.clone()).await;
    let db = StateRuntime::init(
        SqliteConfig::from_sqlite_home(AbsolutePathBuf::try_from(home.path().to_path_buf())?),
        "openai".into(),
    )
    .await?;
    session.services.state_db = Some(db.clone());
    let owner = Arc::new(session);
    db.upsert_thread(
        &ThreadMetadataBuilder::new(
            owner.thread_id,
            home.path().join("image-fixture.jsonl"),
            chrono::Utc::now(),
            codex_protocol::protocol::SessionSource::Cli,
        )
        .build("openai"),
    )
    .await?;

    let accounting = ExtensionAccounting::new(
        Arc::downgrade(&owner),
        config.accounting.clone(),
        "openai".into(),
    );
    let transport = accounting
        .transport(
            codex_api::ReqwestTransport::from_http_client(
                codex_login::default_client::create_client(),
            ),
            &provider,
            "gpt-image-1",
            "images/generations",
            "image",
        )
        .await;
    let api = provider.to_api_provider(Some(codex_protocol::auth::AuthMode::ApiKey))?;
    codex_api::ImagesClient::new(transport, api, Arc::new(FixtureAuth))
        .generate(
            &codex_api::ImageGenerationRequest {
                prompt: "fixture".into(),
                background: None,
                model: "gpt-image-1".into(),
                n: None,
                quality: None,
                size: None,
            },
            http::HeaderMap::new(),
        )
        .await
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;

    let pool = db
        .sqlite()
        .open_read_only_pool(&db.sqlite().state_db_path())
        .await?;
    let attempts: Vec<String> =
        sqlx::query_scalar("SELECT payload FROM draft_accounting_attempts ORDER BY rowid")
            .fetch_all(&pool)
            .await?;
    let turns: Vec<String> = attempts
        .iter()
        .map(|row| {
            serde_json::from_str::<serde_json::Value>(row).expect("attempt payload")["turn"]
                .as_str()
                .expect("turn identity")
                .to_string()
        })
        .collect();
    assert_eq!(
        turns
            .iter()
            .filter(|turn| turn.starts_with("image:"))
            .count(),
        1,
        "turns recorded: {turns:?}"
    );
    let models: Vec<String> = attempts
        .iter()
        .map(|row| {
            serde_json::from_str::<serde_json::Value>(row).expect("attempt payload")["model"]
                .as_str()
                .expect("model")
                .to_string()
        })
        .collect();
    assert_eq!(models, vec!["gpt-image-1".to_string()]);
    // The images response carries its own usage, so the tokens are recorded
    // rather than left unknown.
    let observations: i64 =
        sqlx::query_scalar("SELECT count(*) FROM draft_accounting_observations")
            .fetch_one(&pool)
            .await?;
    assert_eq!(observations, 1);
    Ok(())
}

/// A handle whose session has gone away records nothing, and does not fail the
/// extension's request: the owner is held weakly on purpose.
#[tokio::test]
async fn accounting_extension_client_without_its_session_records_nothing() -> anyhow::Result<()> {
    let server = wiremock::MockServer::start().await;
    let endpoint = format!("{}/v1", server.uri());
    wiremock::Mock::given(wiremock::matchers::method("POST"))
        .and(wiremock::matchers::path("/v1/images/generations"))
        .respond_with(
            wiremock::ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "created": 0,
                "data": [{"b64_json": "aW1hZ2U="}],
                "usage": {"input_tokens": 100, "output_tokens": 40, "total_tokens": 140}
            })),
        )
        .mount(&server)
        .await;

    let home = tempfile::tempdir()?;
    let mut config = crate::session::tests::build_test_config(home.path()).await;
    config.model_provider = codex_model_provider_info::ModelProviderInfo {
        request_max_retries: Some(0),
        stream_max_retries: Some(0),
        supports_websockets: false,
        ..codex_model_provider_info::ModelProviderInfo::create_openai_provider(Some(
            endpoint.clone(),
        ))
    };
    config.model_provider_id = "openai".into();
    config.accounting = crate::config::AccountingMode::Provider {
        scope: Uuid::new_v4(),
        provider_id: "openai".into(),
        wire_api: codex_model_provider_info::WireApi::Responses,
        approved_endpoint: endpoint.clone(),
        approved_query: None,
        pricing: crate::config::PriceAuthority::Unavailable,
    };
    config.features.enable(Feature::Sqlite)?;
    let provider = config.model_provider.clone();

    let (mut session, _context) =
        crate::session::tests::make_session_and_context_for_config(config.clone()).await;
    let db = StateRuntime::init(
        SqliteConfig::from_sqlite_home(AbsolutePathBuf::try_from(home.path().to_path_buf())?),
        "openai".into(),
    )
    .await?;
    session.services.state_db = Some(db.clone());
    let owner = Arc::new(session);
    let accounting = ExtensionAccounting::new(
        Arc::downgrade(&owner),
        config.accounting.clone(),
        "openai".into(),
    );
    drop(owner);

    let transport = accounting
        .transport(
            codex_api::ReqwestTransport::from_http_client(
                codex_login::default_client::create_client(),
            ),
            &provider,
            "gpt-image-1",
            "images/generations",
            "image",
        )
        .await;
    let api = provider.to_api_provider(Some(codex_protocol::auth::AuthMode::ApiKey))?;
    // The request still succeeds; only the recording is withheld.
    codex_api::ImagesClient::new(transport, api, Arc::new(FixtureAuth))
        .generate(
            &codex_api::ImageGenerationRequest {
                prompt: "fixture".into(),
                background: None,
                model: "gpt-image-1".into(),
                n: None,
                quality: None,
                size: None,
            },
            http::HeaderMap::new(),
        )
        .await
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    let pool = db
        .sqlite()
        .open_read_only_pool(&db.sqlite().state_db_path())
        .await?;
    let installed: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM sqlite_master WHERE name = 'draft_accounting_attempts'",
    )
    .fetch_one(&pool)
    .await?;
    assert_eq!(installed, 0, "a session-less handle installed accounting");
    Ok(())
}
