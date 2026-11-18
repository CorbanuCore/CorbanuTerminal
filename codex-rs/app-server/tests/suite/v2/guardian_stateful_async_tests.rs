use super::*;
use codex_protocol::openai_models::AsyncClassifierMode;
use test_case::test_case;

#[test_case(GuardianRisk::Low, ThreadLifecycle::New, ModelReviewRequirement::Optional => ignore["D604 (#2562): background scoring stays off until disclosure-gated"]; "low")]
#[test_case(GuardianRisk::Low, ThreadLifecycle::ReviewContinuations, ModelReviewRequirement::Optional => ignore["D604 (#2562): background scoring stays off until disclosure-gated"]; "retained_sync_review_delivered_once")]
#[test_case(GuardianRisk::High, ThreadLifecycle::ReviewContinuations, ModelReviewRequirement::Optional => ignore["D604 (#2562): background scoring stays off until disclosure-gated"]; "new_sync_review_appended")]
#[test_case(GuardianRisk::LowWithStreamFailure, ThreadLifecycle::ReviewContinuations, ModelReviewRequirement::Optional => ignore["D604 (#2562): background scoring stays off until disclosure-gated"]; "sync_review_restored_after_failed_stream")]
#[test_case(GuardianRisk::Low, ThreadLifecycle::RootUserRestriction, ModelReviewRequirement::Optional => ignore["D604 (#2562): background scoring stays off until disclosure-gated"]; "new_user_instruction_resets_conversation")]
#[test_case(GuardianRisk::LowWithStreamFailure, ThreadLifecycle::New, ModelReviewRequirement::Optional => ignore["D604 (#2562): background scoring stays off until disclosure-gated"]; "failure_after_early_score")]
#[test_case(GuardianRisk::Low, ThreadLifecycle::RootRestrictionDuringClassification, ModelReviewRequirement::Optional => ignore["D604 (#2562): background scoring stays off until disclosure-gated"]; "authorization_changed_during_classification")]
#[test_case(GuardianRisk::High, ThreadLifecycle::New, ModelReviewRequirement::Optional => ignore["D604 (#2562): background scoring stays off until disclosure-gated"]; "high")]
#[test_case(GuardianRisk::InvalidResponse, ThreadLifecycle::New, ModelReviewRequirement::Optional => ignore["D604 (#2562): background scoring stays off until disclosure-gated"]; "failed")]
#[test_case(GuardianRisk::Low, ThreadLifecycle::Resume, ModelReviewRequirement::Optional => ignore["D604 (#2562): background scoring stays off until disclosure-gated"]; "resume")]
#[test_case(GuardianRisk::Low, ThreadLifecycle::Fork, ModelReviewRequirement::Optional => ignore["D604 (#2562): background scoring stays off until disclosure-gated"]; "fork")]
#[test_case(GuardianRisk::Low, ThreadLifecycle::New, ModelReviewRequirement::Required; "mandatory_review")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn conversation_classifier_preserves_approval_matrix(
    risk: GuardianRisk,
    lifecycle: ThreadLifecycle,
    requirement: ModelReviewRequirement,
) -> Result<()> {
    skip_if_no_network!(Ok(()));
    guardian_v2_routes_scoped_tool_approvals(
        risk,
        lifecycle,
        requirement,
        ReviewOutcome::Allow,
        TranscriptContent::Normal,
        GuardianToolScope::AllTools,
        /*sensitive_action*/ None,
        AsyncClassifierMode::Conversation,
    )
    .await?;
    Ok(())
}
