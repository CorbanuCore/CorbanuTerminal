use super::*;
use crate::config::Constrained;
use crate::exec_policy::ExecPolicyManager;
use crate::guardian::GUARDIAN_REVIEWER_NAME;
use crate::plugins::plugins_manager_for_config;
use crate::sandboxing::SandboxPermissions;
use crate::session::step_context::StepContext;
use crate::session::tests::update_turn_settings_for_test;
use crate::session::turn_context::NewTurnContextOptions;
use crate::test_support::models_manager_with_provider;
use crate::tools::context::ToolCallSource;
use crate::tools::context::ToolOutput;
use crate::tools::context::ToolPayload;
use crate::tools::handlers::PreparedPermissionRequest;
use crate::tools::orchestrator::ToolOrchestrator;
use crate::tools::sandboxing::Approvable;
use crate::tools::sandboxing::ApprovalAction;
use crate::tools::sandboxing::ExecApprovalRequirement;
use crate::tools::sandboxing::SandboxAttempt;
use crate::tools::sandboxing::Sandboxable;
use crate::tools::sandboxing::ToolCtx;
use crate::tools::sandboxing::ToolError;
use crate::tools::sandboxing::ToolRuntime;
use crate::turn_diff_tracker::TurnDiffTracker;
use codex_config::ConfigLayerEntry;
use codex_config::ConfigLayerSource;
use codex_config::ConfigRequirements;
use codex_config::ConfigRequirementsToml;
use codex_exec_server::EnvironmentManager;
use codex_execpolicy::Decision;
use codex_execpolicy::Evaluation;
use codex_execpolicy::Policy;
use codex_execpolicy::RuleMatch;
use codex_features::Feature;
use codex_model_provider::create_model_provider;
use codex_network_proxy::EnvironmentNetworkPolicy;
use codex_network_proxy::NetworkDecision;
use codex_network_proxy::NetworkDecisionSource;
use codex_network_proxy::NetworkPolicyDecision;
use codex_network_proxy::NetworkPolicyRequest;
use codex_network_proxy::NetworkProtocol;
use codex_protocol::config_types::ApprovalsReviewer;
use codex_protocol::error::SandboxErr;
use codex_protocol::models::AdditionalPermissionProfile as PermissionProfile;
use codex_protocol::models::ContentItemKind;
use codex_protocol::models::ImageReference;
use codex_protocol::models::NetworkPermissions;
use codex_protocol::models::ResponseInputItem;
use codex_protocol::network_policy::NetworkPolicyDecisionPayload;
use codex_protocol::protocol::AskForApproval;
use codex_protocol::request_permissions::PermissionGrantScope;
use codex_protocol::request_permissions::RequestPermissionProfile;
use codex_protocol::request_permissions::RequestPermissionsResponse;
use core_test_support::PathExt;
use core_test_support::TempDirExt;
use core_test_support::codex_linux_sandbox_exe_or_skip;
use core_test_support::responses::chat_completions_sse;
use core_test_support::responses::ev_assistant_message;
use core_test_support::responses::ev_completed;
use core_test_support::responses::ev_response_created;
use core_test_support::responses::mount_response_once_chat_completions;
use core_test_support::responses::mount_sse_once;
<<<<<<< HEAD
use core_test_support::responses::mount_sse_once_chat_completions;
||||||| 413492cd6c
=======
use core_test_support::responses::mount_sse_sequence;
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
use core_test_support::responses::sse;
use core_test_support::responses::sse_response;
use core_test_support::responses::start_mock_server;
use pretty_assertions::assert_eq;
use std::fs;
use std::sync::Arc;
use std::time::Duration;
use tempfile::tempdir;
use test_case::test_case;
use tokio::time::timeout;
use tokio_util::sync::CancellationToken;

fn expect_text_output<T>(output: &T) -> String
where
    T: ToolOutput + ?Sized,
{
    let response = output.to_response_item(
        "call-guardian",
        &ToolPayload::Function {
            arguments: "{}".to_string(),
        },
    );
    match response {
        ResponseInputItem::FunctionCallOutput { output, .. }
        | ResponseInputItem::CustomToolCallOutput { output, .. } => {
            output.body.to_text().unwrap_or_default()
        }
        other => panic!("expected function output, got {other:?}"),
    }
}

async fn activate_turn_with_new_review_authority(session: &Arc<Session>) -> Arc<TurnContext> {
    let (current_turn, _) = session
        .new_turn_with_sub_id(
            "current-authority-turn".to_string(),
            SessionSettingsUpdate {
                step_settings: StepSettingsUpdate {
                    approval_policy: Some(AskForApproval::Never),
                    approvals_reviewer: Some(ApprovalsReviewer::AutoReview),
                    ..Default::default()
                },
                permission_profile: Some(codex_protocol::models::PermissionProfile::Disabled),
                ..Default::default()
            },
            NewTurnContextOptions::default(),
        )
        .await
        .expect("next turn should accept different approval authority");
    session
        .start_task(
            current_turn,
            Vec::new(),
            super::NeverEndingTask {
                kind: crate::state::TaskKind::Regular,
                listen_to_cancellation_token: true,
            },
        )
        .await;

    let (active_turn, _, _, _) = session
        .active_turn_context_and_strict_auto_review()
        .await
        .expect("next turn should have active review authority");
    assert_eq!(
        (
            active_turn.approval_policy(),
            active_turn.config.approvals_reviewer
        ),
        (AskForApproval::Never, ApprovalsReviewer::AutoReview)
    );
    active_turn
}

fn captured_step_with_user_reviewer(
    turn: &mut Arc<TurnContext>,
    admitted_policy: AskForApproval,
    captured_policy: AskForApproval,
) -> Arc<StepContext> {
    let config = Arc::make_mut(
        &mut Arc::get_mut(turn)
            .expect("turn should not be shared")
            .config,
    );
    config
        .permissions
        .approval_policy
        .set(admitted_policy)
        .expect("set admitted turn approval policy");
    config.approvals_reviewer = ApprovalsReviewer::AutoReview;

    let mut step = StepContext::for_test(Arc::clone(turn));
    let captured = Arc::get_mut(&mut step).expect("step context should not be shared");
    update_selected_settings_for_test(Arc::make_mut(&mut captured.settings), |selected| {
        selected
            .approval_policy
            .set(captured_policy)
            .expect("set captured approval policy");
        selected.approvals_reviewer = ApprovalsReviewer::User;
    });
    step
}

