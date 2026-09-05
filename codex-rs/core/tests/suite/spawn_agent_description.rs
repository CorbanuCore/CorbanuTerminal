#![cfg(not(target_os = "windows"))]
#![allow(clippy::unwrap_used)]

use anyhow::Result;
use codex_core::config::AgentRoleConfig;
use codex_features::Feature;
use codex_login::CodexAuth;
use codex_models_manager::manager::RefreshStrategy;
use codex_models_manager::manager::SharedModelsManager;
use codex_protocol::config_types::ReasoningSummary;
use codex_protocol::openai_models::ConfigShellToolType;
use codex_protocol::openai_models::ModelCapabilityTier;
use codex_protocol::openai_models::ModelInfo;
use codex_protocol::openai_models::ModelOrchestrationMetadata;
use codex_protocol::openai_models::ModelServiceTier;
use codex_protocol::openai_models::ModelVisibility;
use codex_protocol::openai_models::ModelsResponse;
use codex_protocol::openai_models::ReasoningEffort;
use codex_protocol::openai_models::ReasoningEffortPreset;
use codex_protocol::openai_models::TruncationPolicyConfig;
use codex_protocol::openai_models::default_input_modalities;
use codex_protocol::protocol::MultiAgentVersion;
use core_test_support::responses::ev_assistant_message;
use core_test_support::responses::ev_completed;
use core_test_support::responses::ev_function_call;
use core_test_support::responses::ev_response_created;
use core_test_support::responses::mount_models_once;
use core_test_support::responses::mount_sse_once;
use core_test_support::responses::mount_sse_once_match;
use core_test_support::responses::namespace_child_tool;
use core_test_support::responses::sse;
use core_test_support::responses::start_mock_server;
use core_test_support::test_codex::test_codex;
use pretty_assertions::assert_eq;
use serde_json::Value;
use serde_json::json;
use std::time::Duration;
use std::time::Instant;
use test_case::test_case;
use tokio::time::sleep;

#[test_case(false, "gpt-5.6-luna"; "Luna model-only selection")]
#[test_case(true, "gpt-5.6-luna"; "Luna exact provider and model selection")]
#[test_case(false, "gpt-6-astra"; "unpriced Astra model-only selection")]
#[test_case(true, "gpt-6-astra"; "unpriced Astra exact provider and model selection")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn plaintext_spawn_dispatches_selected_openai_child_with_legacy_model_preference(
    explicit_provider: bool,
    model: &'static str,
) -> Result<()> {
    let server = start_mock_server().await;
    let mut args = json!({"task_name": "worker", "message": "Report one fact", "fork_turns": "none", "model": model, "reasoning_effort": "medium"});
    if explicit_provider {
        args["model_provider"] = json!("openai");
    }
    mount_sse_once_match(
        &server,
        wiremock::matchers::body_partial_json(json!({"model": "gpt-5.4"})),
        sse(vec![
            ev_response_created("root-1"),
            ev_function_call("spawn-mixed", "spawn_agent_plaintext", &args.to_string()),
            ev_completed("root-1"),
        ]),
    )
    .await;
    let child = mount_sse_once_match(
        &server,
        wiremock::matchers::body_partial_json(json!({"model": model})),
        sse(vec![
            ev_response_created("child-1"),
            ev_assistant_message("child-result", "Verified child fact"),
            ev_completed("child-1"),
        ]),
    )
    .await;
    let parent = mount_sse_once_match(
        &server,
        |request: &wiremock::Request| {
            request.body_json::<Value>().ok().is_some_and(|body| {
                body["model"] == "gpt-5.4"
                    && body["input"].as_array().is_some_and(|items| {
                        items.iter().any(|item| {
                            item["type"] == "function_call_output"
                                && item["call_id"] == "spawn-mixed"
                        })
                    })
            })
        },
        sse(vec![
            ev_response_created("root-2"),
            ev_assistant_message("root-result", "Spawn inspected"),
            ev_completed("root-2"),
        ]),
    )
    .await;
    let test = test_codex()
        .with_model_info_override(model, |model| {
            model.multi_agent_version = Some(MultiAgentVersion::V1);
            model.tool_mode = None;
            model.use_responses_lite = false;
        })
        .with_model("gpt-5.4")
        .with_config(|config| {
            config
                .features
                .enable(Feature::MultiAgentV2)
                .expect("feature update");
            config
                .model_providers
                .insert("openai".to_string(), config.model_provider.clone());
            config.multi_agent_v2.expose_spawn_agent_model_overrides = true;
            config.multi_agent_v2.hide_spawn_agent_metadata = false;
        })
        .build_with_auto_env(&server)
        .await?;
    test.submit_turn("Spawn the explicitly requested child")
        .await?;
    let output: Value = serde_json::from_str(
        &parent
            .function_call_output_text("spawn-mixed")
            .expect("spawn result"),
    )?;
    assert_eq!(
        (output["model_provider"].as_str(), output["model"].as_str()),
        (Some("openai"), Some(model))
    );
    let deadline = Instant::now() + Duration::from_secs(5);
    while !child
        .requests()
        .iter()
        .any(|request| request.body_json()["model"] == model)
        && Instant::now() < deadline
    {
        sleep(Duration::from_millis(25)).await;
    }
    // Match helpers also observe requests while evaluating a non-matching route.
    let request = child
        .requests()
        .into_iter()
        .map(|request| request.body_json())
        .find(|body| body["model"] == model)
        .expect("selected child request");
    assert!(
        namespace_child_tool(&request, MULTI_AGENT_V2_NAMESPACE, SPAWN_AGENT_TOOL_NAME).is_some(),
        "selected child must retain the V2 runtime"
    );
    Ok(())
}

