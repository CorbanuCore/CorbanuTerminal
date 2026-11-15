//! #2272 (S3b): after untrusted content, a `request_permissions` call that an
//! automatic reviewer would answer goes to the person first. Administrator
//! requirements can force the reviewer whatever the step's reviewer setting
//! (`[auto_review] required_on_models`, or `approvals_reviewer` constrained so
//! the person may not review); the post-taint gate must count those too.

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
) {
    // The test model's slug, for `required_on_models`.
    let (_, probe, _) = crate::session::tests::make_session_and_context_with_rx().await;
    let model = probe.model_info().slug.clone();
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
                let mut requirements: ConfigRequirements =
                    config.config_layer_stack.requirements().clone();
                match forced {
                    Forced::No => {}
                    Forced::ModelRequiresReview => {
                        requirements.auto_review_required_models = Some(Sourced::new(
                            BTreeSet::from([model]),
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
    (session, turn, events)
}

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
        let (session, turn, _events) = tainted_session(AskForApproval::Never, forced).await;
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
        let (session, turn, events) = tainted_session(AskForApproval::OnRequest, forced).await;
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
