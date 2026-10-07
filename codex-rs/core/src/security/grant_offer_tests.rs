use super::*;
use codex_security_policy::ActorChain;
use codex_security_policy::PolicyPrincipal;
use pretty_assertions::assert_eq;
use std::sync::Arc;
use std::sync::Mutex;

fn chain(agents: &[&str]) -> ActorChain {
    let mut actors = vec![PolicyPrincipal::new(PrincipalKind::Human, "human").unwrap()];
    actors.extend(
        agents
            .iter()
            .map(|id| PolicyPrincipal::new(PrincipalKind::Agent, *id).unwrap()),
    );
    ActorChain::new(actors).unwrap()
}

fn state(level: SecurityLevel, actor_chain: ActorChain, epoch: u64, kill: bool) -> PostTaintState {
    PostTaintState {
        taint_generation: 0,
        policy: PolicyBinding::Bound {
            epoch,
            revocation_generation: 0,
            kill_switch_active: kill,
            level,
            actor_chain,
        },
        level,
    }
}

fn aggressive(agents: &[&str]) -> PostTaintState {
    state(
        SecurityLevel::Aggressive,
        chain(agents),
        /*epoch*/ 0,
        /*kill*/ false,
    )
}

fn operation(command: &str) -> String {
    aggressive::command_operation(&["command", "/work", "perms", command])
}

/// The session's state as `confirm` reads it; tests change it in place.
fn live(state: PostTaintState) -> (Arc<Mutex<Option<PostTaintState>>>, CurrentState) {
    let shared = Arc::new(Mutex::new(Some(state)));
    let reader = Arc::clone(&shared);
    (shared, Box::new(move || reader.lock().unwrap().clone()))
}

fn offer_in(
    thread: ThreadId,
    approval_id: &str,
    command: &str,
    state: PostTaintState,
) -> (OfferGuard, Arc<Mutex<Option<PostTaintState>>>) {
    let (shared, current) = live(state.clone());
    let guard = register(
        thread,
        &state,
        approval_id,
        operation(command),
        vec!["cat".to_string(), command.to_string()],
        "/work".to_string(),
        current,
    );
    (guard, shared)
}

fn admit(thread: ThreadId, state: &PostTaintState, command: &str) -> bool {
    aggressive::admit(
        thread,
        state,
        Surface::UnprotectedCommand,
        &operation(command),
        aggressive::now_unix_seconds(),
    )
    .is_some()
}

fn held_by(thread: ThreadId) -> Vec<HeldGrant> {
    held_grants()
        .into_iter()
        .filter(|grant| grant.thread == thread)
        .collect()
}

/// What the orchestrator does when the approval comes back approved.
fn approved(
    guard: &OfferGuard,
    thread: ThreadId,
    state: &PostTaintState,
    command: &str,
) -> Result<BoundedText, String> {
    let confirmed = guard.take_confirmed().ok_or("nothing confirmed")?;
    apply(
        thread,
        state,
        confirmed,
        &operation(command),
        aggressive::now_unix_seconds(),
    )
}

/// The offer shows the exact fields. Confirming "1 run" records it only;
/// the approved approval then lifts the rules for that run, and nothing is
/// held for another run.
#[test]
fn pf_25_s01_one_run_applies_only_to_the_approved_run() {
    let thread = ThreadId::new();
    let state = aggressive(&["main"]);
    let (guard, _shared) = offer_in(thread, "call-1", "~/.ssh/config", state.clone());

    let shown = offer(thread, "call-1").expect("offered under Aggressive");
    assert_eq!(
        (
            shown.actor_chain.clone(),
            shown.resource.as_str(),
            shown.action.as_str(),
            shown.lifetime_seconds,
            shown.cwd.as_str(),
        ),
        (
            vec!["human:human".to_string(), "agent:main".to_string()],
            "protected_data/sandbox_protected_paths",
            "execute",
            600,
            "/work",
        )
    );
    confirm(&shown, GrantUses::Once).unwrap();
    // Confirmed but not yet approved: nothing applies, nothing is held.
    assert!(!admit(thread, &state, "~/.ssh/config"));
    assert_eq!(held_by(thread), Vec::new());

    assert!(
        approved(&guard, thread, &state, "~/.ssh/id_rsa").is_err(),
        "adjacent command"
    );
    confirm(&shown, GrantUses::Once).unwrap();
    assert!(approved(&guard, thread, &state, "~/.ssh/config").is_ok());
    assert!(
        approved(&guard, thread, &state, "~/.ssh/config").is_err(),
        "used"
    );
    assert!(!admit(thread, &state, "~/.ssh/config"), "never held");
    assert_eq!(held_by(thread), Vec::new());
}

