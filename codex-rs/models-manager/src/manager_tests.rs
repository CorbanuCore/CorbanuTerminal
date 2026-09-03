use super::*;
use crate::ModelsManagerConfig;
use chrono::Utc;
use codex_http_client::HttpClientFactory;
use codex_http_client::OutboundProxyPolicy;
use codex_login::AuthCredentialsStoreMode;
use codex_login::AuthKeyringBackendKind;
use codex_login::AuthManager;
use codex_login::CodexAuth;
use codex_login::ExternalAuth;
use codex_login::ExternalAuthRefreshContext;
use codex_login::TokenData;
use codex_protocol::auth::AuthMode;
use codex_protocol::openai_models::ChatReasoningEffortProtocol;
use codex_protocol::openai_models::ChatReasoningProtocol;
use codex_protocol::openai_models::InputModality;
use codex_protocol::openai_models::ModelBilling;
use codex_protocol::openai_models::ModelCapabilityTier;
use codex_protocol::openai_models::ModelOrchestrationMetadata;
use codex_protocol::openai_models::ModelVisibility;
use codex_protocol::openai_models::ModelsResponse;
use codex_protocol::openai_models::ReasoningEffort;
use codex_protocol::openai_models::ReasoningEffortPreset;
use pretty_assertions::assert_eq;
use serde_json::json;
use std::collections::VecDeque;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use tempfile::tempdir;

#[path = "model_info_overrides_tests.rs"]
mod model_info_overrides_tests;

const DEFAULT_HTTP_CLIENT_FACTORY: HttpClientFactory =
    HttpClientFactory::new(OutboundProxyPolicy::ReqwestDefault);
const STANDARD_BASE: &str = include_str!("../../core/src/agent/builtins/standard_base.md");
const STANDARD_BASE_OUTCOME_MARKER: &str = "inspect code before changing it, keep edits scoped";
const STANDARD_BASE_EVIDENCE_MARKER: &str = "only narrate when needed";
const OLD_STANDARD_BASE_MARKER: &str = "Narrate as you work";
const GPT55_GUIDE_MARKER: &str = "vivid inner life";

fn assert_standard_base(actual: &str) {
    assert_eq!(actual.trim_end(), STANDARD_BASE.trim_end());
    assert!(actual.contains("corbanu vault auth-helper <label>"));
    assert!(actual.contains("Never run the helper by itself"));
    assert!(!actual.contains("whitelisted provider"));
}

fn remote_model(slug: &str, display: &str, priority: i32) -> ModelInfo {
    remote_model_with_visibility(slug, display, priority, "list")
}

fn remote_model_with_visibility(
    slug: &str,
    display: &str,
    priority: i32,
    visibility: &str,
) -> ModelInfo {
    serde_json::from_value(json!({
            "slug": slug,
            "display_name": display,
            "description": format!("{display} desc"),
            "default_reasoning_level": "medium",
            "supported_reasoning_levels": [{"effort": "low", "description": "low"}, {"effort": "medium", "description": "medium"}],
            "shell_type": "shell_command",
            "visibility": visibility,
            "minimal_client_version": [0, 1, 0],
            "supported_in_api": true,
            "priority": priority,
            "upgrade": null,
            "base_instructions": "base instructions",
            "support_verbosity": false,
            "default_verbosity": null,
            "apply_patch_tool_type": null,
            "truncation_policy": {"mode": "bytes", "limit": 10_000},
            "supports_parallel_tool_calls": false,
            "supports_image_detail_original": false,
            "context_window": 272_000,
            "max_context_window": 272_000,
            "experimental_supported_tools": [],
        }))
        .expect("valid model")
}

fn assert_models_contain(actual: &[ModelInfo], expected: &[ModelInfo]) {
    for model in expected {
        assert!(
            actual.iter().any(|candidate| candidate.slug == model.slug),
            "expected model {} in cached list",
            model.slug
        );
    }
}

#[derive(Debug)]
struct TestModelsEndpoint {
    has_command_auth: bool,
    uses_codex_backend: bool,
    responses: Mutex<VecDeque<Vec<ModelInfo>>>,
    fetch_count: AtomicUsize,
    observed_proxy_policy: Mutex<Option<OutboundProxyPolicy>>,
}

impl TestModelsEndpoint {
    fn new(responses: Vec<Vec<ModelInfo>>) -> Arc<Self> {
        Arc::new(Self {
            has_command_auth: false,
            uses_codex_backend: true,
            responses: Mutex::new(responses.into()),
            fetch_count: AtomicUsize::new(0),
            observed_proxy_policy: Mutex::new(None),
        })
    }

    fn without_refresh(responses: Vec<Vec<ModelInfo>>) -> Arc<Self> {
        Arc::new(Self {
            has_command_auth: false,
            uses_codex_backend: false,
            responses: Mutex::new(responses.into()),
            fetch_count: AtomicUsize::new(0),
            observed_proxy_policy: Mutex::new(None),
        })
    }

    fn fetch_count(&self) -> usize {
        self.fetch_count.load(Ordering::SeqCst)
    }

    fn observed_proxy_policy(&self) -> Option<OutboundProxyPolicy> {
        *self
            .observed_proxy_policy
            .lock()
            .expect("observed proxy policy lock should not be poisoned")
    }

    async fn list_models(&self) -> CoreResult<(Vec<ModelInfo>, Option<String>)> {
        self.fetch_count.fetch_add(1, Ordering::SeqCst);
        let models = self
            .responses
            .lock()
            .expect("responses lock should not be poisoned")
            .pop_front()
            .unwrap_or_default();
        Ok((models, None))
    }
}

#[derive(Debug)]
struct TestExternalApiKeyAuth;

impl ExternalAuth for TestExternalApiKeyAuth {
    fn resolve(&self) -> codex_login::ExternalAuthFuture<'_, CodexAuth> {
        Box::pin(async { Ok(CodexAuth::from_api_key("test-external-api-key")) })
    }

    fn refresh(
        &self,
        _context: ExternalAuthRefreshContext,
    ) -> codex_login::ExternalAuthFuture<'_, CodexAuth> {
        Box::pin(async { Ok(CodexAuth::from_api_key("test-external-api-key")) })
    }
}

#[derive(Debug)]
struct TestUnresolvedExternalApiKeyAuth;

impl ExternalAuth for TestUnresolvedExternalApiKeyAuth {
    fn resolve(&self) -> codex_login::ExternalAuthFuture<'_, CodexAuth> {
        Box::pin(async { Err(std::io::Error::other("unresolved test auth")) })
    }

    fn refresh(
        &self,
        _context: ExternalAuthRefreshContext,
    ) -> codex_login::ExternalAuthFuture<'_, CodexAuth> {
        Box::pin(async { Err(std::io::Error::other("unresolved test auth")) })
    }
}

impl ModelsEndpointClient for TestModelsEndpoint {
    fn has_command_auth(&self) -> bool {
        self.has_command_auth
    }

    fn uses_codex_backend(&self) -> ModelsEndpointFuture<'_, bool> {
        Box::pin(async { self.uses_codex_backend })
    }

    fn list_models<'a>(
        &'a self,
        _client_version: &'a str,
        http_client_factory: HttpClientFactory,
    ) -> ModelsEndpointFuture<'a, CoreResult<(Vec<ModelInfo>, Option<String>)>> {
        Box::pin(async move {
            *self
                .observed_proxy_policy
                .lock()
                .expect("observed proxy policy lock should not be poisoned") =
                Some(http_client_factory.outbound_proxy_policy());
            TestModelsEndpoint::list_models(self).await
        })
    }
}

fn openai_manager_for_tests(
    codex_home: std::path::PathBuf,
    endpoint_client: Arc<dyn ModelsEndpointClient>,
) -> OpenAiModelsManager {
    openai_manager_for_tests_with_auth(
        codex_home,
        endpoint_client,
        Some(AuthManager::from_auth_for_testing(
            CodexAuth::create_dummy_chatgpt_auth_for_testing(),
        )),
    )
}

fn openai_manager_for_tests_with_auth(
    codex_home: std::path::PathBuf,
    endpoint_client: Arc<dyn ModelsEndpointClient>,
    auth_manager: Option<Arc<AuthManager>>,
) -> OpenAiModelsManager {
    OpenAiModelsManager::new(codex_home, endpoint_client, auth_manager)
}

fn static_manager_for_tests(model_catalog: ModelsResponse) -> StaticModelsManager {
    StaticModelsManager::new(/*auth_manager*/ None, model_catalog)
}

#[tokio::test]
async fn manager_without_cache_fetches_on_every_refresh() {
    let remote_models = vec![remote_model("remote", "Remote", /*priority*/ 0)];
    let endpoint = TestModelsEndpoint::new(vec![remote_models.clone(), remote_models.clone()]);
    let manager = OpenAiModelsManager::new_without_cache(
        endpoint.clone(),
        Some(AuthManager::from_auth_for_testing(
            CodexAuth::create_dummy_chatgpt_auth_for_testing(),
        )),
    );

    let catalog = manager
        .raw_model_catalog(
            RefreshStrategy::OnlineIfUncached,
            DEFAULT_HTTP_CLIENT_FACTORY,
        )
        .await;
    let second_catalog = manager
        .raw_model_catalog(
            RefreshStrategy::OnlineIfUncached,
            DEFAULT_HTTP_CLIENT_FACTORY,
        )
        .await;

    assert_models_contain(&catalog.models, &remote_models);
    assert_models_contain(
        &catalog.models,
        &load_remote_models_from_file().expect("bundled models should parse"),
    );
    assert_eq!(second_catalog, catalog);
    assert_eq!(manager.get_remote_models().await, catalog.models);
    assert_eq!(endpoint.fetch_count(), 2);
}

async fn chatgpt_auth_tokens_for_tests(codex_home: &Path) -> CodexAuth {
    let auth_dot_json = codex_login::AuthDotJson {
        auth_mode: Some(AuthMode::ChatgptAuthTokens),
        openai_api_key: None,
        tokens: Some(TokenData {
            id_token: codex_login::token_data::parse_chatgpt_jwt_claims(
                "eyJhbGciOiJub25lIiwidHlwIjoiSldUIn0.\
eyJlbWFpbCI6InVzZXJAZXhhbXBsZS5jb20iLCJodHRwczovL2FwaS5vcGVuYWkuY29tL2F1dGgiOnsiY2hhdGdwdF9wbGFuX3R5cGUiOiJwcm8iLCJjaGF0Z3B0X3VzZXJfaWQiOiJ1c2VyLWlkIiwiY2hhdGdwdF9hY2NvdW50X2lkIjoiYWNjb3VudC1pZCJ9fQ.\
c2ln",
            )
            .expect("fake id token should parse"),
            access_token: "Access Token".to_string(),
            refresh_token: "test".to_string(),
            account_id: Some("account_id".to_string()),
        }),
        last_refresh: Some(Utc::now()),
        agent_identity: None,
        personal_access_token: None,
        bedrock_api_key: None,
    };
    std::fs::create_dir_all(codex_home).expect("codex home should be created");
    std::fs::write(
        codex_home.join("auth.json"),
        serde_json::to_string(&auth_dot_json).expect("auth should serialize"),
    )
    .expect("auth.json should be written");

    CodexAuth::from_auth_storage(
        codex_home,
        AuthCredentialsStoreMode::File,
        /*chatgpt_base_url*/ None,
        AuthKeyringBackendKind::default(),
        &codex_login::test_support::transport_default_auth_route_config(),
    )
    .await
    .expect("auth should load")
    .expect("auth should be present")
}

