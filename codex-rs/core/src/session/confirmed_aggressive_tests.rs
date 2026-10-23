use codex_protocol::config_types::ApprovalsReviewer;
use codex_protocol::protocol::AskForApproval;
use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::SessionSource;
use codex_protocol::protocol::SubAgentSource;
use pretty_assertions::assert_eq;

use super::super::session::SessionSettingsUpdate;
use super::super::tests::make_session_and_context_with_rx;
use super::super::tests::make_session_configuration_for_tests;
use crate::config::Constrained;
use crate::security::confirmed_approvals::ApprovalsOutcome;

fn settings(
    configuration: &super::super::session::SessionConfiguration,
) -> (AskForApproval, ApprovalsReviewer) {
    (
        configuration.approval_policy.value(),
        configuration.approvals_reviewer,
    )
}

const HELD: (AskForApproval, ApprovalsReviewer) =
    (AskForApproval::UnlessTrusted, ApprovalsReviewer::User);

/// #505: reviews, compaction and memory sessions keep their own settings;
/// child agents are covered.
#[tokio::test]
async fn issue_505_internal_sessions_keep_their_settings() {
    for (source, covered) in [
        (SessionSource::SubAgent(SubAgentSource::Review), false),
        (SessionSource::SubAgent(SubAgentSource::Compact), false),
        (
            SessionSource::SubAgent(SubAgentSource::MemoryConsolidation),
            false,
        ),
        (
            SessionSource::SubAgent(SubAgentSource::Other("guardian".to_string())),
            false,
        ),
        (
            SessionSource::SubAgent(SubAgentSource::ThreadSpawn {
                parent_thread_id: codex_protocol::ThreadId::new(),
                depth: 1,
                agent_path: None,
                agent_nickname: None,
                agent_role: None,
                agent_class: None,
            }),
            true,
        ),
        (SessionSource::Cli, true),
    ] {
        let mut configuration = make_session_configuration_for_tests().await;
        configuration.approval_policy = Constrained::allow_any(AskForApproval::Never);
        configuration.session_source = source.clone();
        let outcome = configuration.require_approval_for_every_command();
        if covered {
            assert_eq!(outcome, ApprovalsOutcome::Held, "{source:?}");
            assert_eq!(settings(&configuration), HELD, "{source:?}");
        } else {
            assert_eq!(outcome, ApprovalsOutcome::NotCovered, "{source:?}");
            assert_eq!(
                configuration.approval_policy.value(),
                AskForApproval::Never,
                "{source:?}"
            );
        }
    }
}

/// #505 (review r1): a managed approval-policy requirement is not
/// overridden, and the reviewer is not changed either.
#[tokio::test]
async fn issue_505_a_managed_approval_policy_is_kept() {
    let mut configuration = make_session_configuration_for_tests().await;
    configuration.approval_policy = Constrained::allow_only(AskForApproval::OnRequest);
    configuration.approvals_reviewer = ApprovalsReviewer::AutoReview;
    let outcome = configuration.require_approval_for_every_command();
    assert!(matches!(outcome, ApprovalsOutcome::Kept(_)), "{outcome:?}");
    assert_eq!(
        settings(&configuration),
        (AskForApproval::OnRequest, ApprovalsReviewer::AutoReview)
    );
    assert!(!configuration.aggressive_approvals);
}

/// #505: once held, a settings update (a client replaying its earlier
/// values with a turn) cannot lower it; other settings still change.
#[tokio::test]
async fn issue_505_settings_updates_cannot_lower_held_approvals() {
    let mut configuration = make_session_configuration_for_tests().await;
    configuration.approval_policy = Constrained::allow_any(AskForApproval::Never);
    assert_eq!(
        configuration.require_approval_for_every_command(),
        ApprovalsOutcome::Held
    );
    let updated = configuration
        .apply(&SessionSettingsUpdate {
            approval_policy: Some(AskForApproval::Never),
            approvals_reviewer: Some(ApprovalsReviewer::AutoReview),
            personality: Some(codex_protocol::config_types::Personality::Pragmatic),
            ..Default::default()
        })
        .expect("update");
    assert_eq!(settings(&updated), HELD);
    assert_eq!(
        updated.personality,
        Some(codex_protocol::config_types::Personality::Pragmatic)
    );
}

/// #505: a running session takes it in its stored settings: the next turn,
/// MCP tool approvals and clients all see it.
#[tokio::test]
async fn issue_505_a_running_session_stores_it_for_turns_mcp_and_clients() {
    let (session, _turn, rx) = make_session_and_context_with_rx().await;
    session
        .update_settings(SessionSettingsUpdate {
            approval_policy: Some(AskForApproval::Never),
            approvals_reviewer: Some(ApprovalsReviewer::AutoReview),
            ..Default::default()
        })
        .await
        .expect("permissive settings");
    while rx.try_recv().is_ok() {}

    assert_eq!(
        session.require_approval_for_every_command().await,
        ApprovalsOutcome::Held
    );

    let applied = loop {
        let event = rx.recv().await.expect("event");
        if let EventMsg::ThreadSettingsApplied(applied) = event.msg {
            break applied;
        }
    };
    assert_eq!(
        (
            applied.thread_settings.approval_policy,
            applied.thread_settings.approvals_reviewer
        ),
        HELD,
        "clients are told"
    );
    let turn = session.new_default_turn().await;
    assert_eq!(
        (turn.approval_policy.value(), turn.config.approvals_reviewer),
        HELD
    );
    let mcp = session.latest_mcp_desired_state(/*auth*/ None).await;
    assert_eq!(
        (
            mcp.config.permissions.approval_policy.value(),
            mcp.config.approvals_reviewer
        ),
        HELD,
        "MCP tool approvals follow"
    );
    // Asking again changes nothing and tells nobody again.
    assert_eq!(
        session.require_approval_for_every_command().await,
        ApprovalsOutcome::Held
    );
    assert!(rx.try_recv().is_err());
}
