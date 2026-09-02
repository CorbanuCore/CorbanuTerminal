use crate::ProviderActivationPolicy;
use crate::ProviderAvailabilityState;
use crate::ProviderCatalogId;
use crate::ProviderConfigurationState;
use crate::ProviderCurrentState;
use crate::ProviderEligibilityState;
use crate::ProviderManagementAction;
use crate::ProviderManagementAttemptId;
use crate::ProviderManagementEffect;
use crate::ProviderManagementMutation;
use crate::ProviderManagementPersistenceResult;
use crate::ProviderManagementPhase;
use crate::ProviderManagementTransition;
use crate::ProviderStatusSnapshot;

pub struct ProviderManagementSession {
    phase: ProviderManagementPhase,
    statuses: Vec<ProviderStatusSnapshot>,
    next_attempt_id: u64,
}

impl ProviderManagementSession {
    pub fn new(statuses: Vec<ProviderStatusSnapshot>) -> Self {
        Self {
            phase: ProviderManagementPhase::Browsing,
            statuses,
            next_attempt_id: 1,
        }
    }

    pub fn phase(&self) -> &ProviderManagementPhase {
        &self.phase
    }

    pub fn statuses(&self) -> &[ProviderStatusSnapshot] {
        &self.statuses
    }

    pub fn can_refresh(&self) -> bool {
        matches!(self.phase, ProviderManagementPhase::Browsing)
    }

    pub fn dispatch(&mut self, action: ProviderManagementAction) -> ProviderManagementTransition {
        let mut effects = Vec::new();
        let mut persistence_result = None;
        let applied = match action {
            ProviderManagementAction::BeginAuthentication { provider_id }
                if matches!(self.phase, ProviderManagementPhase::Browsing) =>
            {
                let Some(status) = self.status(&provider_id) else {
                    return self.transition(false, effects, persistence_result);
                };
                let preserve_inactive = status.configuration
                    == ProviderConfigurationState::Configured
                    && status.eligibility == ProviderEligibilityState::Inactive;
                let attempt_id = self.allocate_attempt();
                self.phase = ProviderManagementPhase::Authenticating {
                    attempt_id,
                    provider_id: provider_id.clone(),
                    preserve_inactive,
                };
                effects.push(ProviderManagementEffect::BeginAuthentication {
                    attempt_id,
                    provider_id,
                });
                true
            }
            ProviderManagementAction::AuthenticationConfigured { provider_id } => {
                let preserve_inactive = match &self.phase {
                    ProviderManagementPhase::Authenticating {
                        provider_id: expected,
                        preserve_inactive,
                        ..
                    } if *expected == provider_id => *preserve_inactive,
                    _ => return self.transition(false, effects, persistence_result),
                };
                if preserve_inactive {
                    self.phase = ProviderManagementPhase::Browsing;
                    effects.push(ProviderManagementEffect::Refresh);
                } else {
                    let attempt_id = self.allocate_attempt();
                    let mutation = ProviderManagementMutation::Eligibility {
                        provider_id: provider_id.clone(),
                        policy: ProviderActivationPolicy::Active,
                    };
                    self.phase = ProviderManagementPhase::Persisting {
                        attempt_id,
                        mutation,
                    };
                    effects.push(ProviderManagementEffect::PersistEligibility {
                        attempt_id,
                        provider_id,
                        policy: ProviderActivationPolicy::Active,
                    });
                }
                true
            }
            ProviderManagementAction::AuthenticationCancelled { provider_id } => {
                if !matches!(
                    &self.phase,
                    ProviderManagementPhase::Authenticating {
                        provider_id: expected,
                        ..
                    } if *expected == provider_id
                ) {
                    return self.transition(false, effects, persistence_result);
                }
                self.phase = ProviderManagementPhase::Browsing;
                effects.push(ProviderManagementEffect::Refresh);
                true
            }
            ProviderManagementAction::RequestPolicy {
                provider_id,
                policy,
            } if matches!(self.phase, ProviderManagementPhase::Browsing) => {
                let Some(status) = self.status(&provider_id) else {
                    return self.transition(false, effects, persistence_result);
                };
                if status.configuration != ProviderConfigurationState::Configured {
                    return self.transition(false, effects, persistence_result);
                }
                match policy {
                    ProviderActivationPolicy::Active
                        if status.eligibility == ProviderEligibilityState::Inactive =>
                    {
                        self.begin_eligibility_mutation(provider_id, policy, &mut effects);
                        true
                    }
                    ProviderActivationPolicy::Inactive
                        if status.eligibility == ProviderEligibilityState::Active
                            && status.current == ProviderCurrentState::NotCurrent =>
                    {
                        self.begin_eligibility_mutation(provider_id, policy, &mut effects);
                        true
                    }
                    ProviderActivationPolicy::Inactive
                        if status.eligibility == ProviderEligibilityState::Active
                            && status.current == ProviderCurrentState::Current =>
                    {
                        self.phase = ProviderManagementPhase::AwaitingReplacement {
                            target_provider_id: provider_id.clone(),
                        };
                        effects.push(ProviderManagementEffect::PresentReplacement {
                            target_provider_id: provider_id,
                        });
                        true
                    }
                    ProviderActivationPolicy::Active | ProviderActivationPolicy::Inactive => false,
                }
            }
            ProviderManagementAction::ChooseReplacement {
                target_provider_id,
                replacement,
            } => {
                if !matches!(
                    &self.phase,
                    ProviderManagementPhase::AwaitingReplacement {
                        target_provider_id: expected,
                    } if *expected == target_provider_id
                ) || replacement.provider_id == target_provider_id
                    || !self
                        .status(&replacement.provider_id)
                        .is_some_and(is_usable_replacement)
                {
                    return self.transition(false, effects, persistence_result);
                }
                let attempt_id = self.allocate_attempt();
                let mutation = ProviderManagementMutation::ReplacementThenDeactivate {
                    target_provider_id: target_provider_id.clone(),
                    replacement: replacement.clone(),
                };
                self.phase = ProviderManagementPhase::Persisting {
                    attempt_id,
                    mutation,
                };
                effects.push(ProviderManagementEffect::PersistReplacementThenDeactivate {
                    attempt_id,
                    target_provider_id,
                    replacement,
                });
                true
            }
            ProviderManagementAction::CancelReplacement { target_provider_id } => {
                if !matches!(
                    &self.phase,
                    ProviderManagementPhase::AwaitingReplacement {
                        target_provider_id: expected,
                    } if *expected == target_provider_id
                ) {
                    return self.transition(false, effects, persistence_result);
                }
                self.phase = ProviderManagementPhase::Browsing;
                effects.push(ProviderManagementEffect::Refresh);
                true
            }
            ProviderManagementAction::PersistenceFinished { attempt_id, result } => {
                if !matches!(
                    &self.phase,
                    ProviderManagementPhase::Persisting {
                        attempt_id: expected,
                        ..
                    } if *expected == attempt_id
                ) {
                    return self.transition(false, effects, persistence_result);
                }
                self.phase = ProviderManagementPhase::Browsing;
                persistence_result = Some(result);
                effects.push(ProviderManagementEffect::Refresh);
                true
            }
            ProviderManagementAction::Refresh { statuses } if self.can_refresh() => {
                self.statuses = statuses;
                true
            }
            _ => false,
        };
        self.transition(applied, effects, persistence_result)
    }