async fn next_exec_approval(
    events: &async_channel::Receiver<Event>,
) -> codex_protocol::protocol::ExecApprovalRequestEvent {
    timeout(Duration::from_secs(5), async {
        loop {
            if let EventMsg::ExecApprovalRequest(approval) =
                events.recv().await.expect("receive approval event").msg
            {
                break approval;
            }
        }
    })
    .await
    .expect("captured action should request user approval")
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn request_permissions_routes_to_guardian_when_reviewer_is_enabled() {
    let server = start_mock_server().await;
<<<<<<< HEAD
    // The default Ambient provider uses the Chat Completions wire API.
    let guardian_assessment = serde_json::json!({
        "risk_level": "low",
        "user_authorization": "high",
        "outcome": "allow",
        "rationale": "The request grants narrowly scoped network access for this turn.",
    })
    .to_string();
    let guardian_request_log = mount_sse_once_chat_completions(
||||||| 413492cd6c
    let guardian_request_log = mount_sse_once(
=======
    let guardian_request_log = mount_sse_sequence(
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        &server,
<<<<<<< HEAD
        chat_completions_sse("z-ai/glm-5.2", &guardian_assessment),
||||||| 413492cd6c
        sse(vec![
            ev_response_created("resp-guardian"),
            ev_assistant_message(
                "msg-guardian",
                &serde_json::json!({
                    "risk_level": "low",
                    "user_authorization": "high",
                    "outcome": "allow",
                    "rationale": "The request grants narrowly scoped network access for this turn.",
                })
                .to_string(),
            ),
            ev_completed("resp-guardian"),
        ]),
=======
        vec![
            sse(vec![
            ev_response_created("resp-guardian"),
            ev_assistant_message(
                "msg-guardian",
                &serde_json::json!({
                    "risk_level": "low",
                    "user_authorization": "high",
                    "outcome": "allow",
                    "rationale": "The request grants narrowly scoped network access for this turn.",
                })
                .to_string(),
            ),
            ev_completed("resp-guardian"),
        ]);
            2
        ],
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
    )
    .await;

<<<<<<< HEAD
    let (mut session, mut turn_context_raw, _codex_home) =
        make_session_and_context_with_provider_auth().await;
    *session.active_turn.lock().await = Some(ActiveTurn::default());
    turn_context_raw
||||||| 413492cd6c
    let (mut session, mut turn_context_raw) = make_session_and_context().await;
    *session.active_turn.lock().await = Some(ActiveTurn::default());
    turn_context_raw
=======
    let (mut session, mut turn_context_raw) = make_session_and_context().await;
    update_turn_settings_for_test(&mut turn_context_raw, |settings| {
        Arc::make_mut(&mut settings.model_info).node_repl_auto_review_required = true;
    });
    Arc::make_mut(&mut turn_context_raw.config)
        .permissions
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        .approval_policy
        .set(AskForApproval::OnRequest)
        .expect("test setup should allow updating approval policy");
    let mut config = (*turn_context_raw.config).clone();
    config
        .features
        .enable(Feature::GuardianApproval)
        .expect("test setup should allow enabling guardian approvals");
    config.approvals_reviewer = ApprovalsReviewer::AutoReview;
    config.model_provider.base_url = Some(format!("{}/v1", server.uri()));
    let config = Arc::new(config);
    let models_manager = models_manager_with_provider(
        config.codex_home.to_path_buf(),
        session.services.auth_manager(),
        config.model_provider.clone(),
    );
    session.services.models_manager = models_manager;
    crate::guardian::test_host::install(&session, &config);
    turn_context_raw.config = Arc::clone(&config);
    turn_context_raw.provider = create_model_provider(
        config.model_provider.clone(),
        turn_context_raw.auth_manager.clone(),
    );
    let image_url = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR4nGP4z8DwHwAFAAH/iZk9HQAAAABJRU5ErkJggg==";
    let evidence = session
        .services
        .thread_extension_data
        .get_or_init(crate::context::NodeReplReviewEvidence::default);
    let image = UserInput::Image {
        image: ImageReference::Inline {
            image_url: image_url.to_string(),
        },
        detail: None,
    };
    evidence.record("js", "cell", "image", vec![image]);
    let session = Arc::new(session);
    let turn_context = Arc::new(turn_context_raw);
    session
        .start_task(
            Arc::clone(&turn_context),
            Vec::new(),
            super::NeverEndingTask {
                kind: crate::state::TaskKind::Regular,
                listen_to_cancellation_token: true,
            },
        )
        .await;
    let step_context = StepContext::for_test(Arc::clone(&turn_context));

    let requested_permissions = RequestPermissionProfile {
        network: Some(NetworkPermissions {
            enabled: Some(true),
        }),
        ..RequestPermissionProfile::default()
    };
    let environment = turn_context
        .initial_environments
        .primary()
        .expect("primary environment")
        .selection();
    let response = tokio::time::timeout(
        Duration::from_secs(45),
        session.request_permissions_for_environment(
            &step_context,
            "perm-call-1".to_string(),
            PreparedPermissionRequest {
                environment_id: environment.environment_id.clone(),
                reason: Some("need network".to_string()),
                permissions: requested_permissions.clone(),
            },
            CancellationToken::new(),
        ),
    )
    .await
    .expect("request_permissions should not wait for a client approval");

    assert_eq!(
        response,
        Some(RequestPermissionsResponse {
            permissions: requested_permissions.clone(),
            scope: PermissionGrantScope::Turn,
            strict_auto_review: false,
        })
    );
    let second_response = session
        .request_permissions_for_environment(
            &step_context,
            "perm-call-2".to_string(),
            PreparedPermissionRequest {
                environment_id: environment.environment_id,
                reason: Some("need network".to_string()),
                permissions: requested_permissions.clone(),
            },
            CancellationToken::new(),
        )
        .await;
    assert_eq!(second_response, response);
    assert_eq!(
        turn_context.granted_permissions(codex_exec_server::LOCAL_ENVIRONMENT_ID),
        Some(requested_permissions.into())
    );

<<<<<<< HEAD
    let guardian_request = guardian_request_log.single_request();
    assert_eq!(guardian_request.path(), "/v1/chat/completions");
||||||| 413492cd6c
    let guardian_request = guardian_request_log.single_request();
    assert_eq!(guardian_request.path(), "/v1/responses");
=======
    let guardian_requests = guardian_request_log.requests();
    assert_eq!(guardian_requests.len(), 2);
    let guardian_request = &guardian_requests[0];
    assert_eq!(guardian_request.path(), "/v1/responses");
    for request in &guardian_requests {
        assert_eq!(request.message_input_image_urls("user"), [image_url]);
    }
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
    assert!(guardian_request.body_contains_text("request_permissions"));
    assert!(guardian_request.body_contains_text("need network"));
}

#[tokio::test]
<<<<<<< HEAD
async fn request_permissions_guardian_review_stops_when_cancelled() {
    // Generous caps so a loaded CI runner is not mistaken for a hang (#1539).
    // The guardian's model reply is held for 60 s, so a request that ignores
    // the cancellation runs into STOP_CAP instead of returning.
    const START_CAP: Duration = Duration::from_secs(30);
    const STOP_CAP: Duration = Duration::from_secs(30);

||||||| 413492cd6c
async fn request_permissions_guardian_review_stops_when_cancelled() {
=======
async fn request_permissions_uses_issuing_step_policy_and_reviewer() {
    let server = start_mock_server().await;
    let guardian_requests = mount_sse_once(
        &server,
        sse(vec![
            ev_assistant_message("guardian", r#"{"outcome":"allow"}"#),
            ev_completed("guardian-review"),
        ]),
    )
    .await;
    let (session, turn, _) = make_session_and_context_with_auth_and_config_and_rx(
        CodexAuth::from_api_key("Test API Key"),
        Vec::new(),
        |config| {
            config.permissions.approval_policy = Constrained::allow_any(AskForApproval::Never);
            config.approvals_reviewer = ApprovalsReviewer::User;
            config.model_provider.base_url = Some(format!("{}/v1", server.uri()));
            config.model_provider.supports_websockets = false;
            config
                .features
                .enable(Feature::GuardianApproval)
                .expect("enable Guardian");
        },
    )
    .await;
    *session.active_turn.lock().await = Some(ActiveTurn::default());
    let mut step = StepContext::for_test(turn);
    let captured = Arc::get_mut(&mut step).expect("unshared step");
    // The issuing step differs from the admitted Never/User turn.
    update_selected_settings_for_test(Arc::make_mut(&mut captured.settings), |selected| {
        selected
            .approval_policy
            .set(AskForApproval::OnRequest)
            .expect("set step policy");
        selected.approvals_reviewer = ApprovalsReviewer::AutoReview;
    });
    let permissions = RequestPermissionProfile {
        network: Some(NetworkPermissions {
            enabled: Some(true),
        }),
        ..Default::default()
    };

    let response = timeout(
        Duration::from_secs(5),
        session.request_permissions_for_environment(
            &step,
            "step-permissions".to_string(),
            PreparedPermissionRequest {
                environment_id: step
                    .environments
                    .primary()
                    .expect("primary environment")
                    .selection
                    .environment_id
                    .clone(),
                reason: Some("need network".to_string()),
                permissions: permissions.clone(),
            },
            CancellationToken::new(),
        ),
    )
    .await
    .expect("issuing step should route to Guardian without a user approval");

    assert_eq!(
        response,
        Some(RequestPermissionsResponse {
            permissions,
            scope: PermissionGrantScope::Turn,
            strict_auto_review: false,
        })
    );
    assert!(
        guardian_requests
            .single_request()
            .body_contains_text("request_permissions")
    );
}

#[derive(Clone, Copy)]
enum ReviewCancellationSource {
    Action,
    ParentShutdown,
}

#[test_case::test_case(ReviewCancellationSource::Action; "action")]
#[test_case::test_case(ReviewCancellationSource::ParentShutdown; "parent_shutdown")]
#[tokio::test]
async fn request_permissions_guardian_review_stops_when_cancelled(
    source: ReviewCancellationSource,
) {
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
    let server = start_mock_server().await;
    // The default Ambient provider reviews over Chat Completions (#1544).
    let guardian_assessment = serde_json::json!({
        "risk_level": "low",
        "user_authorization": "high",
        "outcome": "allow",
        "rationale": "The request grants narrowly scoped network access for this turn.",
    })
    .to_string();
    let guardian_request_log = mount_response_once_chat_completions(
        &server,
        sse_response(chat_completions_sse("z-ai/glm-5.2", &guardian_assessment))
            .set_delay(Duration::from_secs(60)),
    )
    .await;

<<<<<<< HEAD
    let (mut session, mut turn_context_raw, _codex_home) =
        make_session_and_context_with_provider_auth().await;
    *session.active_turn.lock().await = Some(ActiveTurn::default());
    turn_context_raw
||||||| 413492cd6c
    let (mut session, mut turn_context, rx_event) = make_session_and_context_with_rx().await;
    *session.active_turn.lock().await = Some(ActiveTurn::default());
    let turn_context_raw = Arc::get_mut(&mut turn_context).expect("single turn context ref");
    turn_context_raw
=======
    let (mut session, mut turn_context, rx_event) = make_session_and_context_with_rx().await;
    let active_turn = ActiveTurn::default();
    *session.active_turn.lock().await = Some(active_turn);
    let turn_context_raw = Arc::get_mut(&mut turn_context).expect("single turn context ref");
    Arc::make_mut(&mut turn_context_raw.config)
        .permissions
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        .approval_policy
        .set(AskForApproval::OnRequest)
        .expect("test setup should allow updating approval policy");
    let mut config = (*turn_context_raw.config).clone();
    config
        .features
        .enable(Feature::GuardianApproval)
        .expect("test setup should allow enabling guardian approvals");
    config.approvals_reviewer = ApprovalsReviewer::AutoReview;
    config.model_provider.base_url = Some(format!("{}/v1", server.uri()));
    let config = Arc::new(config);
    let models_manager = models_manager_with_provider(
        config.codex_home.to_path_buf(),
        session.services.auth_manager(),
        config.model_provider.clone(),
    );
<<<<<<< HEAD
    session.services.models_manager = models_manager;
||||||| 413492cd6c
    Arc::get_mut(&mut session)
        .expect("single session ref")
        .services
        .models_manager = models_manager;
=======
    Arc::get_mut(&mut session)
        .expect("single session ref")
        .services
        .models_manager = models_manager;
    crate::guardian::test_host::install(&session, &config);
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
    turn_context_raw.config = Arc::clone(&config);
    turn_context_raw.provider = create_model_provider(
        config.model_provider.clone(),
        turn_context_raw.auth_manager.clone(),
    );
    let session = Arc::new(session);
    let turn_context = Arc::new(turn_context_raw);

    let requested_permissions = RequestPermissionProfile {
        network: Some(NetworkPermissions {
            enabled: Some(true),
        }),
        ..RequestPermissionProfile::default()
    };
    let cancellation_token = CancellationToken::new();
    let request_handle = tokio::spawn({
        let session = Arc::clone(&session);
        let turn_context = Arc::clone(&turn_context);
        let requested_permissions = requested_permissions.clone();
        let cancellation_token = cancellation_token.clone();
        async move {
            let environment = turn_context
                .initial_environments
                .primary()
                .expect("primary environment")
                .selection();
            session
                .request_permissions_for_environment(
                    &StepContext::for_test(Arc::clone(&turn_context)),
                    "perm-call-cancelled".to_string(),
                    PreparedPermissionRequest {
                        environment_id: environment.environment_id,
                        reason: Some("need network".to_string()),
                        permissions: requested_permissions,
                    },
                    cancellation_token,
                )
                .await
        }
    });

    // Cancel only once the guardian's model call is in flight: the mock has
    // received it and is holding the reply.
    timeout(START_CAP, async {
        while guardian_request_log.requests().is_empty() {
            assert!(
                !request_handle.is_finished(),
                "request_permissions finished before the guardian model call"
            );
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("the guardian model call should start before cancellation");

    let reviewer_tasks = session
        .services
        .thread_extension_data
        .get::<codex_guardian_reviewer::ReviewerTasks>()
        .expect("reviewer tasks installed");
    match source {
        ReviewCancellationSource::Action => cancellation_token.cancel(),
        ReviewCancellationSource::ParentShutdown => reviewer_tasks.cancellation.cancel(),
    }

    let response = timeout(STOP_CAP, request_handle)
        .await
        .expect("request_permissions should stop when cancelled")
        .expect("request_permissions task should not panic");
    let expected_response = match source {
        ReviewCancellationSource::Action => None,
        ReviewCancellationSource::ParentShutdown => Some(RequestPermissionsResponse {
            permissions: RequestPermissionProfile::default(),
            scope: PermissionGrantScope::Turn,
            strict_auto_review: false,
        }),
    };
    assert_eq!(response, expected_response);
    if matches!(source, ReviewCancellationSource::ParentShutdown) {
        reviewer_tasks.tasks.close();
        timeout(Duration::from_secs(5), reviewer_tasks.tasks.wait())
            .await
            .expect("parent shutdown must finish reviewer cleanup");
    }
    assert_eq!(
        turn_context.granted_permissions(codex_exec_server::LOCAL_ENVIRONMENT_ID),
        None
    );
    let guardian_request = guardian_request_log.single_request();
    assert_eq!(guardian_request.path(), "/v1/chat/completions");
    assert!(guardian_request.body_contains_text("need network"));
}

#[tokio::test]
async fn guardian_allows_exec_command_additional_permissions_requests_past_policy_validation() {
    let server = start_mock_server().await;
    let _request_log = mount_sse_once(
        &server,
        sse(vec![
            ev_response_created("resp-guardian"),
            ev_assistant_message(
                "msg-guardian",
                &serde_json::json!({
                    "risk_level": "low",
                    "user_authorization": "high",
                    "outcome": "allow",
                    "rationale": "The request only widens permissions for a benign local echo command.",
                })
                .to_string(),
            ),
            ev_completed("resp-guardian"),
        ]),
    )
    .await;

<<<<<<< HEAD
    let (mut session, mut turn_context_raw, _codex_home) =
        make_session_and_context_with_provider_auth().await;
    turn_context_raw
||||||| 413492cd6c
    let (mut session, mut turn_context_raw) = make_session_and_context().await;
    turn_context_raw
=======
    let (mut session, mut turn_context_raw) = make_session_and_context().await;
    Arc::make_mut(&mut turn_context_raw.config)
        .permissions
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        .approval_policy
        .set(AskForApproval::OnRequest)
        .expect("test setup should allow updating approval policy");
    session
        .features
        .enable(Feature::ExecPermissionApprovals)
        .expect("test setup should allow enabling request permissions");
    let mut config = (*turn_context_raw.config).clone();
    config
        .permissions
        .set_permission_profile(codex_protocol::models::PermissionProfile::Disabled)
        .expect("test setup should allow disabling the permission profile");
    let TurnEnvironmentState::Ready(environment) =
        &mut turn_context_raw.initial_environments.environments[0]
    else {
        panic!("primary environment should be ready");
    };
    environment.config_mut().permission_profile =
        config.permissions.permission_profile_state().snapshot();
    config.codex_linux_sandbox_exe = codex_linux_sandbox_exe_or_skip!();
    config
        .features
        .enable(Feature::GuardianApproval)
        .expect("test setup should allow enabling guardian approvals");
    config.model_provider.base_url = Some(format!("{}/v1", server.uri()));
    let config = Arc::new(config);
    let models_manager = models_manager_with_provider(
        config.codex_home.to_path_buf(),
        session.services.auth_manager(),
        config.model_provider.clone(),
    );
    session.services.models_manager = models_manager;
    crate::guardian::test_host::install(&session, &config);
    turn_context_raw.config = Arc::clone(&config);
    turn_context_raw.provider = create_model_provider(
        config.model_provider.clone(),
        turn_context_raw.auth_manager.clone(),
    );
    let session = Arc::new(session);
    let turn_context = Arc::new(turn_context_raw);
<<<<<<< HEAD
    // `echo hi` can take seconds to start on a loaded Windows runner (#636).
    let expiration_ms: u64 = if cfg!(windows) { 10_000 } else { 1_000 };
||||||| 413492cd6c
    let expiration_ms: u64 = if cfg!(windows) { 2_500 } else { 1_000 };
=======
    let yield_time_ms: u64 = 10_000;
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a

    let handler = crate::tools::handlers::ExecCommandHandler::default();
    #[allow(deprecated)]
    let workdir = Some(turn_context.cwd.to_string_lossy().to_string());
    let step_context = StepContext::for_test(Arc::clone(&turn_context));
    let resp = handler
        .handle(ToolInvocation {
            session: Arc::clone(&session),
            turn: Arc::clone(&turn_context),
            step_context,
            cancellation_token: CancellationToken::new(),
            tracker: Arc::new(tokio::sync::Mutex::new(TurnDiffTracker::new())),
            call_id: "test-call".to_string(),
            tool_name: codex_tools::ToolName::plain("exec_command"),
            source: crate::tools::context::ToolCallSource::Direct,
            native_call: None,
            environments: Default::default(),
            payload: ToolPayload::Function {
                arguments: serde_json::json!({
                    "cmd": "echo hi",
                    "login": false,
                    "workdir": workdir,
                    "yield_time_ms": yield_time_ms,
                    "sandbox_permissions": SandboxPermissions::WithAdditionalPermissions,
                    "additional_permissions": PermissionProfile {
                        network: Some(NetworkPermissions {
                            enabled: Some(true),
                        }),
                        file_system: None,
                    },
                    "justification": Some("test"),
                })
                .to_string(),
            },
        })
        .await;

    let output = expect_text_output(&resp.expect("expected Ok result"));
    assert!(output.contains("hi"));
}

#[tokio::test]
async fn strict_auto_review_turn_grant_forces_guardian_for_exec_command_policy_skip() {
    let server = start_mock_server().await;
    // The default Ambient provider uses the Chat Completions wire API.
    let guardian_assessment = serde_json::json!({
        "risk_level": "low",
        "user_authorization": "high",
        "outcome": "allow",
        "rationale": "The command stays within the strict turn permission grant.",
    })
    .to_string();
    let guardian_request_log = mount_sse_once_chat_completions(
        &server,
        chat_completions_sse("z-ai/glm-5.2", &guardian_assessment),
    )
    .await;

<<<<<<< HEAD
    let (mut session, mut turn_context_raw, _codex_home) =
        make_session_and_context_with_provider_auth().await;
    let active_turn = crate::state::ActiveTurn::default();
    let originating_turn_state = Arc::clone(&active_turn.turn_state);
    *session.active_turn.lock().await = Some(active_turn);
    session
        .record_granted_request_permissions_for_turn(
            &RequestPermissionsResponse {
                permissions: RequestPermissionProfile {
                    network: Some(NetworkPermissions {
                        enabled: Some(true),
                    }),
                    ..Default::default()
                },
                scope: PermissionGrantScope::Turn,
                strict_auto_review: true,
||||||| 413492cd6c
    let (mut session, mut turn_context_raw) = make_session_and_context().await;
    let active_turn = crate::state::ActiveTurn::default();
    let originating_turn_state = Arc::clone(&active_turn.turn_state);
    *session.active_turn.lock().await = Some(active_turn);
    session
        .record_granted_request_permissions_for_turn(
            &RequestPermissionsResponse {
                permissions: RequestPermissionProfile {
                    network: Some(NetworkPermissions {
                        enabled: Some(true),
                    }),
                    ..Default::default()
                },
                scope: PermissionGrantScope::Turn,
                strict_auto_review: true,
=======
    let (mut session, mut turn_context_raw) = make_session_and_context().await;
    session.record_granted_request_permissions_for_turn(
        &RequestPermissionsResponse {
            permissions: RequestPermissionProfile {
                network: Some(NetworkPermissions {
                    enabled: Some(true),
                }),
                ..Default::default()
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
            },
<<<<<<< HEAD
            codex_exec_server::LOCAL_ENVIRONMENT_ID,
            Some(&originating_turn_state),
            session.services.authority_marker(),
        )
        .await;
||||||| 413492cd6c
            codex_exec_server::LOCAL_ENVIRONMENT_ID,
            Some(&originating_turn_state),
        )
        .await;
=======
            scope: PermissionGrantScope::Turn,
            strict_auto_review: true,
        },
        codex_exec_server::LOCAL_ENVIRONMENT_ID,
        &turn_context_raw,
    );
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a

    Arc::make_mut(&mut turn_context_raw.config)
        .permissions
        .approval_policy
        .set(AskForApproval::Never)
        .expect("test setup should allow updating approval policy");
    let mut config = (*turn_context_raw.config).clone();
    // Keep Never outside Full Access without requiring an OS sandbox for this routing test.
    config
        .permissions
        .set_permission_profile(codex_protocol::models::PermissionProfile::External {
            network: NetworkSandboxPolicy::Restricted,
        })
        .expect("test setup should allow external sandbox permissions");
    let TurnEnvironmentState::Ready(environment) =
        &mut turn_context_raw.initial_environments.environments[0]
    else {
        panic!("primary environment should be ready");
    };
    environment.config_mut().permission_profile =
        config.permissions.permission_profile_state().snapshot();
    config.approvals_reviewer = ApprovalsReviewer::User;
    config.model_provider.base_url = Some(format!("{}/v1", server.uri()));
    let config = Arc::new(config);
    let models_manager = models_manager_with_provider(
        config.codex_home.to_path_buf(),
        session.services.auth_manager(),
        config.model_provider.clone(),
    );
    session.services.models_manager = models_manager;
    crate::guardian::test_host::install(&session, &config);
    turn_context_raw.config = Arc::clone(&config);
    turn_context_raw.provider = create_model_provider(
        config.model_provider.clone(),
        turn_context_raw.auth_manager.clone(),
    );
    let session = Arc::new(session);
    let turn_context = Arc::new(turn_context_raw);
    session
        .start_task(
            Arc::clone(&turn_context),
            Vec::new(),
            super::NeverEndingTask {
                kind: crate::state::TaskKind::Regular,
                listen_to_cancellation_token: true,
            },
        )
        .await;

    let handler = crate::tools::handlers::ExecCommandHandler::default();
    #[allow(deprecated)]
    let workdir = Some(turn_context.cwd.to_string_lossy().to_string());
    let step_context = StepContext::for_test(Arc::clone(&turn_context));
    let resp = handler
        .handle(ToolInvocation {
            session: Arc::clone(&session),
            turn: Arc::clone(&turn_context),
            step_context,
            cancellation_token: CancellationToken::new(),
            tracker: Arc::new(tokio::sync::Mutex::new(TurnDiffTracker::new())),
            call_id: "strict-shell-command-call".to_string(),
            tool_name: codex_tools::ToolName::plain("exec_command"),
            source: ToolCallSource::Direct,
            native_call: None,
            environments: Default::default(),
            payload: ToolPayload::Function {
                arguments: serde_json::json!({
                    "cmd": "echo hi",
                    "login": false,
                    "workdir": workdir,
<<<<<<< HEAD
                    "timeout_ms": if cfg!(windows) { 10_000_u64 } else { 1_000_u64 },
||||||| 413492cd6c
                    "timeout_ms": 1_000_u64,
=======
                    "yield_time_ms": 10_000_u64,
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
                })
                .to_string(),
            },
        })
        .await;

    let output = expect_text_output(&resp.expect("expected Ok result"));
    assert!(output.contains("hi"));
    let guardian_request = guardian_request_log.single_request();
    assert!(guardian_request.body_contains_text("echo hi"));
}

#[test_case(AskForApproval::Never; "policy_precheck")]
#[test_case(AskForApproval::OnRequest; "reviewer_routing")]
#[tokio::test]
async fn network_approval_uses_published_task_authority_within_same_turn(
    admitted_policy: AskForApproval,
) {
    let (session, turn, events) = make_session_and_context_with_auth_and_config_and_rx(
        CodexAuth::from_api_key("Test API Key"),
        Vec::new(),
        move |config| {
            config.permissions.approval_policy = Constrained::allow_any(admitted_policy);
            config.approvals_reviewer = ApprovalsReviewer::AutoReview;
            config
                    .permissions
                    .set_permission_profile(
                        codex_protocol::models::PermissionProfile::workspace_write(),
                    )
                    .expect("set managed permissions");
        },
    )
    .await;
    session
        .start_task(
            Arc::clone(&turn),
            Vec::new(),
            super::NeverEndingTask {
                kind: crate::state::TaskKind::Regular,
                listen_to_cancellation_token: true,
            },
        )
        .await;
    // Inject later-step authority directly while live policy changes remain gated.
    {
        let active = session.active_turn.lock().await;
        let task = active
            .as_ref()
            .expect("active turn")
            .task
            .as_ref()
            .expect("active task");
        let mut settings = task.turn_context.next_step_settings.load_full();
        update_selected_settings_for_test(Arc::make_mut(&mut settings), |selected| {
            selected
                .approval_policy
                .set(AskForApproval::OnRequest)
                .expect("update policy");
            selected.approvals_reviewer = ApprovalsReviewer::User;
        });
        task.turn_context.next_step_settings.store(settings);
    }
    let decision = session
        .services
        .network_approval
        .handle_inline_policy_request(
            Arc::clone(&session),
            NetworkPolicyRequest {
                protocol: NetworkProtocol::Http,
                host: "example.com".to_string(),
                port: 80,
                environment_id: None,
                client_addr: None,
                method: None,
                command: None,
                exec_policy_hint: None,
                execution_id: None,
                disconnect: None,
                cancellation: None,
            },
        );
    tokio::pin!(decision);
    let approval = timeout(Duration::from_secs(5), async {
        loop {
            tokio::select! {
                result = &mut decision => panic!("expected user network approval, got {result:?}"),
                event = events.recv() => {
                    match event.expect("approval event").msg {
                        EventMsg::ExecApprovalRequest(approval) => break approval,
                        EventMsg::GuardianAssessment(_) => panic!("expected the current user reviewer"),
                        _ => {}
                    }
                }
            }
        }
    })
    .await
    .expect("network approval requested");
    assert_eq!(approval.turn_id, turn.sub_id);
    session
        .notify_approval(&approval.call_id, ReviewDecision::Approved)
        .await;
    assert_eq!(
        timeout(Duration::from_secs(5), decision)
            .await
            .expect("network decision"),
        NetworkDecision::Allow
    );
    session.abort_all_tasks(TurnAbortReason::Interrupted).await;
}

#[tokio::test]
async fn delayed_exec_command_uses_its_captured_authority_after_next_turn_starts() {
    let (mut session, mut action_turn, events) = make_session_and_context_with_rx().await;
    // Windows can allow safe echo commands without prompting when its sandbox is disabled.
    let mut exec_policy = Policy::empty();
    exec_policy
        .add_prefix_rule(
            &["echo".to_string(), "captured-action-authority".to_string()],
            Decision::Prompt,
        )
        .expect("test command should require approval");
    Arc::get_mut(&mut session)
        .expect("session should not be shared")
        .services
        .exec_policy = Arc::new(ExecPolicyManager::new(Arc::new(exec_policy)));
    let step_context = captured_step_with_user_reviewer(
        &mut action_turn,
        AskForApproval::Never,
        AskForApproval::OnRequest,
    );
    let current_turn = activate_turn_with_new_review_authority(&session).await;
    assert_ne!(action_turn.sub_id, current_turn.sub_id);

    let call_id = "delayed-captured-authority-shell-command";
    let command = "echo captured-action-authority";
    let handler = crate::tools::handlers::ExecCommandHandler::default();
    let invocation = handler.handle(ToolInvocation {
        session: Arc::clone(&session),
        turn: Arc::clone(&action_turn),
        step_context,
        cancellation_token: CancellationToken::new(),
        tracker: Arc::new(tokio::sync::Mutex::new(TurnDiffTracker::new())),
        call_id: call_id.to_string(),
        tool_name: codex_tools::ToolName::plain("exec_command"),
        source: ToolCallSource::Direct,
        native_call: None,
        environments: Default::default(),
        payload: ToolPayload::Function {
            arguments: serde_json::json!({
                "cmd": command,
                "login": false,
                "sandbox_permissions": SandboxPermissions::RequireEscalated,
                "justification": "verify captured action authority",
            })
            .to_string(),
        },
    });
    let approve = async {
        let approval = next_exec_approval(&events).await;
        assert_eq!(approval.call_id, call_id);
        assert_eq!(approval.turn_id, action_turn.sub_id);
        assert!(approval.command.join(" ").contains(command));
        session
            .notify_approval(call_id, ReviewDecision::Approved)
            .await;
    };

    let (output, ()) = tokio::join!(invocation, approve);
    let output = output.expect("approved shell command should succeed");
    assert!(expect_text_output(output.as_ref()).contains("captured-action-authority"));
}

#[tokio::test]
async fn sandbox_denied_retry_uses_the_action_policy_and_reviewer() {
    #[derive(Default)]
    struct DeniedOnceRuntime {
        attempts: usize,
    }

    impl Approvable<TurnEnvironment> for DeniedOnceRuntime {
        fn exec_approval_requirement(
            &self,
            _request: &TurnEnvironment,
        ) -> Option<ExecApprovalRequirement> {
            Some(ExecApprovalRequirement::Skip {
                bypass_sandbox: false,
                proposed_execpolicy_amendment: None,
            })
        }

        fn approval_action(
            &self,
            request: &TurnEnvironment,
            call_id: &str,
        ) -> std::io::Result<ApprovalAction> {
            Ok(ApprovalAction::ExecCommand {
                id: call_id.to_string(),
                environment_id: codex_exec_server::LOCAL_ENVIRONMENT_ID.to_string(),
                command: vec!["echo".to_string(), "sandbox-retry".to_string()],
                hook_command: "echo sandbox-retry".to_string(),
                cwd: request.cwd().clone(),
                sandbox_permissions: SandboxPermissions::UseDefault,
                additional_permissions: None,
                justification: None,
                tty: false,
                proposed_execpolicy_amendment: None,
            })
        }
    }

    impl Sandboxable for DeniedOnceRuntime {
        fn sandbox_preference(&self) -> codex_sandboxing::SandboxablePreference {
            codex_sandboxing::SandboxablePreference::Auto
        }
    }

    impl ToolRuntime<TurnEnvironment, String> for DeniedOnceRuntime {
        fn turn_environment<'a>(&self, request: &'a TurnEnvironment) -> &'a TurnEnvironment {
            request
        }

        async fn run(
            &mut self,
            _request: &TurnEnvironment,
            _attempt: &SandboxAttempt<'_>,
            _context: &ToolCtx,
        ) -> Result<String, ToolError> {
            self.attempts += 1;
            if self.attempts == 1 {
                return Err(ToolError::Codex(CodexErr::Sandbox(SandboxErr::Denied {
                    output: Box::new(ExecToolCallOutput {
                        exit_code: 1,
                        ..Default::default()
                    }),
                    network_policy_decision: Some(NetworkPolicyDecisionPayload {
                        decision: NetworkPolicyDecision::Ask,
                        source: NetworkDecisionSource::Decider,
                        protocol: Some(NetworkApprovalProtocol::Https),
                        host: Some("example.com".to_string()),
                        port: Some(443),
                        reason: None,
                    }),
                })));
            }
            Ok("sandbox-retry-succeeded".to_string())
        }
    }

    let (session, mut action_turn, events) = make_session_and_context_with_rx().await;
    let step_context = captured_step_with_user_reviewer(
        &mut action_turn,
        AskForApproval::OnRequest,
        AskForApproval::UnlessTrusted,
    );

    let current_turn = activate_turn_with_new_review_authority(&session).await;
    assert_ne!(action_turn.sub_id, current_turn.sub_id);

    let call_id = "captured-action-sandbox-retry";
    let context = ToolCtx {
        session: Arc::clone(&session),
        step_context,
        cancellation_token: CancellationToken::new(),
        call_id: call_id.to_string(),
        tool_name: codex_tools::ToolName::plain("exec_command"),
    };
    let mut environment = context
        .step_context
        .environments
        .primary()
        .expect("primary environment")
        .clone();
    environment.config_mut().network_policy = Some(EnvironmentNetworkPolicy::from_config(
        &NetworkProxyConfig {
            enabled: true,
            ..Default::default()
        },
        /*managed_allowed_domains_only*/ false,
    ));
    let mut orchestrator = ToolOrchestrator::new();
    let mut runtime = DeniedOnceRuntime::default();
    let approve = async {
        let approval = next_exec_approval(&events).await;
        assert_eq!(approval.call_id, call_id);
        assert_eq!(approval.turn_id, action_turn.sub_id);
        assert_eq!(
            approval.network_approval_context,
            Some(NetworkApprovalContext {
                host: "example.com".to_string(),
                protocol: NetworkApprovalProtocol::Https,
            })
        );
        session
            .resolve_exec_approval(
                approval.call_id.clone(),
                Some(current_turn.sub_id.clone()),
                ReviewDecision::Abort,
            )
            .await;
        session
            .resolve_exec_approval(
                approval.call_id,
                Some(approval.turn_id),
                ReviewDecision::Approved,
            )
            .await;
    };

    let (output, ()) = tokio::join!(
        orchestrator.run(&mut runtime, &environment, &context),
        approve
    );
    assert_eq!(
        output
            .expect("approved sandbox retry should succeed")
            .output,
        "sandbox-retry-succeeded"
    );
    assert_eq!(runtime.attempts, 2);
}

#[tokio::test]
async fn guardian_allows_unified_exec_additional_permissions_requests_past_policy_validation() {
    let (mut session, mut turn_context_raw) = make_session_and_context().await;
    Arc::make_mut(&mut turn_context_raw.config)
        .permissions
        .approval_policy
        .set(AskForApproval::OnRequest)
        .expect("test setup should allow updating approval policy");
    Arc::make_mut(&mut turn_context_raw.config)
        .features
        .enable(Feature::GuardianApproval)
        .expect("test setup should allow enabling guardian approvals");
    session
        .features
        .enable(Feature::ExecPermissionApprovals)
        .expect("test setup should allow enabling request permissions");
    let session = Arc::new(session);
    let turn_context = Arc::new(turn_context_raw);
    let tracker = Arc::new(tokio::sync::Mutex::new(TurnDiffTracker::new()));
    let step_context = StepContext::for_test(Arc::clone(&turn_context));

    let handler = ExecCommandHandler::default();
    let resp = handler
        .handle(ToolInvocation {
            session: Arc::clone(&session),
            turn: Arc::clone(&turn_context),
            step_context,
            cancellation_token: CancellationToken::new(),
            tracker: Arc::clone(&tracker),
            call_id: "exec-call".to_string(),
            tool_name: codex_tools::ToolName::plain("exec_command"),
            source: crate::tools::context::ToolCallSource::Direct,
            native_call: None,
            environments: Default::default(),
            payload: ToolPayload::Function {
                arguments: serde_json::json!({
                    "cmd": "echo hi",
                    "sandbox_permissions": SandboxPermissions::WithAdditionalPermissions,
                    "justification": "need additional sandbox permissions",
                })
                .to_string(),
            },
        })
        .await;

    let Err(FunctionCallError::RespondToModel(output)) = resp else {
        panic!("expected validation error result");
    };

    assert_eq!(
        output,
        "missing `additional_permissions`; provide at least one of `network` or `file_system` when using `with_additional_permissions`"
    );
}

#[tokio::test]
async fn compaction_initial_context_preserves_separate_guardian_developer_message() {
    let (session, mut turn_context) = make_session_and_context().await;
    update_turn_settings_for_test(&mut turn_context, |settings| {
        update_selected_settings_for_test(settings, |selected| {
            selected.collaboration_mode.settings.reasoning_effort =
                Some(ReasoningEffortConfig::Persistent);
        });
    });
    let guardian_policy = "guardian policy".to_string();
    let guardian_source =
        SessionSource::SubAgent(SubAgentSource::Other(GUARDIAN_REVIEWER_NAME.to_string()));

    {
        let mut state = session.state.lock().await;
        state.session_configuration.session_source = guardian_source.clone();
    }
    turn_context.session_source = guardian_source;
    turn_context.developer_instructions = Some(guardian_policy.clone());
    let turn_context = Arc::new(turn_context);
    let step_context = StepContext::for_test(Arc::clone(&turn_context));
    let world_state = Arc::new(
        session
            .build_world_state_for_step(&step_context, /*new_window*/ true)
            .await
            .expect("world state should build"),
    );
    let (refreshed, _) = crate::compact::build_compaction_replacement_history(
        &session,
        &step_context,
        &world_state,
        Vec::new(),
    )
    .await;

    let developer_messages = refreshed
        .iter()
        .map(|item| &item.item)
        .filter_map(|item| match item {
            ResponseItem::Message { role, content, .. } if role == "developer" => {
                crate::content_items_to_text(content).map(|text| {
                    (
                        text,
                        item.executed_tool_call_metadata()
                            .and_then(|metadata| metadata.content_item_kinds.clone()),
                    )
                })
            }
            _ => None,
        })
        .collect::<Vec<_>>();

    assert!(
        !developer_messages
            .iter()
            .any(|(message, _)| message.contains("<persistent_mode>")),
        "guardian context must not inherit persistent-mode proactivity"
    );
    assert!(developer_messages.len() >= 2);
    assert_eq!(
        developer_messages.last(),
        Some(&(
            guardian_policy,
            Some(vec![ContentItemKind("guardian.policy".to_string())]),
        ))
    );
}

#[tokio::test]
#[cfg(unix)]
async fn exec_command_allows_sticky_turn_permissions_without_inline_request_permissions_feature() {
    let (mut session, turn_context_raw) = make_session_and_context().await;
    session
        .features
        .enable(Feature::RequestPermissionsTool)
        .expect("test setup should allow enabling request permissions tool");
<<<<<<< HEAD
    *session.active_turn.lock().await = Some(ActiveTurn::default());
    {
        let mut active_turn = session.active_turn.lock().await;
        let active_turn = active_turn.as_mut().expect("active turn");
        let mut turn_state = active_turn.turn_state.lock().await;
        turn_state.record_granted_permissions(
            codex_exec_server::LOCAL_ENVIRONMENT_ID,
            PermissionProfile {
                network: Some(NetworkPermissions {
                    enabled: Some(true),
                }),
                ..Default::default()
            },
            /*granted_under*/ None,
            session.services.authority_marker(),
        );
    }

||||||| 413492cd6c
    *session.active_turn.lock().await = Some(ActiveTurn::default());
    {
        let mut active_turn = session.active_turn.lock().await;
        let active_turn = active_turn.as_mut().expect("active turn");
        let mut turn_state = active_turn.turn_state.lock().await;
        turn_state.record_granted_permissions(
            codex_exec_server::LOCAL_ENVIRONMENT_ID,
            PermissionProfile {
                network: Some(NetworkPermissions {
                    enabled: Some(true),
                }),
                ..Default::default()
            },
        );
    }

=======
    turn_context_raw.record_granted_permissions(
        codex_exec_server::LOCAL_ENVIRONMENT_ID,
        PermissionProfile {
            network: Some(NetworkPermissions {
                enabled: Some(true),
            }),
            ..Default::default()
        },
        /*strict_auto_review*/ false,
    );
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
    let session = Arc::new(session);
    let turn_context = Arc::new(turn_context_raw);

    let handler = crate::tools::handlers::ExecCommandHandler::default();
    #[allow(deprecated)]
    let workdir = Some(turn_context.cwd.to_string_lossy().to_string());
    let step_context = StepContext::for_test(Arc::clone(&turn_context));
    let resp = handler
        .handle(ToolInvocation {
            session: Arc::clone(&session),
            turn: Arc::clone(&turn_context),
            step_context,
            cancellation_token: CancellationToken::new(),
            tracker: Arc::new(tokio::sync::Mutex::new(TurnDiffTracker::new())),
            call_id: "sticky-turn-grant".to_string(),
            tool_name: codex_tools::ToolName::plain("exec_command"),
            source: crate::tools::context::ToolCallSource::Direct,
            native_call: None,
            environments: Default::default(),
            payload: ToolPayload::Function {
                arguments: serde_json::json!({
                    "cmd": "echo hi",
                    "login": false,
<<<<<<< HEAD
                    "timeout_ms": if cfg!(windows) { 10_000_u64 } else { 1_000_u64 },
||||||| 413492cd6c
                    "timeout_ms": 1_000_u64,
=======
                    "yield_time_ms": 10_000_u64,
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
                    "workdir": workdir,
                })
                .to_string(),
            },
        })
        .await;

    match resp {
        Ok(output) => {
            let output = expect_text_output(&output);
            assert!(output.contains("hi"));
        }
        Err(FunctionCallError::RespondToModel(output)) => {
            assert!(
                !output.contains("additional permissions are disabled"),
                "sticky turn permissions should bypass inline validation: {output}"
            );
        }
        Err(err) => panic!("unexpected error: {err:?}"),
    }
}

#[tokio::test]
async fn guardian_subagent_does_not_inherit_parent_exec_policy_rules() {
    let codex_home = tempdir().expect("create codex home");
    let project_dir = tempdir().expect("create project dir");
    let rules_dir = project_dir.path().join("rules");
    fs::create_dir_all(&rules_dir).expect("create rules dir");
    fs::write(
        rules_dir.join("deny.rules"),
        r#"prefix_rule(pattern=["rm"], decision="forbidden")"#,
    )
    .expect("write policy file");

    let mut config = build_test_config(codex_home.path(), project_dir.path()).await;
    config.cwd = project_dir.abs();
    config.config_layer_stack = ConfigLayerStack::new(
        vec![ConfigLayerEntry::new(
            ConfigLayerSource::Project {
                dot_codex_folder: project_dir.path().abs(),
            },
            toml::Value::Table(Default::default()),
        )],
        ConfigRequirements::default(),
        ConfigRequirementsToml::default(),
    )
    .expect("config layer stack");

    let command = [vec!["rm".to_string()]];
    let parent_exec_policy =
        ExecPolicyManager::load(&config.config_layer_stack, /*strict_rules*/ false)
            .await
            .expect("load parent exec policy");
    assert_eq!(
        parent_exec_policy
            .current()
            .check_multiple(command.iter(), &|_| Decision::Allow),
        Evaluation {
            decision: Decision::Forbidden,
            matched_rules: vec![RuleMatch::PrefixRuleMatch {
                matched_prefix: vec!["rm".to_string()],
                decision: Decision::Forbidden,
                resolved_program: None,
                justification: None,
            }],
        }
    );

    let auth_manager = AuthManager::from_auth_for_testing(CodexAuth::from_api_key("Test API Key"));
    let models_manager = models_manager_with_provider(
        config.codex_home.to_path_buf(),
        auth_manager.clone(),
        config.model_provider.clone(),
    );
    let plugins_manager = Arc::new(plugins_manager_for_config(
        &config,
        Arc::clone(&auth_manager),
    ));
    let skills_service = Arc::new(HostSkillsService::new(
        config.codex_home.clone(),
        /*bundled_skills_enabled*/ true,
    ));
    let mcp_manager = Arc::new(McpManager::new(Arc::clone(&plugins_manager)));
    let thread_store = Arc::new(codex_thread_store::LocalThreadStore::new(
        codex_thread_store::LocalThreadStoreConfig::from_config(&config),
        /*state_db*/ None,
    ));

    let mut thread_extension_init = codex_extension_api::ExtensionDataInit::default();
    thread_extension_init.insert(codex_extension_api::SessionIsolation::Isolated);
    let (session, io) = Session::spawn(SessionSpawnArgs {
        attached_thread: false,
        startup: None,
        config,
        prompt_cache_key: PromptCacheKey::default(),
        allow_provider_model_fallback: false,
        instructions: Default::default(),
        installation_id: "11111111-1111-4111-8111-111111111111".to_string(),
        auth_manager,
        models_manager,
        git_root_discovery: Arc::default(),
        environment_manager: Arc::new(EnvironmentManager::default_for_tests()),
        skills_service,
        plugins_manager,
        mcp_manager,
        code_mode_session_provider: Arc::new(codex_code_mode::DisabledCodeModeSessionProvider),
        extensions: codex_extension_api::empty_extension_registry(),
        conversation_history: InitialHistory::New,
        resume_preparation: None,
        history_initialization: HistoryInitialization::New,
        disabled_plugin_ids: None,
        requested_history_mode: None,
        fork_persistence: ForkPersistence::Copied,
        session_source: SessionSource::SubAgent(SubAgentSource::Other(
            GUARDIAN_REVIEWER_NAME.to_string(),
        )),
        forked_from_thread_id: None,
        parent_thread_id: None,
        thread_source: None,
        originator: "test_originator".to_string(),
        agent_control: LocalAgentControl::default().into(),
        dynamic_tools: Vec::new(),
        metrics_service_name: None,
        inherited_environments: None,
        inherited_exec_policy: Some(Arc::new(parent_exec_policy)),
        parent_rollout_thread_trace: codex_rollout_trace::ThreadTraceContext::disabled(),
        user_shell_override: None,
        parent_trace: None,
        environment_requests: Vec::new(),
        thread_extension_init,
        turn_extension_init: Default::default(),
        client_mcp_extensions: ClientMcpExtensions::default(),
        reserved_thread_id: None,
        analytics_events_client: None,
        image_store: crate::thread_manager::passthrough_image_store(),
        thread_store,
        attestation_provider: None,
        external_time_provider: None,
        inherited_multi_agent_version: None,
        git_enrichment_policy: GitEnrichmentPolicy::Skip,
        windows_sandbox_proxy_settings_mode:
            codex_sandboxing::WindowsSandboxProxySettingsMode::Preserve,
    })
    .await
    .expect("spawn guardian subagent");

    assert_eq!(
        session
            .services
            .exec_policy
            .current()
            .check_multiple(command.iter(), &|_| Decision::Allow),
        Evaluation {
            decision: Decision::Allow,
            matched_rules: vec![RuleMatch::HeuristicsRuleMatch {
                command: vec!["rm".to_string()],
                decision: Decision::Allow,
            }],
        }
    );
    drop(io);
}

/// PF-60-S05 AC8: an ephemeral guardian review fork records its paid requests
/// under the conversation it reviews for, in that conversation's ledger, when
/// that conversation is persisted; any other ephemeral session has no
/// accounting owner.
#[tokio::test]
async fn accounting_owner_of_a_guardian_fork_is_the_reviewed_conversation() -> anyhow::Result<()> {
    let (mut session, _turn) = crate::session::tests::make_session_and_context().await;
    assert!(session.live_thread().is_none());
    assert_eq!(
        session.accounting_owner().await.err(),
        Some(crate::accounting::Uncollected::Excluded(
            "ephemeral_session"
        ))
    );
    // An ephemeral fork has no database of its own; its thread store is the
    // reviewed conversation's, and so is its ledger.
    let home = tempfile::tempdir()?;
    let db = codex_state::StateRuntime::init(
        codex_state::SqliteConfig::from_sqlite_home(
            codex_utils_absolute_path::AbsolutePathBuf::try_from(home.path().to_path_buf())?,
        ),
        "openai".into(),
    )
    .await?;
    session.services.state_db = None;
    session.services.thread_store = Arc::new(codex_thread_store::LocalThreadStore::new(
        codex_thread_store::LocalThreadStoreConfig::from_config(
            session.get_config().await.as_ref(),
        ),
        Some(db.clone()),
    ));
    let parent = codex_protocol::ThreadId::new();
    {
        let mut state = session.state.lock().await;
        state.session_configuration.session_source =
            SessionSource::SubAgent(SubAgentSource::Other(GUARDIAN_REVIEWER_NAME.to_string()));
        state.session_configuration.parent_thread_id = Some(parent);
    }
    // A reviewer of a conversation that is not persisted has nowhere to record.
    assert_eq!(
        session.accounting_owner().await.err(),
        Some(crate::accounting::Uncollected::Excluded(
            "parent_not_persisted"
        ))
    );
    db.upsert_thread(
        &codex_state::ThreadMetadataBuilder::new(
            parent,
            home.path().join("parent.jsonl"),
            chrono::Utc::now(),
            SessionSource::Cli,
        )
        .build("openai"),
    )
    .await?;
    let owner = session
        .accounting_owner()
        .await
        .map_err(|why| anyhow::anyhow!("{why:?}"))?;
    assert_eq!(
        (owner.thread, owner.label_prefix.as_str()),
        (parent, "review:")
    );
    assert!(Arc::ptr_eq(&owner.db, &db));
    // PF-60-S06: another sub-agent with no thread of its own works for its
    // parent too, so it records there as `agent:`; without a parent it stays
    // excluded.
    {
        let mut state = session.state.lock().await;
        state.session_configuration.session_source =
            SessionSource::SubAgent(SubAgentSource::Other("another-helper".to_string()));
    }
    let owner = session
        .accounting_owner()
        .await
        .map_err(|why| anyhow::anyhow!("{why:?}"))?;
    assert_eq!(
        (owner.thread, owner.label_prefix.as_str()),
        (parent, "agent:")
    );
    session
        .state
        .lock()
        .await
        .session_configuration
        .parent_thread_id = None;
    assert_eq!(
        session.accounting_owner().await.err(),
        Some(crate::accounting::Uncollected::Excluded(
            "ephemeral_session"
        ))
    );
    Ok(())
}

/// A session with no database of its own whose thread store is `db`'s, as an
/// ephemeral fork's or an internal worker's is.
async fn ephemeral_session_on(db: &Arc<codex_state::StateRuntime>) -> Session {
    let (mut session, _turn) = crate::session::tests::make_session_and_context().await;
    assert!(session.live_thread().is_none());
    session.services.state_db = None;
    session.services.thread_store = Arc::new(codex_thread_store::LocalThreadStore::new(
        codex_thread_store::LocalThreadStoreConfig::from_config(
            session.get_config().await.as_ref(),
        ),
        Some(Arc::clone(db)),
    ));
    session
}

async fn persist_thread(
    db: &codex_state::StateRuntime,
    home: &std::path::Path,
    thread: codex_protocol::ThreadId,
) -> anyhow::Result<()> {
    db.upsert_thread(
        &codex_state::ThreadMetadataBuilder::new(
            thread,
            home.join(format!("{thread}.jsonl")),
            chrono::Utc::now(),
            SessionSource::Cli,
        )
        .build("openai"),
    )
    .await?;
    Ok(())
}

/// PF-60-S04: a side conversation - an ephemeral fork of a persisted
/// conversation - records under the conversation it was forked from, labelled
/// `side:`; a memory consolidation agent records under the conversation that
/// started it, labelled `consolidation:`. Neither records when that
/// conversation is not persisted, and an ephemeral session that is neither
/// stays excluded. A guardian reviewer stays `review:`, and a persisted
/// session records under its own thread, except a `/review` task (#898).
#[tokio::test]
async fn accounting_owner_of_side_and_consolidation_sessions() -> anyhow::Result<()> {
    use crate::accounting::Uncollected;
    let home = tempfile::tempdir()?;
    let db = codex_state::StateRuntime::init(
        codex_state::SqliteConfig::from_sqlite_home(
            codex_utils_absolute_path::AbsolutePathBuf::try_from(home.path().to_path_buf())?,
        ),
        "openai".into(),
    )
    .await?;

    // Side conversation.
    let side = ephemeral_session_on(&db).await;
    assert_eq!(
        side.accounting_owner().await.err(),
        Some(Uncollected::Excluded("ephemeral_session"))
    );
    let parent = codex_protocol::ThreadId::new();
    side.state
        .lock()
        .await
        .session_configuration
        .forked_from_thread_id = Some(parent);
    assert_eq!(
        side.accounting_owner().await.err(),
        Some(Uncollected::Excluded("parent_not_persisted"))
    );
    persist_thread(&db, home.path(), parent).await?;
    // PF-60-S06: an ephemeral fork that is not `/side` is a plain `fork:`.
    let owner = side
        .accounting_owner()
        .await
        .map_err(|why| anyhow::anyhow!("{why:?}"))?;
    assert_eq!(
        (owner.thread, owner.label_prefix.as_str()),
        (parent, "fork:")
    );
    side.state.lock().await.session_configuration.thread_source = Some(
        codex_protocol::protocol::ThreadSource::Feature("side".to_string()),
    );
    let owner = side
        .accounting_owner()
        .await
        .map_err(|why| anyhow::anyhow!("{why:?}"))?;
    assert_eq!(
        (owner.thread, owner.label_prefix.as_str()),
        (parent, "side:")
    );
    // Another feature's fork is not `/side`.
    side.state.lock().await.session_configuration.thread_source = Some(
        codex_protocol::protocol::ThreadSource::Feature("automation".to_string()),
    );
    let owner = side
        .accounting_owner()
        .await
        .map_err(|why| anyhow::anyhow!("{why:?}"))?;
    assert_eq!(
        (owner.thread, owner.label_prefix.as_str()),
        (parent, "fork:")
    );

    // Consolidation: recorded under the conversation that started it, ahead
    // of any fork origin, and only once that conversation is persisted.
    let consolidation = ephemeral_session_on(&db).await;
    let starter = codex_protocol::ThreadId::new();
    consolidation
        .state
        .lock()
        .await
        .session_configuration
        .forked_from_thread_id = Some(parent);
    consolidation
        .services
        .thread_extension_data
        .insert(crate::accounting::StartedBy(starter));
    assert_eq!(
        consolidation.accounting_owner().await.err(),
        Some(Uncollected::Excluded("parent_not_persisted"))
    );
    persist_thread(&db, home.path(), starter).await?;
    let owner = consolidation
        .accounting_owner()
        .await
        .map_err(|why| anyhow::anyhow!("{why:?}"))?;
    assert_eq!(
        (owner.thread, owner.label_prefix.as_str()),
        (starter, "consolidation:")
    );

    // A guardian reviewer stays a review even when it is also a fork.
    let reviewer = ephemeral_session_on(&db).await;
    {
        let mut state = reviewer.state.lock().await;
        state.session_configuration.session_source =
            SessionSource::SubAgent(SubAgentSource::Other(GUARDIAN_REVIEWER_NAME.to_string()));
        state.session_configuration.parent_thread_id = Some(starter);
        state.session_configuration.forked_from_thread_id = Some(parent);
    }
    let owner = reviewer
        .accounting_owner()
        .await
        .map_err(|why| anyhow::anyhow!("{why:?}"))?;
    assert_eq!(
        (owner.thread, owner.label_prefix.as_str()),
        (starter, "review:")
    );

    // A persisted fork, or a persisted session carrying a starter, records
    // under its own thread, unlabelled.
    let (mut persisted, _turn) = crate::session::tests::make_session_and_context().await;
    let config = persisted.get_config().await;
    let store: Arc<dyn codex_thread_store::ThreadStore> =
        Arc::new(codex_thread_store::InMemoryThreadStore::default());
    persisted.services.live_thread = Some(
        LiveThread::create(
            Arc::clone(&store),
            CreateThreadParams {
                session_id: persisted.session_id(),
                thread_id: persisted.thread_id,
                extra_config: None,
                forked_from_id: Some(parent),
                parent_thread_id: None,
                source: SessionSource::Cli,
                thread_source: None,
                originator: "test_originator".to_string(),
                base_instructions: BaseInstructions::default(),
                dynamic_tools: Vec::new(),
                selected_capability_roots: Vec::new(),
                multi_agent_version: None,
                history_mode: Default::default(),
                subagent_history_start_ordinal: None,
                history_base: None,
                initial_window_id: Uuid::now_v7().to_string(),
                metadata: ThreadPersistenceMetadata {
                    cwd: Some(config.cwd.to_path_buf()),
                    model_provider: config.model_provider_id.clone(),
                    memory_mode: ThreadMemoryMode::Disabled,
                },
            },
        )
        .await?,
    );
    persisted.services.thread_store = store;
    persisted.services.state_db = Some(Arc::clone(&db));
    persisted
        .state
        .lock()
        .await
        .session_configuration
        .forked_from_thread_id = Some(parent);
    persisted
        .services
        .thread_extension_data
        .insert(crate::accounting::StartedBy(starter));
    let owner = persisted
        .accounting_owner()
        .await
        .map_err(|why| anyhow::anyhow!("{why:?}"))?;
    assert_eq!(
        (owner.thread, owner.label_prefix.as_str()),
        (persisted.thread_id, "")
    );

    // #898: a `/review` task is persisted as a thread of its own, but that
    // thread names no parent the ledger can resolve; it records under the
    // conversation that started it, as `code-review:`.
    {
        let mut state = persisted.state.lock().await;
        state.session_configuration.session_source =
            SessionSource::SubAgent(SubAgentSource::Review);
        state.session_configuration.parent_thread_id = Some(starter);
    }
    let owner = persisted
        .accounting_owner()
        .await
        .map_err(|why| anyhow::anyhow!("{why:?}"))?;
    assert_eq!(
        (owner.thread, owner.label_prefix.as_str()),
        (starter, "code-review:")
    );
    Ok(())
}

/// PF-60-S06: a sub-agent with no thread of its own records under the
/// conversation it works for, labelled with what it is: a spawned agent
/// `agent:` (also when it was spawned as a fork), a `/review` task
/// `code-review:`. A parent that is not persisted and not live leaves it
/// excluded.
#[tokio::test]
async fn accounting_owner_of_ephemeral_sub_agents() -> anyhow::Result<()> {
    use crate::accounting::Uncollected;
    let home = tempfile::tempdir()?;
    let db = codex_state::StateRuntime::init(
        codex_state::SqliteConfig::from_sqlite_home(
            codex_utils_absolute_path::AbsolutePathBuf::try_from(home.path().to_path_buf())?,
        ),
        "openai".into(),
    )
    .await?;
    let parent = codex_protocol::ThreadId::new();

    let agent = ephemeral_session_on(&db).await;
    {
        let mut state = agent.state.lock().await;
        state.session_configuration.session_source =
            SessionSource::SubAgent(SubAgentSource::ThreadSpawn {
                parent_thread_id: parent,
                depth: 1,
                agent_path: None,
                agent_nickname: None,
                agent_role: None,
                agent_class: None,
            });
        state.session_configuration.forked_from_thread_id = Some(parent);
        state.session_configuration.thread_source = Some(
            codex_protocol::protocol::ThreadSource::Feature("side".to_string()),
        );
    }
    assert_eq!(
        agent.accounting_owner().await.err(),
        Some(Uncollected::Excluded("parent_not_persisted"))
    );
    persist_thread(&db, home.path(), parent).await?;
    let owner = agent
        .accounting_owner()
        .await
        .map_err(|why| anyhow::anyhow!("{why:?}"))?;
    assert_eq!(
        (owner.thread, owner.label_prefix.as_str()),
        (parent, "agent:")
    );

    let review = ephemeral_session_on(&db).await;
    {
        let mut state = review.state.lock().await;
        state.session_configuration.session_source =
            SessionSource::SubAgent(SubAgentSource::Review);
        state.session_configuration.parent_thread_id = Some(parent);
    }
    let owner = review
        .accounting_owner()
        .await
        .map_err(|why| anyhow::anyhow!("{why:?}"))?;
    assert_eq!(
        (owner.thread, owner.label_prefix.as_str()),
        (parent, "code-review:")
    );
    Ok(())
}

/// PF-60-S05 AC9: a session whose paid requests are not collected says so in
/// the log once, however many turns it runs.
#[tokio::test]
async fn an_uncollected_session_is_named_once() {
    use std::sync::atomic::AtomicUsize;
    use std::sync::atomic::Ordering;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;
    struct Excluded(Arc<AtomicUsize>);
    impl<S: tracing::Subscriber> tracing_subscriber::Layer<S> for Excluded {
        fn on_event(
            &self,
            event: &tracing::Event<'_>,
            _: tracing_subscriber::layer::Context<'_, S>,
        ) {
            if event
                .metadata()
                .fields()
                .field("accounting.excluded")
                .is_some()
            {
                self.0.fetch_add(1, Ordering::SeqCst);
            }
        }
    }
    let count = Arc::new(AtomicUsize::new(0));
    let _guard = tracing_subscriber::registry()
        .with(Excluded(Arc::clone(&count)))
        .set_default();
    let (session, _turn) = crate::session::tests::make_session_and_context().await;
    for _ in 0..3 {
        assert!(crate::accounting::collecting_owner(&session).await.is_err());
    }
    assert_eq!(count.load(Ordering::SeqCst), 1);
}
