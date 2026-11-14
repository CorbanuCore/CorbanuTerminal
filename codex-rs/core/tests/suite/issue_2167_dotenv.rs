//! #841/#2167: project credential reads ask after untrusted tool output.
use super::*;
use codex_protocol::models::PermissionProfile;
use codex_protocol::protocol::ReviewDecision;
use pretty_assertions::assert_eq;
use test_case::test_case;

const SECRET: &str = "SYNTHETIC_2167=fixture_only";

async fn dotenv_turn(
    level: SecurityLevel,
    approval: AskForApproval,
    step: Step,
    tainted: bool,
) -> anyhow::Result<(TestCodex, ResponseMock)> {
    let mut steps = vec![step, done_step()];
    if tainted {
        steps.insert(0, injected_step());
    }
    start_turn_with(
        /*home*/ None,
        level,
        approval,
        steps,
        move |config| {
            let (mut file_system, network) =
                PermissionProfile::workspace_write().to_runtime_permissions();
            if level == SecurityLevel::Aggressive {
                use codex_protocol::permissions::FileSystemAccessMode;
                use codex_protocol::permissions::FileSystemPath;
                use codex_protocol::permissions::FileSystemSandboxEntry;
                use codex_protocol::permissions::credential_file_globs;
                file_system.entries.extend(
                    credential_file_globs(&config.cwd.to_string_lossy())
                        .into_iter()
                        .map(|pattern| {
                            FileSystemSandboxEntry::new(
                                FileSystemPath::GlobPattern {
                                    pattern,
                                    scan: None,
                                },
                                FileSystemAccessMode::Deny,
                            )
                        }),
                );
            }
            config
                .permissions
                .set_permission_profile(PermissionProfile::from_runtime_permissions(
                    &file_system,
                    network,
                ))
                .expect("fixture workspace");
            core_test_support::use_elevated_windows_sandbox(config);
            insert_rmcp_server(config).expect("fixture MCP server");
        },
        |test| {
            std::fs::write(test.cwd.path().join(".env"), SECRET).expect("synthetic credential");
        },
        /*pin_cwd_as_workspace_root*/ true,
    )
    .await
}

async fn exec_question(
    test: &TestCodex,
) -> Option<codex_protocol::protocol::ExecApprovalRequestEvent> {
    match wait_for_event(&test.codex, |event| {
        matches!(
            event,
            EventMsg::ExecApprovalRequest(_) | EventMsg::TurnComplete(_)
        )
    })
    .await
    {
        EventMsg::ExecApprovalRequest(request) => Some(request),
        _ => None,
    }
}

// Image errors use structured content items; shell/MCP outputs may be strings.
fn output_text(request: &ResponsesRequest, call_id: &str) -> String {
    request
        .function_call_output_content_and_success(call_id)
        .and_then(|(content, _)| content)
        .expect("tool output text")
}

fn read_step() -> Step {
    call("call-read", "shell_command", json!({"command": "cat .env"}))
}

#[test_case(SecurityLevel::Moderate; "moderate")]
#[test_case(SecurityLevel::Aggressive; "aggressive")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[cfg_attr(windows, serial_test::serial(codex_home))]
async fn issue_2167_shell_read_asks_and_cancel_keeps_value_out(
    level: SecurityLevel,
) -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    let (test, captured) = dotenv_turn(
        level,
        AskForApproval::OnRequest,
        read_step(),
        /*tainted*/ true,
    )
    .await?;
    let request = exec_question(&test).await.expect("dotenv read must ask");
    assert!(
        request
            .reason
            .as_deref()
            .unwrap_or_default()
            .contains("credential access after untrusted content")
    );
    assert_eq!(request.proposed_execpolicy_amendment, None);
    test.codex
        .submit(Op::ExecApproval {
            id: request.effective_approval_id(),
            turn_id: None,
            decision: ReviewDecision::denied("fixture declined"),
        })
        .await?;
    assert!(exec_question(&test).await.is_none());
    let output = output_text(captured.requests().last().expect("result"), "call-read");
    assert!(!output.contains(SECRET), "{output}");
    assert!(output.contains("fixture declined"), "{output}");
    Ok(())
}

