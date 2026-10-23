//! #766: while security level Moderate is in force, Core denies agent
//! commands the Corbanu home's Moderate marker (making it when it is
//! missing), through the real sandbox (Seatbelt on macOS, bubblewrap on
//! Linux), so a `corbanu exec` they start knows Moderate applies. Not under
//! Permissive or Aggressive, and not where the launch enforces Aggressive
//! (its own nested-launch rules apply there).
#![cfg(any(target_os = "macos", target_os = "linux"))]

use std::sync::Arc;

use codex_core::config::Constrained;
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

const CALL_ID: &str = "call-766";
const READ: &str = "MARKER-READ-766";

struct Outcome {
    /// Whether the marker exists after the run.
    made: bool,
    /// Whether the command could read it.
    read: bool,
}

/// `stale`: the mode of a marker left in the home (by an earlier Moderate
/// session, 0o644, or made unreadable by someone else, 0o000).
async fn run(
    level: SecurityLevel,
    aggressive_launch: bool,
    stale: Option<u32>,
) -> anyhow::Result<Outcome> {
    let home = Arc::new(TempDir::new()?);
    let marker = home
        .path()
        .canonicalize()?
        .join(codex_security_level::nested::MODERATE_MARKER);
    if let Some(mode) = stale {
        use std::os::unix::fs::PermissionsExt;
        std::fs::create_dir_all(marker.parent().expect("runtime folder"))?;
        std::fs::write(&marker, "")?;
        std::fs::set_permissions(&marker, std::fs::Permissions::from_mode(mode))?;
    }
    let server = start_mock_server().await;
    let captured = mount_sse_sequence(
        &server,
        vec![
            sse(vec![
                ev_response_created(CALL_ID),
                ev_function_call(
                    CALL_ID,
                    "shell_command",
                    &json!({ "command": format!("cat '{}' && echo {READ}", marker.display()) })
                        .to_string(),
                ),
                ev_completed(CALL_ID),
            ]),
            sse(vec![
                ev_response_created("done"),
                ev_assistant_message("msg-done", "done"),
                ev_completed("done"),
            ]),
        ],
    )
    .await;
    let origin = home.path().to_string_lossy().into_owned();
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
            config
                .permissions
                .set_permission_profile(PermissionProfile::workspace_write_with(
                    &[],
                    NetworkSandboxPolicy::Restricted,
                    /*exclude_tmpdir_env_var*/ true,
                    /*exclude_slash_tmp*/ true,
                ))
                .expect("workspace write");
            if aggressive_launch {
                // What an Aggressive launch puts into every agent command's
                // environment (`launch_enforces_aggressive`).
                config
                    .permissions
                    .shell_environment_policy
                    .r#set
                    .insert(codex_security_level::nested::ORIGIN_ENV.to_string(), origin);
            }
        })
        // Host-sandbox security test: local executor on purpose.
        .build(&server)
        .await?;
    test.codex
        .submit(Op::UserInput {
            items: vec![UserInput::Text {
                text: "read the marker".into(),
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
    let output = captured
        .requests()
        .iter()
        .find_map(|request| request.function_call_output_text(CALL_ID))
        .unwrap_or_default();
    Ok(Outcome {
        made: marker.is_file(),
        read: output.contains(READ),
    })
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_766_only_moderate_denies_the_marker() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    skip_if_sandbox!(Ok(()));
    let mut seen = Vec::new();
    for (level, aggressive_launch, stale) in [
        (SecurityLevel::Moderate, false, None),
        (SecurityLevel::Moderate, false, Some(0o644)),
        (SecurityLevel::Moderate, true, Some(0o644)),
        (SecurityLevel::Permissive, false, Some(0o644)),
        (SecurityLevel::Aggressive, false, Some(0o644)),
        // Made readable again where the rules apply without Moderate
        // (Linux).
        (SecurityLevel::Aggressive, false, Some(0o000)),
    ] {
        // Boxed: the session future is large for the test thread's stack.
        let outcome = Box::pin(run(level, aggressive_launch, stale)).await?;
        seen.push((level, aggressive_launch, stale, outcome.made, outcome.read));
    }
    assert_eq!(
        seen,
        vec![
            // Made and denied.
            (SecurityLevel::Moderate, false, None, true, false),
            (SecurityLevel::Moderate, false, Some(0o644), true, false),
            // Left readable.
            (SecurityLevel::Moderate, true, Some(0o644), true, true),
            (SecurityLevel::Permissive, false, Some(0o644), true, true),
            (SecurityLevel::Aggressive, false, Some(0o644), true, true),
            // Linux only: on macOS such a file never passes for Moderate's.
            (
                SecurityLevel::Aggressive,
                false,
                Some(0o000),
                true,
                cfg!(target_os = "linux")
            ),
        ]
    );
    Ok(())
}
