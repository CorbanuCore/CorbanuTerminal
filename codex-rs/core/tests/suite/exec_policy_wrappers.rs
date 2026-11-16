use super::*;

#[tokio::test]
async fn issue2165_shell_tool_rejects_wrapped_forbidden_command() -> Result<()> {
    let server = start_mock_server().await;
    let mut builder = test_codex().with_config(|config| {
        let rules = config.codex_home.join("rules");
        fs::create_dir_all(&rules).unwrap();
        fs::write(
            rules.join("wrappers.rules"),
            r#"prefix_rule(pattern=["env"], decision="allow")
prefix_rule(pattern=["echo", "blocked"], decision="forbidden")"#,
        )
        .unwrap();
    });
    let test = builder.build_with_auto_env(&server).await?;
    // Sync: upstream replaced `shell_command` with `exec_command`.
    let args = json!({
        "cmd": "env FOO=1 echo blocked",
        "yield_time_ms": 1_000,
    });
    let _command_request = mount_sse_once(
        &server,
        sse(vec![
            ev_response_created("resp-wrapper-1"),
            ev_function_call(
                "wrapped-forbidden",
                "exec_command",
                &serde_json::to_string(&args)?,
            ),
            ev_completed("resp-wrapper-1"),
        ]),
    )
    .await;
    let response = mount_sse_once(
        &server,
        sse(vec![
            ev_assistant_message("msg-wrapper-1", "done"),
            ev_completed("resp-wrapper-2"),
        ]),
    )
    .await;
    submit_user_turn(
        &test,
        "run the command",
        AskForApproval::Never,
        PermissionProfile::Disabled,
        /*collaboration_mode*/ None,
    )
    .await?;
    wait_for_event(&test.codex, |event| {
        matches!(event, EventMsg::TurnComplete(_))
    })
    .await;
    let output = response
        .single_request()
        .function_call_output("wrapped-forbidden");
    let output = output["output"].as_str().expect("tool output");
    assert!(
        output.contains("policy forbids commands starting with `echo blocked`"),
        "unexpected output: {output}"
    );
    Ok(())
}
