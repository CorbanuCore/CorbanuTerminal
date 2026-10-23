//! #748: under security level Moderate, a command its protected-path rules
//! stopped names the level, what is protected and the way forward, to the
//! person (a warning) and to the model (a note on the output), through both
//! shell tools. What Moderate's rules did not add (a write outside the
//! project, or to `.codex`, which workspace-write keeps read-only itself) is
//! not attributed to it, and Permissive is unchanged.

use std::sync::Arc;

use codex_core::config::Constrained;
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
use pretty_assertions::assert_eq;
use serde_json::Value;
use serde_json::json;
use tempfile::TempDir;

const CANARY: &str = "ISSUE748_CANARY";
const CALL_ID: &str = "call-748";

fn call(call_id: &str, name: &str, arguments: Value) -> Vec<Value> {
    vec![
        ev_response_created(call_id),
        ev_function_call(call_id, name, &arguments.to_string()),
        ev_completed(call_id),
    ]
}

struct Outcome {
    /// The output the model got.
    output: String,
    warnings: Vec<String>,
    wrote_codex: bool,
    /// #839: the inspector's recent denials of the session.
    denials: Vec<(&'static str, &'static str)>,
}

/// What each session runs: one command, as the first tool call (a later one
/// would follow untrusted content, the first call's output).
#[derive(Clone, Copy, Debug)]
enum Command {
    /// `cat` of the Corbanu home's `auth.json`, through `shell_command`.
    Read,
    /// The same through `exec_command`.
    ReadExec,
    /// A write to the project's `.codex` with full access, where only
    /// Moderate's rules keep it read-only.
    WriteCodexFullAccess,
    /// The same under workspace-write, which keeps `.codex` read-only itself.
    WriteCodex,
    /// A write beside the project, which the person's own sandbox denies.
    WriteOutside,
}

/// Runs `command` as the session's first tool call, or with `approved`
/// after untrusted content (another tool's output), approved by the person
/// when asked.
async fn run(level: SecurityLevel, command: Command, approved: bool) -> anyhow::Result<Outcome> {
    let home = Arc::new(TempDir::new()?);
    let auth = home.path().canonicalize()?.join("auth.json");
    std::fs::write(&auth, CANARY)?;
    let read = format!("cat '{}'", auth.display());
    let (name, arguments) = match command {
        Command::Read => ("shell_command", json!({ "command": read })),
        Command::ReadExec => (
            "exec_command",
            json!({ "cmd": read, "yield_time_ms": 5_000 }),
        ),
        Command::WriteCodex | Command::WriteCodexFullAccess => (
            "shell_command",
            json!({ "command": "printf x > .codex/p748.toml" }),
        ),
        Command::WriteOutside => (
            "shell_command",
            json!({ "command": "printf x > ../outside-748.txt" }),
        ),
    };
    let server = start_mock_server().await;
    let mut steps = vec![
        call(CALL_ID, name, arguments),
        vec![
            ev_response_created("done"),
            ev_assistant_message("msg-done", "done"),
            ev_completed("done"),
        ],
    ];
    let approval = if approved {
        steps.insert(
            0,
            call(
                "call-untrusted",
                "shell_command",
                json!({ "command": "printf 'notes'" }),
            ),
        );
        AskForApproval::OnRequest
    } else {
        AskForApproval::Never
    };
    let captured = mount_sse_sequence(&server, steps.into_iter().map(sse).collect()).await;
    let test = test_codex()
        .with_home(home)
        .with_model("test-gpt-5.1-codex")
        .with_config(move |config| {
            config.security_level = level;
            config.permissions.approval_policy = Constrained::allow_any(approval);
            for feature in [Feature::SourceEnvelopes, Feature::UnifiedExec] {
                config
                    .features
                    .enable(feature)
                    .expect("test config should allow feature update");
            }
            std::fs::create_dir_all(config.cwd.join(".codex")).expect("the project's .codex");
            // Workspace-write without the temporary folders, so a write
            // beside the project is denied by the person's own sandbox.
            let profile = if matches!(command, Command::WriteCodexFullAccess) {
                PermissionProfile::Disabled
            } else {
                PermissionProfile::workspace_write_with(
                    &[],
                    NetworkSandboxPolicy::Restricted,
                    /*exclude_tmpdir_env_var*/ true,
                    /*exclude_slash_tmp*/ true,
                )
            };
            config
                .permissions
                .set_permission_profile(profile)
                .expect("profile");
        })
        .build(&server)
        .await?;
    test.codex
        .submit(Op::UserInput {
            items: vec![UserInput::Text {
                text: "check the notes".into(),
                text_elements: Vec::new(),
            }],
            final_output_json_schema: None,
            responsesapi_client_metadata: None,
            additional_context: Default::default(),
            thread_settings: Default::default(),
        })
        .await?;
    let mut warnings = Vec::new();
    loop {
        match wait_for_event(&test.codex, |event| {
            matches!(
                event,
                EventMsg::Warning(_) | EventMsg::ExecApprovalRequest(_) | EventMsg::TurnComplete(_)
            )
        })
        .await
        {
            EventMsg::Warning(warning) => warnings.push(warning.message),
            EventMsg::ExecApprovalRequest(request) => {
                test.codex
                    .submit(Op::ExecApproval {
                        id: request.effective_approval_id(),
                        turn_id: None,
                        decision: ReviewDecision::Approved,
                    })
                    .await?;
            }
            _ => break,
        }
    }
    let output = captured
        .requests()
        .iter()
        .find_map(|request| request.function_call_output_text(CALL_ID))
        .unwrap_or_default();
    let wrote_codex = test.cwd_path().join(".codex/p748.toml").exists();
    let denials = security_inspection::observe(
        test.codex_home_path(),
        level,
        Some(test.session_configured.thread_id),
        i64::MAX / 2,
    )
    .denials
    .into_iter()
    .map(|denial| (denial.reason, denial.outcome))
    .collect();
    Ok(Outcome {
        output,
        warnings,
        wrote_codex,
        denials,
    })
}