#[tokio::test]
async fn static_manager_preserves_supported_requested_model_when_fallback_is_allowed() {
    let manager = static_manager_for_tests(ModelsResponse {
        models: vec![
            remote_model("provider-default", "Default", /*priority*/ 0),
            remote_model("provider-supported", "Supported", /*priority*/ 1),
        ],
    });
    let requested_model = Some("provider-supported".to_string());

    let model = manager
        .get_default_model(
            &requested_model,
            /*allow_provider_model_fallback*/ true,
            RefreshStrategy::Offline,
            DEFAULT_HTTP_CLIENT_FACTORY,
        )
        .await;

    assert_eq!(model, "provider-supported");
}

#[tokio::test]
async fn static_manager_falls_back_from_unsupported_requested_model_when_allowed() {
    let manager = static_manager_for_tests(ModelsResponse {
        models: vec![
            remote_model("provider-default", "Default", /*priority*/ 0),
            remote_model("provider-supported", "Supported", /*priority*/ 1),
        ],
    });
    let requested_model = Some("unsupported".to_string());

    let model = manager
        .get_default_model(
            &requested_model,
            /*allow_provider_model_fallback*/ true,
            RefreshStrategy::Offline,
            DEFAULT_HTTP_CLIENT_FACTORY,
        )
        .await;

    assert_eq!(model, "provider-default");
}

#[tokio::test]
async fn static_manager_preserves_unsupported_requested_model_when_fallback_is_disabled() {
    let manager = static_manager_for_tests(ModelsResponse {
        models: vec![remote_model(
            "provider-default",
            "Default",
            /*priority*/ 0,
        )],
    });
    let requested_model = Some("unsupported".to_string());

    let model = manager
        .get_default_model(
            &requested_model,
            /*allow_provider_model_fallback*/ false,
            RefreshStrategy::Offline,
            DEFAULT_HTTP_CLIENT_FACTORY,
        )
        .await;

    assert_eq!(model, "unsupported");
}

#[tokio::test]
async fn static_manager_uses_empty_default_when_fallback_is_allowed_and_catalog_is_empty() {
    let manager = static_manager_for_tests(ModelsResponse { models: Vec::new() });
    let requested_model = Some("unsupported".to_string());

    let model = manager
        .get_default_model(
            &requested_model,
            /*allow_provider_model_fallback*/ true,
            RefreshStrategy::Offline,
            DEFAULT_HTTP_CLIENT_FACTORY,
        )
        .await;

    assert_eq!(model, "");
}

#[tokio::test]
async fn dynamic_manager_preserves_requested_model_when_fallback_is_allowed() {
    let codex_home = tempdir().expect("temp dir");
    let endpoint = TestModelsEndpoint::new(Vec::new());
    let manager = openai_manager_for_tests(codex_home.path().to_path_buf(), endpoint.clone());
    let requested_model = Some("unsupported".to_string());

    let model = manager
        .get_default_model(
            &requested_model,
            /*allow_provider_model_fallback*/ true,
            RefreshStrategy::Online,
            DEFAULT_HTTP_CLIENT_FACTORY,
        )
        .await;

    assert_eq!(model, "unsupported");
    assert_eq!(endpoint.fetch_count(), 0);
}

#[tokio::test]
async fn get_model_info_tracks_fallback_usage() {
    let codex_home = tempdir().expect("temp dir");
    let config = ModelsManagerConfig::default();
    let manager = openai_manager_for_tests(
        codex_home.path().to_path_buf(),
        TestModelsEndpoint::new(Vec::new()),
    );
    let known_slug = manager
        .get_remote_models()
        .await
        .first()
        .expect("bundled models should include at least one model")
        .slug
        .clone();

    let known = manager.get_model_info(known_slug.as_str(), &config).await;
    assert!(!known.used_fallback_model_metadata);
    assert_eq!(known.slug, known_slug);

    let unknown = manager
        .get_model_info("model-that-does-not-exist", &config)
        .await;
    assert!(unknown.used_fallback_model_metadata);
    assert_eq!(unknown.slug, "model-that-does-not-exist");
}

#[tokio::test]
async fn get_model_info_uses_custom_catalog() {
    let config = ModelsManagerConfig::default();
    let mut overlay = remote_model("gpt-overlay", "Overlay", /*priority*/ 0);
    overlay.supports_image_detail_original = true;

    let manager = static_manager_for_tests(ModelsResponse {
        models: vec![overlay],
    });

    let model_info = manager
        .get_model_info("gpt-overlay-experiment", &config)
        .await;

    assert_eq!(model_info.slug, "gpt-overlay-experiment");
    assert_eq!(model_info.display_name, "Overlay");
    assert_eq!(model_info.context_window, Some(272_000));
    assert!(model_info.supports_image_detail_original);
    assert!(!model_info.supports_parallel_tool_calls);
    assert!(!model_info.used_fallback_model_metadata);
}

#[tokio::test]
async fn get_model_info_matches_namespaced_suffix() {
    let config = ModelsManagerConfig::default();
    let mut remote = remote_model("gpt-image", "Image", /*priority*/ 0);
    remote.supports_image_detail_original = true;
    let manager = static_manager_for_tests(ModelsResponse {
        models: vec![remote],
    });
    let namespaced_model = "custom/gpt-image".to_string();

    let model_info = manager.get_model_info(&namespaced_model, &config).await;

    assert_eq!(model_info.slug, namespaced_model);
    assert!(model_info.supports_image_detail_original);
    assert!(!model_info.used_fallback_model_metadata);
}

#[tokio::test]
async fn get_model_info_matches_hyphenated_provider_namespace_suffix() {
    let config = ModelsManagerConfig::default();
    let remote = remote_model("gpt-image", "Image", /*priority*/ 0);
    let manager = static_manager_for_tests(ModelsResponse {
        models: vec![remote],
    });
    let namespaced_model = "openai-codex/gpt-image".to_string();

    let model_info = manager.get_model_info(&namespaced_model, &config).await;

    assert_eq!(model_info.slug, namespaced_model);
    assert!(!model_info.used_fallback_model_metadata);
}

#[tokio::test]
async fn get_model_info_rejects_multi_segment_namespace_suffix_matching() {
    let codex_home = tempdir().expect("temp dir");
    let config = ModelsManagerConfig::default();
    let manager = openai_manager_for_tests(
        codex_home.path().to_path_buf(),
        TestModelsEndpoint::new(Vec::new()),
    );
    let known_slug = manager
        .get_remote_models()
        .await
        .first()
        .expect("bundled models should include at least one model")
        .slug
        .clone();
    let namespaced_model = format!("ns1/ns2/{known_slug}");

    let model_info = manager.get_model_info(&namespaced_model, &config).await;

    assert_eq!(model_info.slug, namespaced_model);
    assert!(model_info.used_fallback_model_metadata);
}

#[tokio::test]
async fn refresh_available_models_sorts_by_priority() {
    let remote_models = vec![
        remote_model("priority-low", "Low", /*priority*/ 1),
        remote_model("priority-high", "High", /*priority*/ 0),
    ];
    let codex_home = tempdir().expect("temp dir");
    let endpoint = TestModelsEndpoint::new(vec![remote_models.clone()]);
    let manager = openai_manager_for_tests(codex_home.path().to_path_buf(), endpoint.clone());

    let available = manager
        .list_models(
            RefreshStrategy::Online,
            HttpClientFactory::new(OutboundProxyPolicy::RespectSystemProxy),
        )
        .await;
    assert_models_contain(&manager.get_remote_models().await, &remote_models);
    assert_eq!(
        endpoint.observed_proxy_policy(),
        Some(OutboundProxyPolicy::RespectSystemProxy)
    );
    let high_idx = available
        .iter()
        .position(|model| model.model == "priority-high")
        .expect("priority-high should be listed");
    let low_idx = available
        .iter()
        .position(|model| model.model == "priority-low")
        .expect("priority-low should be listed");
    assert!(
        high_idx < low_idx,
        "higher priority should be listed before lower priority"
    );
    assert_eq!(endpoint.fetch_count(), 1, "expected a single model fetch");
}

#[tokio::test]
async fn refresh_available_models_merges_chatgpt_remote_with_bundled_catalog() {
    let remote_models = vec![remote_model(
        "chatgpt-visible-source-of-truth",
        "ChatGPT Visible",
        /*priority*/ 0,
    )];
    let codex_home = tempdir().expect("temp dir");
    let endpoint = TestModelsEndpoint::new(vec![remote_models.clone()]);
    let manager = openai_manager_for_tests(codex_home.path().to_path_buf(), endpoint.clone());
    let mut expected = load_remote_models_from_file().expect("bundled models should parse");
    expected.extend(remote_models);

    manager
        .refresh_available_models(
            RefreshStrategy::OnlineIfUncached,
            &DEFAULT_HTTP_CLIENT_FACTORY,
        )
        .await
        .expect("refresh succeeds");

    assert_eq!(manager.get_remote_models().await, expected);
    assert_eq!(endpoint.fetch_count(), 1, "expected a single model fetch");
}

#[tokio::test]
async fn refresh_available_models_merges_cached_chatgpt_remote_with_bundled_catalog() {
    let remote_models = vec![remote_model(
        "chatgpt-cached-source-of-truth",
        "ChatGPT Cached",
        /*priority*/ 0,
    )];
    let codex_home = tempdir().expect("temp dir");
    let fetch_endpoint = TestModelsEndpoint::new(vec![remote_models.clone()]);
    let fetch_manager =
        openai_manager_for_tests(codex_home.path().to_path_buf(), fetch_endpoint.clone());

    fetch_manager
        .refresh_available_models(
            RefreshStrategy::OnlineIfUncached,
            &DEFAULT_HTTP_CLIENT_FACTORY,
        )
        .await
        .expect("initial refresh succeeds");

    let cache_endpoint = TestModelsEndpoint::new(Vec::new());
    let cache_manager =
        openai_manager_for_tests(codex_home.path().to_path_buf(), cache_endpoint.clone());
    let mut expected = load_remote_models_from_file().expect("bundled models should parse");
    expected.extend(remote_models);

    cache_manager
        .refresh_available_models(
            RefreshStrategy::OnlineIfUncached,
            &DEFAULT_HTTP_CLIENT_FACTORY,
        )
        .await
        .expect("cached refresh succeeds");

    assert_eq!(cache_manager.get_remote_models().await, expected);
    assert_eq!(
        cache_endpoint.fetch_count(),
        0,
        "fresh cache should avoid a model fetch"
    );
}

