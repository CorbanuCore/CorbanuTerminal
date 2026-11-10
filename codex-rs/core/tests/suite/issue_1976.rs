//! #1976 (D491, option (a)): on Windows without the Windows sandbox,
//! Moderate and Aggressive refuse agent commands (read-only ones included)
//! and file-tool access, naming the level and how to set the sandbox up.
//! Permissive is unchanged, and so is every level on other platforms.

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
use image::DynamicImage;
use image::ImageBuffer;
use image::Rgba;
use pretty_assertions::assert_eq;
use serde_json::Value;
use serde_json::json;

const CALL_ID: &str = "call-1976";
const RAN: &str = "issue1976-ran";

struct Outcome {
    output: String,
    /// Whether the model got an image.
    attached: bool,
    warnings: Vec<String>,
    /// What the security inspector lists for the session.
    denials: Vec<&'static str>,
}

#[derive(Clone, Copy)]
enum Call {
    /// `echo`, which Windows runs unsandboxed as read-only today.
    ReadOnlyCommand,
    /// The image tool on a plain image in the workspace.
    ViewImage,
    /// Typing into a running command (`write_stdin`); the typing is judged
    /// before the process is looked up, so no process needs to run.
    TypeIntoCommand,
}

fn png() -> anyhow::Result<Vec<u8>> {
    let image = ImageBuffer::from_pixel(2, 2, Rgba([0u8, 128, 255, 255]));
    let mut bytes = std::io::Cursor::new(Vec::new());
    DynamicImage::ImageRgba8(image).write_to(&mut bytes, image::ImageFormat::Png)?;
    Ok(bytes.into_inner())
}

async fn run(level: SecurityLevel, call: Call) -> anyhow::Result<Outcome> {
    let server = start_mock_server().await;
    let test = test_codex()
        .with_config(move |config| {
            config.security_level = level;
            config.permissions.approval_policy = Constrained::allow_any(AskForApproval::Never);
            // Moderate and Aggressive need it to send a request at all.
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
            if matches!(call, Call::TypeIntoCommand) {
                config.use_experimental_unified_exec_tool = true;
                config
                    .features
                    .enable(Feature::UnifiedExec)
                    .expect("test config should allow feature update");
            }
        })
        .build(&server)
        .await?;
    let (tool, arguments): (&str, Value) = match call {
        Call::ReadOnlyCommand => ("shell_command", json!({ "command": format!("echo {RAN}") })),
        Call::ViewImage => {
            let image = test.workspace_path("picture.png");
            std::fs::write(&image, png()?)?;
            ("view_image", json!({ "path": image.display().to_string() }))
        }
        Call::TypeIntoCommand => (
            "write_stdin",
            json!({ "session_id": 1000, "chars": format!("echo {RAN}\n"), "yield_time_ms": 200 }),
        ),
    };
    let captured = mount_sse_sequence(
        &server,
        vec![
            sse(vec![
                ev_response_created(CALL_ID),
                ev_function_call(CALL_ID, tool, &arguments.to_string()),
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
    let mut warnings = Vec::new();
    while let EventMsg::Warning(warning) = wait_for_event(&test.codex, |event| {
        matches!(event, EventMsg::Warning(_) | EventMsg::TurnComplete(_))
    })
    .await
    {
        warnings.push(warning.message);
    }
    let request = captured
        .last_request()
        .ok_or_else(|| anyhow::anyhow!("no request after the tool call"))?;
    Ok(Outcome {
        output: request
            .function_call_output_text(CALL_ID)
            .unwrap_or_default(),
        attached: request
            .function_call_output(CALL_ID)
            .get("output")
            .is_some_and(Value::is_array),
        warnings,
        denials: security_inspection::observe(
            test.codex_home_path(),
            level,
            Some(test.session_configured.thread_id),
            i64::MAX / 2,
        )
        .denials
        .into_iter()
        .map(|denial| denial.reason)
        .collect(),
    })
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_1976_windows_without_its_sandbox_refuses_commands_and_file_tools()
-> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    skip_if_sandbox!(Ok(()));
    let mut wrong = Vec::new();
    for (level, name) in [
        (SecurityLevel::Permissive, "Permissive"),
        (SecurityLevel::Moderate, "Moderate"),
        (SecurityLevel::Aggressive, "Aggressive"),
    ] {
        let refused = cfg!(windows) && level != SecurityLevel::Permissive;
        let command = run(level, Call::ReadOnlyCommand).await?;
        let expected = format!("Security level {name} refused this command");
        let ok = if refused {
            command.output.contains(&expected)
                && command.output.contains("Set up default sandbox")
                && !command.output.contains(RAN)
                && command.warnings.iter().any(|w| w.starts_with(&expected))
        } else {
            command.output.contains(RAN)
                && !command.output.contains("Security level")
                && command
                    .warnings
                    .iter()
                    .all(|w| !w.contains("Windows sandbox"))
        };
        if !ok {
            wrong.push(format!(
                "{name} command: output {}; warnings {:?}",
                command.output, command.warnings
            ));
        }
        let image = run(level, Call::ViewImage).await?;
        let expected = format!("Security level {name} refused this file access");
        let ok = if refused {
            !image.attached
                && image.output.contains(&expected)
                && image.output.contains("Set up default sandbox")
        } else {
            image.attached
        };
        if !ok {
            wrong.push(format!(
                "{name} image: attached {}; output {}; warnings {:?}",
                image.attached, image.output, image.warnings
            ));
        }
    }
    assert_eq!(wrong, Vec::<String>::new());
    Ok(())
}

/// #2000 follow-up: typing into a running command, refused like the command
/// itself, is listed in the security inspector like the other refusals.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_1976_refused_typing_is_listed_in_the_inspector() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    skip_if_sandbox!(Ok(()));
    const TYPING: &str = "typing into a running command without the Windows sandbox";
    let mut wrong = Vec::new();
    for (level, name) in [
        (SecurityLevel::Permissive, "Permissive"),
        (SecurityLevel::Moderate, "Moderate"),
        (SecurityLevel::Aggressive, "Aggressive"),
    ] {
        let refused = cfg!(windows) && level != SecurityLevel::Permissive;
        let typed = run(level, Call::TypeIntoCommand).await?;
        let expected = format!("Security level {name} refused this command");
        let listed = typed.denials.contains(&TYPING);
        let ok = if refused {
            typed.output.contains(&expected) && listed
        } else {
            // The typing reached the process lookup (no process 1000 runs).
            typed.output.contains("Unknown process id 1000")
                && !typed.output.contains("Security level")
                && !listed
        };
        if !ok {
            wrong.push(format!(
                "{name}: output {}; denials {:?}",
                typed.output, typed.denials
            ));
        }
    }
    assert_eq!(wrong, Vec::<String>::new());
    Ok(())
}
