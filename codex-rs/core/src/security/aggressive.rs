//! PF-23-S02: Aggressive grants, the one enforcement point for opening a
//! sensitive surface under Aggressive.
//!
//! Under Aggressive the sandbox's protected-path rules (PF-23-S01 slice 3)
//! apply from the start of the session, and a human approval never lifts
//! them. Only a [`BoundedGrant`] can, and only when it matches exactly:
//! the agent lineage it was issued to, one known surface, one exact
//! operation (a command digest or one process), the session, its expiry and
//! its use limit, under the policy epoch it was issued in. A kill switch, a
//! level change, a revocation or a restart (grants are never stored) ends it.
//! Grants are never inherited: a child agent has its own lineage and ledger,
//! and a grant for it must be derived narrower ([`BoundedGrant::derive_child`]).
//!
//! Issuing is host-only (the grant TUI, PF-25-S01); nothing a model can call
//! reaches [`issue_labelled`].

use crate::security::tainted_action::PolicyBinding;
use crate::security::tainted_action::PostTaintState;
use codex_protocol::ThreadId;
use codex_security_policy::ActorChain;
use codex_security_policy::AuthorizationContext;
use codex_security_policy::AuthorizationRequest;
use codex_security_policy::BoundedGrant;
use codex_security_policy::BoundedText;
use codex_security_policy::PolicyAction;
use codex_security_policy::ProtectedResource;
use codex_security_policy::QuantitativeLimit;
use codex_security_policy::ResourceKind;
use codex_security_policy::SecurityLevel;
use sha2::Digest;
use sha2::Sha256;
use std::collections::HashMap;
use std::sync::LazyLock;
use std::sync::Mutex;
use std::sync::PoisonError;

/// The purpose every Aggressive grant names.
pub(crate) const PURPOSE: &str = "aggressive_grant";
/// The quantitative limit a grant uses to cap how often it is used.
pub(crate) const USES: &str = "uses";
/// Grants one session holds at most; more are refused.
const MAX_GRANTS: usize = 64;

/// Sensitive surfaces a grant can open. Everything else has no grant.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Surface {
    /// Run one exact command, or patch, without the protected-path rules.
    UnprotectedCommand,
    /// Type into one process whose sandbox does not have them.
    UnconfinedProcess,
}

impl Surface {
    const ALL: [Self; 2] = [Self::UnprotectedCommand, Self::UnconfinedProcess];

    fn resource_id(self) -> &'static str {
        match self {
            Self::UnprotectedCommand => "sandbox_protected_paths",
            Self::UnconfinedProcess => "unconfined_process",
        }
    }

    pub(crate) fn action(self) -> PolicyAction {
        match self {
            Self::UnprotectedCommand => PolicyAction::Execute,
            Self::UnconfinedProcess => PolicyAction::Use,
        }
    }

    pub(crate) fn resource(self) -> ProtectedResource {
        ProtectedResource {
            kind: ResourceKind::ProtectedData,
            id: bounded(self.resource_id()),
        }
    }

    fn of(resource: &ProtectedResource) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|surface| surface.resource() == *resource)
    }
}

/// The operation for one exact command or patch in one folder.
pub(crate) fn command_operation(parts: &[&str]) -> String {
    let mut hasher = Sha256::new();
    for part in parts {
        hasher.update((part.len() as u64).to_le_bytes());
        hasher.update(part.as_bytes());
    }
    format!("command:sha256:{:x}", hasher.finalize())
}

/// The operation for typing into one start of one process.
pub(crate) fn process_operation(process_id: i32, start: u64) -> String {
    format!("process:{process_id}:{start}")
}

/// Why a grant was not taken.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub(crate) enum GrantRefusal {
    #[error("grants apply only under Aggressive with a live security policy")]
    NotAggressive,
    #[error("the security kill switch is on")]
    KillSwitch,
    #[error("the grant is invalid: {0}")]
    Invalid(String),
    #[error("the grant is for another session")]
    OtherSession,
    #[error("the grant is for another agent")]
    OtherAgent,
    #[error("the grant names no surface Aggressive can open")]
    UnknownSurface,
    #[error("the grant has already expired")]
    Expired,
    #[error("this session already holds {MAX_GRANTS} grants")]
    TooMany,
    #[error("a grant for this exact operation is already held")]
    Duplicate,
}

