//! #2213: client answers cannot expand the choices offered by an approval prompt.

use super::*;
use crate::session::tests::make_session_and_context_with_rx;
use crate::state::ActiveTurn;
use crate::tools::sandboxing::with_cached_approval;
use pretty_assertions::assert_eq;
use std::time::Duration;

const KEY: &str = "issue-2213-command";
const WAIT: Duration = Duration::from_secs(10);

async fn command_answer(
    additional_permissions: Option<AdditionalPermissionProfile>,
    offered: Option<Vec<ReviewDecision>>,
    proposed: Option<ExecPolicyAmendment>,
    answer: ReviewDecision,
) -> (ReviewDecision, Option<ReviewDecision>, String) {
    let (session, turn, events) = make_session_and_context_with_rx().await;
    *session.active_turn.lock().await = Some(ActiveTurn::default());
    let home = session.codex_home().await.to_path_buf();
    std::fs::create_dir_all(&home).unwrap();
    let ask = tokio::spawn({
        let session = Arc::clone(&session);
        let turn = Arc::clone(&turn);
        async move {
            with_cached_approval(
                &session.services,
                "shell",
                vec![KEY],
                /*fresh_human_authority*/ false,
                || async {
                    #[allow(deprecated)]
                    let cwd = turn.cwd.clone();
                    session
                        .request_command_approval(
                            &turn,
                            KEY.to_string(),
                            Some("issue-2213-subcommand".to_string()),
                            /*environment_id*/ None,
                            vec![KEY.to_string()],
                            cwd,
                            /*reason*/ None,
                            /*network_approval_context*/ None,
                            proposed,
                            additional_permissions,
                            offered,
                            /*plugin_attribution_override*/ None,
                        )
                        .await
                },
            )
            .await
        }
    });
    let event = tokio::time::timeout(WAIT, events.recv())
        .await
        .unwrap()
        .unwrap();
    let EventMsg::ExecApprovalRequest(prompt) = event.msg else {
        panic!("expected command approval");
    };
    handlers::exec_approval(
        &session,
        prompt.effective_approval_id(),
        Some(prompt.turn_id),
        answer,
    )
    .await;
    let decision = tokio::time::timeout(WAIT, ask).await.unwrap().unwrap();
    let cached = session.services.approval_cache().await.get(&KEY);
    let saved =
        std::fs::read_to_string(crate::exec_policy::default_policy_path(&home)).unwrap_or_default();
    (decision, cached, saved)
}

fn assert_refused(decision: ReviewDecision, cached: Option<ReviewDecision>, saved: String) {
    assert!(
        matches!(decision, ReviewDecision::Denied { .. }),
        "{decision:?}"
    );
    assert_eq!(cached, None);
    assert!(saved.is_empty(), "an unoffered rule must not be persisted");
}

#[tokio::test]
async fn issue_2213_command_rejects_unoffered_session_approval() {
    for additional in [None, Some(AdditionalPermissionProfile::default())] {
        let (decision, cached, saved) = command_answer(
            additional,
            /*offered*/ None,
            /*proposed*/ None,
            ReviewDecision::ApprovedForSession,
        )
        .await;
        assert_refused(decision, cached, saved);
    }
}

#[tokio::test]
async fn issue_2213_command_rejects_unoffered_once_approval() {
    let (decision, cached, saved) = command_answer(
        /*additional_permissions*/ None,
        Some(vec![ReviewDecision::Abort]),
        /*proposed*/ None,
        ReviewDecision::Approved,
    )
    .await;
    assert_refused(decision, cached, saved);
}

#[tokio::test]
async fn issue_2213_command_rejects_unoffered_amendment_before_saving() {
    let amendment = ExecPolicyAmendment::new(vec![KEY.to_string()]);
    for proposed in [
        None,
        Some(ExecPolicyAmendment::new(vec![
            KEY.to_string(),
            "narrow".to_string(),
        ])),
    ] {
        let (decision, cached, saved) = command_answer(
            /*additional_permissions*/ None,
            /*offered*/ None,
            proposed,
            ReviewDecision::ApprovedExecpolicyAmendment {
                proposed_execpolicy_amendment: amendment.clone(),
            },
        )
        .await;
        assert_refused(decision, cached, saved);
    }
}

