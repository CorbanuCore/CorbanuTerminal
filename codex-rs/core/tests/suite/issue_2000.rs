//! #2000: a Windows sandbox set up during a session applies to that session.
//! Under Moderate on Windows without one, agent commands are refused (#1976);
//! once the session's Windows sandbox is raised, as the app server does after
//! "Set up default sandbox", the next turn's command is no longer refused for
//! the missing sandbox, without a new session. Other platforms are unchanged.

use codex_core::config::Constrained;
use codex_features::Feature;
use codex_protocol::config_types::WindowsSandboxLevel;
use codex_protocol::models::PermissionProfile;
use codex_protocol::permissions::NetworkSandboxPolicy;
use codex_protocol::protocol::AskForApproval;
use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::Op;
use codex_protocol::protocol::ThreadSettingsOverrides;
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
use core_test_support::test_codex::TestCodex;
use core_test_support::test_codex::test_codex;
use core_test_support::wait_for_event;
use serde_json::json;

const RAN: &str = "issue2000-ran";
const MISSING_SANDBOX: &str = "runs agent commands only inside the default Windows sandbox";
const UNELEVATED: &str = "this Windows sandbox mode (unelevated) can't";

/// The model runs `echo` once in a turn, then finishes it.
fn command_turn(call_id: &str) -> Vec<String> {
    let arguments = json!({ "command": format!("echo {RAN}") }).to_string();
    vec![
        sse(vec![
            ev_response_created(call_id),
            ev_function_call(call_id, "shell_command", &arguments),
            ev_completed(call_id),
        ]),
        sse(vec![
            ev_response_created("done"),
            ev_assistant_message("msg-done", "done"),
            ev_completed("done"),
        ]),
    ]
}

async fn turn(test: &TestCodex) -> anyhow::Result<()> {
    test.codex
        .submit(Op::UserInput {
            items: vec![UserInput::Text {
                text: "go".into(),
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
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_2000_a_windows_sandbox_set_up_mid_session_applies_to_it() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    skip_if_sandbox!(Ok(()));
    let server = start_mock_server().await;
    let test = test_codex()
        .with_config(|config| {
            config.security_level = SecurityLevel::Moderate;
            config.permissions.approval_policy = Constrained::allow_any(AskForApproval::Never);
            // Moderate needs it to send a request at all.
            config
                .features
                .enable(Feature::SourceEnvelopes)
                .expect("test config should allow feature update");
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
    let captured = mount_sse_sequence(
        &server,
        [command_turn("call-before"), command_turn("call-after")].concat(),
    )
    .await;
    let output = |call_id: &str| {
        captured
            .requests()
            .iter()
            .find_map(|request| request.function_call_output_text(call_id))
            .unwrap_or_default()
    };

    turn(&test).await?;
    let before = output("call-before");
    if cfg!(windows) {
        assert!(before.contains(MISSING_SANDBOX), "{before}");
        assert!(!before.contains(RAN), "{before}");
    } else {
        assert!(before.contains(RAN), "{before}");
    }

    // What the app server applies after "Set up default sandbox" (here the
    // non-admin sandbox, which needs no Administrator setup).
    test.codex
        .submit(Op::ThreadSettings {
            thread_settings: ThreadSettingsOverrides {
                windows_sandbox_level: Some(WindowsSandboxLevel::RestrictedToken),
                ..Default::default()
            },
        })
        .await?;
    wait_for_event(&test.codex, |event| {
        matches!(event, EventMsg::ThreadSettingsApplied(_))
    })
    .await;

    turn(&test).await?;
    let after = output("call-after");
    assert!(!after.contains(MISSING_SANDBOX), "{after}");
    // The non-admin sandbox now decides: under Moderate it runs the command
    // or refuses the profile's protected reads (#300), not a missing sandbox.
    assert!(
        after.contains(RAN) || (cfg!(windows) && after.contains(UNELEVATED)),
        "{after}"
    );
    Ok(())
}
