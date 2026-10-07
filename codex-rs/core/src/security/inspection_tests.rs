use std::sync::Arc;

use codex_protocol::SessionId;
use codex_security_policy::ActorChain;
use codex_security_policy::BoundedGrant;
use codex_security_policy::BoundedText;
use codex_security_policy::GrantContext;
use codex_security_policy::GrantScope;
use codex_security_policy::PolicyPrincipal;
use codex_security_policy::PrincipalKind;
use codex_security_policy::RevocationState;
use codex_security_policy::SecuritySettings;
use pretty_assertions::assert_eq;
use tempfile::TempDir;

use super::*;
use crate::security::EffectivePolicyInitialization;
use crate::security::EffectivePolicyView;
use crate::security::PersistedHumanSecurityState;
use crate::security::TrustedSecurityController;
use crate::security::aggressive;
use crate::security::tainted_action::PolicyBinding;
use crate::security::tainted_action::PostTaintState;

const NOW: i64 = 1_000_000;

fn human() -> PolicyPrincipal {
    PolicyPrincipal::new(PrincipalKind::Human, "human:test").unwrap()
}

/// A live session tree on `home`, as a session start registers it.
fn live_tree(home: &TempDir, level: SecurityLevel) -> (EffectivePolicyView, ThreadId) {
    let view = EffectivePolicyView::default();
    let root = ThreadId::new();
    TrustedSecurityController::initialize(
        &view,
        PersistedHumanSecurityState::new(
            SecuritySettings::new(level),
            human(),
            RevocationState::new(),
        )
        .unwrap(),
        root,
        SessionId::from(root),
        EffectivePolicyInitialization::Root,
    )
    .unwrap();
    view.register_home(home.path());
    (view, root)
}

fn agent(thread: ThreadId, depth: usize, level: SecurityLevel, stricter: bool) -> AgentFacts {
    AgentFacts {
        thread,
        depth,
        level,
        stricter_than_session: stricter,
        stopped: false,
    }
}

#[test]
fn pf_41_s01_live_tree_reports_levels_children_and_generation() {
    let home = TempDir::new().unwrap();
    let (view, root) = live_tree(&home, SecurityLevel::Moderate);
    let same = ThreadId::new();
    let stricter = ThreadId::new();
    view.inherit_child(root, same, "task", SecurityLevel::Permissive)
        .unwrap();
    view.inherit_child(root, stricter, "task", SecurityLevel::Aggressive)
        .unwrap();

    let facts = observe(home.path(), SecurityLevel::Permissive, Some(root), NOW);

    let mut children = vec![
        agent(same, 1, SecurityLevel::Moderate, false),
        agent(stricter, 1, SecurityLevel::Aggressive, true),
    ];
    children.sort_by_key(|agent| agent.thread.to_string());
    let mut agents = vec![agent(root, 0, SecurityLevel::Moderate, false)];
    agents.extend(children);
    assert_eq!(
        facts.policy,
        PolicyFacts::Live(TreeFacts {
            in_force: SecurityLevel::Moderate,
            next_start: SecurityLevel::Moderate,
            kill_switch: false,
            epoch: 0,
            revocation_generation: 0,
            agents,
        })
    );
    // A child's view reports the same tree.
    assert_eq!(
        observe(home.path(), SecurityLevel::Permissive, Some(stricter), NOW).policy,
        facts.policy
    );
}

#[test]
fn pf_41_s01_without_a_session_reads_stored_state_and_unreadable_fails_closed() {
    let home = TempDir::new().unwrap();
    assert_eq!(
        observe(
            home.path(),
            SecurityLevel::Moderate,
            /*thread*/ None,
            NOW
        )
        .policy,
        PolicyFacts::Stored {
            level: SecurityLevel::Moderate,
            kill_switch: false,
            unreadable: false,
        }
    );
    // A session of another process (or a remote app server) is not live here.
    assert_eq!(
        observe(
            home.path(),
            SecurityLevel::Permissive,
            Some(ThreadId::new()),
            NOW
        )
        .policy,
        PolicyFacts::Stored {
            level: SecurityLevel::Permissive,
            kill_switch: false,
            unreadable: false,
        }
    );

    std::fs::write(
        home.path().join(super::super::recovery::STATE_FILE),
        "{broken",
    )
    .unwrap();
    assert_eq!(
        observe(
            home.path(),
            SecurityLevel::Permissive,
            /*thread*/ None,
            NOW
        )
        .policy,
        PolicyFacts::Stored {
            level: SecurityLevel::Aggressive,
            kill_switch: true,
            unreadable: true,
        }
    );
}

fn grant_for(root: ThreadId, expires: i64) -> (PostTaintState, BoundedGrant) {
    let text = |value: &str| BoundedText::new(value).unwrap();
    let chain = ActorChain::new(vec![
        human(),
        PolicyPrincipal::new(PrincipalKind::Agent, format!("agent:{root}")).unwrap(),
    ])
    .unwrap();
    let surface = aggressive::Surface::UnprotectedCommand;
    let scope = GrantScope::new(
        surface.resource(),
        [surface.action()],
        GrantContext::new(
            text(&root.to_string()),
            text(&root.to_string()),
            text(aggressive::PURPOSE),
            text("command:sha256:secret-operation"),
        ),
        /*destination*/ None,
        [(text(aggressive::USES), 2)].into_iter().collect(),
    )
    .unwrap();
    let grant = BoundedGrant::issue(
        human(),
        chain.clone(),
        scope,
        NOW - 10,
        expires,
        text("nonce"),
    )
    .unwrap();
    let state = PostTaintState {
        taint_generation: 1,
        policy: PolicyBinding::Bound {
            epoch: 0,
            revocation_generation: 0,
            kill_switch_active: false,
            level: SecurityLevel::Aggressive,
            actor_chain: chain,
        },
        level: SecurityLevel::Aggressive,
    };
    (state, grant)
}