#[tokio::test]
async fn chatgpt_cache_does_not_evict_pfterminal_provider_models() {
    let remote_models = vec![remote_model("gpt-5.5", "GPT-5.5", /*priority*/ 0)];
    let codex_home = tempdir().expect("temp dir");
    let fetch_endpoint = TestModelsEndpoint::new(vec![remote_models]);
    let fetch_manager =
        openai_manager_for_tests(codex_home.path().to_path_buf(), fetch_endpoint.clone());

    fetch_manager
        .refresh_available_models(
            RefreshStrategy::OnlineIfUncached,
            &DEFAULT_HTTP_CLIENT_FACTORY,
        )
        .await
        .expect("initial refresh succeeds");

    let cache_endpoint = TestModelsEndpoint::new(Vec::new());
    let cache_manager = openai_manager_for_tests(codex_home.path().to_path_buf(), cache_endpoint);
    let available = cache_manager
        .list_models(
            RefreshStrategy::OnlineIfUncached,
            DEFAULT_HTTP_CLIENT_FACTORY,
        )
        .await;
    let slugs = available
        .iter()
        .map(|model| model.model.as_str())
        .collect::<Vec<_>>();

    assert!(slugs.contains(&"gpt-5.5"));
    assert!(slugs.contains(&"z-ai/glm-5.2"));
    assert!(slugs.contains(&"moonshotai/kimi-k2.7-code"));
    assert!(slugs.contains(&"glm-5.3"));
    assert!(slugs.contains(&"glm-5.2"));
    assert!(slugs.contains(&"z-ai/glm-5.2"));
    assert!(slugs.contains(&"zai-org/GLM-5.2"));
    assert!(slugs.contains(&"zai/glm-5.2"));
    assert!(slugs.contains(&"zai/glm-5.2-fast"));
}

#[tokio::test]
async fn remote_model_overlay_preserves_bundled_orchestration_metadata() {
    let mut remote_models = vec![remote_model(
        "gpt-5.6-sol",
        "Remote Sol",
        /*priority*/ 0,
    )];
    remote_models[0].orchestration = Some(ModelOrchestrationMetadata::Disabled {
        provider_id: "attacker-provider".to_string(),
        capability: ModelCapabilityTier::Legacy,
        reason: "remote payload attempted to replace local policy".to_string(),
    });
    remote_models[0].supported_reasoning_levels = vec![ReasoningEffortPreset {
        effort: ReasoningEffort::Custom("untrusted-expensive-mode".to_string()),
        description: "remote-only effort".to_string(),
    }];
    let codex_home = tempdir().expect("temp dir");
    let endpoint = TestModelsEndpoint::new(vec![remote_models]);
    let manager = openai_manager_for_tests(codex_home.path().to_path_buf(), endpoint);

    manager
        .refresh_available_models(
            RefreshStrategy::OnlineIfUncached,
            &DEFAULT_HTTP_CLIENT_FACTORY,
        )
        .await
        .expect("refresh succeeds");

    let sol = manager
        .get_remote_models()
        .await
        .into_iter()
        .find(|model| model.slug == "gpt-5.6-sol")
        .expect("remote overlay should retain Sol");
    assert_eq!(
        sol.orchestration,
        Some(ModelOrchestrationMetadata::Eligible {
            provider_id: "openai".to_string(),
            capability: ModelCapabilityTier::Frontier,
            billing: ModelBilling::AuthDependent {
                plan_relative_burn_millis: 1_000,
                api_key_input_milli_usd_per_million_tokens: 5_000,
                api_key_output_milli_usd_per_million_tokens: 30_000,
                api_key_cached_input_milli_usd_per_million_tokens: Some(500),
            },
        })
    );
    assert!(
        sol.supported_reasoning_levels
            .iter()
            .all(|preset| preset.effort.as_str() != "untrusted-expensive-mode"),
        "remote discovery must not broaden the bundled effort policy"
    );
}

#[tokio::test]
async fn get_model_info_keeps_bundled_models_when_chatgpt_remote_is_present() {
    let remote_models = vec![remote_model(
        "chatgpt-merged-model-info",
        "ChatGPT Model Info",
        /*priority*/ 0,
    )];
    let codex_home = tempdir().expect("temp dir");
    let endpoint = TestModelsEndpoint::new(vec![remote_models]);
    let manager = openai_manager_for_tests(codex_home.path().to_path_buf(), endpoint);
    let bundled_slug = load_remote_models_from_file()
        .expect("bundled models should parse")
        .first()
        .expect("bundled models should contain at least one model")
        .slug
        .clone();

    manager
        .refresh_available_models(
            RefreshStrategy::OnlineIfUncached,
            &DEFAULT_HTTP_CLIENT_FACTORY,
        )
        .await
        .expect("refresh succeeds");

    let model_info = manager
        .get_model_info(&bundled_slug, &ModelsManagerConfig::default())
        .await;

    assert_eq!(model_info.slug, bundled_slug);
    assert!(!model_info.used_fallback_model_metadata);
}

#[tokio::test]
async fn refresh_available_models_preserves_bundled_catalog_for_empty_chatgpt_remote() {
    let codex_home = tempdir().expect("temp dir");
    let endpoint = TestModelsEndpoint::new(vec![Vec::new()]);
    let manager = openai_manager_for_tests(codex_home.path().to_path_buf(), endpoint);
    let expected = load_remote_models_from_file().expect("bundled models should parse");

    manager
        .refresh_available_models(
            RefreshStrategy::OnlineIfUncached,
            &DEFAULT_HTTP_CLIENT_FACTORY,
        )
        .await
        .expect("refresh succeeds");

    assert_eq!(manager.get_remote_models().await, expected);
}

#[tokio::test]
async fn chatgpt_catalog_keeps_verified_gpt_5_6_models_when_remote_omits_them() {
    let codex_home = tempdir().expect("temp dir");
    let endpoint = TestModelsEndpoint::new(vec![Vec::new()]);
    let manager = openai_manager_for_tests(codex_home.path().to_path_buf(), endpoint);

    let available = manager
        .list_models(RefreshStrategy::Online, DEFAULT_HTTP_CLIENT_FACTORY)
        .await;
    let picker_visibility = available
        .iter()
        .filter(|model| model.model.starts_with("gpt-5.6-"))
        .map(|model| (model.model.as_str(), model.show_in_picker))
        .collect::<Vec<_>>();

    for slug in ["gpt-5.6-sol", "gpt-5.6-terra", "gpt-5.6-luna"] {
        assert!(
            picker_visibility.contains(&(slug, true)),
            "{slug} should remain available unless the server explicitly hides it"
        );
    }

    assert!(
        available
            .iter()
            .any(|model| model.model == "gpt-5.6-sol" && model.is_default),
        "Sol should be the default visible preset"
    );
}

#[tokio::test]
async fn chatgpt_catalog_honors_explicit_remote_hiding_for_gpt_5_6_models() {
    let mut remote_models = crate::bundled_models_response()
        .expect("bundled models should parse")
        .models
        .into_iter()
        .filter(|model| model.slug.starts_with("gpt-5.6-"))
        .collect::<Vec<_>>();
    for model in &mut remote_models {
        model.visibility = ModelVisibility::Hide;
    }
    let codex_home = tempdir().expect("temp dir");
    let endpoint = TestModelsEndpoint::new(vec![remote_models]);
    let manager = openai_manager_for_tests(codex_home.path().to_path_buf(), endpoint);

    let available = manager
        .list_models(RefreshStrategy::Online, DEFAULT_HTTP_CLIENT_FACTORY)
        .await;

    assert!(
        available
            .iter()
            .filter(|model| model.model.starts_with("gpt-5.6-"))
            .all(|model| !model.show_in_picker),
        "an explicit hidden response must override bundled visibility"
    );
}

#[tokio::test]
async fn chatgpt_catalog_shows_server_advertised_gpt_5_6_models() {
    let mut remote_models = crate::bundled_models_response()
        .expect("bundled models should parse")
        .models
        .into_iter()
        .filter(|model| model.slug.starts_with("gpt-5.6-"))
        .collect::<Vec<_>>();
    for model in &mut remote_models {
        model.visibility = ModelVisibility::List;
        model.description = Some(format!("Server metadata for {}", model.slug));
    }
    let codex_home = tempdir().expect("temp dir");
    let endpoint = TestModelsEndpoint::new(vec![remote_models]);
    let manager = openai_manager_for_tests(codex_home.path().to_path_buf(), endpoint);

    let available = manager
        .list_models(RefreshStrategy::Online, DEFAULT_HTTP_CLIENT_FACTORY)
        .await;
    assert!(
        available
            .iter()
            .any(|model| model.model == "gpt-5.6-sol" && model.is_default)
    );
    let sol = available
        .iter()
        .find(|model| model.model == "gpt-5.6-sol")
        .expect("server-advertised Sol should be available");
    assert!(
        sol.supported_reasoning_efforts
            .iter()
            .any(|level| level.effort == ReasoningEffort::Ultra),
        "runtime model metadata must preserve server-advertised typed efforts"
    );
    let advertised = available
        .into_iter()
        .filter(|model| model.model.starts_with("gpt-5.6-"))
        .map(|model| (model.model, model.description, model.show_in_picker))
        .collect::<Vec<_>>();

    assert_eq!(
        advertised,
        vec![
            (
                "gpt-5.6-sol".to_string(),
                "Server metadata for gpt-5.6-sol".to_string(),
                true,
            ),
            (
                "gpt-5.6-terra".to_string(),
                "Server metadata for gpt-5.6-terra".to_string(),
                true,
            ),
            (
                "gpt-5.6-luna".to_string(),
                "Server metadata for gpt-5.6-luna".to_string(),
                true,
            ),
        ]
    );
}

