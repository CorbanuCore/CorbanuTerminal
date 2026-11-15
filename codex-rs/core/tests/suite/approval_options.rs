//! #2213 exercises the client reply through `exec_command` (upstream has no
//! separate `shell_command` runtime).
use super::*;

#[tokio::test]
async fn issue_2213_unoffered_session_answer_neither_runs_nor_caches_commands() -> Result<()> {
    skip_if_no_network!(Ok(()));
    let server = start_mock_server().await;
    let test = test_codex()
        .with_model("gpt-5.4")
        .with_config(move |config| {
            config.permissions.approval_policy =
                Constrained::allow_any(AskForApproval::UnlessTrusted);
            config
                .set_legacy_sandbox_policy(SandboxPolicy::DangerFullAccess)
                .unwrap();
            config.approvals_reviewer = ApprovalsReviewer::User;
            config.features.enable(Feature::UnifiedExec).unwrap();
        })
        // Native shell and filesystem observation are the subject of this test.
        .build(&server)
        .await?;
    let command = "echo issue-2213-ran > issue-2213.txt";
    let marker = test.workspace_path("issue-2213.txt");
    for (call_id, answer) in [
        ("unoffered-first", ReviewDecision::ApprovedForSession),
        ("unoffered-repeat", ReviewDecision::ApprovedForSession),
        ("offered-once", ReviewDecision::Approved),
    ] {
        let event = exec_command_event(
            call_id,
            command,
            Some(1000),
            SandboxPermissions::UseDefault,
            /*justification*/ None,
        )?;
        let call_mock = mount_sse_once(
            &server,
            sse(vec![
                ev_response_created("call"),
                event,
                ev_completed("call"),
            ]),
        )
        .await;
        let result_mock = mount_sse_once(
            &server,
            sse(vec![
                ev_assistant_message("done", "done"),
                ev_completed("done"),
            ]),
        )
        .await;
        submit_turn(
            &test,
            "run the command",
            AskForApproval::UnlessTrusted,
            SandboxPolicy::DangerFullAccess,
        )
        .await?;
        let prompt = expect_exec_approval(&test, command).await;
        assert!(
            !prompt
                .effective_available_decisions()
                .contains(&ReviewDecision::ApprovedForSession)
        );
        test.codex
            .submit(Op::ExecApproval {
                id: prompt.effective_approval_id(),
                turn_id: Some(prompt.turn_id),
                decision: answer.clone(),
            })
            .await?;
        wait_for_completion(&test).await;
        let _ = call_mock.single_request();
        let result = result_mock.single_request().function_call_output(call_id);
        if answer == ReviewDecision::Approved {
            assert!(fs::read_to_string(&marker)?.contains("issue-2213-ran"));
        } else {
            assert!(!marker.exists(), "unoffered approval ran the command");
            assert!(result.to_string().contains("not offered"), "{result}");
        }
    }
    test.codex.shutdown_and_wait().await?;
    Ok(())
}
