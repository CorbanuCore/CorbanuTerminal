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
    account: Option<NamedCredentialAccount>,
}

/// PF-84: the named account (not the provider's default credential) a request used.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NamedCredentialAccount {
    /// The runtime provider id the account belongs to.
    pub provider_id: String,
    pub name: String,
}

/// A rejected attempt: the catalog provider and, for a named account, which one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RejectedCredential {
    pub provider: String,
    pub account: Option<NamedCredentialAccount>,
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
        self.begin_for_account(scope, status, /*account*/ None);
    }

    /// Like [`Self::begin`]; a request on a named account is tracked apart from
    /// the default credential, whose health it never changes.
    pub fn begin_for_account(
        &mut self,
        scope: String,
        status: &ProviderStatusSnapshot,
        account: Option<NamedCredentialAccount>,
    ) {
        // Do not overwrite an outstanding rejection with its own projected status.
        // Recovery (or a new source) must precede another validation attempt.
        if account.is_none() && status.configuration == ProviderConfigurationState::RecoveryRequired
        {
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
                account,
            },
        );
    }

    /// Reject only a request made with the still-current credential revision.
    pub fn reject(&mut self, scope: &str) -> bool {
        self.reject_provider(scope).is_some()
    }

    /// Return the captured provider, never the selection at notification time.
    pub fn reject_provider(&mut self, scope: &str) -> Option<String> {
        self.reject_credential(scope)
            .map(|rejected| rejected.provider)
    }

    /// Like [`Self::reject_provider`], naming the account. A named account's
    /// rejection is reported but leaves the default credential's health alone.
    pub fn reject_credential(&mut self, scope: &str) -> Option<RejectedCredential> {
        let attempt = self.attempts.remove(scope)?;
        if let Some(account) = attempt.account {
            return Some(RejectedCredential {
                provider: attempt.provider,
                account: Some(account),
            });
        }
        if self.revisions.get(&attempt.provider) != Some(&attempt.revision) {
            return None;
        }
        let provider = attempt.provider.clone();
        self.rejected.insert(attempt.provider.clone(), attempt);
        Some(RejectedCredential {
            provider,
            account: None,
        })
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
            .retain(|_, attempt| attempt.provider != provider || attempt.account.is_some());
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
