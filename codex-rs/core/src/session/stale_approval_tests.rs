//! #945: answers to prompts shown before a security transition are for a
//! stale prompt. A "don't ask again" command rule is not saved (it would be
//! marked as approved under the new level, #626) and the answer approves the
//! command once; a permission request grants nothing.

use super::*;
use crate::session::tests::make_session_and_context_with_rx;
use crate::state::ActiveTurn;
use codex_security_policy::PolicyPrincipal;
use codex_security_policy::PrincipalKind;
use codex_security_policy::RevocationState;
use codex_security_policy::SecurityLevel;
use codex_security_policy::SecuritySettings;
use pretty_assertions::assert_eq;
use std::time::Duration;

const TOOL: &str = "issue-945-tool";

fn initialize_policy(session: &Session) -> crate::security::TrustedSecurityController {
    crate::security::TrustedSecurityController::initialize(
        &session.services.agent_control.effective_security_policy(),
        crate::security::PersistedHumanSecurityState::new(
            SecuritySettings::new(SecurityLevel::Permissive),
            PolicyPrincipal::new(PrincipalKind::Human, "human:issue-945").unwrap(),
            RevocationState::new(),
        )
        .unwrap(),
        session.thread_id,
        codex_protocol::SessionId::from(session.thread_id),
        crate::security::EffectivePolicyInitialization::Root,
    )
    .expect("initialize the session's policy tree")
}

fn raise_level(controller: &crate::security::TrustedSecurityController, level: SecurityLevel) {
    let change = controller
        .confirm_level_change(level, RevocationState::new())
        .unwrap();
    controller.apply_confirmed_change(change).unwrap();
}

async fn prompt_shown(events: &async_channel::Receiver<Event>) -> String {
    tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            let event = events.recv().await.expect("event channel open");
            if let EventMsg::ExecApprovalRequest(prompt) = event.msg {
                return prompt.effective_approval_id();
            }
        }
    })
    .await
    .expect("the prompt was shown")
}

fn saved_rules(codex_home: &std::path::Path) -> String {
    std::fs::read_to_string(crate::exec_policy::default_policy_path(codex_home)).unwrap_or_default()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_945_dont_ask_again_after_a_level_change_is_not_saved() {
    let (session, turn_context, events) = make_session_and_context_with_rx().await;
    let controller = initialize_policy(&session);
    *session.active_turn.lock().await = Some(ActiveTurn::default());
    let codex_home = session.codex_home().await.to_path_buf();
    // The test home may be gone; the rules file must be writable here.
    std::fs::create_dir_all(&codex_home).unwrap();
    let command = vec![TOOL.to_string(), "run".to_string()];
    let amendment = ExecPolicyAmendment::new(command.clone());
    let ask = |call_id: &'static str| {
        let session = Arc::clone(&session);
        let turn_context = Arc::clone(&turn_context);
        let command = command.clone();
        let amendment = amendment.clone();
        tokio::spawn(async move {
            #[allow(deprecated)]
            let cwd = turn_context.cwd.clone();
            session
                .request_command_approval(
                    turn_context.as_ref(),
                    call_id.to_string(),
                    /*approval_id*/ None,
                    /*environment_id*/ None,
                    command,
                    cwd,
                    /*reason*/ None,
                    /*network_approval_context*/ None,
                    Some(amendment),
                    /*additional_permissions*/ None,
                    /*available_decisions*/ None,
                    /*plugin_attribution_override*/ None,
                )
                .await
        })
    };
    let dont_ask_again = ReviewDecision::ApprovedExecpolicyAmendment {
        proposed_execpolicy_amendment: amendment.clone(),
    };

    // Shown under Permissive, answered after the level rose.
    let approval = ask("call-945-stale");
    let id = prompt_shown(&events).await;
    raise_level(&controller, SecurityLevel::Aggressive);
    handlers::exec_approval(&session, id, /*turn_id*/ None, dont_ask_again.clone()).await;
    assert_eq!(approval.await.unwrap(), ReviewDecision::Approved);
    assert!(
        !saved_rules(&codex_home).contains(TOOL),
        "no rule was saved"
    );
    let warned = std::iter::from_fn(|| events.try_recv().ok()).any(|event| {
        matches!(
            event.msg,
            EventMsg::Warning(WarningEvent { message }) if message.contains("security level changed")
        )
    });
    assert!(warned, "the person is told why");

    // Shown and answered under the level in force: saved as before.
    let approval = ask("call-945-current");
    let id = prompt_shown(&events).await;
    handlers::exec_approval(&session, id, /*turn_id*/ None, dont_ask_again.clone()).await;
    assert_eq!(approval.await.unwrap(), dont_ask_again);
    let warnings: Vec<String> = std::iter::from_fn(|| events.try_recv().ok())
        .filter_map(|event| match event.msg {
            EventMsg::Warning(WarningEvent { message }) => Some(message),
            _ => None,
        })
        .collect();
    assert!(saved_rules(&codex_home).contains(TOOL), "{warnings:?}");
}

