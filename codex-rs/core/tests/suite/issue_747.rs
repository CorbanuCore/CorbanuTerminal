//! #747 (Travis, 2026-10-14, decision D3): under security level Moderate no
//! command runs outside the OS sandbox, through the orchestrator and the real
//! sandbox (Seatbelt on macOS, bubblewrap on Linux): not a request to run
//! outside it that a person approves, not the "retry without sandbox?" after
//! a sandbox denial, and not a remembered exec-policy rule (D6: it still
//! skips the prompt). The prompt and the model's instructions say so.
//! Permissive is unchanged.
#![cfg(any(target_os = "macos", target_os = "linux"))]

use std::path::Path;
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
use core_test_support::responses::ev_apply_patch_custom_tool_call;
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

/// Shown on the prompt of a request to run outside the sandbox under
/// Moderate.
const STAYS: &str = "still runs inside the sandbox";
/// #840: on that prompt, what approving lifts (said in terms of the files,
/// not as a credential request).
const LIFTS: &str = "Approving it lifts Moderate's protected-file rules for this one run";
/// In the model's permissions instructions under Moderate.
const INSTRUCTED: &str = "a command never leaves the sandbox it runs in";

/// How the command asks to leave the sandbox, if it does.
#[derive(Clone, Copy, Debug)]
enum Route {
    /// `sandbox_permissions: require_escalated`, approved by the person.
    Escalated,
    /// An ordinary command under `untrusted`, approved by the person; after
    /// the sandbox denies it, the retry without the sandbox needs no new
    /// answer.
    Retry,
    /// An exec-policy allow rule saved earlier ("don't ask again").
    Rule,
    /// #840 (D6): such a rule, for a command that asks to run outside the
    /// sandbox (as the rule was saved under Permissive).
    RuleEscalated,
    /// #840: the same through `exec_command` (unified exec).
    RuleEscalatedExec,
    /// `exec_command` (unified exec) with `require_escalated`, approved.
    EscalatedExec,
    /// `apply_patch` adding a file outside the writable folder, approved: it
    /// is applied inside the sandbox with that one extra write, under every
    /// level (not a run outside the sandbox).
    Patch,
}

struct Run {
    /// Approval prompts shown.
    asked: usize,
    /// The prompts' reasons.
    reasons: Vec<String>,
    /// Whether the command wrote outside the writable folder.
    wrote: bool,
    /// What the model got back.
    output: String,
    /// Whether the model was told that Moderate keeps commands in the sandbox.
    instructed: bool,
}

