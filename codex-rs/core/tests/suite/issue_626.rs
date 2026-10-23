//! Issue #626 (milestone "Aggressive ships" re-run, G11b / AGG-33): a "don't
//! ask again" approval remembered under a less strict level must not approve
//! a command while Aggressive is active (Travis, 2026-10-13). End to end
//! through a session and the real sandbox (Seatbelt on macOS, bubblewrap on
//! Linux):
//! - under Aggressive the person is asked, the prompt says the remembered
//!   approval was not applied, and the rule is kept;
//! - under Permissive and Moderate the same rule still runs the command
//!   unasked;
//! - a rule the person saves while Aggressive is active applies there;
//! - with zsh-fork on, a prompt for a program a script starts says so too.
#![cfg(any(target_os = "macos", target_os = "linux"))]

use std::sync::Arc;

use codex_core::config::Constrained;
use codex_features::Feature;
use codex_protocol::approvals::ExecPolicyAmendment;
use codex_protocol::models::PermissionProfile;
use codex_protocol::protocol::AskForApproval;
use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::Op;
use codex_protocol::protocol::ReviewDecision;
use codex_protocol::user_input::UserInput;
use codex_security_policy::SecurityLevel;
use core_test_support::responses::ev_assistant_message;
use core_test_support::responses::ev_completed;
use core_test_support::responses::ev_function_call;
use core_test_support::responses::ev_response_created;
use core_test_support::responses::mount_sse_sequence;
use core_test_support::responses::sse;
use core_test_support::responses::start_mock_server;
use core_test_support::skip_if_no_network;
use core_test_support::skip_if_sandbox;
use core_test_support::test_codex::TestCodex;
use core_test_support::test_codex::test_codex;
use core_test_support::wait_for_event;
use core_test_support::zsh_fork::zsh_fork_runtime;
use core_test_support::zsh_fork::zsh_fork_test_builder;
use serde_json::json;
use tempfile::TempDir;

/// The file each command creates in the project.
const FILE: &str = "remembered-626.txt";
/// The rule a person saved under Permissive with "don't ask again".
const PERMISSIVE_RULE: &str =
    r#"prefix_rule(pattern=["touch", "remembered-626.txt"], decision="allow")"#;
const NOTE: &str = "A remembered approval (don't ask again for commands that start with \
                    `touch remembered-626.txt`) was not applied because security level \
                    Aggressive is on";

/// A session at `level` whose Corbanu home holds `rules`, that runs
/// `touch remembered-626.txt` `calls` times, one model turn each.
async fn session(level: SecurityLevel, rules: &str, calls: usize) -> anyhow::Result<TestCodex> {
    let home = Arc::new(TempDir::new()?);
    std::fs::create_dir_all(home.path().join("rules"))?;
    std::fs::write(home.path().join("rules").join("default.rules"), rules)?;
    let server = start_mock_server().await;
    let mut responses = Vec::new();
    for call in 0..calls {
        let id = format!("call-{call}");
        responses.push(sse(vec![
            ev_response_created(&id),
            ev_function_call(
                &id,
                "exec_command",
                &json!({ "cmd": format!("touch {FILE}") }).to_string(),
            ),
            ev_completed(&id),
        ]));
        responses.push(sse(vec![
            ev_response_created(&format!("done-{call}")),
            ev_assistant_message(&format!("msg-{call}"), "done"),
            ev_completed(&format!("done-{call}")),
        ]));
    }
    mount_sse_sequence(&server, responses).await;
    let test = test_codex()
        .with_home(home)
        .with_model("test-gpt-5.1-codex")
        .with_config(move |config| {
            config.security_level = level;
            config.permissions.approval_policy =
                Constrained::allow_any(AskForApproval::UnlessTrusted);
            config
                .permissions
                .set_permission_profile(PermissionProfile::workspace_write())
                .expect("workspace write");
            config
                .features
                .enable(Feature::SourceEnvelopes)
                .expect("enable source envelopes");
            config.use_experimental_unified_exec_tool = true;
            config
                .features
                .enable(Feature::UnifiedExec)
                .expect("enable unified exec");
        })
        // Host-sandbox security test: local executor on purpose, also in the
        // remote lane (#708).
        .build(&server)
        .await?;
    std::mem::forget(server);
    Ok(test)
}

