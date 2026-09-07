pub(crate) mod cache;
pub mod collaboration_mode_presets;
pub(crate) mod config;
pub mod manager;
pub mod model_info;
pub mod model_presets;
pub mod test_support;

pub use codex_protocol::auth::AuthMode;
pub use config::ModelsManagerConfig;

pub use codex_protocol::openai_models::OPENAI_CODEX_COMPAT_VERSION;

/// Load the bundled model catalog shipped with `codex-models-manager`.
pub fn bundled_models_response()
-> std::result::Result<codex_protocol::openai_models::ModelsResponse, serde_json::Error> {
    serde_json::from_str(include_str!("../models.json"))
}

/// Convert the client version string to a whole version string (e.g. "1.2.3-alpha.4" -> "1.2.3").
pub fn client_version_to_whole() -> String {
    OPENAI_CODEX_COMPAT_VERSION.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn client_version_uses_openai_compat_version() {
        assert_eq!(client_version_to_whole(), OPENAI_CODEX_COMPAT_VERSION);
    }

    #[test]
    fn astra_overrides_keep_native_instructions_and_context_ceiling() {
        let model = bundled_models_response()
            .expect("bundled models")
            .models
            .into_iter()
            .find(|model| model.slug == "gpt-6-astra")
            .expect("Astra metadata");
        for personality_enabled in [false, true] {
            let resolved = model_info::with_config_overrides(
                model.clone(),
                &ModelsManagerConfig {
                    personality_enabled,
                    model_context_window: Some(1_050_000),
                    ..Default::default()
                },
            );
            assert_eq!(resolved.context_window, Some(872_000));
            let instructions = resolved.get_model_instructions(None);
            assert_eq!(instructions, model.base_instructions);
            assert!(instructions.len() < 40_000);
        }
    }

    #[test]
    fn template_only_catalog_preserves_instructions_without_personality() {
        let mut model = model_info::model_info_from_slug("template-only-fixture");
        let mut messages = bundled_models_response()
            .expect("bundled models")
            .models
            .into_iter()
            .find_map(|model| model.model_messages)
            .expect("model messages");
        messages.instructions_template = Some("Native operating contract".to_string());
        messages.instructions_variables = None;
        model.base_instructions.clear();
        model.model_messages = Some(messages);
        let resolved = model_info::with_config_overrides(model, &ModelsManagerConfig::default());
        assert_eq!(
            resolved.get_model_instructions(None),
            "Native operating contract"
        );
    }
}
