//! #2222: real image edits must pass protected-read admission before reading.
use super::*;
use codex_protocol::request_user_input::RequestUserInputAnswer;
use codex_protocol::request_user_input::RequestUserInputResponse;
use codex_security_policy::SecurityLevel;
use pretty_assertions::assert_eq;
use std::collections::HashMap;
use test_case::test_case;

#[test_case(SecurityLevel::Moderate, AskForApproval::OnRequest, Some("Cancel"), true, ".env", false; "moderate_cancel")]
#[test_case(SecurityLevel::Moderate, AskForApproval::OnRequest, Some("Allow once"), true, ".env", true; "moderate_allow")]
#[test_case(SecurityLevel::Aggressive, AskForApproval::OnRequest, Some("Allow once"), true, ".env", false; "aggressive_denial_survives_approval")]
#[test_case(SecurityLevel::Moderate, AskForApproval::Never, None, true, ".env", false; "moderate_approvals_off")]
#[test_case(SecurityLevel::Aggressive, AskForApproval::Never, None, true, ".env", false; "aggressive_approvals_off")]
#[test_case(SecurityLevel::Permissive, AskForApproval::Never, None, true, ".env", true; "permissive_unchanged")]
#[test_case(SecurityLevel::Moderate, AskForApproval::Never, None, false, ".env", true; "before_taint")]
#[test_case(SecurityLevel::Moderate, AskForApproval::Never, None, true, "ordinary.png", true; "ordinary_image")]
#[test_case(SecurityLevel::Moderate, AskForApproval::Never, None, true, ".env.example", true; "template_image")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[cfg_attr(windows, serial_test::serial(codex_home))]
async fn issue_2222_reference_image_read(
    level: SecurityLevel,
    approval: AskForApproval,
    answer: Option<&str>,
    tainted: bool,
    filename: &str,
    uploaded: bool,
) -> Result<()> {
    skip_if_no_network!(Ok(()));
    let server = responses::start_mock_server().await;
    let edit = Mock::given(method("POST"))
        .and(path("/v1/images/edits"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "created": 1,
            "data": [{"b64_json": TINY_PNG_BASE64}],
        })))
        .expect(u64::from(uploaded))
        .mount_as_scoped(&server)
        .await;
    let auth = CodexAuth::create_dummy_chatgpt_auth_for_testing();
    let extensions = image_generation_extensions(&auth, |_config| None);
    let test = test_codex()
        .with_auth(auth)
        .with_extensions(extensions)
        .with_session_cwd_as_workspace_root()
        .with_model_info_override("gpt-5.4", |model_info| {
            model_info.use_responses_lite = true;
            model_info.input_modalities = vec![InputModality::Text, InputModality::Image];
        })
        .with_config(move |config| {
            config.security_level = level;
            config
                .features
                .enable(Feature::SourceEnvelopes)
                .expect("envelopes");
            config.permissions.approval_policy = Constrained::allow_any(approval);
            let (mut fs, network) = PermissionProfile::workspace_write().to_runtime_permissions();
            if level == SecurityLevel::Aggressive {
                fs.entries.extend(
                    codex_protocol::permissions::credential_file_globs(
                        &config.cwd.to_string_lossy(),
                    )
                    .into_iter()
                    .map(|pattern| {
                        FileSystemSandboxEntry::new(
                            FileSystemPath::GlobPattern {
                                pattern,
                                scan: None,
                            },
                            FileSystemAccessMode::Deny,
                        )
                    }),
                );
            }
            config
                .permissions
                .set_permission_profile(PermissionProfile::from_runtime_permissions(&fs, network))
                .expect("workspace permissions");
            core_test_support::use_elevated_windows_sandbox(config);
        })
        // Exercise the local file sandbox, including in the remote test lane.
        .build(&server)
        .await?;
    let image_path = test.config.cwd.join(filename);
    std::fs::write(&image_path, TINY_PNG_BYTES)?;
    let ordinary = test.config.cwd.join("first.png");
    std::fs::write(&ordinary, TINY_PNG_BYTES)?;
    let mut steps = Vec::new();
    if tainted {
        // A normal image read gives the model untrusted tool output without
        // running a shell; the following edit must still ask for its own read.
        steps.push(responses::sse(vec![
            responses::ev_response_created("taint"),
            responses::ev_function_call(
                "taint",
                "view_image",
                &json!({"path": ordinary}).to_string(),
            ),
            responses::ev_completed("taint"),
        ]));
    }
    steps.push(responses::sse(vec![
        responses::ev_response_created("edit"),
        responses::ev_function_call_with_namespace(
            "edit",
            "image_gen",
            "imagegen",
            &json!({
                "prompt": "Edit this image. A .env mentioned in prose is not an input path.",
                // Check every reference, including a protected second path.
                "referenced_image_paths": [ordinary, image_path],
            })
            .to_string(),
        ),
        responses::ev_completed("edit"),
    ]));
    steps.push(responses::sse(vec![
        responses::ev_response_created("done"),
        responses::ev_assistant_message("done", "done"),
        responses::ev_completed("done"),
    ]));
    let captured = responses::mount_sse_sequence(&server, steps).await;
    test.codex
        .submit(Op::UserInput {
            items: vec![UserInput::Text {
                text: "Read the image and then edit the references".into(),
                text_elements: Vec::new(),
            }],
            final_output_json_schema: None,
            responsesapi_client_metadata: None,
            additional_context: Default::default(),
            thread_settings: Default::default(),
        })
        .await?;
    let event = wait_for_event(&test.codex, |event| {
        matches!(
            event,
            EventMsg::RequestUserInput(_) | EventMsg::TurnComplete(_)
        )
    })
    .await;
    if let Some(label) = answer {
        let EventMsg::RequestUserInput(request) = event else {
            panic!("protected image reference must ask before the edit");
        };
        assert!(
            edit.received_requests().await.is_empty(),
            "uploaded before approval"
        );
        assert!(
            request.questions[0]
                .question
                .contains("credential access after untrusted content")
        );
        test.codex
            .submit(Op::UserInputAnswer {
                id: request.turn_id.clone(),
                response: RequestUserInputResponse {
                    answers: HashMap::from([(
                        request.questions[0].id.clone(),
                        RequestUserInputAnswer {
                            answers: vec![label.to_string()],
                        },
                    )]),
                },
            })
            .await?;
        wait_for_event(&test.codex, |event| {
            matches!(event, EventMsg::TurnComplete(_))
        })
        .await;
    } else {
        assert!(
            matches!(event, EventMsg::TurnComplete(_)),
            "unexpected approval"
        );
    }
    let requests = edit.received_requests().await;
    assert_eq!(requests.len(), usize::from(uploaded));
    let request = captured.last_request().context("tool result")?;
    if uploaded {
        assert_eq!(
            request.function_call_output("edit")["output"],
            json!([{
                "type": "input_image", "image_url": TINY_PNG_DATA_URL,
            }])
        );
        let body: serde_json::Value = serde_json::from_slice(&requests[0].body)?;
        assert_eq!(body["images"].as_array().context("edit images")?.len(), 2);
    } else {
        let output = request
            .function_call_output_content_and_success("edit")
            .and_then(|(content, _)| content)
            .context("refusal")?;
        let expected = match answer {
            Some("Cancel") => "you declined credential access",
            Some(_) => "unable to read referenced image",
            None => "approvals are off",
        };
        assert!(output.contains(expected), "{level:?}: {output}");
        assert!(!output.contains(TINY_PNG_BASE64));
    }
    Ok(())
}