#[tokio::test]
async fn remote_catalog_preserves_bundled_reasoning_when_remote_omits_it() {
    let mut remote_model = crate::bundled_models_response()
        .expect("bundled models should parse")
        .models
        .into_iter()
        .find(|model| model.slug == "deepseek/deepseek-v4-pro")
        .expect("bundled catalog should include DeepSeek V4 Pro");
    remote_model.display_name = "Remote DeepSeek V4 Pro".to_string();
    remote_model.default_reasoning_level = None;
    remote_model.supported_reasoning_levels.clear();

    let codex_home = tempdir().expect("temp dir");
    let endpoint = TestModelsEndpoint::new(vec![vec![remote_model]]);
    let manager = openai_manager_for_tests(codex_home.path().to_path_buf(), endpoint);

    let available = manager
        .list_models(RefreshStrategy::Online, DEFAULT_HTTP_CLIENT_FACTORY)
        .await;
    let deepseek = available
        .iter()
        .find(|model| model.model == "deepseek/deepseek-v4-pro")
        .expect("remote DeepSeek model should remain available");

    assert_eq!(deepseek.display_name, "Remote DeepSeek V4 Pro");
    assert_eq!(deepseek.default_reasoning_effort, ReasoningEffort::High);
    assert_eq!(
        deepseek
            .supported_reasoning_efforts
            .iter()
            .map(|level| level.effort.clone())
            .collect::<Vec<_>>(),
        vec![ReasoningEffort::High, ReasoningEffort::XHigh]
    );
}

#[tokio::test]
async fn refresh_available_models_merges_hidden_only_chatgpt_remote_with_bundled_catalog() {
    let hidden_remote = remote_model_with_visibility(
        "chatgpt-hidden-only",
        "ChatGPT Hidden",
        /*priority*/ 0,
        "hide",
    );
    let codex_home = tempdir().expect("temp dir");
    let endpoint = TestModelsEndpoint::new(vec![vec![hidden_remote.clone()]]);
    let manager = openai_manager_for_tests(codex_home.path().to_path_buf(), endpoint);
    let mut expected = load_remote_models_from_file().expect("bundled models should parse");
    expected.push(hidden_remote);

    manager
        .refresh_available_models(
            RefreshStrategy::OnlineIfUncached,
            &DEFAULT_HTTP_CLIENT_FACTORY,
        )
        .await
        .expect("refresh succeeds");

    assert_eq!(manager.get_remote_models().await, expected);
}

#[tokio::test]
async fn refresh_available_models_keeps_merging_for_api_auth() {
    let remote_models = vec![remote_model(
        "api-auth-visible-remote",
        "API Auth Visible",
        /*priority*/ 0,
    )];
    let codex_home = tempdir().expect("temp dir");
    let endpoint = Arc::new(TestModelsEndpoint {
        has_command_auth: true,
        uses_codex_backend: false,
        responses: Mutex::new(vec![remote_models.clone()].into()),
        fetch_count: AtomicUsize::new(0),
        observed_proxy_policy: Mutex::new(None),
    });
    let manager = openai_manager_for_tests_with_auth(
        codex_home.path().to_path_buf(),
        endpoint.clone(),
        Some(AuthManager::from_auth_for_testing(CodexAuth::from_api_key(
            "test-api-key",
        ))),
    );
    let mut expected = load_remote_models_from_file().expect("bundled models should parse");
    expected.extend(remote_models);

    manager
        .refresh_available_models(
            RefreshStrategy::OnlineIfUncached,
            &DEFAULT_HTTP_CLIENT_FACTORY,
        )
        .await
        .expect("refresh succeeds");

    assert_eq!(manager.get_remote_models().await, expected);
    assert_eq!(endpoint.fetch_count(), 1, "expected a single model fetch");
}

#[tokio::test]
async fn refresh_available_models_uses_cache_when_fresh() {
    let remote_models = vec![remote_model("cached", "Cached", /*priority*/ 5)];
    let codex_home = tempdir().expect("temp dir");
    let endpoint = TestModelsEndpoint::new(vec![remote_models.clone()]);
    let manager = openai_manager_for_tests(codex_home.path().to_path_buf(), endpoint.clone());

    manager
        .refresh_available_models(
            RefreshStrategy::OnlineIfUncached,
            &DEFAULT_HTTP_CLIENT_FACTORY,
        )
        .await
        .expect("first refresh succeeds");
    assert_models_contain(&manager.get_remote_models().await, &remote_models);

    // Second call should read from cache and avoid the network.
    manager
        .refresh_available_models(
            RefreshStrategy::OnlineIfUncached,
            &DEFAULT_HTTP_CLIENT_FACTORY,
        )
        .await
        .expect("cached refresh succeeds");
    assert_models_contain(&manager.get_remote_models().await, &remote_models);
    assert_eq!(
        endpoint.fetch_count(),
        1,
        "cache hit should avoid a second model fetch"
    );
}

#[tokio::test]
async fn refresh_available_models_refetches_when_cache_stale() {
    let initial_models = vec![remote_model("stale", "Stale", /*priority*/ 1)];
    let codex_home = tempdir().expect("temp dir");
    let updated_models = vec![remote_model("fresh", "Fresh", /*priority*/ 9)];
    let endpoint = TestModelsEndpoint::new(vec![initial_models.clone(), updated_models.clone()]);
    let manager = openai_manager_for_tests(codex_home.path().to_path_buf(), endpoint.clone());

    manager
        .refresh_available_models(
            RefreshStrategy::OnlineIfUncached,
            &DEFAULT_HTTP_CLIENT_FACTORY,
        )
        .await
        .expect("initial refresh succeeds");

    // Rewrite cache with an old timestamp so it is treated as stale.
    manager
        .cache_manager
        .as_ref()
        .expect("cached model manager")
        .manipulate_cache_for_test(|fetched_at| {
            *fetched_at = Utc::now() - chrono::Duration::hours(1);
        })
        .await
        .expect("cache manipulation succeeds");

    manager
        .refresh_available_models(
            RefreshStrategy::OnlineIfUncached,
            &DEFAULT_HTTP_CLIENT_FACTORY,
        )
        .await
        .expect("second refresh succeeds");
    assert_models_contain(&manager.get_remote_models().await, &updated_models);
    assert_eq!(
        endpoint.fetch_count(),
        2,
        "stale cache refresh should fetch models again"
    );
}

#[tokio::test]
async fn refresh_available_models_refetches_when_version_mismatch() {
    let initial_models = vec![remote_model("old", "Old", /*priority*/ 1)];
    let codex_home = tempdir().expect("temp dir");
    let updated_models = vec![remote_model("new", "New", /*priority*/ 2)];
    let endpoint = TestModelsEndpoint::new(vec![initial_models.clone(), updated_models.clone()]);
    let manager = openai_manager_for_tests(codex_home.path().to_path_buf(), endpoint.clone());

    manager
        .refresh_available_models(
            RefreshStrategy::OnlineIfUncached,
            &DEFAULT_HTTP_CLIENT_FACTORY,
        )
        .await
        .expect("initial refresh succeeds");

    manager
        .cache_manager
        .as_ref()
        .expect("cached model manager")
        .mutate_cache_for_test(|cache| {
            let client_version = crate::client_version_to_whole();
            cache.client_version = Some(format!("{client_version}-mismatch"));
        })
        .await
        .expect("cache mutation succeeds");

    manager
        .refresh_available_models(
            RefreshStrategy::OnlineIfUncached,
            &DEFAULT_HTTP_CLIENT_FACTORY,
        )
        .await
        .expect("second refresh succeeds");
    assert_models_contain(&manager.get_remote_models().await, &updated_models);
    assert_eq!(
        endpoint.fetch_count(),
        2,
        "version mismatch should fetch models again"
    );
}

#[tokio::test]
async fn refresh_available_models_drops_removed_remote_models() {
    let initial_models = vec![remote_model(
        "remote-old",
        "Remote Old",
        /*priority*/ 1,
    )];
    let codex_home = tempdir().expect("temp dir");
    let refreshed_models = vec![remote_model(
        "remote-new",
        "Remote New",
        /*priority*/ 1,
    )];
    let endpoint = TestModelsEndpoint::new(vec![initial_models, refreshed_models]);
    let mut manager = openai_manager_for_tests(codex_home.path().to_path_buf(), endpoint.clone());
    manager
        .cache_manager
        .as_mut()
        .expect("cached model manager")
        .set_ttl(Duration::ZERO);

    manager
        .refresh_available_models(
            RefreshStrategy::OnlineIfUncached,
            &DEFAULT_HTTP_CLIENT_FACTORY,
        )
        .await
        .expect("initial refresh succeeds");

    manager
        .refresh_available_models(
            RefreshStrategy::OnlineIfUncached,
            &DEFAULT_HTTP_CLIENT_FACTORY,
        )
        .await
        .expect("second refresh succeeds");

    let available = manager
        .try_list_models()
        .expect("models should be available");
    assert!(
        available.iter().any(|preset| preset.model == "remote-new"),
        "new remote model should be listed"
    );
    assert!(
        !available.iter().any(|preset| preset.model == "remote-old"),
        "removed remote model should not be listed"
    );
    assert_eq!(
        endpoint.fetch_count(),
        2,
        "second refresh should fetch models again"
    );
}

#[tokio::test]
async fn refresh_available_models_skips_network_without_chatgpt_auth() {
    let dynamic_slug = "dynamic-model-only-for-test-noauth";
    let codex_home = tempdir().expect("temp dir");
    let endpoint = TestModelsEndpoint::without_refresh(vec![vec![remote_model(
        dynamic_slug,
        "No Auth",
        /*priority*/ 1,
    )]]);
    let manager = openai_manager_for_tests_with_auth(
        codex_home.path().to_path_buf(),
        endpoint.clone(),
        /*auth_manager*/ None,
    );

    manager
        .refresh_available_models(RefreshStrategy::Online, &DEFAULT_HTTP_CLIENT_FACTORY)
        .await
        .expect("refresh should no-op without chatgpt auth");
    let cached_remote = manager.get_remote_models().await;
    assert!(
        !cached_remote
            .iter()
            .any(|candidate| candidate.slug == dynamic_slug),
        "remote refresh should be skipped without chatgpt auth"
    );
    assert_eq!(
        endpoint.fetch_count(),
        0,
        "endpoint that cannot refresh should avoid model fetches"
    );
}

#[derive(Debug)]
struct TestAuthAwareModelsEndpoint {
    auth_manager: Option<Arc<AuthManager>>,
    responses: Mutex<VecDeque<Vec<ModelInfo>>>,
    fetch_count: AtomicUsize,
}

impl TestAuthAwareModelsEndpoint {
    fn new(auth_manager: Option<Arc<AuthManager>>, responses: Vec<Vec<ModelInfo>>) -> Arc<Self> {
        Arc::new(Self {
            auth_manager,
            responses: Mutex::new(responses.into()),
            fetch_count: AtomicUsize::new(0),
        })
    }

    fn fetch_count(&self) -> usize {
        self.fetch_count.load(Ordering::SeqCst)
    }

    async fn uses_codex_backend(&self) -> bool {
        match self.auth_manager.as_ref() {
            Some(auth_manager) => auth_manager
                .auth()
                .await
                .as_ref()
                .is_some_and(CodexAuth::uses_codex_backend),
            None => false,
        }
    }

