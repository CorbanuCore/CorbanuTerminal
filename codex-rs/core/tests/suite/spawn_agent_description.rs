#![cfg(not(target_os = "windows"))]
#![allow(clippy::unwrap_used)]

use anyhow::Result;
use codex_features::Feature;
use codex_login::CodexAuth;
use codex_models_manager::manager::RefreshStrategy;
use codex_models_manager::manager::SharedModelsManager;
use codex_protocol::config_types::ReasoningSummary;
use codex_protocol::openai_models::ConfigShellToolType;
use codex_protocol::openai_models::ModelInfo;
use codex_protocol::openai_models::ModelServiceTier;
use codex_protocol::openai_models::ModelVisibility;
use codex_protocol::openai_models::ModelsResponse;
use codex_protocol::openai_models::ReasoningEffort;
use codex_protocol::openai_models::ReasoningEffortPreset;
use codex_protocol::openai_models::TruncationPolicyConfig;
use codex_protocol::openai_models::default_input_modalities;
use core_test_support::responses::ev_completed;
use core_test_support::responses::ev_response_created;
use core_test_support::responses::mount_models_once;
use core_test_support::responses::mount_sse_once;
use core_test_support::responses::namespace_child_tool;
use core_test_support::responses::sse;
use core_test_support::responses::start_mock_server;
use core_test_support::test_codex::test_codex;
use serde_json::Value;
use std::time::Duration;
use std::time::Instant;
use tokio::time::sleep;

const MULTI_AGENT_V1_NAMESPACE: &str = "multi_agent_v1";
const SPAWN_AGENT_TOOL_NAME: &str = "spawn_agent";

fn spawn_agent_description(body: &Value) -> Option<String> {
    namespace_child_tool(body, MULTI_AGENT_V1_NAMESPACE, SPAWN_AGENT_TOOL_NAME)
        .and_then(|tool| tool.get("description"))
        .and_then(Value::as_str)
        .map(str::to_string)
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
        orchestration: None,
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
        supports_reasoning_summaries: false,
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
        auto_compact_token_limit: None,
        comp_hash: None,
        effective_context_window_percent: 95,
        experimental_supported_tools: Vec::new(),
    }
}

async fn wait_for_model_available(manager: &SharedModelsManager, slug: &str) {
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        let available_models = manager.list_models(RefreshStrategy::Online).await;
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
        description.contains(
            "- `ambient` / `z-ai/glm-5.2`; metered $0.76/$2.42 per M tok, balanced; text-only; efforts: medium (default), xhigh"
        ),
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
            .contains("Current inherited runtime: `ambient` / `visible-model`; effort medium."),
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
        !description.contains("- model `visible-model`"),
        "remote models without orchestration policy must fail closed as overrides: {description:?}"
    );
    assert!(
        !description.contains("hidden-model"),
        "hidden picker model should be omitted from spawn_agent description: {description:?}"
    );
    assert!(
        description.contains(
            "Do not spawn sub-agents unless the user explicitly asks for sub-agents, delegation, or parallel agent work."
        ),
        "expected explicit authorization rule in spawn_agent description: {description:?}"
    );
    assert!(
        !description.contains("### When to delegate vs. do the subtask yourself"),
        "spawn_agent description should not include extra when-to-use delegation guidance: {description:?}"
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

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn spawn_agent_description_filters_policy_and_marks_sol_ultra_frontier() -> Result<()> {
    let server = start_mock_server().await;
    let frontier_efforts = vec![
        ReasoningEffortPreset {
            effort: ReasoningEffort::High,
            description: "Deep work".to_string(),
        },
        ReasoningEffortPreset {
            effort: ReasoningEffort::Custom("max".to_string()),
            description: "Maximum reasoning".to_string(),
        },
        ReasoningEffortPreset {
            effort: ReasoningEffort::Custom("ultra".to_string()),
            description: "Maximum reasoning with automatic delegation".to_string(),
        },
    ];
    mount_models_once(
        &server,
        ModelsResponse {
            models: vec![
                test_model_info(
                    "gpt-5.6-sol",
                    "GPT-5.6-Sol",
                    "Frontier coding model",
                    ModelVisibility::List,
                    ReasoningEffort::High,
                    frontier_efforts.clone(),
                    Vec::new(),
                ),
                test_model_info(
                    "k3",
                    "Kimi K3",
                    "Frontier code model",
                    ModelVisibility::List,
                    ReasoningEffort::High,
                    frontier_efforts,
                    Vec::new(),
                ),
                test_model_info(
                    "gpt-5.5",
                    "GPT-5.5",
                    "Superseded coding model",
                    ModelVisibility::List,
                    ReasoningEffort::High,
                    vec![ReasoningEffortPreset {
                        effort: ReasoningEffort::High,
                        description: "Deep work".to_string(),
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
        .with_model("gpt-5.6-sol")
        .with_config(|config| {
            config.model_provider_id = "openai".to_string();
            config
                .features
                .enable(Feature::Collab)
                .expect("test config should allow feature update");
            config.multi_agent_v2.hide_spawn_agent_metadata = false;
            config.agent_provider_allowlist = Some(vec!["openai".to_string()]);
        });
    let test = builder.build(&server).await?;
    wait_for_model_available(&test.thread_manager.get_models_manager(), "gpt-5.6-sol").await;

    test.submit_turn("hello").await?;

    let description = spawn_agent_description(&resp_mock.single_request().body_json())
        .expect("spawn_agent description should be present");
    assert!(
        description.contains(
            "`openai` / `gpt-5.6-sol`; plan, burn 1x, frontier; frontier efforts: max, ultra (ultra includes automatic delegation)"
        ),
        "expected authorized Sol frontier allocation metadata: {description:?}"
    );
    assert!(
        !description.contains("`kimi-code` / `k3`"),
        "operator-disallowed providers must not be advertised as available: {description:?}"
    );
    assert!(
        !description.contains("`openai` / `gpt-5.5`"),
        "catalogue-disabled GPT-5.5 must not be advertised as spawnable: {description:?}"
    );
    assert!(
        description
            .contains("If the user names a provider or model, treat it as an exact constraint"),
        "expected exact-model no-substitution policy: {description:?}"
    );

    Ok(())
}
