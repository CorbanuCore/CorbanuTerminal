//! PF-23-S03: confirmed security-level, revocation and kill-switch
//! transitions, owned by the trusted controller.
//!
//! ```text
//!            prepare(request, probes)            commit(store)
//!   current ------------------------> Prepared -------------> committed
//!      ^                                 |  stale epoch, refused merge,
//!      +------------- cancel ------------+  failed save: nothing changes
//! ```
//!
//! Only the trusted human-confirmation path holds the controller, and a
//! prepared transition is bound to the policy epoch it was confirmed under.
//! A commit merges into the stored state under the store's lock (other
//! sessions and processes on the same home may have committed since), makes
//! it durable ([`super::super::recovery`]), then swaps it in, advancing the
//! epoch and the revocation generation. Grants, pending post-taint approvals,
//! "for session" approval caches and child snapshots are bound to those, so
//! none survives. New protected work reads the policy under its lock, so it
//! is fenced the moment the commit returns; work already running is neither
//! cancelled nor relabeled.
//!
//! A stricter level, a revocation and the kill switch apply now, also to the
//! other policy trees of this process on the same home, and still apply when
//! the save fails (the result says they will not survive a restart). A
//! downgrade applies at the next start (the launch-time controls of this
//! process cannot be widened in place; a new session in this process is such
//! a start), while its revocation applies now. A downgrade or kill-switch
//! release that cannot be saved changes nothing.

use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::LazyLock;
use std::sync::Mutex;
use std::sync::PoisonError;
use std::sync::Weak;

use codex_protocol::security::SecurityControlAction;
use codex_security_policy::RevocationEvent;
use codex_security_policy::RevocationReason;
use codex_security_policy::RevocationState;
use codex_security_policy::RevocationTarget;
use codex_security_policy::SecurityLevel;
use thiserror::Error;

use super::EffectivePolicyState;
use super::SecurityPolicyError;
use super::SharedEffectivePolicy;
use super::TrustedSecurityController;
use super::trusted_requests::ConfirmedSecurityRequest;
use crate::security::recovery::DurableSecurityState;
use crate::security::recovery::TransitionStore;
use crate::security::recovery::TransitionWrite;

