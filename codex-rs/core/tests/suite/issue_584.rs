//! Issue #584: under Aggressive the project's `.codex` and `.agents` stay
//! read-only whatever anyone answers (#520), so nobody is asked about a
//! write there that is refused afterwards anyway, end to end through the
//! orchestrator and the real sandbox (Seatbelt on macOS, bubblewrap on
//! Linux):
//! - `request_permissions` for such a write is refused before anyone is
//!   asked, and the person and the model are told why; other entries of the
//!   same request are still asked about, without it;
//! - a command that writes only there is still asked about (Aggressive asks
//!   about every command) but offered no "Grant once"; its prompt and its
//!   denial say it cannot be granted, not even once. A command that also
//!   writes a git hook keeps the offer (`pf_25_s01`).
#![cfg(any(target_os = "macos", target_os = "linux"))]

use std::sync::Arc;

use codex_core::config::Constrained;
use codex_core::security_grant;
use codex_features::Feature;
use codex_protocol::models::FileSystemPermissions;
use codex_protocol::models::PermissionProfile;
use codex_protocol::permissions::NetworkSandboxPolicy;
use codex_protocol::protocol::AskForApproval;
use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::Op;
use codex_protocol::protocol::ReviewDecision;
use codex_protocol::request_permissions::PermissionGrantScope;
use codex_protocol::request_permissions::RequestPermissionProfile;
use codex_protocol::request_permissions::RequestPermissionsResponse;
use codex_protocol::user_input::UserInput;
use codex_security_policy::SecurityLevel;
use codex_utils_absolute_path::AbsolutePathBuf;
use core_test_support::responses::ResponseMock;
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
use serde_json::Value;
use serde_json::json;
use tempfile::TempDir;

const NEVER: &str = "cannot be granted, not even once";
const NEVER_ADDITIONAL: &str = "neither an additional permission nor \"Grant once\"";

fn tool_step(call_id: &str, tool: &str, arguments: Value) -> Vec<Value> {
    vec![
        ev_response_created(call_id),
        ev_function_call(call_id, tool, &arguments.to_string()),
        ev_completed(call_id),
    ]
}

fn done_step() -> Vec<Value> {
    vec![
        ev_response_created("done"),
        ev_assistant_message("msg-done", "done"),
        ev_completed("done"),
    ]
}

/// An Aggressive session (Aggressive's own sandbox: writes only in the
/// current folder) in `workspace`, which has `.codex`, `.agents` and
/// `.git/hooks`, with the permission-request tool on (it is off in a TUI
/// launched under Aggressive, not in other clients or before the restart),
/// running `steps`.
async fn start(
    workspace: &AbsolutePathBuf,
    steps: Vec<Vec<Value>>,
) -> anyhow::Result<(TestCodex, ResponseMock)> {
    for folder in [".codex", ".agents", ".git/hooks"] {
        std::fs::create_dir_all(workspace.join(folder).as_path())?;
    }
    let server = start_mock_server().await;
    let captured = mount_sse_sequence(&server, steps.into_iter().map(sse).collect()).await;
    let session_cwd = workspace.clone();
    let test = test_codex()
        .with_home(Arc::new(TempDir::new()?))
        .with_model("test-gpt-5.1-codex")
        .with_config(move |config| {
            config.cwd = session_cwd;
            config.security_level = SecurityLevel::Aggressive;
            config.permissions.approval_policy =
                Constrained::allow_any(AskForApproval::UnlessTrusted);
            config
                .permissions
                .set_permission_profile(PermissionProfile::workspace_write_with(
                    &[],
                    NetworkSandboxPolicy::Restricted,
                    /*exclude_tmpdir_env_var*/ true,
                    /*exclude_slash_tmp*/ true,
                ))
                .expect("workspace write");
            for feature in [Feature::SourceEnvelopes, Feature::RequestPermissionsTool] {
                config.features.enable(feature).expect("enable feature");
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
                text: "update the agent settings".into(),
                text_elements: Vec::new(),
            }],
            final_output_json_schema: None,
            responsesapi_client_metadata: None,
            additional_context: Default::default(),
            thread_settings: Default::default(),
        })
        .await?;
    Ok((test, captured))
}

fn workspace() -> anyhow::Result<(TempDir, AbsolutePathBuf)> {
    let dir = TempDir::new()?;
    let path = AbsolutePathBuf::from_absolute_path(dir.path().canonicalize()?)?;
    Ok((dir, path))
}

fn writes(paths: &[AbsolutePathBuf]) -> RequestPermissionProfile {
    RequestPermissionProfile {
        file_system: Some(FileSystemPermissions::from_read_write_roots(
            /*read*/ None,
            Some(paths.to_vec()),
        )),
        ..Default::default()
    }
}

fn request_step(permissions: &RequestPermissionProfile) -> Vec<Value> {
    tool_step(
        "call-grant",
        "request_permissions",
        json!({ "reason": "update the agent settings", "permissions": permissions }),
    )
}

/// Warnings shown until the turn completes; panics if anyone is asked.
async fn warnings_until_done(test: &TestCodex) -> Vec<String> {
    let mut warnings = Vec::new();
    loop {
        match wait_for_event(&test.codex, |event| {
            matches!(
                event,
                EventMsg::Warning(_)
                    | EventMsg::RequestPermissions(_)
                    | EventMsg::ExecApprovalRequest(_)
                    | EventMsg::TurnComplete(_)
            )
        })
        .await
        {
            EventMsg::Warning(warning) => warnings.push(warning.message),
            EventMsg::TurnComplete(_) => return warnings,
            other => panic!("nobody should be asked: {other:?}"),
        }
    }
}