#[tokio::test]
async fn issue_2213_command_preserves_offered_answers() {
    for answer in [ReviewDecision::Approved, ReviewDecision::ApprovedForSession] {
        let (decision, cached, saved) = command_answer(
            /*additional_permissions*/ None,
            Some(vec![answer.clone(), ReviewDecision::Abort]),
            /*proposed*/ None,
            answer.clone(),
        )
        .await;
        assert_eq!(decision, answer);
        assert_eq!(
            cached,
            (answer == ReviewDecision::ApprovedForSession).then_some(answer)
        );
        assert!(saved.is_empty());
    }
    let amendment = ExecPolicyAmendment::new(vec![KEY.to_string()]);
    let answer = ReviewDecision::ApprovedExecpolicyAmendment {
        proposed_execpolicy_amendment: amendment.clone(),
    };
    let (decision, cached, saved) = command_answer(
        /*additional_permissions*/ None,
        /*offered*/ None,
        Some(amendment),
        answer.clone(),
    )
    .await;
    assert_eq!(decision, answer);
    assert_eq!(cached, None);
    assert!(saved.contains(KEY));
}

#[tokio::test]
async fn issue_2213_patch_rejects_command_amendments_and_preserves_session_choice() {
    for answer in [
        ReviewDecision::ApprovedExecpolicyAmendment {
            proposed_execpolicy_amendment: ExecPolicyAmendment::new(vec![KEY.to_string()]),
        },
        ReviewDecision::NetworkPolicyAmendment {
            network_policy_amendment: NetworkPolicyAmendment {
                host: "unoffered.example".to_string(),
                action: NetworkPolicyRuleAction::Allow,
            },
        },
        ReviewDecision::Approved,
        ReviewDecision::ApprovedForSession,
    ] {
        let (session, turn, events) = make_session_and_context_with_rx().await;
        *session.active_turn.lock().await = Some(ActiveTurn::default());
        let ask = tokio::spawn({
            let session = Arc::clone(&session);
            async move {
                with_cached_approval(
                    &session.services,
                    "apply_patch",
                    vec![KEY],
                    /*fresh_human_authority*/ false,
                    || {
                        session.request_patch_approval(
                            &turn,
                            KEY.to_string(),
                            HashMap::new(),
                            /*reason*/ None,
                            /*grant_root*/ None,
                        )
                    },
                )
                .await
            }
        });
        let event = tokio::time::timeout(WAIT, events.recv())
            .await
            .unwrap()
            .unwrap();
        assert!(matches!(event.msg, EventMsg::ApplyPatchApprovalRequest(_)));
        handlers::patch_approval(&session, KEY.to_string(), answer.clone()).await;
        let decision = tokio::time::timeout(WAIT, ask).await.unwrap().unwrap();
        let cached = session.services.approval_cache().await.get(&KEY);
        match answer {
            ReviewDecision::Approved | ReviewDecision::ApprovedForSession => {
                assert_eq!(decision, answer);
                assert_eq!(
                    cached,
                    (answer == ReviewDecision::ApprovedForSession).then_some(answer)
                );
            }
            _ => assert_refused(decision, cached, String::new()),
        }
    }
}

#[tokio::test]
async fn issue_2213_permissions_never_cache_unrequested_access() {
    use codex_protocol::models::NetworkPermissions;
    use codex_protocol::request_permissions::RequestPermissionsArgs;

    for scope in [PermissionGrantScope::Turn, PermissionGrantScope::Session] {
        let (session, turn, events) = make_session_and_context_with_rx().await;
        *session.active_turn.lock().await = Some(ActiveTurn::default());
        let environment = turn.environments.primary().unwrap().selection();
        let environment_id = environment.environment_id.clone();
        let requested = RequestPermissionProfile {
            network: Some(NetworkPermissions {
                enabled: Some(false),
            }),
            ..Default::default()
        };
        let ask = tokio::spawn({
            let session = Arc::clone(&session);
            async move {
                session
                    .request_permissions_for_environment(
                        &turn,
                        KEY.to_string(),
                        RequestPermissionsArgs {
                            environment_id: None,
                            reason: None,
                            permissions: requested,
                        },
                        environment,
                        CancellationToken::new(),
                    )
                    .await
            }
        });
        let event = tokio::time::timeout(WAIT, events.recv())
            .await
            .unwrap()
            .unwrap();
        assert!(matches!(event.msg, EventMsg::RequestPermissions(_)));
        session
            .notify_request_permissions_response(
                KEY,
                RequestPermissionsResponse {
                    permissions: RequestPermissionProfile {
                        network: Some(NetworkPermissions {
                            enabled: Some(true),
                        }),
                        ..Default::default()
                    },
                    scope,
                    strict_auto_review: false,
                },
            )
            .await;
        let response = tokio::time::timeout(WAIT, ask)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        assert_ne!(
            response
                .permissions
                .network
                .as_ref()
                .and_then(|network| network.enabled),
            Some(true)
        );
        let stored = session.granted_session_permissions(&environment_id).await;
        assert_ne!(
            stored
                .and_then(|permissions| permissions.network)
                .and_then(|network| network.enabled),
            Some(true)
        );
    }
}
