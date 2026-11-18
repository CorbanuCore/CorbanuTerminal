//! #2272 (S3b): after untrusted content, a `request_permissions` call that an
//! automatic reviewer would answer goes to the person first. Administrator
//! requirements can force the reviewer whatever the step's reviewer setting
//! (`[auto_review] required_on_models`, or `approvals_reviewer` constrained so
//! the person may not review); the post-taint gate must count those too.

use crate::config::Config;
use crate::security::protected_surface::ToolOrigin;
use crate::security::protected_surface::check_dispatch;
use crate::session::session::Session;
use crate::session::step_context::StepContext;
use crate::session::turn_context::TurnContext;
use crate::tools::context::ToolInvocation;
use crate::tools::context::ToolPayload;
use crate::tools::router::ToolCallSource;
use crate::turn_diff_tracker::TurnDiffTracker;
use codex_config::ConfigLayerStack;
use codex_config::ConfigRequirements;
use codex_config::Constrained;
use codex_config::RequirementSource;
use codex_config::Sourced;
use codex_protocol::config_types::ApprovalsReviewer;
use codex_protocol::models::FunctionCallOutputPayload;
use codex_protocol::models::ResponseItem;
use codex_protocol::protocol::AskForApproval;
use codex_protocol::protocol::EventMsg;
use codex_security_policy::SecurityLevel;
use codex_tools::ToolName;
use pretty_assertions::assert_eq;
use std::collections::BTreeSet;
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone, Copy, Debug)]
enum Forced {
    No,
    ModelRequiresReview,
    PersonMayNotReview,
}

/// A Moderate session that has seen untrusted content, with `forced` written
/// into the session's administrator requirements.
async fn tainted_session(
    policy: AskForApproval,
    forced: Forced,
) -> (
    Arc<Session>,
    Arc<TurnContext>,
    async_channel::Receiver<codex_protocol::protocol::Event>,
    wiremock::MockServer,
) {
    // The test model's slug, for `required_on_models`.
    let (_, probe, _) = crate::session::tests::make_session_and_context_with_rx().await;
    let model = probe.model_info().slug.clone();
    // Automatic reviews these tests start reach a local stand-in that
    // refuses them at once, never a real service.
    let reviewer = wiremock::MockServer::start().await;
    wiremock::Mock::given(wiremock::matchers::any())
        .respond_with(wiremock::ResponseTemplate::new(401))
        .mount(&reviewer)
        .await;
    let reviewer_url = format!("{}/v1", reviewer.uri());
    let (session, turn, events) =
        crate::session::tests::make_session_and_context_with_auth_and_config_and_rx(
            codex_login::CodexAuth::from_api_key("Test API Key"),
            Vec::new(),
            move |config| {
                config
                    .permissions
                    .approval_policy
                    .set(policy)
                    .expect("approval policy");
                force_reviewer(config, forced, &model);
                config.model_provider.base_url = Some(reviewer_url.clone());
            },
        )
        .await;
    let client = (*session.services.model_client())
        .clone()
        .with_ingress_level(SecurityLevel::Moderate)
        .with_source_envelopes(/*enabled*/ true);
    client.note_recorded_for_taint(&[ResponseItem::FunctionCallOutput {
        name: None,
        namespace: None,
        id: None,
        call_id: Some("call-read".into()),
        output: FunctionCallOutputPayload::from_text("README".into()),
        internal_chat_message_metadata_passthrough: None,
    }]);
    session.services.replace_model_client(client);
    (session, turn, events, reviewer)
}

