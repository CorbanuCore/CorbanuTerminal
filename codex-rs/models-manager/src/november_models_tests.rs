use super::*;
use pretty_assertions::assert_eq;

#[test]
fn november_catalog_additions_preserve_every_existing_entry() {
    let original: ModelsResponse =
        serde_json::from_str(include_str!("../models.json")).expect("original catalog");
    let combined = crate::bundled_models_response().expect("combined catalog");
    assert_eq!(&combined.models[..original.models.len()], original.models);
    let mut slugs = std::collections::HashSet::new();
    assert!(
        combined
            .models
            .iter()
            .all(|model| slugs.insert(&model.slug))
    );

    for uses_codex_backend in [false, true] {
        let presets = codex_protocol::openai_models::ModelPreset::filter_by_auth(
            combined
                .models
                .clone()
                .into_iter()
                .map(Into::into)
                .collect(),
            uses_codex_backend,
        );
        for (slug, provider, effort, context) in [
            (
                "gpt-6.2-astra",
                "openai",
                ReasoningEffort::Medium,
                1_050_000,
            ),
            ("gpt-6.2-sol", "openai", ReasoningEffort::Medium, 1_050_000),
            (
                "claude-fable-5-2",
                "anthropic",
                ReasoningEffort::High,
                1_000_000,
            ),
        ] {
            let preset = presets
                .iter()
                .find(|preset| preset.model == slug)
                .expect(slug);
            assert!(preset.show_in_picker);
            assert_eq!(preset.provider_id.as_deref(), Some(provider));
            assert_eq!(preset.default_reasoning_effort, effort);
            let model = combined
                .models
                .iter()
                .find(|model| model.slug == slug)
                .expect(slug);
            assert_eq!(
                (
                    model.context_window,
                    model.max_context_window,
                    model.max_output_tokens
                ),
                (Some(context), Some(context), Some(128_000))
            );
            assert!(
                !model
                    .supported_reasoning_levels
                    .iter()
                    .any(|level| matches!(
                        level.effort,
                        ReasoningEffort::None | ReasoningEffort::Minimal
                    ))
            );
            if provider == "openai" {
                assert_eq!(
                    model.service_tier_for_request(Some("ultrafast".to_string())),
                    Some("ultrafast".to_string())
                );
                assert_eq!(model.default_service_tier, None);
            }
        }
    }
}

#[tokio::test]
async fn november_models_survive_remote_omission_without_becoming_defaults() {
    let codex_home = tempdir().expect("temp dir");
    let endpoint = TestModelsEndpoint::new(vec![Vec::new()]);
    let manager = openai_manager_for_tests(codex_home.path().to_path_buf(), endpoint);
    let available = manager
        .list_models(RefreshStrategy::Online, DEFAULT_HTTP_CLIENT_FACTORY)
        .await;
    for slug in ["gpt-6.2-astra", "gpt-6.2-sol", "claude-fable-5-2"] {
        let model = available
            .iter()
            .find(|model| model.model == slug)
            .expect(slug);
        assert!(model.show_in_picker);
        assert!(!model.is_default);
    }
    assert_eq!(
        available
            .iter()
            .filter(|model| model.is_default)
            .map(|model| model.model.as_str())
            .collect::<Vec<_>>(),
        vec!["gpt-6-sol"]
    );
}

#[tokio::test]
async fn november_remote_overlay_preserves_effort_pricing_and_output_limits() {
    let remote = remote_model("gpt-6.2-astra", "GPT-6.2 Astra", /*priority*/ 0);
    let codex_home = tempdir().expect("temp dir");
    let endpoint = TestModelsEndpoint::new(vec![vec![remote]]);
    let manager = openai_manager_for_tests(codex_home.path().to_path_buf(), endpoint);
    manager
        .list_models(RefreshStrategy::Online, DEFAULT_HTTP_CLIENT_FACTORY)
        .await;
    let models = manager.get_remote_models().await;
    let model = models
        .iter()
        .find(|model| model.slug == "gpt-6.2-astra")
        .expect("overlay");
    let bundled = crate::bundled_models_response()
        .expect("catalog")
        .models
        .into_iter()
        .find(|model| model.slug == "gpt-6.2-astra")
        .expect("bundled");
    assert_eq!(
        (
            &model.orchestration,
            &model.supported_reasoning_levels,
            &model.default_reasoning_level,
            model.max_output_tokens
        ),
        (
            &bundled.orchestration,
            &bundled.supported_reasoning_levels,
            &bundled.default_reasoning_level,
            Some(128_000)
        )
    );
}
