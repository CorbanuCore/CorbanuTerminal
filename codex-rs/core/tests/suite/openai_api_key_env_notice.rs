//! The visible notice for requests billed to the `OPENAI_API_KEY` fallback.

use anyhow::Result;
use codex_core::CodexThread;
use codex_login::CodexAuth;
use codex_login::OPENAI_API_KEY_ENV_FALLBACK_NOTICE;
use codex_login::OPENAI_API_KEY_ENV_VAR;
use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::Op;
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

/// Runs two turns and returns the warnings each one emitted.
async fn warnings_for_two_turns(auth: CodexAuth, openai: bool) -> Result<Vec<Vec<String>>> {
    let server = start_mock_server().await;
    let responses = (1..=2)
        .map(|turn| {
            sse(vec![
                ev_assistant_message(&format!("msg-{turn}"), "ok"),
                ev_completed(&format!("resp-{turn}")),
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

    let mut warnings = Vec::new();
    for prompt in ["first", "second"] {
        warnings.push(submit_and_collect_warnings(&test.codex, prompt).await?);
    }
    assert_eq!(requests.requests().len(), 2);
    Ok(warnings)
}

async fn submit_and_collect_warnings(codex: &CodexThread, text: &str) -> Result<Vec<String>> {
    codex
        .submit(Op::UserInput {
            items: vec![UserInput::Text {
                text: text.to_string(),
                text_elements: Vec::new(),
            }],
            final_output_json_schema: None,
            responsesapi_client_metadata: None,
            additional_context: Default::default(),
            thread_settings: Default::default(),
        })
        .await?;
    let mut warnings = Vec::new();
    wait_for_event(codex, |event| {
        if let EventMsg::Warning(warning) = event {
            warnings.push(warning.message.clone());
        }
        matches!(event, EventMsg::TurnComplete(_))
    })
    .await;
    Ok(warnings)
}

fn notice_count(warnings: &[String]) -> usize {
    warnings
        .iter()
        .filter(|message| message.as_str() == OPENAI_API_KEY_ENV_FALLBACK_NOTICE)
        .count()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn openai_api_key_env_fallback_is_announced_once_per_session() -> Result<()> {
    skip_if_no_network!(Ok(()));

    let warnings = warnings_for_two_turns(
        CodexAuth::from_api_key_env("sk-env", OPENAI_API_KEY_ENV_VAR),
        /*openai*/ true,
    )
    .await?;

    assert_eq!(notice_count(&warnings[0]), 1, "{warnings:?}");
    assert_eq!(notice_count(&warnings[1]), 0, "{warnings:?}");
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
        let warnings = warnings_for_two_turns(auth, /*openai*/ true).await?;
        assert_eq!(notice_count(&warnings.concat()), 0, "{warnings:?}");
    }
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn openai_api_key_env_fallback_is_not_announced_for_other_providers() -> Result<()> {
    skip_if_no_network!(Ok(()));

    let warnings = warnings_for_two_turns(
        CodexAuth::from_api_key_env("sk-env", OPENAI_API_KEY_ENV_VAR),
        /*openai*/ false,
    )
    .await?;

    assert_eq!(notice_count(&warnings.concat()), 0, "{warnings:?}");
    Ok(())
}