    async fn list_models(&self) -> CoreResult<(Vec<ModelInfo>, Option<String>)> {
        self.fetch_count.fetch_add(1, Ordering::SeqCst);
        let models = self
            .responses
            .lock()
            .expect("responses lock should not be poisoned")
            .pop_front()
            .unwrap_or_default();
        Ok((models, None))
    }
}

impl ModelsEndpointClient for TestAuthAwareModelsEndpoint {
    fn has_command_auth(&self) -> bool {
        false
    }

    fn uses_codex_backend(&self) -> ModelsEndpointFuture<'_, bool> {
        Box::pin(TestAuthAwareModelsEndpoint::uses_codex_backend(self))
    }

    fn list_models<'a>(
        &'a self,
        _client_version: &'a str,
        _http_client_factory: HttpClientFactory,
    ) -> ModelsEndpointFuture<'a, CoreResult<(Vec<ModelInfo>, Option<String>)>> {
        Box::pin(TestAuthAwareModelsEndpoint::list_models(self))
    }
}

#[tokio::test]
async fn refresh_available_models_skips_network_when_external_api_key_overrides_chatgpt_auth() {
    let dynamic_slug = "dynamic-model-only-for-test-external-api-key";
    let codex_home = tempdir().expect("temp dir");
    let auth_manager =
        AuthManager::from_auth_for_testing(CodexAuth::create_dummy_chatgpt_auth_for_testing());
    auth_manager
        .set_external_auth(Arc::new(TestExternalApiKeyAuth))
        .await
        .expect("external API key auth should resolve");
    let endpoint = TestAuthAwareModelsEndpoint::new(
        Some(Arc::clone(&auth_manager)),
        vec![vec![remote_model(
            dynamic_slug,
            "External API Key",
            /*priority*/ 1,
        )]],
    );
    let manager = openai_manager_for_tests_with_auth(
        codex_home.path().to_path_buf(),
        endpoint.clone(),
        Some(auth_manager),
    );

    manager
        .refresh_available_models(RefreshStrategy::Online, &DEFAULT_HTTP_CLIENT_FACTORY)
        .await
        .expect("refresh should no-op with API key auth");
    let cached_remote = manager.get_remote_models().await;

    assert!(
        !cached_remote
            .iter()
            .any(|candidate| candidate.slug == dynamic_slug),
        "remote refresh should be skipped when external API key auth is active"
    );
    assert_eq!(
        endpoint.fetch_count(),
        0,
        "endpoint should avoid model fetches when external API key auth is active"
    );
}

#[tokio::test]
async fn refresh_available_models_uses_cached_chatgpt_when_external_api_key_is_unresolved() {
    let dynamic_slug = "dynamic-model-only-for-test-unresolved-external-api-key";
    let codex_home = tempdir().expect("temp dir");
    let auth_manager =
        AuthManager::from_auth_for_testing(CodexAuth::create_dummy_chatgpt_auth_for_testing());
    auth_manager
        .set_external_auth(Arc::new(TestUnresolvedExternalApiKeyAuth))
        .await
        .expect_err("unresolved external auth should be rejected");
    let endpoint = TestAuthAwareModelsEndpoint::new(
        Some(Arc::clone(&auth_manager)),
        vec![vec![remote_model(
            dynamic_slug,
            "Unresolved External API Key",
            /*priority*/ 1,
        )]],
    );
    let manager = openai_manager_for_tests_with_auth(
        codex_home.path().to_path_buf(),
        endpoint.clone(),
        Some(auth_manager),
    );

    manager
        .refresh_available_models(RefreshStrategy::Online, &DEFAULT_HTTP_CLIENT_FACTORY)
        .await
        .expect("refresh should fall back to cached ChatGPT auth");

    assert!(
        manager
            .get_remote_models()
            .await
            .iter()
            .any(|candidate| candidate.slug == dynamic_slug),
        "remote refresh should include models fetched with cached ChatGPT auth"
    );
    assert_eq!(
        endpoint.fetch_count(),
        1,
        "endpoint should fetch models when unresolved external API key falls back to ChatGPT auth"
    );
}

#[tokio::test]
async fn refresh_available_models_fetches_with_chatgpt_auth_tokens() {
    let dynamic_slug = "dynamic-model-only-for-test-chatgpt-auth-tokens";
    let codex_home = tempdir().expect("temp dir");
    let endpoint = TestModelsEndpoint::new(vec![vec![remote_model(
        dynamic_slug,
        "ChatGPT Auth Tokens",
        /*priority*/ 1,
    )]]);
    let auth = chatgpt_auth_tokens_for_tests(codex_home.path()).await;
    let manager = openai_manager_for_tests_with_auth(
        codex_home.path().to_path_buf(),
        endpoint.clone(),
        Some(AuthManager::from_auth_for_testing(auth)),
    );

    manager
        .refresh_available_models(RefreshStrategy::Online, &DEFAULT_HTTP_CLIENT_FACTORY)
        .await
        .expect("refresh should fetch with ChatGPT auth tokens");

    assert!(
        manager
            .get_remote_models()
            .await
            .iter()
            .any(|candidate| candidate.slug == dynamic_slug),
        "remote refresh should include models fetched with ChatGPT auth tokens"
    );
    assert_eq!(
        endpoint.fetch_count(),
        1,
        "endpoint should fetch models with ChatGPT auth tokens"
    );
}

#[test]
fn build_available_models_picks_default_after_hiding_hidden_models() {
    let manager = static_manager_for_tests(ModelsResponse { models: Vec::new() });

    let hidden_model =
        remote_model_with_visibility("hidden", "Hidden", /*priority*/ 0, "hide");
    let visible_model =
        remote_model_with_visibility("visible", "Visible", /*priority*/ 1, "list");

    let expected_hidden = ModelPreset::from(hidden_model.clone());
    let mut expected_visible = ModelPreset::from(visible_model.clone());
    expected_visible.is_default = true;

    let available = manager.build_available_models(vec![hidden_model, visible_model]);

    assert_eq!(available, vec![expected_hidden, expected_visible]);
}

#[tokio::test]
async fn static_manager_reads_latest_auth_mode() {
    let auth_manager =
        AuthManager::from_auth_for_testing(CodexAuth::create_dummy_chatgpt_auth_for_testing());
    let chatgpt_only_model = {
        let mut model = remote_model("chatgpt-only", "ChatGPT Only", /*priority*/ 0);
        model.supported_in_api = false;
        model
    };
    let api_model = remote_model("api-model", "API Model", /*priority*/ 1);
    let manager = StaticModelsManager::new(
        Some(Arc::clone(&auth_manager)),
        ModelsResponse {
            models: vec![chatgpt_only_model, api_model],
        },
    );

    let chatgpt_models = manager
        .list_models(RefreshStrategy::Online, DEFAULT_HTTP_CLIENT_FACTORY)
        .await;
    assert_eq!(
        chatgpt_models
            .iter()
            .map(|model| model.model.as_str())
            .collect::<Vec<_>>(),
        vec!["chatgpt-only", "api-model"]
    );

    auth_manager
        .set_external_auth(Arc::new(TestExternalApiKeyAuth))
        .await
        .expect("external API key auth should resolve");
    let api_models = manager
        .list_models(RefreshStrategy::Online, DEFAULT_HTTP_CLIENT_FACTORY)
        .await;

    assert_eq!(
        api_models
            .iter()
            .map(|model| model.model.as_str())
            .collect::<Vec<_>>(),
        vec!["api-model"]
    );
}

#[test]
fn bundled_models_json_roundtrips() {
    let response = crate::bundled_models_response()
        .unwrap_or_else(|err| panic!("bundled models.json should parse: {err}"));

    let serialized =
        serde_json::to_string(&response).expect("bundled models.json should serialize");
    let roundtripped: ModelsResponse =
        serde_json::from_str(&serialized).expect("serialized models.json should deserialize");

    assert_eq!(
        response, roundtripped,
        "bundled models.json should round trip through serde"
    );
    assert!(
        !response.models.is_empty(),
        "bundled models.json should contain at least one model"
    );
}

#[test]
fn bundled_models_json_tracks_verified_image_capabilities() {
    let response = crate::bundled_models_response()
        .unwrap_or_else(|err| panic!("bundled models.json should parse: {err}"));

    let supports_images = |slug: &str| {
        response
            .models
            .iter()
            .find(|model| model.slug == slug)
            .unwrap_or_else(|| panic!("bundled models.json should include {slug}"))
            .input_modalities
            .contains(&InputModality::Image)
    };

    for slug in [
        "moonshotai/kimi-k2.7-code",
        "minimax/minimax-m3",
        "google/gemini-3.5-flash",
        "claude-opus-5-plan",
        "claude-fable-5-1-plan",
        "claude-fable-5-plan",
        "claude-opus-5",
        "claude-fable-5-1",
        "claude-fable-5",
    ] {
        assert!(supports_images(slug), "{slug} should accept image input");
    }

    for slug in [
        "z-ai/glm-5.2",
        "zai/glm-5.2",
        "deepseek-v4-pro",
        "deepseek/deepseek-v4-pro",
        "deepseek/deepseek-v4-pro-0813",
        "deepseek/deepseek-v4-flash-0731",
        "tencent/hy3:free",
        "openrouter/owl-alpha",
    ] {
        assert!(!supports_images(slug), "{slug} should remain text-only");
    }
}

#[test]
fn bundled_claude_5_models_have_provider_reported_output_limits() {
    // Verified against Anthropic's authenticated `/v1/models/{model_id}` responses on 2026-08-01.
    let response = crate::bundled_models_response()
        .unwrap_or_else(|err| panic!("bundled models.json should parse: {err}"));

    for (slug, max_output_tokens) in [
        ("claude-opus-5-plan", 128_000),
        ("claude-fable-5-1-plan", 128_000),
        ("claude-fable-5-plan", 128_000),
        ("claude-opus-5", 128_000),
        ("claude-fable-5-1", 128_000),
        ("claude-fable-5", 128_000),
    ] {
        let model = response
            .models
            .iter()
            .find(|model| model.slug == slug)
            .unwrap_or_else(|| panic!("bundled models.json should include {slug}"));
        assert_eq!(
            model.max_output_tokens,
            Some(max_output_tokens),
            "{slug} must be requestable on the Anthropic wire"
        );
    }
}

