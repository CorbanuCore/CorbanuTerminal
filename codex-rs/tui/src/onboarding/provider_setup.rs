use std::collections::BTreeSet;

use codex_provider_auth::ProviderAvailabilityState;
use codex_provider_auth::ProviderCatalogId;
use codex_provider_auth::ProviderConfigurationState;
use codex_provider_auth::ProviderCurrentState;
use codex_provider_auth::ProviderEligibilityState;
use codex_provider_auth::ProviderRuntimeId;
use codex_provider_auth::ProviderSetupCapability;
use codex_provider_auth::ProviderStatusSnapshot;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct ProviderSetupContinuationId(u64);

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DeferredProviderSetup {
    CorbanuPlan {
        continuation_id: ProviderSetupContinuationId,
        has_usable_fallback: bool,
    },
}

impl DeferredProviderSetup {
    pub(crate) fn continuation_id(&self) -> ProviderSetupContinuationId {
        match self {
            Self::CorbanuPlan {
                continuation_id, ..
            } => *continuation_id,
        }
    }

    pub(crate) fn has_usable_fallback(&self) -> bool {
        match self {
            Self::CorbanuPlan {
                has_usable_fallback,
                ..
            } => *has_usable_fallback,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ProviderSetupPhase {
    ProviderList,
    Authenticating {
        provider_id: ProviderCatalogId,
    },
    Activating {
        provider_id: ProviderCatalogId,
        runtime_provider_id: ProviderRuntimeId,
    },
    Deferred(DeferredProviderSetup),
    Finished,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProviderSetupSnapshot {
    pub(crate) phase: ProviderSetupPhase,
    pub(crate) configured: BTreeSet<ProviderCatalogId>,
    pub(crate) usable: BTreeSet<ProviderCatalogId>,
    pub(crate) queued_corbanu: bool,
    pub(crate) first_fresh_runtime: Option<ProviderRuntimeId>,
    pub(crate) preserve_initial_current: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ProviderSetupAction {
    Begin {
        provider_id: ProviderCatalogId,
    },
    AuthConfigured {
        provider_id: ProviderCatalogId,
        runtime_provider_id: ProviderRuntimeId,
    },
    ActivationResolved {
        provider_id: ProviderCatalogId,
        runtime_provider_id: ProviderRuntimeId,
        usable: bool,
    },
    SelectExisting {
        provider_id: ProviderCatalogId,
        runtime_provider_id: ProviderRuntimeId,
    },
    AuthCancelled,
    AuthFailed,
    QueueCorbanu(bool),
    Done,
    DeferredPlanConfigured {
        continuation_id: ProviderSetupContinuationId,
    },
    DeferredPlanCancelled {
        continuation_id: ProviderSetupContinuationId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ProviderSetupEffect {
    Activate(ProviderCatalogId),
    PersistInitialSelection(ProviderRuntimeId),
    BeginDeferred(DeferredProviderSetup),
    Finish,
    ReturnToProviderList,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProviderSetupTransition {
    pub(crate) snapshot: ProviderSetupSnapshot,
    pub(crate) effects: Vec<ProviderSetupEffect>,
    pub(crate) applied: bool,
}

pub(crate) struct ProviderSetupSession {
    snapshot: ProviderSetupSnapshot,
    next_continuation_id: u64,
}

impl ProviderSetupSession {
    pub(crate) fn from_statuses(statuses: &[ProviderStatusSnapshot]) -> Self {
        let configured = statuses
            .iter()
            .filter(|status| status.configuration == ProviderConfigurationState::Configured)
            .map(|status| status.id.clone())
            .collect();
        let usable = statuses
            .iter()
            .filter(|status| provider_is_usable(status))
            .map(|status| status.id.clone())
            .collect();
        let preserve_initial_current = statuses.iter().any(|status| {
            status.current == ProviderCurrentState::Current && provider_is_usable(status)
        });
        Self {
            snapshot: ProviderSetupSnapshot {
                phase: ProviderSetupPhase::ProviderList,
                configured,
                usable,
                queued_corbanu: false,
                first_fresh_runtime: None,
                preserve_initial_current,
            },
            next_continuation_id: 1,
        }
    }

    pub(crate) fn snapshot(&self) -> &ProviderSetupSnapshot {
        &self.snapshot
    }

    pub(crate) fn can_finish(&self) -> bool {
        self.has_selected_usable_provider() || self.snapshot.queued_corbanu
    }

    pub(crate) fn can_refresh_statuses(&self) -> bool {
        matches!(self.snapshot.phase, ProviderSetupPhase::ProviderList)
    }

    pub(crate) fn refresh_from_statuses(&mut self, statuses: &[ProviderStatusSnapshot]) -> bool {
        if !matches!(self.snapshot.phase, ProviderSetupPhase::ProviderList) {
            return false;
        }
        self.snapshot.configured = statuses
            .iter()
            .filter(|status| status.configuration == ProviderConfigurationState::Configured)
            .map(|status| status.id.clone())
            .collect();
        self.snapshot.usable = statuses
            .iter()
            .filter(|status| provider_is_usable(status))
            .map(|status| status.id.clone())
            .collect();
        self.snapshot.preserve_initial_current = statuses.iter().any(|status| {
            status.current == ProviderCurrentState::Current && provider_is_usable(status)
        });
        true
    }

    pub(crate) fn dispatch(&mut self, action: ProviderSetupAction) -> ProviderSetupTransition {
        let mut effects = Vec::new();
        let applied = match action {
            ProviderSetupAction::Begin { provider_id }
                if matches!(self.snapshot.phase, ProviderSetupPhase::ProviderList) =>
            {
                self.snapshot.phase = ProviderSetupPhase::Authenticating { provider_id };
                true
            }
            ProviderSetupAction::AuthConfigured {
                provider_id,
                runtime_provider_id,
            } if matches!(
                &self.snapshot.phase,
                ProviderSetupPhase::Authenticating { provider_id: expected }
                    if *expected == provider_id
            ) =>
            {
                self.snapshot.phase = ProviderSetupPhase::Activating {
                    provider_id: provider_id.clone(),
                    runtime_provider_id,
                };
                effects.push(ProviderSetupEffect::Activate(provider_id));
                true
            }
            ProviderSetupAction::ActivationResolved {
                provider_id,
                runtime_provider_id,
                usable,
            } if matches!(
                &self.snapshot.phase,
                ProviderSetupPhase::Activating {
                    provider_id: expected_provider,
                    runtime_provider_id: expected_runtime,
                } if *expected_provider == provider_id && *expected_runtime == runtime_provider_id
            ) =>
            {
                self.snapshot.configured.insert(provider_id.clone());
                if usable {
                    self.snapshot.usable.insert(provider_id);
                }
                if usable
                    && !self.snapshot.preserve_initial_current
                    && self.snapshot.first_fresh_runtime.is_none()
                {
                    self.snapshot.first_fresh_runtime = Some(runtime_provider_id.clone());
                    effects.push(ProviderSetupEffect::PersistInitialSelection(
                        runtime_provider_id,
                    ));
                }
                self.snapshot.phase = ProviderSetupPhase::ProviderList;
                true
            }
            ProviderSetupAction::SelectExisting {
                provider_id,
                runtime_provider_id,
            } if matches!(self.snapshot.phase, ProviderSetupPhase::ProviderList) => {
                self.snapshot.usable.insert(provider_id);
                // Unlike enrollment, this action is an explicit provider choice. Honor it even
                // when another provider was current when the setup session opened.
                self.snapshot.preserve_initial_current = false;
                self.snapshot.first_fresh_runtime = Some(runtime_provider_id.clone());
                effects.push(ProviderSetupEffect::PersistInitialSelection(
                    runtime_provider_id,
                ));
                true
            }
            ProviderSetupAction::AuthCancelled | ProviderSetupAction::AuthFailed
                if matches!(
                    self.snapshot.phase,
                    ProviderSetupPhase::Authenticating { .. }
                        | ProviderSetupPhase::Activating { .. }
                ) =>
            {
                self.snapshot.phase = ProviderSetupPhase::ProviderList;
                effects.push(ProviderSetupEffect::ReturnToProviderList);
                true
            }
            ProviderSetupAction::QueueCorbanu(queued)
                if matches!(self.snapshot.phase, ProviderSetupPhase::ProviderList) =>
            {
                self.snapshot.queued_corbanu = queued;
                true
            }
            ProviderSetupAction::Done
                if matches!(self.snapshot.phase, ProviderSetupPhase::ProviderList)
                    && self.can_finish() =>
            {
                if self.snapshot.queued_corbanu {
                    let continuation = DeferredProviderSetup::CorbanuPlan {
                        continuation_id: self.allocate_continuation_id(),
                        has_usable_fallback: self.has_selected_usable_provider(),
                    };
                    self.snapshot.phase = ProviderSetupPhase::Deferred(continuation.clone());
                    effects.push(ProviderSetupEffect::BeginDeferred(continuation));
                } else {
                    self.snapshot.phase = ProviderSetupPhase::Finished;
                    effects.push(ProviderSetupEffect::Finish);
                }
                true
            }
            ProviderSetupAction::DeferredPlanConfigured { continuation_id }
                if self.matches_continuation(continuation_id) =>
            {
                self.snapshot.phase = ProviderSetupPhase::Finished;
                effects.push(ProviderSetupEffect::Finish);
                true
            }
            ProviderSetupAction::DeferredPlanCancelled { continuation_id }
                if self.matches_continuation(continuation_id) =>
            {
                if !self.has_selected_usable_provider() {
                    self.snapshot.queued_corbanu = false;
                    self.snapshot.phase = ProviderSetupPhase::ProviderList;
                    effects.push(ProviderSetupEffect::ReturnToProviderList);
                } else {
                    self.snapshot.phase = ProviderSetupPhase::Finished;
                    effects.push(ProviderSetupEffect::Finish);
                }
                true
            }
            _ => false,
        };
        ProviderSetupTransition {
            snapshot: self.snapshot.clone(),
            effects,
            applied,
        }
    }

    fn allocate_continuation_id(&mut self) -> ProviderSetupContinuationId {
        let id = ProviderSetupContinuationId(self.next_continuation_id);
        self.next_continuation_id = self.next_continuation_id.saturating_add(1);
        id
    }

    fn matches_continuation(&self, id: ProviderSetupContinuationId) -> bool {
        matches!(
            &self.snapshot.phase,
            ProviderSetupPhase::Deferred(DeferredProviderSetup::CorbanuPlan {
                continuation_id,
                ..
            }) if *continuation_id == id
        )
    }

    fn has_selected_usable_provider(&self) -> bool {
        self.snapshot.preserve_initial_current || self.snapshot.first_fresh_runtime.is_some()
    }
}

fn provider_is_usable(status: &ProviderStatusSnapshot) -> bool {
    let configured_and_ready = status.configuration == ProviderConfigurationState::Configured
        && status.eligibility == ProviderEligibilityState::Active
        && status.availability == ProviderAvailabilityState::Ready;
    if !configured_and_ready {
        return false;
    }

    // Local, command-auth, and status-only entries have no interactive setup action. They may be
    // selected explicitly and validated at the real runtime boundary, but an incidental built-in
    // entry must not make a fresh setup completable or preserve an unrelated missing current.
    let requires_explicit_selection = status.methods.iter().any(|method| {
        matches!(
            method.capability,
            ProviderSetupCapability::Local { .. }
                | ProviderSetupCapability::CommandAuth { .. }
                | ProviderSetupCapability::StatusOnly { .. }
        )
    });
    !requires_explicit_selection || status.current == ProviderCurrentState::Current
}

pub(crate) fn provider_is_explicitly_selectable(status: &ProviderStatusSnapshot) -> bool {
    !matches!(
        status.eligibility,
        ProviderEligibilityState::Inactive | ProviderEligibilityState::Unavailable
    ) && matches!(
        status.availability,
        ProviderAvailabilityState::Ready | ProviderAvailabilityState::StatusOnly
    )
}

pub(crate) fn provider_should_offer_existing_selection(
    status: &ProviderStatusSnapshot,
    has_noninteractive_capability: bool,
) -> bool {
    provider_is_explicitly_selectable(status)
        && (status.configuration == ProviderConfigurationState::Configured
            || has_noninteractive_capability)
}

#[cfg(test)]
#[path = "provider_setup_tests.rs"]
mod tests;
