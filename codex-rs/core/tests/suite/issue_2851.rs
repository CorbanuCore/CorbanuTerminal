//! D750 (#2851, Travis 2026-11-23, option (b)): a Linux (bubblewrap) sandbox
//! that fails to start after unified exec's 150 ms early-exit window is
//! decided as an early failure: the launcher probe decides (its `bwrap: `
//! lines only), and a retry outside the sandbox needs a person's fresh,
//! once-only answer. An ordinary late denial, and a command that prints a
//! startup failure's text late, stay plain denials. The sandbox launcher
//! here is a stand-in that waits past the window first.
#![cfg(target_os = "linux")]

use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use codex_core::config::Constrained;
use codex_features::Feature;
use codex_protocol::models::PermissionProfile;
use codex_protocol::permissions::NetworkSandboxPolicy;
use codex_protocol::protocol::AskForApproval;
use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::Op;
use codex_protocol::protocol::ReviewDecision;
use codex_protocol::user_input::UserInput;
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
use pretty_assertions::assert_eq;
use serde_json::json;
use tempfile::TempDir;

/// What bubblewrap prints where it cannot set up a user namespace.
const BWRAP_FAILURE: &str = "bwrap: setting up uid map: Permission denied";
/// The reason the retry prompt shows.
const STARTUP_RETRY: &str = "sandbox failed to start; retry without sandbox?";

/// The sandbox launcher stand-in: it waits past the early-exit window, then
/// either fails as bubblewrap does without user namespaces (whatever the
/// command, so the probe fails too) or starts the command after `--`.
#[derive(Clone, Copy, Debug)]
enum Launcher {
    FailsLate,
    StartsLate,
}

fn launcher(dir: &Path, launcher: Launcher) -> anyhow::Result<PathBuf> {
    let body = match launcher {
        Launcher::FailsLate => format!("#!/bin/sh\nsleep 0.5\nprintf '{BWRAP_FAILURE}\\n' >&2\nexit 1\n"),
        Launcher::StartsLate => "#!/bin/sh\nsleep 0.5\nwhile [ \"$#\" -gt 0 ] && [ \"$1\" != \"--\" ]; do shift; done\nshift\nexec \"$@\"\n".to_string(),
    };
    let path = dir.join("launcher");
    std::fs::write(&path, body)?;
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755))?;
    Ok(path)
}

struct Run {
    /// The reasons of the approval prompts shown.
    reasons: Vec<String>,
    /// The answers the last prompt offered.
    offered: Vec<ReviewDecision>,
    /// How often the command itself ran.
    runs: usize,
    /// What the model got back.
    output: String,
}

/// Runs `command` (which first notes its run) through `launcher`; the
/// prompts are answered `decisions` in order (the last one repeated).
async fn run(
    launcher_kind: Launcher,
    command: &str,
    approval: AskForApproval,
    decisions: &[ReviewDecision],
) -> anyhow::Result<Run> {
    let scratch = TempDir::new()?;
    let launcher = launcher(scratch.path(), launcher_kind)?;
    let ran = scratch.path().join("ran");
    let cmd = format!("echo x >> '{}'; {command}", ran.display());
    let server = start_mock_server().await;
    let captured = mount_sse_sequence(
        &server,
        vec![
            sse(vec![
                ev_response_created("call-0"),
                ev_function_call("call-0", "exec_command", &json!({ "cmd": cmd }).to_string()),
                ev_completed("call-0"),
            ]),
            sse(vec![
                ev_response_created("done"),
                ev_assistant_message("msg-done", "done"),
                ev_completed("done"),
            ]),
        ],
    )
    .await;
    let test = test_codex()
        .with_home(Arc::new(TempDir::new()?))
        .with_model("test-gpt-5.1-codex")
        .with_config(move |config| {
            config.permissions.approval_policy = Constrained::allow_any(approval);
            let folder = config.cwd.clone();
            config
                .permissions
                .set_permission_profile(PermissionProfile::workspace_write_with(
                    &[folder],
                    NetworkSandboxPolicy::Restricted,
                    /*exclude_tmpdir_env_var*/ true,
                    /*exclude_slash_tmp*/ true,
                ))
                .expect("workspace write");
            config.codex_linux_sandbox_exe = Some(launcher);
            config.use_experimental_unified_exec_tool = true;
            config
                .features
                .enable(Feature::UnifiedExec)
                .expect("enable unified exec");
        })
        // Host-sandbox test: local executor on purpose.
        .build(&server)
        .await?;
    std::mem::forget(server);
    test.codex
        .submit(Op::UserInput {
            items: vec![UserInput::Text {
                text: "run it".into(),
                text_elements: Vec::new(),
            }],
            final_output_json_schema: None,
            responsesapi_client_metadata: None,
            additional_context: Default::default(),
            thread_settings: Default::default(),
        })
        .await?;
    let mut reasons = Vec::new();
    let mut offered = Vec::new();
    loop {
        let EventMsg::ExecApprovalRequest(request) = wait_for_event(&test.codex, |event| {
            matches!(
                event,
                EventMsg::ExecApprovalRequest(_) | EventMsg::TurnComplete(_)
            )
        })
        .await
        else {
            break;
        };
        reasons.push(request.reason.clone().unwrap_or_default());
        offered = request.effective_available_decisions();
        test.codex
            .submit(Op::ExecApproval {
                id: request.effective_approval_id(),
                turn_id: None,
                decision: decisions
                    .get(reasons.len() - 1)
                    .or(decisions.last())
                    .cloned()
                    .unwrap_or(ReviewDecision::Approved),
            })
            .await?;
    }
    let output = captured
        .requests()
        .last()
        .and_then(|request| request.function_call_output_text("call-0"))
        .unwrap_or_default();
    let runs = std::fs::read_to_string(&ran)
        .map(|text| text.lines().count())
        .unwrap_or(0);
    Ok(Run {
        reasons,
        offered,
        runs,
        output,
    })
}

