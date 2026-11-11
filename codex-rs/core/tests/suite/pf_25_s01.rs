//! PF-25-S01: an Aggressive grant confirmed for a command approval lifts the
//! write protections for that run only, end to end through the orchestrator
//! (offer key = the approval id the TUI answers, digest at offer = digest at
//! run). #442: reads of credential files and the Corbanu home are never
//! granted.
//!
//! Each session runs in its own workspace root, as a real session does
//! (#1098): there the project's `.git`, `.codex` and `.agents` are read-only
//! (the upstream sandbox's protection) and no grant lifts them, so the hook a
//! grant writes is in a `core.hooksPath` folder (`.husky`), as the docs say.
#![cfg(target_os = "macos")]

use std::path::Path;
use std::sync::Arc;

use codex_core::config::Constrained;
use codex_core::security_grant;
use codex_core::security_grant::GrantUses;
use codex_features::Feature;
use codex_protocol::models::PermissionProfile;
use codex_protocol::protocol::AskForApproval;
use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::ExecApprovalRequestEvent;
use codex_protocol::protocol::Op;
use codex_protocol::protocol::ReviewDecision;
use codex_protocol::user_input::UserInput;
use codex_security_policy::SecurityLevel;
use core_test_support::responses::ResponseMock;
use core_test_support::responses::ResponsesRequest;
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

const CANARY: &str = "pf25-canary-notes";

type Step = Vec<Value>;

fn shell_step(call_id: &str, command: &str) -> Step {
    let arguments = json!({ "command": command }).to_string();
    vec![
        ev_response_created(call_id),
        ev_function_call(call_id, "shell_command", &arguments),
        ev_completed(call_id),
    ]
}

fn done_step() -> Step {
    vec![
        ev_response_created("done"),
        ev_assistant_message("msg-done", "done"),
        ev_completed("done"),
    ]
}

/// A file in the Corbanu home, and a command that reads it without naming
/// the home folder.
fn canary_home() -> anyhow::Result<(Arc<TempDir>, String)> {
    let home = Arc::new(TempDir::new()?);
    let path = home.path().canonicalize()?.join("pf25-notes.txt");
    std::fs::write(&path, CANARY)?;
    let path = path.to_string_lossy().into_owned();
    let folder = path.len() - "/pf25-notes.txt".len();
    let (head, tail) = super::pf_23_s01::split_home_name(&path[..folder]);
    Ok((
        home,
        format!("cat \"$(printf '{head}%s/pf25-notes.txt' {tail})\""),
    ))
}

async fn start(home: Arc<TempDir>, steps: Vec<Step>) -> anyhow::Result<(TestCodex, ResponseMock)> {
    start_in(home, steps, |_| Ok(())).await
}