/// Submits one turn; returns its approval request, or `None` when the turn
/// completed without asking.
async fn turn(
    test: &TestCodex,
) -> anyhow::Result<Option<codex_protocol::protocol::ExecApprovalRequestEvent>> {
    test.codex
        .submit(Op::UserInput {
            items: vec![UserInput::Text {
                text: "create the file".into(),
                text_elements: Vec::new(),
            }],
            final_output_json_schema: None,
            responsesapi_client_metadata: None,
            additional_context: Default::default(),
            thread_settings: Default::default(),
        })
        .await?;
    Ok(
        match wait_for_event(&test.codex, |event| {
            matches!(
                event,
                EventMsg::ExecApprovalRequest(_) | EventMsg::TurnComplete(_)
            )
        })
        .await
        {
            EventMsg::ExecApprovalRequest(approval) => Some(approval),
            _ => None,
        },
    )
}

async fn answer(
    test: &TestCodex,
    approval: &codex_protocol::protocol::ExecApprovalRequestEvent,
    decision: ReviewDecision,
) -> anyhow::Result<()> {
    test.codex
        .submit(Op::ExecApproval {
            id: approval.effective_approval_id(),
            turn_id: None,
            decision,
        })
        .await?;
    wait_for_event(&test.codex, |event| {
        matches!(event, EventMsg::TurnComplete(_))
    })
    .await;
    Ok(())
}

fn rules(test: &TestCodex) -> String {
    std::fs::read_to_string(test.codex_home_path().join("rules").join("default.rules"))
        .unwrap_or_default()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_626_rule_remembered_under_permissive_is_suspended_under_aggressive()
-> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    skip_if_sandbox!(Ok(()));
    let test = session(SecurityLevel::Aggressive, PERMISSIVE_RULE, /*calls*/ 1).await?;
    let Some(approval) = turn(&test).await? else {
        anyhow::bail!(
            "ran unasked under Aggressive (created: {})",
            test.workspace_path(FILE).exists()
        );
    };
    let reason = approval.reason.clone().unwrap_or_default();
    assert!(reason.contains(NOTE), "{reason}");
    answer(&test, &approval, ReviewDecision::denied("no")).await?;
    assert!(!test.workspace_path(FILE).exists());
    // Kept, unchanged: it applies again under Permissive.
    assert_eq!(rules(&test), PERMISSIVE_RULE);
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_626_permissive_and_moderate_still_run_a_remembered_command_unasked()
-> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    skip_if_sandbox!(Ok(()));
    for level in [SecurityLevel::Permissive, SecurityLevel::Moderate] {
        let test = session(level, PERMISSIVE_RULE, /*calls*/ 1).await?;
        assert!(turn(&test).await?.is_none(), "{level:?} asked");
        assert!(test.workspace_path(FILE).exists(), "{level:?}");
    }
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_626_rule_remembered_under_aggressive_applies_there() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    skip_if_sandbox!(Ok(()));
    let test = session(SecurityLevel::Aggressive, "", /*calls*/ 2).await?;
    let Some(approval) = turn(&test).await? else {
        anyhow::bail!("Aggressive did not ask the first time");
    };
    assert!(
        !approval
            .reason
            .clone()
            .unwrap_or_default()
            .contains("remembered approval")
    );
    let amendment = approval
        .proposed_execpolicy_amendment
        .clone()
        .unwrap_or_else(|| ExecPolicyAmendment::new(vec!["touch".to_string(), FILE.to_string()]));
    answer(
        &test,
        &approval,
        ReviewDecision::ApprovedExecpolicyAmendment {
            proposed_execpolicy_amendment: amendment,
        },
    )
    .await?;
    assert!(test.workspace_path(FILE).exists());
    assert!(
        rules(&test).contains("Remembered while security level Aggressive was active."),
        "{}",
        rules(&test)
    );
    std::fs::remove_file(test.workspace_path(FILE))?;
    assert!(turn(&test).await?.is_none(), "asked again");
    assert!(test.workspace_path(FILE).exists());
    Ok(())
}

