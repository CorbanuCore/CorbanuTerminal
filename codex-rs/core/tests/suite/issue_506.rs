//! Issue #506 (from #443, AGG-04): under Aggressive, agent commands cannot
//! list the Corbanu home, from the moment Aggressive is confirmed in
//! `/security` and in a session started under it, and the denial is
//! explained where it happens. The home's runtime entries (here `skills`)
//! stay usable; Permissive and Moderate sessions keep listing it.
#![cfg(target_os = "macos")]

use std::sync::Arc;

use codex_core::config::Constrained;
use codex_core::security_level_change::LevelBasis;
use codex_core::security_level_change::LevelChangeKind;
use codex_core::security_level_change::Probes;
use codex_core::security_level_change::commit_human_level_change;
use codex_features::Feature;
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
use serde_json::json;
use tempfile::TempDir;

const CANARY: &str = "listing-canary-506";
const SKILL_NOTE: &str = "skill-note-506";
const EXPLANATION: &str = "Security level Aggressive blocked listing";

struct Run {
    /// What the model got back for the command.
    output: String,
    /// Warnings shown to the person.
    warnings: Vec<String>,
}

/// What an agent command runs.
#[derive(Clone, Copy)]
enum Command {
    /// List the Corbanu home.
    ListHome,
    /// List it as agents usually spell it: `"$CODEX_HOME"`.
    ListHomeVariable,
    /// List and read the home's `skills` folder (a runtime entry).
    UseSkills,
    /// #554: search the folder that holds the home (`find ~`).
    FindAbove,
    /// #554: list that folder recursively (`ls -aR ~`).
    ListAboveRecursively,
}