/// [`start`], with `project` run on the session's folder before the turn.
async fn start_in(
    home: Arc<TempDir>,
    steps: Vec<Step>,
    project: impl FnOnce(&Path) -> std::io::Result<()>,
) -> anyhow::Result<(TestCodex, ResponseMock)> {
    let server = start_mock_server().await;
    let captured = mount_sse_sequence(&server, steps.into_iter().map(sse).collect()).await;
    let test = test_codex()
        .with_home(home)
        .with_session_cwd_as_workspace_root()
        .with_model("test-gpt-5.1-codex")
        .with_config(|config| {
            config.security_level = SecurityLevel::Aggressive;
            config.permissions.approval_policy =
                Constrained::allow_any(AskForApproval::UnlessTrusted);
            // Aggressive's own sandbox: writes only in the current folder.
            config
                .permissions
                .set_permission_profile(PermissionProfile::workspace_write())
                .expect("workspace write");
            config
                .features
                .enable(Feature::SourceEnvelopes)
                .expect("enable source envelopes");
        })
        .build_with_auto_env(&server)
        .await?;
    std::mem::forget(server);
    project(test.cwd_path())?;
    test.codex
        .submit(Op::UserInput {
            items: vec![UserInput::Text {
                text: "read the notes".into(),
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

/// A git project whose hooks folder is `.husky` (`core.hooksPath`), with the
/// project's `.git/hooks`, `.codex` and `.agents` folders.
fn husky_project(cwd: &Path) -> std::io::Result<()> {
    for folder in [".git/hooks", ".husky", ".codex", ".agents"] {
        std::fs::create_dir_all(cwd.join(folder))?;
    }
    std::fs::write(cwd.join(".git/HEAD"), "ref: refs/heads/main\n")?;
    std::fs::write(cwd.join(".git/config"), "[core]\n\thooksPath = .husky\n")
}

async fn next_approval(test: &TestCodex) -> Option<ExecApprovalRequestEvent> {
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
    }
}

async fn approve(test: &TestCodex, approval: &ExecApprovalRequestEvent) -> anyhow::Result<()> {
    test.codex
        .submit(Op::ExecApproval {
            id: approval.effective_approval_id(),
            turn_id: None,
            decision: ReviewDecision::Approved,
        })
        .await?;
    Ok(())
}

fn output_text(request: &ResponsesRequest, call_id: &str) -> String {
    request
        .function_call_output_text(call_id)
        .unwrap_or_default()
}

/// The first run, granted "1 run" in the review and approved, writes a git
/// hook in the `core.hooksPath` folder (a write protection); the same command
/// again, only approved, gets the rules.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn pf_25_s01_confirmed_grant_lifts_the_write_protections_for_the_approved_run_only()
-> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    skip_if_sandbox!(Ok(()));
    let home = Arc::new(TempDir::new()?);
    let write = "printf pf25 > .husky/pf25-hook";
    let (test, captured) = start_in(
        home,
        vec![
            shell_step("call-granted", write),
            shell_step("call-again", write),
            done_step(),
        ],
        husky_project,
    )
    .await?;
    let thread = test.session_configured.thread_id;
    let hooks = test.cwd_path().join(".husky");

    let first = next_approval(&test).await.expect("the write asks first");
    let offer = security_grant::offer(thread, &first.effective_approval_id())
        .expect("Core offers a grant for the approval the TUI answers");
    assert_eq!(offer.command, first.command);
    security_grant::confirm(&offer, GrantUses::Once)?;
    approve(&test, &first).await?;

    let second = next_approval(&test)
        .await
        .expect("the same write asks again");
    // The granted run wrote the hook before the second command ran.
    assert_eq!(std::fs::read_to_string(hooks.join("pf25-hook"))?, "pf25");
    std::fs::remove_file(hooks.join("pf25-hook"))?;
    assert!(security_grant::offer(thread, &second.effective_approval_id()).is_some());
    approve(&test, &second).await?;
    assert!(next_approval(&test).await.is_none());

    let requests = captured.requests();
    let again = output_text(&requests[requests.len() - 1], "call-again");
    assert!(again.contains("Operation not permitted"), "{again}");
    assert!(!hooks.join("pf25-hook").exists());
    assert!(
        security_grant::held_grants()
            .iter()
            .all(|grant| grant.thread != thread),
        "a 1-run grant is never held"
    );
    Ok(())
}

/// #442 (Travis, 2026-10-11): under Aggressive a read of the Corbanu home
/// (like a credential file) is never granted, not even once: a grant
/// confirmed and approved for it, for one run or held until it expires,
/// still leaves the read denied. (The command hides the path, so a grant is
/// offered; one that names it is offered none, below.)
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_442_a_confirmed_grant_never_lifts_protected_reads() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    skip_if_sandbox!(Ok(()));
    for uses in [GrantUses::Once, GrantUses::UntilExpiry] {
        let (home, read) = canary_home()?;
        let (test, captured) =
            start(home, vec![shell_step("call-granted", &read), done_step()]).await?;
        let thread = test.session_configured.thread_id;
        let approval = next_approval(&test).await.expect("the read asks first");
        let offer =
            security_grant::offer(thread, &approval.effective_approval_id()).expect("offer");
        security_grant::confirm(&offer, uses)?;
        approve(&test, &approval).await?;
        assert!(next_approval(&test).await.is_none());
        let requests = captured.requests();
        let granted = output_text(&requests[requests.len() - 1], "call-granted");
        assert!(!granted.contains(CANARY), "{uses:?}: {granted}");
        assert!(
            granted.contains("Operation not permitted"),
            "{uses:?}: {granted}"
        );
    }
    Ok(())
}

/// #442: a command that names a protected path (here it copies a Corbanu
/// home file) is offered no grant under Aggressive; its approval prompt says
/// why and how to change the level.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_442_no_grant_is_offered_for_a_named_protected_read() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    skip_if_sandbox!(Ok(()));
    let home = Arc::new(TempDir::new()?);
    let path = home.path().canonicalize()?.join("pf25-notes.txt");
    std::fs::write(&path, CANARY)?;
    let copy = format!("cp '{}' copy.txt", path.display());
    let (test, captured) = start(home, vec![shell_step("call-copy", &copy), done_step()]).await?;
    let thread = test.session_configured.thread_id;
    let approval = next_approval(&test).await.expect("the copy asks first");
    assert!(security_grant::offer(thread, &approval.effective_approval_id()).is_none());
    let reason = approval.reason.clone().unwrap_or_default();
    assert!(
        reason.contains("cannot be granted, not even once"),
        "{reason}"
    );
    assert!(reason.contains("Permissive in /security"), "{reason}");
    approve(&test, &approval).await?;
    assert!(next_approval(&test).await.is_none());
    let requests = captured.requests();
    let output = output_text(&requests[requests.len() - 1], "call-copy");
    assert!(!output.contains(CANARY), "{output}");
    assert!(!test.cwd_path().join("copy.txt").exists());
    Ok(())
}

/// A grant confirmed in the review but declined in the approval does not
/// apply: the command does not run, and nothing is held.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn pf_25_s01_declined_approval_drops_the_confirmed_grant() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    skip_if_sandbox!(Ok(()));
    let (home, read) = canary_home()?;
    let (test, captured) = start(home, vec![shell_step("call-read", &read), done_step()]).await?;
    let thread = test.session_configured.thread_id;
    let approval = next_approval(&test).await.expect("asks first");
    let offer = security_grant::offer(thread, &approval.effective_approval_id()).expect("offer");
    security_grant::confirm(&offer, GrantUses::UntilExpiry)?;
    test.codex
        .submit(Op::ExecApproval {
            id: approval.effective_approval_id(),
            turn_id: None,
            decision: ReviewDecision::denied("declined"),
        })
        .await?;
    assert!(next_approval(&test).await.is_none());
    assert!(security_grant::offer(thread, &approval.effective_approval_id()).is_none());
    assert!(
        security_grant::held_grants()
            .iter()
            .all(|grant| grant.thread != thread)
    );
    let requests = captured.requests();
    let output = output_text(&requests[requests.len() - 1], "call-read");
    assert!(!output.contains(CANARY), "{output}");
    Ok(())
}