#[tokio::test]
async fn remote_catalog_cannot_erase_bundled_output_limit() {
    let mut remote_model = crate::bundled_models_response()
        .expect("bundled models should parse")
        .models
        .into_iter()
        .find(|model| model.slug == "claude-opus-5")
        .expect("bundled catalog should include Claude Opus 5");
    remote_model.max_output_tokens = None;

    let codex_home = tempdir().expect("temp dir");
    let endpoint = TestModelsEndpoint::new(vec![vec![remote_model]]);
    let manager = openai_manager_for_tests(codex_home.path().to_path_buf(), endpoint);
    manager
        .refresh_available_models(RefreshStrategy::Online, &DEFAULT_HTTP_CLIENT_FACTORY)
        .await
        .expect("refresh succeeds");

    let opus = manager
        .get_remote_models()
        .await
        .into_iter()
        .find(|model| model.slug == "claude-opus-5")
        .expect("remote overlay should retain Claude Opus 5");
    assert_eq!(opus.max_output_tokens, Some(128_000));
}

#[test]
fn bundled_models_json_contains_gpt_5_6_family_metadata() {
    let response = crate::bundled_models_response()
        .unwrap_or_else(|err| panic!("bundled models.json should parse: {err}"));
    let actual = response
        .models
        .iter()
        .filter(|model| model.slug.starts_with("gpt-5.6-"))
        .map(|model| {
            (
                model.slug.as_str(),
                model.context_window,
                model.default_reasoning_level.clone(),
                model
                    .supported_reasoning_levels
                    .iter()
                    .map(|level| level.effort.clone())
                    .collect::<Vec<_>>(),
                model.visibility,
            )
        })
        .collect::<Vec<_>>();

    assert_eq!(
        actual,
        vec![
            (
                "gpt-5.6-sol",
                Some(372_000),
                Some(ReasoningEffort::Low),
                vec![
                    ReasoningEffort::Low,
                    ReasoningEffort::Medium,
                    ReasoningEffort::High,
                    ReasoningEffort::XHigh,
                    ReasoningEffort::Max,
                    ReasoningEffort::Ultra,
                ],
                ModelVisibility::List,
            ),
            (
                "gpt-5.6-terra",
                Some(372_000),
                Some(ReasoningEffort::Medium),
                vec![
                    ReasoningEffort::Low,
                    ReasoningEffort::Medium,
                    ReasoningEffort::High,
                    ReasoningEffort::XHigh,
                    ReasoningEffort::Max,
                    ReasoningEffort::Ultra,
                ],
                ModelVisibility::List,
            ),
            (
                "gpt-5.6-luna",
                Some(372_000),
                Some(ReasoningEffort::Medium),
                vec![
                    ReasoningEffort::Low,
                    ReasoningEffort::Medium,
                    ReasoningEffort::High,
                    ReasoningEffort::XHigh,
                    ReasoningEffort::Max,
                ],
                ModelVisibility::List,
            ),
        ]
    );
}

#[test]
fn bundled_models_have_complete_orchestration_contracts() {
    let response = crate::bundled_models_response()
        .unwrap_or_else(|err| panic!("bundled models.json should parse: {err}"));

    for model in &response.models {
        let metadata = model.orchestration.as_ref().unwrap_or_else(|| {
            panic!(
                "bundled model {} must explicitly be eligible or disabled for orchestration",
                model.slug
            )
        });
        assert!(
            !metadata.provider_id().is_empty(),
            "{} must identify its exact provider route",
            model.slug
        );
        match metadata {
            ModelOrchestrationMetadata::Eligible { billing, .. } => match billing {
                ModelBilling::Plan {
                    relative_burn_millis,
                } => assert!(
                    *relative_burn_millis > 0,
                    "{} must have a positive relative plan burn",
                    model.slug
                ),
                ModelBilling::PlanSchedule {
                    off_peak_relative_burn_millis,
                    peak_relative_burn_millis,
                    peak_start_utc_hour,
                    peak_end_utc_hour,
                    peak_weekdays,
                    promotional_off_peak_relative_burn_millis,
                    promotion_valid_through_utc,
                } => {
                    assert!(
                        *off_peak_relative_burn_millis > 0 && *peak_relative_burn_millis > 0,
                        "{} must have positive scheduled plan burn",
                        model.slug
                    );
                    assert!(
                        *peak_start_utc_hour < 24
                            && *peak_end_utc_hour <= 24
                            && peak_start_utc_hour < peak_end_utc_hour,
                        "{} must have a valid UTC peak window",
                        model.slug
                    );
                    assert!(
                        peak_weekdays.is_none_or(|weekdays| !weekdays.is_empty()),
                        "{} must not specify an empty peak weekday set",
                        model.slug
                    );
                    assert_eq!(
                        promotional_off_peak_relative_burn_millis.is_some(),
                        promotion_valid_through_utc.is_some(),
                        "{} must specify both promotion burn and expiry or neither",
                        model.slug
                    );
                }
                // Both values are required by the enum; zero remains valid for an
                // explicitly free metered route.
                ModelBilling::Metered { .. } => {}
                ModelBilling::AuthDependent { .. } => {}
                ModelBilling::Local => {}
            },
            ModelOrchestrationMetadata::Disabled { reason, .. } => assert!(
                !reason.trim().is_empty(),
                "{} must explain why it cannot receive spawned work",
                model.slug
            ),
        }
    }
}

#[test]
fn bundled_orchestration_policy_distinguishes_gpt_5_6_tiers_and_disables_gpt_5_5() {
    let response = crate::bundled_models_response()
        .unwrap_or_else(|err| panic!("bundled models.json should parse: {err}"));
    let metadata = |slug: &str| {
        response
            .models
            .iter()
            .find(|model| model.slug == slug)
            .and_then(|model| model.orchestration.clone())
            .unwrap_or_else(|| panic!("{slug} must have orchestration metadata"))
    };

    assert_eq!(
        metadata("gpt-5.6-sol"),
        ModelOrchestrationMetadata::Eligible {
            provider_id: "openai".to_string(),
            capability: ModelCapabilityTier::Frontier,
            billing: ModelBilling::AuthDependent {
                plan_relative_burn_millis: 1_000,
                api_key_input_milli_usd_per_million_tokens: 5_000,
                api_key_output_milli_usd_per_million_tokens: 30_000,
                api_key_cached_input_milli_usd_per_million_tokens: Some(500),
            },
        }
    );
    assert_eq!(
        metadata("gpt-5.6-terra"),
        ModelOrchestrationMetadata::Eligible {
            provider_id: "openai".to_string(),
            capability: ModelCapabilityTier::Balanced,
            billing: ModelBilling::AuthDependent {
                plan_relative_burn_millis: 500,
                api_key_input_milli_usd_per_million_tokens: 2_500,
                api_key_output_milli_usd_per_million_tokens: 15_000,
                api_key_cached_input_milli_usd_per_million_tokens: Some(250),
            },
        }
    );
    assert_eq!(
        metadata("gpt-5.6-luna"),
        ModelOrchestrationMetadata::Eligible {
            provider_id: "openai".to_string(),
            capability: ModelCapabilityTier::Fast,
            billing: ModelBilling::AuthDependent {
                plan_relative_burn_millis: 200,
                api_key_input_milli_usd_per_million_tokens: 1_000,
                api_key_output_milli_usd_per_million_tokens: 6_000,
                api_key_cached_input_milli_usd_per_million_tokens: Some(100),
            },
        }
    );
    assert_eq!(
        metadata("gpt-5.5"),
        ModelOrchestrationMetadata::Disabled {
            provider_id: "openai".to_string(),
            capability: ModelCapabilityTier::Legacy,
            reason: "superseded by GPT-5.6 and lower capability than Sol, Terra, and Luna"
                .to_string(),
        }
    );
}

#[test]
fn bundled_models_json_contains_ambient_and_zai_models() {
    let response = crate::bundled_models_response()
        .unwrap_or_else(|err| panic!("bundled models.json should parse: {err}"));

    let ambient_default = response
        .models
        .iter()
        .find(|model| model.slug == "z-ai/glm-5.2")
        .expect("bundled models.json should include the Ambient GLM 5.2 default");

    assert_eq!(ambient_default.display_name, "Ambient GLM 5.2");
    assert_eq!(ambient_default.context_window, Some(202_752));
    assert_eq!(
        ambient_default.default_reasoning_level,
        Some(ReasoningEffort::Medium)
    );
    assert_eq!(
        ambient_default
            .supported_reasoning_levels
            .iter()
            .map(|level| level.effort.clone())
            .collect::<Vec<_>>(),
        vec![ReasoningEffort::Medium, ReasoningEffort::XHigh]
    );
    assert_eq!(ambient_default.visibility, ModelVisibility::List);
    assert!(ambient_default.supports_parallel_tool_calls);
    assert_standard_base(&ambient_default.base_instructions);
    assert!(!ambient_default.used_fallback_model_metadata);

    let zai_glm_5_3 = response
        .models
        .iter()
        .find(|model| model.slug == "glm-5.3")
        .expect("bundled models.json should include Z.AI GLM 5.3");

    assert_eq!(
        (
            zai_glm_5_3.context_window,
            zai_glm_5_3.max_output_tokens,
            zai_glm_5_3.default_reasoning_level.clone(),
            zai_glm_5_3.chat_completions.reasoning_protocol,
            zai_glm_5_3.chat_completions.reasoning_effort_protocol,
        ),
        (
            Some(1_000_000),
            Some(128_000),
            Some(ReasoningEffort::Max),
            ChatReasoningProtocol::PreservedRequired,
            ChatReasoningEffortProtocol::LowHighMaxRequiredDefaultMax,
        )
    );
    assert_eq!(
        zai_glm_5_3
            .supported_reasoning_levels
            .iter()
            .map(|level| level.effort.clone())
            .collect::<Vec<_>>(),
        vec![
            ReasoningEffort::Low,
            ReasoningEffort::High,
            ReasoningEffort::Max,
        ]
    );
    assert_eq!(zai_glm_5_3.visibility, ModelVisibility::List);
    assert!(zai_glm_5_3.supports_parallel_tool_calls);
    assert_eq!(
        zai_glm_5_3
            .orchestration
            .as_ref()
            .and_then(ModelOrchestrationMetadata::billing),
        Some(&ModelBilling::PlanSchedule {
            off_peak_relative_burn_millis: 1_000,
            peak_relative_burn_millis: 3_000,
            peak_start_utc_hour: 6,
            peak_end_utc_hour: 10,
            peak_weekdays: Some(codex_protocol::openai_models::WeekdaySet::weekdays_only()),
            promotional_off_peak_relative_burn_millis: None,
            promotion_valid_through_utc: None,
        })
    );
    let preset = ModelPreset::from(zai_glm_5_3.clone());
    assert_eq!(preset.provider_id.as_deref(), Some("zai"));
    assert_standard_base(&zai_glm_5_3.base_instructions);
    assert!(!zai_glm_5_3.used_fallback_model_metadata);

    let ambient_kimi = response
        .models
        .iter()
        .find(|model| model.slug == "moonshotai/kimi-k2.7-code")
        .expect("bundled models.json should include Ambient Kimi K2.7 Code");

    assert_eq!(ambient_kimi.display_name, "Ambient Kimi K2.7 Code");
    assert_eq!(ambient_kimi.context_window, Some(262_144));
    assert_eq!(
        ambient_kimi.default_reasoning_level,
        Some(ReasoningEffort::Medium)
    );
    assert_eq!(
        ambient_kimi
            .supported_reasoning_levels
            .iter()
            .map(|level| level.effort.clone())
            .collect::<Vec<_>>(),
        vec![ReasoningEffort::Medium, ReasoningEffort::XHigh]
    );
    assert_eq!(ambient_kimi.visibility, ModelVisibility::List);
    assert!(ambient_kimi.supports_parallel_tool_calls);
    assert_standard_base(&ambient_kimi.base_instructions);
    assert!(!ambient_kimi.used_fallback_model_metadata);

    let ambient = response
        .models
        .iter()
        .find(|model| model.slug == "ambient/large")
        .expect("bundled models.json should include ambient/large");

    assert_eq!(ambient.display_name, "Ambient Large");
    assert_eq!(ambient.context_window, Some(131_072));
    assert_eq!(ambient.visibility, ModelVisibility::Hide);
    assert!(ambient.supports_parallel_tool_calls);
    assert_standard_base(&ambient.base_instructions);
    assert!(!ambient.used_fallback_model_metadata);
}