/// #945: a "don't ask again" answer with no open prompt (its turn ended)
/// saves nothing.
#[tokio::test]
async fn issue_945_dont_ask_again_without_an_open_prompt_is_not_saved() {
    let (session, _turn_context, _events) = make_session_and_context_with_rx().await;
    initialize_policy(&session);
    *session.active_turn.lock().await = Some(ActiveTurn::default());
    let codex_home = session.codex_home().await.to_path_buf();
    std::fs::create_dir_all(&codex_home).unwrap();
    handlers::exec_approval(
        &session,
        "call-945-gone".to_string(),
        /*turn_id*/ None,
        ReviewDecision::ApprovedExecpolicyAmendment {
            proposed_execpolicy_amendment: ExecPolicyAmendment::new(vec![TOOL.to_string()]),
        },
    )
    .await;
    assert!(!saved_rules(&codex_home).contains(TOOL));
}

/// #945: a permission request answered after the level rose grants nothing,
/// for the session or the turn; one answered under the level in force does.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_945_permission_grant_after_a_level_change_grants_nothing() {
    use codex_protocol::request_permissions::PermissionGrantScope;
    use codex_protocol::request_permissions::RequestPermissionProfile;
    use codex_protocol::request_permissions::RequestPermissionsArgs;
    use codex_protocol::request_permissions::RequestPermissionsResponse;

    let (session, turn_context, events) = make_session_and_context_with_rx().await;
    let controller = initialize_policy(&session);
    *session.active_turn.lock().await = Some(ActiveTurn::default());
    let environment = turn_context
        .environments
        .primary()
        .expect("a primary environment")
        .selection();
    let environment_id = environment.environment_id.clone();
    let network = RequestPermissionProfile {
        network: Some(codex_protocol::models::NetworkPermissions {
            enabled: Some(true),
        }),
        ..RequestPermissionProfile::default()
    };
    let ask = |call_id: &'static str| {
        let session = Arc::clone(&session);
        let turn_context = Arc::clone(&turn_context);
        let environment = environment.clone();
        let network = network.clone();
        tokio::spawn(async move {
            session
                .request_permissions_for_environment(
                    &turn_context,
                    call_id.to_string(),
                    RequestPermissionsArgs {
                        environment_id: None,
                        reason: Some("issue 945".to_string()),
                        permissions: network,
                    },
                    environment,
                    CancellationToken::new(),
                )
                .await
        })
    };
    let request_shown = async || {
        tokio::time::timeout(Duration::from_secs(10), async {
            loop {
                let event = events.recv().await.expect("event channel open");
                if let EventMsg::RequestPermissions(request) = event.msg {
                    return request.call_id;
                }
            }
        })
        .await
        .expect("the request was shown")
    };
    let grant = |permissions: RequestPermissionProfile| RequestPermissionsResponse {
        permissions,
        scope: PermissionGrantScope::Session,
        strict_auto_review: false,
    };

    // Shown under Permissive, answered after the level rose.
    let request = ask("call-945-grant-stale");
    let call_id = request_shown().await;
    raise_level(&controller, SecurityLevel::Moderate);
    session
        .notify_request_permissions_response(&call_id, grant(network.clone()))
        .await;
    let response = request.await.unwrap().expect("a response");
    assert!(response.permissions.is_empty(), "{response:?}");
    assert_eq!(
        session.granted_session_permissions(&environment_id).await,
        None
    );
    assert_eq!(
        session.granted_turn_permissions(&environment_id).await,
        None
    );

    // Shown and answered under the level in force: granted as before.
    let request = ask("call-945-grant-current");
    let call_id = request_shown().await;
    session
        .notify_request_permissions_response(&call_id, grant(network.clone()))
        .await;
    let response = request.await.unwrap().expect("a response");
    assert_eq!(response, grant(network.clone()));
    assert!(
        session
            .granted_session_permissions(&environment_id)
            .await
            .is_some()
    );
}

