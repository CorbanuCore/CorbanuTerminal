//! PF-25-S01: one Aggressive grant, offered by Core while it waits for the
//! human's answer to one exact command, and confirmed only in the TUI.
//!
//! Under Aggressive a human approval never lifts the protected-path rules
//! (PF-23-S02); a matching grant does. While a command approval is open,
//! Core records an offer: the command, its folder, the agent lineage (actor
//! chain) and session it runs in, and the policy epoch. The offer is data,
//! never authority. [`confirm`], called by the TUI's grant review after the
//! person chose "Grant", only records their choice on the offer. The
//! orchestrator applies it, and only when that same approval comes back
//! approved: then this command runs once without the rules ("1 run", never
//! held), or the grant is held for any run of the exact command until it
//! expires ([`GRANT_LIFETIME_SECONDS`]). A declined, cancelled or abandoned
//! approval ends the offer and the choice with it.
//!
//! No `Op`, app-server method, tool or model output reaches [`confirm`]; the
//! issuer is the human principal of the session's live policy, never a field
//! of the offer; and the grant names the actor chain of the session that
//! asked, so its parent, siblings and other sessions gain nothing. Grants
//! live in memory only ([`super::aggressive`]): a level change, revocation,
//! kill switch or restart ends them.

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::LazyLock;
use std::sync::Mutex;
use std::sync::PoisonError;
use std::sync::Weak;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

use codex_protocol::ThreadId;
use codex_security_policy::BoundedGrant;
use codex_security_policy::BoundedText;
use codex_security_policy::GrantContext;
use codex_security_policy::GrantScope;
use codex_security_policy::PrincipalKind;
use codex_security_policy::SecurityLevel;

use super::aggressive;
use super::aggressive::Surface;
use super::tainted_action::PolicyBinding;
use super::tainted_action::PostTaintState;
use crate::session::session::Session;

pub use super::aggressive::HeldGrant;

/// How long a confirmed grant lasts.
pub const GRANT_LIFETIME_SECONDS: i64 = 10 * 60;

/// How often a confirmed grant may be used before it expires.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GrantUses {
    /// One run of the command (the default).
    #[default]
    Once,
    /// Any run of the exact command until the grant expires.
    UntilExpiry,
}
/// Open offers one session may have at once; past it, no grant is offered.
const MAX_OFFERS_PER_THREAD: usize = 8;

/// What the grant review shows, field by field. Built by Core from the
/// command it is about to run; the TUI only displays it and hands it back.
/// A value built anywhere else grants nothing: [`confirm`] issues only an
/// open offer equal to it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GrantOffer {
    pub thread: ThreadId,
    /// The approval this offer belongs to.
    pub approval_id: String,
    /// The agent lineage the grant is for, root first (`human:…`, `agent:…`).
    pub actor_chain: Vec<String>,
    pub resource: String,
    pub action: String,
    /// The exact command digest a grant names.
    pub operation: String,
    pub command: Vec<String>,
    pub cwd: String,
    pub lifetime_seconds: i64,
    /// The policy the offer was made under; a grant needs it unchanged.
    pub epoch: u64,
    pub revocation_generation: u64,
}

/// A confirmed grant.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfirmedGrant {
    pub grant_id: String,
    pub expires_at_unix_seconds: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum GrantError {
    #[error("this request was already answered or ended. Nothing was granted")]
    Ended,
    #[error(
        "the security state or the request changed since this was shown. Nothing was granted; review it again"
    )]
    Changed,
    #[error("not granted: {0}")]
    Refused(String),
}

/// Reads the session's policy state now; `None` once the session ended.
/// Called outside the offers lock.
type CurrentState = Arc<dyn Fn() -> Option<PostTaintState> + Send + Sync>;

struct Pending {
    offer: GrantOffer,
    current: CurrentState,
    /// Which registration this is: a guard acts only on its own.
    nonce: u64,
    /// The person's confirmed choice, applied only if this approval is
    /// approved.
    confirmed: Option<Confirmed>,
    /// The orchestrator took the choice: a later confirmation is too late.
    closed: bool,
}

static NEXT_NONCE: AtomicU64 = AtomicU64::new(1);

/// A grant the person confirmed, waiting for its approval to be approved.
pub(crate) struct Confirmed {
    grant: BoundedGrant,
    uses: GrantUses,
    command: Vec<String>,
    /// The policy it was confirmed under; any commit since voids it.
    epoch: u64,
    revocation_generation: u64,
}

type OfferKey = (ThreadId, String);

