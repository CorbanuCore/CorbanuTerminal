//! Session-level post-taint gates; every executable is a harmless local fixture.
use super::*;
use codex_protocol::models::PermissionProfile;
use codex_protocol::protocol::ReviewDecision;
use pretty_assertions::assert_eq;
use std::os::unix::fs::PermissionsExt;

async fn assignment_turn(
    prefix: &str,
    level: SecurityLevel,
    policy: AskForApproval,
) -> anyhow::Result<(TestCodex, ResponseMock)> {
    let separator = if prefix.ends_with('&') { " " } else { "; " };
    let command = format!("{prefix}{separator}./git fetch e\"$r\"");
    start_turn_with(
        /*home*/ None,
        level,
        policy,
        vec![
            injected_step(),
            call("call-fetch", "shell_command", json!({"command": command})),
            done_step(),
        ],
        |config| {
            config
                .permissions
                .set_permission_profile(PermissionProfile::workspace_write())
                .expect("fixture workspace is writable");
        },
        |test| {
            // A name-compatible fixture, not the host Git binary: no network,
            // real repository/config or credentials can be reached on a regression.
            let git = test.cwd.path().join("git");
            std::fs::write(&git, "#!/bin/sh\nprintf invoked > ran\n").expect("fixture");
            std::fs::set_permissions(git, std::fs::Permissions::from_mode(0o755))
                .expect("fixture mode");
        },
        /*pin_cwd_as_workspace_root*/ true,
    )
    .await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn conditional_assignment_requires_fresh_approval_and_denial_never_executes()
-> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    skip_if_wine_exec!(Ok(()), "local POSIX shell fixture");
    for prefix in [
        "false && r=origin",
        "(r=origin)",
        "r=origin | cat",
        "f() { r=origin; }",
        "false && { r=origin; }",
        "false &&\nr=origin",
        "r=origin && true &",
        "false && # continuation\nr=origin",
        "r=origin && { true; } >log &",
        "if r=origin; then :; fi | cat",
    ] {
        let (test, captured) =
            assignment_turn(prefix, SecurityLevel::Moderate, AskForApproval::OnRequest).await?;
        let approval = next_exec_approval(&test)
            .await
            .expect("fresh post-taint approval");
        assert!(
            approval
                .reason
                .as_deref()
                .unwrap_or_default()
                .contains("after untrusted content")
        );
        assert_eq!(approval.proposed_execpolicy_amendment, None);
        assert!(!test.cwd.path().join("ran").exists());
        test.codex
            .submit(Op::ExecApproval {
                id: approval.effective_approval_id(),
                turn_id: None,
                decision: ReviewDecision::denied("synthetic denial"),
            })
            .await?;
        assert!(next_exec_approval(&test).await.is_none());
        assert!(!test.cwd.path().join("ran").exists());
        let output = output_text(captured.requests().last().expect("result"), "call-fetch");
        assert!(output.contains("synthetic denial"), "{prefix}: {output}");
    }
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn conditional_assignment_executes_only_after_fresh_approval() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    skip_if_wine_exec!(Ok(()), "local POSIX shell fixture");
    let (test, captured) = assignment_turn(
        "false && r=origin",
        SecurityLevel::Moderate,
        AskForApproval::OnRequest,
    )
    .await?;
    let approval = next_exec_approval(&test).await.expect("fresh approval");
    assert!(!test.cwd.path().join("ran").exists());
    test.codex
        .submit(Op::ExecApproval {
            id: approval.effective_approval_id(),
            turn_id: None,
            decision: ReviewDecision::Approved,
        })
        .await?;
    assert!(next_exec_approval(&test).await.is_none());
    let output = output_text(captured.requests().last().expect("result"), "call-fetch");
    assert_eq!(
        std::fs::read_to_string(test.cwd.path().join("ran")).ok(),
        Some("invoked".into()),
        "{output}"
    );
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn conditional_assignment_with_approvals_off_never_executes() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    skip_if_wine_exec!(Ok(()), "local POSIX shell fixture");
    let (test, captured) = assignment_turn(
        "false && r=origin",
        SecurityLevel::Moderate,
        AskForApproval::Never,
    )
    .await?;
    assert!(next_exec_approval(&test).await.is_none());
    assert!(!test.cwd.path().join("ran").exists());
    assert!(
        output_text(captured.requests().last().expect("result"), "call-fetch")
            .contains("approvals are off")
    );
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn unconditional_assignment_and_permissive_controls_execute_fixture() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    skip_if_wine_exec!(Ok(()), "local POSIX shell fixture");
    for (prefix, level) in [
        ("r=origin", SecurityLevel::Moderate),
        ("false && r=origin", SecurityLevel::Permissive),
    ] {
        let (test, captured) = assignment_turn(prefix, level, AskForApproval::Never).await?;
        assert!(next_exec_approval(&test).await.is_none());
        let output = output_text(captured.requests().last().expect("result"), "call-fetch");
        assert_eq!(
            std::fs::read_to_string(test.cwd.path().join("ran")).ok(),
            Some("invoked".into()),
            "{prefix}: {output}"
        );
        assert!(
            !output_text(captured.requests().last().expect("result"), "call-fetch")
                .contains("approvals are off")
        );
    }
    Ok(())
}
