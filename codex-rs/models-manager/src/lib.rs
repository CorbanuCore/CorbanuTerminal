pub mod cache;
mod catalog;
pub mod collaboration_mode_presets;
pub(crate) mod config;
pub mod manager;
pub mod model_info;
pub mod model_presets;
mod originator;
pub mod test_support;

pub use catalog::CatalogModel;
pub use catalog::ModelOrigin;
pub use codex_protocol::auth::AuthMode;
pub use config::ModelsManagerConfig;
pub use originator::ModelCatalogOriginator;

pub use codex_protocol::openai_models::OPENAI_CODEX_COMPAT_VERSION;

/// Load the bundled model catalog shipped with `codex-models-manager`.
pub fn bundled_models_response()
-> std::result::Result<codex_protocol::openai_models::ModelsResponse, serde_json::Error> {
    // Keep catalog shards below the repository blob limit without duplicating
    // or changing existing model metadata when adding a new release.
    let mut catalog: codex_protocol::openai_models::ModelsResponse =
        serde_json::from_str(include_str!("../models.json"))?;
    let additions: codex_protocol::openai_models::ModelsResponse =
        serde_json::from_str(include_str!("../models-2026-11-17.json"))?;
    catalog.models.extend(additions.models);
    Ok(catalog)
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

    fn instructions(model: &codex_protocol::openai_models::ModelInfo) -> &str {
        model
            .model_messages
            .as_ref()
            .and_then(|messages| messages.instructions_template.as_deref())
            .unwrap_or_default()
    }

    #[test]
    fn astra_overrides_keep_native_instructions_and_context_ceiling() {
        let model = bundled_models_response()
            .expect("bundled models")
            .models
            .into_iter()
            .find(|model| model.slug == "gpt-6-astra")
            .expect("Astra metadata");
        let resolved = model_info::with_config_overrides(
            model.clone(),
            &ModelsManagerConfig {
                model_context_window: Some(1_050_000),
                ..Default::default()
            },
        );
        assert_eq!(resolved.context_window, Some(872_000));
        assert_eq!(instructions(&resolved), instructions(&model));
        assert!(!instructions(&resolved).is_empty());
        assert!(instructions(&resolved).len() < 40_000);
    }

    /// Templates are literal text now; none may still carry the retired
    /// personality placeholder.
    #[test]
    fn bundled_templates_carry_no_personality_placeholder() {
        let stale = bundled_models_response()
            .expect("bundled models")
            .models
            .into_iter()
            .filter(|model| instructions(model).contains("{{ personality }}"))
            .map(|model| model.slug)
            .collect::<Vec<_>>();
        assert_eq!(stale, Vec::<String>::new());
    }

    #[test]
    fn template_only_catalog_preserves_instructions() {
        let mut model = model_info::model_info_from_slug("template-only-fixture");
        let mut messages = bundled_models_response()
            .expect("bundled models")
            .models
            .into_iter()
            .find_map(|model| model.model_messages)
            .expect("model messages");
        messages.instructions_template = Some("Native operating contract".to_string());
        model.model_messages = Some(messages);
        let resolved = model_info::with_config_overrides(model, &ModelsManagerConfig::default());
        assert_eq!(instructions(&resolved), "Native operating contract");
    }
}