/// The warnings that name a security level.
fn level_warnings(outcome: &Outcome) -> Vec<&String> {
    outcome
        .warnings
        .iter()
        .filter(|warning| warning.starts_with("Security level"))
        .collect()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_748_moderate_refusals_name_the_level() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    skip_if_sandbox!(Ok(()));
    let mut wrong = Vec::new();
    for (command, expected) in [
        (
            Command::Read,
            Some("Security level Moderate blocked access to"),
        ),
        (
            Command::ReadExec,
            Some("Security level Moderate blocked access to"),
        ),
        (
            Command::WriteCodexFullAccess,
            Some("Security level Moderate blocked writing `.codex/p748.toml`"),
        ),
        // The person's own sandbox stopped these, not Moderate.
        (Command::WriteCodex, None),
        (Command::WriteOutside, None),
    ] {
        let moderate = run(SecurityLevel::Moderate, command, /*approved*/ false).await?;
        let warnings = level_warnings(&moderate);
        let explained = match expected {
            Some(expected) => {
                warnings.len() == 1
                    && warnings[0].starts_with(expected)
                    && warnings[0].contains("a person can switch to Permissive in /security")
                    && moderate.output.contains(warnings[0].as_str())
            }
            None => warnings.is_empty() && !moderate.output.contains("Security level"),
        };
        // #839: what Moderate explained is listed under recent denials too,
        // and nothing else is.
        let listed = match command {
            Command::Read | Command::ReadExec => vec![(
                "reading a protected file",
                "denied by the sandbox under Moderate",
            )],
            Command::WriteCodexFullAccess => vec![(
                "writing a file that runs code later, or Corbanu's own files",
                "denied by the sandbox under Moderate",
            )],
            Command::WriteCodex | Command::WriteOutside => Vec::new(),
        };
        if moderate.denials != listed {
            wrong.push(format!(
                "Moderate {command:?} recent denials: {:?}",
                moderate.denials
            ));
        }
        if !explained || moderate.output.contains(CANARY) || moderate.wrote_codex {
            wrong.push(format!(
                "Moderate {command:?}: {:?} {}",
                moderate.warnings, moderate.output
            ));
        }

        let permissive = run(SecurityLevel::Permissive, command, /*approved*/ false).await?;
        let reads = matches!(command, Command::Read | Command::ReadExec);
        if !level_warnings(&permissive).is_empty()
            || permissive.output.contains("Security level")
            || (reads && !permissive.output.contains(CANARY))
            || permissive.wrote_codex != matches!(command, Command::WriteCodexFullAccess)
        {
            wrong.push(format!(
                "Permissive {command:?}: {:?} {}",
                permissive.warnings, permissive.output
            ));
        }
    }
    assert_eq!(wrong, Vec::<String>::new());
    Ok(())
}

/// #748 review: a protected read a person approves after untrusted content
/// runs with Moderate's rules lifted, so nothing blames Moderate for it.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_748_an_approved_read_is_not_attributed_to_moderate() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    skip_if_sandbox!(Ok(()));
    let approved = run(
        SecurityLevel::Moderate,
        Command::Read,
        /*approved*/ true,
    )
    .await?;
    assert!(approved.output.contains(CANARY), "{}", approved.output);
    assert!(
        level_warnings(&approved).is_empty(),
        "{:?}",
        approved.warnings
    );
    assert!(
        !approved.output.contains("Security level"),
        "{}",
        approved.output
    );
    Ok(())
}
