use super::*;
use codex_protocol::openai_models::ModelPreset;
use pretty_assertions::assert_eq;

// Captured from the ChatGPT /models endpoint with client_version=0.167.0.
// Long instruction strings are replaced with fixture text; metadata and nested
// message shapes are unchanged. No authentication or account data is included.
const CATALOG: &str = include_str!("../tests/fixtures/chatgpt-0.167.0.json");

#[tokio::test]
async fn chatgpt_62_catalog_preserves_native_runtime_contract() {
    let raw: serde_json::Value = serde_json::from_str(CATALOG).expect("catalog JSON");
    let catalog: ModelsResponse = serde_json::from_str(CATALOG).expect("native catalog parses");
    let home = tempdir().expect("isolated home");
    let endpoint = TestModelsEndpoint::new(vec![catalog.models.clone()]);
    let manager = openai_manager_for_tests(home.path().to_path_buf(), endpoint);
    let presets = manager
        .list_models(RefreshStrategy::Online, DEFAULT_HTTP_CLIENT_FACTORY)
        .await;
    let resolved = manager.get_remote_models().await;
    let bundled = crate::bundled_models_response().expect("bundled catalog");

    for mut expected in catalog.models {
        let actual = resolved
            .iter()
            .find(|m| m.slug == expected.slug)
            .expect("resolved model");
        let source = raw["models"]
            .as_array()
            .expect("models")
            .iter()
            .find(|m| m["slug"] == expected.slug)
            .expect("source");
        let serialized = serde_json::to_value(actual).expect("serialized model");
        // Check fields used to build requests, tools, input limits and compaction,
        // rather than merely accepting a successful deserialize with defaults.
        for field in [
            "shell_type",
            "apply_patch_tool_type",
            "web_search_tool_type",
            "truncation_policy",
            "supports_parallel_tool_calls",
            "supports_image_detail_original",
            "context_window",
            "max_context_window",
            "effective_context_window_percent",
            "experimental_supported_tools",
            "input_modalities",
            "supports_search_tool",
            "use_responses_lite",
            "tool_mode",
            "multi_agent_version",
            "comp_hash",
            "default_reasoning_summary",
            "support_verbosity",
            "default_verbosity",
        ] {
            if let Some(value) = source.get(field) {
                if field == "shell_type" {
                    // Sync deliberately canonicalizes the legacy shell_command spelling
                    // to UnifiedExec in ConfigShellToolType; verify both sides explicitly.
                    assert_eq!(value, &json!("shell_command"));
                    assert_eq!(serialized[field], json!("unified_exec"));
                } else if field == "supports_parallel_tool_calls" {
                    // Upstream 86b1123ff6 retired this gate: all prompts allow parallel
                    // tools, while client.rs still disables them for Responses Lite.
                    assert_eq!(value, &json!(true));
                    assert!(serialized.get(field).is_none());
                    assert!(actual.use_responses_lite);
                } else {
                    assert_eq!(&serialized[field], value, "native field {field}");
                }
            }
        }
        assert!(actual.supports_reasoning_summary_parameter);
        let local = bundled
            .models
            .iter()
            .find(|m| m.slug == expected.slug)
            .expect("local model");
        expected.orchestration = local.orchestration.clone();
        expected.chat_completions = local.chat_completions.clone();
        expected.default_reasoning_level = local.default_reasoning_level.clone();
        expected.supported_reasoning_levels = local.supported_reasoning_levels.clone();
        expected.max_output_tokens = local.max_output_tokens;
        assert_eq!(actual, &expected);

        for personality in [None, Some(codex_protocol::config_types::Personality::None)] {
            let configured = crate::model_info::with_config_overrides(
                actual.clone(),
                &ModelsManagerConfig {
                    personality,
                    model_context_window: Some(1_050_000),
                    ..Default::default()
                },
            );
            assert_eq!(configured.context_window, actual.max_context_window);
            assert_eq!(
                configured
                    .model_messages
                    .as_ref()
                    .and_then(|messages| messages.instructions_template.as_deref()),
                Some("Fixture instruction text.")
            );
            assert_eq!(configured.auto_compact_token_limit(), Some(784_800));
        }
        let visible = ModelPreset::filter_by_auth(presets.clone(), /*chatgpt_mode*/ true);
        assert!(
            visible
                .iter()
                .any(|p| p.model == actual.slug && p.show_in_picker)
        );
    }
}

#[tokio::test]
async fn chatgpt_compat_upgrade_rejects_fresh_old_version_cache() {
    let home = tempdir().expect("isolated home");
    let cache = FileModelsCache::new(
        home.path().join("models_cache.json"),
        std::time::Duration::from_secs(300),
    );
    cache
        .store(&ModelsCacheEntry {
            fetched_at: Utc::now(),
            etag: None,
            client_version: Some("0.156.0".to_string()),
            identity: Some("test-provider".to_string()),
            models: vec![remote_model(
                "obsolete-cache-model",
                "Old",
                /*priority*/ 0,
            )],
        })
        .await
        .expect("old cache");
    assert!(cache.load("0.156.0").await.expect("cache read").is_some());
    let catalog: ModelsResponse = serde_json::from_str(CATALOG).expect("native catalog");
    let endpoint = TestModelsEndpoint::new(vec![catalog.models]);
    let manager = openai_manager_for_tests(home.path().to_path_buf(), endpoint.clone());
    manager
        .list_models(
            RefreshStrategy::OnlineIfUncached,
            DEFAULT_HTTP_CLIENT_FACTORY,
        )
        .await;
    assert_eq!(endpoint.fetch_count(), 1);
    assert!(
        manager
            .server_advertised_models()
            .expect("refreshed catalog")
            .contains(&"gpt-6.2-astra".to_string())
    );
}