struct Entry {
    grant: BoundedGrant,
    /// The command the human granted (PF-25-S01); shown in `/security`.
    command: Vec<String>,
    epoch: u64,
    revocation_generation: u64,
    used: u64,
}

impl Entry {
    fn use_limit(&self) -> Option<u64> {
        self.grant
            .scope
            .quantitative_limits
            .get(&bounded(USES))
            .copied()
    }

    fn live(&self, epoch: u64, revocation_generation: u64, now: i64) -> bool {
        self.epoch == epoch
            && self.revocation_generation == revocation_generation
            && !self.grant.is_expired_at(now)
            && self.use_limit().is_none_or(|limit| self.used < limit)
    }
}

static LEDGER: LazyLock<Mutex<HashMap<ThreadId, Vec<Entry>>>> = LazyLock::new(Default::default);

fn bounded(text: &str) -> BoundedText {
    #[allow(clippy::expect_used)]
    BoundedText::new(text).expect("fixed policy text is bounded")
}

/// The policy a grant is bound to, if grants can apply at all.
fn aggressive_binding(state: &PostTaintState) -> Result<(u64, u64, &ActorChain), GrantRefusal> {
    match &state.policy {
        PolicyBinding::Bound {
            epoch,
            revocation_generation,
            kill_switch_active,
            actor_chain,
            ..
        } if state.level == SecurityLevel::Aggressive => {
            if *kill_switch_active {
                return Err(GrantRefusal::KillSwitch);
            }
            Ok((*epoch, *revocation_generation, actor_chain))
        }
        _ => Err(GrantRefusal::NotAggressive),
    }
}

/// [`issue_labelled`] without a label.
#[cfg(test)]
pub(crate) fn issue(
    thread: ThreadId,
    state: &PostTaintState,
    grant: BoundedGrant,
    now_unix_seconds: i64,
) -> Result<(), GrantRefusal> {
    issue_labelled(thread, state, grant, Vec::new(), now_unix_seconds)
}

/// Whether `grant` may apply to `thread` under the policy in `state` now:
/// Aggressive and live, this session and agent, a known surface, unexpired.
/// Returns the policy epoch and revocation generation it binds to.
pub(crate) fn check(
    thread: ThreadId,
    state: &PostTaintState,
    grant: &BoundedGrant,
    now_unix_seconds: i64,
) -> Result<(u64, u64), GrantRefusal> {
    let (epoch, revocation_generation, actor_chain) = aggressive_binding(state)?;
    grant
        .validate()
        .map_err(|error| GrantRefusal::Invalid(error.to_string()))?;
    let thread_text = thread.to_string();
    let context = &grant.scope.context;
    if context.session_id.as_str() != thread_text
        || context.task_id.as_str() != thread_text
        || context.purpose.as_str() != PURPOSE
    {
        return Err(GrantRefusal::OtherSession);
    }
    if grant.actor_chain != *actor_chain {
        return Err(GrantRefusal::OtherAgent);
    }
    let Some(surface) = Surface::of(&grant.scope.resource) else {
        return Err(GrantRefusal::UnknownSurface);
    };
    if grant
        .scope
        .actions
        .iter()
        .any(|action| *action != surface.action())
        || grant.scope.destination.is_some()
        || grant
            .scope
            .quantitative_limits
            .keys()
            .any(|asset| asset.as_str() != USES)
    {
        return Err(GrantRefusal::UnknownSurface);
    }
    if grant.is_expired_at(now_unix_seconds) {
        return Err(GrantRefusal::Expired);
    }
    Ok((epoch, revocation_generation))
}

/// Host-only: hold `grant` for `thread` under the policy in `state`. Only
/// a grant the human confirmed in the grant review reaches it
/// (`grant_offer`, PF-25-S01); no model-reachable path does. `command` is
/// what the human saw.
pub(crate) fn issue_labelled(
    thread: ThreadId,
    state: &PostTaintState,
    grant: BoundedGrant,
    command: Vec<String>,
    now_unix_seconds: i64,
) -> Result<(), GrantRefusal> {
    let (epoch, revocation_generation) = check(thread, state, &grant, now_unix_seconds)?;
    let mut ledger = LEDGER.lock().unwrap_or_else(PoisonError::into_inner);
    let entries = ledger.entry(thread).or_default();
    entries.retain(|entry| entry.live(epoch, revocation_generation, now_unix_seconds));
    // A live grant for the same operation already opens it.
    if entries.iter().any(|entry| {
        entry.grant.scope.resource == grant.scope.resource
            && entry.grant.scope.context == grant.scope.context
    }) {
        return Err(GrantRefusal::Duplicate);
    }
    if entries.len() >= MAX_GRANTS {
        return Err(GrantRefusal::TooMany);
    }
    entries.push(Entry {
        grant,
        command,
        epoch,
        revocation_generation,
        used: 0,
    });
    Ok(())
}