static OFFERS: LazyLock<Mutex<HashMap<OfferKey, Pending>>> = LazyLock::new(Default::default);

fn offers() -> std::sync::MutexGuard<'static, HashMap<OfferKey, Pending>> {
    OFFERS.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Removes its offer (and any confirmed choice) when the approval it belongs
/// to is answered.
#[must_use]
pub(crate) struct OfferGuard(Option<(OfferKey, u64)>);

impl OfferGuard {
    /// The approval was approved: the person's confirmed grant, if any. The
    /// offer is closed from now on.
    pub(crate) fn take_confirmed(&self) -> Option<Confirmed> {
        let (key, nonce) = self.0.as_ref()?;
        let mut offers = offers();
        let pending = offers
            .get_mut(key)
            .filter(|pending| pending.nonce == *nonce)?;
        pending.closed = true;
        pending.confirmed.take()
    }
}

impl Drop for OfferGuard {
    fn drop(&mut self) {
        if let Some((key, nonce)) = self.0.take() {
            let mut offers = offers();
            if offers
                .get(&key)
                .is_some_and(|pending| pending.nonce == nonce)
            {
                offers.remove(&key);
            }
        }
    }
}

/// The live Aggressive policy a grant can bind to: epoch, revocation
/// generation and actor chain. `None` under any other level, without a live
/// policy, or with the kill switch on.
fn binding(state: &PostTaintState) -> Option<(u64, u64, &codex_security_policy::ActorChain)> {
    match &state.policy {
        PolicyBinding::Bound {
            epoch,
            revocation_generation,
            kill_switch_active: false,
            actor_chain,
            ..
        } if state.level == SecurityLevel::Aggressive => {
            Some((*epoch, *revocation_generation, actor_chain))
        }
        _ => None,
    }
}

fn principal_label(kind: PrincipalKind, id: &str) -> String {
    let kind = match kind {
        PrincipalKind::Human => "human",
        PrincipalKind::Agent => "agent",
        PrincipalKind::Tool => "tool",
        PrincipalKind::Service => "service",
    };
    // Ids such as `human:session:…` already name their kind.
    if id.starts_with(&format!("{kind}:")) {
        id.to_string()
    } else {
        format!("{kind}:{id}")
    }
}

/// Offer a grant for the approval `approval_id` of `session`, which is
/// about to run `command` in `cwd` (`operation` is its grant digest). Only
/// under a live Aggressive policy; at most [`MAX_OFFERS_PER_THREAD`] per
/// session. The offer ends when the guard drops.
pub(crate) fn offer_for_approval(
    session: &Arc<Session>,
    state: &PostTaintState,
    approval_id: &str,
    operation: String,
    command: Vec<String>,
    cwd: String,
) -> OfferGuard {
    let weak: Weak<Session> = Arc::downgrade(session);
    register(
        session.thread_id(),
        state,
        approval_id,
        operation,
        command,
        cwd,
        Arc::new(move || weak.upgrade()?.services.model_client().post_taint_state()),
    )
}

fn register(
    thread: ThreadId,
    state: &PostTaintState,
    approval_id: &str,
    operation: String,
    command: Vec<String>,
    cwd: String,
    current: CurrentState,
) -> OfferGuard {
    let Some((epoch, revocation_generation, actor_chain)) = binding(state) else {
        return OfferGuard(None);
    };
    let surface = Surface::UnprotectedCommand;
    let offer = GrantOffer {
        thread,
        approval_id: approval_id.to_string(),
        actor_chain: chain_labels(actor_chain),
        resource: format!("protected_data/{}", surface.resource().id.as_str()),
        action: "execute".to_string(),
        operation,
        command,
        cwd,
        lifetime_seconds: GRANT_LIFETIME_SECONDS,
        epoch,
        revocation_generation,
    };
    let key = (thread, approval_id.to_string());
    let mut offers = offers();
    // Two open approvals with one id (a provider reusing call ids): the
    // review could show one command under the other's approval. Offer
    // nothing for either.
    if offers.remove(&key).is_some() {
        tracing::warn!(
            target: "codex_core::security::grant_offer",
            %thread,
            "two open approvals share an id; no grant offered for them"
        );
        return OfferGuard(None);
    }
    let open = offers.keys().filter(|(owner, _)| *owner == thread).count();
    if open >= MAX_OFFERS_PER_THREAD {
        tracing::warn!(
            target: "codex_core::security::grant_offer",
            %thread,
            "too many open grant offers; none offered for this command"
        );
        return OfferGuard(None);
    }
    let nonce = NEXT_NONCE.fetch_add(1, Ordering::Relaxed);
    offers.insert(
        key.clone(),
        Pending {
            offer,
            current,
            nonce,
            confirmed: None,
            closed: false,
        },
    );
    OfferGuard(Some((key, nonce)))
}

