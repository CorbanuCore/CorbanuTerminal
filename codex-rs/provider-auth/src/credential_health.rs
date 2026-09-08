//! Session-local observations bound to a provider's credential revision.
//! No secrets or response text are retained. Storage presence is not proof of health.

use std::collections::BTreeMap;

use crate::CredentialControl;
use crate::ProviderAvailabilityState;
use crate::ProviderConfigurationState;
use crate::ProviderCredentialSource;
use crate::ProviderMethodState;
use crate::ProviderMethodStatus;
use crate::ProviderRecoveryReason;
use crate::ProviderStatusCatalog;
use crate::ProviderStatusSnapshot;
use crate::ProviderUnavailableReason;

#[derive(Clone, Debug)]
struct Attempt {
    provider: String,
    revision: u64,
    methods: Vec<ProviderMethodStatus>,
}

/// Bounded, secret-free ledger for typed authentication rejections. Callers must
/// begin an attempt before sending a request and invalidate after credential changes.
#[derive(Clone, Debug, Default)]
pub struct ProviderCredentialHealth {
    revisions: BTreeMap<String, u64>,
    attempts: BTreeMap<String, Attempt>,
    rejected: BTreeMap<String, Attempt>,
}

impl ProviderCredentialHealth {
    pub fn begin(&mut self, scope: String, status: &ProviderStatusSnapshot) {
        // Do not overwrite an outstanding rejection with its own projected status.
        // Recovery (or a new source) must precede another validation attempt.
        if status.configuration == ProviderConfigurationState::RecoveryRequired {
            return;
        }
        let provider = status.id.as_str().to_string();
        let revision = *self.revisions.entry(provider.clone()).or_default();
        if self.attempts.len() >= 64 {
            self.attempts.pop_first();
        }
        self.attempts.insert(
            scope,
            Attempt {
                provider,
                revision,
                methods: status.methods.clone(),
            },
        );
    }

    /// Reject only a request made with the still-current credential revision.
    pub fn reject(&mut self, scope: &str) -> bool {
        let Some(attempt) = self.attempts.remove(scope) else {
            return false;
        };
        if self.revisions.get(&attempt.provider) != Some(&attempt.revision) {
            return false;
        }
        self.rejected.insert(attempt.provider.clone(), attempt);
        true
    }

    pub fn finish(&mut self, scope: &str) {
        self.attempts.remove(scope);
    }

    /// A matching successful credential replacement permits a new validation attempt.
    /// It does not claim the provider has accepted the newly stored credential.
    pub fn credential_changed(&mut self, provider: &str) {
        let revision = self.revisions.entry(provider.to_string()).or_default();
        *revision = revision.wrapping_add(1);
        self.rejected.remove(provider);
        self.attempts
            .retain(|_, attempt| attempt.provider != provider);
    }

    pub fn apply(&self, statuses: &mut ProviderStatusCatalog) {
        for status in &mut statuses.entries {
            let Some(rejected) = self.rejected.get(status.id.as_str()) else {
                continue;
            };
            if rejected.methods != status.methods {
                continue;
            }
            let mut changed = false;
            for method in &mut status.methods {
                if let ProviderMethodState::Configured {
                    source, control, ..
                } = method.state
                    && source != ProviderCredentialSource::Local
                    && control != CredentialControl::None
                {
                    method.state = ProviderMethodState::RecoveryRequired {
                        reason: ProviderRecoveryReason::CredentialRejected { source, control },
                    };
                    changed = true;
                }
            }
            if changed {
                status.configuration = ProviderConfigurationState::RecoveryRequired;
                status.availability = ProviderAvailabilityState::Unavailable {
                    reason: ProviderUnavailableReason::RecoveryRequired,
                };
                // Preserve activation and current selection; neither is auth health.
            }
        }
    }
}

#[cfg(test)]
#[path = "credential_health_tests.rs"]
mod tests;
