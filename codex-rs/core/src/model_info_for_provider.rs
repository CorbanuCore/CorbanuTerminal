//! PF-84 (#572): model metadata for the provider a config runs on.

use codex_model_provider::create_model_provider;
use codex_models_manager::manager::SharedModelsManager;
use codex_protocol::openai_models::ModelInfo;

use crate::config::Config;

/// `models_manager`'s metadata for `model`. A session's models manager is
/// the one of the provider it started on; after a switch to Amazon Bedrock
/// (whose models only Bedrock's own catalog knows) its fallback metadata is
/// replaced by that static catalog's (no network).
pub(crate) async fn model_info_for_config(
    models_manager: &SharedModelsManager,
    model: &str,
    config: &Config,
) -> ModelInfo {
    let models_config = config.to_models_manager_config();
    let model_info = models_manager.get_model_info(model, &models_config).await;
    if !model_info.used_fallback_model_metadata || !config.model_provider.is_amazon_bedrock() {
        return model_info;
    }
    create_model_provider(config.model_provider.clone(), /*auth_manager*/ None)
        .models_manager_without_cache(/*config_model_catalog*/ None)
        .get_model_info(model, &models_config)
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use codex_model_provider_info::ModelProviderInfo;
    use std::sync::Arc;

    /// A session that started on another provider (its models manager
    /// knows no Bedrock model) gets Bedrock's metadata for a Bedrock turn,
    /// and fallback metadata stays for an unknown model elsewhere.
    #[tokio::test]
    async fn bedrock_turn_gets_bedrock_metadata_from_another_providers_manager() {
        let mut config = crate::config::test_config().await;
        let other: SharedModelsManager =
            Arc::new(codex_models_manager::manager::StaticModelsManager::new(
                /*auth_manager*/ None,
                Default::default(),
            ));
        let model = codex_model_provider_info::AMAZON_BEDROCK_GPT_5_4_MODEL_ID;
        assert!(
            other
                .get_model_info(model, &config.to_models_manager_config())
                .await
                .used_fallback_model_metadata
        );

        config.model_provider =
            ModelProviderInfo::create_amazon_bedrock_provider(/*aws*/ None);
        let bedrock = model_info_for_config(&other, model, &config).await;
        assert!(!bedrock.used_fallback_model_metadata);
        assert!(!bedrock.supported_reasoning_levels.is_empty());

        config.model_provider = ModelProviderInfo::create_openai_provider(/*base_url*/ None);
        assert!(
            model_info_for_config(&other, "unknown-model-572", &config)
                .await
                .used_fallback_model_metadata
        );
    }
}