const MULTI_AGENT_V1_NAMESPACE: &str = "multi_agent_v1";
const MULTI_AGENT_V2_NAMESPACE: &str = "collaboration";
const SPAWN_AGENT_TOOL_NAME: &str = "spawn_agent";

fn spawn_agent_description(body: &Value) -> Option<String> {
    namespace_child_tool(body, MULTI_AGENT_V1_NAMESPACE, SPAWN_AGENT_TOOL_NAME)
        .and_then(|tool| tool.get("description"))
        .and_then(Value::as_str)
        .map(str::to_string)
}

fn spawn_agent_exposes_agent_type(body: &Value, namespace: &str) -> bool {
    namespace_child_tool(body, namespace, SPAWN_AGENT_TOOL_NAME)
        .and_then(|tool| tool.pointer("/parameters/properties/agent_type"))
        .is_some()
}

fn test_model_info(
    slug: &str,
    display_name: &str,
    description: &str,
    visibility: ModelVisibility,
    default_reasoning_level: ReasoningEffort,
    supported_reasoning_levels: Vec<ReasoningEffortPreset>,
    service_tiers: Vec<ModelServiceTier>,
) -> ModelInfo {
    ModelInfo {
        slug: slug.to_string(),
        orchestration: Some(ModelOrchestrationMetadata::Disabled {
            provider_id: "openai".to_string(),
            capability: ModelCapabilityTier::Unclassified,
            reason: "No allocation economics for this synthetic runtime".to_string(),
        }),
        chat_completions: Default::default(),
        display_name: display_name.to_string(),
        description: Some(description.to_string()),
        default_reasoning_level: Some(default_reasoning_level),
        supported_reasoning_levels,
        shell_type: ConfigShellToolType::ShellCommand,
        visibility,
        supported_in_api: true,
        input_modalities: default_input_modalities(),
        used_fallback_model_metadata: false,
        supports_search_tool: false,
        use_responses_lite: false,
        auto_review_model_override: None,
        tool_mode: None,
        multi_agent_version: None,
        priority: 1,
        additional_speed_tiers: Vec::new(),
        service_tiers,
        default_service_tier: None,
        upgrade: None,
        base_instructions: "base instructions".to_string(),
        model_messages: None,
        include_skills_usage_instructions: false,
        supports_reasoning_summary_parameter: true,
        default_reasoning_summary: ReasoningSummary::Auto,
        support_verbosity: false,
        default_verbosity: None,
        availability_nux: None,
        apply_patch_tool_type: None,
        web_search_tool_type: Default::default(),
        truncation_policy: TruncationPolicyConfig::bytes(/*limit*/ 10_000),
        supports_parallel_tool_calls: false,
        supports_image_detail_original: false,
        context_window: Some(272_000),
        max_context_window: None,
        max_output_tokens: None,
        auto_compact_token_limit: None,
        comp_hash: None,
        effective_context_window_percent: 95,
        experimental_supported_tools: Vec::new(),
    }
}

