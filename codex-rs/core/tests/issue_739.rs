//! #739 (Travis, 2026-10-14, option B, security sign-off): under security
//! level Permissive with protected launch armed (`secretless_agent_launch`),
//! whose launch contract refuses every command outside the OS sandbox, no
//! approval to run outside it is offered. Through the orchestrator and the
//! real sandbox (Seatbelt on macOS, bubblewrap on Linux):
//! - a request to run outside the sandbox is refused before anyone is asked,
//!   and the refusal says plainly that it isn't possible here;
//! - a remembered exec-policy rule that would skip the sandbox is not
//!   applied silently: the command runs inside it and the person is told;
//! - "retry without sandbox?" is not offered after a sandbox denial, and the
//!   person is told why;
//! - the model's instructions say so.
//!
//! Nothing ends in "Protected launch refused". Moderate keeps its own
//! handling (#747). Permissive without protected launch is unchanged
//! (`suite::issue_747`).
//!
//! Its own test binary, because arming protected launch is process-wide.
#![cfg(any(target_os = "macos", target_os = "linux"))]
#![allow(clippy::expect_used)]

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
use pretty_assertions::assert_eq;
use serde_json::json;
use tempfile::TempDir;

/// The refusal of a request to run outside the sandbox.
const NOT_POSSIBLE: &str = "asked to run outside the OS sandbox, and that isn't possible here";
/// The note when a remembered rule would have skipped the sandbox.
const RULE_NOTE: &str =
    "A remembered approval or allow rule would run this command outside the OS sandbox";
/// The note when no retry without the sandbox is offered.
const RETRY_NOTE: &str = "\"retry without sandbox?\" is not offered";
/// In the model's permissions instructions.
const INSTRUCTED: &str = "Corbanu Terminal's protected launch (secretless_agent_launch) is on";
/// What the launch contract says when it refuses a run (the old behaviour).
const CONTRACT_REFUSED: &str = "Protected launch refused";

/// Protected launch keeps the first home it loads for the whole process:
/// that folder is kept until the process ends.
static FIRST_HOME: std::sync::OnceLock<Arc<TempDir>> = std::sync::OnceLock::new();

#[derive(Clone, Copy, Debug)]
enum Route {
    /// `shell_command` with `require_escalated`.
    Escalated,
    /// `exec_command` (unified exec) with `require_escalated`.
    EscalatedExec,
    /// An exec-policy allow rule saved earlier ("don't ask again").
    Rule,
    /// An ordinary command under `untrusted`, approved; the sandbox denies it.
    Retry,
    /// An ordinary command writing inside the current folder (control: the
    /// sandboxed run works under protected launch).
    Inside,
}

#[derive(Debug)]
struct Run {
    asked: usize,
    wrote: bool,
    output: String,
    warnings: Vec<String>,
    instructed: bool,
}