/// "Until it expires" is held when approved: the exact command runs again
/// without asking for another grant, nothing adjacent does, and `/security`
/// lists it.
#[test]
fn pf_25_s01_until_expiry_is_held_after_approval() {
    let thread = ThreadId::new();
    let state = aggressive(&["main"]);
    let (guard, _shared) = offer_in(thread, "call", "x", state.clone());
    let shown = offer(thread, "call").unwrap();
    let confirmed = confirm(&shown, GrantUses::UntilExpiry).unwrap();
    assert_eq!(held_by(thread), Vec::new(), "not before approval");
    approved(&guard, thread, &state, "x").unwrap();
    assert!(admit(thread, &state, "x") && admit(thread, &state, "x"));
    assert!(!admit(thread, &state, "y"));
    let held = held_by(thread);
    assert_eq!(held.len(), 1);
    assert_eq!(held[0].grant_id, confirmed.grant_id);
    assert_eq!(held[0].label, "cat x");
    assert_eq!(held[0].uses_left, None);
}

/// No offer without a live Aggressive policy, or with the kill switch on.
#[test]
fn pf_25_s01_offers_only_under_live_aggressive() {
    let lineage = chain(&["main"]);
    for state in [
        state(SecurityLevel::Moderate, lineage.clone(), 0, false),
        state(SecurityLevel::Permissive, lineage.clone(), 0, false),
        state(SecurityLevel::Aggressive, lineage, 0, true),
        PostTaintState {
            taint_generation: 0,
            policy: PolicyBinding::Unavailable,
            level: SecurityLevel::Aggressive,
        },
    ] {
        let thread = ThreadId::new();
        let (_guard, _shared) = offer_in(thread, "call", "x", state.clone());
        assert_eq!(offer(thread, "call"), None, "{state:?}");
    }
}

/// A declined, cancelled or abandoned approval drops its guard: the offer
/// and a confirmed choice end with it, and a late confirmation fails.
#[test]
fn pf_25_s01_unapproved_approval_leaves_nothing() {
    let thread = ThreadId::new();
    let state = aggressive(&["main"]);
    let (guard, _shared) = offer_in(thread, "call", "x", state.clone());
    let shown = offer(thread, "call").unwrap();
    confirm(&shown, GrantUses::UntilExpiry).unwrap();
    drop(guard);
    assert_eq!(offer(thread, "call"), None);
    assert_eq!(confirm(&shown, GrantUses::Once), Err(GrantError::Ended));
    assert!(!admit(thread, &state, "x"));
    assert_eq!(held_by(thread), Vec::new());
}

/// A shown offer that differs from Core's (another command, a forged actor
/// chain, a longer lifetime, another approval) records nothing.
#[test]
fn pf_25_s01_forged_or_changed_offer_records_nothing() {
    let thread = ThreadId::new();
    let state = aggressive(&["main"]);
    let (guard, _shared) = offer_in(thread, "call", "x", state);
    let shown = offer(thread, "call").unwrap();

    let mut other_command = shown.clone();
    other_command.operation = operation("y");
    let mut other_actor = shown.clone();
    other_actor.actor_chain = vec!["human:human".to_string()];
    let mut longer = shown.clone();
    longer.lifetime_seconds = 86_400;
    for forged in [other_command, other_actor, longer] {
        assert_eq!(confirm(&forged, GrantUses::Once), Err(GrantError::Changed));
    }
    let mut other_approval = shown;
    other_approval.approval_id = "call-other".to_string();
    assert_eq!(
        confirm(&other_approval, GrantUses::Once),
        Err(GrantError::Ended)
    );
    assert!(guard.take_confirmed().is_none());
}

