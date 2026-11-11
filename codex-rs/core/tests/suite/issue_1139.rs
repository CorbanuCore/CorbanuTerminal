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

#[cfg(not(windows))]
const HOOK_WRITE: &str = "printf 'echo hi\\n' > .git/hooks/pre-commit";
/// PowerShell, which runs `shell_command` on Windows, has no `printf` (#1967).
#[cfg(windows)]
const HOOK_WRITE: &str = "echo 'echo hi' > .git/hooks/pre-commit";

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
    /// Whether a warning names the hook file, by whatever spelling of its
    /// path (checked while the project folder exists).
    names_hook: bool,
}

async fn run_1139(level: SecurityLevel, command: Command) -> anyhow::Result<Outcome> {
    let (name, arguments) = match command {
        Command::Hook => ("shell_command", json!({ "command": HOOK_WRITE })),
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
            // #1967: the sandbox that keeps `.git` read-only on Windows.
            core_test_support::use_elevated_windows_sandbox(config);
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
        hook_written: git.join("hooks/pre-commit").exists(),
        config_changed: std::fs::read_to_string(git.join("config"))?.contains("injected1139"),
        names_hook: warnings
            .iter()
            .any(|warning| names_file(warning, &git.join("hooks").join("pre-commit"))),
        warnings,
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

/// Whether `warning` is Moderate's "blocked writing" warning for `file`,
/// whose folder must exist, by whatever spelling of its path.
fn names_file(warning: &str, file: &std::path::Path) -> bool {
    let Some(rest) = warning.strip_prefix("Security level Moderate blocked writing `") else {
        return false;
    };
    let Some((named, tail)) = rest.split_once('`') else {
        return false;
    };
    let named = std::path::Path::new(named);
    let same_folder = match (named.parent(), file.parent()) {
        (Some(a), Some(b)) => match (std::fs::canonicalize(a), std::fs::canonicalize(b)) {
            (Ok(a), Ok(b)) => a == b,
            _ => false,
        },
        _ => false,
    };
    same_folder && named.file_name() == file.file_name() && tail.starts_with(": under Moderate")
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial_test::serial(codex_home)]
async fn issue_1139_moderate_git_folder_writes_name_the_level() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    skip_if_sandbox!(Ok(()));
    let mut wrong = Vec::new();
    for (command, path) in [
        (Command::Hook, ".git/hooks/pre-commit"),
        (Command::Config, ".git/config"),
    ] {
        let moderate = run_1139(SecurityLevel::Moderate, command).await?;
        let warnings = level_warnings(&moderate);
        let named = warnings.len() == 1
            && match command {
                // PowerShell reports the hook it could not write by its full
                // path, which the warning names; compare it as a file, as
                // TEMP may be spelled with short names (#1967).
                Command::Hook if cfg!(windows) => moderate.names_hook,
                _ => warnings[0].starts_with(&format!(
                    "Security level Moderate blocked writing `{path}`: under Moderate"
                )),
            };
        let explained = named
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

        let permissive = run_1139(SecurityLevel::Permissive, command).await?;
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
