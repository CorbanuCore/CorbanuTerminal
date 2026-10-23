//! #515 / #227: MCP tools reach Chat Completions models under flat names, and
//! a call to a flat name runs the namespaced MCP tool through the same
//! approval prompt as on the Responses wire.

use std::collections::HashMap;
use std::time::Duration;

use codex_config::types::AppToolApproval;
use codex_config::types::McpServerConfig;
use codex_config::types::McpServerTransportConfig;
use codex_core::config::Constrained;
use codex_features::Feature;
use codex_login::CodexAuth;
use codex_model_provider_info::ModelProviderInfo;
use codex_model_provider_info::WireApi;
use codex_protocol::approvals::ElicitationRequest;
use codex_protocol::protocol::AskForApproval;
use codex_protocol::protocol::ElicitationAction;
use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::Op;
use codex_protocol::user_input::UserInput;
use core_test_support::responses::chat_completions_sse;
use core_test_support::responses::chat_completions_tool_call_sse;
use core_test_support::responses::mount_chat_completions_sequence;
use core_test_support::responses::start_mock_server;
use core_test_support::skip_if_no_network;
use core_test_support::stdio_server_bin;
use core_test_support::test_codex::test_codex;
use core_test_support::wait_for_event;
use core_test_support::wait_for_mcp_server;
use pretty_assertions::assert_eq;
use serde_json::Value;
use serde_json::json;

// A direct-tool chat model, like the Z.AI route in #515.
const MODEL: &str = "glm-5.3-flash";
const CALL_ID: &str = "call-mcp";
const FLAT_NAME: &str = "mcp__rmcp__echo_tool";

fn rmcp_server(approval: AppToolApproval) -> anyhow::Result<McpServerConfig> {
    Ok(McpServerConfig {
        auth: Default::default(),
        transport: McpServerTransportConfig::Stdio {
            command: stdio_server_bin()?,
            args: Vec::new(),
            env: None,
            env_vars: Vec::new(),
            cwd: None,
        },
        environment_id: codex_config::DEFAULT_MCP_SERVER_ENVIRONMENT_ID.to_string(),
        enabled: true,
        required: false,
        supports_parallel_tool_calls: false,
        disabled_reason: None,
        startup_timeout_sec: Some(Duration::from_secs(10)),
        tool_timeout_sec: None,
        default_tools_approval_mode: Some(approval),
        enabled_tools: None,
        disabled_tools: None,
        scopes: None,
        oauth: None,
        oauth_resource: None,
        tools: HashMap::new(),
    })
}

fn function_tool_names(body: &Value) -> Vec<String> {
    body["tools"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|tool| tool["function"]["name"].as_str().map(str::to_string))
        .collect()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn mcp_tool_call_round_trips_on_chat_completions() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));

    let server = start_mock_server().await;
    let chat = mount_chat_completions_sequence(
        &server,
        vec![
            chat_completions_tool_call_sse(
                CALL_ID,
                // The sanitized flat name of the raw `echo-tool`.
                FLAT_NAME,
                &json!({ "message": "ping" }).to_string(),
            ),
            chat_completions_sse(MODEL, "done"),
        ],
    )
    .await;
    let endpoint = format!("{}/v1", server.uri());
    let server_config = rmcp_server(AppToolApproval::Prompt)?;
    let test = test_codex()
        .with_auth(CodexAuth::from_api_key("synthetic-chat-mcp"))
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
            config.permissions.approval_policy = Constrained::allow_any(AskForApproval::OnRequest);
            for feature in [Feature::CodeMode, Feature::CodeModeOnly] {
                config
                    .features
                    .disable(feature)
                    .expect("direct tools for this test");
            }
            let mut servers = config.mcp_servers.get().clone();
            servers.insert("rmcp".to_string(), server_config);
            config
                .mcp_servers
                .set(servers)
                .expect("test MCP servers accept any configuration");
        })
        .build(&server)
        .await?;
    wait_for_mcp_server(&test.codex, "rmcp").await?;

    test.codex
        .submit(Op::UserInput {
            items: vec![UserInput::Text {
                text: "echo ping with the rmcp tool".into(),
                text_elements: Vec::new(),
            }],
            final_output_json_schema: None,
            responsesapi_client_metadata: None,
            additional_context: Default::default(),
            thread_settings: Default::default(),
        })
        .await?;

    // The flat call runs the namespaced tool, so the MCP approval prompt asks first.
    let EventMsg::ElicitationRequest(request) = wait_for_event(&test.codex, |event| {
        matches!(
            event,
            EventMsg::ElicitationRequest(_)
                | EventMsg::RequestUserInput(_)
                | EventMsg::TurnComplete(_)
        )
    })
    .await
    else {
        let outputs: Vec<Value> = chat
            .requests()
            .last()
            .and_then(|request| request.body_json()["messages"].as_array().cloned())
            .unwrap_or_default()
            .into_iter()
            .filter(|message| message["role"] == "tool")
            .collect();
        panic!("expected the MCP approval prompt before the turn completes: {outputs:?}");
    };
    assert_eq!(request.server_name, "rmcp");
    let ElicitationRequest::Form { message, .. } = &request.request else {
        panic!("expected a form approval, got {:?}", request.request);
    };
    assert!(message.contains("echo-tool"), "{message}");
    test.codex
        .submit(Op::ResolveElicitation {
            server_name: request.server_name,
            request_id: request.id,
            decision: ElicitationAction::Accept,
            content: None,
            meta: None,
        })
        .await?;

    let EventMsg::McpToolCallEnd(end) = wait_for_event(&test.codex, |event| {
        matches!(event, EventMsg::McpToolCallEnd(_))
    })
    .await
    else {
        unreachable!("event guard guarantees McpToolCallEnd");
    };
    assert_eq!(end.call_id, CALL_ID);
    assert_eq!(end.invocation.server, "rmcp");
    assert_eq!(end.invocation.tool, "echo-tool");
    assert_eq!(end.invocation.arguments, Some(json!({ "message": "ping" })));
    assert!(end.result.is_ok(), "{:?}", end.result);
    wait_for_event(&test.codex, |event| {
        matches!(event, EventMsg::TurnComplete(_))
    })
    .await;

    let requests = chat.requests();
    assert_eq!(requests.len(), 2);
    let first = requests[0].body_json();
    let tool_names = function_tool_names(&first);
    for expected in ["mcp__rmcp__echo", FLAT_NAME] {
        assert!(
            tool_names.iter().any(|name| name == expected),
            "missing {expected} in {tool_names:?}"
        );
    }
    assert!(
        first["tools"]
            .as_array()
            .into_iter()
            .flatten()
            .all(|tool| tool["type"] == "function" || tool["type"] == "web_search"),
        "only function tools on the chat wire: {first}"
    );

    let second = requests[1].body_json();
    let messages = second["messages"].as_array().expect("chat messages");
    let replayed_call = messages
        .iter()
        .flat_map(|message| message["tool_calls"].as_array().into_iter().flatten())
        .find(|call| call["id"] == CALL_ID)
        .expect("the tool call is replayed");
    assert_eq!(replayed_call["function"]["name"], FLAT_NAME);
    let tool_result = messages
        .iter()
        .find(|message| message["role"] == "tool" && message["tool_call_id"] == CALL_ID)
        .expect("the MCP result is returned to the model");
    assert!(
        tool_result["content"].to_string().contains("ping"),
        "{tool_result}"
    );
    Ok(())
}
