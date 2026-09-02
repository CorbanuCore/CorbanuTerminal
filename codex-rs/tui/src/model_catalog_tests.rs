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
