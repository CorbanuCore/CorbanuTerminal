//! PF-24-S02: the human `/security` confirmation's route into the PF-23-S03
//! transition (`effective_policy::transition`).
//!
//! The only caller is the trusted TUI, after a person pressed the confirm key
//! on the review screen. No `Op`, app-server method, tool or model output
//! reaches this module: a request cannot arrive over the wire, and the
//! controller it uses is never handed to agent runtimes.
//!
//! A commit goes through the live policy tree of this process on the Corbanu
//! home (the strictest one), so a stricter level and its revocation apply to
//! running sessions now and reach the other trees on that home. Without a
//! live tree it goes through a tree built from the stored state, which only
//! saves. Either way the result is durable in `security_state.json` before it
//! is reported, except a stricter level whose save failed (it applies now and
//! says it will not survive a restart).
//!
//! The commit blocks for up to the store's lock wait (2 s); call it off the
//! async runtime.

use std::path::Path;

use codex_config::ConfigLayerSource;
use codex_protocol::SessionId;
use codex_protocol::ThreadId;
use codex_protocol::security::SecurityControlAction;
use codex_protocol::security::SecurityControlRequest;
use codex_security_policy::PolicyPrincipal;
use codex_security_policy::PrincipalKind;
pub use codex_security_policy::SecurityLevel;
use codex_security_policy::SecuritySettings;
use thiserror::Error;

use super::EffectivePolicyInitialization;
use super::EffectivePolicyView;
use super::PersistedHumanSecurityState;
use super::TrustedSecurityController;
use super::recovery;
use super::recovery::HomeTransitionStore;
use super::transition::ProbeOutcome;
use super::transition::TransitionError;
use super::transition::TransitionKind;
use super::transition::live_controller;
use crate::config::Config;

/// `security_state.json` as the next start reads it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StoredSecurityState {
    Absent,
    Level(SecurityLevel),
    /// Present but unreadable: the next start enforces Aggressive and the
    /// kill switch.
    Unreadable(String),
}

impl StoredSecurityState {
    pub fn load(codex_home: &Path) -> Self {
        match recovery::load(codex_home) {
            Ok(None) => Self::Absent,
            Ok(Some(state)) => Self::Level(state.level),
            Err(recovery::LoadError::Io(reason) | recovery::LoadError::Corrupt(reason)) => {
                Self::Unreadable(reason)
            }
        }
    }
}

/// What the person reviewed. A confirmation commits only while it is still
/// true, so nothing they did not see is changed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LevelBasis {
    pub in_force: SecurityLevel,
    pub next_start: SecurityLevel,
    pub kill_switch_active: bool,
    pub stored: StoredSecurityState,
}

impl LevelBasis {
    /// `configured` is [`configured_level`] of the session's config.
    pub fn read(codex_home: &Path, configured: SecurityLevel) -> Self {
        let stored = StoredSecurityState::load(codex_home);
        if let Some(Ok((in_force, next_start, kill_switch_active))) =
            live_controller(codex_home).map(|controller| controller.in_force())
        {
            return Self {
                in_force,
                next_start,
                kill_switch_active,
                stored,
            };
        }
        let recovered = recovery::recover(codex_home, configured);
        Self {
            in_force: recovered.level,
            next_start: recovered.level,
            kill_switch_active: recovered.unreadable.is_some()
                || recovered.revocations.kill_switch_active,
            stored,
        }
    }
}

/// The strictest `[security] level` any config layer sets, without the
/// stored state.
pub fn configured_level(config: &Config) -> SecurityLevel {
    crate::config::layered_security_floor(&config.config_layer_stack)
}

/// The strictest level a layer other than the user's own `config.toml` sets
/// (a repository, a profile, `-c`, managed configuration). A downgrade below
/// it is saved but the next start still enforces it.
pub fn non_user_level_floor(config: &Config) -> SecurityLevel {
    config
        .config_layer_stack
        .layers_high_to_low()
        .into_iter()
        .filter(|layer| !matches!(layer.name, ConfigLayerSource::User { .. }))
        .filter_map(|layer| {
            layer
                .config
                .get("security")?
                .get("level")?
                .clone()
                .try_into::<SecurityLevel>()
                .ok()
        })
        .max()
        .unwrap_or_default()
}