async fn run(level: SecurityLevel, route: Route) -> anyhow::Result<Run> {
    let outside = TempDir::new()?;
    let target = outside.path().canonicalize()?.join("notes.txt");
    // No redirection: an exec-policy rule applies only to commands it can
    // parse.
    let command = format!("touch '{}'", target.display());
    let call = match route {
        Route::Escalated | Route::RuleEscalated => ev_function_call(
            "call-0",
            "shell_command",
            &json!({
                "command": command,
                "sandbox_permissions": "require_escalated",
                "justification": "write the notes",
            })
            .to_string(),
        ),
        Route::EscalatedExec | Route::RuleEscalatedExec => ev_function_call(
            "call-0",
            "exec_command",
            &json!({
                "cmd": command,
                "sandbox_permissions": "require_escalated",
                "justification": "write the notes",
            })
            .to_string(),
        ),
        Route::Patch => ev_apply_patch_custom_tool_call(
            "call-0",
            &format!(
                "*** Begin Patch\n*** Add File: {}\n+moderate-747\n*** End Patch",
                target.display()
            ),
        ),
        Route::Retry | Route::Rule => ev_function_call(
            "call-0",
            "shell_command",
            &json!({ "command": command }).to_string(),
        ),
    };
    let home = Arc::new(TempDir::new()?);
    if matches!(
        route,
        Route::Rule | Route::RuleEscalated | Route::RuleEscalatedExec
    ) {
        std::fs::create_dir_all(home.path().join("rules"))?;
        std::fs::write(
            home.path().join("rules/policy.rules"),
            "prefix_rule(pattern=[\"touch\"], decision=\"allow\")\n",
        )?;
    }
    let server = start_mock_server().await;
    let captured = mount_sse_sequence(
        &server,
        vec![
            sse(vec![
                ev_response_created("call-0"),
                call,
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
    let approval = match route {
        Route::Retry => AskForApproval::UnlessTrusted,
        Route::Escalated
        | Route::Rule
        | Route::RuleEscalated
        | Route::RuleEscalatedExec
        | Route::EscalatedExec
        | Route::Patch => AskForApproval::OnRequest,
    };
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
            if matches!(route, Route::EscalatedExec | Route::RuleEscalatedExec) {
                config.use_experimental_unified_exec_tool = true;
                config
                    .features
                    .enable(Feature::UnifiedExec)
                    .expect("enable unified exec");
            }
        })
        // Host-sandbox security test: local executor on purpose, also in the
        // remote lane (#708).
        .build(&server)
        .await?;
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
    let mut reasons = Vec::new();
    let mut errors = Vec::new();
    loop {
        let (reason, op) = match wait_for_event(&test.codex, |event| {
            matches!(
                event,
                EventMsg::ExecApprovalRequest(_)
                    | EventMsg::ApplyPatchApprovalRequest(_)
                    | EventMsg::TurnComplete(_)
                    | EventMsg::Error(_)
            )
        })
        .await
        {
            EventMsg::ExecApprovalRequest(approval) => (
                approval.reason.clone(),
                Op::ExecApproval {
                    id: approval.effective_approval_id(),
                    turn_id: None,
                    decision: ReviewDecision::Approved,
                },
            ),
            EventMsg::ApplyPatchApprovalRequest(approval) => (
                approval.reason,
                Op::PatchApproval {
                    id: approval.call_id,
                    decision: ReviewDecision::Approved,
                },
            ),
            EventMsg::Error(error) => {
                errors.push(error.message);
                continue;
            }
            _ => break,
        };
        reasons.push(reason.unwrap_or_default());
        asked += 1;
        test.codex.submit(op).await?;
    }
    let requests = captured.requests();
    anyhow::ensure!(
        !requests.is_empty(),
        "{level:?} {route:?}: the model was never asked: {errors:?}"
    );
    let output = requests
        .last()
        .map(|request| {
            request
                .function_call_output_text("call-0")
                .unwrap_or_else(|| {
                    request
                        .custom_tool_call_output("call-0")
                        .get("output")
                        .map(ToString::to_string)
                        .unwrap_or_default()
                })
        })
        .unwrap_or_default();
    Ok(Run {
        asked,
        reasons,
        wrote: written(&target),
        output,
        instructed: requests[0].body_contains_text(INSTRUCTED),
    })
}

fn written(target: &Path) -> bool {
    target.exists()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_747_moderate_never_runs_a_command_outside_the_sandbox() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    skip_if_sandbox!(Ok(()));
    let mut wrong = Vec::new();
    // (level, route, prompts expected, writes outside)
    for (level, route, asks, writes) in [
        (SecurityLevel::Moderate, Route::Escalated, 1, false),
        // The sandbox denies the write; no retry outside it is offered.
        (SecurityLevel::Moderate, Route::Retry, 1, false),
        // D6: the remembered rule still skips the prompt, inside the sandbox.
        (SecurityLevel::Moderate, Route::Rule, 0, false),
        (SecurityLevel::Permissive, Route::Escalated, 1, true),
        // Approved once, its retry outside the sandbox runs unasked.
        (SecurityLevel::Permissive, Route::Retry, 1, true),
        (SecurityLevel::Permissive, Route::Rule, 0, true),
        (SecurityLevel::Moderate, Route::EscalatedExec, 1, false),
        (SecurityLevel::Permissive, Route::EscalatedExec, 1, true),
        (SecurityLevel::Moderate, Route::Patch, 1, true),
        (SecurityLevel::Permissive, Route::Patch, 1, true),
        // Review r1 finding 1: Core's Aggressive without its launch profile
        // (confirmed before the restart, or only `[security] level`; an
        // unreadable policy counts as Aggressive) keeps every command in the
        // sandbox already: its protected-path rules apply from the start and
        // no approval lifts them, so their denied reads keep the run in it.
        (SecurityLevel::Aggressive, Route::Escalated, 1, false),
        (SecurityLevel::Aggressive, Route::EscalatedExec, 1, false),
        (SecurityLevel::Aggressive, Route::Rule, 0, false),
        // #840 (D6): a rule saved under Permissive still skips the prompt of
        // a command that asks to leave the sandbox; it runs inside it.
        (SecurityLevel::Moderate, Route::RuleEscalated, 0, false),
        (SecurityLevel::Moderate, Route::RuleEscalatedExec, 0, false),
        (SecurityLevel::Permissive, Route::RuleEscalated, 0, true),
        (SecurityLevel::Permissive, Route::RuleEscalatedExec, 0, true),
        // Aggressive suspends rules not saved under it (#626): it asks.
        (SecurityLevel::Aggressive, Route::RuleEscalated, 1, false),
        (
            SecurityLevel::Aggressive,
            Route::RuleEscalatedExec,
            1,
            false,
        ),
    ] {
        let run = run(level, route).await?;
        let moderate = level == SecurityLevel::Moderate;
        // The prompt note and the model instruction are Moderate's only.
        // Only a request to leave the sandbox gets the note on its prompt.
        let noted = moderate && matches!(route, Route::Escalated | Route::EscalatedExec);
        if run.asked != asks
            || run.wrote != writes
            || run.reasons.iter().any(|reason| reason.contains(STAYS)) != noted
            || run.reasons.iter().any(|reason| reason.contains(LIFTS)) != noted
            || run
                .reasons
                .iter()
                .any(|reason| reason.contains("credential protection"))
            || run.instructed != moderate
            || run.output.contains("Protected launch refused")
        {
            wrong.push(format!(
                "{level:?} {route:?}: asked={} wrote={} instructed={} \
                 reasons={:?} output={:?}",
                run.asked, run.wrote, run.instructed, run.reasons, run.output
            ));
        }
    }
    assert_eq!(wrong, Vec::<String>::new());
    Ok(())
}
