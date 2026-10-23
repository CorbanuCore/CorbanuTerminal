//! #439 (AGG-12/13): under Corbanu Terminal's Aggressive launch, a command
//! that fails for lack of network names the level to the person (a warning)
//! and the model (a note on its output), and the permissions instructions
//! say web search is off. Nothing changes without the level.

use anyhow::Result;
use codex_features::Feature;
use codex_protocol::models::PermissionProfile;
use codex_protocol::protocol::AskForApproval;
use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::Op;
use codex_protocol::user_input::UserInput;
use core_test_support::responses::ev_assistant_message;
use core_test_support::responses::ev_completed;
use core_test_support::responses::ev_function_call;
use core_test_support::responses::ev_response_created;
use core_test_support::responses::mount_sse_once;
use core_test_support::responses::sse;
use core_test_support::responses::start_mock_server;
use core_test_support::test_codex::local_selections;
use core_test_support::test_codex::test_codex;
use core_test_support::test_codex::turn_permission_fields;
use core_test_support::wait_for_event;
use serde_json::Value;
use serde_json::json;

const CALL_ID: &str = "sec-439-no-network";

struct Outcome {
    output: String,
    warnings: Vec<String>,
    developer: Vec<String>,
}

async fn run(aggressive: bool) -> Result<Outcome> {
    let server = start_mock_server().await;
    let mut builder = test_codex().with_config(move |config| {
        config
            .features
            .enable(Feature::UnifiedExec)
            .expect("test config should allow feature update");
        if aggressive {
            // What Corbanu Terminal's Aggressive launch sets (see
            // `Permissions::launch_enforces_aggressive`).
            config.permissions.shell_environment_policy.r#set.insert(
                codex_security_level::nested::ORIGIN_ENV.to_string(),
                "/tmp/sec-439-origin".to_string(),
            );
        }
    });
    let test = builder.build(&server).await?;
    let args = json!({
        "cmd": "echo 'curl: (6) Could not resolve host: example.com' >&2; exit 6",
        "yield_time_ms": 5_000,
    });
    let first = mount_sse_once(
        &server,
        sse(vec![
            ev_response_created("resp-sec-439-1"),
            ev_function_call(CALL_ID, "exec_command", &serde_json::to_string(&args)?),
            ev_completed("resp-sec-439-1"),
        ]),
    )
    .await;
    let second = mount_sse_once(
        &server,
        sse(vec![
            ev_assistant_message("msg-sec-439", "done"),
            ev_completed("resp-sec-439-2"),
        ]),
    )
    .await;

    let session_model = test.session_configured.model.clone();
    let (sandbox_policy, permission_profile) = turn_permission_fields(
        PermissionProfile::workspace_write(),
        test.config.cwd.as_path(),
    );
    test.codex
        .submit(Op::UserInput {
            items: vec![UserInput::Text {
                text: "fetch example.com".into(),
                text_elements: Vec::new(),
            }],
            final_output_json_schema: None,
            responsesapi_client_metadata: None,
            additional_context: Default::default(),
            thread_settings: codex_protocol::protocol::ThreadSettingsOverrides {
                environments: Some(local_selections(test.config.cwd.clone())),
                approval_policy: Some(AskForApproval::Never),
                sandbox_policy: Some(sandbox_policy),
                permission_profile,
                collaboration_mode: Some(codex_protocol::config_types::CollaborationMode {
                    mode: codex_protocol::config_types::ModeKind::Default,
                    settings: codex_protocol::config_types::Settings {
                        model: session_model,
                        reasoning_effort: None,
                        developer_instructions: None,
                    },
                }),
                ..Default::default()
            },
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
    let output = second
        .single_request()
        .function_call_output(CALL_ID)
        .get("output")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    Ok(Outcome {
        output,
        warnings,
        developer: first.single_request().message_input_texts("developer"),
    })
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sec_439_no_network_names_the_level_only_under_aggressive() -> Result<()> {
    let aggressive = run(/*aggressive*/ true).await?;
    assert!(
        aggressive
            .output
            .contains("looks blocked by security level Aggressive"),
        "output: {}",
        aggressive.output
    );
    assert!(
        aggressive
            .warnings
            .iter()
            .any(|warning| warning.contains("looks blocked by security level Aggressive")),
        "warnings: {:?}",
        aggressive.warnings
    );
    assert!(
        aggressive
            .developer
            .iter()
            .any(|message| message.contains("web search is off")
                && message.contains("security level Aggressive")),
        "developer: {:?}",
        aggressive.developer
    );

    let permissive = run(/*aggressive*/ false).await?;
    assert!(
        permissive.output.contains("Could not resolve host")
            && !permissive.output.contains("Aggressive"),
        "output: {}",
        permissive.output
    );
    assert!(
        permissive
            .warnings
            .iter()
            .all(|warning| !warning.contains("Aggressive")),
        "warnings: {:?}",
        permissive.warnings
    );
    assert!(
        permissive
            .developer
            .iter()
            .all(|message| !message.contains("security level Aggressive")),
        "developer: {:?}",
        permissive.developer
    );
    Ok(())
}
