//! D625 (a) / #2562 (S3b row 51): administrator requirements that force the automatic reviewer
//! (`[auto_review] required_on_models`, or `approvals_reviewer` constrained so the person may not
//! review) decide who answers whatever the step's reviewer setting says. Labelling, the person-only
//! notes and the protected-write scope must not treat such a review as the person's.

use super::*;
use crate::session::step_context::StepContext;
use crate::tools::sandboxing::ExecApprovalRequirement;
use codex_config::ConfigLayerStack;
use codex_config::ConfigRequirements;
use codex_config::Constrained;
use codex_config::RequirementSource;
use codex_config::Sourced;
use codex_protocol::config_types::ApprovalsReviewer;
use pretty_assertions::assert_eq;
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug)]
enum Forced {
    No,
    ModelRequiresReview,
    PersonMayNotReview,
}

async fn reviewer_for(forced: Forced, requirement: ExecApprovalRequirement) -> ApprovalReviewer {
    let (_, probe, _) = crate::session::tests::make_session_and_context_with_rx().await;
    let model = probe.model_info().slug.clone();
    let (session, turn, _events) =
        crate::session::tests::make_session_and_context_with_auth_and_config_and_rx(
            codex_login::CodexAuth::from_api_key("Test API Key"),
            Vec::new(),
            move |config| {
                config
                    .permissions
                    .approval_policy
                    .set(AskForApproval::OnRequest)
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
    let step = StepContext::for_test(Arc::clone(&turn));
    // The step itself still names the person; only the requirements force the reviewer.
    assert_eq!(step.settings.approvals_reviewer(), ApprovalsReviewer::User);
    let ctx = ApprovalContext {
        exec_requirement: Some(requirement),
        review_context: GuardianReviewContext::from(&step),
        cancellation_token: None,
        call_id: "call-forced".to_string(),
        tool_name: ToolName::plain("exec_command"),
        strict_auto_review: false,
        approval_reason: None,
        retry_reason: None,
        network_approval_context: None,
    };
    assert_eq!(ApprovalReviewer::for_context(&ctx), ApprovalReviewer::User);
    ApprovalReviewer::for_session_context(&session, &ctx).await
}

fn needs_approval() -> ExecApprovalRequirement {
    ExecApprovalRequirement::NeedsApproval {
        reason: None,
        proposed_execpolicy_amendment: None,
    }
}

#[tokio::test]
async fn requirement_forced_review_is_not_labelled_the_persons() {
    assert_eq!(
        reviewer_for(Forced::No, needs_approval()).await,
        ApprovalReviewer::User
    );
    assert_eq!(
        reviewer_for(Forced::ModelRequiresReview, needs_approval()).await,
        ApprovalReviewer::Guardian
    );
    assert_eq!(
        reviewer_for(Forced::PersonMayNotReview, needs_approval()).await,
        ApprovalReviewer::Guardian
    );
}

#[tokio::test]
async fn human_only_requirements_stay_with_the_person_when_review_is_forced() {
    for forced in [Forced::ModelRequiresReview, Forced::PersonMayNotReview] {
        assert_eq!(
            reviewer_for(
                forced,
                ExecApprovalRequirement::NeedsHumanApproval { reason: None }
            )
            .await,
            ApprovalReviewer::User,
            "{forced:?}"
        );
    }
}
