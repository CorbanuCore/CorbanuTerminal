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

/// The offer shows the exact fields; confirming issues one grant for this
/// command only, used once, and `/security` lists it until it is used.
#[test]
fn pf_25_s01_confirm_issues_one_exact_single_use_grant() {
    let thread = ThreadId::new();
    let state = aggressive(&["main"]);
    let (_guard, _shared) = offer_in(thread, "call-1", "~/.ssh/config", state.clone());

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
    assert!(
        !admit(thread, &state, "~/.ssh/config"),
        "an offer grants nothing"
    );

    let confirmed = confirm(&shown, GrantUses::Once).unwrap();
    let held: Vec<_> = held_grants()
        .into_iter()
        .filter(|grant| grant.thread == thread)
        .collect();
    assert_eq!(held.len(), 1);
    assert_eq!(held[0].grant_id, confirmed.grant_id);
    assert_eq!(held[0].label, "cat '~/.ssh/config'");
    assert_eq!(held[0].uses_left, Some(1));

    assert!(!admit(thread, &state, "~/.ssh/id_rsa"), "adjacent command");
    assert!(admit(thread, &state, "~/.ssh/config"));
    assert!(!admit(thread, &state, "~/.ssh/config"), "one use");
    assert!(held_grants().iter().all(|grant| grant.thread != thread));
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

/// Esc, deny or any other answer ends the approval: the offer is gone and
/// a late confirmation grants nothing.
#[test]
fn pf_25_s01_answered_approval_ends_the_offer() {
    let thread = ThreadId::new();
    let state = aggressive(&["main"]);
    let (guard, _shared) = offer_in(thread, "call", "x", state.clone());
    let shown = offer(thread, "call").unwrap();
    drop(guard);
    assert_eq!(offer(thread, "call"), None);
    assert_eq!(confirm(&shown, GrantUses::Once), Err(GrantError::Ended));
    assert!(!admit(thread, &state, "x"));
}

/// A shown offer that differs from Core's (another command, a forged actor
/// chain or approval) grants nothing.
#[test]
fn pf_25_s01_forged_or_changed_offer_grants_nothing() {
    let thread = ThreadId::new();
    let state = aggressive(&["main"]);
    let (_guard, _shared) = offer_in(thread, "call", "x", state.clone());
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
    assert!(!admit(thread, &state, "x") && !admit(thread, &state, "y"));
}

/// A level change, revocation or kill switch since the offer was shown, or
/// the session ending, grants nothing.
#[test]
fn pf_25_s01_state_change_since_shown_grants_nothing() {
    let lineage = chain(&["main"]);
    for (now, expected) in [
        (
            Some(state(SecurityLevel::Aggressive, lineage.clone(), 1, false)),
            GrantError::Changed,
        ),
        (
            Some(state(SecurityLevel::Aggressive, lineage.clone(), 0, true)),
            GrantError::Changed,
        ),
        (
            Some(state(SecurityLevel::Permissive, lineage, 0, false)),
            GrantError::Changed,
        ),
        (None, GrantError::Ended),
    ] {
        let thread = ThreadId::new();
        let (_guard, shared) = offer_in(thread, "call", "x", aggressive(&["main"]));
        let shown = offer(thread, "call").unwrap();
        *shared.lock().unwrap() = now;
        assert_eq!(confirm(&shown, GrantUses::Once), Err(expected));
        assert!(held_grants().iter().all(|grant| grant.thread != thread));
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
    let (_guard, _shared) = offer_in(child, "call", "x", child_state.clone());
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
    confirm(&shown, GrantUses::Once).unwrap();
    assert!(!admit(parent, &aggressive(&["main"]), "x"));
    assert!(!admit(
        sibling,
        &aggressive(&["main", "worker", "helper", "goblin"]),
        "x"
    ));
    assert!(admit(child, &child_state, "x"));
}

/// "Until it expires": the exact command runs as often as needed, nothing
/// adjacent does, and `/security` lists it with no use limit.
#[test]
fn pf_25_s01_until_expiry_grant_has_no_use_limit() {
    let thread = ThreadId::new();
    let state = aggressive(&["main"]);
    let (_guard, _shared) = offer_in(thread, "call", "x", state.clone());
    let shown = offer(thread, "call").unwrap();
    confirm(&shown, GrantUses::UntilExpiry).unwrap();
    assert!(admit(thread, &state, "x") && admit(thread, &state, "x"));
    assert!(!admit(thread, &state, "y"));
    let held: Vec<_> = held_grants()
        .into_iter()
        .filter(|grant| grant.thread == thread)
        .collect();
    assert_eq!(held.len(), 1);
    assert_eq!(held[0].uses_left, None);
    assert!(held[0].expires_at_unix_seconds > aggressive::now_unix_seconds());
}

/// Flooding: at most eight open offers per session; a repeated confirmation
/// of the same offer is refused as a duplicate.
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
    confirm(&first, GrantUses::Once).unwrap();
    assert!(matches!(
        confirm(&first, GrantUses::Once),
        Err(GrantError::Refused(_))
    ));
    let same_command = offer(thread, "call-1").unwrap();
    assert!(matches!(
        confirm(&same_command, GrantUses::Once),
        Err(GrantError::Refused(_))
    ));
    drop(guards);
    assert_eq!(offer(thread, "call-0"), None);
}
