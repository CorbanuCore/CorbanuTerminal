//! Issue #623 (milestone "Aggressive ships" re-run, G27 / AGG-R13): under
//! Aggressive an agent command that creates a link to the project's
//! `.agents` or `.codex` is refused before it runs, so nothing is created
//! (Travis, 2026-10-13; decision #520). End to end through a session and the
//! real sandbox (Seatbelt on macOS, bubblewrap on Linux), for relative and
//! absolute targets and a link made by an interpreter.
#![cfg(any(target_os = "macos", target_os = "linux"))]

use std::path::Path;
use std::sync::Arc;

use codex_core::config::Constrained;
use codex_core::security_inspection;
use codex_features::Feature;
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
use core_test_support::test_codex::test_codex;
use core_test_support::wait_for_event;
use serde_json::json;
use tempfile::TempDir;

const REFUSED: &str =
    "Security level Aggressive refused this command before it ran: it creates a link";

/// What one command did.
struct Run {
    /// Whether the person was asked (approved when asked).
    asked: bool,
    output: String,
    warnings: Vec<String>,
    denials: Vec<(&'static str, &'static str)>,
    /// Whether `ag` exists in the project afterwards (as a link or anything).
    created: bool,
    /// `.agents/notes.md` afterwards.
    notes: String,
}

/// Runs `command` (built from the project's path) at `level` in a project
/// holding `.agents/notes.md`, approving it if asked.
async fn run(level: SecurityLevel, command: impl FnOnce(&Path) -> String) -> anyhow::Result<Run> {
    run_calls(level, |root| {
        vec![("call-0", "exec_command", json!({ "cmd": command(root) }))]
    })
    .await
}

/// [`run`] for several tool calls, one model step each; the last call's
/// output is reported.
async fn run_calls(
    level: SecurityLevel,
    calls: impl FnOnce(&Path) -> Vec<(&'static str, &'static str, serde_json::Value)>,
) -> anyhow::Result<Run> {
    let server = start_mock_server().await;
    let test = test_codex()
        .with_home(Arc::new(TempDir::new()?))
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
    let root = test.cwd_path().canonicalize()?;
    std::fs::create_dir_all(root.join(".agents"))?;
    std::fs::write(root.join(".agents").join("notes.md"), "orig\n")?;
    let calls = calls(&root);
    let last = calls.last().map(|(id, _, _)| *id).unwrap_or_default();
    let mut steps: Vec<_> = calls
        .iter()
        .map(|(id, tool, arguments)| {
            sse(vec![
                ev_response_created(id),
                ev_function_call(id, tool, &arguments.to_string()),
                ev_completed(id),
            ])
        })
        .collect();
    steps.push(sse(vec![
        ev_response_created("done"),
        ev_assistant_message("msg-done", "done"),
        ev_completed("done"),
    ]));
    let captured = mount_sse_sequence(&server, steps).await;
    std::mem::forget(server);
    let thread = test.session_configured.thread_id;
    test.codex
        .submit(Op::UserInput {
            items: vec![UserInput::Text {
                text: "run the command".into(),
                text_elements: Vec::new(),
            }],
            final_output_json_schema: None,
            responsesapi_client_metadata: None,
            additional_context: Default::default(),
            thread_settings: Default::default(),
        })
        .await?;
    let mut asked = false;
    let mut warnings = Vec::new();
    loop {
        match wait_for_event(&test.codex, |event| {
            matches!(
                event,
                EventMsg::ExecApprovalRequest(_) | EventMsg::Warning(_) | EventMsg::TurnComplete(_)
            )
        })
        .await
        {
            EventMsg::ExecApprovalRequest(approval) => {
                asked = true;
                test.codex
                    .submit(Op::ExecApproval {
                        id: approval.effective_approval_id(),
                        turn_id: None,
                        decision: ReviewDecision::Approved,
                    })
                    .await?;
            }
            EventMsg::Warning(warning) => warnings.push(warning.message),
            _ => break,
        }
    }
    let output = captured
        .requests()
        .last()
        .and_then(|request| request.function_call_output_text(last))
        .unwrap_or_default();
    let denials =
        security_inspection::observe(test.codex_home_path(), level, Some(thread), i64::MAX / 2)
            .denials
            .into_iter()
            .map(|denial| (denial.reason, denial.outcome))
            .collect();
    Ok(Run {
        asked,
        output,
        warnings,
        denials,
        created: std::fs::symlink_metadata(root.join("ag")).is_ok(),
        notes: std::fs::read_to_string(root.join(".agents").join("notes.md"))?,
    })
}