    fn status(&self, provider_id: &ProviderCatalogId) -> Option<&ProviderStatusSnapshot> {
        self.statuses
            .iter()
            .find(|status| status.id == *provider_id)
    }

    fn begin_eligibility_mutation(
        &mut self,
        provider_id: ProviderCatalogId,
        policy: ProviderActivationPolicy,
        effects: &mut Vec<ProviderManagementEffect>,
    ) {
        let attempt_id = self.allocate_attempt();
        let mutation = ProviderManagementMutation::Eligibility {
            provider_id: provider_id.clone(),
            policy,
        };
        self.phase = ProviderManagementPhase::Persisting {
            attempt_id,
            mutation,
        };
        effects.push(ProviderManagementEffect::PersistEligibility {
            attempt_id,
            provider_id,
            policy,
        });
    }

    fn allocate_attempt(&mut self) -> ProviderManagementAttemptId {
        let attempt_id = ProviderManagementAttemptId(self.next_attempt_id);
        self.next_attempt_id = self.next_attempt_id.saturating_add(1);
        attempt_id
    }

    fn transition(
        &self,
        applied: bool,
        effects: Vec<ProviderManagementEffect>,
        persistence_result: Option<ProviderManagementPersistenceResult>,
    ) -> ProviderManagementTransition {
        ProviderManagementTransition {
            phase: self.phase.clone(),
            statuses: self.statuses.clone(),
            effects,
            applied,
            persistence_result,
        }
    }
}

fn is_usable_replacement(status: &ProviderStatusSnapshot) -> bool {
    status.configuration == ProviderConfigurationState::Configured
        && status.eligibility == ProviderEligibilityState::Active
        && status.current == ProviderCurrentState::NotCurrent
        && status.availability == ProviderAvailabilityState::Ready
}

#[cfg(test)]
#[path = "management_tests.rs"]
mod tests;
