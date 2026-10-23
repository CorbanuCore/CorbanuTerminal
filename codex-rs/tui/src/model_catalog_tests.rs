use super::*;
use codex_protocol::openai_models::InputModality;
use codex_protocol::openai_models::ReasoningEffort;
use pretty_assertions::assert_eq;

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
    catalog.set_session_recovery_only(/*session_only*/ true);

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
        preset("gpt-5.6-sol", /*provider_id*/ None),
    ];

    include_runtime_model(&mut models, "custom-model", "custom");
    include_runtime_model(&mut models, "gpt-5.6-sol", "openai");

    assert_eq!(models.len(), 2);
}

#[test]
fn runtime_sync_preserves_shared_opaque_custom_routes_but_not_idle_local_runtimes() {
    let catalog = ModelCatalog::new(vec![preset("shared-model", Some("provider-a"))]);

    catalog.sync_runtime_models(
        [
            "provider-c",
            "provider-a",
            "provider-b",
            "ollama",
            "lmstudio",
        ],
        Some("shared-model"),
        "provider-b",
    );
    catalog.sync_runtime_models(
        ["provider-b", "provider-c", "provider-a"],
        Some("shared-model"),
        "provider-b",
    );

    let models = catalog.try_list_models().unwrap();
    assert_eq!(models.len(), 3);
    assert_eq!(models[1].provider_id.as_deref(), Some("provider-b"));
    assert_eq!(models[1].id, "provider-b:shared-model");
    assert_eq!(models[2].id, "provider-c:shared-model");
}

#[test]
fn claude_current_does_not_seed_local_or_custom_providers_on_manager_refresh() {
    use codex_model_provider_info::*;
    let current = CLAUDE_FABLE_5_1_PLAN_MODEL;
    let catalog = ModelCatalog::new(vec![preset(current, Some(CLAUDE_PLAN_PROVIDER_ID))]);
    for _ in 0..3 {
        catalog.sync_runtime_models(
            [
                CLAUDE_PLAN_PROVIDER_ID,
                "ollama",
                "lmstudio",
                "custom",
                ANTHROPIC_PROVIDER_ID,
            ],
            Some(current),
            CLAUDE_PLAN_PROVIDER_ID,
        );
    }
    let models = catalog.try_list_models().unwrap();
    assert_eq!(
        models
            .iter()
            .map(|p| (p.model.as_str(), p.provider_id.as_deref()))
            .collect::<Vec<_>>(),
        vec![
            (current, Some(CLAUDE_PLAN_PROVIDER_ID)),
            (ANTHROPIC_DEFAULT_MODEL, Some(ANTHROPIC_PROVIDER_ID))
        ],
    );
}

#[test]
fn runtime_refresh_does_not_clone_models_into_incompatible_provider_tabs() {
    use codex_model_provider_info::CLAUDE_FABLE_5_1_PLAN_MODEL;
    use codex_model_provider_info::CLAUDE_FABLE_5_PLAN_MODEL;
    use codex_model_provider_info::CLAUDE_PLAN_MODEL;
    use codex_model_provider_info::CLAUDE_PLAN_PROVIDER_ID;
    use codex_model_provider_info::OPENAI_PROVIDER_ID;
    use codex_model_provider_info::ZAI_DEFAULT_MODEL;

    for model in [
        CLAUDE_PLAN_MODEL,
        CLAUDE_FABLE_5_PLAN_MODEL,
        CLAUDE_FABLE_5_1_PLAN_MODEL,
        ZAI_DEFAULT_MODEL,
    ] {
        let owner = canonical_catalog_provider(model).unwrap();
        let catalog = ModelCatalog::new(vec![preset(model, Some(owner))]);
        for _ in 0..2 {
            catalog.sync_runtime_models(
                [OPENAI_PROVIDER_ID, CLAUDE_PLAN_PROVIDER_ID],
                Some(model),
                owner,
            );
        }
        let models = catalog.try_list_models().unwrap();
        assert!(
            models
                .iter()
                .all(|p| p.model != model || p.provider_id.as_deref() == Some(owner))
        );
    }
}

#[test]
fn provider_models_are_added_once_with_exact_identity_and_never_as_default() {
    let mut listed = preset("openai.gpt-5.5", Some("amazon-bedrock"));
    listed.display_name = "listed".to_string();
    let catalog = ModelCatalog::new(vec![listed, preset("glm-5.3-flash", Some("zai"))]);
    let mut extra = preset("openai.gpt-5.4", /*provider_id*/ None);
    extra.is_default = true;
    let added = vec![preset("openai.gpt-5.5", /*provider_id*/ None), extra];

    catalog.set_provider_models("amazon-bedrock", added.clone(), /*current_model*/ None);
    catalog.set_provider_models("amazon-bedrock", added, /*current_model*/ None);

    let models = catalog.try_list_models().unwrap();
    assert_eq!(models.len(), 3);
    assert_eq!(models[0].display_name, "listed");
    assert_eq!(models[2].id, "amazon-bedrock:openai.gpt-5.4");
    assert_eq!(models[2].provider_id.as_deref(), Some("amazon-bedrock"));
    assert!(!models[2].is_default);

    // Setting none removes only the ones added this way, except the model
    // the session is on.
    catalog.set_provider_models("amazon-bedrock", Vec::new(), Some("openai.gpt-5.4"));
    let ids = catalog
        .try_list_models()
        .unwrap()
        .into_iter()
        .map(|preset| preset.id)
        .collect::<Vec<_>>();
    assert_eq!(
        ids,
        vec![
            "openai.gpt-5.5",
            "glm-5.3-flash",
            "amazon-bedrock:openai.gpt-5.4"
        ]
    );
    catalog.set_provider_models("amazon-bedrock", Vec::new(), /*current_model*/ None);
    let ids = catalog
        .try_list_models()
        .unwrap()
        .into_iter()
        .map(|preset| preset.id)
        .collect::<Vec<_>>();
    assert_eq!(ids, vec!["openai.gpt-5.5", "glm-5.3-flash"]);
}

/// PF-84: a model list fetched for another account replaces the listed
/// models (one the old account had is gone, a new one appears) and keeps the
/// models added for runtime and GPU providers.
#[test]
fn listed_models_are_replaced_for_another_account() {
    let listed = vec![
        preset("shared", /*provider_id*/ None),
        preset("chatgpt-only", /*provider_id*/ None),
    ];
    let mut all = listed.clone();
    all.push(preset("rental", Some("gpu-a")));
    let catalog = ModelCatalog::new(all);
    catalog.mark_listed(&listed);
    include_runtime_model(
        &mut catalog.models.write().expect("models"),
        "custom",
        "zai",
    );

    catalog.replace_listed_models(vec![
        preset("shared", /*provider_id*/ None),
        preset("api-only", /*provider_id*/ None),
    ]);

    let models = catalog.try_list_models().expect("infallible catalog read");
    assert_eq!(
        models
            .iter()
            .map(|preset| preset.id.as_str())
            .collect::<Vec<_>>(),
        vec!["shared", "api-only", "rental", "zai:custom"]
    );
}
