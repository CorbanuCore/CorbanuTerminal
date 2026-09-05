use codex_protocol::config_types::Personality;
use codex_protocol::config_types::ReasoningSummary;
use codex_protocol::openai_models::ConfigShellToolType;
use codex_protocol::openai_models::ModelInfo;
use codex_protocol::openai_models::ModelInstructionsVariables;
use codex_protocol::openai_models::ModelMessages;
use codex_protocol::openai_models::ModelVisibility;
use codex_protocol::openai_models::ReasoningEffort;
use codex_protocol::openai_models::ReasoningEffortPreset;
use codex_protocol::openai_models::TruncationMode;
use codex_protocol::openai_models::TruncationPolicyConfig;
use codex_protocol::openai_models::WebSearchToolType;
use codex_protocol::openai_models::default_input_modalities;

use crate::config::ModelsManagerConfig;
use codex_utils_output_truncation::approx_bytes_for_tokens;
use tracing::warn;

pub const BASE_INSTRUCTIONS: &str = include_str!("../prompt.md");
const DEFAULT_PERSONALITY_HEADER: &str = "You are Codex, a coding agent based on GPT-5. You and the user share the same workspace and collaborate to achieve the user's goals.";
const COMPAT_PERSONALITY_HEADER: &str = "You are Codex, a coding agent. You and the user share the same workspace and collaborate to achieve the user's goals.";
const LOCAL_FRIENDLY_TEMPLATE: &str =
    "You optimize for team morale and being a supportive teammate as much as code quality.";
const LOCAL_PRAGMATIC_TEMPLATE: &str = "You are a deeply pragmatic, effective software engineer.";
const PERSONALITY_PLACEHOLDER: &str = "{{ personality }}";
const PERSONALITY_SECTION_HEADER: &str = "# Personality";

pub fn with_config_overrides(mut model: ModelInfo, config: &ModelsManagerConfig) -> ModelInfo {
    if config.model_supports_reasoning_summaries == Some(true) {
        model.supports_reasoning_summary_parameter = true;
    }
    if let Some(context_window) = config.model_context_window {
        model.context_window = Some(
            model
                .max_context_window
                .map_or(context_window, |max_context_window| {
                    context_window.min(max_context_window)
                }),
        );
    }
    if let Some(max_output_tokens) = config.model_max_output_tokens {
        model.max_output_tokens = Some(
            model
                .max_output_tokens
                .map_or(max_output_tokens, |catalog_limit| {
                    max_output_tokens.min(catalog_limit)
                }),
        );
    }
    if let Some(auto_compact_token_limit) = config.model_auto_compact_token_limit {
        model.auto_compact_token_limit = Some(auto_compact_token_limit);
    }
    if let Some(token_limit) = config.tool_output_token_limit {
        model.truncation_policy = match model.truncation_policy.mode {
            TruncationMode::Bytes => {
                let byte_limit =
                    i64::try_from(approx_bytes_for_tokens(token_limit)).unwrap_or(i64::MAX);
                TruncationPolicyConfig::bytes(byte_limit)
            }
            TruncationMode::Tokens => {
                let limit = i64::try_from(token_limit).unwrap_or(i64::MAX);
                TruncationPolicyConfig::tokens(limit)
            }
        };
    }

    if let Some(base_instructions) = &config.base_instructions {
        model.base_instructions = base_instructions.clone();
        clear_instruction_messages(&mut model);
    } else {
        if config.personality_enabled && config.personality == Some(Personality::None) {
            model.base_instructions = strip_personality_section(model.base_instructions);
            if let Some(instructions_template) = model
                .model_messages
                .as_mut()
                .and_then(|messages| messages.instructions_template.as_mut())
            {
                *instructions_template =
                    strip_personality_section(std::mem::take(instructions_template));
            }
        }
        if !config.personality_enabled {
            // Newer native catalogs carry the complete instructions in the
            // template and leave the legacy base empty. Disabling personality
            // must not discard the model's entire operating contract.
            if model.base_instructions.is_empty() {
                model.base_instructions = model.get_model_instructions(Some(Personality::None));
            }
            clear_instruction_messages(&mut model);
        }
    }

    model
}

fn strip_personality_section(mut instructions: String) -> String {
    let mut section_start = None;
    let mut section_end = None;
    let mut offset = 0;

    for line_with_ending in instructions.split_inclusive('\n') {
        let line = match line_with_ending.strip_suffix('\n') {
            Some(line) => line.strip_suffix('\r').unwrap_or(line),
            None => line_with_ending,
        };
        if section_start.is_some() {
            if is_h1_heading(line) {
                section_end = Some(offset);
                break;
            }
        } else if line == PERSONALITY_SECTION_HEADER {
            section_start = Some(offset);
        }
        offset += line_with_ending.len();
    }

    if let Some(section_start) = section_start {
        let section_end = section_end.unwrap_or(instructions.len());
        instructions.replace_range(section_start..section_end, "");
    }

    instructions
}