/// Writes `forced` into `config`'s administrator requirements.
fn force_reviewer(config: &mut Config, forced: Forced, model: &str) {
    let mut requirements: ConfigRequirements = config.config_layer_stack.requirements().clone();
    match forced {
        Forced::No => {}
        Forced::ModelRequiresReview => {
            requirements.auto_review_required_models = Some(Sourced::new(
                BTreeSet::from([model.to_string()]),
                RequirementSource::Unknown,
            ));
        }
        Forced::PersonMayNotReview => {
            requirements.approvals_reviewer.value =
                Constrained::allow_only(ApprovalsReviewer::AutoReview);
        }
    }
    config.config_layer_stack = ConfigLayerStack::new(
        config
            .config_layer_stack
            .all_layers_low_to_high()
            .cloned()
            .collect(),
        requirements,
        config.config_layer_stack.requirements_toml().clone(),
    )
    .expect("replace test requirements");
}

fn network_request(turn: &TurnContext) -> super::PreparedPermissionRequest {
    let environment = turn
        .initial_environments
        .primary()
        .expect("primary environment")
        .selection();
    super::PreparedPermissionRequest {
        environment_id: environment.environment_id,
        reason: None,
        permissions: codex_protocol::request_permissions::RequestPermissionProfile {
            network: Some(codex_protocol::models::NetworkPermissions {
                enabled: Some(true),
            }),
            ..Default::default()
        },
    }
}

/// The post-taint question for `call-permissions`, answered `answer`.
async fn answer_question(session: &Session, turn: &TurnContext, answer: &str) {
    let id = crate::security::protected_surface::question_id("call-permissions");
    session
        .notify_user_input_response(
            &turn.sub_id,
            codex_protocol::request_user_input::RequestUserInputResponse {
                answers: std::collections::HashMap::from([(
                    id,
                    codex_protocol::request_user_input::RequestUserInputAnswer {
                        answers: vec![answer.to_string()],
                    },
                )]),
            },
        )
        .await;
}

/// D625 (a) (ii)'s refusal at the review itself.
const RELOAD_REFUSAL: &str = "administrator requirements changed while this permissions request";

fn invocation(session: &Arc<Session>, turn: &Arc<TurnContext>) -> ToolInvocation {
    let step_context = StepContext::for_test(Arc::clone(turn));
    // The step's own reviewer setting stays the person: only the
    // requirements force the automatic reviewer.
    assert_eq!(
        step_context.settings.approvals_reviewer(),
        ApprovalsReviewer::User
    );
    ToolInvocation {
        environments: Default::default(),
        native_call: None,
        session: Arc::clone(session),
        step_context,
        turn: Arc::clone(turn),
        cancellation_token: tokio_util::sync::CancellationToken::new(),
        tracker: Arc::new(tokio::sync::Mutex::new(TurnDiffTracker::new())),
        call_id: "call-permissions".to_string(),
        tool_name: ToolName::plain("request_permissions"),
        source: ToolCallSource::Direct,
        payload: ToolPayload::Function {
            arguments: r#"{"permissions":{"network":{"enabled":true}}}"#.to_string(),
        },
    }
}

/// With approvals off, a forced reviewer means the gate refuses the request
/// (the person cannot be asked); without one, the request's own prompt reaches
/// the person, so the gate lets it through.
#[tokio::test]
async fn issue_2272_forced_reviewer_cannot_answer_post_taint_permission_requests() {
    for forced in [
        Forced::No,
        Forced::ModelRequiresReview,
        Forced::PersonMayNotReview,
    ] {
        let (session, turn, _events, _reviewer) =
            tainted_session(AskForApproval::Never, forced).await;
        let result = check_dispatch(&invocation(&session, &turn), ToolOrigin::Builtin).await;
        match forced {
            Forced::No => assert_eq!(result, Ok(()), "{forced:?}"),
            Forced::ModelRequiresReview | Forced::PersonMayNotReview => {
                let refusal = result.expect_err("refused");
                assert!(
                    refusal.starts_with("Not run: Security level Moderate:")
                        && refusal.contains("after untrusted content"),
                    "{forced:?}: {refusal}"
                );
            }
        }
    }
}

