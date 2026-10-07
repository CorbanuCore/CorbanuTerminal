//! PF-25-S02: the human revocation and kill switch confirmed in `/security`.
//!
//! The only caller is the trusted TUI, after a person pressed the confirm key
//! on a review screen, as for [`super::level_change`]: no `Op`, app-server
//! method, tool or model output reaches this module.
//!
//! - One grant: it ends now ([`revoke_grant`]). Grants live in memory only,
//!   so there is nothing to save.
//! - All active authority and the kill switch go through PF-23-S03's
//!   transition: they apply now to the session's policy tree and the other
//!   trees of this process on the home (grants, "for session" approvals,
//!   broker channels end), and are saved in `security_state.json`, so revoked
//!   authority never comes back and the kill switch holds after a restart.
//!   Only the kill switch the person saw can be turned off, and turning it
//!   off keeps the level (`TransitionKind::KillSwitchRelease`).
//!
//! The commit blocks for up to the store's lock wait (2 s); call it off the
//! async runtime.

use std::path::Path;

use codex_protocol::ThreadId;
use codex_protocol::security::SecurityControlAction;
use codex_protocol::security::SecurityControlRequest;
use codex_security_policy::RevocationReason;
use codex_security_policy::RevocationTarget;
pub use codex_security_policy::SecurityLevel;

use super::level_change::LevelBasis;
use super::level_change::LevelChangeError;
use super::level_change::stored_controller;
use super::recovery::HomeTransitionStore;
use super::transition::ProbeOutcome;
use super::transition::live_controller;

/// What the person confirmed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HumanRevocation {
    /// Every grant, "for session" approval and broker channel ends now.
    AllActiveAuthority,
    /// Turn the kill switch on.
    KillSwitchOn,
    /// Turn off the kill switch the person saw; the level stays.
    KillSwitchOff,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RevocationReport {
    /// Level in force in this process's sessions now (unchanged).
    pub in_force: SecurityLevel,
    pub kill_switch_active: bool,
    /// Whether running sessions of this process took it.
    pub live: bool,
    /// It applied now but could not be saved: it holds until this process
    /// ends, not across a restart.
    pub not_saved: Option<String>,
}

/// Commit `choice`. `reviewed` is the basis the review showed; `configured`
/// and `thread` as for [`LevelBasis::read`].
pub fn commit_human_revocation(
    codex_home: &Path,
    configured: SecurityLevel,
    thread: Option<ThreadId>,
    reviewed: &LevelBasis,
    choice: HumanRevocation,
    now_unix_seconds: i64,
) -> Result<RevocationReport, LevelChangeError> {
    if LevelBasis::read(codex_home, configured, thread) != *reviewed {
        return Err(LevelChangeError::Changed);
    }
    let (target, reason) = match choice {
        HumanRevocation::AllActiveAuthority => (
            RevocationTarget::AllActiveAuthority,
            RevocationReason::HumanRequest,
        ),
        HumanRevocation::KillSwitchOn => (
            RevocationTarget::KillSwitch { active: true },
            RevocationReason::KillSwitch,
        ),
        HumanRevocation::KillSwitchOff => (
            RevocationTarget::KillSwitch { active: false },
            RevocationReason::HumanRequest,
        ),
    };
    let (controller, live) = match thread.and_then(|thread| live_controller(codex_home, thread)) {
        Some(controller) => (controller, true),
        None => (stored_controller(codex_home, configured)?, false),
    };
    let epoch = match reviewed.epoch() {
        Some(epoch) if live => epoch,
        Some(_) | None => controller.authority_epoch()?,
    };
    let request =
        SecurityControlRequest::new(epoch, SecurityControlAction::Revoke { target, reason })
            .map_err(|error| LevelChangeError::Refused(error.to_string()))?;
    let confirmed = controller.confirm_security_request(request, now_unix_seconds)?;
    let prepared = controller.prepare_transition(confirmed, ProbeOutcome::Passed)?;
    let committed = controller.commit_transition(
        prepared,
        &HomeTransitionStore::new(codex_home),
        now_unix_seconds,
    )?;
    Ok(RevocationReport {
        in_force: committed.level,
        kill_switch_active: committed.kill_switch_active,
        live,
        not_saved: committed.not_saved,
    })
}

/// End the grant `grant_id` now. Returns whether one was held.
pub fn revoke_grant(grant_id: &str) -> bool {
    super::aggressive::revoke_grant(grant_id)
}

#[cfg(test)]
#[path = "revocation_change_tests.rs"]
mod tests;
