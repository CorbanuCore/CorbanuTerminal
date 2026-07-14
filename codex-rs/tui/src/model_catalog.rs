use codex_protocol::openai_models::ModelPreset;
use std::convert::Infallible;

#[derive(Debug, Clone)]
pub(crate) struct ModelCatalog {
    models: Vec<ModelPreset>,
}

impl ModelCatalog {
    pub(crate) fn new(models: Vec<ModelPreset>) -> Self {
        Self { models }
    }

    pub(crate) fn try_list_models(&self) -> Result<Vec<ModelPreset>, Infallible> {
        Ok(self.models.clone())
    }

    pub(crate) fn provider_for_model(&self, model: &str) -> Option<String> {
        self.models
            .iter()
            .find(|preset| preset.model == model)
            .and_then(|preset| preset.provider_id.clone())
    }
}