/// A sandbox that fails to start after the window is asked about again,
/// once only, and runs outside it only on the person's answer.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_2851_a_late_sandbox_startup_failure_is_asked_about_again() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    skip_if_sandbox!(Ok(()));
    let run = run(
        Launcher::FailsLate,
        "printf 'ran-2851\\n'",
        AskForApproval::UnlessTrusted,
        &[ReviewDecision::Approved],
    )
    .await?;
    assert_eq!(run.reasons.len(), 2, "{:?} {:?}", run.reasons, run.output);
    assert_eq!(run.reasons[1], STARTUP_RETRY);
    assert_eq!(
        run.offered,
        vec![
            ReviewDecision::Approved,
            ReviewDecision::denied("rejected by user"),
            ReviewDecision::Abort,
        ],
        "once only"
    );
    assert_eq!(run.runs, 1, "ran once, outside the sandbox");
    assert!(run.output.contains("ran-2851"), "{}", run.output);
    Ok(())
}

/// Declined, the retry does not run and the denial stands.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_2851_a_declined_late_startup_retry_does_not_run() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    skip_if_sandbox!(Ok(()));
    let run = run(
        Launcher::FailsLate,
        "printf 'ran-2851\\n'",
        AskForApproval::UnlessTrusted,
        &[
            ReviewDecision::Approved,
            ReviewDecision::denied("rejected by user"),
        ],
    )
    .await?;
    assert_eq!(run.reasons.len(), 2, "{:?}", run.reasons);
    assert_eq!(run.reasons[1], STARTUP_RETRY);
    assert_eq!(run.runs, 0, "never ran");
    assert!(!run.output.contains("ran-2851"), "{}", run.output);
    Ok(())
}

/// On request, a command the person did not approve is not asked about
/// again after its sandbox failed to start (as for an early failure).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_2851_an_unapproved_late_startup_failure_stays_denied() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    skip_if_sandbox!(Ok(()));
    let run = run(
        Launcher::FailsLate,
        "printf 'ran-2851\\n'",
        AskForApproval::OnRequest,
        &[ReviewDecision::Approved],
    )
    .await?;
    assert_eq!(run.reasons, Vec::<String>::new());
    assert_eq!(run.runs, 0, "never ran");
    Ok(())
}

/// With approvals off nobody is asked and nothing runs outside the sandbox.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_2851_a_late_startup_failure_is_refused_with_approvals_off() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    skip_if_sandbox!(Ok(()));
    let run = run(
        Launcher::FailsLate,
        "printf 'ran-2851\\n'",
        AskForApproval::Never,
        &[ReviewDecision::Approved],
    )
    .await?;
    assert_eq!(run.reasons, Vec::<String>::new());
    assert_eq!(run.runs, 0, "never ran");
    assert!(run.output.contains("bwrap: "), "{}", run.output);
    Ok(())
}

/// An ordinary late denial, and a command that prints bubblewrap's startup
/// failure itself late (in a sandbox that started), stay plain denials:
/// not asked about again, not run again.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_2851_late_denials_of_a_started_sandbox_stay_final() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    skip_if_sandbox!(Ok(()));
    for command in [
        "printf 'touch: cannot touch x: Permission denied\\n' >&2; exit 1".to_string(),
        format!("printf '{BWRAP_FAILURE}\\n' >&2; exit 1"),
    ] {
        let run = run(
            Launcher::StartsLate,
            &command,
            AskForApproval::UnlessTrusted,
            &[ReviewDecision::Approved],
        )
        .await?;
        assert_eq!(run.reasons.len(), 1, "{command}: {:?}", run.reasons);
        assert!(
            !run.reasons.iter().any(|reason| reason == STARTUP_RETRY),
            "{command}"
        );
        assert_eq!(run.runs, 1, "{command}: not run again");
    }
    Ok(())
}