/// A level change, revocation or kill switch since the offer was shown, or
/// the session ending, records nothing; one after confirming, before the
/// run, makes the grant not apply.
#[test]
fn pf_25_s01_state_change_records_or_applies_nothing() {
    let lineage = chain(&["main"]);
    let changed = [
        state(SecurityLevel::Aggressive, lineage.clone(), 1, false),
        state(SecurityLevel::Aggressive, lineage.clone(), 0, true),
        state(SecurityLevel::Permissive, lineage, 0, false),
        state(SecurityLevel::Aggressive, chain(&["other"]), 0, false),
    ];
    for now in changed.clone().into_iter().map(Some).chain([None]) {
        let thread = ThreadId::new();
        let (guard, shared) = offer_in(thread, "call", "x", aggressive(&["main"]));
        let shown = offer(thread, "call").unwrap();
        let expected = if now.is_some() {
            GrantError::Changed
        } else {
            GrantError::Ended
        };
        *shared.lock().unwrap() = now;
        assert_eq!(confirm(&shown, GrantUses::Once), Err(expected));
        assert!(guard.take_confirmed().is_none());
    }
    for now in changed {
        let thread = ThreadId::new();
        let (guard, _shared) = offer_in(thread, "call", "x", aggressive(&["main"]));
        let shown = offer(thread, "call").unwrap();
        confirm(&shown, GrantUses::Once).unwrap();
        assert!(approved(&guard, thread, &now, "x").is_err(), "{now:?}");
    }
}

/// A child's request shows the child's whole lineage and grants only the
/// child: its parent and a sibling gain nothing.
#[test]
fn pf_25_s01_descendant_request_grants_only_that_actor() {
    let parent = ThreadId::new();
    let child = ThreadId::new();
    let sibling = ThreadId::new();
    let child_state = aggressive(&["main", "worker", "helper", "orc"]);
    let (guard, _shared) = offer_in(child, "call", "x", child_state.clone());
    let shown = offer(child, "call").unwrap();
    assert_eq!(
        shown.actor_chain,
        vec![
            "human:human",
            "agent:main",
            "agent:worker",
            "agent:helper",
            "agent:orc"
        ]
    );
    confirm(&shown, GrantUses::UntilExpiry).unwrap();
    approved(&guard, child, &child_state, "x").unwrap();
    assert!(!admit(parent, &aggressive(&["main"]), "x"));
    assert!(!admit(
        sibling,
        &aggressive(&["main", "worker", "helper", "goblin"]),
        "x"
    ));
    assert!(admit(child, &child_state, "x"));
}

/// Flooding: at most eight open offers per session. Two open approvals
/// with one id get no offer. A second held grant for the same command is
/// refused as a duplicate.
#[test]
fn pf_25_s01_offers_are_rate_limited_and_deduplicated() {
    let thread = ThreadId::new();
    let state = aggressive(&["main"]);
    let guards: Vec<_> = (0..MAX_OFFERS_PER_THREAD + 2)
        .map(|index| offer_in(thread, &format!("call-{index}"), "x", state.clone()))
        .collect();
    let open = (0..MAX_OFFERS_PER_THREAD + 2)
        .filter(|index| offer(thread, &format!("call-{index}")).is_some())
        .count();
    assert_eq!(open, MAX_OFFERS_PER_THREAD);

    let first = offer(thread, "call-0").unwrap();
    confirm(&first, GrantUses::UntilExpiry).unwrap();
    approved(&guards[0].0, thread, &state, "x").unwrap();
    let second = offer(thread, "call-1").unwrap();
    confirm(&second, GrantUses::UntilExpiry).unwrap();
    assert!(approved(&guards[1].0, thread, &state, "x").is_err());
    drop(guards);
    assert_eq!(offer(thread, "call-0"), None);

    let other = ThreadId::new();
    let (_first, _shared) = offer_in(other, "same", "x", state.clone());
    let (_second, _shared2) = offer_in(other, "same", "y", state);
    assert_eq!(offer(other, "same"), None, "shared id: no offer");
}

/// Labels never span lines.
#[test]
fn pf_25_s01_display_command_escapes_controls() {
    assert_eq!(
        display_command(&[
            "sh".to_string(),
            "-c".to_string(),
            "a\nb\u{1b}[2J".to_string()
        ]),
        "sh -c 'a\\nb\\u{1b}[2J'"
    );
}
