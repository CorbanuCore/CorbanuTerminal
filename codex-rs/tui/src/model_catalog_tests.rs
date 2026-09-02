use super::*;
use codex_protocol::openai_models::InputModality;
use codex_protocol::openai_models::ReasoningEffort;

fn preset(model: &str, provider_id: Option<&str>) -> ModelPreset {
    ModelPreset {
        id: model.to_string(),
        model: model.to_string(),
        provider_id: provider_id.map(str::to_string),
        orchestration: None,
        display_name: model.to_string(),
        description: String::new(),
        default_reasoning_effort: ReasoningEffort::None,
        supported_reasoning_efforts: Vec::new(),
        supports_personality: false,
        additional_speed_tiers: Vec::new(),
        service_tiers: Vec::new(),
        default_service_tier: None,
        is_default: false,
        upgrade: None,
        show_in_picker: true,
        multi_agent_version: None,
        availability_nux: None,
        supported_in_api: true,
        input_modalities: vec![InputModality::Text],
    }
}

#[test]
fn runtime_refresh_replaces_only_gpu_models() {
    let catalog = ModelCatalog::new(vec![
        preset("static", Some("ambient")),
        preset("old-rental", Some("gpu-old")),
    ]);

    catalog.replace_gpu_models(vec![preset("new-rental", Some("gpu-new"))]);

    let models = catalog.try_list_models().expect("infallible catalog read");
    assert_eq!(
        models
            .iter()
            .map(|preset| preset.model.as_str())
            .collect::<Vec<_>>(),
        vec!["static", "new-rental"]
    );
    assert_eq!(
        catalog.provider_for_model("new-rental").as_deref(),
        Some("gpu-new")
    );
}

#[test]
fn resumed_session_recovery_is_consumed_once() {
    let catalog = ModelCatalog::new(Vec::new());
    catalog.set_session_recovery_only(true);

    assert!(catalog.take_session_recovery_only());
    assert!(!catalog.take_session_recovery_only());
}

#[test]
fn duplicate_model_slugs_retain_exact_catalog_provider_identity() {
    let catalog = ModelCatalog::new(vec![
        preset("shared-model", Some("provider-a")),
        preset("shared-model", Some("provider-b")),
    ]);

    let models = catalog.try_list_models().unwrap();
    assert_eq!(models[0].provider_id.as_deref(), Some("provider-a"));
    assert_eq!(models[1].provider_id.as_deref(), Some("provider-b"));
}

#[test]
fn configured_runtime_model_is_added_with_exact_provider_identity() {
    let mut models = vec![preset("shared-model", Some("provider-a"))];

    include_runtime_model(&mut models, "shared-model", "provider-b");

    assert_eq!(models.len(), 2);
    assert_eq!(models[1].model, "shared-model");
    assert_eq!(models[1].provider_id.as_deref(), Some("provider-b"));
    assert_eq!(models[1].id, "provider-b:shared-model");
}

#[test]
fn configured_runtime_model_does_not_duplicate_existing_exact_or_inferred_provider() {
    let mut models = vec![
        preset("custom-model", Some("custom")),
        preset("gpt-5.6-sol", None),
    ];

    include_runtime_model(&mut models, "custom-model", "custom");
    include_runtime_model(&mut models, "gpt-5.6-sol", "openai");

    assert_eq!(models.len(), 2);
}

#[test]
fn runtime_sync_is_idempotent_deterministic_and_preserves_exact_provider_identity() {
    let catalog = ModelCatalog::new(vec![preset("shared-model", Some("provider-a"))]);

    catalog.sync_runtime_models(
        ["provider-c", "provider-a", "provider-b"],
        Some("shared-model"),
    );
    catalog.sync_runtime_models(
        ["provider-b", "provider-c", "provider-a"],
        Some("shared-model"),
    );

    let models = catalog.try_list_models().unwrap();
    assert_eq!(models.len(), 3);
    assert_eq!(models[1].provider_id.as_deref(), Some("provider-b"));
    assert_eq!(models[1].id, "provider-b:shared-model");
    assert_eq!(models[2].provider_id.as_deref(), Some("provider-c"));
    assert_eq!(models[2].id, "provider-c:shared-model");
}
