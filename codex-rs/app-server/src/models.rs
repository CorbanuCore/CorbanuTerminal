use codex_app_server_protocol::Model;
use codex_app_server_protocol::ModelServiceTier;
use codex_app_server_protocol::ModelUpgradeInfo;
use codex_app_server_protocol::ReasoningEffortOption;
use codex_protocol::openai_models::ModelPreset;
use codex_protocol::openai_models::ReasoningEffortPreset;

<<<<<<< HEAD
pub async fn supported_models(
    thread_manager: Arc<ThreadManager>,
    include_hidden: bool,
    http_client_factory: HttpClientFactory,
) -> Vec<Model> {
    thread_manager
        // `model/list` is part of TUI bootstrap, so it must not wait for a
        // provider request on a cold cache. The manager always starts with the
        // bundled catalog and can overlay a fresh disk cache here. The app
        // server's models refresh worker updates it from the network in the
        // background.
        .list_models(RefreshStrategy::Offline, http_client_factory)
        .await
||||||| 413492cd6c
pub async fn supported_models(
    thread_manager: Arc<ThreadManager>,
    include_hidden: bool,
    http_client_factory: HttpClientFactory,
) -> Vec<Model> {
    thread_manager
        .list_models(RefreshStrategy::OnlineIfUncached, http_client_factory)
        .await
=======
pub fn supported_models(models: Vec<ModelPreset>, include_hidden: bool) -> Vec<Model> {
    models
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        .into_iter()
        .filter(|preset| include_hidden || preset.show_in_picker)
        .map(model_from_preset)
        .collect()
}

fn model_from_preset(preset: ModelPreset) -> Model {
    Model {
        id: preset.id.to_string(),
        model: preset.model.to_string(),
        upgrade: preset.upgrade.as_ref().map(|upgrade| upgrade.id.clone()),
        upgrade_info: preset.upgrade.as_ref().map(|upgrade| ModelUpgradeInfo {
            model: upgrade.id.clone(),
            upgrade_copy: upgrade.upgrade_copy.clone(),
            model_link: upgrade.model_link.clone(),
            migration_markdown: upgrade.migration_markdown.clone(),
            retirement_at: upgrade
                .retirement_at
                .as_ref()
                .map(chrono::DateTime::timestamp),
        }),
        availability_nux: preset.availability_nux.map(Into::into),
        display_name: preset.display_name.to_string(),
        description: preset.description.to_string(),
<<<<<<< HEAD
        orchestration: preset.orchestration,
||||||| 413492cd6c
=======
        model_specialty: preset.model_specialty,
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        hidden: !preset.show_in_picker,
        supported_reasoning_efforts: reasoning_efforts_from_preset(
            preset.supported_reasoning_efforts,
        ),
        default_reasoning_effort: preset.default_reasoning_effort,
        input_modalities: preset.input_modalities,
        supports_personality: false,
        multi_agent_version: preset.multi_agent_version.map(Into::into),
        additional_speed_tiers: preset.additional_speed_tiers,
        service_tiers: preset
            .service_tiers
            .into_iter()
            .map(|service_tier| ModelServiceTier {
                id: service_tier.id,
                name: service_tier.name,
                description: service_tier.description,
            })
            .collect(),
        default_service_tier: preset.default_service_tier,
        available_access_programs: preset.available_access_programs.map(Into::into),
        is_default: preset.is_default,
    }
}

fn reasoning_efforts_from_preset(
    efforts: Vec<ReasoningEffortPreset>,
) -> Vec<ReasoningEffortOption> {
    efforts
        .into_iter()
        .map(|preset| ReasoningEffortOption {
            reasoning_effort: preset.effort,
            description: preset.description,
        })
        .collect()
}