async fn wait_for_model_available(manager: &SharedModelsManager, slug: &str) {
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        let available_models = manager
            .list_models(
                RefreshStrategy::Online,
                codex_core::test_support::default_http_client_factory(),
            )
            .await;
        if available_models.iter().any(|model| model.model == slug) {
            return;
        }
        if Instant::now() >= deadline {
            panic!("timed out waiting for remote model {slug} to appear");
        }
        sleep(Duration::from_millis(25)).await;
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn spawn_agent_description_lists_visible_models_and_reasoning_efforts() -> Result<()> {
    let server = start_mock_server().await;
    mount_models_once(
        &server,
        ModelsResponse {
            models: vec![
                test_model_info(
                    "visible-model",
                    "Visible Model",
                    "Fast and capable",
                    ModelVisibility::List,
                    ReasoningEffort::Medium,
                    vec![
                        ReasoningEffortPreset {
                            effort: ReasoningEffort::Low,
                            description: "Quick scan".to_string(),
                        },
                        ReasoningEffortPreset {
                            effort: ReasoningEffort::Medium,
                            description: "Balanced".to_string(),
                        },
                        ReasoningEffortPreset {
                            effort: ReasoningEffort::High,
                            description: "Deep dive".to_string(),
                        },
                    ],
                    vec![ModelServiceTier {
                        id: "priority".to_string(),
                        name: "Fast".to_string(),
                        description: "1.5x speed, increased usage".to_string(),
                    }],
                ),
                test_model_info(
                    "hidden-model",
                    "Hidden Model",
                    "Should not be shown",
                    ModelVisibility::Hide,
                    ReasoningEffort::Low,
                    vec![ReasoningEffortPreset {
                        effort: ReasoningEffort::Low,
                        description: "Not visible".to_string(),
                    }],
                    Vec::new(),
                ),
            ],
        },
    )
    .await;
    let resp_mock = mount_sse_once(
        &server,
        sse(vec![ev_response_created("resp1"), ev_completed("resp1")]),
    )
    .await;

    let mut builder = test_codex()
        .with_auth(CodexAuth::create_dummy_chatgpt_auth_for_testing())
        .with_model("visible-model")
        .with_config(|config| {
            config
                .features
                .enable(Feature::Collab)
                .expect("test config should allow feature update");
            config.multi_agent_v2.hide_spawn_agent_metadata = false;
        });
    let test = builder.build(&server).await?;
    wait_for_model_available(&test.thread_manager.get_models_manager(), "visible-model").await;

    test.submit_turn("hello").await?;

    let body = resp_mock.single_request().body_json();
    let description =
        spawn_agent_description(&body).expect("spawn_agent description should be present");

    assert!(
        description.contains("- `openai` / `gpt-5.6-terra`"),
        "expected an eligible exact provider route in spawn_agent description: {description:?}"
    );
    assert!(
        description.contains(
            "Available authorized exact runtime overrides (optional; omit both fields to inherit the current runtime). Pass the provider as `model_provider` and the model as `model`."
        ),
        "expected provider/model choices to be framed as exact runtime pairs: {description:?}"
    );
    assert!(
        description
            .contains("Current inherited runtime: `openai` / `visible-model`; effort medium."),
        "expected the exact parent runtime in the spawn catalogue: {description:?}"
    );
    assert!(
        description.contains(
            "Default allocation policy: compare the task with this catalogue before every spawn."
        ),
        "expected model-aware default allocation policy: {description:?}"
    );
    assert!(
        description.contains(
            "Spawned agents inherit your current model by default. Omit `model` to use that preferred default; set `model` only when an explicit override is needed."
        ),
        "expected inherited-model guidance in spawn_agent description: {description:?}"
    );
    assert!(
        description.contains(
            "Do not set the `model` field unless the user explicitly asks for a different model or there is a clear task-specific reason."
        ),
        "expected model override usage guidance in spawn_agent description: {description:?}"
    );
    assert!(
        description.contains(
            "`openai` / `visible-model`; explicit-choice only; allocation economics unavailable;"
        ),
        "remote models without economics remain explicit choices: {description:?}"
    );
    assert!(
        !description.contains("hidden-model"),
        "hidden picker model should be omitted from spawn_agent description: {description:?}"
    );
    assert!(
        description.contains(
            "Do not spawn sub-agents unless the user or applicable AGENTS.md/skill instructions explicitly ask for sub-agents, delegation, or parallel agent work."
        ),
        "expected explicit authorization rule in spawn_agent description: {description:?}"
    );
    assert!(
        description.contains(
            "Requests for depth, thoroughness, research, investigation, or detailed codebase analysis do not count as permission to spawn."
        ) && description.contains("### When to delegate vs. do the subtask yourself"),
        "expected delegation decision guidance in spawn_agent description: {description:?}"
    );
    assert!(
        description.contains(
            "Agent-role guidance below only helps choose which agent to use after spawning is already authorized; it never authorizes spawning by itself."
        ),
        "expected agent-role clarification in spawn_agent description: {description:?}"
    );
    assert!(
        !description.contains("A mini model can solve many tasks faster than the main model."),
        "spawn_agent description should not encourage choosing a smaller model by default: {description:?}"
    );

    Ok(())
}

#[test_case(false; "without custom roles")]
#[test_case(true; "with custom roles")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn openai_reserved_spawn_schema_is_not_mutated_by_pf_role_configuration(
    has_agent_role: bool,
) -> Result<()> {
    let server = start_mock_server().await;
    let response = mount_sse_once(
        &server,
        sse(vec![ev_response_created("resp1"), ev_completed("resp1")]),
    )
    .await;
    let test = test_codex()
        .with_config(move |config| {
            config
                .features
                .enable(Feature::Collab)
                .expect("test config should allow feature update");
            config
                .features
                .enable(Feature::MultiAgentV2)
                .expect("test config should allow feature update");
            if has_agent_role {
                config.agent_roles.insert(
                    "researcher".to_string(),
                    AgentRoleConfig {
                        description: Some("Research role".to_string()),
                        config_file: None,
                        nickname_candidates: None,
                    },
                );
            }
        })
        .build_with_auto_env(&server)
        .await?;

    test.submit_turn("hello").await?;

    let body = response.single_request().body_json();
    assert!(!spawn_agent_exposes_agent_type(
        &body,
        MULTI_AGENT_V2_NAMESPACE
    ));
    let spawn = namespace_child_tool(&body, MULTI_AGENT_V2_NAMESPACE, SPAWN_AGENT_TOOL_NAME)
        .expect("native spawn should be present");
    let properties = spawn
        .pointer("/parameters/properties")
        .and_then(Value::as_object)
        .expect("native spawn should use object parameters");
    assert_eq!(
        properties.keys().cloned().collect::<Vec<_>>(),
        vec![
            "fork_turns".to_string(),
            "message".to_string(),
            "task_name".to_string()
        ]
    );
    assert_eq!(
        spawn.pointer("/parameters/properties/message/encrypted"),
        Some(&Value::Bool(true))
    );
    let tools = body
        .get("tools")
        .and_then(Value::as_array)
        .expect("request should contain tools");
    for adapter_name in [
        "spawn_agent_plaintext",
        "send_message_plaintext",
        "followup_task_plaintext",
    ] {
        let adapter = tools
            .iter()
            .find(|tool| tool.get("name").and_then(Value::as_str) == Some(adapter_name))
            .unwrap_or_else(|| panic!("expected {adapter_name} in the model request"));
        if adapter_name == "spawn_agent_plaintext" {
            let description = adapter["description"]
                .as_str()
                .expect("adapter description");
            assert!(description.contains(
                "`openai` / `gpt-6-astra`; explicit-choice only; allocation economics unavailable;"
            ));
        }
        assert_eq!(
            adapter.get("type").and_then(Value::as_str),
            Some("function")
        );
        assert_eq!(
            adapter.pointer("/parameters/properties/message/encrypted"),
            None,
            "cross-provider adapter must send an ordinary plaintext message field"
        );
    }
    Ok(())
}

