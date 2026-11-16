//! Corbanu's catalog contract is separate from the pinned upstream scenario catalog.

use anyhow::Result;
use codex_models_manager::bundled_models_response;
use codex_protocol::models::PermissionProfile;
use codex_protocol::openai_models::ReasoningEffort;
use core_test_support::responses::ev_completed;
use core_test_support::responses::mount_sse_once;
use core_test_support::responses::sse;
use core_test_support::responses::start_mock_server;
use core_test_support::skip_if_no_network;
use core_test_support::test_codex::test_codex;
use pretty_assertions::assert_eq;

#[test_case::test_case(None, "medium"; "catalog_default")]
#[test_case::test_case(Some(ReasoningEffort::High), "high"; "explicit_effort")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn corbanu_astra_catalog_controls_request_defaults_and_tools(
    effort: Option<ReasoningEffort>,
    expected_effort: &str,
) -> Result<()> {
    skip_if_no_network!(Ok(()));
    let server = start_mock_server().await;
    let test = test_codex()
        .with_model("gpt-6-astra")
        .with_config(move |config| {
            config.model_catalog = Some(bundled_models_response().expect("Corbanu catalog"));
            config.model_reasoning_effort = effort;
            config
                .permissions
                .set_permission_profile(PermissionProfile::workspace_write())
                .expect("writable fixture");
        })
        .build_with_auto_env(&server)
        .await?;
    let mock = mount_sse_once(&server, sse(vec![ev_completed("done")])).await;
    test.submit_text_turn("report the session configuration")
        .await?;
    let body = mock.single_request().body_json();
    assert_eq!(body["model"], "gpt-6-astra");
    assert_eq!(body["reasoning"]["effort"], expected_effort);
    let encoded = serde_json::to_string(&body)?;
    assert!(
        encoded.contains("Corbanu"),
        "Corbanu instructions must reach the model"
    );
    let tools = body["input"]
        .as_array()
        .expect("Responses Lite input")
        .iter()
        .find(|item| item["type"] == "additional_tools")
        .expect("additional tool definitions");
    let exec = core_test_support::responses::namespace_child_tool(tools, "functions", "exec")
        .expect("code-mode exec");
    assert!(
        exec["description"]
            .as_str()
            .expect("exec description")
            .contains("structured_edit"),
        "Corbanu edit tools remain available"
    );
    assert!(
        core_test_support::responses::namespace_child_tool(
            tools,
            "functions",
            "spawn_agent_plaintext"
        )
        .is_some(),
        "Corbanu runtime selection remains available"
    );
    assert!(
        core_test_support::responses::namespace_child_tool(
            tools,
            "functions",
            "request_user_input_async"
        )
        .is_none()
    );
    assert!(
        !tools["tools"]
            .as_array()
            .expect("tool namespaces")
            .iter()
            .any(|tool| tool["name"] == "clock"),
        "the catalog does not opt into clock tools"
    );
    assert_eq!(
        body.get("tools"),
        None,
        "Astra keeps Responses Lite tool input"
    );
    assert!(
        body["input"]
            .as_array()
            .is_some_and(|items| { items.iter().any(|item| item["type"] == "additional_tools") })
    );
    Ok(())
}