/// The PF-29 preflight for a stricter level: passed, or why not.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Probes {
    Passed,
    Blocked(Vec<String>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LevelChangeKind {
    /// Applies now.
    Stricter,
    /// Saved now; enforced from the next start.
    Downgrade,
    /// The level in force is chosen again; a pending downgrade is undone.
    Unchanged,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LevelChangeReport {
    pub kind: LevelChangeKind,
    /// Level in force in this process's sessions now.
    pub in_force: SecurityLevel,
    pub next_start: SecurityLevel,
    /// Whether running sessions of this process took it.
    pub live: bool,
    /// A stricter level applied now but could not be saved.
    pub not_saved: Option<String>,
}

#[derive(Debug, Error)]
pub enum LevelChangeError {
    #[error("the security state changed since you reviewed it; review it again")]
    Changed,
    #[error("{0}")]
    Refused(String),
    /// Nothing was saved and nothing changed.
    #[error("{0}")]
    NotSaved(String),
}

impl From<TransitionError> for LevelChangeError {
    fn from(error: TransitionError) -> Self {
        match error {
            TransitionError::StoredLevelChanged(_) | TransitionError::StoredStateChanged => {
                Self::Changed
            }
            TransitionError::Persist(_) => Self::NotSaved(error.to_string()),
            TransitionError::Blocked(_)
            | TransitionError::NotATransition
            | TransitionError::Policy(_)
            | TransitionError::Revocation(_) => Self::Refused(error.to_string()),
        }
    }
}

impl From<super::SecurityPolicyError> for LevelChangeError {
    fn from(error: super::SecurityPolicyError) -> Self {
        Self::Refused(error.to_string())
    }
}

/// Commit the level a person confirmed in `/security`. `reviewed` is the
/// basis shown on the review screen; `configured` as for [`LevelBasis::read`].
pub fn commit_human_level_change(
    codex_home: &Path,
    configured: SecurityLevel,
    reviewed: &LevelBasis,
    target: SecurityLevel,
    probes: Probes,
    now_unix_seconds: i64,
) -> Result<LevelChangeReport, LevelChangeError> {
    if LevelBasis::read(codex_home, configured) != *reviewed {
        return Err(LevelChangeError::Changed);
    }
    let (controller, live) = match live_controller(codex_home) {
        Some(controller) => (controller, true),
        None => (stored_controller(codex_home, configured)?, false),
    };
    let request = SecurityControlRequest::new(
        controller.authority_epoch()?,
        SecurityControlAction::SetLevel { level: target },
    )
    .map_err(|error| LevelChangeError::Refused(error.to_string()))?;
    let confirmed = controller.confirm_security_request(request, now_unix_seconds)?;
    let prepared = controller.prepare_transition(
        confirmed,
        match probes {
            Probes::Passed => ProbeOutcome::Passed,
            Probes::Blocked(blockers) => ProbeOutcome::Blocked(blockers),
        },
    )?;
    let committed = controller.commit_transition(
        prepared,
        &HomeTransitionStore::new(codex_home),
        now_unix_seconds,
    )?;
    Ok(LevelChangeReport {
        kind: match committed.kind {
            TransitionKind::Restrictive => LevelChangeKind::Stricter,
            TransitionKind::Downgrade => LevelChangeKind::Downgrade,
            TransitionKind::KillSwitchRelease | TransitionKind::Unchanged => {
                LevelChangeKind::Unchanged
            }
        },
        in_force: committed.level,
        next_start: committed.next_start_level,
        live,
        not_saved: committed.not_saved,
    })
}

/// A policy tree built from the stored state, for a commit made while no
/// session of this process uses the home. It saves; nothing else holds it.
fn stored_controller(
    codex_home: &Path,
    configured: SecurityLevel,
) -> Result<TrustedSecurityController, LevelChangeError> {
    let recovered = recovery::recover(codex_home, configured);
    let root = ThreadId::new();
    let persisted = PersistedHumanSecurityState::new(
        SecuritySettings::new(recovered.level),
        PolicyPrincipal::new(PrincipalKind::Human, "human:security-picker")
            .map_err(|error| LevelChangeError::Refused(error.to_string()))?,
        recovered.revocations,
    )?;
    Ok(TrustedSecurityController::initialize(
        &EffectivePolicyView::default(),
        persisted,
        root,
        SessionId::from(root),
        EffectivePolicyInitialization::Root,
    )?)
}

#[cfg(test)]
#[path = "level_change_tests.rs"]
mod tests;
