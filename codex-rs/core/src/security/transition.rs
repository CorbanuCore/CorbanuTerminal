//! PF-23-S03: confirmed security-level, revocation and kill-switch
//! transitions, owned by the trusted controller.
//!
//! ```text
//!            prepare(request, probes)            commit(store)
//!   current ------------------------> Prepared -------------> committed
//!      ^                                 |  persist fails, stale epoch:
//!      +------------- cancel ------------+  nothing changes
//! ```
//!
//! Only the trusted human-confirmation path holds the controller, and a
//! prepared transition is bound to the policy epoch it was confirmed under:
//! any other change in between makes it stale. A commit makes the new state
//! durable first ([`super::super::recovery`]), then swaps it in under the
//! policy write lock, advancing the epoch and the revocation generation. Every
//! cached decision, grant, pending approval and child snapshot is bound to
//! those, so none survives. Protected work reads the policy under the same
//! lock, so new work is fenced the moment the commit returns; work already
//! running is not relabeled as cancelled.
//!
//! A restrictive change applies now. A downgrade applies at the next start
//! (the launch-time controls of this process cannot be widened in place),
//! while its revocation still applies now.

use std::sync::Arc;
use std::sync::PoisonError;
use std::sync::Weak;

use codex_protocol::security::SecurityControlAction;
use codex_security_policy::RevocationEvent;
use codex_security_policy::RevocationReason;
use codex_security_policy::RevocationTarget;
use codex_security_policy::SecurityLevel;
use codex_security_policy::SecuritySettings;
use thiserror::Error;

use super::PersistedHumanSecurityState;
use super::SecurityPolicyError;
use super::TrustedSecurityController;
use super::trusted_requests::ConfirmedSecurityRequest;
use crate::security::recovery::DurableSecurityState;
use crate::security::recovery::TransitionStore;

