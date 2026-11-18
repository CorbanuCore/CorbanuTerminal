//! D618 (#2562): a request to run outside the sandbox is only offered under OnRequest.

use super::*;
use crate::sandboxing::SandboxPermissions;
use crate::turn_diff_tracker::TurnDiffTracker;
use codex_protocol::protocol::AskForApproval;
use codex_protocol::protocol::GranularApprovalConfig;
use pretty_assertions::assert_eq;

fn granular(sandbox_approval: bool) -> AskForApproval {
    AskForApproval::Granular(GranularApprovalConfig {
        sandbox_approval,
        rules: true,
        skill_approval: true,
        request_permissions: true,
        mcp_elicitations: true,
    })
}

async fn escalated_exec_result(policy: AskForApproval) -> Result<(), String> {
    let (session, mut turn_context_raw) = make_session_and_context().await;
    Arc::make_mut(&mut turn_context_raw.config)
        .permissions
        .approval_policy
        .set(policy)
        .expect("test setup should allow updating approval policy");
    let session = Arc::new(session);
    let turn_context = Arc::new(turn_context_raw);
    let step_context = StepContext::for_test(Arc::clone(&turn_context));
    let handler = ExecCommandHandler::default();
    let handled = handler.handle(ToolInvocation {
        session,
        turn: Arc::clone(&turn_context),
        step_context,
        cancellation_token: CancellationToken::new(),
        tracker: Arc::new(tokio::sync::Mutex::new(TurnDiffTracker::new())),
        call_id: "exec-call".to_string(),
        tool_name: codex_tools::ToolName::plain("exec_command"),
        source: crate::tools::context::ToolCallSource::Direct,
        native_call: None,
        environments: Default::default(),
        payload: ToolPayload::Function {
            arguments: serde_json::json!({
                "cmd": "echo hi",
                "sandbox_permissions": SandboxPermissions::RequireEscalated,
                "justification": "need unsandboxed execution",
            })
            .to_string(),
        },
    });
    // A request that is not refused waits for the approval answer; that is the
    // OnRequest path, so a bounded wait counts as "reached the approval flow".
    let Ok(resp) = tokio::time::timeout(std::time::Duration::from_secs(10), handled).await else {
        return Ok(());
    };
    match resp {
        Err(FunctionCallError::RespondToModel(output))
            if output.contains("you cannot ask for escalated permissions") =>
        {
            Err(output)
        }
        _ => Ok(()),
    }
}

#[tokio::test]
async fn escalation_is_refused_under_every_policy_except_on_request() {
    for policy in [
        AskForApproval::Never,
        AskForApproval::UnlessTrusted,
        granular(/*sandbox_approval*/ true),
        granular(/*sandbox_approval*/ false),
    ] {
        assert_eq!(
            escalated_exec_result(policy).await,
            Err(format!(
                "approval policy is {policy:?}; reject command — you cannot ask for escalated permissions if the approval policy is {policy:?}"
            )),
        );
    }
}

#[tokio::test]
async fn escalation_still_reaches_the_approval_flow_under_on_request() {
    assert_eq!(
        escalated_exec_result(AskForApproval::OnRequest).await,
        Ok(())
    );
}