/// D51 (#1001): permission grants made before a security transition end at
/// it, for the session and the turn, like hosts approved "for session"; a
/// grant answered before it is not recorded after it. Grants made under the
/// new level hold.
#[tokio::test]
async fn issue_1001_permission_grants_end_at_a_level_change() {
    let (session, turn_context, _events) = make_session_and_context_with_rx().await;
    let controller = initialize_policy(&session);
    *session.active_turn.lock().await = Some(ActiveTurn::default());
    let turn_state = {
        let active = session.active_turn.lock().await;
        Arc::clone(&active.as_ref().expect("active turn").turn_state)
    };
    let environment_id = turn_context
        .environments
        .primary()
        .expect("a primary environment")
        .selection()
        .environment_id;
    let network = RequestPermissionProfile {
        network: Some(codex_protocol::models::NetworkPermissions {
            enabled: Some(true),
        }),
        ..RequestPermissionProfile::default()
    };
    let grant = |scope| RequestPermissionsResponse {
        permissions: network.clone(),
        scope,
        strict_auto_review: false,
    };
    let record = async |scope, granted_under| {
        session
            .record_granted_request_permissions_for_turn(
                &grant(scope),
                &environment_id,
                Some(&turn_state),
                granted_under,
            )
            .await;
    };
    let held = async || {
        (
            session.granted_session_permissions(&environment_id).await,
            session.granted_turn_permissions(&environment_id).await,
        )
    };
    let granted: Option<codex_protocol::models::AdditionalPermissionProfile> =
        Some(network.clone().into());

    // Granted under Permissive, for the session and the turn.
    let permissive = session.services.authority_marker();
    record(PermissionGrantScope::Session, permissive).await;
    record(PermissionGrantScope::Turn, permissive).await;
    assert_eq!(held().await, (granted.clone(), granted.clone()));

    // The level rises: both end.
    raise_level(&controller, SecurityLevel::Moderate);
    assert_eq!(held().await, (None, None));

    // Answered under Permissive, recorded after the change: not kept.
    record(PermissionGrantScope::Session, permissive).await;
    record(PermissionGrantScope::Turn, permissive).await;
    assert_eq!(held().await, (None, None));

    // Granted under the level in force: kept as before.
    let moderate = session.services.authority_marker();
    record(PermissionGrantScope::Session, moderate).await;
    record(PermissionGrantScope::Turn, moderate).await;
    assert_eq!(held().await, (granted.clone(), granted));
}

