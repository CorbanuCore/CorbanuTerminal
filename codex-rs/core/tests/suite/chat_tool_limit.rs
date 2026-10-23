//! #688: on a wire without namespaces (Chat Completions), the tools left out
//! by the 128-tool limit are reported to the person once per session, not
//! only in the log on every request.

use codex_core::StartThreadOptions;
use codex_features::Feature;
use codex_login::CodexAuth;
use codex_model_provider_info::ModelProviderInfo;
use codex_model_provider_info::WireApi;
use codex_protocol::dynamic_tools::DynamicToolFunctionSpec;
use codex_protocol::dynamic_tools::DynamicToolNamespaceSpec;
use codex_protocol::dynamic_tools::DynamicToolNamespaceTool;
use codex_protocol::dynamic_tools::DynamicToolSpec;
use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::Op;
use codex_protocol::user_input::UserInput;
use core_test_support::responses::chat_completions_sse;
use core_test_support::responses::mount_chat_completions_sequence;
use core_test_support::responses::start_mock_server;
use core_test_support::skip_if_no_network;
use core_test_support::test_codex::test_codex;
use core_test_support::wait_for_event;
use pretty_assertions::assert_eq;

const MODEL: &str = "glm-5.3-flash";
const NOTICE: &str =
    "Corbanu sends at most 128 tools per request to providers without tool namespaces";

fn bulk_namespace(count: usize) -> DynamicToolSpec {
    DynamicToolSpec::Namespace(DynamicToolNamespaceSpec {
        name: "bulk".to_string(),
        description: "Many tools.".to_string(),
        tools: (0..count)
            .map(|index| {
                DynamicToolNamespaceTool::Function(DynamicToolFunctionSpec {
                    name: format!("tool_{index:03}"),
                    description: format!("Bulk tool {index}."),
                    input_schema: serde_json::json!({
                        "type": "object",
                        "properties": {},
                        "additionalProperties": false,
                    }),
                    defer_loading: false,
                })
            })
            .collect(),
    })
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn tools_over_the_chat_limit_are_reported_once_per_session() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));

    let server = start_mock_server().await;
    let chat = mount_chat_completions_sequence(
        &server,
        vec![
            chat_completions_sse(MODEL, "first"),
            chat_completions_sse(MODEL, "second"),
        ],
    )
    .await;
    let endpoint = format!("{}/v1", server.uri());
    let base = test_codex()
        .with_auth(CodexAuth::from_api_key("synthetic-chat-tool-limit"))
        .with_config(move |config| {
            config.model = Some(MODEL.into());
            config.model_provider_id = "openai".into();
            config.model_provider = ModelProviderInfo {
                request_max_retries: Some(0),
                stream_max_retries: Some(0),
                wire_api: WireApi::Chat,
                supports_websockets: false,
                ..ModelProviderInfo::create_openai_provider(Some(endpoint))
            };
            config.model_catalog = Some(
                codex_models_manager::bundled_models_response().expect("bundled model catalog"),
            );
            for feature in [Feature::CodeMode, Feature::CodeModeOnly] {
                config
                    .features
                    .disable(feature)
                    .expect("direct tools for this test");
            }
        })
        .build(&server)
        .await?;
    let thread = base
        .thread_manager
        .start_thread(StartThreadOptions {
            dynamic_tools: vec![bulk_namespace(/*count*/ 200)],
            ..StartThreadOptions::new(base.config.clone())
        })
        .await?;

    let mut notices = Vec::new();
    for text in ["first", "second"] {
        thread
            .thread
            .submit(Op::UserInput {
                items: vec![UserInput::Text {
                    text: text.into(),
                    text_elements: Vec::new(),
                }],
                final_output_json_schema: None,
                responsesapi_client_metadata: None,
                additional_context: Default::default(),
                thread_settings: Default::default(),
            })
            .await?;
        loop {
            match wait_for_event(&thread.thread, |event| {
                matches!(event, EventMsg::Warning(_) | EventMsg::TurnComplete(_))
            })
            .await
            {
                EventMsg::Warning(warning) if warning.message.contains(NOTICE) => {
                    notices.push(warning.message);
                }
                EventMsg::TurnComplete(_) => break,
                _ => {}
            }
        }
    }

    assert_eq!(notices.len(), 1, "{notices:?}");
    assert!(
        notices[0].contains("`bulk__tool_") && notices[0].contains(" more."),
        "names the left-out tools: {}",
        notices[0]
    );
    let requests = chat.requests();
    assert_eq!(requests.len(), 2);
    for request in requests {
        let body = request.body_json();
        let tools = body["tools"].as_array().map_or(0, Vec::len);
        assert!(tools <= 128, "{tools} tools sent");
    }
    Ok(())
}