#[test_case(None; "all configured providers")]
#[test_case(Some(vec!["openai".to_string()]); "explicit openai-only policy")]
#[test_case(Some(Vec::new()); "explicit deny-all policy")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn spawn_agent_v2_catalog_respects_provider_policy_with_mixed_model_defaults(
    allowlist: Option<Vec<String>>,
) -> Result<()> {
    let server = start_mock_server().await;
    let response = mount_sse_once(
        &server,
        sse(vec![ev_response_created("resp1"), ev_completed("resp1")]),
    )
    .await;
    let expected_openai = allowlist
        .as_ref()
        .is_none_or(|ids| ids.iter().any(|id| id == "openai"));
    let expected_kimi = allowlist.is_none();
    let test = test_codex()
        // Reproduce the account catalog's Luna preference without overriding it in
        // production. The bundled Kimi model has no multi-agent version preference.
        .with_model_info_override("gpt-5.6-luna", |model| {
            model.multi_agent_version = Some(MultiAgentVersion::V1);
        })
        .with_model("gpt-5.4")
        .with_config(move |config| {
            config
                .features
                .enable(Feature::MultiAgentV2)
                .expect("feature update");
            config.agent_provider_allowlist = allowlist;
            config.multi_agent_v2.expose_spawn_agent_model_overrides = true;
        })
        .build_with_auto_env(&server)
        .await?;
    test.submit_turn("Which exact child runtimes can I request?")
        .await?;
    let body = response.single_request().body_json();
    let native = namespace_child_tool(&body, MULTI_AGENT_V2_NAMESPACE, SPAWN_AGENT_TOOL_NAME)
        .expect("native spawn tool");
    let plaintext = body["tools"]
        .as_array()
        .unwrap()
        .iter()
        .find(|tool| tool["name"] == "spawn_agent_plaintext")
        .expect("cross-provider plaintext adapter");
    // The reserved native tool cannot select overrides; the plaintext adapter owns
    // the actionable catalog and exact provider/model parameters on OpenAI.
    let description = plaintext["description"].as_str().unwrap();
    assert_eq!(
        (
            description.contains("`openai` / `gpt-5.6-luna`"),
            description.contains("`openai` / `gpt-6-astra`; explicit-choice only;"),
            description.contains("`kimi-code` / `k3`")
        ),
        (expected_openai, expected_openai, expected_kimi),
        "provider policy, not engine preference or allocation economics, owns discovery: {description}"
    );
    assert!(
        plaintext
            .pointer("/parameters/properties/model_provider")
            .is_some()
    );
    assert!(plaintext.pointer("/parameters/properties/model").is_some());
    assert_eq!(
        native.pointer("/parameters/properties/message/encrypted"),
        Some(&Value::Bool(true))
    );
    assert_eq!(
        plaintext.pointer("/parameters/properties/message/encrypted"),
        None
    );
    Ok(())
}