/// D51: strict auto-review, turned on with a turn grant, ends with it at a
/// security transition, and is not turned on by a grant answered before one.
#[tokio::test]
async fn issue_1001_strict_auto_review_ends_with_its_grant() {
    let (session, turn_context, _events) = make_session_and_context_with_rx().await;
    let controller = initialize_policy(&session);
    *session.active_turn.lock().await = Some(ActiveTurn::default());
    let turn_state = {
        let active = session.active_turn.lock().await;
        Arc::clone(&active.as_ref().expect("active turn").turn_state)
    };
    let environment_id = turn_context
        .environments
        .primary()
        .expect("a primary environment")
        .selection()
        .environment_id;
    let strict_grant = RequestPermissionsResponse {
        permissions: RequestPermissionProfile {
            network: Some(codex_protocol::models::NetworkPermissions {
                enabled: Some(true),
            }),
            ..RequestPermissionProfile::default()
        },
        scope: PermissionGrantScope::Turn,
        strict_auto_review: true,
    };
    let record = async |granted_under| {
        session
            .record_granted_request_permissions_for_turn(
                &strict_grant,
                &environment_id,
                Some(&turn_state),
                granted_under,
            )
            .await;
    };

    let permissive = session.services.authority_marker();
    record(permissive).await;
    assert!(session.strict_auto_review_enabled_for_turn().await);
    raise_level(&controller, SecurityLevel::Moderate);
    assert!(!session.strict_auto_review_enabled_for_turn().await);
    // Answered under Permissive, recorded after the change: no grant, no
    // strict auto-review.
    record(permissive).await;
    assert!(!session.strict_auto_review_enabled_for_turn().await);
    record(session.services.authority_marker()).await;
    assert!(session.strict_auto_review_enabled_for_turn().await);
}

/// D52 (#1001): a host allowed for good while Aggressive is in force is saved
/// marked, so it applies under Aggressive; one allowed under a less strict
/// level is saved unmarked; one answered before a transition saves nothing.
#[tokio::test]
async fn issue_1001_host_allowed_for_good_is_marked_only_under_aggressive() {
    use codex_protocol::approvals::NetworkApprovalContext;
    use codex_protocol::approvals::NetworkApprovalProtocol;
    use codex_protocol::approvals::NetworkPolicyAmendment;
    use codex_protocol::approvals::NetworkPolicyRuleAction;

    let (session, _turn_context, _events) = make_session_and_context_with_rx().await;
    let controller = initialize_policy(&session);
    let codex_home = session.codex_home().await.to_path_buf();
    std::fs::create_dir_all(&codex_home).unwrap();
    let allow = async |host: &str, shown_under| {
        session
            .persist_network_policy_amendment(
                &NetworkPolicyAmendment {
                    host: host.to_string(),
                    action: NetworkPolicyRuleAction::Allow,
                },
                &NetworkApprovalContext {
                    host: host.to_string(),
                    protocol: NetworkApprovalProtocol::Https,
                },
                Some(shown_under),
                || {},
            )
            .await
    };
    let line_for = |host: &str| {
        saved_rules(&codex_home)
            .lines()
            .find(|line| line.contains(host))
            .map(str::to_string)
    };
    // The mark: `exec_policy::aggressive::APPROVED_UNDER_AGGRESSIVE`.
    let mark = "Remembered while security level Aggressive was active.";

    // Under Permissive: saved unmarked.
    allow("permissive.example", session.services.authority_marker())
        .await
        .expect("saved");
    let line = line_for("permissive.example").expect("saved");
    assert!(!line.contains(mark), "{line}");

    // Answered before a transition: nothing saved.
    let before = session.services.authority_marker();
    raise_level(&controller, SecurityLevel::Moderate);
    assert!(allow("stale.example", before).await.is_err());
    assert_eq!(line_for("stale.example"), None);

    // Under Aggressive: saved marked.
    let client = (*session.services.model_client())
        .clone()
        .with_ingress_level(SecurityLevel::Aggressive);
    session.services.replace_model_client(client);
    allow("aggressive.example", session.services.authority_marker())
        .await
        .expect("saved");
    let line = line_for("aggressive.example").expect("saved");
    assert!(line.contains(mark), "{line}");
}