#[test]
fn bundled_models_json_routes_standard_base_without_clobbering_gpt55() {
    let response = crate::bundled_models_response()
        .unwrap_or_else(|err| panic!("bundled models.json should parse: {err}"));

    for slug in [
        "z-ai/glm-5.2",
        "moonshotai/kimi-k2.7-code",
        "zai/glm-5.2",
        "zai/glm-5.2-fast",
        "zai-org/GLM-5.2",
        "ambient/large",
        "glm-5.2",
        "minimax/minimax-m3",
        "openrouter/owl-alpha",
        "google/gemini-3.5-flash",
        "x-ai/grok-4.5",
        "deepseek-v4-flash",
        "deepseek/deepseek-v4-pro",
        "deepseek/deepseek-v4-flash-0731",
        "tencent/hy3:free",
        "muse-spark-1.1",
        "claude-opus-5-plan",
        "claude-fable-5-1-plan",
        "claude-fable-5-plan",
        "claude-opus-5",
        "claude-fable-5-1",
        "claude-fable-5",
    ] {
        let model = response
            .models
            .iter()
            .find(|model| model.slug == slug)
            .unwrap_or_else(|| panic!("bundled models.json should include {slug}"));

        assert_standard_base(&model.base_instructions);
        assert!(
            model
                .base_instructions
                .contains(STANDARD_BASE_OUTCOME_MARKER)
        );
        assert!(
            model
                .base_instructions
                .contains(STANDARD_BASE_EVIDENCE_MARKER)
        );
        assert!(!model.base_instructions.contains(OLD_STANDARD_BASE_MARKER));
    }

    let gpt55 = response
        .models
        .iter()
        .find(|model| model.slug == "gpt-5.5")
        .expect("bundled models.json should include gpt-5.5");

    assert!(gpt55.base_instructions.contains(GPT55_GUIDE_MARKER));
    assert!(
        !gpt55
            .base_instructions
            .contains(STANDARD_BASE_EVIDENCE_MARKER)
    );
    assert_ne!(gpt55.base_instructions.trim_end(), STANDARD_BASE.trim_end());
}

#[test]
fn bundled_models_json_contains_kimi_code_k3() {
    let response = crate::bundled_models_response()
        .unwrap_or_else(|err| panic!("bundled models.json should parse: {err}"));
    let kimi = response
        .models
        .iter()
        .find(|model| model.slug == "k3")
        .expect("bundled models.json should include Kimi Code K3");

    assert_eq!(kimi.display_name, "Kimi Code K3");
    assert_eq!(kimi.context_window, Some(262_144));
    assert_eq!(kimi.max_context_window, Some(1_048_576));
    assert_eq!(kimi.default_reasoning_level, Some(ReasoningEffort::Max));
    assert_eq!(
        kimi.supported_reasoning_levels
            .iter()
            .map(|level| level.effort.clone())
            .collect::<Vec<_>>(),
        vec![
            ReasoningEffort::Low,
            ReasoningEffort::High,
            ReasoningEffort::Max,
        ]
    );
    assert!(kimi.supports_parallel_tool_calls);
    assert_eq!(
        kimi.chat_completions.reasoning_protocol,
        ChatReasoningProtocol::PreservedRequired
    );
    assert_standard_base(&kimi.base_instructions);

    let conservative =
        crate::model_info::with_config_overrides(kimi.clone(), &ModelsManagerConfig::default());
    assert_eq!(conservative.context_window, Some(262_144));
    assert_eq!(conservative.auto_compact_token_limit(), Some(235_929));

    let entitled_config = ModelsManagerConfig {
        model_context_window: Some(1_048_576),
        ..Default::default()
    };
    let entitled = crate::model_info::with_config_overrides(kimi.clone(), &entitled_config);
    assert_eq!(entitled.context_window, Some(1_048_576));
    assert_eq!(entitled.auto_compact_token_limit(), Some(943_718));

    // Resume rebuilds model metadata from the persisted configuration. Applying
    // the same entitled config must therefore preserve both the effective
    // window and the compaction boundary.
    let resumed = crate::model_info::with_config_overrides(kimi.clone(), &entitled_config);
    assert_eq!(resumed.context_window, entitled.context_window);
    assert_eq!(
        resumed.auto_compact_token_limit(),
        entitled.auto_compact_token_limit()
    );
}

