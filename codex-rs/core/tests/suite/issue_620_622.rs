//! Issues #620, #621 and #622 (milestone "Aggressive ships" re-run, AGG-18,
//! AGG-R07, AGG-R09, AGG-R17), end to end through the orchestrator and the
//! real sandbox (Seatbelt on macOS, bubblewrap on Linux):
//! - #621: an interpreter that lists the Corbanu home (`python3 -c
//!   "…os.listdir('<home>')…"`, `perl -e`) is recognised as a protected
//!   read before it runs: no grant is offered and the prompt says it cannot
//!   be granted, not even once; the denial is explained where it fails.
//! - #620: an append to `~/.zshrc`, a start-up file outside the current
//!   folder, is offered no grant either (Aggressive's sandbox never writes
//!   there, so a grant could not apply), and its denial says so.
//! - #622: both denials are listed under the inspector's recent denials.
#![cfg(any(target_os = "macos", target_os = "linux"))]

use std::sync::Arc;

use codex_core::config::Constrained;
use codex_core::security_grant;
use codex_core::security_inspection;
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
use serde_json::json;
use tempfile::TempDir;

const CANARY: &str = "listing-canary-621";
const NEVER: &str = "cannot be granted, not even once";

/// Builds a command from the Corbanu home's path.
type Command = fn(&str) -> String;

/// What one approved command showed.
struct Run {
    /// Whether Core offered a grant on its approval prompt.
    offered: bool,
    /// The approval prompt's reason.
    reason: String,
    /// What the model got back.
    output: String,
    /// Warnings shown to the person.
    warnings: Vec<String>,
    /// The inspector's recent denials for the session (reason, outcome).
    denials: Vec<(&'static str, &'static str)>,
}