#[test]
fn pf_41_s01_grants_show_expiry_and_uses_without_ids_and_end_on_expiry() {
    let home = TempDir::new().unwrap();
    let (_view, root) = live_tree(&home, SecurityLevel::Aggressive);
    let (state, grant) = grant_for(root, NOW + 300);
    let grant_id = grant.grant_id.as_str().to_string();
    aggressive::issue(root, &state, grant, NOW).unwrap();

    let facts = observe(home.path(), SecurityLevel::Aggressive, Some(root), NOW);
    assert_eq!(
        facts.grants,
        vec![GrantFacts {
            thread: root,
            surface: "one command without the protected-path rules",
            expires_at: NOW + 300,
            used: 0,
            limit: Some(2),
        }]
    );
    let shown = format!("{facts:?}");
    assert!(!shown.contains(&grant_id) && !shown.contains("secret-operation"));

    // Expired grants are not shown as held.
    assert_eq!(
        observe(
            home.path(),
            SecurityLevel::Aggressive,
            Some(root),
            NOW + 300
        )
        .grants,
        Vec::new()
    );
    aggressive::revoke_all(root);
}

#[test]
fn pf_41_s01_grants_of_a_stopped_session_are_not_listed() {
    let home = TempDir::new().unwrap();
    let view = EffectivePolicyView::default();
    let root = ThreadId::new();
    // Unreadable state at start: Aggressive, and the session is stopped.
    TrustedSecurityController::initialize(
        &view,
        PersistedHumanSecurityState::new(
            SecuritySettings::new(SecurityLevel::Aggressive),
            human(),
            RevocationState::new(),
        )
        .unwrap(),
        root,
        SessionId::from(root),
        EffectivePolicyInitialization::UnreadableState,
    )
    .unwrap();
    view.register_home(home.path());
    let (state, grant) = grant_for(root, NOW + 300);
    aggressive::issue(root, &state, grant, NOW).unwrap();

    let facts = observe(home.path(), SecurityLevel::Aggressive, Some(root), NOW);
    assert!(
        matches!(&facts.policy, PolicyFacts::Live(tree) if tree.agents[0].stopped),
        "{:?}",
        facts.policy
    );
    assert_eq!(facts.grants, Vec::new());
    aggressive::revoke_all(root);
}

#[test]
fn pf_41_s01_denials_are_fixed_text_and_correlated_to_the_session_tree() {
    let home = TempDir::new().unwrap();
    let (view, root) = live_tree(&home, SecurityLevel::Aggressive);
    let child = ThreadId::new();
    view.inherit_child(root, child, "task", SecurityLevel::Aggressive)
        .unwrap();
    let elsewhere = ThreadId::new();
    record_protected_action(
        Some(child),
        ProtectedActionKind::Vault,
        "refused_kill_switch",
    );
    record_protected_action(Some(elsewhere), ProtectedActionKind::Vault, "declined");
    record_protected_action(Some(root), ProtectedActionKind::Vault, "approved");
    drop(PendingProtectedAction::new(
        root,
        ProtectedActionKind::Persistence,
    ));
    let mut answered = PendingProtectedAction::new(root, ProtectedActionKind::Vault);
    answered.answered();
    drop(answered);
    record_launch_denial(&LaunchDenied::ProtectedPathReadable(
        "/home/user/.ssh/id_rsa".to_string(),
    ));

    let denials = observe(home.path(), SecurityLevel::Aggressive, Some(root), NOW).denials;
    let shown = denials
        .iter()
        .map(|denial| (denial.thread, denial.reason, denial.outcome))
        .collect::<Vec<_>>();
    // Other tests record into the same process-wide list, so check
    // presence, not position.
    for expected in [
        (None, "a protected path was readable", "launch refused"),
        (
            Some(root),
            ProtectedActionKind::Persistence.describe(),
            "declined by you (turn interrupted)",
        ),
        (Some(child), "vault access", "refused: kill switch on"),
    ] {
        assert!(shown.contains(&expected), "{expected:?} in {shown:?}");
    }
    // Approvals (and answered checks) are not denials; other sessions'
    // denials are not shown.
    assert_eq!(
        shown
            .iter()
            .filter(|(thread, ..)| *thread == Some(root) || *thread == Some(elsewhere))
            .count(),
        1,
        "{shown:?}"
    );
    assert!(!format!("{denials:?}").contains(".ssh"));
}

#[test]
fn pf_41_s01_taint_is_read_from_the_registered_session() {
    let thread = ThreadId::new();
    assert_eq!(taint_for(thread), TaintFacts::NotObserved);
    let ingress = Arc::new(Mutex::new(NativeIngress::default()));
    register_ingress(thread, &ingress);
    assert_eq!(taint_for(thread), TaintFacts::Off);

    ingress.lock().unwrap().set_labelled_mode(/*enabled*/ true);
    assert_eq!(taint_for(thread), TaintFacts::Generation(0));
    ingress.lock().unwrap().note_unrecorded_input();
    assert_eq!(taint_for(thread), TaintFacts::Generation(1));

    drop(ingress);
    assert_eq!(taint_for(thread), TaintFacts::NotObserved);
}