/// With approvals on, a forced reviewer means the person is asked first.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_2272_forced_reviewer_routes_post_taint_permission_requests_to_the_person() {
    for forced in [Forced::ModelRequiresReview, Forced::PersonMayNotReview] {
        let (session, turn, events, _reviewer) =
            tainted_session(AskForApproval::OnRequest, forced).await;
        let invocation = invocation(&session, &turn);
        let dispatch =
            tokio::spawn(async move { check_dispatch(&invocation, ToolOrigin::Builtin).await });
        let question = tokio::time::timeout(Duration::from_secs(10), async {
            loop {
                let event = events.recv().await.expect("event channel open");
                if let EventMsg::RequestUserInput(shown) = event.msg {
                    return shown.questions[0].question.clone();
                }
            }
        })
        .await
        .unwrap_or_else(|_| panic!("{forced:?}: the person was not asked"));
        assert!(
            question.starts_with("Allow `request_permissions`?"),
            "{forced:?}: {question}"
        );
        dispatch.abort();
    }
}

/// D625 (a) (ii): the dispatch check lets a post-taint request through
/// without asking because the person answers its own prompt; requirements
/// then reload and force the automatic reviewer before that prompt. The
/// request's review must still ask the person first, not let the automatic
/// reviewer answer alone. Declining refuses the request.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn d625_requirements_reload_after_dispatch_still_asks_the_person_first() {
    for forced in [Forced::ModelRequiresReview, Forced::PersonMayNotReview] {
        let (session, turn, events, _reviewer) =
            tainted_session(AskForApproval::OnRequest, Forced::No).await;
        *session.active_turn.lock().await = Some(crate::state::ActiveTurn::default());
        let invocation = invocation(&session, &turn);
        let step_context = Arc::clone(&invocation.step_context);
        assert_eq!(
            tokio::time::timeout(
                Duration::from_secs(10),
                check_dispatch(&invocation, ToolOrigin::Builtin)
            )
            .await,
            Ok(Ok(())),
            "{forced:?}: the person answers the request's own prompt"
        );

        // The host's refresh carries fresh administrator requirements.
        let live = session.get_config().await;
        let mut reloaded = (*live).clone();
        force_reviewer(&mut reloaded, forced, &turn.model_info().slug);
        assert_eq!(
            session.refresh_mcp_config(live, reloaded).await,
            crate::ConfigRefreshOutcome::Published,
            "{forced:?}"
        );
        assert!(
            session
                .get_config()
                .await
                .config_layer_stack
                .requirements()
                .automatic_reviewer_forced(&turn.model_info().slug),
            "{forced:?}: the reload forces the automatic reviewer"
        );

        let request = network_request(&turn);
        let mut review = tokio::spawn({
            let session = Arc::clone(&session);
            async move {
                session
                    .request_permissions_explained(
                        &step_context,
                        "call-permissions".to_string(),
                        request,
                        tokio_util::sync::CancellationToken::new(),
                    )
                    .await
            }
        });
        let shown = tokio::time::timeout(Duration::from_secs(10), async {
            loop {
                tokio::select! {
                    finished = &mut review => {
                        panic!("{forced:?}: answered without asking the person: {finished:?}")
                    }
                    event = events.recv() => match event.expect("event channel open").msg {
                        EventMsg::RequestUserInput(shown) => return shown,
                        EventMsg::RequestPermissions(_) => {
                            panic!("{forced:?}: reached a prompt without the post-taint question")
                        }
                        _ => {}
                    },
                }
            }
        })
        .await
        .unwrap_or_else(|_| panic!("{forced:?}: the person was not asked"));
        assert!(
            shown.questions[0]
                .question
                .starts_with("Allow `request_permissions`?"),
            "{forced:?}: {:?}",
            shown.questions[0].question
        );

        answer_question(&session, &turn, "Cancel").await;
        let refusal = tokio::time::timeout(Duration::from_secs(10), review)
            .await
            .expect("review finished")
            .expect("review joined")
            .expect_err("declined");
        assert!(
            refusal.contains("you declined") && refusal.contains("after untrusted content"),
            "{forced:?}: {refusal}"
        );
    }
}