/// #442 (Travis, 2026-10-11): the project's `.codex` and `.agents` stay
/// read-only under Aggressive even with "Grant once"; the grant still lifts
/// Core's other write protections (a `core.hooksPath` hook here) for the same
/// run.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_442_a_confirmed_grant_never_lifts_codex_or_agents() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    skip_if_sandbox!(Ok(()));
    let home = Arc::new(TempDir::new()?);
    let write = "printf pf25 > .husky/pf25-hook; echo hook=$?; \
                 printf pf25 > .codex/pf25.toml; echo codex=$?; \
                 printf pf25 > .agents/pf25.md; echo agents=$?";
    let (test, captured) = start_in(
        home,
        vec![shell_step("call-granted", write), done_step()],
        husky_project,
    )
    .await?;
    let thread = test.session_configured.thread_id;
    let cwd = test.cwd_path();

    let approval = next_approval(&test).await.expect("the write asks first");
    let offer = security_grant::offer(thread, &approval.effective_approval_id()).expect("offer");
    security_grant::confirm(&offer, GrantUses::Once)?;
    approve(&test, &approval).await?;
    assert!(next_approval(&test).await.is_none());

    let requests = captured.requests();
    let output = output_text(&requests[requests.len() - 1], "call-granted");
    assert!(output.contains("hook=0"), "{output}");
    assert!(output.contains("codex=1"), "{output}");
    assert!(output.contains("agents=1"), "{output}");
    assert_eq!(
        std::fs::read_to_string(cwd.join(".husky/pf25-hook"))?,
        "pf25"
    );
    assert!(!cwd.join(".codex/pf25.toml").exists());
    assert!(!cwd.join(".agents/pf25.md").exists());
    Ok(())
}

/// #740, #1098: in a session's own folder the project's `.git` (its hooks
/// included) is read-only and no grant lifts it, so none is offered for a
/// write to `.git/hooks`; the approval prompt says why, and the approved run
/// writes nothing.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_1098_no_grant_is_offered_for_the_projects_git_hooks() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    skip_if_sandbox!(Ok(()));
    let home = Arc::new(TempDir::new()?);
    let write = "printf pf25 > .git/hooks/pf25-hook; echo hook=$?";
    let (test, captured) = start_in(
        home,
        vec![shell_step("call-hook", write), done_step()],
        husky_project,
    )
    .await?;
    let thread = test.session_configured.thread_id;
    let approval = next_approval(&test).await.expect("the write asks first");
    assert!(security_grant::offer(thread, &approval.effective_approval_id()).is_none());
    let reason = approval.reason.clone().unwrap_or_default();
    assert!(reason.contains("as the project's .git folder"), "{reason}");
    assert!(
        reason.contains("cannot be granted, not even once"),
        "{reason}"
    );
    approve(&test, &approval).await?;
    assert!(next_approval(&test).await.is_none());
    let requests = captured.requests();
    let output = output_text(&requests[requests.len() - 1], "call-hook");
    assert!(output.contains("hook=1"), "{output}");
    assert!(!test.cwd_path().join(".git/hooks/pf25-hook").exists());
    Ok(())
}