fn is_h1_heading(line: &str) -> bool {
    let Some(rest) = line.strip_prefix('#') else {
        return false;
    };
    rest.is_empty() || rest.starts_with(' ') || rest.starts_with('\t')
}

fn clear_instruction_messages(model: &mut ModelInfo) {
    if let Some(model_messages) = &mut model.model_messages {
        model_messages.instructions_template = None;
        model_messages.instructions_variables = None;
        if model_messages.approvals.is_none()
            && model_messages.auto_review.is_none()
            && model_messages.permissions.is_none()
            && model_messages.token_budget.is_none()
        {
            model.model_messages = None;
        }
    }
}

/// Build a minimal fallback model descriptor for missing/unknown slugs.
pub fn model_info_from_slug(slug: &str) -> ModelInfo {
    let is_deepseek_v4_flash = slug == "deepseek-ai/DeepSeek-V4-Flash-0731";
    let curated_gpu_model = match slug {
        "deepseek-ai/DeepSeek-V4-Flash-0731" => Some(("DeepSeek V4 Flash 0731", 384_000)),
        "zai-org/GLM-5.2-FP8" => Some(("GLM 5.2 FP8", 131_072)),
        _ => None,
    };
    if curated_gpu_model.is_none() {
        warn!("Unknown model {slug} is used. This will use fallback model metadata.");
    }
    let (display_name, context_window, used_fallback_model_metadata) = curated_gpu_model
        .map_or_else(
            || (slug.to_string(), 272_000, true),
            |(display_name, context_window)| (display_name.to_string(), context_window, false),
        );
    ModelInfo {
        slug: slug.to_string(),
        orchestration: None,
        chat_completions: Default::default(),
        display_name,
        description: None,
        default_reasoning_level: is_deepseek_v4_flash.then_some(ReasoningEffort::High),
        supported_reasoning_levels: if is_deepseek_v4_flash {
            vec![
                ReasoningEffortPreset {
                    effort: ReasoningEffort::Low,
                    description: "Faster DeepSeek V4 reasoning effort".to_string(),
                },
                ReasoningEffortPreset {
                    effort: ReasoningEffort::High,
                    description: "DeepSeek V4 thinking mode".to_string(),
                },
                ReasoningEffortPreset {
                    effort: ReasoningEffort::Max,
                    description: "Maximum DeepSeek V4 reasoning effort".to_string(),
                },
            ]
        } else {
            Vec::new()
        },
        shell_type: ConfigShellToolType::Default,
        visibility: ModelVisibility::None,
        supported_in_api: true,
        priority: 99,
        additional_speed_tiers: Vec::new(),
        service_tiers: Vec::new(),
        default_service_tier: None,
        availability_nux: None,
        upgrade: None,
        base_instructions: BASE_INSTRUCTIONS.to_string(),
        model_messages: local_personality_messages_for_slug(slug),
        include_skills_usage_instructions: false,
        supports_reasoning_summary_parameter: true,
        default_reasoning_summary: ReasoningSummary::Auto,
        support_verbosity: false,
        default_verbosity: None,
        apply_patch_tool_type: None,
        web_search_tool_type: WebSearchToolType::Text,
        truncation_policy: TruncationPolicyConfig::bytes(/*limit*/ 10_000),
        supports_parallel_tool_calls: false,
        supports_image_detail_original: false,
        context_window: Some(context_window),
        max_context_window: Some(context_window),
        max_output_tokens: None,
        auto_compact_token_limit: None,
        comp_hash: None,
        effective_context_window_percent: 95,
        experimental_supported_tools: Vec::new(),
        input_modalities: default_input_modalities(),
        used_fallback_model_metadata,
        supports_search_tool: false,
        use_responses_lite: false,
        auto_review_model_override: None,
        tool_mode: None,
        multi_agent_version: None,
    }
}

fn local_personality_messages_for_slug(slug: &str) -> Option<ModelMessages> {
    let header = match slug {
        "gpt-5.2-codex" | "exp-codex-personality" => DEFAULT_PERSONALITY_HEADER,
        "deepseek-ai/DeepSeek-V4-Flash-0731" | "zai-org/GLM-5.2-FP8" => COMPAT_PERSONALITY_HEADER,
        _ => return None,
    };

    Some(ModelMessages {
        instructions_template: Some(format!(
            "{header}\n\n{PERSONALITY_PLACEHOLDER}\n\n{BASE_INSTRUCTIONS}"
        )),
        instructions_variables: Some(ModelInstructionsVariables {
            personality_default: Some(String::new()),
            personality_friendly: Some(LOCAL_FRIENDLY_TEMPLATE.to_string()),
            personality_pragmatic: Some(LOCAL_PRAGMATIC_TEMPLATE.to_string()),
        }),
        approvals: None,
        auto_review: None,
        permissions: None,
        token_budget: None,
    })
}

#[cfg(test)]
#[path = "model_info_tests.rs"]
mod tests;