/// A request only for the project's `.codex` and `.agents` is refused
/// before anyone is asked; the person and the model are told why.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_584_a_permission_request_no_grant_can_lift_is_refused_unasked() -> anyhow::Result<()>
{
    skip_if_no_network!(Ok(()));
    skip_if_sandbox!(Ok(()));
    let (_dir, cwd) = workspace()?;
    let request = writes(&[cwd.join(".codex"), cwd.join(".agents")]);
    let (test, captured) = start(&cwd, vec![request_step(&request), done_step()]).await?;
    let warnings = warnings_until_done(&test).await;
    assert!(
        warnings
            .iter()
            .any(|warning| warning.contains(NEVER_ADDITIONAL)),
        "{warnings:?}"
    );
    let requests = captured.requests();
    let output = requests[requests.len() - 1]
        .function_call_output_text("call-grant")
        .unwrap_or_default();
    assert!(output.contains(NEVER_ADDITIONAL), "{output}");
    assert!(output.contains("Permissive in /security"), "{output}");
    Ok(())
}

/// The other entries of the same request are still asked about, without
/// the `.codex` one; the person is told why it was left out.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_584_a_permission_request_is_asked_without_what_no_grant_can_lift()
-> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    skip_if_sandbox!(Ok(()));
    let (_dir, cwd) = workspace()?;
    let (_outside_dir, outside) = workspace()?;
    let request = writes(&[cwd.join(".codex"), outside.clone()]);
    let (test, captured) = start(&cwd, vec![request_step(&request), done_step()]).await?;
    let mut warned = false;
    let asked = loop {
        match wait_for_event(&test.codex, |event| {
            matches!(
                event,
                EventMsg::Warning(_) | EventMsg::RequestPermissions(_) | EventMsg::TurnComplete(_)
            )
        })
        .await
        {
            EventMsg::Warning(warning) => warned |= warning.message.contains(NEVER_ADDITIONAL),
            EventMsg::RequestPermissions(request) => break request.permissions,
            other => panic!("the other entry should be asked about: {other:?}"),
        }
    };
    assert!(warned, "the person is told why .codex was left out");
    let asked_json = serde_json::to_string(&asked)?;
    let codex = cwd.join(".codex").as_path().to_string_lossy().into_owned();
    assert!(!asked_json.contains(&codex), "{asked_json}");
    assert!(
        asked_json.contains(outside.as_path().to_string_lossy().as_ref()),
        "{asked_json}"
    );
    test.codex
        .submit(Op::RequestPermissionsResponse {
            id: "call-grant".to_string(),
            response: RequestPermissionsResponse {
                permissions: asked,
                scope: PermissionGrantScope::Turn,
                strict_auto_review: false,
            },
        })
        .await?;
    wait_for_event(&test.codex, |event| {
        matches!(event, EventMsg::TurnComplete(_))
    })
    .await;
    let requests = captured.requests();
    let granted = requests[requests.len() - 1]
        .function_call_output_text("call-grant")
        .unwrap_or_default();
    assert!(!granted.contains(&codex), "{granted}");
    Ok(())
}

/// A command that writes only `.codex` and `.agents` is asked about (every
/// command is, under Aggressive) but offered no grant; the prompt says why,
/// and the real sandbox still refuses both writes, explained where they
/// fail.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_584_a_command_writing_only_agent_settings_is_offered_no_grant() -> anyhow::Result<()>
{
    skip_if_no_network!(Ok(()));
    skip_if_sandbox!(Ok(()));
    let (_dir, cwd) = workspace()?;
    // Fails (the last write's status), so the denial is explained.
    let write = "printf x > .agents/issue584.md; echo agents=$?; \
                 printf x > .codex/issue584.toml";
    let (test, captured) = start(
        &cwd,
        vec![
            tool_step("call-write", "shell_command", json!({ "command": write })),
            done_step(),
        ],
    )
    .await?;
    let thread = test.session_configured.thread_id;
    let EventMsg::ExecApprovalRequest(approval) = wait_for_event(&test.codex, |event| {
        matches!(
            event,
            EventMsg::ExecApprovalRequest(_) | EventMsg::TurnComplete(_)
        )
    })
    .await
    else {
        panic!("the write asks first");
    };
    assert!(
        security_grant::offer(thread, &approval.effective_approval_id()).is_none(),
        "no Grant once for a write no grant lifts"
    );
    let reason = approval.reason.clone().unwrap_or_default();
    assert!(reason.contains(NEVER), "{reason}");
    // The first path it writes, as spelled.
    assert!(reason.contains("`.agents/issue584.md`"), "{reason}");
    test.codex
        .submit(Op::ExecApproval {
            id: approval.effective_approval_id(),
            turn_id: None,
            decision: ReviewDecision::Approved,
        })
        .await?;
    let warnings = warnings_until_done(&test).await;
    let requests = captured.requests();
    let output = requests[requests.len() - 1]
        .function_call_output_text("call-write")
        .unwrap_or_default();
    assert!(output.contains("agents=1"), "{output}");
    assert!(output.contains("Exit code: 1"), "{output}");
    assert!(!cwd.join(".codex/issue584.toml").as_path().exists());
    assert!(!cwd.join(".agents/issue584.md").as_path().exists());
    // The denial names the first write it reports (both were denied).
    assert!(
        warnings
            .iter()
            .any(|warning| warning.contains(NEVER) && warning.contains("`.agents/issue584.md`")),
        "the denial is explained: {warnings:?} / {output}"
    );
    Ok(())
}
