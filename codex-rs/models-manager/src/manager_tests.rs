use super::*;
use crate::ModelsManagerConfig;
use crate::cache::FileModelsCache;
use crate::cache::ModelsCache;
use crate::cache::ModelsCacheEntry;
use crate::cache::ModelsCacheError;
use crate::cache::ModelsCacheFuture;
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
use codex_protocol::error::CodexErr;
use codex_protocol::openai_models::ChatReasoningEffortProtocol;
use codex_protocol::openai_models::ChatReasoningProtocol;
use codex_protocol::openai_models::InputModality;
use codex_protocol::openai_models::LongContextRates;
use codex_protocol::openai_models::MeteredRates;
use codex_protocol::openai_models::ModelAccessPrograms;
use codex_protocol::openai_models::ModelBilling;
use codex_protocol::openai_models::ModelCapabilityTier;
use codex_protocol::openai_models::ModelOrchestrationMetadata;
use codex_protocol::openai_models::ModelOrigin;
use codex_protocol::openai_models::ModelVisibility;
use codex_protocol::openai_models::ModelsResponse;
use codex_protocol::openai_models::ReasoningEffort;
use codex_protocol::openai_models::ReasoningEffortPreset;
use codex_protocol::openai_models::UtcHourWindow;
use codex_protocol::openai_models::WeekdaySet;
use codex_protocol::turn_input::CyberAccessProgram;
use pretty_assertions::assert_eq;
use serde_json::json;
use std::collections::VecDeque;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use tempfile::tempdir;

#[path = "api_key_discovery_tests.rs"]
mod api_key_discovery_tests;

#[path = "cache_identity_tests.rs"]
mod cache_identity_tests;

#[path = "model_info_overrides_tests.rs"]
mod model_info_overrides_tests;

#[path = "catalog_origin_tests.rs"]
mod catalog_origin_tests;

const DEFAULT_HTTP_CLIENT_FACTORY: HttpClientFactory =
    HttpClientFactory::new(OutboundProxyPolicy::ReqwestDefault);
const STANDARD_BASE: &str = include_str!("../../core/assets/agent/builtins/standard_base.md");
const GLM_FLASH_WORK_PATTERN: &str =
    include_str!("../../core/assets/agent/builtins/glm_flash_work_pattern.md");
const STANDARD_BASE_OUTCOME_MARKER: &str = "inspect code before changing it, keep edits scoped";
const STANDARD_BASE_EVIDENCE_MARKER: &str = "only narrate when needed";
const OLD_STANDARD_BASE_MARKER: &str = "Narrate as you work";
const GPT55_GUIDE_MARKER: &str = "vivid inner life";

/// Corbanu keeps the bundled catalog beside a provider's list; for slugs the
/// bundle does not name, the merged list is the bundle followed by them.
fn with_bundled(models: Vec<ModelInfo>) -> Vec<ModelInfo> {
    let mut merged = load_remote_models_from_file().expect("bundled models should parse");
    merged.extend(models);
    merged
}

fn instructions(model: &ModelInfo) -> &str {
    model
        .model_messages
        .as_ref()
        .and_then(|messages| messages.instructions_template.as_deref())
        .unwrap_or_default()
}

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
            "model_messages": {
                "instructions_template": "base instructions",
                "approvals": null,
                "auto_review": null,
                "permissions": null
            },
            "support_verbosity": false,
            "default_verbosity": null,
            "apply_patch_tool_type": null,
            "truncation_policy": {"mode": "bytes", "limit": 10_000},
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

#[derive(Debug, Default)]
struct TestModelsEndpoint {
    has_command_auth: bool,
    uses_codex_backend: bool,
    responses: Mutex<VecDeque<CoreResult<Vec<ModelInfo>>>>,
    etag: Option<String>,
    fetch_count: AtomicUsize,
    observed_proxy_policy: Mutex<Option<OutboundProxyPolicy>>,
}

#[derive(Debug, Default)]
struct TestModelsCache {
    entry: Mutex<Option<ModelsCacheEntry>>,
    load_error: bool,
    store_error: bool,
    stored_entries: Mutex<Vec<ModelsCacheEntry>>,
}

impl TestModelsCache {
    fn with_entry(entry: ModelsCacheEntry) -> Arc<Self> {
        Arc::new(Self {
            entry: Mutex::new(Some(entry)),
            load_error: false,
            store_error: false,
            stored_entries: Mutex::new(Vec::new()),
        })
    }

    fn failing(load_error: bool, store_error: bool) -> Arc<Self> {
        Arc::new(Self {
            entry: Mutex::new(None),
            load_error,
            store_error,
            stored_entries: Mutex::new(Vec::new()),
        })
    }

    fn stored_entries(&self) -> Vec<ModelsCacheEntry> {
        self.stored_entries
            .lock()
            .expect("stored entries lock should not be poisoned")
            .clone()
    }
}

impl ModelsCache for TestModelsCache {
    fn load<'a>(
        &'a self,
        _client_version: &'a str,
    ) -> ModelsCacheFuture<'a, Result<Option<ModelsCacheEntry>, ModelsCacheError>> {
        Box::pin(async move {
            if self.load_error {
                return Err(ModelsCacheError::new("test load failure"));
            }
            Ok(self
                .entry
                .lock()
                .expect("cache entry lock should not be poisoned")
                .clone())
        })
    }

    fn store<'a>(
        &'a self,
        entry: &'a ModelsCacheEntry,
    ) -> ModelsCacheFuture<'a, Result<(), ModelsCacheError>> {
        Box::pin(async move {
            if self.store_error {
                return Err(ModelsCacheError::new("test store failure"));
            }
            *self.entry.lock().unwrap() = Some(entry.clone());
            self.stored_entries
                .lock()
                .expect("stored entries lock should not be poisoned")
                .push(entry.clone());
            Ok(())
        })
    }

    fn refresh_ttl<'a>(
        &'a self,
        _client_version: &'a str,
        _identity: &'a str,
        _etag: &'a str,
    ) -> ModelsCacheFuture<'a, Result<(), ModelsCacheError>> {
        Box::pin(async move {
            let refreshed = {
                let mut entry = self
                    .entry
                    .lock()
                    .expect("cache entry lock should not be poisoned");
                let entry = entry
                    .as_mut()
                    .ok_or_else(|| ModelsCacheError::new("cache not found"))?;
                entry.fetched_at = Utc::now();
                entry.clone()
            };
            self.store(&refreshed).await
        })
    }
}

impl TestModelsEndpoint {
    fn new(responses: Vec<Vec<ModelInfo>>) -> Arc<Self> {
        Arc::new(Self {
            uses_codex_backend: true,
            responses: Mutex::new(responses.into_iter().map(Ok).collect()),
            ..Self::default()
        })
    }

    fn without_refresh(responses: Vec<Vec<ModelInfo>>) -> Arc<Self> {
        Arc::new(Self {
            responses: Mutex::new(responses.into_iter().map(Ok).collect()),
            ..Self::default()
        })
    }

