//! Issue #443 (milestone "Aggressive ships", AGG-04): confirming Aggressive in
//! `/security` raises Core's level in the running session, and from that
//! moment agent commands cannot use the Corbanu vault, before any restart
//! loads the Aggressive rules file.
#![cfg(unix)]

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
use core_test_support::skip_if_wine_exec;
use core_test_support::test_codex::test_codex;
use core_test_support::wait_for_event;
use serde_json::json;

const REFUSAL: &str = "Security level Aggressive: agents cannot use the Corbanu vault.";
const RAN: &str = "fake corbanu ran";

/// Stand-ins for `corbanu` and `codex`, called by absolute path (the vault
/// rule matches any path), so no real vault or OS keychain is ever reached.
fn fake_bin() -> anyhow::Result<tempfile::TempDir> {
    use std::os::unix::fs::PermissionsExt;
    let bin = tempfile::TempDir::new()?;
    for name in ["corbanu", "codex"] {
        let path = bin.path().join(name);
        std::fs::write(&path, format!("#!/bin/sh\necho \"{RAN}: $*\"\n"))?;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755))?;
    }
    Ok(bin)
}

/// A Permissive session (approvals off, so nothing waits for a person); with
/// `confirm`, the person confirms Aggressive before the turn, as `/security`
/// does. Each vault command and the output the model got back for it.
/// With `unified`, the model uses `exec_command` (unified exec) instead of
/// `shell_command`.
async fn vault_commands_after(
    confirm: bool,
    unified: bool,
) -> anyhow::Result<Vec<(String, String)>> {
    let bin = fake_bin()?;
    let bin = bin.path().display();
    let commands = [
        format!("{bin}/corbanu vault list"),
        format!("bash -c '{bin}/corbanu vault get provider/example'"),
        format!("echo ok && {bin}/codex vault list"),
    ];
    let server = start_mock_server().await;
    let mut steps = Vec::new();
    for (index, command) in commands.iter().enumerate() {
        let id = format!("call-{index}");
        steps.push(sse(vec![
            ev_response_created(&id),
            if unified {
                ev_function_call(&id, "exec_command", &json!({ "cmd": command }).to_string())
            } else {
                ev_function_call(
                    &id,
                    "shell_command",
                    &json!({ "command": command }).to_string(),
                )
            },
            ev_completed(&id),
        ]));
    }
    steps.push(sse(vec![
        ev_response_created("done"),
        ev_assistant_message("msg-done", "done"),
        ev_completed("done"),
    ]));
    let captured = mount_sse_sequence(&server, steps).await;
    let test = test_codex()
        .with_model("test-gpt-5.1-codex")
        .with_config(move |config| {
            config.security_level = SecurityLevel::Permissive;
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
        // Host-sandbox security test: local executor on purpose, also in the
        // remote lane (#708).
        .build(&server)
        .await?;
    if confirm {
        // Off the async runtime, as the TUI commits (#505: the commit waits
        // for this process's sessions to take approval for every command).
        let home = test.codex_home_path().to_path_buf();
        let thread = Some(test.session_configured.thread_id);
        let report = tokio::task::spawn_blocking(move || {
            let home = home.as_path();
            let basis = LevelBasis::read(home, SecurityLevel::Permissive, thread);
            assert!(basis.live, "the session's policy tree runs in this process");
            commit_human_level_change(
                home,
                SecurityLevel::Permissive,
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
                text: "run the vault commands".into(),
                text_elements: Vec::new(),
            }],
            final_output_json_schema: None,
            responsesapi_client_metadata: None,
            additional_context: Default::default(),
            thread_settings: Default::default(),
        })
        .await?;
    wait_for_event(&test.codex, |event| {
        matches!(event, EventMsg::TurnComplete(_))
    })
    .await;
    let requests = captured.requests();
    let last = requests.last().expect("the model was asked again");
    Ok(commands
        .into_iter()
        .enumerate()
        .map(|(index, command)| {
            let output = last
                .function_call_output_text(&format!("call-{index}"))
                .unwrap_or_default();
            (command, output)
        })
        .collect())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_443_vault_commands_are_refused_as_soon_as_aggressive_is_confirmed()
-> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    skip_if_wine_exec!(
        Ok(()),
        "this security test runs host shell commands; Wine-exec routes them to a Windows executor with a non-native cwd"
    );
    for unified in [false, true] {
        for (command, output) in vault_commands_after(/*confirm*/ true, unified).await? {
            assert!(
                output.contains(REFUSAL),
                "unified={unified} {command}: {output}"
            );
            assert!(
                !output.contains(RAN),
                "unified={unified} {command}: {output}"
            );
        }
    }
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_443_permissive_sessions_keep_their_vault_commands() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    skip_if_wine_exec!(
        Ok(()),
        "this security test runs host shell commands; Wine-exec routes them to a Windows executor with a non-native cwd"
    );
    for unified in [false, true] {
        for (command, output) in vault_commands_after(/*confirm*/ false, unified).await? {
            assert!(
                !output.contains(REFUSAL),
                "unified={unified} {command}: {output}"
            );
            assert!(
                output.contains(RAN),
                "unified={unified} {command}: {output}"
            );
        }
    }
    Ok(())
}