#[test_case(SecurityLevel::Moderate, true; "moderate_allows")]
#[test_case(SecurityLevel::Aggressive, false; "aggressive_still_denies")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[cfg_attr(windows, serial_test::serial(codex_home))]
async fn issue_2167_fresh_approval_respects_level_read_denials(
    level: SecurityLevel,
    readable: bool,
) -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    let (test, captured) = dotenv_turn(
        level,
        AskForApproval::OnRequest,
        read_step(),
        /*tainted*/ true,
    )
    .await?;
    let request = exec_question(&test).await.expect("dotenv read must ask");
    test.codex
        .submit(Op::ExecApproval {
            id: request.effective_approval_id(),
            turn_id: None,
            decision: ReviewDecision::Approved,
        })
        .await?;
    assert!(exec_question(&test).await.is_none());
    let output = output_text(captured.requests().last().expect("result"), "call-read");
    assert_eq!(output.contains(SECRET), readable, "{level:?}: {output}");
    Ok(())
}

#[test_case(SecurityLevel::Moderate, true, false; "moderate_tainted")]
#[test_case(SecurityLevel::Aggressive, true, false; "aggressive_tainted")]
#[test_case(SecurityLevel::Permissive, true, true; "permissive_unchanged")]
#[test_case(SecurityLevel::Moderate, false, true; "moderate_before_taint")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[cfg_attr(windows, serial_test::serial(codex_home))]
async fn issue_2167_shell_read_with_approvals_off(
    level: SecurityLevel,
    tainted: bool,
    readable: bool,
) -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    let (test, captured) = dotenv_turn(level, AskForApproval::Never, read_step(), tainted).await?;
    assert!(exec_question(&test).await.is_none());
    let output = output_text(captured.requests().last().expect("result"), "call-read");
    assert_eq!(output.contains(SECRET), readable, "{level:?}: {output}");
    assert_eq!(
        output.contains("approvals are off"),
        !readable,
        "{level:?}: {output}"
    );
    Ok(())
}

// The read-only MCP echo fixture proves dispatch never reaches the server
// before approval; the classifier tests exercise actual read_file arguments.
#[test_case(SecurityLevel::Moderate; "moderate")]
#[test_case(SecurityLevel::Aggressive; "aggressive")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[cfg_attr(windows, serial_test::serial(codex_home))]
async fn issue_2167_mcp_file_path_asks_before_dispatch(level: SecurityLevel) -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    let (test, captured) = dotenv_turn(
        level,
        AskForApproval::OnRequest,
        mcp_step(".env"),
        /*tainted*/ true,
    )
    .await?;
    let request = next_question(&test)
        .await
        .expect("MCP credential path must ask");
    assert!(
        request.questions[0]
            .question
            .contains("credential access after untrusted content")
    );
    answer(&test, &request, "Cancel").await;
    assert!(next_question(&test).await.is_none());
    let output = output_text(captured.requests().last().expect("result"), "call-mcp");
    assert!(
        output.contains("you declined credential access"),
        "{output}"
    );
    Ok(())
}

#[test_case(SecurityLevel::Moderate; "moderate")]
#[test_case(SecurityLevel::Aggressive; "aggressive")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[cfg_attr(windows, serial_test::serial(codex_home))]
async fn issue_2167_builtin_file_read_asks_before_open(level: SecurityLevel) -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    let (test, captured) = dotenv_turn(
        level,
        AskForApproval::OnRequest,
        call("call-image", "view_image", json!({"path": ".env"})),
        /*tainted*/ true,
    )
    .await?;
    let request = next_question(&test)
        .await
        .expect("builtin credential read must ask");
    assert!(
        request.questions[0]
            .question
            .contains("credential access after untrusted content")
    );
    answer(&test, &request, "Cancel").await;
    assert!(next_question(&test).await.is_none());
    let output = output_text(captured.requests().last().expect("result"), "call-image");
    assert!(
        output.contains("you declined credential access"),
        "{output}"
    );
    assert!(!output.contains(SECRET), "{output}");
    Ok(())
}

#[test_case("mcp"; "mcp")]
#[test_case("image"; "builtin")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[cfg_attr(windows, serial_test::serial(codex_home))]
async fn issue_2167_permissive_file_routes_are_unchanged(route: &str) -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    let step = if route == "mcp" {
        mcp_step(".env")
    } else {
        call("call-image", "view_image", json!({"path": ".env"}))
    };
    let (test, captured) = dotenv_turn(
        SecurityLevel::Permissive,
        AskForApproval::Never,
        step,
        /*tainted*/ true,
    )
    .await?;
    assert!(next_question(&test).await.is_none());
    let call_id = if route == "mcp" {
        "call-mcp"
    } else {
        "call-image"
    };
    let output = output_text(captured.requests().last().expect("result"), call_id);
    assert!(!output.contains("approvals are off"), "{output}");
    if route == "mcp" {
        assert!(output.contains(".env"), "{output}");
    } else {
        // The read succeeds; the synthetic dotenv text is not an image.
        assert!(
            output.contains("image content omitted because it could not be processed"),
            "{output}"
        );
    }
    Ok(())
}