fn assert_refused(name: &str, run: &Run) {
    assert!(!run.created, "{name}: the link was created");
    assert!(!run.asked, "{name}: asked about a link that cannot be made");
    assert_eq!(run.notes, "orig\n", "{name}");
    assert!(run.output.contains(REFUSED), "{name}: {}", run.output);
    assert!(
        run.warnings
            .iter()
            .any(|warning| warning.starts_with(REFUSED)),
        "{name}: {:?}",
        run.warnings
    );
    assert!(
        run.warnings
            .iter()
            .any(|warning| warning.contains("cannot be granted, not even once")),
        "{name}: {:?}",
        run.warnings
    );
    assert!(
        run.denials.contains(&(
            "creating a link to the project's .codex or .agents, or Corbanu's files",
            "refused before it ran; never granted under Aggressive"
        )),
        "{name}: {:?}",
        run.denials
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_623_link_into_agent_settings_is_not_created_under_aggressive() -> anyhow::Result<()>
{
    skip_if_no_network!(Ok(()));
    skip_if_sandbox!(Ok(()));
    let relative = run(SecurityLevel::Aggressive, |_| {
        "ln -s .agents ag && echo x >> ag/notes.md".to_string()
    })
    .await?;
    assert_refused("relative", &relative);
    let absolute = run(SecurityLevel::Aggressive, |root| {
        format!("ln -s '{}/.agents' ag", root.display())
    })
    .await?;
    assert_refused("absolute", &absolute);
    if std::process::Command::new("python3")
        .args(["-c", "0"])
        .output()
        .is_ok_and(|output| output.status.success())
    {
        let python = run(SecurityLevel::Aggressive, |_| {
            "python3 -c \"import os; os.symlink('.agents', 'ag')\"".to_string()
        })
        .await?;
        assert_refused("python3", &python);
    }
    Ok(())
}

/// Review r2: typing the link command into a running shell is refused too.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_623_link_typed_into_a_running_shell_is_not_created() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    skip_if_sandbox!(Ok(()));
    let typed = run_calls(SecurityLevel::Aggressive, |_| {
        vec![
            (
                "call-open",
                "exec_command",
                json!({ "cmd": "/bin/sh -i", "yield_time_ms": 300, "tty": true }),
            ),
            (
                "call-stdin",
                "write_stdin",
                json!({ "session_id": 1000, "chars": "ln -s .agents ag\n", "yield_time_ms": 500 }),
            ),
        ]
    })
    .await?;
    assert!(
        !typed.created,
        "the typed link was created: {}",
        typed.output
    );
    assert!(typed.output.contains(REFUSED), "{}", typed.output);
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_623_other_links_and_permissive_are_unchanged() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    skip_if_sandbox!(Ok(()));
    // Under Aggressive a link elsewhere in the project is made as before.
    let other = run(SecurityLevel::Aggressive, |_| "ln -s notes ag".to_string()).await?;
    assert!(other.created, "{}", other.output);
    assert!(!other.output.contains(REFUSED));
    // Permissive and Moderate are unchanged: the link is made.
    for level in [SecurityLevel::Permissive, SecurityLevel::Moderate] {
        let run = run(level, |_| "ln -s .agents ag".to_string()).await?;
        assert!(run.created, "{level:?}: {}", run.output);
        assert!(!run.output.contains(REFUSED), "{level:?}");
    }
    Ok(())
}
