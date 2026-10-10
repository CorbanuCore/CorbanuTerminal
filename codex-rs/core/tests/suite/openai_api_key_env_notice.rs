//! The visible notice for requests billed to the `OPENAI_API_KEY` fallback.

use anyhow::Result;
use codex_core::CodexThread;
use codex_login::CodexAuth;
use codex_login::OPENAI_API_KEY_ENV_FALLBACK_NOTICE;
use codex_login::OPENAI_API_KEY_ENV_VAR;
use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::Op;
use codex_protocol::protocol::ReviewRequest;
use codex_protocol::protocol::ReviewTarget;
use codex_protocol::user_input::UserInput;
use core_test_support::responses::ev_assistant_message;
use core_test_support::responses::ev_completed;
use core_test_support::responses::mount_sse_sequence;
use core_test_support::responses::sse;
use core_test_support::responses::start_mock_server;
use core_test_support::skip_if_no_network;
use core_test_support::test_codex::test_codex;
use core_test_support::wait_for_event;
use pretty_assertions::assert_eq;

#[derive(Clone, Copy)]
enum Step {
    Turn,
    Review,
    Compact,
}

fn env_fallback() -> CodexAuth {
    CodexAuth::from_api_key_env("sk-env", OPENAI_API_KEY_ENV_VAR)
}

/// Runs `steps` on one thread and returns how many notices each one emitted.
async fn notices_per_step(auth: CodexAuth, openai: bool, steps: &[Step]) -> Result<Vec<usize>> {
    let server = start_mock_server().await;
    let review_json = serde_json::json!({
        "findings": [],
        "overall_correctness": "good",
        "overall_explanation": "Looks fine.",
        "overall_confidence_score": 0.9
    })
    .to_string();
    let responses = steps
        .iter()
        .enumerate()
        .map(|(index, step)| {
            let text = match step {
                Step::Turn => "ok".to_string(),
                Step::Review => review_json.clone(),
                Step::Compact => "summary".to_string(),
            };
            sse(vec![
                ev_assistant_message(&format!("msg-{index}"), &text),
                ev_completed(&format!("resp-{index}")),
            ])
        })
        .collect();
    let requests = mount_sse_sequence(&server, responses).await;
    let test = test_codex()
        .with_auth(auth)
        .with_config(move |config| {
            if !openai {
                // A custom endpoint that doesn't use OpenAI sign-in.
                config.model_provider.name = "Local".to_string();
                config.model_provider.requires_openai_auth = false;
            }
        })
        .build(&server)
        .await?;

    let mut notices = Vec::new();
    for step in steps {
        let op = match step {
            Step::Turn => Op::UserInput {
                items: vec![UserInput::Text {
                    text: "say ok".to_string(),
                    text_elements: Vec::new(),
                }],
                final_output_json_schema: None,
                responsesapi_client_metadata: None,
                additional_context: Default::default(),
                thread_settings: Default::default(),
            },
            Step::Review => Op::Review {
                review_request: ReviewRequest {
                    target: ReviewTarget::Custom {
                        instructions: "Review my changes".to_string(),
                    },
                    user_facing_hint: None,
                },
            },
            Step::Compact => Op::Compact,
        };
        notices.push(submit_and_count_notices(&test.codex, op).await?);
    }
    assert_eq!(requests.requests().len(), steps.len());
    Ok(notices)
}

async fn submit_and_count_notices(codex: &CodexThread, op: Op) -> Result<usize> {
    codex.submit(op).await?;
    let mut notices = 0;
    wait_for_event(codex, |event| {
        if let EventMsg::Warning(warning) = event
            && warning.message == OPENAI_API_KEY_ENV_FALLBACK_NOTICE
        {
            notices += 1;
        }
        matches!(event, EventMsg::TurnComplete(_))
    })
    .await;
    Ok(notices)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn openai_api_key_env_fallback_is_announced_once_per_session() -> Result<()> {
    skip_if_no_network!(Ok(()));

    let notices = notices_per_step(
        env_fallback(),
        /*openai*/ true,
        &[Step::Turn, Step::Turn, Step::Review],
    )
    .await?;

    // The review runs in a sub-agent session; it doesn't announce again.
    assert_eq!(notices, vec![1, 0, 0]);
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn openai_api_key_env_fallback_is_announced_by_a_first_review_or_compaction() -> Result<()> {
    skip_if_no_network!(Ok(()));

    let review_first = notices_per_step(
        env_fallback(),
        /*openai*/ true,
        &[Step::Review, Step::Turn],
    )
    .await?;
    let compact_first = notices_per_step(
        env_fallback(),
        /*openai*/ true,
        &[Step::Compact, Step::Turn],
    )
    .await?;

    assert_eq!(review_first, vec![1, 0]);
    assert_eq!(compact_first, vec![1, 0]);
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn saved_or_session_credentials_are_not_announced() -> Result<()> {
    skip_if_no_network!(Ok(()));

    for auth in [
        CodexAuth::from_api_key("sk-saved"),
        CodexAuth::from_api_key_env("sk-codex", codex_login::CODEX_API_KEY_ENV_VAR),
        CodexAuth::create_dummy_chatgpt_auth_for_testing(),
    ] {
        let notices = notices_per_step(auth, /*openai*/ true, &[Step::Turn, Step::Turn]).await?;
        assert_eq!(notices, vec![0, 0]);
    }
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn openai_api_key_env_fallback_is_not_announced_for_other_providers() -> Result<()> {
    skip_if_no_network!(Ok(()));

    let notices = notices_per_step(
        env_fallback(),
        /*openai*/ false,
        &[Step::Turn, Step::Turn],
    )
    .await?;

    assert_eq!(notices, vec![0, 0]);
    Ok(())
}
