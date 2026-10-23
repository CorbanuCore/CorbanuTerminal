//! #1087 (MOD-09 item 5): under Moderate or Aggressive, the image tool's
//! read of a protected file that the protected-path rules refused names the
//! level on screen (a warning) and to the model, and is listed under the
//! inspector's recent denials, as a refused command is (#442, #748).
//! Permissive is unchanged.

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

const CALL_ID: &str = "call-1087";

struct Outcome {
    /// The output the model got.
    output: String,
    /// Whether the model got the file as an image.
    attached: bool,
    warnings: Vec<String>,
    /// The inspector's recent denials of the session.
    denials: Vec<(&'static str, &'static str)>,
}

/// Runs the image tool on the Corbanu home's `auth.json` under `level`.
async fn run(level: SecurityLevel) -> anyhow::Result<(Outcome, String)> {
    let home = Arc::new(TempDir::new()?);
    let auth = home.path().canonicalize()?.join("auth.json");
    std::fs::write(&auth, "ISSUE1087_CANARY")?;
    let auth = auth.display().to_string();
    let server = start_mock_server().await;
    let captured = mount_sse_sequence(
        &server,
        vec![
            sse(vec![
                ev_response_created(CALL_ID),
                ev_function_call(CALL_ID, "view_image", &json!({ "path": auth }).to_string()),
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
    let test = test_codex()
        .with_home(home)
        .with_config(move |config| {
            config.security_level = level;
            config.permissions.approval_policy = Constrained::allow_any(AskForApproval::Never);
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
    test.codex
        .submit(Op::UserInput {
            items: vec![UserInput::Text {
                text: "look at the picture".into(),
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
        .ok_or_else(|| anyhow::anyhow!("no request after the image tool"))?;
    // An error is text; an attached image is a list of content items.
    let output = request
        .function_call_output_text(CALL_ID)
        .unwrap_or_default();
    let attached = request
        .function_call_output(CALL_ID)
        .get("output")
        .is_some_and(serde_json::Value::is_array);
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
    Ok((
        Outcome {
            output,
            attached,
            warnings,
            denials,
        },
        auth,
    ))
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_1087_a_refused_image_read_names_the_level() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    skip_if_sandbox!(Ok(()));
    let mut wrong = Vec::new();
    for (level, expected, denial) in [
        (
            SecurityLevel::Moderate,
            "Security level Moderate blocked access to",
            (
                "reading a protected file",
                "denied by the sandbox under Moderate",
            ),
        ),
        (
            SecurityLevel::Aggressive,
            "Security level Aggressive blocked access to",
            (
                "reading a protected file",
                "denied by the sandbox; never granted under Aggressive",
            ),
        ),
    ] {
        let (outcome, auth) = run(level).await?;
        let explained = outcome.warnings.len() == 1
            && outcome.warnings[0].starts_with(&format!("{expected} `{auth}`"))
            && outcome.warnings[0].contains("a person can switch to Permissive in /security")
            && outcome.output.contains(outcome.warnings[0].as_str());
        if !explained || outcome.attached || outcome.denials != vec![denial] {
            wrong.push(format!(
                "{level:?}: warnings {:?}; output {}; attached {}; denials {:?}",
                outcome.warnings, outcome.output, outcome.attached, outcome.denials
            ));
        }
    }
    let (permissive, _) = run(SecurityLevel::Permissive).await?;
    if !permissive.warnings.is_empty()
        || permissive.output.contains("Security level")
        || !permissive.denials.is_empty()
    {
        wrong.push(format!(
            "Permissive: warnings {:?}; output {}; denials {:?}",
            permissive.warnings, permissive.output, permissive.denials
        ));
    }
    assert_eq!(wrong, Vec::<String>::new());
    Ok(())
}
