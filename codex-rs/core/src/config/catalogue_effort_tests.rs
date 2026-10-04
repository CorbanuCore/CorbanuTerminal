use super::ConfigBuilder;
use codex_protocol::openai_models::ChatReasoningEffortProtocol;
use pretty_assertions::assert_eq;
use tempfile::tempdir;
use toml::Value;

#[tokio::test]
async fn exact_catalogue_efforts_survive_config_load() {
    let catalogue = codex_models_manager::bundled_models_response().expect("catalogue");
    for model in catalogue.models.iter().filter(|model| {
        model.slug.starts_with("glm-")
            && model.chat_completions.reasoning_effort_protocol
                != ChatReasoningEffortProtocol::ProviderDefault
    }) {
        for level in &model.supported_reasoning_levels {
            let home = tempdir().expect("home");
            let config = ConfigBuilder::default()
                .codex_home(home.path().to_path_buf())
                .cli_overrides(vec![
                    (
                        "model_provider".to_string(),
                        Value::String("zai".to_string()),
                    ),
                    ("model".to_string(), Value::String(model.slug.clone())),
                    (
                        "model_reasoning_effort".to_string(),
                        Value::String(level.effort.as_str().to_string()),
                    ),
                ])
                .build()
                .await
                .expect("exact model configuration");
            assert_eq!(
                (config.model, config.model_reasoning_effort),
                (Some(model.slug.clone()), Some(level.effort.clone())),
            );
        }
    }
}
