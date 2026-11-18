//! Native Windows shell-tool proof: unread scripts reach approval before execution.
use super::*;
use base64::Engine as _;
use codex_core::shell::get_shell_by_model_provided_path;

#[tokio::test]
async fn issue2556_unread_powershell_asks_before_writing() -> Result<()> {
    let powershell = codex_shell_command::powershell::try_find_powershell_executable_blocking()
        .expect("native Windows PowerShell");
    let dynamic = "& ('Set-'+'Content') marker.txt ran";
    let encoded = base64::engine::general_purpose::STANDARD.encode(
        dynamic
            .encode_utf16()
            .flat_map(u16::to_le_bytes)
            .collect::<Vec<_>>(),
    );
    for script in [
        dynamic.to_string(),
        format!("powershell.exe -EncodedCommand {encoded}"),
        "powershell.exe -Command \"& ('Set-'+'Content')\" \"marker.txt ran\"".to_string(),
        "'Set-Content marker.txt ran' | powershell.exe -NoProfile".to_string(),
    ] {
        let server = start_mock_server().await;
        let mut builder = test_codex()
            .with_user_shell(get_shell_by_model_provided_path(
                &powershell.as_path().to_path_buf(),
            ))
            .with_config(|config| {
                let rules = config.codex_home.join("rules");
                fs::create_dir_all(&rules).unwrap();
                fs::write(
                    rules.join("powershell.rules"),
                    r#"prefix_rule(pattern=["Set-Content"], decision="forbidden")"#,
                )
                .unwrap();
            });
        // shell_command always executes on the host; this test requires its
        // native Windows PowerShell parser, not a remote executor.
        let test = builder.build(&server).await?;
        let args = json!({"command": script, "timeout_ms": 1_000});
        let _request = mount_sse_once(
            &server,
            sse(vec![
                ev_response_created("ps-unread-1"),
                ev_function_call("ps-unread", "shell_command", &serde_json::to_string(&args)?),
                ev_completed("ps-unread-1"),
            ]),
        )
        .await;
        let response = mount_sse_once(
            &server,
            sse(vec![
                ev_assistant_message("ps-unread-done", "done"),
                ev_completed("ps-unread-2"),
            ]),
        )
        .await;
        submit_user_turn(
            &test,
            "run the command",
            AskForApproval::OnRequest,
            PermissionProfile::Disabled,
            /*collaboration_mode*/ None,
        )
        .await?;
        let event = wait_for_event(&test.codex, |event| {
            matches!(
                event,
                EventMsg::ExecApprovalRequest(_) | EventMsg::TurnComplete(_)
            )
        })
        .await;
        let EventMsg::ExecApprovalRequest(approval) = event else {
            panic!("unread script did not ask: {script}");
        };
        assert!(approval.proposed_execpolicy_amendment.is_none());
        assert!(!test.config.cwd.join("marker.txt").exists(), "{script}");
        test.codex
            .submit(Op::ExecApproval {
                id: approval.effective_approval_id(),
                turn_id: None,
                decision: ReviewDecision::denied("test denies unread script"),
            })
            .await?;
        wait_for_event(&test.codex, |event| {
            matches!(event, EventMsg::TurnComplete(_))
        })
        .await;
        let output = response.single_request().function_call_output("ps-unread");
        assert!(output["output"].as_str().is_some(), "{output:?}");
        assert!(!test.config.cwd.join("marker.txt").exists(), "{script}");
    }
    Ok(())
}
