use super::*;
use codex_security_policy::GrantContext;
use codex_security_policy::GrantScope;
use codex_security_policy::PolicyPrincipal;
use codex_security_policy::PrincipalKind;
use pretty_assertions::assert_eq;
use std::collections::BTreeMap;

const NOW: i64 = 1_000_000;

fn human() -> PolicyPrincipal {
    PolicyPrincipal::new(PrincipalKind::Human, "human").unwrap()
}

fn chain(agents: &[&str]) -> ActorChain {
    let mut actors = vec![human()];
    actors.extend(
        agents
            .iter()
            .map(|id| PolicyPrincipal::new(PrincipalKind::Agent, *id).unwrap()),
    );
    ActorChain::new(actors).unwrap()
}

fn state(level: SecurityLevel, actor_chain: ActorChain, epoch: u64, kill: bool) -> PostTaintState {
    PostTaintState {
        taint_generation: 1,
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

fn aggressive(actor_chain: ActorChain) -> PostTaintState {
    state(
        SecurityLevel::Aggressive,
        actor_chain,
        /*epoch*/ 0,
        /*kill*/ false,
    )
}

fn scope(thread: ThreadId, surface: Surface, operation: &str, uses: Option<u64>) -> GrantScope {
    let text = |value: &str| BoundedText::new(value).unwrap();
    let limits: BTreeMap<_, _> = uses.map(|uses| (text(USES), uses)).into_iter().collect();
    GrantScope::new(
        surface.resource(),
        [surface.action()],
        GrantContext::new(
            text(&thread.to_string()),
            text(&thread.to_string()),
            text(PURPOSE),
            text(operation),
        ),
        /*destination*/ None,
        limits,
    )
    .unwrap()
}

fn grant(actor_chain: ActorChain, scope: GrantScope, expires: i64) -> BoundedGrant {
    BoundedGrant::issue(
        human(),
        actor_chain,
        scope,
        NOW - 10,
        expires,
        BoundedText::new("nonce").unwrap(),
    )
    .unwrap()
}

fn command() -> String {
    command_operation(&["command", "/work", "cat ~/.ssh/config"])
}

/// One narrow grant opens exactly its surface and operation, for its uses.
#[test]
fn pf_23_s02_grant_opens_only_its_surface_operation_and_uses() {
    let thread = ThreadId::new();
    let lineage = chain(&["main"]);
    let state = aggressive(lineage.clone());
    let operation = command();
    let narrow = grant(
        lineage,
        scope(thread, Surface::UnprotectedCommand, &operation, Some(1)),
        NOW + 60,
    );
    issue(thread, &state, narrow.clone(), NOW).unwrap();

    // Adjacent surface, adjacent operation: no.
    for (surface, operation) in [
        (Surface::UnconfinedProcess, operation.clone()),
        (
            Surface::UnprotectedCommand,
            command_operation(&["command", "/work", "cat ~/.ssh/id_rsa"]),
        ),
        (
            Surface::UnprotectedCommand,
            command_operation(&["command", "/other", "cat ~/.ssh/config"]),
        ),
    ] {
        assert_eq!(admit(thread, &state, surface, &operation, NOW), None);
    }
    assert_eq!(
        admit(thread, &state, Surface::UnprotectedCommand, &operation, NOW),
        Some(narrow.grant_id)
    );
    // Its one use is spent.
    assert_eq!(
        admit(thread, &state, Surface::UnprotectedCommand, &operation, NOW),
        None
    );
}

/// Expiry, a policy epoch change, the kill switch, another level and a
/// revocation each end a grant.
#[test]
fn pf_23_s02_grant_ends_with_expiry_policy_change_and_revocation() {
    let thread = ThreadId::new();
    let lineage = chain(&["main"]);
    let operation = command();
    let held = || {
        grant(
            lineage.clone(),
            scope(thread, Surface::UnprotectedCommand, &operation, None),
            NOW + 60,
        )
    };
    let live = aggressive(lineage.clone());
    issue(thread, &live, held(), NOW).unwrap();
    let ended = [
        (live.clone(), NOW + 60),
        (
            state(SecurityLevel::Aggressive, lineage.clone(), 1, false),
            NOW,
        ),
        (
            state(SecurityLevel::Aggressive, lineage.clone(), 0, true),
            NOW,
        ),
        (
            state(SecurityLevel::Moderate, lineage.clone(), 0, false),
            NOW,
        ),
    ];
    for (state, now) in ended {
        assert_eq!(
            admit(thread, &state, Surface::UnprotectedCommand, &operation, now),
            None
        );
    }
    // The epoch-1 check above dropped it: it does not come back.
    assert_eq!(
        admit(thread, &live, Surface::UnprotectedCommand, &operation, NOW),
        None
    );

    issue(thread, &live, held(), NOW).unwrap();
    revoke_all(thread);
    assert_eq!(
        admit(thread, &live, Surface::UnprotectedCommand, &operation, NOW),
        None
    );

    assert_eq!(
        issue(thread, &live, held(), NOW + 60),
        Err(GrantRefusal::Expired)
    );
    for refused in [
        state(SecurityLevel::Moderate, lineage.clone(), 0, false),
        state(SecurityLevel::Permissive, lineage.clone(), 0, false),
    ] {
        assert_eq!(
            issue(thread, &refused, held(), NOW),
            Err(GrantRefusal::NotAggressive)
        );
    }
    assert_eq!(
        issue(
            thread,
            &state(SecurityLevel::Aggressive, lineage.clone(), 0, true),
            held(),
            NOW
        ),
        Err(GrantRefusal::KillSwitch)
    );
    let unbound = PostTaintState {
        taint_generation: 1,
        policy: PolicyBinding::Unavailable,
        level: SecurityLevel::Aggressive,
    };
    assert_eq!(
        issue(thread, &unbound, held(), NOW),
        Err(GrantRefusal::NotAggressive)
    );
}

/// A parent's grant never reaches a child agent: the child's lineage and
/// session differ. A grant for the child must be derived, and no wider.
#[test]
fn pf_23_s02_grants_are_not_inherited_and_derive_only_narrower() {
    let parent_thread = ThreadId::new();
    let child_thread = ThreadId::new();
    let parent = chain(&["main"]);
    let child = chain(&["main", "child"]);
    let operation = command();
    let parent_grant = grant(
        parent.clone(),
        scope(parent_thread, Surface::UnprotectedCommand, &operation, None),
        NOW + 60,
    );
    issue(
        parent_thread,
        &aggressive(parent),
        parent_grant.clone(),
        NOW,
    )
    .unwrap();
    let child_state = aggressive(child.clone());
    assert_eq!(
        admit(
            child_thread,
            &child_state,
            Surface::UnprotectedCommand,
            &operation,
            NOW
        ),
        None
    );
    // Same session, longer lineage: still not the grant's agent.
    assert_eq!(
        admit(
            parent_thread,
            &child_state,
            Surface::UnprotectedCommand,
            &operation,
            NOW
        ),
        None
    );
    assert_eq!(
        issue(child_thread, &child_state, parent_grant.clone(), NOW),
        Err(GrantRefusal::OtherSession)
    );

    let derive = |scope: GrantScope, expires: i64| {
        BoundedGrant::derive_child(
            &parent_grant,
            child.clone(),
            scope,
            NOW,
            expires,
            BoundedText::new("child-nonce").unwrap(),
        )
    };
    // Wider (later expiry, another surface) cannot be derived.
    assert!(
        derive(
            scope(parent_thread, Surface::UnprotectedCommand, &operation, None),
            NOW + 120
        )
        .is_err()
    );
    assert!(
        derive(
            scope(parent_thread, Surface::UnconfinedProcess, &operation, None),
            NOW + 30
        )
        .is_err()
    );
    // A derived grant is bound to the parent's session context, so it never
    // names the child's session: the child cannot hold it either.
    let derived = derive(
        scope(parent_thread, Surface::UnprotectedCommand, &operation, None),
        NOW + 30,
    )
    .unwrap();
    assert_eq!(
        issue(child_thread, &child_state, derived, NOW),
        Err(GrantRefusal::OtherSession)
    );
    // The child gets its own grant from the human instead.
    let own = grant(
        child.clone(),
        scope(child_thread, Surface::UnprotectedCommand, &operation, None),
        NOW + 30,
    );
    issue(child_thread, &child_state, own.clone(), NOW).unwrap();
    assert_eq!(
        admit(
            child_thread,
            &child_state,
            Surface::UnprotectedCommand,
            &operation,
            NOW
        ),
        Some(own.grant_id)
    );
}

/// Unknown surfaces, extra actions, destinations, other limits, other
/// agents and tampered grants are refused when issued.
#[test]
fn pf_23_s02_unknown_or_malformed_grants_are_refused() {
    let thread = ThreadId::new();
    let lineage = chain(&["main"]);
    let state = aggressive(lineage.clone());
    let text = |value: &str| BoundedText::new(value).unwrap();
    let context = || {
        GrantContext::new(
            text(&thread.to_string()),
            text(&thread.to_string()),
            text(PURPOSE),
            text(&command()),
        )
    };
    let refused = |scope: GrantScope, actor_chain: ActorChain| {
        issue(thread, &state, grant(actor_chain, scope, NOW + 60), NOW)
    };
    let unknown = GrantScope::new(
        ProtectedResource::new(ResourceKind::VaultCredential, "anything").unwrap(),
        [PolicyAction::Use],
        context(),
        None,
        BTreeMap::new(),
    )
    .unwrap();
    assert_eq!(
        refused(unknown, lineage.clone()),
        Err(GrantRefusal::UnknownSurface)
    );
    let extra_action = GrantScope::new(
        Surface::UnprotectedCommand.resource(),
        [PolicyAction::Execute, PolicyAction::Export],
        context(),
        None,
        BTreeMap::new(),
    )
    .unwrap();
    assert_eq!(
        refused(extra_action, lineage.clone()),
        Err(GrantRefusal::UnknownSurface)
    );
    let destination = GrantScope::new(
        Surface::UnprotectedCommand.resource(),
        [PolicyAction::Execute],
        context(),
        Some(text("example.com")),
        BTreeMap::new(),
    )
    .unwrap();
    assert_eq!(
        refused(destination, lineage.clone()),
        Err(GrantRefusal::UnknownSurface)
    );
    let other_limit = GrantScope::new(
        Surface::UnprotectedCommand.resource(),
        [PolicyAction::Execute],
        context(),
        None,
        [(text("bytes"), 10)].into_iter().collect(),
    )
    .unwrap();
    assert_eq!(
        refused(other_limit, lineage.clone()),
        Err(GrantRefusal::UnknownSurface)
    );
    assert_eq!(
        refused(
            scope(thread, Surface::UnprotectedCommand, &command(), None),
            chain(&["other"])
        ),
        Err(GrantRefusal::OtherAgent)
    );
    let mut tampered = grant(
        lineage,
        scope(thread, Surface::UnprotectedCommand, &command(), Some(1)),
        NOW + 60,
    );
    tampered.scope.quantitative_limits.insert(text(USES), 99);
    assert!(matches!(
        issue(thread, &state, tampered, NOW),
        Err(GrantRefusal::Invalid(_))
    ));
}