/// Runs `command` (built from the Corbanu home's path) in an Aggressive
/// session, approves it once (y, no grant) and reports what happened. With
/// `shell_home`, agent commands get that folder as `HOME`.
async fn run(
    command: impl FnOnce(&str) -> String,
    shell_home: Option<&std::path::Path>,
) -> anyhow::Result<Run> {
    let shell_home = shell_home.map(|home| home.display().to_string());
    let home = Arc::new(TempDir::new()?);
    let path = home.path().canonicalize()?;
    std::fs::write(path.join(CANARY), "x")?;
    let command = command(&path.display().to_string());
    let server = start_mock_server().await;
    let captured = mount_sse_sequence(
        &server,
        vec![
            sse(vec![
                ev_response_created("call-0"),
                ev_function_call(
                    "call-0",
                    "exec_command",
                    &json!({ "cmd": command }).to_string(),
                ),
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
        .with_home(home)
        .with_model("test-gpt-5.1-codex")
        .with_config(move |config| {
            if let Some(home) = shell_home {
                config
                    .permissions
                    .shell_environment_policy
                    .r#set
                    .insert("HOME".to_string(), home);
            }
            config.security_level = SecurityLevel::Aggressive;
            config.permissions.approval_policy =
                Constrained::allow_any(AskForApproval::UnlessTrusted);
            // Aggressive's own sandbox: writes only in the current folder,
            // not in /tmp or $TMPDIR either.
            config
                .permissions
                .set_permission_profile(PermissionProfile::workspace_write_with(
                    &[],
                    NetworkSandboxPolicy::Restricted,
                    /*exclude_tmpdir_env_var*/ true,
                    /*exclude_slash_tmp*/ true,
                ))
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
    let EventMsg::ExecApprovalRequest(approval) = wait_for_event(&test.codex, |event| {
        matches!(
            event,
            EventMsg::ExecApprovalRequest(_) | EventMsg::TurnComplete(_)
        )
    })
    .await
    else {
        anyhow::bail!("the command did not ask first: {command}");
    };
    let offered = security_grant::offer(thread, &approval.effective_approval_id()).is_some();
    test.codex
        .submit(Op::ExecApproval {
            id: approval.effective_approval_id(),
            turn_id: None,
            decision: ReviewDecision::Approved,
        })
        .await?;
    let mut warnings = Vec::new();
    while let EventMsg::Warning(warning) = wait_for_event(&test.codex, |event| {
        matches!(event, EventMsg::TurnComplete(_) | EventMsg::Warning(_))
    })
    .await
    {
        warnings.push(warning.message);
    }
    let requests = captured.requests();
    let last = requests.last().expect("the model was asked again");
    let denials = security_inspection::observe(
        test.codex_home_path(),
        SecurityLevel::Aggressive,
        Some(thread),
        i64::MAX / 2,
    )
    .denials
    .into_iter()
    .map(|denial| (denial.reason, denial.outcome))
    .collect();
    Ok(Run {
        offered,
        reason: approval.reason.unwrap_or_default(),
        output: last.function_call_output_text("call-0").unwrap_or_default(),
        warnings,
        denials,
    })
}

/// Whether `program` runs here (outside any sandbox).
fn runs(program: &str, args: &[&str]) -> bool {
    std::process::Command::new(program)
        .args(args)
        .output()
        .is_ok_and(|output| output.status.success())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_621_622_interpreter_listing_of_the_home_is_never_granted_and_is_listed()
-> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    skip_if_sandbox!(Ok(()));
    fn python(home: &str) -> String {
        format!("python3 -c \"import os; print(os.listdir('{home}'))\"")
    }
    fn perl(home: &str) -> String {
        format!("perl -e 'opendir(my $d, \"{home}\") or die \"$!: {home}\\n\"; print readdir($d)'")
    }
    let commands: Vec<(&str, Command)> = [("perl", perl as Command), ("python3", python)]
        .into_iter()
        .filter(|(program, _)| runs(program, &["-e", "0"]) || runs(program, &["-c", "0"]))
        .collect();
    assert!(!commands.is_empty(), "neither perl nor python3 runs here");
    for (name, command) in commands {
        let run = run(command, /*shell_home*/ None).await?;
        assert!(!run.offered, "{name}: no grant for a protected read");
        assert!(run.reason.contains(NEVER), "{name}: {}", run.reason);
        assert!(!run.output.contains(CANARY), "{name}: {}", run.output);
        let warning = run
            .warnings
            .iter()
            .find(|warning| warning.starts_with("Security level Aggressive blocked listing"))
            .unwrap_or_else(|| panic!("{name}: not explained: {:?}", run.warnings));
        assert!(warning.contains(NEVER), "{name}: {warning}");
        assert!(
            run.output.contains(warning.as_str()),
            "{name}: {}",
            run.output
        );
        assert!(
            run.denials.contains(&(
                "listing the Corbanu home",
                "denied by the sandbox; never granted under Aggressive"
            )),
            "{name}: {:?}",
            run.denials
        );
    }
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_620_startup_file_outside_the_folder_is_offered_no_grant() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    skip_if_sandbox!(Ok(()));
    // The command's `~` is a folder of its own (Core still names the start-up
    // file by the real home folder), and it appends nothing: no start-up
    // file of the person running the test is touched, whatever happens.
    let shell_home = TempDir::new()?;
    let run = run(
        |_| "printf '' >> ~/.zshrc".to_string(),
        Some(shell_home.path()),
    )
    .await?;
    assert!(!shell_home.path().join(".zshrc").exists());
    assert!(!run.offered, "no grant for ~/.zshrc");
    assert!(
        run.reason
            .contains("Security level Aggressive blocks writing `~/.zshrc`"),
        "{}",
        run.reason
    );
    assert!(run.reason.contains(NEVER), "{}", run.reason);
    let output = run.output.to_lowercase();
    assert!(
        [
            "operation not permitted",
            "permission denied",
            "read-only file system"
        ]
        .iter()
        .any(|denial| output.contains(denial)),
        "{}",
        run.output
    );
    assert!(
        run.warnings
            .iter()
            .any(|warning| warning
                .starts_with("Security level Aggressive blocks writing `~/.zshrc`")),
        "{:?}",
        run.warnings
    );
    assert!(
        run.denials.contains(&(
            "writing a start-up file outside the current folder",
            "denied by the sandbox; never granted under Aggressive"
        )),
        "{:?}",
        run.denials
    );
    Ok(())
}

/// #902: the harness gives the path lookups these tests rely on the test
/// bound, so a loaded machine does not drop the explanation or the withheld
/// grant offer (they took over the 2 s production bound on a busy Mac).
#[test]
fn issue_902_harness_gives_the_lookups_the_test_bound() {
    assert_eq!(
        codex_core::test_support::security_lookup_timeout(),
        std::time::Duration::from_secs(20)
    );
}