/// Result of the isolation, migration and screening checks (PF-29
/// preflight) a protected level needs before it may be claimed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ProbeOutcome {
    Passed,
    Blocked(Vec<String>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TransitionKind {
    /// A stricter level, a revocation or the kill switch: applies now.
    Restrictive,
    /// A lower level: saved now, enforced from the next start.
    Downgrade,
    /// The kill switch is turned off; the level is unchanged.
    KillSwitchRelease,
    /// The level in force is chosen again (this also undoes a pending
    /// downgrade).
    Unchanged,
}

/// A confirmed request waiting for commit or cancel. Not cloneable, and
/// stale once the policy epoch moves.
pub(crate) struct PreparedTransition {
    confirmed: ConfirmedSecurityRequest,
    kind: TransitionKind,
    from: SecurityLevel,
    to: SecurityLevel,
    event: Option<(RevocationTarget, RevocationReason)>,
}

impl PreparedTransition {
    pub(crate) fn kind(&self) -> TransitionKind {
        self.kind
    }

    /// Level in force now and the level the commit would select.
    pub(crate) fn levels(&self) -> (SecurityLevel, SecurityLevel) {
        (self.from, self.to)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CommittedTransition {
    pub(crate) kind: TransitionKind,
    pub(crate) epoch: u64,
    pub(crate) revocation_generation: u64,
    /// Level in force now.
    pub(crate) level: SecurityLevel,
    pub(crate) next_start_level: SecurityLevel,
    pub(crate) kill_switch_active: bool,
}

/// Told after a committed restrictive transition, kill switch or run end,
/// e.g. the isolated credential broker revoking its references and closing
/// its open channels.
pub(crate) trait RevocationSink: Send + Sync {
    fn revoke(&self);
}

#[derive(Debug, Error)]
pub(crate) enum TransitionError {
    #[error("this level cannot be enforced until these checks pass: {}", .0.join("; "))]
    Blocked(Vec<String>),
    #[error("grants are issued in the grant screen, not as a transition")]
    NotATransition,
    #[error("the security state could not be saved, so nothing changed: {0}")]
    Persist(String),
    #[error(transparent)]
    Policy(#[from] SecurityPolicyError),
    #[error(transparent)]
    Revocation(#[from] codex_security_policy::RevocationError),
}

impl TrustedSecurityController {
    /// Bind a confirmed human request to the current epoch. A stricter
    /// protected level needs `probes` to have passed.
    pub(crate) fn prepare_transition(
        &self,
        confirmed: ConfirmedSecurityRequest,
        probes: ProbeOutcome,
    ) -> Result<PreparedTransition, TransitionError> {
        let from = {
            let guard = self.read_state()?;
            let state = guard
                .as_ref()
                .ok_or(SecurityPolicyError::RuntimeNotInitialized)?;
            super::trusted_requests::check_epoch(state, confirmed.request())?;
            state.persisted.settings.level
        };
        let (kind, to, event) = match confirmed.request().action() {
            SecurityControlAction::SetLevel { level } => {
                let kind = match level.cmp(&from) {
                    std::cmp::Ordering::Greater => TransitionKind::Restrictive,
                    std::cmp::Ordering::Less => TransitionKind::Downgrade,
                    std::cmp::Ordering::Equal => TransitionKind::Unchanged,
                };
                let event = (kind != TransitionKind::Unchanged).then_some((
                    RevocationTarget::AllActiveAuthority,
                    RevocationReason::SecurityLevelChange,
                ));
                (kind, *level, event)
            }
            SecurityControlAction::Revoke { target, reason } => {
                let kind = match target {
                    RevocationTarget::KillSwitch { active: false } => {
                        TransitionKind::KillSwitchRelease
                    }
                    RevocationTarget::KillSwitch { active: true }
                    | RevocationTarget::Grant { .. }
                    | RevocationTarget::Mandate { .. }
                    | RevocationTarget::Actor { .. }
                    | RevocationTarget::AllActiveAuthority => TransitionKind::Restrictive,
                };
                (kind, from, Some((target.clone(), *reason)))
            }
            SecurityControlAction::CreateGrant { .. } => {
                return Err(TransitionError::NotATransition);
            }
        };
        if to != from
            && let ProbeOutcome::Blocked(blockers) = probes
        {
            return Err(TransitionError::Blocked(blockers));
        }
        Ok(PreparedTransition {
            confirmed,
            kind,
            from,
            to,
            event,
        })
    }

    /// Esc: nothing was written and nothing changes.
    pub(crate) fn cancel_transition(&self, prepared: PreparedTransition) {
        drop(prepared);
    }

    /// Persist, then apply in one critical section. On any error the state
    /// in force, the epoch and the stored state are as before (a failed
    /// `config.toml` write after the state file can only leave the next start
    /// stricter).
    pub(crate) fn commit_transition(
        &self,
        prepared: PreparedTransition,
        store: &dyn TransitionStore,
        now_unix_seconds: i64,
    ) -> Result<CommittedTransition, TransitionError> {
        let PreparedTransition {
            confirmed,
            kind,
            from,
            to,
            event,
        } = prepared;
        // A single grant or mandate leaves the broker's channels open.
        let closes_channels = kind == TransitionKind::Restrictive
            && (to != from
                || !matches!(
                    event,
                    Some((
                        RevocationTarget::Grant { .. } | RevocationTarget::Mandate { .. },
                        _
                    ))
                ));
        let committed = {
            let mut guard = self.write_state()?;
            let state = guard
                .as_mut()
                .ok_or(SecurityPolicyError::RuntimeNotInitialized)?;
            super::trusted_requests::check_epoch(state, confirmed.request())?;
            if state.persisted.human_authority != *confirmed.authority() {
                return Err(SecurityPolicyError::AuthorityMismatch.into());
            }
            let mut revocations = state.persisted.revocations.clone();
            if let Some((target, reason)) = event {
                revocations.apply(&RevocationEvent::new(
                    state.persisted.human_authority.clone(),
                    target,
                    reason,
                    now_unix_seconds,
                )?)?;
            }
            let in_force = match kind {
                TransitionKind::Restrictive => to,
                TransitionKind::Downgrade
                | TransitionKind::KillSwitchRelease
                | TransitionKind::Unchanged => state.persisted.settings.level,
            };
            let next_start = match kind {
                TransitionKind::KillSwitchRelease => state.next_start_level,
                TransitionKind::Restrictive
                | TransitionKind::Downgrade
                | TransitionKind::Unchanged => to,
            };
            let next = PersistedHumanSecurityState::new(
                SecuritySettings::new(in_force),
                state.persisted.human_authority.clone(),
                revocations,
            )?;
            let next_epoch = state
                .epoch
                .checked_add(1)
                .ok_or(SecurityPolicyError::EpochOverflow)?;
            store
                .persist(&DurableSecurityState::new(
                    next_start,
                    next.revocations.clone(),
                ))
                .map_err(|err| TransitionError::Persist(err.to_string()))?;
            state.persisted = next;
            state.next_start_level = next_start;
            state.epoch = next_epoch;
            // Grants are bound to the epoch and generation that just moved;
            // drop them from the ledger of every agent in this tree as well.
            for thread in state.agents.keys() {
                crate::security::aggressive::revoke_all(*thread);
            }
            CommittedTransition {
                kind,
                epoch: next_epoch,
                revocation_generation: state.persisted.revocations.generation,
                level: in_force,
                next_start_level: next_start,
                kill_switch_active: state.persisted.revocations.kill_switch_active,
            }
        };
        if closes_channels {
            self.notify_revocation_sinks();
        }
        tracing::info!(
            target: "codex_core::security::transition",
            kind = ?committed.kind,
            level = %committed.level,
            next_start_level = %committed.next_start_level,
            epoch = committed.epoch,
            revocation_generation = committed.revocation_generation,
            "security transition committed"
        );
        Ok(committed)
    }

    /// Run end: every sink revokes, whatever the level.
    pub(crate) fn notify_revocation_sinks(&self) {
        notify(&self.shared.sinks);
    }
}

impl super::EffectivePolicyView {
    /// The tree-wide policy epoch and revocation generation; `None` before
    /// the policy is initialized. Cached decisions are bound to it.
    pub(crate) fn authority_marker(&self) -> Option<(u64, u64)> {
        let guard = self.read_state().ok()?;
        let state = guard.as_ref()?;
        Some((state.epoch, state.persisted.revocations.generation))
    }

    /// Register a sink for restrictive transitions, the kill switch and run
    /// end. Held weakly: a dropped sink is skipped.
    pub(crate) fn register_revocation_sink(&self, sink: Weak<dyn RevocationSink>) {
        let mut sinks = self
            .shared
            .sinks
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        sinks.retain(|sink| sink.strong_count() > 0);
        sinks.push(sink);
    }
}

/// The production trigger the isolated broker had none of (PF-27-S04).
impl RevocationSink for crate::session::session::Session {
    fn revoke(&self) {
        if let Some(proxy) = self.services.network_proxy.load_full() {
            let revoked = proxy.revoke_brokered_credentials();
            tracing::info!(
                target: "codex_core::security::transition",
                revoked,
                "brokered credentials revoked"
            );
        }
    }
}

fn notify(sinks: &std::sync::Mutex<Vec<Weak<dyn RevocationSink>>>) {
    let live: Vec<Arc<dyn RevocationSink>> = sinks
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .iter()
        .filter_map(Weak::upgrade)
        .collect();
    for sink in live {
        sink.revoke();
    }
}

#[cfg(test)]
#[path = "transition_tests.rs"]
mod tests;
