use std::collections::BTreeMap;

use codex_model_provider_info::canonical_provider_id;

use crate::ExplicitProviderSelection;
use crate::ProviderAvailabilityState;
use crate::ProviderCatalog;
use crate::ProviderConfigurationState;
use crate::ProviderEligibilityState;
use crate::ProviderRuntimeId;
use crate::ProviderStatusCatalog;

/// The product boundary requesting permission to use a provider runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderUseContext {
    AutomaticDefault,
    ModelPicker,
    ExplicitRequest,
    Resume,
    NativeSpawn,
}

/// Secret-free outcome of a provider's real runtime authorization adapter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderRuntimeAuthorization {
    NotChecked,
    Authorized,
    Rejected,
}

/// Authorization results keyed by exact runtime-provider identity.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProviderRuntimeAuthorizations(BTreeMap<String, ProviderRuntimeAuthorization>);

impl ProviderRuntimeAuthorizations {
    pub fn set(&mut self, runtime_provider_id: impl Into<String>, state: ProviderRuntimeAuthorization) {
        self.0.insert(runtime_provider_id.into(), state);
    }

    pub fn get(&self, runtime_provider_id: &str) -> ProviderRuntimeAuthorization {
        self.0
            .get(runtime_provider_id)
            .copied()
            .unwrap_or(ProviderRuntimeAuthorization::NotChecked)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderUseBlocker {
    UnknownProvider,
    Inactive,
    NotConfigured,
    Checking,
    RecoveryRequired,
    Unavailable,
    RuntimeAuthorizationRequired,
    RuntimeAuthorizationRejected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderUseDecision {
    Ready(ExplicitProviderSelection),
    RequiresRuntimeAuthorization(ExplicitProviderSelection),
    Blocked {
        requested_runtime_provider_id: String,
        reason: ProviderUseBlocker,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CurrentSelectionDecision {
    Preserve(ExplicitProviderSelection),
    RequireExplicitRecovery {
        requested_runtime_provider_id: String,
        requested_model: String,
        reason: ProviderUseBlocker,
    },
}

pub struct ProviderRuntimeSelectionPolicy;

impl ProviderRuntimeSelectionPolicy {
    pub fn assess(
        catalog: &ProviderCatalog,
        statuses: &ProviderStatusCatalog,
        authorizations: &ProviderRuntimeAuthorizations,
        runtime_provider_id: &str,
        model: impl Into<String>,
        context: ProviderUseContext,
    ) -> ProviderUseDecision {
        let model = model.into();
        let Some(entry) = catalog.entries().iter().find(|entry| {
            entry.runtime_provider_ids.iter().any(|runtime| {
                runtime.as_str() == runtime_provider_id
                    || canonical_provider_id(runtime.as_str())
                        == canonical_provider_id(runtime_provider_id)
            })
        }) else {
            return blocked(runtime_provider_id, ProviderUseBlocker::UnknownProvider);
        };
        let Some(status) = statuses.get(entry.id.as_str()) else {
            return blocked(runtime_provider_id, ProviderUseBlocker::Unavailable);
        };
        let selection = ExplicitProviderSelection {
            provider_id: entry.id.clone(),
            runtime_provider_id: ProviderRuntimeId(runtime_provider_id.to_string()),
            model,
        };

        if status.eligibility == ProviderEligibilityState::Inactive {
            return blocked(runtime_provider_id, ProviderUseBlocker::Inactive);
        }
        if status.availability == ProviderAvailabilityState::StatusOnly {
            return match authorizations.get(runtime_provider_id) {
                ProviderRuntimeAuthorization::Authorized => ProviderUseDecision::Ready(selection),
                ProviderRuntimeAuthorization::Rejected => blocked(
                    runtime_provider_id,
                    ProviderUseBlocker::RuntimeAuthorizationRejected,
                ),
                ProviderRuntimeAuthorization::NotChecked
                    if matches!(
                        context,
                        ProviderUseContext::ExplicitRequest
                            | ProviderUseContext::Resume
                            | ProviderUseContext::NativeSpawn
                    ) => ProviderUseDecision::RequiresRuntimeAuthorization(selection),
                ProviderRuntimeAuthorization::NotChecked => blocked(
                    runtime_provider_id,
                    ProviderUseBlocker::RuntimeAuthorizationRequired,
                ),
            };
        }
        if status.configuration == ProviderConfigurationState::Checking
            || status.availability == ProviderAvailabilityState::Checking
        {
            return blocked(runtime_provider_id, ProviderUseBlocker::Checking);
        }
        if status.configuration == ProviderConfigurationState::RecoveryRequired {
            return blocked(runtime_provider_id, ProviderUseBlocker::RecoveryRequired);
        }
        if status.configuration == ProviderConfigurationState::NotConfigured
            || status.eligibility == ProviderEligibilityState::NotConfigured
        {
            return blocked(runtime_provider_id, ProviderUseBlocker::NotConfigured);
        }
        if status.configuration == ProviderConfigurationState::Unavailable
            || status.eligibility == ProviderEligibilityState::Unavailable
            || !matches!(status.availability, ProviderAvailabilityState::Ready)
        {
            return blocked(runtime_provider_id, ProviderUseBlocker::Unavailable);
        }
        if status.eligibility != ProviderEligibilityState::Active {
            return blocked(runtime_provider_id, ProviderUseBlocker::Unavailable);
        }
        ProviderUseDecision::Ready(selection)
    }

    pub fn current(
        catalog: &ProviderCatalog,
        statuses: &ProviderStatusCatalog,
        authorizations: &ProviderRuntimeAuthorizations,
        runtime_provider_id: &str,
        model: impl Into<String>,
    ) -> CurrentSelectionDecision {
        let model = model.into();
        match Self::assess(
            catalog,
            statuses,
            authorizations,
            runtime_provider_id,
            model.clone(),
            ProviderUseContext::ExplicitRequest,
        ) {
            ProviderUseDecision::Ready(selection) => CurrentSelectionDecision::Preserve(selection),
            ProviderUseDecision::RequiresRuntimeAuthorization(_) => {
                CurrentSelectionDecision::RequireExplicitRecovery {
                    requested_runtime_provider_id: runtime_provider_id.to_string(),
                    requested_model: model,
                    reason: ProviderUseBlocker::RuntimeAuthorizationRequired,
                }
            }
            ProviderUseDecision::Blocked { reason, .. } => {
                CurrentSelectionDecision::RequireExplicitRecovery {
                    requested_runtime_provider_id: runtime_provider_id.to_string(),
                    requested_model: model,
                    reason,
                }
            }
        }
    }
}

fn blocked(runtime_provider_id: &str, reason: ProviderUseBlocker) -> ProviderUseDecision {
    ProviderUseDecision::Blocked {
        requested_runtime_provider_id: runtime_provider_id.to_string(),
        reason,
    }
}

#[cfg(test)]
#[path = "runtime_selection_tests.rs"]
mod tests;
