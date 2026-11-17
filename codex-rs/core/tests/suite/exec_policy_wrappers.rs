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

#[tokio::test]
async fn issue2455_shell_tool_rejects_forbidden_command_behind_unshare_chroot() -> Result<()> {
    let server = start_mock_server().await;
    let mut builder = test_codex().with_config(|config| {
        let rules = config.codex_home.join("rules");
        fs::create_dir_all(&rules).unwrap();
        fs::write(
            rules.join("wrappers.rules"),
            r#"prefix_rule(pattern=["unshare"], decision="allow")
prefix_rule(pattern=["echo", "blocked"], decision="forbidden")"#,
        )
        .unwrap();
    });
    // `shell_command` is a local-only tool. Keep exercising its exec-policy path
    // even when the suite runs with a remote exec-server environment.
    let test = builder.build(&server).await?;
    let args = json!({
        "command": "unshare -r chroot / echo blocked",
        "timeout_ms": 1_000,
    });
    let _command_request = mount_sse_once(
        &server,
        sse(vec![
            ev_response_created("resp-unshare-1"),
            ev_function_call(
                "unshare-forbidden",
                "shell_command",
                &serde_json::to_string(&args)?,
            ),
            ev_completed("resp-unshare-1"),
        ]),
    )
    .await;
    let response = mount_sse_once(
        &server,
        sse(vec![
            ev_assistant_message("msg-unshare-1", "done"),
            ev_completed("resp-unshare-2"),
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
        .function_call_output("unshare-forbidden");
    let output = output["output"].as_str().expect("tool output");
    assert!(
        output.contains("policy forbids commands starting with `echo blocked`"),
        "unexpected output: {output}"
    );
    Ok(())
}

/// #2455 follow-up (D670): the shell tool refuses a forbidden command run
/// through the shell builtins `command`/`exec`, `time`, a busybox applet,
/// `systemd-run`, `pkexec` or `bwrap`, even when the launcher is allowed.
#[tokio::test]
async fn issue2455b_shell_tool_rejects_forbidden_command_behind_more_launchers() -> Result<()> {
    for (launcher, line) in [
        ("command", "command -p echo blocked"),
        ("exec", "exec echo blocked"),
        ("time", "time -p echo blocked"),
        ("busybox", "busybox echo blocked"),
        ("systemd-run", "systemd-run --scope --user -q echo blocked"),
        ("pkexec", "pkexec --user root echo blocked"),
        ("bwrap", "bwrap --ro-bind / / --dev /dev echo blocked"),
    ] {
        let server = start_mock_server().await;
        let mut builder = test_codex().with_config(move |config| {
            let rules = config.codex_home.join("rules");
            fs::create_dir_all(&rules).unwrap();
            fs::write(
                rules.join("wrappers.rules"),
                format!(
                    r#"prefix_rule(pattern=["{launcher}"], decision="allow")
prefix_rule(pattern=["echo", "blocked"], decision="forbidden")"#
                ),
            )
            .unwrap();
        });
        // `shell_command` is a local-only tool; see the tests above.
        let test = builder.build(&server).await?;
        let args = json!({
            "command": line,
            "timeout_ms": 1_000,
        });
        let _command_request = mount_sse_once(
            &server,
            sse(vec![
                ev_response_created("resp-launcher-1"),
                ev_function_call(
                    "launcher-forbidden",
                    "shell_command",
                    &serde_json::to_string(&args)?,
                ),
                ev_completed("resp-launcher-1"),
            ]),
        )
        .await;
        let response = mount_sse_once(
            &server,
            sse(vec![
                ev_assistant_message("msg-launcher-1", "done"),
                ev_completed("resp-launcher-2"),
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
            .function_call_output("launcher-forbidden");
        let output = output["output"].as_str().expect("tool output");
        assert!(
            output.contains("policy forbids commands starting with `echo blocked`"),
            "{line}: unexpected output: {output}"
        );
    }
    Ok(())
}