/// Result of the isolation, migration and screening checks (PF-29
/// preflight) a stricter protected level needs before it may be claimed.
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

    /// Whether the broker's channels close: every revocation except of one
    /// grant or mandate, and every level change.
    fn closes_channels(&self) -> bool {
        match &self.event {
            None
            | Some((RevocationTarget::Grant { .. } | RevocationTarget::Mandate { .. }, _))
            | Some((RevocationTarget::KillSwitch { active: false }, _)) => false,
            Some((
                RevocationTarget::KillSwitch { active: true }
                | RevocationTarget::Actor { .. }
                | RevocationTarget::AllActiveAuthority,
                _,
            )) => true,
        }
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
    /// Set when a restrictive transition applied but could not be saved: it
    /// holds until this process ends, not across a restart.
    pub(crate) not_saved: Option<String>,
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
    #[error(
        "another session stored a stricter level ({0}) since this one was shown; review it again"
    )]
    StoredLevelChanged(SecurityLevel),
    #[error("the kill switch was changed in another session since this was shown; review it again")]
    StoredStateChanged,
    #[error("the security state could not be saved, so nothing changed: {0}")]
    Persist(String),
    #[error(transparent)]
    Policy(#[from] SecurityPolicyError),
    #[error(transparent)]
    Revocation(#[from] codex_security_policy::RevocationError),
}

/// One commit at a time in this process; the store's lock orders processes.
static COMMITS: Mutex<()> = Mutex::new(());

/// Policy trees of this process by Corbanu home, so a restrictive commit in
/// one (`/new` starts another) reaches the rest.
static TREES: LazyLock<Mutex<Vec<HomeTree>>> = LazyLock::new(Default::default);

/// One policy tree and the Corbanu home it belongs to.
type HomeTree = (PathBuf, Weak<SharedEffectivePolicy>);

impl TrustedSecurityController {
    /// Bind a confirmed human request to the current epoch. A stricter
    /// protected level needs `probes` to have passed.
    pub(crate) fn prepare_transition(
        &self,
        confirmed: ConfirmedSecurityRequest,
        probes: ProbeOutcome,
    ) -> Result<PreparedTransition, TransitionError> {
        let (from, kill_switch_active) = {
            let guard = self.read_state()?;
            let state = guard
                .as_ref()
                .ok_or(SecurityPolicyError::RuntimeNotInitialized)?;
            super::trusted_requests::check_epoch(state, confirmed.request())?;
            (
                state.persisted.settings.level,
                state.persisted.revocations.kill_switch_active,
            )
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
                    // Only the kill switch this session shows as on can be
                    // released, never one it has not seen.
                    RevocationTarget::KillSwitch { active: false } if !kill_switch_active => {
                        return Err(TransitionError::StoredStateChanged);
                    }
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
        if to > from
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

    /// Merge into the stored state and save it, then apply. See the module
    /// documentation for what applies when.
    pub(crate) fn commit_transition(
        &self,
        prepared: PreparedTransition,
        store: &dyn TransitionStore,
        now_unix_seconds: i64,
    ) -> Result<CommittedTransition, TransitionError> {
        let _commit = COMMITS.lock().unwrap_or_else(PoisonError::into_inner);
        let (memory, authority) = {
            let guard = self.read_state()?;
            let state = guard
                .as_ref()
                .ok_or(SecurityPolicyError::RuntimeNotInitialized)?;
            super::trusted_requests::check_epoch(state, prepared.confirmed.request())?;
            if state.persisted.human_authority != *prepared.confirmed.authority() {
                return Err(SecurityPolicyError::AuthorityMismatch.into());
            }
            (
                state.persisted.revocations.clone(),
                state.persisted.human_authority.clone(),
            )
        };
        let event = prepared
            .event
            .clone()
            .map(|(target, reason)| {
                RevocationEvent::new(authority, target, reason, now_unix_seconds)
            })
            .transpose()?;
        // What this tree holds, what is stored, and this event: nothing either
        // side revoked comes back. A stored level is a floor; a revocation
        // never stores a level of its own (one raised by a repository's
        // config stays with that repository).
        let merge = |stored: Option<DurableSecurityState>| {
            let mut revocations = memory.clone();
            let stored_level = match &stored {
                Some(stored) => {
                    revocations.merge(&stored.revocations)?;
                    // A release is for the kill switch the human saw, not a
                    // newer one another session turned on.
                    if prepared.kind == TransitionKind::KillSwitchRelease
                        && revocations.kill_switch_event_id() != memory.kill_switch_event_id()
                    {
                        return Err(TransitionError::StoredStateChanged);
                    }
                    stored.level
                }
                None => SecurityLevel::Permissive,
            };
            if let Some(event) = &event {
                revocations.apply(event)?;
            }
            let level = match prepared.kind {
                TransitionKind::Restrictive if prepared.to == prepared.from => stored_level,
                TransitionKind::Restrictive | TransitionKind::Unchanged => {
                    stored_level.max(prepared.to)
                }
                // A level stored by another session after this one was shown
                // is not lowered without the human seeing it.
                TransitionKind::Downgrade if stored_level > prepared.from => {
                    return Err(TransitionError::StoredLevelChanged(stored_level));
                }
                TransitionKind::Downgrade => prepared.to,
                TransitionKind::KillSwitchRelease => stored_level,
            };
            Ok(DurableSecurityState::new(level, revocations))
        };
        let mut merged = None;
        let write = match prepared.kind {
            TransitionKind::Downgrade => TransitionWrite::Downgrade,
            TransitionKind::Unchanged => TransitionWrite::Level,
            TransitionKind::Restrictive if prepared.to != prepared.from => TransitionWrite::Level,
            TransitionKind::Restrictive | TransitionKind::KillSwitchRelease => {
                TransitionWrite::Revocation
            }
        };
        let saved = store.update(write, &mut |stored| {
            let next = merge(stored)?;
            merged = Some(next.clone());
            Ok(next)
        });
        let (next, not_saved) = match saved {
            Ok(next) => (next, None),
            Err(error @ TransitionError::Persist(_))
                if prepared.kind == TransitionKind::Restrictive =>
            {
                // An emergency stop never waits for the disk.
                let next = match merged {
                    Some(next) => next,
                    None => merge(None)?,
                };
                (next, Some(error.to_string()))
            }
            Err(error) => return Err(error),
        };
        let committed = self.apply(&prepared, next.clone(), not_saved)?;
        if prepared.kind == TransitionKind::Restrictive
            && let Some(home) = store.home()
        {
            propagate(&self.shared, home, &next, prepared.closes_channels());
        }
        if prepared.closes_channels() {
            self.notify_revocation_sinks();
        }
        tracing::info!(
            target: "codex_core::security::transition",
            kind = ?committed.kind,
            level = %committed.level,
            next_start_level = %committed.next_start_level,
            epoch = committed.epoch,
            revocation_generation = committed.revocation_generation,
            saved = committed.not_saved.is_none(),
            "security transition committed"
        );
        Ok(committed)
    }

    fn apply(
        &self,
        prepared: &PreparedTransition,
        next: DurableSecurityState,
        not_saved: Option<String>,
    ) -> Result<CommittedTransition, TransitionError> {
        let mut guard = self.write_state()?;
        let state = guard
            .as_mut()
            .ok_or(SecurityPolicyError::RuntimeNotInitialized)?;
        super::trusted_requests::check_epoch(state, prepared.confirmed.request())?;
        let in_force = match prepared.kind {
            TransitionKind::Restrictive => state.persisted.settings.level.max(prepared.to),
            TransitionKind::Downgrade
            | TransitionKind::KillSwitchRelease
            | TransitionKind::Unchanged => state.persisted.settings.level,
        };
        adopt(state, in_force, next.revocations)?;
        if let Some((RevocationTarget::Actor { actor_id }, _)) = &prepared.event {
            for binding in state.agents.values_mut() {
                if binding
                    .actor_chain
                    .as_slice()
                    .iter()
                    .any(|actor| actor.id == *actor_id)
                {
                    binding.force_deny = true;
                }
            }
        }
        // A revocation or kill-switch change leaves what the next start
        // enforces as it was.
        if prepared.to != prepared.from || prepared.kind == TransitionKind::Unchanged {
            state.next_start_level = next.level;
        }
        Ok(CommittedTransition {
            kind: prepared.kind,
            epoch: state.epoch,
            revocation_generation: state.persisted.revocations.generation,
            level: in_force,
            next_start_level: state.next_start_level,
            kill_switch_active: state.persisted.revocations.kill_switch_active,
            not_saved,
        })
    }

    /// A session that read the stored state just before another tree's
    /// commit was saved and propagated catches up here, after it registered
    /// its home: the stricter level and every stored revocation apply.
    pub(crate) fn catch_up(&self, stored: &crate::security::recovery::Recovery) {
        if stored.unreadable.is_some() {
            return;
        }
        let Ok(mut guard) = self.write_state() else {
            return;
        };
        let Some(state) = guard.as_mut() else {
            return;
        };
        let level = state.persisted.settings.level.max(stored.level);
        let mut revocations = state.persisted.revocations.clone();
        if revocations.merge(&stored.revocations).is_err()
            || (level == state.persisted.settings.level
                && revocations == state.persisted.revocations)
        {
            return;
        }
        if adopt(state, level, revocations).is_ok() {
            state.next_start_level = state.next_start_level.max(stored.level);
        }
    }

    /// Run end: every sink revokes, whatever the level.
    pub(crate) fn notify_revocation_sinks(&self) {
        notify(&self.shared);
    }
}

/// Swap in a level and revocation state: the epoch moves and every grant of
/// the tree is dropped.
fn adopt(
    state: &mut EffectivePolicyState,
    level: SecurityLevel,
    revocations: RevocationState,
) -> Result<(), TransitionError> {
    let next = super::PersistedHumanSecurityState::new(
        codex_security_policy::SecuritySettings::new(level),
        state.persisted.human_authority.clone(),
        revocations,
    )?;
    state.epoch = state
        .epoch
        .checked_add(1)
        .ok_or(SecurityPolicyError::EpochOverflow)?;
    state.persisted = next;
    for thread in state.agents.keys() {
        crate::security::aggressive::revoke_all(*thread);
    }
    Ok(())
}

/// A restrictive commit reaches the other trees of this process on `home`:
/// their level rises to it, they take the merged revocations (the kill
/// switch included) when those are newer, and their sinks revoke.
fn propagate(
    origin: &Arc<SharedEffectivePolicy>,
    home: &Path,
    next: &DurableSecurityState,
    closes_channels: bool,
) {
    let home = canonical(home);
    let trees: Vec<Arc<SharedEffectivePolicy>> = {
        let mut trees = TREES.lock().unwrap_or_else(PoisonError::into_inner);
        trees.retain(|(_, tree)| tree.strong_count() > 0);
        trees
            .iter()
            .filter(|(tree_home, _)| *tree_home == home)
            .filter_map(|(_, tree)| tree.upgrade())
            .filter(|tree| !Arc::ptr_eq(tree, origin))
            .collect()
    };
    for tree in trees {
        {
            let mut guard = tree.state.write().unwrap_or_else(PoisonError::into_inner);
            let Some(state) = guard.as_mut() else {
                continue;
            };
            let level = state.persisted.settings.level.max(next.level);
            let mut revocations = state.persisted.revocations.clone();
            let merged = revocations
                .merge(&next.revocations)
                .map_err(TransitionError::from);
            if let Err(error) = merged.and_then(|()| adopt(state, level, revocations)) {
                tracing::warn!(
                    target: "codex_core::security::transition",
                    %error,
                    "could not apply a restrictive transition to another session"
                );
                continue;
            }
            state.next_start_level = state.next_start_level.max(next.level);
        }
        if closes_channels {
            notify(&tree);
        }
    }
}

impl super::EffectivePolicyView {
    /// The tree-wide policy epoch and revocation generation; `None` before
    /// the policy is initialized. "For session" approval caches are bound to
    /// it.
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

    /// Record which Corbanu home this tree belongs to, so restrictive
    /// commits of other trees on it reach this one.
    pub(crate) fn register_home(&self, home: &Path) {
        let home = canonical(home);
        let mut trees = TREES.lock().unwrap_or_else(PoisonError::into_inner);
        trees.retain(|(_, tree)| tree.strong_count() > 0);
        if !trees
            .iter()
            .any(|(_, tree)| std::ptr::eq(tree.as_ptr(), Arc::as_ptr(&self.shared)))
        {
            trees.push((home, Arc::downgrade(&self.shared)));
        }
    }
}

/// The production trigger the isolated broker had none of (PF-27-S04).
/// It also drops the hosts approved "for session".
impl RevocationSink for crate::session::session::Session {
    fn revoke(&self) {
        self.services
            .network_approval
            .forget_session_approved_hosts();
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

fn canonical(home: &Path) -> PathBuf {
    std::fs::canonicalize(home).unwrap_or_else(|_| home.to_path_buf())
}

fn notify(shared: &SharedEffectivePolicy) {
    let live: Vec<Arc<dyn RevocationSink>> = shared
        .sinks
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