async fn run(level: SecurityLevel, route: Route) -> anyhow::Result<Run> {
    let outside = TempDir::new()?;
    let home = Arc::new(TempDir::new()?);
    FIRST_HOME.get_or_init(|| Arc::clone(&home));
    // Arms protected launch for this process when the config loads.
    std::fs::write(
        home.path().join("config.toml"),
        "[features]\nsecretless_agent_launch = true\n",
    )?;
    if matches!(route, Route::Rule) {
        std::fs::create_dir_all(home.path().join("rules"))?;
        std::fs::write(
            home.path().join("rules/policy.rules"),
            "prefix_rule(pattern=[\"touch\"], decision=\"allow\")\n",
        )?;
    }
    let approval = match route {
        Route::Retry => AskForApproval::UnlessTrusted,
        _ => AskForApproval::OnRequest,
    };
    let unified_exec = matches!(route, Route::EscalatedExec);
    let server = start_mock_server().await;
    let test = test_codex()
        .with_home(home)
        .with_model("test-gpt-5.1-codex")
        .with_config(move |config| {
            config.security_level = level;
            config.permissions.approval_policy = Constrained::allow_any(approval);
            // Writes only in the current folder (not /tmp or $TMPDIR, where
            // `outside` is); network off.
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
            // Moderate needs the untrusted-content labels.
            config
                .features
                .enable(Feature::SourceEnvelopes)
                .expect("enable source envelopes");
            if unified_exec {
                config.use_experimental_unified_exec_tool = true;
                config
                    .features
                    .enable(Feature::UnifiedExec)
                    .expect("enable unified exec");
            }
        })
        // Host-sandbox security test: local executor on purpose.
        .build(&server)
        .await?;
    let target = match route {
        Route::Inside => test.cwd_path().join("notes.txt"),
        _ => outside.path().canonicalize()?.join("notes.txt"),
    };
    // No redirection: an exec-policy rule applies only to commands it can
    // parse.
    let command = format!("touch '{}'", target.display());
    let call = match route {
        Route::Escalated => ev_function_call(
            "call-0",
            "shell_command",
            &json!({
                "command": command,
                "sandbox_permissions": "require_escalated",
                "justification": "This command writes outside the workspace. Do you want to allow it?",
            })
            .to_string(),
        ),
        Route::EscalatedExec => ev_function_call(
            "call-0",
            "exec_command",
            &json!({
                "cmd": command,
                "sandbox_permissions": "require_escalated",
                "justification": "This command writes outside the workspace. Do you want to allow it?",
            })
            .to_string(),
        ),
        Route::Rule | Route::Retry | Route::Inside => ev_function_call(
            "call-0",
            "shell_command",
            &json!({ "command": command }).to_string(),
        ),
    };
    let mut responses = vec![sse(vec![
        ev_response_created("call-0"),
        call,
        ev_completed("call-0"),
    ])];
    // The rule's note is shown once per session: a second matching command.
    if matches!(route, Route::Rule) {
        responses.push(sse(vec![
            ev_response_created("call-1"),
            ev_function_call(
                "call-1",
                "shell_command",
                &json!({ "command": command }).to_string(),
            ),
            ev_completed("call-1"),
        ]));
    }
    responses.push(sse(vec![
        ev_response_created("done"),
        ev_assistant_message("msg-done", "done"),
        ev_completed("done"),
    ]));
    let captured = mount_sse_sequence(&server, responses).await;
    std::mem::forget(server);
    test.codex
        .submit(Op::UserInput {
            items: vec![UserInput::Text {
                text: "write the notes".into(),
                text_elements: Vec::new(),
            }],
            final_output_json_schema: None,
            responsesapi_client_metadata: None,
            additional_context: Default::default(),
            thread_settings: Default::default(),
        })
        .await?;
    let mut asked = 0;
    let mut warnings = Vec::new();
    let mut errors = Vec::new();
    loop {
        match wait_for_event(&test.codex, |event| {
            matches!(
                event,
                EventMsg::ExecApprovalRequest(_)
                    | EventMsg::Warning(_)
                    | EventMsg::TurnComplete(_)
                    | EventMsg::Error(_)
            )
        })
        .await
        {
            EventMsg::ExecApprovalRequest(approval) => {
                asked += 1;
                test.codex
                    .submit(Op::ExecApproval {
                        id: approval.effective_approval_id(),
                        turn_id: None,
                        decision: ReviewDecision::Approved,
                    })
                    .await?;
            }
            EventMsg::Warning(warning) => warnings.push(warning.message),
            EventMsg::Error(error) => errors.push(error.message),
            _ => break,
        }
    }
    let requests = captured.requests();
    anyhow::ensure!(
        !requests.is_empty(),
        "{level:?} {route:?}: the model was never asked: {errors:?}"
    );
    let output = requests
        .last()
        .and_then(|request| request.function_call_output_text("call-0"))
        .unwrap_or_default();
    Ok(Run {
        asked,
        wrote: target.exists(),
        output,
        warnings,
        instructed: requests[0].body_contains_text(INSTRUCTED),
    })
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_739_permissive_under_protected_launch_offers_no_run_outside_the_sandbox()
-> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    skip_if_sandbox!(Ok(()));
    use SecurityLevel::Moderate;
    use SecurityLevel::Permissive;
    let mut wrong = Vec::new();
    // (level, route, prompts expected, writes, text in the output, warning)
    // The refusal is also a warning, so the person sees it, not only the model.
    for (level, route, asks, writes, said, warned) in [
        (
            Permissive,
            Route::Escalated,
            0,
            false,
            Some(NOT_POSSIBLE),
            Some(NOT_POSSIBLE),
        ),
        (
            Permissive,
            Route::EscalatedExec,
            0,
            false,
            Some(NOT_POSSIBLE),
            Some(NOT_POSSIBLE),
        ),
        // Still no prompt, but inside the sandbox, and the person is told.
        (Permissive, Route::Rule, 0, false, None, Some(RULE_NOTE)),
        (Permissive, Route::Retry, 1, false, None, Some(RETRY_NOTE)),
        (Permissive, Route::Inside, 0, true, None, None),
        // #747: Moderate still asks (an approval lifts its protected-file
        // rules for that one command) and keeps the run in the sandbox.
        (Moderate, Route::Escalated, 1, false, None, None),
    ] {
        // Boxed: the turn's future is too large for the test thread's stack.
        let run = Box::pin(run(level, route)).await?;
        let instructed = level == Permissive;
        let warned_ok = match warned {
            // Once, also where a second command matches the rule.
            Some(note) => {
                run.warnings
                    .iter()
                    .filter(|warning| warning.contains(note))
                    .count()
                    == 1
            }
            None => !run.warnings.iter().any(|warning| {
                [RULE_NOTE, RETRY_NOTE, NOT_POSSIBLE]
                    .iter()
                    .any(|note| warning.contains(note))
            }),
        };
        if run.asked != asks
            || run.wrote != writes
            || said.is_some_and(|text| !run.output.contains(text))
            || run.output.contains(CONTRACT_REFUSED)
            || !warned_ok
            || run.instructed != instructed
        {
            wrong.push(format!("{level:?} {route:?}: {run:?}"));
        }
    }
    assert_eq!(wrong, Vec::<String>::new());
    Ok(())
}