/// The grant that opens `surface` for `operation` now, if one matches; a
/// match uses it once. `None` whenever grants cannot apply.
pub(crate) fn admit(
    thread: ThreadId,
    state: &PostTaintState,
    surface: Surface,
    operation: &str,
    now_unix_seconds: i64,
) -> Option<BoundedText> {
    let Ok((epoch, revocation_generation, actor_chain)) = aggressive_binding(state) else {
        // Another level, the kill switch or no live policy ends every grant:
        // they never come back when Aggressive or the switch returns.
        revoke_all(thread);
        return None;
    };
    let request = AuthorizationRequest::new(
        actor_chain.clone(),
        surface.resource(),
        surface.action(),
        AuthorizationContext {
            now_unix_seconds,
            session_id: BoundedText::new(thread.to_string()).ok()?,
            task_id: BoundedText::new(thread.to_string()).ok()?,
            purpose: bounded(PURPOSE),
            operation: BoundedText::new(operation).ok()?,
            destination: None,
            quantity: QuantitativeLimit::new(USES, /*max_units*/ 1).ok(),
            grant_id: None,
        },
    )
    .ok()?;
    let mut ledger = LEDGER.lock().unwrap_or_else(PoisonError::into_inner);
    let entries = ledger.get_mut(&thread)?;
    entries.retain(|entry| entry.live(epoch, revocation_generation, now_unix_seconds));
    let entry = entries
        .iter_mut()
        .find(|entry| entry.grant.matches_request(&request).unwrap_or(false))?;
    entry.used += 1;
    let grant_id = entry.grant.grant_id.clone();
    tracing::info!(
        target: "codex_core::security::aggressive",
        surface = ?surface,
        grant_id = grant_id.as_str(),
        "aggressive grant used"
    );
    Some(grant_id)
}

/// Drop every grant `thread` holds (also a committed transition, PF-23-S03,
/// and revocation, PF-25-S02).
pub(crate) fn revoke_all(thread: ThreadId) {
    LEDGER
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .remove(&thread);
}

/// A grant held now, as `/security` lists it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HeldGrant {
    pub thread: ThreadId,
    pub grant_id: String,
    /// The command, as one escaped line.
    pub label: String,
    pub command: Vec<String>,
    pub uses_left: Option<u64>,
    pub expires_at_unix_seconds: i64,
}

/// Every grant of this process that is unexpired and has uses left. A level
/// change, revocation or kill switch removes grants from the ledger when it
/// commits, so what is listed is what can still be used.
pub(crate) fn held(now_unix_seconds: i64) -> Vec<HeldGrant> {
    let ledger = LEDGER.lock().unwrap_or_else(PoisonError::into_inner);
    let mut held: Vec<HeldGrant> = ledger
        .iter()
        .flat_map(|(thread, entries)| {
            entries.iter().filter_map(move |entry| {
                let uses_left = entry
                    .use_limit()
                    .map(|limit| limit.saturating_sub(entry.used));
                (!entry.grant.is_expired_at(now_unix_seconds) && uses_left != Some(0)).then(|| {
                    HeldGrant {
                        thread: *thread,
                        grant_id: entry.grant.grant_id.as_str().to_string(),
                        label: super::grant_offer::display_command(&entry.command),
                        command: entry.command.clone(),
                        uses_left,
                        expires_at_unix_seconds: entry.grant.expires_at_unix_seconds,
                    }
                })
            })
        })
        .collect();
    held.sort_by_key(|grant| grant.expires_at_unix_seconds);
    held
}

pub(crate) fn now_unix_seconds() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs() as i64)
}

#[cfg(test)]
#[path = "aggressive_tests.rs"]
mod tests;