#[test_case(true, false; "wait agent remains available without clock sleep")]
#[test_case(true, true; "wait agent remains available with clock sleep")]
#[test_case(false, false; "wait agent can be disabled without clock sleep")]
#[test_case(false, true; "wait agent can be disabled with clock sleep")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn multi_agent_v2_wait_agent_tool_follows_configuration(
    wait_agent_enabled: bool,
    sleep_tool_enabled: bool,
) -> Result<()> {
    let current_time_reminder = if sleep_tool_enabled {
        r#"
[features.current_time_reminder]
enabled = true
sleep_tool = true
"#
    } else {
        ""
    };
    let config_toml = format!(
        r#"
[features.multi_agent_v2]
enabled = true
wait_agent_enabled = {wait_agent_enabled}
{current_time_reminder}"#
    );
    let server = start_mock_server().await;
    let response = mount_sse_once(
        &server,
        sse(vec![ev_response_created("resp1"), ev_completed("resp1")]),
    )
    .await;
    let test = test_codex()
        .with_pre_build_hook(move |home| {
            std::fs::write(home.join("config.toml"), &config_toml)
                .expect("write multi-agent configuration");
        })
        .build_with_auto_env(&server)
        .await?;

    test.submit_turn("hello").await?;

    let request = response.single_request();
    let body = request.body_json();
    assert!(namespace_child_tool(&body, MULTI_AGENT_V2_NAMESPACE, SPAWN_AGENT_TOOL_NAME).is_some());
    assert_eq!(
        namespace_child_tool(&body, MULTI_AGENT_V2_NAMESPACE, "wait_agent").is_some(),
        wait_agent_enabled
    );
    assert_eq!(
        namespace_child_tool(&body, "clock", "sleep").is_some(),
        sleep_tool_enabled
    );

    Ok(())
}
