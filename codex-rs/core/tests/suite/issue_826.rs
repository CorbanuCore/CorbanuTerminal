//! #826 (Travis, 2026-10-15): under Moderate and Aggressive (also where only
//! the launch enforces Aggressive) agent commands may not create user
//! namespaces, through the real Linux sandbox: `unshare -Ur` can no longer
//! open the vault store or the Moderate marker that bubblewrap hides.
//! Permissive is unchanged, also after untrusted content.
#![cfg(target_os = "linux")]

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

const FIRST_CALL: &str = "call-826-first";
const CALL_ID: &str = "call-826";
const VAULT_CANARY: &str = "vault-canary-826";
const VAULT_OPENED: &str = "VAULT-OPENED-826";
const MARKER_READ: &str = "MARKER-READ-826";
const NAMESPACE_MADE: &str = "NAMESPACE-MADE-826";
/// What `unshare` prints when the sandbox refuses the namespace.
const REFUSED: &str = "unshare failed: Operation not permitted";

#[derive(Debug, PartialEq, Eq)]
struct Outcome {
    /// The command listed the vault store through a namespace of its own.
    vault_listed: bool,
    /// Or the Moderate marker.
    marker_read: bool,
    /// `unshare -Ur true` succeeded.
    namespace_made: bool,
    /// `unshare` was refused with `EPERM`.
    refused: bool,
    /// The sandbox marked the command's environment as denying namespaces.
    marked: bool,
}

async fn run(
    level: SecurityLevel,
    aggressive_launch: bool,
    untrusted_first: bool,
) -> anyhow::Result<(Outcome, String)> {
    let home = Arc::new(TempDir::new()?);
    let home_path = home.path().canonicalize()?;
    std::fs::create_dir_all(home_path.join("secrets"))?;
    let vault = home_path.join("secrets/probe.txt");
    std::fs::write(&vault, VAULT_CANARY)?;
    let marker = home_path.join(codex_security_level::nested::MODERATE_MARKER);
    // The placeholders bubblewrap hides the vault store and the marker with
    // are what nested-launch detection probes; root in a namespace of its own
    // could open them (their content stays hidden).
    let script = format!(
        "echo marker=${{CODEX_SANDBOX_USER_NAMESPACES_DENIED:-unset}}; unshare -Ur ls -a '{}' && echo {VAULT_OPENED}; unshare -Ur cat '{}' && echo {MARKER_READ}; unshare -Ur true && echo {NAMESPACE_MADE}; true",
        home_path.join("secrets").display(),
        marker.display()
    );
    let mut responses = Vec::new();
    if untrusted_first {
        responses.push(sse(vec![
            ev_response_created(FIRST_CALL),
            ev_function_call(
                FIRST_CALL,
                "shell_command",
                &json!({ "command": "echo first" }).to_string(),
            ),
            ev_completed(FIRST_CALL),
        ]));
    }
    responses.push(sse(vec![
        ev_response_created(CALL_ID),
        ev_function_call(
            CALL_ID,
            "shell_command",
            &json!({ "command": script }).to_string(),
        ),
        ev_completed(CALL_ID),
    ]));
    responses.push(sse(vec![
        ev_response_created("done"),
        ev_assistant_message("msg-done", "done"),
        ev_completed("done"),
    ]));
    let server = start_mock_server().await;
    let captured = mount_sse_sequence(&server, responses).await;
    let origin = home_path.to_string_lossy().into_owned();
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
                text: "make a user namespace".into(),
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
    assert!(!output.contains(VAULT_CANARY), "{output}");
    Ok((
        Outcome {
            vault_listed: output.contains(VAULT_OPENED),
            marker_read: output.contains(MARKER_READ),
            namespace_made: output.contains(NAMESPACE_MADE),
            refused: output.contains(REFUSED),
            marked: output.contains("marker=1"),
        },
        output,
    ))
}

const DENIED: Outcome = Outcome {
    vault_listed: false,
    marker_read: false,
    namespace_made: false,
    refused: true,
    marked: true,
};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_826_protected_levels_deny_user_namespaces() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    skip_if_sandbox!(Ok(()));
    let mut wrong = Vec::new();
    for (level, aggressive_launch) in [
        (SecurityLevel::Moderate, false),
        (SecurityLevel::Aggressive, false),
        // Only the launch enforces Aggressive (Core's level stays lower).
        (SecurityLevel::Permissive, true),
        (SecurityLevel::Moderate, true),
    ] {
        // Boxed: the session future is large for the test thread's stack.
        let (outcome, output) = Box::pin(run(
            level,
            aggressive_launch,
            /*untrusted_first*/ false,
        ))
        .await?;
        if outcome != DENIED {
            wrong.push(format!(
                "{level:?} aggressive_launch={aggressive_launch}: {outcome:?}\n{output}"
            ));
        }
    }
    assert_eq!(wrong, Vec::<String>::new());
    Ok(())
}

/// Permissive, before and after untrusted content: the sandbox neither
/// refuses the namespace nor marks the command (where the host allows a
/// nested namespace, it is made, as before).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_826_permissive_is_unchanged() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    skip_if_sandbox!(Ok(()));
    for untrusted_first in [false, true] {
        let (outcome, output) = Box::pin(run(
            SecurityLevel::Permissive,
            /*aggressive_launch*/ false,
            untrusted_first,
        ))
        .await?;
        eprintln!("Permissive untrusted_first={untrusted_first}: {outcome:?}\n{output}");
        assert!(
            !outcome.marked && output.contains("marker=unset"),
            "untrusted_first={untrusted_first}: {output}"
        );
    }
    Ok(())
}