/// D625 (a) (ii), at the review itself: requirements that force the
/// automatic reviewer were reloaded after the checks before the review, and
/// the person has not allowed this call. The review refuses the request
/// rather than let the reviewer answer alone. Untainted, it goes ahead.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn d625_review_refuses_a_forced_review_the_person_did_not_allow() {
    for forced in [Forced::ModelRequiresReview, Forced::PersonMayNotReview] {
        let (session, turn, _events, _reviewer) =
            tainted_session(AskForApproval::OnRequest, forced).await;
        let step_context = StepContext::for_test(Arc::clone(&turn));
        let context = crate::tools::ApprovalContext {
            exec_requirement: None,
            review_context: crate::guardian::GuardianReviewContext::from(&step_context),
            cancellation_token: None,
            call_id: "call-permissions".to_string(),
            tool_name: ToolName::plain("request_permissions"),
            strict_auto_review: false,
            approval_reason: None,
            retry_reason: None,
            network_approval_context: None,
        };
        let action = network_request(&turn).approval_action("call-permissions", &turn.sub_id);
        let decision = tokio::time::timeout(
            Duration::from_secs(10),
            session.request_guardian_approval(action.clone(), &context),
        )
        .await
        .expect("review finished");
        match decision {
            Some(codex_protocol::protocol::ReviewDecision::Denied { rejection })
                if rejection.contains(RELOAD_REFUSAL) => {}
            other => panic!("{forced:?}: the forced reviewer answered alone: {other:?}"),
        }

        // Once the person allowed this call, the forced reviewer reviews it.
        turn.post_taint_policy_answers.note("call-permissions");
        let decision = tokio::time::timeout(
            Duration::from_secs(10),
            session.request_guardian_approval(action, &context),
        )
        .await
        .expect("review finished");
        assert!(
            !matches!(
                &decision,
                Some(codex_protocol::protocol::ReviewDecision::Denied { rejection })
                    if rejection.contains(RELOAD_REFUSAL)
            ),
            "{forced:?}: {decision:?}"
        );
    }
}

/// D625 (a): the person allowed the request at the dispatch check (forced
/// review already in force); the request's review does not ask again.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn d625_request_allowed_at_dispatch_is_not_asked_again() {
    let (session, turn, events, _reviewer) =
        tainted_session(AskForApproval::OnRequest, Forced::ModelRequiresReview).await;
    *session.active_turn.lock().await = Some(crate::state::ActiveTurn::default());
    let invocation = invocation(&session, &turn);
    let step_context = Arc::clone(&invocation.step_context);
    let dispatch =
        tokio::spawn(async move { check_dispatch(&invocation, ToolOrigin::Builtin).await });
    tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            if let EventMsg::RequestUserInput(_) = events.recv().await.expect("event").msg {
                return;
            }
        }
    })
    .await
    .expect("the person was asked at dispatch");
    answer_question(&session, &turn, "Allow once").await;
    assert_eq!(
        tokio::time::timeout(Duration::from_secs(10), dispatch)
            .await
            .expect("dispatch finished")
            .expect("dispatch joined"),
        Ok(())
    );

    let mut review = tokio::spawn({
        let session = Arc::clone(&session);
        let request = network_request(&turn);
        async move {
            session
                .request_permissions_explained(
                    &step_context,
                    "call-permissions".to_string(),
                    request,
                    tokio_util::sync::CancellationToken::new(),
                )
                .await
        }
    });
    let finished = tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            tokio::select! {
                finished = &mut review => return finished.expect("review joined"),
                event = events.recv() => {
                    if let EventMsg::RequestUserInput(shown) = event.expect("event").msg {
                        panic!("asked again: {:?}", shown.questions[0].question);
                    }
                }
            }
        }
    })
    .await
    .expect("review finished");
    assert!(
        !matches!(&finished, Err(refusal) if refusal.contains(RELOAD_REFUSAL)),
        "{finished:?}"
    );
}