/// One turn, in a session started at `level`, that runs `command` first
/// (before any tool output could taint the session); with `confirm`, the
/// person confirms Aggressive in `/security` first (no restart).
async fn run(
    level: SecurityLevel,
    confirm: bool,
    unified: bool,
    command: Command,
) -> anyhow::Result<Run> {
    // The home sits in a folder of its own, which #554's commands list.
    let above_dir = TempDir::new()?;
    let home = Arc::new(TempDir::new_in(above_dir.path())?);
    let path = home.path().canonicalize()?;
    let above = above_dir.path().canonicalize()?;
    std::fs::write(path.join(CANARY), "x")?;
    std::fs::create_dir_all(path.join("skills"))?;
    std::fs::write(path.join("skills/note.md"), SKILL_NOTE)?;
    let path = path.display();
    let command = match command {
        Command::ListHome => format!("ls -a '{path}'"),
        Command::ListHomeVariable => "ls -a \"$CODEX_HOME\"".to_string(),
        Command::UseSkills => format!("ls '{path}/skills' && cat '{path}/skills/note.md'"),
        Command::FindAbove => format!("find '{}'", above.display()),
        Command::ListAboveRecursively => format!("ls -aR '{}'", above.display()),
    };
    let server = start_mock_server().await;
    let captured = mount_sse_sequence(
        &server,
        vec![
            sse(vec![
                ev_response_created("call-0"),
                if unified {
                    ev_function_call(
                        "call-0",
                        "exec_command",
                        &json!({ "cmd": command }).to_string(),
                    )
                } else {
                    ev_function_call(
                        "call-0",
                        "shell_command",
                        &json!({ "command": command }).to_string(),
                    )
                },
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
            config.security_level = level;
            config.permissions.approval_policy = Constrained::allow_any(AskForApproval::Never);
            config
                .features
                .enable(Feature::SourceEnvelopes)
                .expect("enable source envelopes");
            if unified {
                config.use_experimental_unified_exec_tool = true;
                config
                    .features
                    .enable(Feature::UnifiedExec)
                    .expect("enable unified exec");
            }
        })
        .build_with_auto_env(&server)
        .await?;
    if confirm {
        // Off the async runtime, as the TUI commits (#505: the commit waits
        // for this process's sessions to take approval for every command).
        let home = test.codex_home_path().to_path_buf();
        let thread = Some(test.session_configured.thread_id);
        let report = tokio::task::spawn_blocking(move || {
            let home = home.as_path();
            let basis = LevelBasis::read(home, level, thread);
            commit_human_level_change(
                home,
                level,
                thread,
                &basis,
                SecurityLevel::Aggressive,
                Probes::Passed,
                /*now_unix_seconds*/ 1_000,
            )
        })
        .await??;
        assert_eq!(report.kind, LevelChangeKind::Stricter);
        assert!(report.live, "Aggressive applies in this session now");
    }
    test.codex
        .submit(Op::UserInput {
            items: vec![UserInput::Text {
                text: "look at the Corbanu home".into(),
                text_elements: Vec::new(),
            }],
            final_output_json_schema: None,
            responsesapi_client_metadata: None,
            additional_context: Default::default(),
            thread_settings: Default::default(),
        })
        .await?;
    let mut warnings = Vec::new();
    // #505: once Aggressive is confirmed a person approves every command
    // that is not known read-only; approve it (the sandbox still decides).
    loop {
        match wait_for_event(&test.codex, |event| {
            matches!(
                event,
                EventMsg::TurnComplete(_) | EventMsg::Warning(_) | EventMsg::ExecApprovalRequest(_)
            )
        })
        .await
        {
            EventMsg::Warning(warning) => warnings.push(warning.message),
            EventMsg::ExecApprovalRequest(approval) => {
                test.codex
                    .submit(Op::ExecApproval {
                        id: approval.effective_approval_id(),
                        turn_id: None,
                        decision: codex_protocol::protocol::ReviewDecision::Approved,
                    })
                    .await?;
            }
            _ => break,
        }
    }
    let requests = captured.requests();
    let last = requests.last().expect("the model was asked again");
    Ok(Run {
        output: last.function_call_output_text("call-0").unwrap_or_default(),
        warnings,
    })
}

/// Listing is denied and explained; the home's runtime entries still work.
async fn assert_listing_denied(
    level: SecurityLevel,
    confirm: bool,
    unified: bool,
) -> anyhow::Result<()> {
    let context = format!("{level:?} confirm={confirm} unified={unified}");
    let listed = run(level, confirm, unified, Command::ListHome).await?;
    assert!(
        !listed.output.contains(CANARY),
        "{context}: {}",
        listed.output
    );
    assert!(
        listed.output.contains("Operation not permitted"),
        "{context}: {}",
        listed.output
    );
    let warning = listed
        .warnings
        .iter()
        .find(|warning| warning.starts_with(EXPLANATION))
        .unwrap_or_else(|| panic!("{context}: no explanation in {:?}", listed.warnings));
    assert!(warning.contains("not even once"), "{context}: {warning}");
    assert!(
        warning.contains("Permissive in /security"),
        "{context}: {warning}"
    );
    assert!(
        listed.output.contains(warning.as_str()),
        "{context}: {}",
        listed.output
    );
    // As agents usually spell it.
    let variable = run(level, confirm, unified, Command::ListHomeVariable).await?;
    assert!(
        !variable.output.contains(CANARY),
        "{context}: {}",
        variable.output
    );
    assert!(
        variable
            .warnings
            .iter()
            .any(|warning| warning.starts_with(&format!("{EXPLANATION} `$CODEX_HOME`"))),
        "{context}: {:?}",
        variable.warnings
    );
    // #554: listed from the folder above, the home is denied and explained
    // the same way.
    for command in [Command::FindAbove, Command::ListAboveRecursively] {
        let above = run(level, confirm, unified, command).await?;
        assert!(
            !above.output.contains(CANARY),
            "{context}: {}",
            above.output
        );
        assert!(
            above.output.contains("Operation not permitted"),
            "{context}: {}",
            above.output
        );
        assert!(
            above
                .warnings
                .iter()
                .any(|warning| warning.starts_with(EXPLANATION)),
            "{context}: no explanation in {:?}",
            above.warnings
        );
    }
    let skills = run(level, confirm, unified, Command::UseSkills).await?;
    assert!(
        skills.output.contains("note.md"),
        "{context}: {}",
        skills.output
    );
    assert!(
        skills.output.contains(SKILL_NOTE),
        "{context}: {}",
        skills.output
    );
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_506_aggressive_sessions_cannot_list_the_corbanu_home() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    skip_if_sandbox!(Ok(()));
    for unified in [false, true] {
        assert_listing_denied(SecurityLevel::Aggressive, /*confirm*/ false, unified).await?;
    }
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_506_listing_is_denied_as_soon_as_aggressive_is_confirmed() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    skip_if_sandbox!(Ok(()));
    assert_listing_denied(
        SecurityLevel::Permissive,
        /*confirm*/ true,
        /*unified*/ false,
    )
    .await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_506_permissive_and_moderate_keep_listing_the_home() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    skip_if_sandbox!(Ok(()));
    for level in [SecurityLevel::Permissive, SecurityLevel::Moderate] {
        for command in [Command::ListHome, Command::FindAbove] {
            let listed = run(
                level, /*confirm*/ false, /*unified*/ false, command,
            )
            .await?;
            assert!(
                listed.output.contains(CANARY),
                "{level:?}: {}",
                listed.output
            );
            assert!(
                !listed
                    .warnings
                    .iter()
                    .any(|warning| warning.starts_with(EXPLANATION)),
                "{level:?}: {:?}",
                listed.warnings
            );
        }
    }
    Ok(())
}
