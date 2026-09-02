use codex_app_server_client::AppServerRequestHandle;
use codex_provider_auth::ExplicitProviderSelection;
use codex_provider_auth::ProviderManagementAction;
use codex_provider_auth::ProviderManagementPhase;
use codex_provider_auth::ProviderManagementSession;
use codex_provider_auth::ProviderManagementTransition;
use codex_provider_auth::ProviderStatusSnapshot;

use crate::app_event_sender::AppEventSender;
use crate::legacy_core::config::Config;
use crate::provider_account_auth_host::ProviderAccountAuthHost;
use crate::provider_status_host::ProviderStatusHost;

pub(crate) struct ProviderManagementHost {
    status_host: ProviderStatusHost,
    account_auth_host: ProviderAccountAuthHost,
    session: ProviderManagementSession,
    focused_provider: Option<codex_provider_auth::ProviderCatalogId>,
}

impl ProviderManagementHost {
    pub(crate) fn new_with_statuses(
        config: &Config,
        request_handle: AppServerRequestHandle,
        app_event_tx: AppEventSender,
        status_host: ProviderStatusHost,
        statuses: Vec<ProviderStatusSnapshot>,
    ) -> Self {
        let session = ProviderManagementSession::new(statuses);
        let account_auth_host = ProviderAccountAuthHost::new(
            request_handle,
            app_event_tx,
            status_host.clone(),
            config.clone(),
        );
        Self {
            status_host,
            account_auth_host,
            session,
            focused_provider: None,
        }
    }

    pub(crate) fn status_host(&self) -> &ProviderStatusHost {
        &self.status_host
    }

    pub(crate) fn account_auth_host(&mut self) -> &mut ProviderAccountAuthHost {
        &mut self.account_auth_host
    }

    pub(crate) fn account_auth_host_ref(&self) -> &ProviderAccountAuthHost {
        &self.account_auth_host
    }

    pub(crate) fn statuses(&self) -> &[ProviderStatusSnapshot] {
        self.session.statuses()
    }

    pub(crate) fn phase(&self) -> &ProviderManagementPhase {
        self.session.phase()
    }

    pub(crate) fn focused_provider(&self) -> Option<&codex_provider_auth::ProviderCatalogId> {
        self.focused_provider.as_ref()
    }

    pub(crate) fn remember_focused_provider(
        &mut self,
        provider_id: codex_provider_auth::ProviderCatalogId,
    ) {
        self.focused_provider = Some(provider_id);
    }

    pub(crate) fn authenticating_provider(
        &self,
    ) -> Option<&codex_provider_auth::ProviderCatalogId> {
        match self.session.phase() {
            ProviderManagementPhase::Authenticating { provider_id, .. } => Some(provider_id),
            _ => None,
        }
    }

    pub(crate) fn authenticating_attempt(
        &self,
    ) -> Option<(
        codex_provider_auth::ProviderManagementAttemptId,
        codex_provider_auth::ProviderCatalogId,
    )> {
        match self.session.phase() {
            ProviderManagementPhase::Authenticating {
                attempt_id,
                provider_id,
                ..
            } => Some((*attempt_id, provider_id.clone())),
            _ => None,
        }
    }

    pub(crate) fn dispatch(
        &mut self,
        action: ProviderManagementAction,
    ) -> ProviderManagementTransition {
        let focused_provider = action_provider(&action).cloned();
        let transition = self.session.dispatch(action);
        if transition.applied
            && let Some(provider_id) = focused_provider
        {
            self.remember_focused_provider(provider_id);
        }
        transition
    }

    pub(crate) fn apply_statuses(
        &mut self,
        statuses: Vec<ProviderStatusSnapshot>,
    ) -> ProviderManagementTransition {
        let transition = self
            .session
            .dispatch(ProviderManagementAction::Refresh { statuses });
        if transition.applied
            && self.focused_provider.as_ref().is_some_and(|provider_id| {
                !transition
                    .statuses
                    .iter()
                    .any(|status| status.id == *provider_id)
            })
        {
            self.focused_provider = None;
        }
        transition
    }

    pub(crate) fn update_account_metadata(
        &mut self,
        metadata: crate::provider_status_host::ProviderAccountMetadata,
    ) -> bool {
        self.status_host.update_account_metadata(metadata);
        self.session.can_refresh()
    }

    pub(crate) fn replacement_candidates(
        &self,
        current_model: Option<String>,
    ) -> Vec<ExplicitProviderSelection> {
        replacement_candidates_for(&self.status_host, self.session.statuses(), current_model)
    }
}

fn action_provider(
    action: &ProviderManagementAction,
) -> Option<&codex_provider_auth::ProviderCatalogId> {
    match action {
        ProviderManagementAction::BeginAuthentication { provider_id }
        | ProviderManagementAction::AuthenticationConfigured { provider_id }
        | ProviderManagementAction::AuthenticationCancelled { provider_id }
        | ProviderManagementAction::RequestPolicy { provider_id, .. } => Some(provider_id),
        ProviderManagementAction::ChooseReplacement {
            target_provider_id, ..
        }
        | ProviderManagementAction::CancelReplacement { target_provider_id } => {
            Some(target_provider_id)
        }
        ProviderManagementAction::PersistenceFinished { .. }
        | ProviderManagementAction::Refresh { .. } => None,
    }
}

fn replacement_candidates_for(
    status_host: &ProviderStatusHost,
    statuses: &[ProviderStatusSnapshot],
    current_model: Option<String>,
) -> Vec<ExplicitProviderSelection> {
    statuses
        .iter()
        .filter(|status| {
            status.configuration == codex_provider_auth::ProviderConfigurationState::Configured
                && status.eligibility == codex_provider_auth::ProviderEligibilityState::Active
                && status.current == codex_provider_auth::ProviderCurrentState::NotCurrent
                && status.availability == codex_provider_auth::ProviderAvailabilityState::Ready
        })
        .filter_map(|status| {
            let entry = status_host.catalog().get(status.id.as_str())?;
            let runtime_provider_id = entry.runtime_provider_ids.first()?.clone();
            let model = codex_model_provider_info::resolve_model_for_provider(
                current_model.clone(),
                runtime_provider_id.as_str(),
            )?;
            Some(ExplicitProviderSelection {
                provider_id: status.id.clone(),
                runtime_provider_id,
                model,
            })
        })
        .collect()
}

#[cfg(test)]
#[path = "provider_management_host_tests.rs"]
mod tests;
