//! Corbanu owner decisions D604/D605/D606 (#2562): background scoring never builds its
//! transcript transport, a serving model's "no review needed" answer never approves, and
//! the Decisions comparison sampler is removed.

use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use codex_core::config::Config;
use codex_extension_api::ApprovalDecision;
use codex_extension_api::ApprovalDecisionInput;
use codex_extension_api::ExtensionData;
use codex_extension_api::ExtensionFuture;
use codex_extension_api::ExtensionRegistry;
use codex_extension_api::ExtensionRegistryBuilder;
use codex_extension_api::GuardianV2Enabled;
use codex_extension_api::SynchronousApprovalReviewer;
use codex_extension_api::ThreadStartInput;
use codex_extension_api::ToolCallSource;
use codex_extension_api::ToolName;
use codex_extension_api::ToolPayload;
use codex_extension_api::ToolStartInput;
use codex_features::Feature;
use codex_login::AuthManager;
use codex_login::CodexAuth;
use codex_model_provider_info::ModelProviderInfo;
use codex_protocol::ThreadId;
use codex_protocol::approvals::GuardianReviewReason;
use codex_protocol::config_types::ApprovalsReviewer;
use codex_protocol::openai_models::GuardianReviewMode;
use codex_protocol::openai_models::GuardianScope;
use codex_protocol::openai_models::ModelInfo;
use codex_protocol::protocol::AskForApproval;
use codex_protocol::protocol::ReviewDecision;
use codex_protocol::protocol::SessionSource;
use core_test_support::responses;
use core_test_support::responses::WebSocketConnectionConfig;
use core_test_support::skip_if_no_network;
use core_test_support::test_codex::TestCodex;
use core_test_support::test_codex::test_codex;
use pretty_assertions::assert_eq;

use super::BACKGROUND_SCORING_DISCLOSURE_GATED;
use crate::async_scorer::config::GuardianV2Config;
use crate::async_scorer::conversation::ConversationBackend;
use crate::async_scorer::sampler::INITIAL_WEBSOCKET_CONNECTIONS;
use crate::async_scorer::sampler::LunaSampler;

/// Records the reason it was asked for, then answers like a denying reviewer.
#[derive(Default)]
struct RecordingReviewer(std::sync::Mutex<Vec<GuardianReviewReason>>);

impl SynchronousApprovalReviewer for RecordingReviewer {
    fn review<'a>(
        &'a self,
        reason: GuardianReviewReason,
        _async_approval: Option<ExtensionFuture<'a, ()>>,
    ) -> ExtensionFuture<'a, Option<ReviewDecision>> {
        self.0.lock().unwrap().push(reason);
        Box::pin(async { Some(ReviewDecision::denied("synchronous review")) })
    }
}

struct Started {
    test: TestCodex,
    registry: ExtensionRegistry<Config>,
    session_store: ExtensionData,
    server: responses::WebSocketTestServer,
}

/// Starts a thread whose resolved Guardian policy has adaptive (scored) coverage.
async fn start_thread_with_scoring_policy() -> Result<Started> {
    let thread_server = responses::start_mock_server().await;
    let test = test_codex()
        .with_config(|config| config.approvals_reviewer = ApprovalsReviewer::AutoReview)
        .build_with_auto_env(&thread_server)
        .await?;
    let connections = vec![
        WebSocketConnectionConfig {
            requests: Vec::new(),
            response_headers: Vec::new(),
            accept_delay: None,
            close_after_requests: true,
        };
        INITIAL_WEBSOCKET_CONNECTIONS
    ];
    let server = responses::start_websocket_server_with_headers(connections).await;
    let mut config = test.config.clone();
    config.model_provider = ModelProviderInfo::create_openai_provider(Some(format!(
        "http://{}/v1",
        server.uri().trim_start_matches("ws://")
    )));
    config.features.enable(Feature::GuardianV2)?;
    let mut builder = ExtensionRegistryBuilder::new();
    super::install(
        &mut builder,
        AuthManager::from_auth_for_testing(CodexAuth::from_api_key("test-api-key")),
        Arc::downgrade(&test.thread_manager),
    );
    let registry = builder.build();
    let session_store = ExtensionData::new("session-1");
    let thread_store = test.codex.thread_extension_data();
    // Keeps computer use adaptive for this model, so the approval path below is scored.
    let mut model = thread_store.get::<ModelInfo>().unwrap().as_ref().clone();
    model.node_repl_auto_review_required = true;
    thread_store.insert(model);
    registry.thread_lifecycle_contributors()[0]
        .on_thread_start(ThreadStartInput {
            config: &config,
            session_source: &SessionSource::Exec,
            persistent_thread_state_available: false,
            environments: &[],
            mcp_resource_client: None,
            extension_metrics: None,
            session_store: &session_store,
            thread_store,
        })
        .await;
    // The policy would score without the D604 gate, so the absences below are meaningful.
    let policy = thread_store
        .get::<GuardianV2Config>()
        .expect("Guardian configuration should still load")
        .policy_for_model(thread_store.get::<ModelInfo>().as_deref());
    assert!(policy.scoring_enabled());
    assert_eq!(
        policy.review_mode(GuardianScope::ComputerUse),
        GuardianReviewMode::Adaptive
    );
    Ok(Started {
        test,
        registry,
        session_store,
        server,
    })
}