    fn with_command_auth(responses: Vec<CoreResult<Vec<ModelInfo>>>) -> Arc<Self> {
        Arc::new(Self {
            has_command_auth: true,
            responses: Mutex::new(responses.into()),
            ..Self::default()
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

    async fn list_models(&self) -> CoreResult<ModelsEndpointResponse> {
        self.fetch_count.fetch_add(1, Ordering::SeqCst);
        let models = self
            .responses
            .lock()
            .expect("responses lock should not be poisoned")
            .pop_front()
            .unwrap_or_else(|| Ok(Vec::new()))?;
        Ok(ModelsEndpointResponse {
            models,
            etag: self.etag.clone(),
            identity: self.identity().expect("test endpoint identity"),
        })
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
    fn supports_api_key_models(&self) -> bool {
        true
    }

    fn identity(&self) -> Option<String> {
        Some("test-provider".to_string())
    }

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
    ) -> ModelsEndpointFuture<'a, CoreResult<ModelsEndpointResponse>> {
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

async fn mutate_file_cache_for_test<F>(codex_home: &Path, f: F)
where
    F: FnOnce(&mut ModelsCacheEntry),
{
    let client_version = crate::client_version_to_whole();
    let cache = FileModelsCache::new(codex_home.join(MODEL_CACHE_FILE), DEFAULT_MODEL_CACHE_TTL);
    let mut entry = cache
        .load(&client_version)
        .await
        .expect("cache load succeeds")
        .expect("cache entry exists");
    f(&mut entry);
    cache.store(&entry).await.expect("cache store succeeds");
}

fn static_manager_for_tests(model_catalog: ModelsResponse) -> StaticModelsManager {
    StaticModelsManager::new(
        /*auth_manager*/ None,
        model_catalog,
        ModelOrigin::ExplicitConfig,
    )
}

#[tokio::test]
async fn file_cache_implements_models_cache_contract() {
    let codex_home = tempdir().expect("temp dir");
    let cache = FileModelsCache::new(
        codex_home.path().join(MODEL_CACHE_FILE),
        DEFAULT_MODEL_CACHE_TTL,
    );
    let client_version = crate::client_version_to_whole();
    let entry = ModelsCacheEntry {
        identity: Some("test-provider".to_string()),
        fetched_at: Utc::now(),
        etag: Some("file-etag".to_string()),
        client_version: Some(client_version.clone()),
        models: vec![ModelInfo {
            available_access_programs: Some(ModelAccessPrograms {
                cyber: vec![
                    CyberAccessProgram::Standard,
                    CyberAccessProgram::DaybreakBlue,
                ],
            }),
            ..remote_model("file-cached", "File Cached", /*priority*/ 0)
        }],
    };

    cache.store(&entry).await.expect("cache store succeeds");

    assert_eq!(
        cache
            .load(&client_version)
            .await
            .expect("cache load succeeds"),
        Some(entry)
    );
}

#[tokio::test]
async fn file_cache_refresh_ttl_renews_expired_entry_without_serving_it_stale() {
    let codex_home = tempdir().expect("temp dir");
    let cache = FileModelsCache::new(
        codex_home.path().join(MODEL_CACHE_FILE),
        DEFAULT_MODEL_CACHE_TTL,
    );
    let client_version = crate::client_version_to_whole();
    let expired_at = Utc::now() - chrono::Duration::hours(1);
    let entry = ModelsCacheEntry {
        identity: Some("test-provider".to_string()),
        fetched_at: expired_at,
        etag: Some("expired-etag".to_string()),
        client_version: Some(client_version.clone()),
        models: vec![remote_model(
            "expired-file-cache",
            "Expired File Cache",
            /*priority*/ 0,
        )],
    };
    cache.store(&entry).await.expect("cache store succeeds");

    assert_eq!(
        cache
            .load(&client_version)
            .await
            .expect("cache load succeeds"),
        None,
        "an expired entry must not be returned before revalidation"
    );

    cache
        .refresh_ttl(
            &client_version,
            entry.identity.as_deref().unwrap(),
            entry.etag.as_deref().unwrap(),
        )
        .await
        .expect("TTL refresh succeeds");

    let refreshed = cache
        .load(&client_version)
        .await
        .expect("cache load succeeds")
        .expect("revalidated entry is fresh");
    assert!(refreshed.fetched_at > expired_at);
    let mut expected = entry;
    expected.fetched_at = refreshed.fetched_at;
    assert_eq!(refreshed, expected);
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

#[tokio::test]
async fn injected_cache_hit_avoids_remote_fetch() {
    let cached_models = vec![remote_model("cached", "Cached", /*priority*/ 0)];
    let cache = TestModelsCache::with_entry(ModelsCacheEntry {
        identity: Some("test-provider".to_string()),
        fetched_at: Utc::now(),
        etag: Some("cached-etag".to_string()),
        client_version: Some(crate::client_version_to_whole()),
        models: cached_models.clone(),
    });
    let endpoint = TestModelsEndpoint::new(vec![vec![remote_model(
        "remote", "Remote", /*priority*/ 0,
    )]]);
    let manager = OpenAiModelsManager::new_with_cache(
        cache,
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

    assert_eq!(catalog.models, with_bundled(cached_models));
    assert_eq!(endpoint.fetch_count(), 0);
}

#[tokio::test]
async fn injected_cache_read_error_falls_back_and_persists_remote_models() {
    let remote_models = vec![remote_model("remote", "Remote", /*priority*/ 0)];
    let cache = TestModelsCache::failing(/*load_error*/ true, /*store_error*/ false);
    let endpoint = TestModelsEndpoint::new(vec![remote_models.clone()]);
    let manager = OpenAiModelsManager::new_with_cache(
        cache.clone(),
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

    assert_eq!(catalog.models, with_bundled(remote_models.clone()));
    assert_eq!(endpoint.fetch_count(), 1);
    let stored_entries = cache.stored_entries();
    assert_eq!(
        stored_entries,
        vec![ModelsCacheEntry {
            identity: Some("test-provider".to_string()),
            fetched_at: stored_entries[0].fetched_at,
            etag: None,
            client_version: Some(crate::client_version_to_whole()),
            models: remote_models,
        }]
    );
}

#[tokio::test]
async fn injected_cache_write_error_does_not_fail_remote_refresh() {
    let remote_models = vec![remote_model("remote", "Remote", /*priority*/ 0)];
    let cache = TestModelsCache::failing(/*load_error*/ false, /*store_error*/ true);
    *cache.entry.lock().unwrap() = Some(ModelsCacheEntry {
        identity: Some("test-provider".to_string()),
        fetched_at: Utc::now(),
        etag: None,
        client_version: Some(crate::client_version_to_whole()),
        models: vec![remote_model("cached", "Cached", /*priority*/ 0)],
    });
    let endpoint = TestModelsEndpoint::new(vec![remote_models.clone()]);
    let manager = OpenAiModelsManager::new_with_cache(
        cache,
        endpoint.clone(),
        Some(AuthManager::from_auth_for_testing(
            CodexAuth::create_dummy_chatgpt_auth_for_testing(),
        )),
    );

    let catalog = manager
        .raw_model_catalog(RefreshStrategy::Online, DEFAULT_HTTP_CLIENT_FACTORY)
        .await;

    assert_eq!(catalog.models, with_bundled(remote_models.clone()));
    assert_eq!(
        manager
            .model_catalog_with_origins(RefreshStrategy::InMemory, DEFAULT_HTTP_CLIENT_FACTORY)
            .await,
        load_remote_models_from_file()
            .expect("bundled models should parse")
            .into_iter()
            .map(|model| CatalogModel {
                model: model.into(),
                origin: ModelOrigin::Bundled,
            })
            .chain(remote_models.into_iter().map(|model| CatalogModel {
                model: model.into(),
                origin: ModelOrigin::Provider,
            }))
            .collect::<Vec<_>>()
    );
    assert_eq!(endpoint.fetch_count(), 1);
}

#[tokio::test]
async fn injected_cache_ttl_refresh_preserves_cached_payload() {
    let cached_models = vec![remote_model("cached", "Cached", /*priority*/ 0)];
    let cached_at = Utc::now() - chrono::Duration::minutes(1);
    let cache = TestModelsCache::with_entry(ModelsCacheEntry {
        identity: Some("test-provider".to_string()),
        fetched_at: cached_at,
        etag: Some("cached-etag".to_string()),
        client_version: Some(crate::client_version_to_whole()),
        models: cached_models.clone(),
    });
    let manager = OpenAiModelsManager::new_with_cache(
        cache.clone(),
        TestModelsEndpoint::new(Vec::new()),
        Some(AuthManager::from_auth_for_testing(CodexAuth::from_api_key(
            "test-api-key",
        ))),
    );
    manager.set_api_key_model_discovery_enabled(/*enabled*/ true);

    manager
        .raw_model_catalog(
            RefreshStrategy::OnlineIfUncached,
            DEFAULT_HTTP_CLIENT_FACTORY,
        )
        .await;
    manager
        .refresh_if_new_etag("cached-etag".to_string(), DEFAULT_HTTP_CLIENT_FACTORY)
        .await;

    let stored_entries = cache.stored_entries();
    assert_eq!(stored_entries.len(), 1);
    assert!(stored_entries[0].fetched_at > cached_at);
    assert_eq!(stored_entries[0].etag.as_deref(), Some("cached-etag"));
    assert_eq!(
        stored_entries[0].client_version,
        Some(crate::client_version_to_whole())
    );
    assert_eq!(stored_entries[0].models, cached_models);
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
            oauth_provenance: None,
            login_owner_id: None,
            account_id: Some("account_id".to_string()),
        }),
        last_refresh: Some(Utc::now()),
        agent_identity: None,
        personal_access_token: None,
        bedrock_api_key: None,
        bedrock_access_keys: None,
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
async fn get_model_info_tracks_metadata_origin() {
    let codex_home = tempdir().expect("temp dir");
    let config = ModelsManagerConfig {
        base_instructions: Some("Custom instructions".to_string()),
        ..Default::default()
    };
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
    assert_eq!(known.origin, ModelOrigin::Bundled);
    assert_eq!(known.slug, known_slug);

    let unknown = manager
        .get_model_info("model-that-does-not-exist", &config)
        .await;
    assert_eq!(unknown.origin, ModelOrigin::Synthetic);
    assert_eq!(unknown.slug, "model-that-does-not-exist");
    assert_eq!(
        unknown
            .model_messages
            .as_ref()
            .and_then(|messages| messages.instructions_template.as_deref()),
        config.base_instructions.as_deref()
    );
}

#[tokio::test]
async fn get_model_info_applies_long_context_override_to_bundled_gpt_5_6_models() {
    let codex_home = tempdir().expect("temp dir");
    let manager = openai_manager_for_tests(
        codex_home.path().to_path_buf(),
        TestModelsEndpoint::new(Vec::new()),
    );
    let config = ModelsManagerConfig {
        model_context_window: Some(1_000_000),
        ..Default::default()
    };

    for slug in ["gpt-5.6-sol", "gpt-5.6-terra", "gpt-5.6-luna"] {
        let model_info = manager.get_model_info(slug, &config).await;
        let mut expected = manager
            .get_model_info(slug, &ModelsManagerConfig::default())
            .await;
        // Corbanu's GPT-5.6 rows stop at 372K.
        expected.context_window = Some(372_000);

        assert_eq!(model_info, expected);
    }
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
    assert_eq!(model_info.origin, ModelOrigin::ExplicitConfig);

    let config = ModelsManagerConfig {
        model_context_window: Some(128_000),
        ..Default::default()
    };
    let mut expected = model_info;
    expected.context_window = Some(128_000);
    assert_eq!(
        manager
            .get_model_info("gpt-overlay-experiment", &config)
            .await,
        expected
    );
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
    assert_eq!(model_info.origin, ModelOrigin::ExplicitConfig);
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
    assert_eq!(model_info.origin, ModelOrigin::ExplicitConfig);
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
    assert_eq!(model_info.origin, ModelOrigin::Synthetic);
}

#[tokio::test]
async fn corbanu_api_public_models_inherit_exact_catalog_metadata() {
    let codex_home = tempdir().expect("temp dir");
    let config = ModelsManagerConfig::default();
    let manager = openai_manager_for_tests(
        codex_home.path().to_path_buf(),
        TestModelsEndpoint::new(Vec::new()),
    );

    for (public_model, catalog_model) in [
        ("corbanu/glm-5.3-flash", "zai/glm-5.3-flash"),
        ("corbanu/glm-5.3", "zai/glm-5.3"),
        ("corbanu/gpt-5.6-luna", "gpt-5.6-luna"),
        ("corbanu/gpt-5.6-sol", "gpt-5.6-sol"),
        ("corbanu/kimi-k3", "vercel/moonshotai/kimi-k3"),
        ("corbanu/deepseek-v4-pro", "deepseek-v4-pro"),
        ("corbanu/deepseek-v4.1-flash", "corbanu/deepseek-v4.1-flash"),
    ] {
        let public_info = manager.get_model_info(public_model, &config).await;
        let catalog_info = manager.get_model_info(catalog_model, &config).await;

        assert_eq!(public_info.slug, public_model);
        assert_ne!(public_info.origin, ModelOrigin::Synthetic);
        assert_eq!(public_info.context_window, catalog_info.context_window);
        assert_eq!(
            public_info.max_output_tokens,
            catalog_info.max_output_tokens
        );
        assert_eq!(public_info.input_modalities, catalog_info.input_modalities);
        assert_eq!(
            public_info.supported_reasoning_levels,
            catalog_info.supported_reasoning_levels
        );
    }
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
async fn remote_overlay_keeps_retired_ambient_model_out_of_picker() {
    let endpoint = TestModelsEndpoint::new(vec![vec![remote_model(
        "moonshotai/kimi-k2.7-code",
        "Old cached Ambient Kimi",
        /*priority*/ 0,
    )]]);
    let codex_home = tempdir().expect("temp dir");
    let manager = openai_manager_for_tests(codex_home.path().to_path_buf(), endpoint);
    let models = manager
        .list_models(
            RefreshStrategy::OnlineIfUncached,
            DEFAULT_HTTP_CLIENT_FACTORY,
        )
        .await;
    assert_eq!(
        models
            .iter()
            .filter(|model| model.provider_id.as_deref() == Some("ambient") && model.show_in_picker)
            .map(|model| model.model.as_str())
            .collect::<Vec<_>>(),
        vec!["z-ai/glm-5.2"]
    );
}

async fn assert_catalogue_issue_128_saved_ids(manager: &dyn ModelsManager) {
    let presets = manager
        .list_models(RefreshStrategy::Offline, DEFAULT_HTTP_CLIENT_FACTORY)
        .await;
    let bundled = load_remote_models_from_file().expect("bundled models");
    for (slug, provider) in [
        ("x-ai/grok-4.6", "openrouter"),
        ("openrouter/owl-alpha", "openrouter"),
        ("x-ai/grok-4.5", "openrouter"),
        ("deepseek/deepseek-v4-pro", "openrouter"),
        ("deepseek/deepseek-v4-flash-0731", "openrouter"),
        ("gpt-5.5", "openai"),
    ] {
        let saved = Some(slug.to_string());
        let selected = manager
            .get_default_model(
                &saved,
                /*allow_provider_model_fallback*/ false,
                RefreshStrategy::Offline,
                DEFAULT_HTTP_CLIENT_FACTORY,
            )
            .await;
        let resolved = manager
            .get_model_info(&selected, &ModelsManagerConfig::default())
            .await;
        let original = bundled.iter().find(|model| model.slug == slug).unwrap();
        let preset = presets.iter().find(|model| model.model == slug).unwrap();
        assert_eq!(
            (
                selected.as_str(),
                resolved.slug.as_str(),
                preset.provider_id.as_deref(),
                preset.show_in_picker,
                preset.is_default,
                &resolved.orchestration,
                resolved.max_output_tokens,
                resolved.origin == ModelOrigin::Synthetic,
            ),
            (
                slug,
                slug,
                Some(provider),
                false,
                false,
                &original.orchestration,
                original.max_output_tokens,
                false,
            ),
            "saved compatibility contract for {slug}"
        );
        assert!(matches!(
            resolved.orchestration,
            Some(ModelOrchestrationMetadata::Disabled { .. })
        ));
    }
}

#[tokio::test]
async fn catalogue_issue_128_saved_ids_without_discovery() {
    let codex_home = tempdir().expect("temp dir");
    let endpoint = TestModelsEndpoint::without_refresh(Vec::new());
    let manager = openai_manager_for_tests(codex_home.path().to_path_buf(), endpoint.clone());
    manager
        .list_models(
            RefreshStrategy::OnlineIfUncached,
            DEFAULT_HTTP_CLIENT_FACTORY,
        )
        .await;
    assert_catalogue_issue_128_saved_ids(&manager).await;
    assert_eq!(endpoint.fetch_count(), 0);
    assert!(!codex_home.path().join(MODEL_CACHE_FILE).exists());
}

#[tokio::test]
async fn catalogue_issue_128_remote_and_cached_overlays_preserve_saved_ids() {
    let retired = load_remote_models_from_file()
        .expect("bundled models")
        .into_iter()
        .filter(|model| model.visibility == ModelVisibility::Hide)
        .collect::<Vec<_>>();
    assert!(!retired.is_empty());
    let mut advertised = retired
        .iter()
        .cloned()
        .map(|mut model| {
            model.visibility = ModelVisibility::List;
            model.orchestration = Some(ModelOrchestrationMetadata::Eligible {
                provider_id: "unrelated-provider".to_string(),
                capability: ModelCapabilityTier::Frontier,
                billing: ModelBilling::AuthDependent {
                    plan_relative_burn_millis: 1_000,
                    api_key_input_milli_usd_per_million_tokens: 1,
                    api_key_output_milli_usd_per_million_tokens: 1,
                    api_key_cached_input_milli_usd_per_million_tokens: None,
                    api_key_cache_write_milli_usd_per_million_tokens: None,
                    api_key_long_context: None,
                    api_key_valid_through_utc: None,
                },
            });
            model
        })
        .collect::<Vec<_>>();
    // A separate discovered model's output limit must not leak into a saved ID.
    let mut unrelated = remote_model("unrelated-capped-model", "Unrelated", /*priority*/ 0);
    unrelated.max_output_tokens = Some(8_192);
    advertised.push(unrelated);
    let codex_home = tempdir().expect("temp dir");
    let manager = openai_manager_for_tests(
        codex_home.path().to_path_buf(),
        TestModelsEndpoint::new(vec![advertised]),
    );
    let online = manager
        .list_models(RefreshStrategy::Online, DEFAULT_HTTP_CLIENT_FACTORY)
        .await;
    assert_catalogue_issue_128_saved_ids(&manager).await;
    let cached_endpoint = TestModelsEndpoint::new(vec![Vec::new()]);
    let cached_manager =
        openai_manager_for_tests(codex_home.path().to_path_buf(), cached_endpoint.clone());
    let cached = cached_manager
        .list_models(
            RefreshStrategy::OnlineIfUncached,
            DEFAULT_HTTP_CLIENT_FACTORY,
        )
        .await;
    assert_eq!(cached, online);
    assert_eq!(cached_endpoint.fetch_count(), 0);
    assert_catalogue_issue_128_saved_ids(&cached_manager).await;
    for retired_model in retired {
        let preset = online
            .iter()
            .find(|model| model.model == retired_model.slug)
            .expect("hidden metadata retained");
        assert!(!preset.show_in_picker, "{} resurrected", preset.model);
        let explicit = manager
            .get_model_info(&retired_model.slug, &ModelsManagerConfig::default())
            .await;
        assert_eq!(explicit.slug, retired_model.slug);
        assert_ne!(explicit.origin, ModelOrigin::Synthetic);
    }

    // Expire the actual persisted cache, then simulate discovery omitting the
    // retained IDs. Refetch must still resolve every saved ID from the bundle.
    mutate_file_cache_for_test(codex_home.path(), |cache| {
        cache.fetched_at = Utc::now() - chrono::Duration::hours(1);
    })
    .await;
    let refreshed_manager =
        openai_manager_for_tests(codex_home.path().to_path_buf(), cached_endpoint.clone());
    refreshed_manager
        .list_models(
            RefreshStrategy::OnlineIfUncached,
            DEFAULT_HTTP_CLIENT_FACTORY,
        )
        .await;
    assert_eq!(cached_endpoint.fetch_count(), 1);
    assert_catalogue_issue_128_saved_ids(&refreshed_manager).await;
    assert!(
        refreshed_manager
            .get_remote_models()
            .await
            .iter()
            .all(|model| model.slug != "unrelated-capped-model"),
        "expired discovery records must not survive the empty refresh"
    );
}

/// The explicit disposition of each #128 retained ID. Sources and dates are in
/// `qa/release/unreleased/model128-compatibility-contract.md`; change both together.
#[derive(Clone, Copy, Debug, PartialEq)]
enum RetainedIdDisposition {
    /// Still served upstream; the row carries a verified or deliberately conservative cap.
    Served,
    /// Still served upstream on the Responses wire, which never sends an output cap, so
    /// the provider's own limit applies and the row deliberately states none.
    ServedCapOmitted,
    /// No longer served upstream. The row stays, keeping its last cap and carrying a
    /// retirement note, so a saved ID resolves to itself and never to another model.
    RetiredUpstream,
}

#[derive(Debug, PartialEq)]
struct RetainedIdRow {
    provider_id: String,
    visibility: ModelVisibility,
    disabled: bool,
    max_output_tokens: Option<i64>,
    reason_says_retired: bool,
    description_says_retired: bool,
}

#[test]
fn catalogue_issue_128_retained_ids_have_explicit_dispositions() {
    use codex_model_provider_info::WireApi;
    use codex_model_provider_info::built_in_model_providers;
    use codex_model_provider_info::default_model_max_output_tokens_for_provider;

    let bundled = load_remote_models_from_file().expect("bundled models");
    let providers = built_in_model_providers(/*openai_base_url*/ None);
    for (slug, provider_id, disposition, max_output_tokens) in [
        (
            "x-ai/grok-4.6",
            "openrouter",
            RetainedIdDisposition::Served,
            Some(128_000),
        ),
        (
            "x-ai/grok-4.5",
            "openrouter",
            RetainedIdDisposition::Served,
            Some(128_000),
        ),
        (
            "deepseek/deepseek-v4-pro",
            "openrouter",
            RetainedIdDisposition::Served,
            Some(384_000),
        ),
        (
            "deepseek/deepseek-v4-flash-0731",
            "openrouter",
            RetainedIdDisposition::Served,
            Some(384_000),
        ),
        (
            "openrouter/owl-alpha",
            "openrouter",
            RetainedIdDisposition::RetiredUpstream,
            Some(262_144),
        ),
        (
            "gpt-5.5",
            "openai",
            RetainedIdDisposition::ServedCapOmitted,
            None,
        ),
    ] {
        let row = bundled
            .iter()
            .find(|model| model.slug == slug)
            .unwrap_or_else(|| panic!("{slug} must stay in the bundled catalogue"));
        let (row_provider, reason) = match &row.orchestration {
            Some(ModelOrchestrationMetadata::Disabled {
                provider_id,
                reason,
                ..
            }) => (provider_id.clone(), Some(reason.as_str())),
            Some(orchestration) => (orchestration.provider_id().to_string(), None),
            None => (String::new(), None),
        };
        let retired = disposition == RetainedIdDisposition::RetiredUpstream;
        assert_eq!(
            RetainedIdRow {
                provider_id: row_provider,
                visibility: row.visibility,
                disabled: reason.is_some(),
                max_output_tokens: row.max_output_tokens,
                reason_says_retired: reason
                    .is_some_and(|reason| reason.starts_with("retired upstream:")),
                description_says_retired: row
                    .description
                    .as_deref()
                    .is_some_and(|description| description.contains("Retired:")),
            },
            RetainedIdRow {
                provider_id: provider_id.to_string(),
                visibility: ModelVisibility::Hide,
                disabled: true,
                max_output_tokens,
                reason_says_retired: retired,
                description_says_retired: retired,
            },
            "disposition of {slug}"
        );
        if disposition == RetainedIdDisposition::ServedCapOmitted {
            assert_eq!(
                (
                    providers.get(provider_id).map(|provider| provider.wire_api),
                    default_model_max_output_tokens_for_provider(provider_id, slug),
                ),
                (Some(WireApi::Responses), None),
                "{slug}: omitting the cap is only safe on a wire that never sends one, \
                 with no provider default supplying a cap the row deliberately omits"
            );
        }
    }
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
        billing: None,
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
                api_key_input_milli_usd_per_million_tokens: 4_000,
                api_key_output_milli_usd_per_million_tokens: 20_000,
                api_key_cached_input_milli_usd_per_million_tokens: Some(400),
                api_key_cache_write_milli_usd_per_million_tokens: Some(5_000),
                api_key_long_context: Some(LongContextRates {
                    above_input_tokens: 272_000,
                    input_milli_usd_per_million_tokens: 8_000,
                    output_milli_usd_per_million_tokens: 30_000,
                    cached_input_milli_usd_per_million_tokens: Some(800),
                    cache_write_milli_usd_per_million_tokens: Some(10_000),
                }),
                api_key_valid_through_utc: Some("2026-11-21T23:59:59.999Z".to_string()),
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
    // Corbanu keeps the bundled catalog beside a ChatGPT remote list.
    assert_eq!(model_info.origin, ModelOrigin::Bundled);
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
async fn chatgpt_catalog_keeps_bundled_openai_models_when_remote_omits_them() {
    let codex_home = tempdir().expect("temp dir");
    let endpoint = TestModelsEndpoint::new(vec![Vec::new()]);
    let manager = openai_manager_for_tests(codex_home.path().to_path_buf(), endpoint);

    let available = manager
        .list_models(RefreshStrategy::Online, DEFAULT_HTTP_CLIENT_FACTORY)
        .await;
    let picker_visibility = available
        .iter()
        .filter(|model| model.provider_id.as_deref() == Some("openai"))
        .map(|model| (model.model.as_str(), model.show_in_picker))
        .collect::<Vec<_>>();

    for slug in [
        "gpt-6-astra",
        "gpt-6-sol",
        "gpt-6-luna",
        "gpt-5.6-sol",
        "gpt-5.6-terra",
        "gpt-5.6-luna",
    ] {
        assert!(
            picker_visibility.contains(&(slug, true)),
            "{slug} should remain available unless the server explicitly hides it"
        );
    }

    assert!(
        available
            .iter()
            .any(|model| model.model == "gpt-6-sol" && model.is_default),
        "GPT-6 Sol should be the default visible preset"
    );
}

#[tokio::test]
async fn chatgpt_catalog_honors_explicit_remote_hiding_for_current_openai_models() {
    let mut remote_models = crate::bundled_models_response()
        .expect("bundled models should parse")
        .models
        .into_iter()
        .filter(|model| model.slug == "gpt-6-astra" || model.slug.starts_with("gpt-5.6-"))
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
            .filter(|model| model.model == "gpt-6-astra" || model.model.starts_with("gpt-5.6-"))
            .all(|model| !model.show_in_picker),
        "an explicit hidden response must override bundled visibility"
    );
}

#[tokio::test]
async fn default_falls_back_to_gpt_5_6_sol_when_the_server_hides_gpt_6_sol() {
    let mut remote_models = crate::bundled_models_response()
        .expect("bundled models should parse")
        .models
        .into_iter()
        .filter(|model| model.slug == "gpt-6-sol")
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
    let defaults = available
        .iter()
        .filter(|model| model.is_default)
        .map(|model| model.model.as_str())
        .collect::<Vec<_>>();
    assert_eq!(defaults, vec!["gpt-5.6-sol"]);
}

#[test]
fn gpt_6_sol_defaults_to_high_effort() {
    let sol = crate::bundled_models_response()
        .expect("bundled models should parse")
        .models
        .into_iter()
        .find(|model| model.slug == "gpt-6-sol")
        .expect("bundled gpt-6-sol");
    assert_eq!(sol.default_reasoning_level, Some(ReasoningEffort::High));
}

#[tokio::test]
async fn default_ignores_a_server_list_that_names_neither_sol_candidate() {
    let remote_models = crate::bundled_models_response()
        .expect("bundled models should parse")
        .models
        .into_iter()
        .filter(|model| model.slug == "gpt-5.6-luna")
        .collect::<Vec<_>>();
    let codex_home = tempdir().expect("temp dir");
    let endpoint = TestModelsEndpoint::new(vec![remote_models]);
    let manager = openai_manager_for_tests(codex_home.path().to_path_buf(), endpoint);

    let available = manager
        .list_models(RefreshStrategy::Online, DEFAULT_HTTP_CLIENT_FACTORY)
        .await;
    let defaults = available
        .iter()
        .filter(|model| model.is_default)
        .map(|model| model.model.as_str())
        .collect::<Vec<_>>();
    assert_eq!(defaults, vec!["gpt-6-sol"]);
}

#[tokio::test]
async fn astra_remote_overlay_keeps_manual_policy_and_supported_efforts() {
    let bundled = crate::bundled_models_response()
        .expect("bundled catalog")
        .models
        .into_iter()
        .find(|model| model.slug == "gpt-6-astra")
        .expect("Astra catalog entry");
    let mut remote = bundled.clone();
    remote.supported_reasoning_levels = vec![ReasoningEffortPreset {
        effort: ReasoningEffort::None,
        description: "unsupported remote effort".to_string(),
    }];
    remote.default_reasoning_level = Some(ReasoningEffort::None);
    remote.orchestration = None;
    remote.max_output_tokens = None;
    let codex_home = tempdir().expect("temp dir");
    let manager = openai_manager_for_tests(
        codex_home.path().to_path_buf(),
        TestModelsEndpoint::new(vec![vec![remote]]),
    );
    let available = manager
        .list_models(RefreshStrategy::Online, DEFAULT_HTTP_CLIENT_FACTORY)
        .await;
    let actual = available
        .into_iter()
        .find(|model| model.model == "gpt-6-astra")
        .expect("Astra remains selectable");
    assert_eq!(actual, ModelPreset::from(bundled.clone()));
    let resolved = manager
        .get_model_info("gpt-6-astra", &ModelsManagerConfig::default())
        .await;
    assert_eq!(resolved.max_output_tokens, bundled.max_output_tokens);
    assert_ne!(resolved.origin, ModelOrigin::Synthetic);
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
    // The server did not advertise gpt-6-sol to this account, so the bundled
    // entry stays selectable but the default falls back to gpt-5.6-sol.
    assert!(
        available
            .iter()
            .any(|model| model.model == "gpt-5.6-sol" && model.is_default)
    );
    assert!(
        available
            .iter()
            .any(|model| model.model == "gpt-6-sol" && !model.is_default)
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
async fn refresh_available_models_keeps_merging_for_custom_api_auth() {
    let remote_models = vec![remote_model(
        "api-auth-visible-remote",
        "API Auth Visible",
        /*priority*/ 0,
    )];
    let codex_home = tempdir().expect("temp dir");
    let endpoint = Arc::new(TestModelsEndpoint {
        has_command_auth: true,
        responses: Mutex::new(vec![Ok(remote_models.clone())].into()),
        ..TestModelsEndpoint::default()
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
async fn authoritative_catalog_preserves_hidden_models_and_matches_exact_ids() {
    let model =
        remote_model_with_visibility("gpt-6-sol", "Provider Model", /*priority*/ 0, "hide");
    let endpoint = TestModelsEndpoint::with_command_auth(vec![Ok(vec![model.clone()])]);
    let manager = OpenAiModelsManager::new_without_cache(endpoint, /*auth_manager*/ None)
        .with_provider_catalog();
    assert_eq!(
        manager
            .raw_model_catalog(RefreshStrategy::Online, DEFAULT_HTTP_CLIENT_FACTORY)
            .await
            .models,
        vec![model]
    );
    for unknown in ["gpt-6-sol-other", "namespace/gpt-6-sol"] {
        assert_eq!(
            manager
                .get_model_info(unknown, &ModelsManagerConfig::default())
                .await,
            CatalogModel {
                model: model_info::model_info_from_slug(unknown).into(),
                origin: ModelOrigin::Synthetic
            }
        );
    }
}

#[tokio::test]
async fn authoritative_catalog_access_rejection_invalidates_cache_until_refresh_succeeds() {
    for error in [
        catalog_http_error(/*status*/ 401),
        catalog_http_error(/*status*/ 403),
        CodexErr::RefreshTokenFailed(codex_protocol::error::RefreshTokenFailedError::new(
            codex_protocol::error::RefreshTokenFailedReason::Revoked,
            "credentials revoked",
        )),
        CodexErrorDetails::GatewayLoginRequired(
            codex_login::GatewayAuthError::LoginRequired.to_string(),
        )
        .into(),
    ] {
        // Like a native TTL backend, this cache may return an invalidated stored entry.
        let cache = Arc::new(TestModelsCache::default());
        let models = vec![remote_model(
            "provider-model",
            "Provider Model",
            /*priority*/ 0,
        )];
        let endpoint = TestModelsEndpoint::with_command_auth(vec![
            Ok(models.clone()),
            Err(error),
            Ok(models.clone()),
        ]);
        let manager = OpenAiModelsManager::new_with_cache(
            cache.clone(),
            endpoint.clone(),
            /*auth_manager*/ None,
        )
        .with_provider_catalog();
        assert_eq!(
            manager
                .raw_model_catalog(RefreshStrategy::Online, DEFAULT_HTTP_CLIENT_FACTORY)
                .await
                .models,
            models
        );
        for strategy in [RefreshStrategy::Online, RefreshStrategy::Offline] {
            assert_eq!(
                manager
                    .raw_model_catalog(strategy, DEFAULT_HTTP_CLIENT_FACTORY)
                    .await,
                ModelsResponse::default()
            );
        }
        let restarted =
            OpenAiModelsManager::new_with_cache(cache, endpoint, /*auth_manager*/ None)
                .with_provider_catalog();
        assert_eq!(
            restarted
                .raw_model_catalog(RefreshStrategy::Offline, DEFAULT_HTTP_CLIENT_FACTORY)
                .await,
            ModelsResponse::default()
        );
        assert_eq!(
            restarted
                .raw_model_catalog(
                    RefreshStrategy::OnlineIfUncached,
                    DEFAULT_HTTP_CLIENT_FACTORY,
                )
                .await
                .models,
            models
        );
    }
}

fn catalog_http_error(status: u16) -> CodexErr {
    CodexErr::UnexpectedStatus(codex_protocol::error::UnexpectedResponseError {
        status: status.try_into().expect("valid status"),
        body: String::new(),
        user_message: None,
        url: None,
        cf_ray: None,
        request_id: None,
        identity_authorization_error: None,
        identity_error_code: None,
    })
}

#[tokio::test]
async fn authoritative_catalog_refresh_failure_preserves_snapshot_and_freshness() {
    for error in [
        CodexErr::RequestTimeout,
        CodexErr::Stream("network unavailable".to_string()),
        catalog_http_error(/*status*/ 429),
        catalog_http_error(/*status*/ 503),
        catalog_http_error(/*status*/ 404),
        CodexErr::InvalidRequest("malformed catalog".to_string()),
        CodexErr::InvalidRequest(codex_login::GatewayAuthError::LoginInProgress.to_string()),
    ] {
        let cache = Arc::new(TestModelsCache::default());
        let mut model = remote_model("provider-model", "Provider Model", /*priority*/ 0);
        model.supports_search_tool = true;
        model.context_window = Some(1_000_000);
        let models = vec![model];
        let endpoint = Arc::new(TestModelsEndpoint {
            has_command_auth: true,
            etag: Some("last-successful-etag".into()),
            responses: Mutex::new(vec![Ok(models.clone()), Err(error), Ok(Vec::new())].into()),
            ..TestModelsEndpoint::default()
        });
        let manager = OpenAiModelsManager::new_with_cache(
            cache.clone(),
            endpoint.clone(),
            /*auth_manager*/ None,
        )
        .with_provider_catalog();
        let initial = manager
            .raw_model_catalog(RefreshStrategy::Online, DEFAULT_HTTP_CLIENT_FACTORY)
            .await;
        let initial_info = manager
            .get_model_info("provider-model", &ModelsManagerConfig::default())
            .await;
        let stored = cache.stored_entries();
        assert_eq!(initial.models, models);

        manager
            .refresh_if_new_etag("new-etag".to_string(), DEFAULT_HTTP_CLIENT_FACTORY)
            .await;
        assert_eq!(manager.get_remote_models().await, models);
        assert_eq!(
            manager
                .get_model_info("provider-model", &ModelsManagerConfig::default())
                .await,
            initial_info
        );
        assert_eq!(manager.remote_models.read().await.entry, stored[0]);
        assert_eq!(cache.stored_entries(), stored);

        let restarted = OpenAiModelsManager::new_with_cache(
            cache.clone(),
            endpoint.clone(),
            /*auth_manager*/ None,
        )
        .with_provider_catalog();
        assert_eq!(
            restarted
                .raw_model_catalog(
                    RefreshStrategy::OnlineIfUncached,
                    DEFAULT_HTTP_CLIENT_FACTORY,
                )
                .await,
            initial
        );
        assert_eq!(endpoint.fetch_count(), 2);

        // A successful empty response remains authoritative and is persisted.
        assert_eq!(
            manager
                .raw_model_catalog(RefreshStrategy::Online, DEFAULT_HTTP_CLIENT_FACTORY)
                .await,
            ModelsResponse::default()
        );
        assert_eq!(
            cache.stored_entries().last().unwrap().models,
            Vec::<ModelInfo>::new()
        );
    }
}

#[tokio::test]
async fn failed_discovery_does_not_reuse_a_catalog_from_another_identity() {
    let stored = ModelsCacheEntry {
        fetched_at: Utc::now(),
        etag: Some("previous-etag".into()),
        client_version: Some(crate::client_version_to_whole()),
        identity: Some("previous-provider-or-account".into()),
        models: vec![remote_model(
            "private-model",
            "Private Model",
            /*priority*/ 0,
        )],
    };
    let cache = TestModelsCache::with_entry(stored.clone());
    let endpoint = TestModelsEndpoint::with_command_auth(vec![Err(CodexErr::RequestTimeout)]);
    let manager =
        OpenAiModelsManager::new_with_cache(cache.clone(), endpoint, /*auth_manager*/ None)
            .with_provider_catalog();
    assert_eq!(
        manager
            .raw_model_catalog(
                RefreshStrategy::OnlineIfUncached,
                DEFAULT_HTTP_CLIENT_FACTORY
            )
            .await,
        ModelsResponse::default()
    );
    assert_eq!(cache.entry.lock().unwrap().as_ref(), Some(&stored));
    assert_eq!(manager.get_remote_models().await, Vec::<ModelInfo>::new());
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
async fn online_refresh_updates_access_programs_with_unchanged_etag() {
    let codex_home = tempdir().expect("temp dir");
    let granted_model = ModelInfo {
        available_access_programs: Some(ModelAccessPrograms {
            cyber: vec![
                CyberAccessProgram::Standard,
                CyberAccessProgram::DaybreakBlue,
            ],
        }),
        ..remote_model("access-programs", "Access Programs", /*priority*/ 0)
    };
    let revoked_model = ModelInfo {
        available_access_programs: Some(ModelAccessPrograms { cyber: Vec::new() }),
        ..granted_model.clone()
    };
    let responses = vec![vec![granted_model.clone()], vec![revoked_model.clone()]];
    let endpoint = Arc::new(TestModelsEndpoint {
        uses_codex_backend: true,
        responses: Mutex::new(responses.into_iter().map(Ok).collect()),
        etag: Some("stable-catalog-etag".to_string()),
        ..TestModelsEndpoint::default()
    });
    let manager = openai_manager_for_tests(codex_home.path().to_path_buf(), endpoint.clone());

    // Caller-specific access can change without changing the catalog ETag.
    // Corbanu lists the bundled catalog too, whose default stays the default.
    let preset = |list: Vec<ModelPreset>| {
        list.into_iter()
            .find(|preset| preset.model == "access-programs")
    };
    for model in [granted_model, revoked_model] {
        let expected = ModelPreset::from(model.clone());
        assert_eq!(
            preset(
                manager
                    .list_models(RefreshStrategy::Online, DEFAULT_HTTP_CLIENT_FACTORY)
                    .await
            ),
            Some(expected.clone())
        );
        assert_eq!(manager.get_remote_models().await, with_bundled(vec![model]));
        assert_eq!(
            manager.remote_models.read().await.entry.etag.as_deref(),
            Some("stable-catalog-etag")
        );

        // A new manager must read the latest access metadata from the disk cache.
        let cache_endpoint = TestModelsEndpoint::new(Vec::new());
        let cache_manager =
            openai_manager_for_tests(codex_home.path().to_path_buf(), cache_endpoint.clone());
        assert_eq!(
            preset(
                cache_manager
                    .list_models(
                        RefreshStrategy::OnlineIfUncached,
                        DEFAULT_HTTP_CLIENT_FACTORY
                    )
                    .await
            ),
            Some(expected)
        );
        assert_eq!(cache_endpoint.fetch_count(), 0);
        assert_eq!(
            cache_manager
                .remote_models
                .read()
                .await
                .entry
                .etag
                .as_deref(),
            Some("stable-catalog-etag")
        );
    }
    assert_eq!(endpoint.fetch_count(), 2);
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
    mutate_file_cache_for_test(codex_home.path(), |cache| {
        cache.fetched_at = Utc::now() - chrono::Duration::hours(1);
    })
    .await;

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

    mutate_file_cache_for_test(codex_home.path(), |cache| {
        let client_version = crate::client_version_to_whole();
        cache.client_version = Some(format!("{client_version}-mismatch"));
    })
    .await;

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
    let manager = OpenAiModelsManager::new_with_cache(
        Arc::new(FileModelsCache::new(
            codex_home.path().join(MODEL_CACHE_FILE),
            Duration::ZERO,
        )),
        endpoint.clone(),
        Some(AuthManager::from_auth_for_testing(
            CodexAuth::create_dummy_chatgpt_auth_for_testing(),
        )),
    );

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
async fn refresh_available_models_skips_network_without_auth() {
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
        .expect("refresh should no-op without auth");
    let cached_remote = manager.get_remote_models().await;
    assert!(
        !cached_remote
            .iter()
            .any(|candidate| candidate.slug == dynamic_slug),
        "remote refresh should be skipped without auth"
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

    async fn list_models(&self) -> CoreResult<ModelsEndpointResponse> {
        self.fetch_count.fetch_add(1, Ordering::SeqCst);
        let models = self
            .responses
            .lock()
            .expect("responses lock should not be poisoned")
            .pop_front()
            .unwrap_or_default();
        Ok(ModelsEndpointResponse {
            models,
            etag: None,
            identity: self.identity().expect("test endpoint identity"),
        })
    }
}

impl ModelsEndpointClient for TestAuthAwareModelsEndpoint {
    fn supports_api_key_models(&self) -> bool {
        true
    }

    fn identity(&self) -> Option<String> {
        self.auth_manager
            .as_ref()
            .and_then(|manager| manager.auth_mode())
            .map(|mode| format!("{mode:?}"))
    }

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
    ) -> ModelsEndpointFuture<'a, CoreResult<ModelsEndpointResponse>> {
        Box::pin(TestAuthAwareModelsEndpoint::list_models(self))
    }
}

#[tokio::test]
async fn refresh_available_models_fetches_when_external_api_key_overrides_chatgpt_auth() {
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
    manager.set_api_key_model_discovery_enabled(/*enabled*/ true);

    manager
        .refresh_available_models(RefreshStrategy::Online, &DEFAULT_HTTP_CLIENT_FACTORY)
        .await
        .expect("refresh should fetch with API key auth");
    let cached_remote = manager.get_remote_models().await;

    assert_eq!(
        cached_remote,
        with_bundled(vec![remote_model(
            dynamic_slug,
            "External API Key",
            /*priority*/ 1
        )])
    );
    assert_eq!(endpoint.fetch_count(), 1);
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
    manager.set_api_key_model_discovery_enabled(/*enabled*/ true);

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
        ModelOrigin::ExplicitConfig,
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

/// The Anthropic Messages wire requires `max_tokens`, and `zai-anthropic`
/// serves the bare `glm-…` slugs, so each needs a catalogued output limit.
/// Without one every turn on that route failed before reaching the provider.
#[test]
fn bundled_zai_anthropic_models_have_output_limits() {
    let response = crate::bundled_models_response()
        .unwrap_or_else(|err| panic!("bundled models.json should parse: {err}"));

    let missing = response
        .models
        .iter()
        .filter(|model| model.slug.starts_with("glm-") && model.max_output_tokens.is_none())
        .map(|model| model.slug.as_str())
        .collect::<Vec<_>>();

    assert_eq!(missing, Vec::<&str>::new());
}

/// #1567: every GLM 5.3 row (native Z.AI, OpenRouter and Vercel) carries
/// Z.AI's documented 128K maximum output (https://docs.z.ai/guides/llm/glm-5.3,
/// https://docs.z.ai/guides/vlm/glm-5.3-flash), the #189 rule, not the
/// context window or a gateway figure.
#[test]
fn bundled_glm_5_3_rows_use_zai_documented_output_cap() {
    let response = crate::bundled_models_response()
        .unwrap_or_else(|err| panic!("bundled models.json should parse: {err}"));
    let mut caps = response
        .models
        .iter()
        .filter(|model| {
            matches!(
                model.slug.as_str(),
                "glm-5.3"
                    | "glm-5.3-flash"
                    | "z-ai/glm-5.3-flash"
                    | "zai/glm-5.3"
                    | "zai/glm-5.3-flash"
            )
        })
        .map(|model| (model.slug.as_str(), model.max_output_tokens))
        .collect::<Vec<_>>();
    caps.sort();

    assert_eq!(
        caps,
        vec![
            ("glm-5.3", Some(128_000)),
            ("glm-5.3-flash", Some(128_000)),
            ("z-ai/glm-5.3-flash", Some(128_000)),
            ("zai/glm-5.3", Some(128_000)),
            ("zai/glm-5.3-flash", Some(128_000)),
        ]
    );
}

/// The Anthropic Messages wire requires `max_tokens`, so every model a built-in
/// Anthropic-wire provider can run must resolve to an output limit (#181).
#[test]
fn bundled_anthropic_wire_provider_models_have_output_limits() {
    use codex_model_provider_info::CORBANU_API_CLAUDE_FABLE_5_MODEL;
    use codex_model_provider_info::CORBANU_API_KIMI_K3_MODEL;
    use codex_model_provider_info::WireApi;
    use codex_model_provider_info::built_in_model_providers;
    use codex_model_provider_info::canonical_catalog_provider;
    use codex_model_provider_info::corrected_catalog_provider;
    use codex_model_provider_info::default_model_max_output_tokens_for_provider;
    use codex_model_provider_info::resolve_model_for_provider;

    const UNLISTED_MODEL: &str = "unlisted-vendor/unlisted-model";

    let response = crate::bundled_models_response()
        .unwrap_or_else(|err| panic!("bundled models.json should parse: {err}"));
    let mut providers = built_in_model_providers(/*openai_base_url*/ None)
        .into_iter()
        .filter(|(_, provider)| provider.wire_api == WireApi::Anthropic)
        .map(|(provider_id, _)| provider_id)
        .collect::<Vec<_>>();
    providers.sort();
    assert!(
        providers.contains(&"openrouter-anthropic".to_string())
            && providers.contains(&"baseten-anthropic".to_string()),
        "expected the Anthropic-wire gateways in {providers:?}"
    );

    let mut missing = Vec::new();
    for provider_id in &providers {
        // Pass-through gateways (OpenRouter) accept any slug. Walk the slugs
        // catalogued for that gateway; allow-list providers resolve only to
        // the slugs they accept.
        let passes_through =
            resolve_model_for_provider(Some(UNLISTED_MODEL.to_string()), provider_id).as_deref()
                == Some(UNLISTED_MODEL);
        let family = provider_id
            .strip_suffix("-anthropic")
            .unwrap_or(provider_id);
        // Catalogue slugs, plus public slugs that providers accept but that
        // borrow another row's metadata (Corbanu API identities).
        let requested = std::iter::once(None)
            .chain(response.models.iter().map(|model| Some(model.slug.clone())))
            .chain(
                [CORBANU_API_KIMI_K3_MODEL, CORBANU_API_CLAUDE_FABLE_5_MODEL]
                    .map(|model| Some(model.to_string())),
            );
        for request in requested {
            let explicit = request.is_some();
            let Some(model) = resolve_model_for_provider(request, provider_id) else {
                continue;
            };
            // Impossible pairs are moved to another provider before a turn.
            if corrected_catalog_provider(&model, provider_id).is_some() {
                continue;
            }
            if passes_through
                && explicit
                && !canonical_catalog_provider(&model)
                    .is_some_and(|owner| owner == provider_id.as_str() || owner == family)
            {
                continue;
            }
            let config = ModelsManagerConfig {
                model_max_output_tokens: default_model_max_output_tokens_for_provider(
                    provider_id,
                    &model,
                ),
                ..Default::default()
            };
            let info = construct_model_info_from_candidates(
                &model,
                &response.models,
                ModelOrigin::Bundled,
                &config,
            );
            if info.origin == ModelOrigin::Synthetic || info.max_output_tokens.is_none() {
                missing.push(format!("{provider_id}: {model}"));
            }
        }
    }
    missing.sort();
    missing.dedup();

    assert_eq!(missing, Vec::<String>::new());
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
        "claude-opus-5-5-plan",
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
    // Fable 5.1 keeps the existing Fable family request ceiling on the Claude Plan wire.
    let response = crate::bundled_models_response()
        .unwrap_or_else(|err| panic!("bundled models.json should parse: {err}"));

    for (slug, max_output_tokens) in [
        ("claude-opus-5-plan", 128_000),
        ("claude-opus-5-5-plan", 128_000),
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
                ModelBilling::MeteredSchedule {
                    peak_windows,
                    peak_weekdays,
                    off_peak_dates_utc,
                    ..
                } => {
                    assert!(
                        !peak_windows.is_empty()
                            && peak_windows.iter().all(|window| {
                                window.start_utc_hour < 24
                                    && window.end_utc_hour <= 24
                                    && window.start_utc_hour != window.end_utc_hour
                            }),
                        "{} must have valid UTC peak windows",
                        model.slug
                    );
                    assert!(
                        peak_weekdays.is_none_or(|weekdays| !weekdays.is_empty()),
                        "{} must not specify an empty peak weekday set",
                        model.slug
                    );
                    for date in off_peak_dates_utc {
                        assert!(
                            chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d").is_ok(),
                            "{}: off-peak date {date} must be YYYY-MM-DD",
                            model.slug
                        );
                    }
                }
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
                api_key_input_milli_usd_per_million_tokens: 4_000,
                api_key_output_milli_usd_per_million_tokens: 20_000,
                api_key_cached_input_milli_usd_per_million_tokens: Some(400),
                api_key_cache_write_milli_usd_per_million_tokens: Some(5_000),
                api_key_long_context: Some(LongContextRates {
                    above_input_tokens: 272_000,
                    input_milli_usd_per_million_tokens: 8_000,
                    output_milli_usd_per_million_tokens: 30_000,
                    cached_input_milli_usd_per_million_tokens: Some(800),
                    cache_write_milli_usd_per_million_tokens: Some(10_000),
                }),
                api_key_valid_through_utc: Some("2026-11-21T23:59:59.999Z".to_string()),
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
                api_key_input_milli_usd_per_million_tokens: 2_000,
                api_key_output_milli_usd_per_million_tokens: 12_000,
                api_key_cached_input_milli_usd_per_million_tokens: Some(200),
                api_key_cache_write_milli_usd_per_million_tokens: Some(2_500),
                api_key_long_context: Some(LongContextRates {
                    above_input_tokens: 272_000,
                    input_milli_usd_per_million_tokens: 4_000,
                    output_milli_usd_per_million_tokens: 18_000,
                    cached_input_milli_usd_per_million_tokens: Some(400),
                    cache_write_milli_usd_per_million_tokens: Some(5_000),
                }),
                api_key_valid_through_utc: None,
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
                api_key_input_milli_usd_per_million_tokens: 200,
                api_key_output_milli_usd_per_million_tokens: 1_200,
                api_key_cached_input_milli_usd_per_million_tokens: Some(20),
                api_key_cache_write_milli_usd_per_million_tokens: Some(250),
                api_key_long_context: Some(LongContextRates {
                    above_input_tokens: 272_000,
                    input_milli_usd_per_million_tokens: 400,
                    output_milli_usd_per_million_tokens: 1_800,
                    cached_input_milli_usd_per_million_tokens: Some(40),
                    cache_write_milli_usd_per_million_tokens: Some(500),
                }),
                api_key_valid_through_utc: None,
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
            billing: Some(ModelBilling::Metered {
                input_milli_usd_per_million_tokens: 5_000,
                output_milli_usd_per_million_tokens: 30_000,
                cached_input_milli_usd_per_million_tokens: Some(500),
                cache_write_milli_usd_per_million_tokens: None,
                long_context: Some(LongContextRates {
                    above_input_tokens: 272_000,
                    input_milli_usd_per_million_tokens: 10_000,
                    output_milli_usd_per_million_tokens: 45_000,
                    // The page states 2x input and 1.5x output only.
                    cached_input_milli_usd_per_million_tokens: None,
                    cache_write_milli_usd_per_million_tokens: None,
                }),
                valid_through_utc: None,
            }),
        }
    );
}

/// The money in a user's ledger is whatever these two rows state, and a wrong
/// rate is silent: the turn still runs, still records tokens, and still prints
/// a total. So the published figures are pinned here, exactly, the same way the
/// 5.6 tiers are.
#[test]
fn bundled_orchestration_states_published_rates_for_opus_5_5_and_gpt_6_sol() {
    let response = crate::bundled_models_response()
        .unwrap_or_else(|err| panic!("bundled models.json should parse: {err}"));
    let model = |slug: &str| {
        response
            .models
            .iter()
            .find(|model| model.slug == slug)
            .unwrap_or_else(|| panic!("{slug} must be in the bundled catalogue"))
    };

    // Anthropic: $4 / $20 per million, cache reads $0.20 per million.
    assert_eq!(
        model("claude-opus-5-5").orchestration,
        Some(ModelOrchestrationMetadata::Eligible {
            provider_id: "anthropic".to_string(),
            capability: ModelCapabilityTier::Frontier,
            billing: ModelBilling::Metered {
                input_milli_usd_per_million_tokens: 4_000,
                output_milli_usd_per_million_tokens: 20_000,
                cached_input_milli_usd_per_million_tokens: Some(200),
                cache_write_milli_usd_per_million_tokens: None,
                long_context: None,
                valid_through_utc: None,
            },
        })
    );
    assert_eq!(model("claude-opus-5-5").context_window, Some(1_000_000));
    assert_eq!(model("claude-opus-5-5").max_output_tokens, Some(128_000));

    // OpenAI: $2 / $10 per million, cache reads $0.20, cache writes $2.50 per
    // million; above 272K input tokens 2x input and cache rates, 1.5x output.
    //
    // Metered, not auth-dependent. OpenAI publishes those API rates and nothing
    // about subscription capacity for this model, and Codex rejects it outright
    // on some ChatGPT accounts. An `AuthDependent` row would state a plan burn
    // no vendor quoted, for capacity the account may not have; `Metered` prices
    // an API-key turn exactly and leaves a subscription turn recording tokens
    // with no money claimed, which is the truthful answer here.
    assert_eq!(
        model("gpt-6-sol").orchestration,
        Some(ModelOrchestrationMetadata::Eligible {
            provider_id: "openai".to_string(),
            capability: ModelCapabilityTier::Frontier,
            billing: ModelBilling::Metered {
                input_milli_usd_per_million_tokens: 2_000,
                output_milli_usd_per_million_tokens: 10_000,
                cached_input_milli_usd_per_million_tokens: Some(200),
                cache_write_milli_usd_per_million_tokens: Some(2_500),
                long_context: Some(LongContextRates {
                    above_input_tokens: 272_000,
                    input_milli_usd_per_million_tokens: 4_000,
                    output_milli_usd_per_million_tokens: 15_000,
                    cached_input_milli_usd_per_million_tokens: Some(400),
                    cache_write_milli_usd_per_million_tokens: Some(5_000),
                }),
                valid_through_utc: None,
            },
        })
    );
    assert_eq!(model("gpt-6-sol").context_window, Some(1_050_000));
    assert_eq!(model("gpt-6-sol").max_output_tokens, Some(128_000));
}

/// #368: GPT-6.1 Sol at the Standard rates OpenAI publishes, read 2026-10-14
/// from developers.openai.com/api/docs/pricing and
/// developers.openai.com/api/docs/models/gpt-6.1-sol: $2 input, $0.10 cached
/// input (5% of input), $2.50 cache writes (1.25x), $10 output per million;
/// above 272K input tokens 2x input and cache rates and 1.5x output. Manual
/// selection only, like GPT-6 Astra and Luna: it states its price for
/// accounting and takes no automatically allocated work.
#[test]
fn bundled_orchestration_states_published_rates_for_gpt_6_1_sol() {
    let response = crate::bundled_models_response()
        .unwrap_or_else(|err| panic!("bundled models.json should parse: {err}"));
    let model = response
        .models
        .iter()
        .find(|model| model.slug == "gpt-6.1-sol")
        .expect("gpt-6.1-sol must be in the bundled catalogue");
    assert_eq!(
        model.orchestration,
        Some(ModelOrchestrationMetadata::Disabled {
            provider_id: "openai".to_string(),
            capability: ModelCapabilityTier::Frontier,
            reason: "manual selection only; automatic allocation economics are not configured"
                .to_string(),
            billing: Some(ModelBilling::Metered {
                input_milli_usd_per_million_tokens: 2_000,
                output_milli_usd_per_million_tokens: 10_000,
                cached_input_milli_usd_per_million_tokens: Some(100),
                cache_write_milli_usd_per_million_tokens: Some(2_500),
                long_context: Some(LongContextRates {
                    above_input_tokens: 272_000,
                    input_milli_usd_per_million_tokens: 4_000,
                    output_milli_usd_per_million_tokens: 15_000,
                    cached_input_milli_usd_per_million_tokens: Some(200),
                    cache_write_milli_usd_per_million_tokens: Some(5_000),
                }),
                valid_through_utc: None,
            }),
        })
    );
    assert_eq!(model.context_window, Some(1_050_000));
    assert_eq!(model.max_output_tokens, Some(128_000));
    assert_eq!(model.visibility, ModelVisibility::List);
    assert!(instructions(model).contains("based on GPT-6.1 Sol"));
}

/// Off-peak holiday dates are whole UTC days on which a weekday peak would
/// otherwise apply. Listing a weekend or repeating a date is a transcription
/// slip, and rows of one provider must agree: `scripts/check_price_calendars.py`
/// reads each list as covering through the end of its last year (#368).
#[test]
fn bundled_off_peak_dates_are_sorted_unique_weekdays_shared_per_provider() {
    let response = crate::bundled_models_response()
        .unwrap_or_else(|err| panic!("bundled models.json should parse: {err}"));
    let mut lists = std::collections::BTreeMap::<String, Vec<String>>::new();
    for model in &response.models {
        let Some(metadata) = model.orchestration.as_ref() else {
            continue;
        };
        let Some(ModelBilling::MeteredSchedule {
            peak_weekdays,
            off_peak_dates_utc,
            ..
        }) = metadata.billing()
        else {
            continue;
        };
        for pair in off_peak_dates_utc.windows(2) {
            assert!(pair[0] < pair[1], "{}: {pair:?} out of order", model.slug);
        }
        for date in off_peak_dates_utc {
            let day = chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d")
                .unwrap_or_else(|err| panic!("{}: {date}: {err}", model.slug));
            let weekday = day.format("%A").to_string();
            assert!(
                peak_weekdays.is_none_or(|weekdays| weekdays.names().contains(&weekday.as_str())),
                "{}: {date} is not a peak weekday",
                model.slug
            );
        }
        if let Some(other) = lists.insert(
            metadata.provider_id().to_string(),
            off_peak_dates_utc.clone(),
        ) {
            assert_eq!(&other, off_peak_dates_utc, "{}", model.slug);
        }
    }
    assert!(lists.contains_key("deepseek"));
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
    assert_standard_base(instructions(ambient_default));

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
    assert_eq!(
        zai_glm_5_3
            .orchestration
            .as_ref()
            .and_then(ModelOrchestrationMetadata::billing),
        Some(&ModelBilling::Metered {
            input_milli_usd_per_million_tokens: 1_400,
            output_milli_usd_per_million_tokens: 4_400,
            cached_input_milli_usd_per_million_tokens: Some(260),
            cache_write_milli_usd_per_million_tokens: None,
            long_context: None,
            valid_through_utc: None,
        })
    );
    let preset = ModelPreset::from(zai_glm_5_3.clone());
    assert_eq!(preset.provider_id.as_deref(), Some("zai"));
    assert_standard_base(instructions(zai_glm_5_3));

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
    assert_eq!(ambient_kimi.visibility, ModelVisibility::Hide);
    assert_standard_base(instructions(ambient_kimi));

    let ambient = response
        .models
        .iter()
        .find(|model| model.slug == "ambient/large")
        .expect("bundled models.json should include ambient/large");

    assert_eq!(ambient.display_name, "Ambient Large");
    assert_eq!(ambient.context_window, Some(131_072));
    assert_eq!(ambient.visibility, ModelVisibility::Hide);
    assert_standard_base(instructions(ambient));
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
        "zai/glm-5.3",
        "vercel/moonshotai/kimi-k3",
        "vercel/deepseek/deepseek-v4-pro",
        "zai-org/GLM-5.2",
        "ambient/large",
        "glm-5.2",
        "minimax/minimax-m3",
        "openrouter/owl-alpha",
        "google/gemini-3.5-flash",
        "x-ai/grok-4.5",
        "deepseek-flash",
        "deepseek-v4-flash",
        "deepseek/deepseek-v4-pro",
        "deepseek/deepseek-v4-flash-0731",
        "tencent/hy3:free",
        "muse-spark-1.1",
        "claude-opus-5-plan",
        "claude-opus-5-5-plan",
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

        assert_standard_base(instructions(model));
        assert!(instructions(model).contains(STANDARD_BASE_OUTCOME_MARKER));
        assert!(instructions(model).contains(STANDARD_BASE_EVIDENCE_MARKER));
        assert!(!instructions(model).contains(OLD_STANDARD_BASE_MARKER));
    }

    let gpt55 = response
        .models
        .iter()
        .find(|model| model.slug == "gpt-5.5")
        .expect("bundled models.json should include gpt-5.5");

    assert!(instructions(gpt55).contains(GPT55_GUIDE_MARKER));
    assert!(!instructions(gpt55).contains(STANDARD_BASE_EVIDENCE_MARKER));
    assert_ne!(instructions(gpt55).trim_end(), STANDARD_BASE.trim_end());
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
    assert_eq!(
        kimi.chat_completions.reasoning_protocol,
        ChatReasoningProtocol::PreservedRequired
    );
    assert_standard_base(instructions(kimi));

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
    assert_eq!(openrouter_owl.visibility, ModelVisibility::Hide);
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

    for model in [
        grok,
        deepseek_pro,
        deepseek_pro_0813,
        deepseek_flash,
        openrouter_owl,
        hy3,
    ] {
        assert_eq!(model.visibility, ModelVisibility::Hide);
        assert!(matches!(
            model.orchestration,
            Some(ModelOrchestrationMetadata::Disabled { .. })
        ));
    }
    assert_eq!(kimi.visibility, ModelVisibility::List);
    for model in [hy3, kimi] {
        assert_standard_base(instructions(model));
    }

    let meta = response
        .models
        .iter()
        .find(|model| model.slug == "muse-spark-1.1")
        .expect("bundled models.json should include Meta Muse Spark 1.1");
    assert_eq!(meta.context_window, Some(1_048_576));
    assert!(meta.apply_patch_tool_type.is_none());
    assert_standard_base(instructions(meta));

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
            "claude-fable-5-plan",
            "Claude Fable 5 Plan",
            "Claude Fable 5 through Claude Code subscription auth in Corbanu Terminal.",
        ),
        (
            "claude-fable-5-1-plan",
            "Claude Fable 5.1 Plan",
            "Claude Fable 5.1 through Claude Code subscription auth in Corbanu Terminal.",
        ),
    ] {
        let claude_fable_plan = response
            .models
            .iter()
            .find(|model| model.slug == slug)
            .unwrap_or_else(|| panic!("bundled models.json should include {display_name}"));

        assert_eq!(claude_fable_plan.display_name, display_name);
        assert_eq!(claude_fable_plan.description.as_deref(), Some(description));
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
    }

    let claude_fable_5_1 = response
        .models
        .iter()
        .find(|model| model.slug == "claude-fable-5-1")
        .expect("bundled models.json should include Claude Fable 5.1");
    assert_eq!(
        (
            claude_fable_5_1.display_name.as_str(),
            claude_fable_5_1.description.as_deref(),
            claude_fable_5_1.visibility,
        ),
        (
            "Claude Fable 5.1",
            Some("Claude Fable 5.1 through the Anthropic Messages API."),
            ModelVisibility::List,
        )
    );

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

    for (slug, display_name, context_window, max_output_tokens) in [
        (
            "zai/glm-5.3-flash",
            "Vercel GLM 5.3 Flash",
            1_000_000,
            128_000,
        ),
        ("zai/glm-5.3", "Vercel GLM 5.3", 1_000_000, 128_000),
        (
            "vercel/moonshotai/kimi-k3",
            "Vercel Kimi K3",
            1_000_000,
            128_000,
        ),
        (
            "vercel/deepseek/deepseek-v4-pro",
            "Vercel DeepSeek V4 Pro",
            1_000_000,
            384_000,
        ),
    ] {
        let model = response
            .models
            .iter()
            .find(|model| model.slug == slug)
            .unwrap_or_else(|| panic!("bundled models.json should include {slug}"));
        assert_eq!(model.display_name, display_name);
        assert_eq!(model.context_window, Some(context_window));
        assert_eq!(model.max_output_tokens, Some(max_output_tokens));
        assert_eq!(
            model
                .orchestration
                .as_ref()
                .map(codex_protocol::openai_models::ModelOrchestrationMetadata::provider_id),
            Some("vercel")
        );
        assert_eq!(model.visibility, ModelVisibility::List);
    }

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
    // DeepSeek's pricing page (2026-09-10): V4.1 Flash is served as
    // `deepseek-flash`; the legacy `deepseek-v4-flash` name is still accepted but is
    // served by V4.1 Flash and billed at its price. Peak is 01:00-04:00 and
    // 06:00-10:00 UTC Monday-Friday, except Chinese public holidays; off-peak is half.
    let rates = |input, output, cached| MeteredRates {
        input_milli_usd_per_million_tokens: input,
        output_milli_usd_per_million_tokens: output,
        cached_input_milli_usd_per_million_tokens: Some(cached),
    };
    let window = |start_utc_hour, end_utc_hour| UtcHourWindow {
        start_utc_hour,
        end_utc_hour,
    };
    let v4_1_peak = Some(ModelOrchestrationMetadata::Eligible {
        provider_id: "deepseek".to_string(),
        capability: ModelCapabilityTier::Fast,
        billing: ModelBilling::MeteredSchedule {
            off_peak: rates(150, 600, 3),
            peak: rates(300, 1_200, 6),
            peak_windows: vec![window(1, 4), window(6, 10)],
            peak_weekdays: Some(WeekdaySet::weekdays_only()),
            // State Council 2026 schedule: the remaining weekday holidays.
            off_peak_dates_utc: [
                "2026-09-25",
                "2026-10-01",
                "2026-10-02",
                "2026-10-05",
                "2026-10-06",
                "2026-10-07",
            ]
            .map(String::from)
            .to_vec(),
        },
    });
    for (slug, display_name) in [
        ("deepseek-flash", "DeepSeek V4.1 Flash (Direct)"),
        ("deepseek-v4-flash", "DeepSeek V4 Flash 0731 (Direct)"),
    ] {
        let deepseek = response
            .models
            .iter()
            .find(|model| model.slug == slug)
            .unwrap_or_else(|| panic!("bundled models.json should include {slug}"));

        assert_eq!(deepseek.display_name, display_name);
        assert_eq!(deepseek.visibility, ModelVisibility::List);
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
        assert_eq!(deepseek.orchestration, v4_1_peak, "{slug}");
        assert_standard_base(instructions(deepseek));
    }
}

/// GLM 5.3 Flash worked in long chains of small edits and reasoned about half as
/// much per step as under Hermes, fully passing 11 of 30 benchmark tasks against
/// Hermes's 19 of 30. The standard base plus an explicit work pattern (batch
/// independent calls, write complete implementations, verify every requirement)
/// passed 13 of 20. Every GLM 5.3 Flash route carries that pattern.
#[test]
fn glm_5_3_flash_routes_use_standard_base_with_work_pattern() {
    let response = crate::bundled_models_response()
        .unwrap_or_else(|err| panic!("bundled models.json should parse: {err}"));
    let (first, rest) = STANDARD_BASE
        .split_once("\n\n")
        .expect("standard base has an opening paragraph");
    let expected = format!("{first}\n\n{}\n\n{rest}", GLM_FLASH_WORK_PATTERN.trim_end());

    for slug in ["glm-5.3-flash", "z-ai/glm-5.3-flash", "zai/glm-5.3-flash"] {
        let model = response
            .models
            .iter()
            .find(|model| model.slug == slug)
            .unwrap_or_else(|| panic!("bundled models.json should include {slug}"));
        assert_eq!(
            instructions(model).trim_end(),
            expected.trim_end(),
            "{slug}"
        );
    }
}

/// PF-84: a list that needs a fetch is current only with a fresh cache of
/// this client version, the same check refreshing applies.
#[tokio::test]
async fn load_cached_list_needs_a_usable_cache() {
    let codex_home = tempdir().expect("temp dir");
    let endpoint = TestModelsEndpoint::new(vec![vec![remote_model(
        "cached-model",
        "Cached",
        /*priority*/ 1,
    )]]);
    let manager = openai_manager_for_tests(codex_home.path().to_path_buf(), endpoint.clone());
    assert!(!manager.load_cached_list().await, "no cache yet");

    manager
        .list_models(RefreshStrategy::Online, DEFAULT_HTTP_CLIENT_FACTORY)
        .await;
    let reloaded = openai_manager_for_tests(codex_home.path().to_path_buf(), endpoint);
    assert!(reloaded.load_cached_list().await, "fresh cache");
    mutate_file_cache_for_test(codex_home.path(), |cache| {
        cache.fetched_at = Utc::now() - chrono::Duration::hours(1);
    })
    .await;
    assert!(!reloaded.load_cached_list().await, "stale cache");

    let no_fetch = openai_manager_for_tests(
        tempdir().expect("temp dir").path().to_path_buf(),
        TestModelsEndpoint::without_refresh(Vec::new()),
    );
    assert!(
        no_fetch.load_cached_list().await,
        "an API-key list needs no fetch"
    );
}