/// The open offer for approval `approval_id` of `thread`, if Core made one.
pub fn offer(thread: ThreadId, approval_id: &str) -> Option<GrantOffer> {
    offers()
        .get(&(thread, approval_id.to_string()))
        .map(|pending| pending.offer.clone())
}

/// The human confirmed `shown` with `uses` in the grant review. Records
/// exactly that grant on the offer, if it is still open, unchanged, and the
/// session's policy and actor chain are still the ones it was made under.
/// Nothing applies until the same approval comes back approved. Call only
/// from the TUI's confirm key.
pub fn confirm(shown: &GrantOffer, uses: GrantUses) -> Result<ConfirmedGrant, GrantError> {
    let key = (shown.thread, shown.approval_id.clone());
    let (current, nonce) = {
        let offers = offers();
        let pending = offers
            .get(&key)
            .filter(|pending| !pending.closed)
            .ok_or(GrantError::Ended)?;
        if pending.offer != *shown {
            return Err(GrantError::Changed);
        }
        (Arc::clone(&pending.current), pending.nonce)
    };
    let state = current().ok_or(GrantError::Ended)?;
    let (epoch, revocation_generation, actor_chain) = binding(&state).ok_or(GrantError::Changed)?;
    if (epoch, revocation_generation) != (shown.epoch, shown.revocation_generation)
        || chain_labels(actor_chain) != shown.actor_chain
    {
        return Err(GrantError::Changed);
    }
    let issuer = actor_chain
        .as_slice()
        .first()
        .filter(|principal| principal.kind == PrincipalKind::Human)
        .cloned()
        .ok_or_else(|| GrantError::Refused("the session has no human principal".to_string()))?;
    let refused = |error: &dyn std::fmt::Display| GrantError::Refused(error.to_string());
    let text = |value: &str| BoundedText::new(value).map_err(|error| refused(&error));
    let thread = shown.thread.to_string();
    let surface = Surface::UnprotectedCommand;
    let scope = GrantScope::new(
        surface.resource(),
        [surface.action()],
        GrantContext::new(
            text(&thread)?,
            text(&thread)?,
            text(aggressive::PURPOSE)?,
            text(&shown.operation)?,
        ),
        /*destination*/ None,
        match uses {
            GrantUses::Once => BTreeMap::from([(text(aggressive::USES)?, 1)]),
            GrantUses::UntilExpiry => BTreeMap::new(),
        },
    )
    .map_err(|error| refused(&error))?;
    let now = aggressive::now_unix_seconds();
    let expires = now.saturating_add(shown.lifetime_seconds);
    let grant = BoundedGrant::issue(
        issuer,
        actor_chain.clone(),
        scope,
        now,
        expires,
        text(&uuid::Uuid::new_v4().to_string())?,
    )
    .map_err(|error| refused(&error))?;
    aggressive::check(shown.thread, &state, &grant, now).map_err(|error| refused(&error))?;
    let grant_id = grant.grant_id.as_str().to_string();
    let mut offers = offers();
    let pending = offers
        .get_mut(&key)
        .filter(|pending| pending.nonce == nonce && !pending.closed)
        .ok_or(GrantError::Ended)?;
    if pending.offer != *shown {
        return Err(GrantError::Changed);
    }
    pending.confirmed = Some(Confirmed {
        grant,
        uses,
        command: shown.command.clone(),
        epoch,
        revocation_generation,
    });
    tracing::info!(
        target: "codex_core::security::grant_offer",
        thread = %shown.thread,
        grant_id = grant_id.as_str(),
        ?uses,
        "human confirmed an Aggressive grant; it applies if the approval is approved"
    );
    Ok(ConfirmedGrant {
        grant_id,
        expires_at_unix_seconds: expires,
    })
}