#[test]
fn background_scoring_stays_off_until_disclosure_gated() {
    assert!(!BACKGROUND_SCORING_DISCLOSURE_GATED);
}

/// D606: the Decisions comparison sampler is gone; its feature key only still parses.
#[test]
fn d606_decisions_comparison_feature_is_removed() {
    assert_eq!(
        Feature::GuardianV2DecisionsComparison.stage(),
        codex_features::Stage::Removed
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn d604_scoring_policy_never_builds_or_uses_the_transcript_transport() -> Result<()> {
    skip_if_no_network!(Ok(()));
    let started = start_thread_with_scoring_policy().await?;
    let thread_store = started.test.codex.thread_extension_data();

    assert!(thread_store.get::<LunaSampler>().is_none());
    assert!(thread_store.get::<ConversationBackend>().is_none());
    assert!(thread_store.get::<GuardianV2Enabled>().is_none());

    let turn_store = ExtensionData::new("turn-1");
    for (call_id, tool_name, arguments) in [
        (
            "call-read",
            ToolName::plain("read_file"),
            r#"{"path":"README.md"}"#,
        ),
        (
            "call-shell",
            ToolName::plain("exec_command"),
            r#"{"cmd":"ls"}"#,
        ),
    ] {
        let payload = ToolPayload::Function {
            arguments: arguments.to_owned(),
        };
        started.registry.tool_lifecycle_contributors()[0]
            .on_tool_start(ToolStartInput {
                serving_assessment: None,
                permissions: Box::pin(async { Some(Default::default()) }),
                session_store: &started.session_store,
                thread_store,
                turn_store: &turn_store,
                turn_id: "turn-1",
                root_turn_id: Some("root-turn"),
                call_id,
                originating_item_id: None,
                tool_name: &tool_name,
                mcp_tool: None,
                payload: &payload,
                conversation_history: started.test.codex.conversation_history_snapshot().await,
                source: ToolCallSource::Direct,
            })
            .await;
    }
    // Give any (forbidden) background prewarm or scoring task time to connect.
    tokio::time::sleep(Duration::from_millis(500)).await;

    assert_eq!(started.server.handshakes().len(), 0);
    assert!(thread_store.get::<LunaSampler>().is_none());
    assert!(thread_store.get::<GuardianV2Enabled>().is_none());
    started.test.codex.shutdown_and_wait().await?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn d605_serving_no_review_needed_never_approves_an_action() -> Result<()> {
    skip_if_no_network!(Ok(()));
    let started = start_thread_with_scoring_policy().await?;
    let thread_store = started.test.codex.thread_extension_data();
    let action = serde_json::json!({
        "tool": "computer_use",
        "tool_name": "js",
        "server": "node_repl",
        "id": "call-js",
    });

    for serving_assessment in [Some(false), Some(true)] {
        let reviewer = RecordingReviewer::default();
        let input = ApprovalDecisionInput {
            serving_assessment,
            permissions: Some(&Default::default()),
            approval_id: "approval-1",
            tool_call_id: Some("call-js"),
            action: &action,
            thread_id: ThreadId::from_string(thread_store.level_id())?,
            thread_store,
            category: GuardianScope::ComputerUse,
            approval_policy: AskForApproval::OnRequest,
            approvals_reviewer: ApprovalsReviewer::AutoReview,
            require_guardian: false,
            require_fresh_review: false,
            full_access: false,
            metrics: None,
            synchronous_reviewer: &reviewer,
        };
        let decision = started.registry.decide_approval(&input).await;

        assert_eq!(
            decision,
            Some(ApprovalDecision::Reviewed(ReviewDecision::denied(
                "synchronous review"
            ))),
            "serving assessment {serving_assessment:?} must not skip review"
        );
        let reasons = reviewer.0.lock().unwrap().clone();
        assert_eq!(
            reasons.len(),
            1,
            "exactly one synchronous review: {reasons:?}"
        );
        if serving_assessment == Some(true) {
            assert_eq!(reasons[0], GuardianReviewReason::ElevatedRisk);
        }
    }
    started.test.codex.shutdown_and_wait().await?;
    Ok(())
}
