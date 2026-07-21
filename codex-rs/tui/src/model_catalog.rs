use codex_protocol::openai_models::ModelPreset;
use std::convert::Infallible;
use std::sync::RwLock;

#[derive(Debug)]
pub(crate) struct ModelCatalog {
    models: RwLock<Vec<ModelPreset>>,
}

impl ModelCatalog {
    pub(crate) fn new(models: Vec<ModelPreset>) -> Self {
        Self {
            models: RwLock::new(models),
        }
    }

    pub(crate) fn try_list_models(&self) -> Result<Vec<ModelPreset>, Infallible> {
        Ok(self
            .models
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone())
    }

    pub(crate) fn provider_for_model(&self, model: &str) -> Option<String> {
        self.models
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .iter()
            .find(|preset| preset.model == model)
            .and_then(|preset| preset.provider_id.clone())
    }

    pub(crate) fn replace_gpu_models(&self, runtime_models: Vec<ModelPreset>) {
        let mut models = self
            .models
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        models.retain(|preset| {
            !preset
                .provider_id
                .as_deref()
                .is_some_and(|provider_id| provider_id.starts_with("gpu-"))
        });
        models.extend(runtime_models);
    }
}

#[cfg(test)]
#[path = "model_catalog_tests.rs"]
mod tests;