/// The approval of `operation` came back approved with `confirmed` on it:
/// apply it under the policy in `state` now. "1 run" lifts the rules for
/// this run only and is never held; "until it expires" is held and used.
/// Returns the grant id, or why it does not apply (the rules then stay).
pub(crate) fn apply(
    thread: ThreadId,
    state: &PostTaintState,
    confirmed: Confirmed,
    operation: &str,
    now_unix_seconds: i64,
) -> Result<BoundedText, String> {
    let Confirmed {
        grant,
        uses,
        command,
        epoch,
        revocation_generation,
    } = confirmed;
    if grant.scope.context.operation.as_str() != operation {
        return Err("the grant is for another command".to_string());
    }
    let bound = aggressive::check(thread, state, &grant, now_unix_seconds)
        .map_err(|error| error.to_string())?;
    if bound != (epoch, revocation_generation) {
        return Err("the security state changed since the grant was confirmed".to_string());
    }
    match uses {
        GrantUses::Once => Ok(grant.grant_id),
        GrantUses::UntilExpiry => {
            aggressive::issue_labelled(thread, state, grant, command, now_unix_seconds)
                .map_err(|error| error.to_string())?;
            aggressive::admit(
                thread,
                state,
                Surface::UnprotectedCommand,
                operation,
                now_unix_seconds,
            )
            .ok_or_else(|| "the held grant does not match this run".to_string())
        }
    }
}

fn is_invisible_format(character: char) -> bool {
    matches!(
        character,
        '\u{00ad}'
            | '\u{061c}'
            | '\u{180e}'
            | '\u{200b}'..='\u{200f}'
            | '\u{202a}'..='\u{202e}'
            | '\u{2060}'..='\u{2064}'
            | '\u{2066}'..='\u{2069}'
            | '\u{feff}'
    )
}

fn chain_labels(actor_chain: &codex_security_policy::ActorChain) -> Vec<String> {
    actor_chain
        .as_slice()
        .iter()
        .map(|principal| principal_label(principal.kind, principal.id.as_str()))
        .collect()
}

/// A command as one line: shell-joined, control characters escaped, so a
/// label never spans lines or hides text.
pub fn display_command(command: &[String]) -> String {
    let joined =
        shlex::try_join(command.iter().map(String::as_str)).unwrap_or_else(|_| command.join(" "));
    escape_controls(&joined)
}

/// `text` with control characters and invisible format characters (bidi
/// overrides and isolates, zero-width characters) shown as escapes (`\n`,
/// `\u{1b}`, `\u{202e}`), so what is shown is what runs.
pub fn escape_controls(text: &str) -> String {
    text.chars()
        .flat_map(|character| {
            if character.is_control() || is_invisible_format(character) {
                character.escape_default().collect::<Vec<_>>()
            } else {
                vec![character]
            }
        })
        .collect()
}

/// Test support for the TUI's grant review tests: an open offer for
/// `command` in `thread` under a live Aggressive policy, as the
/// orchestrator makes one. It grants nothing: a confirmed choice applies
/// only when the orchestrator that made an offer sees its approval approved,
/// and none waits on this one.
#[doc(hidden)]
pub fn open_test_offer(thread: ThreadId, approval_id: &str, command: Vec<String>) -> TestOffer {
    let Ok(actor_chain) =
        codex_security_policy::PolicyPrincipal::new(PrincipalKind::Human, "human")
            .map_err(|_| ())
            .and_then(|human| codex_security_policy::ActorChain::new(vec![human]).map_err(|_| ()))
    else {
        return TestOffer(OfferGuard(None));
    };
    let state = PostTaintState {
        taint_generation: 0,
        policy: PolicyBinding::Bound {
            epoch: 0,
            revocation_generation: 0,
            kill_switch_active: false,
            level: SecurityLevel::Aggressive,
            actor_chain,
        },
        level: SecurityLevel::Aggressive,
    };
    if offers().contains_key(&(thread, approval_id.to_string())) {
        // Never touches a real offer.
        return TestOffer(OfferGuard(None));
    }
    let current = state.clone();
    TestOffer(register(
        thread,
        &state,
        approval_id,
        aggressive::command_operation(&["test", &command.join(" ")]),
        command,
        "/work".to_string(),
        Arc::new(move || Some(current.clone())),
    ))
}

/// An offer from [`open_test_offer`]; it ends when dropped.
#[doc(hidden)]
pub struct TestOffer(OfferGuard);

impl TestOffer {
    /// The choice the person confirmed on it, if any.
    pub fn confirmed_uses(&self) -> Option<GrantUses> {
        let (key, nonce) = self.0.0.as_ref()?;
        offers()
            .get(key)
            .filter(|pending| pending.nonce == *nonce)?
            .confirmed
            .as_ref()
            .map(|confirmed| confirmed.uses)
    }
}

/// Grants held now, for `/security`.
pub fn held_grants() -> Vec<HeldGrant> {
    aggressive::held(aggressive::now_unix_seconds())
}

#[cfg(test)]
#[path = "grant_offer_tests.rs"]
mod tests;
