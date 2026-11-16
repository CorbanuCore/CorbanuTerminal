//! D635: exercise discovery below config's reserved OpenAI-name/URL validation.

use super::*;
use codex_arg0::Arg0DispatchPaths;
use codex_config::CloudConfigBundleLoader;
use codex_config::LoaderOverrides;
use codex_login::CodexAuth;
use codex_model_provider_info::ModelProviderInfo;
use codex_protocol::openai_models::ModelVisibility;
use codex_protocol::openai_models::ModelsResponse;
use pretty_assertions::assert_eq;
use tempfile::TempDir;
use test_case::test_case;
use wiremock::Mock;
use wiremock::MockServer;
use wiremock::ResponseTemplate;
use wiremock::matchers::method;
use wiremock::matchers::path;

async fn discovery_config(
    manager: &ConfigManager,
    provider: &ModelProviderInfo,
) -> std::io::Result<Arc<Config>> {
    let mut config = manager
        .load_startup_config(/*fallback_cwd*/ None)
        .await?
        .config;
    // Only this in-memory fixture bypasses validation. A custom TOML provider
    // named OpenAI remains forbidden, and real OpenAI catalog URLs stay restricted.
    config.model_provider = provider.clone();
    Ok(Arc::new(config))
}

fn discovery_manager(config: &Config, auth: Arc<AuthManager>) -> SharedModelsManager {
    let provider =
        codex_model_provider::create_model_provider(config.model_provider.clone(), Some(auth));
    let manager = provider.models_manager(
        config.codex_home.to_path_buf(),
        config.model_catalog.clone(),
    );
    manager.set_api_key_model_discovery_enabled(
        config.features.enabled(Feature::ApiKeyModelDiscovery),
    );
    manager
}

#[test_case(None, None, true; "default on")]
#[test_case(None, Some(false), true; "app rollback")]
#[test_case(None, Some(true), true; "app rollout")]
#[test_case(Some(false), Some(true), true; "user opt out")]
#[test_case(Some(true), Some(false), true; "user opt in")]
#[test_case(Some(true), Some(false), false; "base URL without catalog opt in")]
#[tokio::test]
async fn api_key_model_discovery_startup_enablement_respects_user_config(
    user_enablement: Option<bool>,
    app_enablement: Option<bool>,
    catalog_opt_in: bool,
) -> anyhow::Result<()> {
    let server = MockServer::start().await;
    let mut remote_model = codex_models_manager::bundled_models_response()?
        .models
        .remove(0);
    remote_model.slug = "rollout-model".into();
    remote_model.visibility = ModelVisibility::List;
    remote_model.supported_in_api = true;
    Mock::given(method("GET"))
        .and(path("/v1/models"))
        .respond_with(
            ResponseTemplate::new(/*s*/ 200).set_body_json(ModelsResponse {
                models: vec![remote_model.clone()],
            }),
        )
        .mount(&server)
        .await;
    let home = TempDir::new()?;
    let feature_config = user_enablement
        .map(|enabled| format!("[features]\napi_key_model_discovery = {enabled}\n"))
        .unwrap_or_default();
    std::fs::write(
        home.path().join("config.toml"),
        format!("model_provider = \"openai\"\n{feature_config}"),
    )?;
    let config_manager = ConfigManager::new(
        home.path().to_path_buf(),
        Vec::new(),
        LoaderOverrides::without_managed_config_for_tests(),
        /*strict_config*/ true,
        CloudConfigBundleLoader::default(),
        Arg0DispatchPaths::default(),
        Arc::new(codex_config::NoopThreadConfigLoader),
    );
    let provider = ModelProviderInfo {
        name: "OpenAI".into(),
        base_url: Some(format!("{}/v1", server.uri())),
        model_catalog_url: catalog_opt_in.then(|| format!("{}/v1/models", server.uri()).into()),
        requires_openai_auth: true,
        ..ModelProviderInfo::default()
    };
    let auth = AuthManager::from_auth_for_testing(CodexAuth::from_api_key("test-key"));
    let config = discovery_config(&config_manager, &provider).await?;
    let startup_manager = discovery_manager(&config, Arc::clone(&auth));
    let catalog = ModelCatalog::new(
        config_manager.clone(),
        /*startup_cwd*/ None,
        Arc::clone(&config),
        Arc::clone(&startup_manager),
        Arc::clone(&auth),
    );
    catalog.list_models(RefreshStrategy::Online).await?;

    // Exercise a runtime setting before the account transition replaces the manager.
    if let Some(enabled) = app_enablement {
        config_manager
            .extend_runtime_feature_enablement([("api_key_model_discovery".to_string(), !enabled)])
            .expect("runtime feature lock");
        let config = discovery_config(&config_manager, &provider).await?;
        catalog
            .set_api_key_model_discovery_enabled(
                config.features.enabled(Feature::ApiKeyModelDiscovery),
            )
            .await;
    }
    let provider = ModelProviderInfo {
        name: "After login".into(),
        ..provider
    };
    let config = discovery_config(&config_manager, &provider).await?;
    let replacement = discovery_manager(&config, Arc::clone(&auth));
    assert!(!Arc::ptr_eq(&startup_manager, &replacement));
    let catalog = ModelCatalog::new(
        config_manager.clone(),
        /*startup_cwd*/ None,
        config,
        replacement,
        auth,
    );
    catalog.list_models(RefreshStrategy::Online).await?;

    if let Some(enabled) = app_enablement {
        config_manager
            .extend_runtime_feature_enablement([("api_key_model_discovery".to_string(), enabled)])
            .expect("runtime feature lock");
    }
    let config = discovery_config(&config_manager, &provider).await?;
    catalog
        .set_api_key_model_discovery_enabled(config.features.enabled(Feature::ApiKeyModelDiscovery))
        .await;
    let actual = catalog.list_models(RefreshStrategy::Online).await?;
    let enabled = user_enablement.or(app_enablement).unwrap_or(true) && catalog_opt_in;
    let mut expected = if enabled {
        vec![ModelPreset::from(remote_model)]
    } else if catalog_opt_in {
        // An explicit catalog suppresses the bundled list even when discovery is off.
        Vec::new()
    } else {
        let mut bundled = codex_models_manager::bundled_models_response()?.models;
        bundled.sort_by_key(|model| model.priority);
        ModelPreset::filter_by_auth(
            bundled.into_iter().map(Into::into).collect(),
            /*chatgpt_mode*/ false,
        )
    };
    ModelPreset::mark_default_by_picker_visibility(&mut expected);
    // The product's OpenAI default wins when present in the bundled catalog.
    if let Some(index) = expected
        .iter()
        .position(|preset| preset.model == "gpt-6-sol" && preset.show_in_picker)
    {
        for preset in &mut expected {
            preset.is_default = false;
        }
        expected[index].is_default = true;
    }
    assert_eq!(actual, expected);
    let requests = server.received_requests().await.expect("request recording");
    if user_enablement == Some(false) || !catalog_opt_in {
        assert!(requests.is_empty(), "startup opt-out must never fetch");
    } else {
        assert!(
            !requests.is_empty(),
            "discovery must actually reach the catalog"
        );
    }
    Ok(())
}
