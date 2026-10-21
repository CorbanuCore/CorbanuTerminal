//! #1139 (D94, MOD-11): under security level Moderate, a write into the
//! project's `.git` (a hook, its config) that the sandbox stops names the
//! level and the way forward, to the person (a warning) and to the model (a
//! note on the output), through both shell tools. Nothing is written, and
//! Permissive is unchanged (no level is named).

use std::sync::Arc;

use codex_core::config::Constrained;
use codex_core::security_inspection;
use codex_features::Feature;
use codex_protocol::models::PermissionProfile;
use codex_protocol::permissions::NetworkSandboxPolicy;
use codex_protocol::protocol::AskForApproval;
use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::Op;
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

const CALL_ID: &str = "call-1139";

/// MOD-11's first two items.
#[derive(Clone, Copy, Debug)]
enum Command {
    /// A hook written through `shell_command`.
    Hook,
    /// `git config` through `exec_command`.
    Config,
}

struct Outcome {
    output: String,
    warnings: Vec<String>,
    hook_written: bool,
    config_changed: bool,
    denials: Vec<(&'static str, &'static str)>,
}

async fn run(level: SecurityLevel, command: Command) -> anyhow::Result<Outcome> {
    let (name, arguments) = match command {
        Command::Hook => (
            "shell_command",
            json!({ "command": "printf 'echo hi\\n' > .git/hooks/pre-commit" }),
        ),
        Command::Config => (
            "exec_command",
            json!({ "cmd": "git config user.name injected1139", "yield_time_ms": 5_000 }),
        ),
    };
    let server = start_mock_server().await;
    let steps = vec![
        vec![
            ev_response_created(CALL_ID),
            ev_function_call(CALL_ID, name, &arguments.to_string()),
            ev_completed(CALL_ID),
        ],
        vec![
            ev_response_created("done"),
            ev_assistant_message("msg-done", "done"),
            ev_completed("done"),
        ],
    ];
    let captured = mount_sse_sequence(&server, steps.into_iter().map(sse).collect()).await;
    let test = test_codex()
        .with_home(Arc::new(TempDir::new()?))
        .with_model("test-gpt-5.1-codex")
        .with_config(move |config| {
            config.security_level = level;
            config.permissions.approval_policy = Constrained::allow_any(AskForApproval::Never);
            for feature in [Feature::SourceEnvelopes, Feature::UnifiedExec] {
                config
                    .features
                    .enable(feature)
                    .expect("test config should allow feature update");
            }
            let status = std::process::Command::new("git")
                .args(["init", "-q"])
                .current_dir(config.cwd.as_path())
                .status()
                .expect("git init");
            assert!(status.success(), "git init");
            std::fs::create_dir_all(config.cwd.join(".git/hooks")).expect(".git/hooks");
            config
                .permissions
                .set_permission_profile(PermissionProfile::workspace_write_with(
                    &[],
                    NetworkSandboxPolicy::Restricted,
                    /*exclude_tmpdir_env_var*/ true,
                    /*exclude_slash_tmp*/ true,
                ))
                .expect("profile");
        })
        .build(&server)
        .await?;
    test.codex
        .submit(Op::UserInput {
            items: vec![UserInput::Text {
                text: "set up the hook".into(),
                text_elements: Vec::new(),
            }],
            final_output_json_schema: None,
            responsesapi_client_metadata: None,
            additional_context: Default::default(),
            thread_settings: Default::default(),
        })
        .await?;
    let mut warnings = Vec::new();
    while let EventMsg::Warning(warning) = wait_for_event(&test.codex, |event| {
        matches!(event, EventMsg::Warning(_) | EventMsg::TurnComplete(_))
    })
    .await
    {
        warnings.push(warning.message);
    }
    let output = captured
        .requests()
        .iter()
        .find_map(|request| request.function_call_output_text(CALL_ID))
        .unwrap_or_default();
    let git = test.cwd_path().join(".git");
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
        hook_written: git.join("hooks/pre-commit").exists(),
        config_changed: std::fs::read_to_string(git.join("config"))?.contains("injected1139"),
        denials,
    })
}

fn level_warnings(outcome: &Outcome) -> Vec<&String> {
    outcome
        .warnings
        .iter()
        .filter(|warning| warning.starts_with("Security level"))
        .collect()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_1139_moderate_git_folder_writes_name_the_level() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    skip_if_sandbox!(Ok(()));
    let mut wrong = Vec::new();
    for (command, path) in [
        (Command::Hook, ".git/hooks/pre-commit"),
        (Command::Config, ".git/config"),
    ] {
        let moderate = run(SecurityLevel::Moderate, command).await?;
        let warnings = level_warnings(&moderate);
        let explained = warnings.len() == 1
            && warnings[0].starts_with(&format!(
                "Security level Moderate blocked writing `{path}`: under Moderate"
            ))
            && warnings[0]
                .contains("keeps the project's .git folder (git hooks and config) read-only")
            && warnings[0].contains("run the command in their own terminal")
            && warnings[0].contains("switch to Permissive in /security")
            && moderate.output.contains(warnings[0].as_str());
        let listed = vec![(
            "writing the project's .git folder (git hooks and config)",
            "denied by the sandbox under Moderate, even when you approve",
        )];
        if !explained || moderate.hook_written || moderate.config_changed {
            wrong.push(format!(
                "Moderate {command:?}: {:?} {}",
                moderate.warnings, moderate.output
            ));
        }
        if moderate.denials != listed {
            wrong.push(format!(
                "Moderate {command:?} denials: {:?}",
                moderate.denials
            ));
        }

        let permissive = run(SecurityLevel::Permissive, command).await?;
        if !level_warnings(&permissive).is_empty() || permissive.output.contains("Security level") {
            wrong.push(format!(
                "Permissive {command:?}: {:?} {}",
                permissive.warnings, permissive.output
            ));
        }
    }
    assert_eq!(wrong, Vec::<String>::new());
    Ok(())
}
