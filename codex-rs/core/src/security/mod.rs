// PF-13-S02 connects this staged trusted boundary to vault resolution.
#[allow(dead_code)]
pub(crate) mod credential_capability;
mod effective_policy;

pub(crate) use effective_policy::EffectivePolicyView;
pub(crate) use effective_policy::PersistedHumanSecurityState;
pub(crate) use effective_policy::SecurityPolicyError;
pub(crate) use effective_policy::TrustedSecurityController;

#[cfg(test)]
#[path = "effective_policy_tests.rs"]
mod effective_policy_tests;