/// #626 follow-up (Travis, 2026-10-13): with zsh-fork on, a program a
/// command's script starts is checked when it starts. When a remembered
/// approval suspended under Aggressive would have run it unasked, its prompt
/// carries the same note as a shell command's prompt. Here the script names
/// the file through a variable, so only the started `touch` matches the rule.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_626_zsh_fork_prompt_names_the_suspended_approval() -> anyhow::Result<()> {
    zsh_fork_prompt_names_the_suspended_approval(/*unified_exec*/ false).await
}

/// The same through unified exec's zsh-fork backend.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_626_unified_exec_zsh_fork_prompt_names_the_suspended_approval() -> anyhow::Result<()>
{
    zsh_fork_prompt_names_the_suspended_approval(/*unified_exec*/ true).await
}

async fn zsh_fork_prompt_names_the_suspended_approval(unified_exec: bool) -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    skip_if_sandbox!(Ok(()));
    let Some(runtime) = zsh_fork_runtime("issue_626 zsh-fork note")? else {
        return Ok(());
    };
    let script = format!("f={FILE}; touch \"$f\"");
    let (tool, arguments) = if unified_exec {
        ("exec_command", json!({ "cmd": script }))
    } else {
        (
            "shell_command",
            json!({ "command": script, "timeout_ms": 30_000 }),
        )
    };
    let home = Arc::new(TempDir::new()?);
    std::fs::create_dir_all(home.path().join("rules"))?;
    std::fs::write(
        home.path().join("rules").join("default.rules"),
        PERMISSIVE_RULE,
    )?;
    let server = start_mock_server().await;
    mount_sse_sequence(
        &server,
        vec![
            sse(vec![
                ev_response_created("call-zsh"),
                ev_function_call("call-zsh", tool, &arguments.to_string()),
                ev_completed("call-zsh"),
            ]),
            sse(vec![
                ev_response_created("done-zsh"),
                ev_assistant_message("msg-zsh", "done"),
                ev_completed("done-zsh"),
            ]),
        ],
    )
    .await;
    let test = zsh_fork_test_builder(runtime, AskForApproval::UnlessTrusted)
        .with_home(home)
        .with_config(move |config| {
            config.security_level = SecurityLevel::Aggressive;
            if unified_exec {
                config.use_experimental_unified_exec_tool = true;
                for feature in [Feature::UnifiedExec, Feature::UnifiedExecZshFork] {
                    config
                        .features
                        .enable(feature)
                        .expect("enable unified exec zsh-fork");
                }
            }
            config
                .permissions
                .set_permission_profile(PermissionProfile::workspace_write())
                .expect("workspace write");
            config
                .features
                .enable(Feature::SourceEnvelopes)
                .expect("enable source envelopes");
        })
        .build(&server)
        .await?;
    std::mem::forget(server);

    let mut approval = turn(&test).await?;
    let mut prompts = Vec::new();
    while let Some(request) = approval {
        let started_touch = request.command.iter().any(|arg| arg.ends_with("/touch"));
        prompts.push((request.command.clone(), request.reason.clone()));
        if started_touch {
            let reason = request.reason.clone().unwrap_or_default();
            assert!(reason.contains(NOTE), "{reason}");
            answer(&test, &request, ReviewDecision::denied("no")).await?;
            assert!(!test.workspace_path(FILE).exists());
            // Kept, unchanged: it applies again under Permissive.
            assert_eq!(rules(&test), PERMISSIVE_RULE);
            return Ok(());
        }
        test.codex
            .submit(Op::ExecApproval {
                id: request.effective_approval_id(),
                turn_id: None,
                decision: ReviewDecision::Approved,
            })
            .await?;
        approval = match wait_for_event(&test.codex, |event| {
            matches!(
                event,
                EventMsg::ExecApprovalRequest(_) | EventMsg::TurnComplete(_)
            )
        })
        .await
        {
            EventMsg::ExecApprovalRequest(next) => Some(next),
            _ => None,
        };
    }
    anyhow::bail!(
        "no prompt for the started touch (created: {}); prompts: {prompts:?}",
        test.workspace_path(FILE).exists()
    )
}