#[test]
fn bundled_models_json_contains_openrouter_models() {
    let response = crate::bundled_models_response()
        .unwrap_or_else(|err| panic!("bundled models.json should parse: {err}"));

    let openrouter_owl = response
        .models
        .iter()
        .find(|model| model.slug == "openrouter/owl-alpha")
        .expect("bundled models.json should include OpenRouter Owl Alpha");

    assert_eq!(openrouter_owl.display_name, "OpenRouter Owl Alpha");
    assert_eq!(openrouter_owl.context_window, Some(1_048_756));
    assert_eq!(openrouter_owl.default_reasoning_level, None);
    assert!(openrouter_owl.supported_reasoning_levels.is_empty());
    assert_eq!(openrouter_owl.visibility, ModelVisibility::List);
    assert!(
        openrouter_owl
            .description
            .as_deref()
            .unwrap_or_default()
            .contains("$0/M input, $0/M output")
    );

    let openrouter_model = |slug: &str| {
        response
            .models
            .iter()
            .find(|model| model.slug == slug)
            .unwrap_or_else(|| panic!("bundled models.json should include {slug}"))
    };

    let direct_deepseek_pro = response
        .models
        .iter()
        .find(|model| model.slug == "deepseek-v4-pro")
        .expect("bundled models.json should include direct DeepSeek V4 Pro");
    assert_eq!(direct_deepseek_pro.display_name, "DeepSeek V4 Pro (Direct)");
    assert_eq!(direct_deepseek_pro.context_window, Some(1_048_576));
    assert_eq!(direct_deepseek_pro.max_output_tokens, Some(384_000));
    assert_eq!(
        direct_deepseek_pro.default_reasoning_level,
        Some(ReasoningEffort::High)
    );
    assert_eq!(
        direct_deepseek_pro
            .supported_reasoning_levels
            .iter()
            .map(|level| level.effort.clone())
            .collect::<Vec<_>>(),
        vec![ReasoningEffort::High, ReasoningEffort::Max]
    );

    let grok = openrouter_model("x-ai/grok-4.6");
    assert_eq!(grok.display_name, "OpenRouter Grok 4.6");
    assert_eq!(grok.context_window, Some(500_000));
    assert_eq!(grok.default_reasoning_level, Some(ReasoningEffort::High));
    assert_eq!(
        grok.supported_reasoning_levels
            .iter()
            .map(|level| level.effort.clone())
            .collect::<Vec<_>>(),
        vec![
            ReasoningEffort::Low,
            ReasoningEffort::Medium,
            ReasoningEffort::High,
            ReasoningEffort::XHigh,
        ]
    );

    assert_eq!(
        openrouter_model("x-ai/grok-4.5").display_name,
        "OpenRouter Grok 4.5"
    );

    let deepseek_pro = openrouter_model("deepseek/deepseek-v4-pro");
    assert_eq!(deepseek_pro.display_name, "OpenRouter DeepSeek V4 Pro");
    assert_eq!(deepseek_pro.context_window, Some(1_048_576));
    assert_eq!(
        deepseek_pro.default_reasoning_level,
        Some(ReasoningEffort::High)
    );
    assert_eq!(
        deepseek_pro
            .supported_reasoning_levels
            .iter()
            .map(|level| level.effort.clone())
            .collect::<Vec<_>>(),
        vec![ReasoningEffort::High, ReasoningEffort::XHigh]
    );

    let deepseek_pro_0813 = openrouter_model("deepseek/deepseek-v4-pro-0813");
    assert_eq!(
        deepseek_pro_0813.display_name,
        "OpenRouter DeepSeek V4 Pro 0813"
    );
    assert_eq!(deepseek_pro_0813.context_window, Some(1_048_576));
    assert_eq!(deepseek_pro_0813.max_output_tokens, Some(384_000));
    assert_eq!(
        deepseek_pro_0813.default_reasoning_level,
        Some(ReasoningEffort::High)
    );
    assert_eq!(
        deepseek_pro_0813
            .supported_reasoning_levels
            .iter()
            .map(|level| level.effort.clone())
            .collect::<Vec<_>>(),
        vec![
            ReasoningEffort::Low,
            ReasoningEffort::High,
            ReasoningEffort::Max,
        ]
    );

    let deepseek_flash = openrouter_model("deepseek/deepseek-v4-flash-0731");
    assert_eq!(
        deepseek_flash.display_name,
        "OpenRouter DeepSeek V4 Flash 0731"
    );
    assert_eq!(deepseek_flash.context_window, Some(1_048_576));
    assert_eq!(
        deepseek_flash.default_reasoning_level,
        Some(ReasoningEffort::High)
    );
    assert_eq!(
        deepseek_flash
            .supported_reasoning_levels
            .iter()
            .map(|level| level.effort.clone())
            .collect::<Vec<_>>(),
        vec![
            ReasoningEffort::Low,
            ReasoningEffort::High,
            ReasoningEffort::Max,
        ]
    );

    let hy3 = openrouter_model("tencent/hy3:free");
    assert_eq!(hy3.display_name, "OpenRouter Tencent Hy3 Free");
    assert_eq!(hy3.context_window, Some(262_144));
    assert_eq!(hy3.default_reasoning_level, Some(ReasoningEffort::None));
    assert_eq!(
        hy3.supported_reasoning_levels
            .iter()
            .map(|level| level.effort.clone())
            .collect::<Vec<_>>(),
        vec![
            ReasoningEffort::None,
            ReasoningEffort::Low,
            ReasoningEffort::High,
        ]
    );

    let kimi = openrouter_model("moonshotai/kimi-k3");
    assert_eq!(kimi.display_name, "OpenRouter Kimi K3");
    assert_eq!(kimi.context_window, Some(1_048_576));
    assert_eq!(kimi.default_reasoning_level, Some(ReasoningEffort::Max));
    assert_eq!(
        kimi.supported_reasoning_levels
            .iter()
            .map(|level| level.effort.clone())
            .collect::<Vec<_>>(),
        vec![
            ReasoningEffort::Low,
            ReasoningEffort::High,
            ReasoningEffort::Max,
        ]
    );
    assert_eq!(
        kimi.input_modalities,
        vec![InputModality::Text, InputModality::Image]
    );
    assert_eq!(
        kimi.chat_completions.reasoning_protocol,
        ChatReasoningProtocol::PreservedRequired
    );
    assert!(
        kimi.description
            .as_deref()
            .unwrap_or_default()
            .contains("$3.00/M input, $0.30/M cached input, $15.00/M output")
    );

    for model in [grok, deepseek_pro, deepseek_flash, hy3, kimi] {
        assert_eq!(model.visibility, ModelVisibility::List);
        assert!(!model.supports_parallel_tool_calls);
        assert_standard_base(&model.base_instructions);
    }

    let meta = response
        .models
        .iter()
        .find(|model| model.slug == "muse-spark-1.1")
        .expect("bundled models.json should include Meta Muse Spark 1.1");
    assert_eq!(meta.context_window, Some(1_048_576));
    assert!(meta.apply_patch_tool_type.is_none());
    assert!(meta.supports_parallel_tool_calls);
    assert_standard_base(&meta.base_instructions);

    let claude_opus = response
        .models
        .iter()
        .find(|model| model.slug == "claude-opus-5")
        .expect("bundled models.json should include Claude Opus 5");

    assert_eq!(claude_opus.display_name, "Claude Opus 5");
    assert_eq!(claude_opus.context_window, Some(1_000_000));
    assert_eq!(
        claude_opus.default_reasoning_level,
        Some(ReasoningEffort::High)
    );
    assert_eq!(
        claude_opus
            .supported_reasoning_levels
            .iter()
            .map(|level| level.effort.clone())
            .collect::<Vec<_>>(),
        vec![
            ReasoningEffort::Low,
            ReasoningEffort::Medium,
            ReasoningEffort::High,
            ReasoningEffort::XHigh,
            ReasoningEffort::Max,
        ]
    );
    assert_eq!(claude_opus.visibility, ModelVisibility::List);
    assert!(
        response.models.iter().all(|model| !matches!(
            model.slug.as_str(),
            "claude-opus-4-8" | "claude-opus-4-8-plan"
        )),
        "deprecated Claude Opus 4.8 variants must not appear in the bundled catalog"
    );

    for (slug, display_name, description) in [
        (
            "claude-fable-5-1-plan",
            "Claude Fable 5.1 Plan",
            "Claude Fable 5.1 through Claude Code subscription auth in Corbanu Terminal.",
        ),
        (
            "claude-fable-5-1",
            "Claude Fable 5.1",
            "Claude Fable 5.1 through the Anthropic Messages API.",
        ),
    ] {
        let model = response
            .models
            .iter()
            .find(|model| model.slug == slug)
            .unwrap_or_else(|| panic!("bundled models.json should include {display_name}"));
        assert_eq!(
            (
                model.display_name.as_str(),
                model.description.as_deref(),
                model.visibility,
            ),
            (display_name, Some(description), ModelVisibility::List,)
        );
    }

    let claude_fable_plan = response
        .models
        .iter()
        .find(|model| model.slug == "claude-fable-5-plan")
        .expect("bundled models.json should include Claude Fable 5 Plan");

    assert_eq!(claude_fable_plan.display_name, "Claude Fable 5 Plan");
    assert_eq!(
        claude_fable_plan.description.as_deref(),
        Some("Claude Fable 5 through Claude Code subscription auth in Corbanu Terminal.")
    );
    assert_eq!(claude_fable_plan.context_window, Some(1_000_000));
    assert_eq!(
        claude_fable_plan.default_reasoning_level,
        Some(ReasoningEffort::High)
    );
    assert_eq!(
        claude_fable_plan
            .supported_reasoning_levels
            .iter()
            .map(|level| level.effort.clone())
            .collect::<Vec<_>>(),
        vec![
            ReasoningEffort::Low,
            ReasoningEffort::Medium,
            ReasoningEffort::High,
            ReasoningEffort::XHigh,
            ReasoningEffort::Max,
        ]
    );
    assert_eq!(claude_fable_plan.visibility, ModelVisibility::List);

    let claude_fable = response
        .models
        .iter()
        .find(|model| model.slug == "claude-fable-5")
        .expect("bundled models.json should include Claude Fable 5");

    assert_eq!(claude_fable.display_name, "Claude Fable 5");
    assert_eq!(
        claude_fable.description.as_deref(),
        Some("Claude Fable 5 through the Anthropic Messages API.")
    );
    assert_eq!(claude_fable.context_window, Some(1_000_000));
    assert_eq!(
        claude_fable.default_reasoning_level,
        Some(ReasoningEffort::High)
    );
    assert_eq!(claude_fable.visibility, ModelVisibility::List);

    for slug in [
        "minimax/minimax-m3",
        "openrouter/owl-alpha",
        "google/gemini-3.5-flash",
        "zai-org/GLM-5.2",
        "zai/glm-5.2",
        "zai/glm-5.2-fast",
    ] {
        assert!(
            response.models.iter().any(|model| model.slug == slug),
            "bundled models.json should include {slug}"
        );
    }

    let baseten_glm = response
        .models
        .iter()
        .find(|model| model.slug == "zai-org/GLM-5.2")
        .expect("bundled models.json should include Baseten GLM 5.2");

    assert_eq!(baseten_glm.display_name, "Baseten GLM 5.2");
    assert_eq!(baseten_glm.context_window, Some(1_048_576));
    assert_eq!(baseten_glm.default_reasoning_level, None);
    assert!(baseten_glm.supported_reasoning_levels.is_empty());
    assert_eq!(baseten_glm.visibility, ModelVisibility::List);
    assert!(
        baseten_glm
            .description
            .as_deref()
            .unwrap_or_default()
            .contains("$1.40/M input, $0.14/M cached input, $4.40/M output")
    );

    let vercel_glm = response
        .models
        .iter()
        .find(|model| model.slug == "zai/glm-5.2")
        .expect("bundled models.json should include Vercel GLM 5.2");

    assert_eq!(vercel_glm.display_name, "Vercel GLM 5.2");
    assert_eq!(vercel_glm.context_window, Some(1_048_576));
    assert_eq!(vercel_glm.max_output_tokens, Some(128_000));
    assert_eq!(
        vercel_glm.default_reasoning_level,
        Some(ReasoningEffort::Medium)
    );
    assert_eq!(
        vercel_glm
            .supported_reasoning_levels
            .iter()
            .map(|level| level.effort.clone())
            .collect::<Vec<_>>(),
        vec![ReasoningEffort::Medium, ReasoningEffort::XHigh]
    );
    assert_eq!(vercel_glm.visibility, ModelVisibility::List);
    assert!(
        vercel_glm
            .description
            .as_deref()
            .unwrap_or_default()
            .contains("$1.40/M input, $0.26/M cached input, $4.40/M output")
    );

    let vercel_fast = response
        .models
        .iter()
        .find(|model| model.slug == "zai/glm-5.2-fast")
        .expect("bundled models.json should include Vercel GLM 5.2 Fast");

    assert_eq!(vercel_fast.display_name, "Vercel GLM 5.2 Fast");
    assert_eq!(vercel_fast.context_window, Some(1_048_576));
    assert_eq!(vercel_fast.max_output_tokens, Some(128_000));
    assert_eq!(
        vercel_fast.default_reasoning_level,
        Some(ReasoningEffort::Medium)
    );
    assert_eq!(
        vercel_fast
            .supported_reasoning_levels
            .iter()
            .map(|level| level.effort.clone())
            .collect::<Vec<_>>(),
        vec![ReasoningEffort::Medium, ReasoningEffort::XHigh]
    );
    assert_eq!(vercel_fast.visibility, ModelVisibility::List);
    assert!(
        vercel_fast
            .description
            .as_deref()
            .unwrap_or_default()
            .contains("$2.10/M input, $0.21/M cached input, $6.60/M output")
    );

    let openrouter_gemini = response
        .models
        .iter()
        .find(|model| model.slug == "google/gemini-3.5-flash")
        .expect("bundled models.json should include OpenRouter Gemini 3.5 Flash");

    assert_eq!(openrouter_gemini.default_reasoning_level, None);
    assert_eq!(
        openrouter_gemini
            .supported_reasoning_levels
            .iter()
            .map(|level| level.effort.clone())
            .collect::<Vec<_>>(),
        vec![
            ReasoningEffort::Minimal,
            ReasoningEffort::Low,
            ReasoningEffort::Medium,
            ReasoningEffort::High,
        ]
    );
}

#[test]
fn bundled_models_json_contains_direct_deepseek_flash() {
    let response = crate::bundled_models_response()
        .unwrap_or_else(|err| panic!("bundled models.json should parse: {err}"));
    let deepseek = response
        .models
        .iter()
        .find(|model| model.slug == "deepseek-v4-flash")
        .expect("bundled models.json should include direct DeepSeek V4 Flash");

    assert_eq!(deepseek.display_name, "DeepSeek V4 Flash 0731 (Direct)");
    assert_eq!(deepseek.context_window, Some(1_048_576));
    assert_eq!(deepseek.max_context_window, Some(1_048_576));
    assert_eq!(deepseek.max_output_tokens, Some(384_000));
    assert_eq!(
        deepseek.default_reasoning_level,
        Some(ReasoningEffort::High)
    );
    assert_eq!(
        deepseek
            .supported_reasoning_levels
            .iter()
            .map(|level| level.effort.clone())
            .collect::<Vec<_>>(),
        vec![ReasoningEffort::High, ReasoningEffort::Max]
    );
    assert_eq!(
        deepseek.orchestration,
        Some(ModelOrchestrationMetadata::Eligible {
            provider_id: "deepseek".to_string(),
            capability: ModelCapabilityTier::Fast,
            billing: ModelBilling::Metered {
                input_milli_usd_per_million_tokens: 140,
                output_milli_usd_per_million_tokens: 280,
                cached_input_milli_usd_per_million_tokens: Some(3),
            },
        })
    );
    assert_standard_base(&deepseek.base_instructions);
}
