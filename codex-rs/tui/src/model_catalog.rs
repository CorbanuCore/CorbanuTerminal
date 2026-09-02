use codex_model_provider_info::canonical_catalog_provider;
use codex_protocol::openai_models::InputModality;
use codex_protocol::openai_models::ModelPreset;
use codex_protocol::openai_models::ReasoningEffort;
use std::convert::Infallible;
use std::sync::RwLock;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

pub(crate) fn include_runtime_model(
    models: &mut Vec<ModelPreset>,
    model: &str,
    runtime_provider_id: &str,
) {
    let model = model.trim();
    let runtime_provider_id = runtime_provider_id.trim();
    if model.is_empty()
        || runtime_provider_id.is_empty()
        || models.iter().any(|preset| {
            preset.model == model
                && preset
                    .provider_id
                    .as_deref()
                    .or_else(|| canonical_catalog_provider(&preset.model))
                    == Some(runtime_provider_id)
        })
    {
        return;
    }

    let mut preset = models
        .iter()
        .find(|preset| preset.model == model)
        .cloned()
        .unwrap_or_else(|| ModelPreset {
            id: format!("{runtime_provider_id}:{model}"),
            model: model.to_string(),
            provider_id: None,
            orchestration: None,
            display_name: model.to_string(),
            description: "Configured runtime provider".to_string(),
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
        });
    preset.id = format!("{runtime_provider_id}:{model}");
    preset.provider_id = Some(runtime_provider_id.to_string());
    models.push(preset);
}

#[derive(Debug)]
pub(crate) struct ModelCatalog {
    models: RwLock<Vec<ModelPreset>>,
    provider_policy: RwLock<Option<crate::chatwidget::provider_model_policy::ProviderModelPolicy>>,
    session_recovery_only: AtomicBool,
}

impl ModelCatalog {
    pub(crate) fn new(models: Vec<ModelPreset>) -> Self {
        Self {
            models: RwLock::new(models),
            provider_policy: RwLock::new(None),
            session_recovery_only: AtomicBool::new(false),
        }
    }

    pub(crate) fn set_session_recovery_only(&self, session_only: bool) {
        self.session_recovery_only
            .store(session_only, Ordering::Release);
    }

    pub(crate) fn take_session_recovery_only(&self) -> bool {
        self.session_recovery_only.swap(false, Ordering::AcqRel)
    }

    pub(crate) fn set_provider_policy(
        &self,
        policy: crate::chatwidget::provider_model_policy::ProviderModelPolicy,
    ) {
        *self
            .provider_policy
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(policy);
    }

    pub(crate) fn provider_policy(
        &self,
    ) -> Option<crate::chatwidget::provider_model_policy::ProviderModelPolicy> {
        self.provider_policy
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    pub(crate) fn has_provider_policy(&self) -> bool {
        self.provider_policy
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_some()
    }

    pub(crate) fn update_current_provider(&self, runtime_provider_id: &str) {
        if let Some(policy) = self
            .provider_policy
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_mut()
        {
            policy.set_current_runtime(runtime_provider_id);
        }
    }

    pub(crate) fn refresh_provider_policy(&self) {
        if let Some(policy) = self
            .provider_policy
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_mut()
        {
            policy.refresh();
        }
    }

    pub(crate) fn sync_runtime_models<'a>(
        &self,
        runtime_provider_ids: impl IntoIterator<Item = &'a str>,
        preferred_model: Option<&str>,
    ) {
        let mut runtime_provider_ids = runtime_provider_ids.into_iter().collect::<Vec<_>>();
        runtime_provider_ids.sort_unstable();
        let mut models = self
            .models
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for runtime_provider_id in runtime_provider_ids {
            let Some(model) = codex_model_provider_info::resolve_model_for_provider(
                preferred_model.map(str::to_owned),
                runtime_provider_id,
            ) else {
                continue;
            };
            include_runtime_model(&mut models, &model, runtime_provider_id);
        }
    }

    pub(crate) fn preset_is_selectable(&self, preset: &ModelPreset) -> bool {
        self.provider_policy()
            .is_none_or(|policy| policy.preset_is_selectable(preset))
    }

    pub(crate) fn provider_is_selectable(&self, provider: &str, model: &str) -> bool {
        self.provider_policy()
            .is_none_or(|policy| policy.provider_is_selectable(provider, model))
    }

    pub(crate) fn provider_use_decision(
        &self,
        provider: &str,
        model: &str,
        context: codex_provider_auth::ProviderUseContext,
    ) -> Option<codex_provider_auth::ProviderUseDecision> {
        self.provider_policy()
            .map(|policy| policy.assess(provider, model, context))
    }

    pub(crate) fn current_requires_recovery(&self, provider: &str, model: &str) -> bool {
        self.provider_policy().is_some_and(|policy| {
            matches!(
                policy.current(provider, model),
                codex_provider_auth::CurrentSelectionDecision::RequireExplicitRecovery { .. }
            )
        })
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
