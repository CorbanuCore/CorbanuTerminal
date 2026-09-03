//! Registry of model providers supported by Codex.
//!
//! Providers can be defined in two places:
//!   1. Built-in defaults compiled into the binary so Codex works out-of-the-box.
//!   2. User-defined entries inside `~/.codex/config.toml` under the `model_providers`
//!      key. These override or extend the defaults at runtime.

use codex_api::Provider as ApiProvider;
use codex_api::RetryConfig as ApiRetryConfig;
use codex_api::is_azure_responses_provider;
use codex_product_brand::PLAN_NAME;

/// Display name for the plan's Anthropic-wire non-private Fable route.
const PLAN_ANTHROPIC_NAME: &str = "Corbanu Plan (Fable, non-private)";
use codex_protocol::auth::AuthMode;
use codex_protocol::config_types::ModelProviderAuthInfo;
use codex_protocol::error::CodexErr;
use codex_protocol::error::EnvVarError;
use codex_protocol::error::Result as CodexResult;
use codex_utils_absolute_path::AbsolutePathBuf;
use http::HeaderMap;
use http::header::HeaderName;
use http::header::HeaderValue;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;
use std::num::NonZeroU64;
use std::time::Duration;

const DEFAULT_ANTHROPIC_REQUEST_BODY_MAX_BYTES: usize = 30_000_000;
const DEFAULT_ANTHROPIC_RETRY_BODY_MAX_BYTES: usize = 15_000_000;

const DEFAULT_STREAM_IDLE_TIMEOUT_MS: u64 = 600_000;
const DEFAULT_STREAM_ACTIONABLE_TIMEOUT_MS: u64 = 180_000;
const DEFAULT_STREAM_LONG_FAILURE_RETRY_THRESHOLD_MS: u64 = 60_000;
const DEFAULT_STREAM_LONG_FAILURE_MAX_RETRIES: u64 = 1;
const DEFAULT_STREAM_MAX_RETRIES: u64 = 5;
const DEFAULT_REQUEST_MAX_RETRIES: u64 = 4;
pub const DEFAULT_WEBSOCKET_CONNECT_TIMEOUT_MS: u64 = 15_000;
/// Canonical executable guaranteed by every supported Corbanu Terminal install.
const CORBANU_PROVIDER_AUTH_COMMAND: &str = "corbanu";
/// Hard cap for user-configured `stream_max_retries`.
const MAX_STREAM_MAX_RETRIES: u64 = 100;
/// Hard cap for user-configured `request_max_retries`.
const MAX_REQUEST_MAX_RETRIES: u64 = 100;

fn claude_provider_auth_timeout_ms() -> NonZeroU64 {
    // Claude Code refresh may itself take up to 30 seconds, and the managed
    // source must unlock the encrypted vault before emitting its token. Keep
    // only Claude's outer provider boundary above both inner operations.
    match NonZeroU64::new(60_000) {
        Some(timeout_ms) => timeout_ms,
        None => panic!("Claude provider auth timeout must be non-zero"),
    }
}

fn claude_provider_auth_cwd() -> AbsolutePathBuf {
    // Claude Code resolves a relative CLAUDE_CONFIG_DIR against the caller's
    // working directory. Keep the provider helper on that same directory so
    // its credentials-file identity cannot drift from the profile selected by
    // the TUI. The command remains a bare installed executable resolved via
    // PATH, never a project-relative program.
    AbsolutePathBuf::current_dir().unwrap_or_else(|err| {
        panic!("current directory must resolve to determine Claude provider auth cwd: {err}")
    })
}

const OPENAI_PROVIDER_NAME: &str = "OpenAI";
const OPENAI_ACTOR_AUTHORIZATION_HEADER: &str = "x-openai-actor-authorization";
pub const OPENAI_PROVIDER_ID: &str = "openai";
/// OpenAI backend compatibility version for protocol-gated model access.
///
/// This is intentionally separate from Corbanu Terminal's product version. The OpenAI
/// provider sends this value to first-party endpoints so fork-specific release
/// numbering does not make the backend treat a compatible client as obsolete.
pub const OPENAI_CODEX_COMPAT_VERSION: &str = "0.144.1";
pub const CHATGPT_CODEX_BASE_URL: &str = "https://chatgpt.com/backend-api/codex";
const ANTHROPIC_PROVIDER_NAME: &str = "Anthropic";
pub const ANTHROPIC_PROVIDER_ID: &str = "anthropic";
pub const ANTHROPIC_BASE_URL: &str = "https://api.anthropic.com/v1";
pub const ANTHROPIC_DEFAULT_MODEL: &str = "claude-opus-5";
pub const ANTHROPIC_LEGACY_OPUS_4_8_MODEL: &str = "claude-opus-4-8";
pub const CLAUDE_FABLE_5_1_MODEL: &str = "claude-fable-5-1";
pub const CLAUDE_FABLE_5_MODEL: &str = "claude-fable-5";
pub const ANTHROPIC_API_KEY_ENV_VAR: &str = "ANTHROPIC_API_KEY";
const CLAUDE_PLAN_PROVIDER_NAME: &str = "Claude Plan";
pub const CLAUDE_PLAN_PROVIDER_ID: &str = "claude-plan";
pub const CLAUDE_PLAN_MODEL: &str = "claude-opus-5-plan";
pub const CLAUDE_PLAN_UPSTREAM_MODEL: &str = ANTHROPIC_DEFAULT_MODEL;
pub const CLAUDE_PLAN_LEGACY_OPUS_4_8_MODEL: &str = "claude-opus-4-8-plan";
pub const CLAUDE_FABLE_5_1_PLAN_MODEL: &str = "claude-fable-5-1-plan";
pub const CLAUDE_FABLE_5_1_PLAN_UPSTREAM_MODEL: &str = CLAUDE_FABLE_5_1_MODEL;
pub const CLAUDE_FABLE_5_PLAN_MODEL: &str = "claude-fable-5-plan";
pub const CLAUDE_FABLE_5_PLAN_UPSTREAM_MODEL: &str = CLAUDE_FABLE_5_MODEL;
const AMBIENT_PROVIDER_NAME: &str = "Ambient";
pub const AMBIENT_PROVIDER_ID: &str = "ambient";
pub const AMBIENT_BASE_URL: &str = "https://api.ambient.xyz/v1";
pub const AMBIENT_DEFAULT_MODEL: &str = "z-ai/glm-5.2";
/// Input context accepted by Ambient's GLM 5.2 chat route.
///
/// The model can have a larger native context on other providers, so this
/// limit belongs to the provider/model route rather than the shared model
/// catalog.
pub const AMBIENT_GLM_5_2_CONTEXT_WINDOW: i64 = 101_376;
pub const AMBIENT_LEGACY_GLM_5_2_FP8_MODEL: &str = "zai-org/GLM-5.2-FP8";
pub const AMBIENT_KIMI_K2_7_CODE_MODEL: &str = "moonshotai/kimi-k2.7-code";
pub const AMBIENT_API_KEY_ENV_VAR: &str = "AMBIENT_API_KEY";
/// Current public provider identifier.
///
/// Both earlier identifiers remain accepted, while the original PFTerminal ID
/// stays canonical in persisted state until a versioned migration exists.
pub const CORBANU_PLAN_PROVIDER_ID: &str = "corbanu-plan";
pub const CORBANU_TERMINAL_PLAN_PROVIDER_ID: &str = "corbanu-terminal-plan";
pub const CORBANU_PLAN_API_KEY_ENV_VAR: &str = "CORBANU_PLAN_API_KEY";
pub const PFTERMINAL_PLAN_PROVIDER_ID: &str = "pfterminal-plan";
/// Anthropic-wire sibling of the Corbanu Plan provider. Same gateway, same
/// customer key; serves the plan's non-private `claude-fable-5` and
/// `claude-fable-5-1` routes.
pub const PFTERMINAL_PLAN_ANTHROPIC_PROVIDER_ID: &str = "pfterminal-plan-anthropic";
pub const CORBANU_PLAN_ANTHROPIC_PROVIDER_ID: &str = "corbanu-plan-anthropic";
/// Context reliably served by the SkyAPI Fable route used by Corbanu Plan.
pub const PFTERMINAL_PLAN_FABLE_CONTEXT_WINDOW: i64 = 128_000;
/// Completion ceiling that leaves input headroom on the SkyAPI Fable route.
pub const PFTERMINAL_PLAN_FABLE_MAX_OUTPUT_TOKENS: i64 = 32_768;
pub const PFTERMINAL_PLAN_GATEWAY_ORIGIN: &str = "https://api.corbanu.com";
pub const PFTERMINAL_PLAN_DEFAULT_BASE_URL: &str = "https://api.corbanu.com/v1";
pub const PFTERMINAL_PLAN_API_KEY_ENV_VAR: &str = "PFTERMINAL_PLAN_API_KEY";

/// Normalize public provider aliases to the stable identifier used by existing
/// configuration and credential storage.
pub fn canonical_provider_id(provider_id: &str) -> &str {
    match provider_id {
        CORBANU_PLAN_PROVIDER_ID | CORBANU_TERMINAL_PLAN_PROVIDER_ID => PFTERMINAL_PLAN_PROVIDER_ID,
        CORBANU_PLAN_ANTHROPIC_PROVIDER_ID => PFTERMINAL_PLAN_ANTHROPIC_PROVIDER_ID,
        _ => provider_id,
    }
}
const KIMI_CODE_PROVIDER_NAME: &str = "Kimi Code";
pub const KIMI_CODE_PROVIDER_ID: &str = "kimi-code";
pub const KIMI_CODE_BASE_URL: &str = "https://api.kimi.com/coding/v1";
pub const KIMI_CODE_K3_MODEL: &str = "k3";
pub const KIMI_CODE_API_KEY_ENV_VAR: &str = "KIMI_API_KEY";
const ZAI_PROVIDER_NAME: &str = "Z.AI";
pub const ZAI_PROVIDER_ID: &str = "zai";
pub const ZAI_BASE_URL: &str = "https://api.z.ai/api/coding/paas/v4";
const ZAI_ANTHROPIC_PROVIDER_NAME: &str = "Z.AI Anthropic";
pub const ZAI_ANTHROPIC_PROVIDER_ID: &str = "zai-anthropic";
pub const ZAI_ANTHROPIC_BASE_URL: &str = "https://api.z.ai/api/anthropic/v1";
pub const ZAI_DEFAULT_MODEL: &str = "glm-5.2";
pub const ZAI_API_KEY_ENV_VAR: &str = "ZAI_API_KEY";
const OPENROUTER_PROVIDER_NAME: &str = "OpenRouter";
pub const OPENROUTER_PROVIDER_ID: &str = "openrouter";
pub const OPENROUTER_BASE_URL: &str = "https://openrouter.ai/api/v1";
const OPENROUTER_ANTHROPIC_PROVIDER_NAME: &str = "OpenRouter Anthropic";
pub const OPENROUTER_ANTHROPIC_PROVIDER_ID: &str = "openrouter-anthropic";
pub const OPENROUTER_DEFAULT_MODEL: &str = "z-ai/glm-5.2";
pub const OPENROUTER_GROK_4_6_MODEL: &str = "x-ai/grok-4.6";
pub const OPENROUTER_API_KEY_ENV_VAR: &str = "OPENROUTER_API_KEY";
const DEEPSEEK_PROVIDER_NAME: &str = "DeepSeek";
pub const DEEPSEEK_PROVIDER_ID: &str = "deepseek";
pub const DEEPSEEK_BASE_URL: &str = "https://api.deepseek.com";
pub const DEEPSEEK_DEFAULT_MODEL: &str = "deepseek-v4-flash";
pub const DEEPSEEK_PRO_MODEL: &str = "deepseek-v4-pro";
pub const OPENROUTER_DEEPSEEK_V4_PRO_0813_MODEL: &str = "deepseek/deepseek-v4-pro-0813";
pub const DEEPSEEK_API_KEY_ENV_VAR: &str = "DEEPSEEK_API_KEY";
const META_PROVIDER_NAME: &str = "Meta";
pub const META_PROVIDER_ID: &str = "meta";
pub const META_BASE_URL: &str = "https://api.meta.ai/v1";
pub const META_DEFAULT_MODEL: &str = "muse-spark-1.1";
pub const META_API_KEY_ENV_VAR: &str = "MODEL_API_KEY";
const BASETEN_PROVIDER_NAME: &str = "Baseten";
pub const BASETEN_PROVIDER_ID: &str = "baseten";
pub const BASETEN_BASE_URL: &str = "https://inference.baseten.co/v1";
const BASETEN_ANTHROPIC_PROVIDER_NAME: &str = "Baseten Anthropic";
pub const BASETEN_ANTHROPIC_PROVIDER_ID: &str = "baseten-anthropic";
pub const BASETEN_DEFAULT_MODEL: &str = "zai-org/GLM-5.2";
pub const BASETEN_API_KEY_ENV_VAR: &str = "BASETEN_API_KEY";
const VERCEL_PROVIDER_NAME: &str = "Vercel";
pub const VERCEL_PROVIDER_ID: &str = "vercel";
pub const VERCEL_BASE_URL: &str = "https://ai-gateway.vercel.sh/v1";
const VERCEL_GATEWAY_HOST: &str = "ai-gateway.vercel.sh";

/// Host-only comparison: the gateway is reachable under several path prefixes
/// (`/v1`, `/v1/messages`, `/v1/responses`) and with or without a trailing
/// slash, and none of that changes the routing behaviour we need to correct.
fn is_vercel_gateway_base_url(base_url: &str) -> bool {
    let without_scheme = base_url
        .trim()
        .split_once("://")
        .map_or(base_url.trim(), |(_, rest)| rest);
    let host = without_scheme
        .split(['/', '?', '#'])
        .next()
        .unwrap_or_default();
    let host = host.rsplit_once('@').map_or(host, |(_, rest)| rest);
    let host = host.split_once(':').map_or(host, |(host, _)| host);
    host.eq_ignore_ascii_case(VERCEL_GATEWAY_HOST)
}
const VERCEL_ANTHROPIC_PROVIDER_NAME: &str = "Vercel Anthropic";
pub const VERCEL_ANTHROPIC_PROVIDER_ID: &str = "vercel-anthropic";
const VERCEL_ANTHROPIC_FAST_PROVIDER_NAME: &str = "Vercel Anthropic Fast";
pub const VERCEL_ANTHROPIC_FAST_PROVIDER_ID: &str = "vercel-anthropic-fast";
pub const VERCEL_DEFAULT_MODEL: &str = "zai/glm-5.2";
pub const VERCEL_GLM_5_2_FAST_MODEL: &str = "zai/glm-5.2-fast";
pub const VERCEL_API_KEY_ENV_VAR: &str = "AI_GATEWAY_API_KEY";

/// Built-in catalog providers eligible for impossible-pair correction. User-defined providers
/// (e.g. a private Azure deployment) are never second-guessed.
const PAIR_CORRECTION_KNOWN_PROVIDERS: [&str; 19] = [
    OPENAI_PROVIDER_ID,
    ANTHROPIC_PROVIDER_ID,
    CLAUDE_PLAN_PROVIDER_ID,
    AMBIENT_PROVIDER_ID,
    PFTERMINAL_PLAN_PROVIDER_ID,
    PFTERMINAL_PLAN_ANTHROPIC_PROVIDER_ID,
    KIMI_CODE_PROVIDER_ID,
    ZAI_PROVIDER_ID,
    ZAI_ANTHROPIC_PROVIDER_ID,
    OPENROUTER_PROVIDER_ID,
    OPENROUTER_ANTHROPIC_PROVIDER_ID,
    DEEPSEEK_PROVIDER_ID,
    META_PROVIDER_ID,
    BASETEN_PROVIDER_ID,
    BASETEN_ANTHROPIC_PROVIDER_ID,
    VERCEL_PROVIDER_ID,
    VERCEL_ANTHROPIC_PROVIDER_ID,
    VERCEL_ANTHROPIC_FAST_PROVIDER_ID,
    AMAZON_BEDROCK_PROVIDER_ID,
];

const VERCEL_FAMILY_PROVIDERS: [&str; 3] = [
    VERCEL_PROVIDER_ID,
    VERCEL_ANTHROPIC_PROVIDER_ID,
    VERCEL_ANTHROPIC_FAST_PROVIDER_ID,
];

/// Return the canonical built-in provider for a picker-visible model.
///
/// This is a catalog ownership mapping, not an exclusivity claim: gateways and
/// user-defined providers may also serve the same model. Callers use it when
/// they must present an exact, known-good provider/model pair instead of
/// forcing a model to guess a provider identifier from a display name.
pub fn canonical_catalog_provider(model: &str) -> Option<&'static str> {
    let model = model.trim();
    if model.is_empty() {
        return None;
    }
    if model == BASETEN_DEFAULT_MODEL {
        return Some(BASETEN_PROVIDER_ID);
    }
    if model == AMBIENT_DEFAULT_MODEL
        || model == AMBIENT_KIMI_K2_7_CODE_MODEL
        || model.starts_with("ambient/")
        || model.starts_with("zai-org/")
    {
        return Some(AMBIENT_PROVIDER_ID);
    }
    if model == KIMI_CODE_K3_MODEL {
        return Some(KIMI_CODE_PROVIDER_ID);
    }
    if model == ZAI_DEFAULT_MODEL || model.starts_with("glm-") {
        return Some(ZAI_PROVIDER_ID);
    }
    if matches!(
        model,
        CLAUDE_PLAN_MODEL
            | CLAUDE_PLAN_LEGACY_OPUS_4_8_MODEL
            | CLAUDE_FABLE_5_1_PLAN_MODEL
            | CLAUDE_FABLE_5_PLAN_MODEL
    ) {
        return Some(CLAUDE_PLAN_PROVIDER_ID);
    }
    if model == ANTHROPIC_DEFAULT_MODEL
        || matches!(model, CLAUDE_FABLE_5_1_MODEL | CLAUDE_FABLE_5_MODEL)
    {
        return Some(CLAUDE_PLAN_PROVIDER_ID);
    }
    if model == META_DEFAULT_MODEL {
        return Some(META_PROVIDER_ID);
    }
    if matches!(model, DEEPSEEK_DEFAULT_MODEL | DEEPSEEK_PRO_MODEL) {
        return Some(DEEPSEEK_PROVIDER_ID);
    }
    if matches!(
        model,
        OPENROUTER_DEFAULT_MODEL
            | "minimax/minimax-m3"
            | "openrouter/owl-alpha"
            | "google/gemini-3.5-flash"
            | OPENROUTER_GROK_4_6_MODEL
            | "x-ai/grok-4.5"
            | OPENROUTER_DEEPSEEK_V4_PRO_0813_MODEL
            | "deepseek/deepseek-v4-pro"
            | "deepseek/deepseek-v4-flash-0731"
            | "tencent/hy3:free"
            | "moonshotai/kimi-k3"
    ) {
        return Some(OPENROUTER_PROVIDER_ID);
    }
    if model == VERCEL_GLM_5_2_FAST_MODEL {
        return Some(VERCEL_ANTHROPIC_FAST_PROVIDER_ID);
    }
    if model == VERCEL_DEFAULT_MODEL {
        return Some(VERCEL_PROVIDER_ID);
    }
    if matches!(
        model,
        AMAZON_BEDROCK_GPT_5_5_MODEL_ID | AMAZON_BEDROCK_GPT_5_4_MODEL_ID
    ) {
        return Some(AMAZON_BEDROCK_PROVIDER_ID);
    }
    if model.starts_with("gpt-") || model.starts_with("codex-auto-") {
        return Some(OPENAI_PROVIDER_ID);
    }
    None
}

/// Returns the provider a session must use when its (model, provider) pair is impossible — the
/// model belongs to a specific catalog provider family and the given provider is a different
/// catalog provider that cannot serve it. Pairs go stale when thread metadata loses the provider
/// and a fallback (config default, parent provider) is recorded next to a role- or
/// rollout-derived model; running such a turn 400s/404s at the remote ("Unknown model").
///
/// Only unambiguous families are corrected:
/// - `zai/…` slugs (Vercel gateway GLM ids) off the Vercel provider family;
/// - the Claude plan models off `claude-plan`;
/// - bare Claude models off unrelated providers and onto the authenticated
///   `claude-plan` route (never implicitly onto metered direct Anthropic);
/// - bare `glm-…` slugs (Z.AI-direct ids) off either Z.AI dialect;
/// - the bare `k3` subscription model off Kimi Code;
/// - the bare DeepSeek Responses model off the direct DeepSeek provider;
/// - bare `gpt-…` slugs off OpenAI (Bedrock uses `openai.gpt-…` ids).
///
/// Servable-but-unusual pairs (e.g. ambient serving `z-ai/glm-5.2`), unknown models, and
/// user-defined providers return `None` so intentional setups keep working.
pub fn corrected_catalog_provider(model: &str, provider: &str) -> Option<&'static str> {
    let model = model.trim();
    let provider = provider.trim();
    if model.is_empty()
        || provider.is_empty()
        || !PAIR_CORRECTION_KNOWN_PROVIDERS.contains(&provider)
    {
        return None;
    }
    if model.starts_with("zai/") && !VERCEL_FAMILY_PROVIDERS.contains(&provider) {
        return Some(VERCEL_ANTHROPIC_FAST_PROVIDER_ID);
    }
    if matches!(
        model,
        CLAUDE_PLAN_MODEL
            | CLAUDE_PLAN_LEGACY_OPUS_4_8_MODEL
            | CLAUDE_FABLE_5_1_PLAN_MODEL
            | CLAUDE_FABLE_5_PLAN_MODEL
    ) && provider != CLAUDE_PLAN_PROVIDER_ID
    {
        return Some(CLAUDE_PLAN_PROVIDER_ID);
    }
    if model.starts_with("claude-")
        && provider != ANTHROPIC_PROVIDER_ID
        && provider != CLAUDE_PLAN_PROVIDER_ID
        && provider != PFTERMINAL_PLAN_ANTHROPIC_PROVIDER_ID
    {
        return Some(CLAUDE_PLAN_PROVIDER_ID);
    }
    if model.starts_with("glm-")
        && provider != ZAI_PROVIDER_ID
        && provider != ZAI_ANTHROPIC_PROVIDER_ID
    {
        return Some(ZAI_PROVIDER_ID);
    }
    if model == KIMI_CODE_K3_MODEL && provider != KIMI_CODE_PROVIDER_ID {
        return Some(KIMI_CODE_PROVIDER_ID);
    }
    if matches!(model, DEEPSEEK_DEFAULT_MODEL | DEEPSEEK_PRO_MODEL)
        && provider != DEEPSEEK_PROVIDER_ID
        && provider != PFTERMINAL_PLAN_PROVIDER_ID
    {
        return Some(DEEPSEEK_PROVIDER_ID);
    }
    if model.starts_with("gpt-") && provider != OPENAI_PROVIDER_ID {
        return Some(OPENAI_PROVIDER_ID);
    }
    None
}

pub fn resolve_model_for_provider(
    model: Option<String>,
    model_provider_id: &str,
) -> Option<String> {
    match model_provider_id {
        PFTERMINAL_PLAN_ANTHROPIC_PROVIDER_ID => match model {
            Some(model)
                if matches!(model.trim(), CLAUDE_FABLE_5_1_MODEL | CLAUDE_FABLE_5_MODEL) =>
            {
                Some(model)
            }
            _ => Some(CLAUDE_FABLE_5_MODEL.to_string()),
        },
        PFTERMINAL_PLAN_PROVIDER_ID
            if model.as_deref().map(str::trim) == Some(DEEPSEEK_PRO_MODEL) =>
        {
            Some(DEEPSEEK_PRO_MODEL.to_string())
        }
        AMBIENT_PROVIDER_ID | PFTERMINAL_PLAN_PROVIDER_ID => match model {
            Some(model) if model.trim() == AMBIENT_LEGACY_GLM_5_2_FP8_MODEL => {
                Some(AMBIENT_DEFAULT_MODEL.to_string())
            }
            Some(model)
                if model.trim().starts_with("ambient/")
                    || model.trim().starts_with("z-ai/")
                    || model.trim() == AMBIENT_KIMI_K2_7_CODE_MODEL
                    || (model.trim().starts_with("zai-org/")
                        && model.trim() != BASETEN_DEFAULT_MODEL) =>
            {
                Some(model)
            }
            _ => Some(AMBIENT_DEFAULT_MODEL.to_string()),
        },
        KIMI_CODE_PROVIDER_ID => match model {
            Some(model) if model.trim() == KIMI_CODE_K3_MODEL => Some(model),
            _ => Some(KIMI_CODE_K3_MODEL.to_string()),
        },
        ZAI_PROVIDER_ID | ZAI_ANTHROPIC_PROVIDER_ID => match model {
            Some(model) if model.trim().starts_with("glm-") => Some(model),
            _ => Some(ZAI_DEFAULT_MODEL.to_string()),
        },
        ANTHROPIC_PROVIDER_ID => match model {
            Some(model)
                if model.trim().starts_with("claude-")
                    && model.trim() != CLAUDE_PLAN_MODEL
                    && model.trim() != CLAUDE_PLAN_LEGACY_OPUS_4_8_MODEL
                    && model.trim() != CLAUDE_FABLE_5_1_PLAN_MODEL
                    && model.trim() != CLAUDE_FABLE_5_PLAN_MODEL =>
            {
                Some(model)
            }
            _ => Some(ANTHROPIC_DEFAULT_MODEL.to_string()),
        },
        CLAUDE_PLAN_PROVIDER_ID => match model {
            Some(model) if model.trim() == ANTHROPIC_DEFAULT_MODEL => {
                Some(CLAUDE_PLAN_MODEL.to_string())
            }
            Some(model) if model.trim() == ANTHROPIC_LEGACY_OPUS_4_8_MODEL => {
                Some(CLAUDE_PLAN_LEGACY_OPUS_4_8_MODEL.to_string())
            }
            Some(model) if model.trim() == CLAUDE_FABLE_5_1_MODEL => {
                Some(CLAUDE_FABLE_5_1_PLAN_MODEL.to_string())
            }
            Some(model) if model.trim() == CLAUDE_FABLE_5_MODEL => {
                Some(CLAUDE_FABLE_5_PLAN_MODEL.to_string())
            }
            Some(model)
                if matches!(
                    model.trim(),
                    CLAUDE_PLAN_MODEL
                        | CLAUDE_PLAN_LEGACY_OPUS_4_8_MODEL
                        | CLAUDE_FABLE_5_1_PLAN_MODEL
                        | CLAUDE_FABLE_5_PLAN_MODEL
                ) =>
            {
                Some(model)
            }
            _ => Some(CLAUDE_PLAN_MODEL.to_string()),
        },
        OPENROUTER_PROVIDER_ID | OPENROUTER_ANTHROPIC_PROVIDER_ID => match model {
            Some(model) if !model.trim().is_empty() => Some(model),
            _ => Some(OPENROUTER_DEFAULT_MODEL.to_string()),
        },
        DEEPSEEK_PROVIDER_ID => match model {
            Some(model) if matches!(model.trim(), DEEPSEEK_DEFAULT_MODEL | DEEPSEEK_PRO_MODEL) => {
                Some(model)
            }
            _ => Some(DEEPSEEK_DEFAULT_MODEL.to_string()),
        },
        META_PROVIDER_ID => match model {
            Some(model) if model.trim() == META_DEFAULT_MODEL => Some(model),
            _ => Some(META_DEFAULT_MODEL.to_string()),
        },
        BASETEN_PROVIDER_ID | BASETEN_ANTHROPIC_PROVIDER_ID => match model {
            Some(model) if model.trim() == BASETEN_DEFAULT_MODEL => Some(model),
            _ => Some(BASETEN_DEFAULT_MODEL.to_string()),
        },
        VERCEL_PROVIDER_ID | VERCEL_ANTHROPIC_PROVIDER_ID => match model {
            Some(model)
                if matches!(
                    model.trim(),
                    VERCEL_DEFAULT_MODEL | VERCEL_GLM_5_2_FAST_MODEL
                ) =>
            {
                Some(model)
            }
            _ => Some(VERCEL_DEFAULT_MODEL.to_string()),
        },
        VERCEL_ANTHROPIC_FAST_PROVIDER_ID => match model {
            Some(model)
                if matches!(
                    model.trim(),
                    VERCEL_DEFAULT_MODEL | VERCEL_GLM_5_2_FAST_MODEL
                ) =>
            {
                Some(model)
            }
            _ => Some(VERCEL_GLM_5_2_FAST_MODEL.to_string()),
        },
        _ => model,
    }
}

/// Return a provider-route context ceiling when it is narrower than the
/// model's shared catalog capability.
pub fn default_model_context_window_for_provider(
    model_provider_id: &str,
    model: &str,
) -> Option<i64> {
    match (canonical_provider_id(model_provider_id), model.trim()) {
        (AMBIENT_PROVIDER_ID | PFTERMINAL_PLAN_PROVIDER_ID, AMBIENT_DEFAULT_MODEL) => {
            Some(AMBIENT_GLM_5_2_CONTEXT_WINDOW)
        }
        (PFTERMINAL_PLAN_ANTHROPIC_PROVIDER_ID, CLAUDE_FABLE_5_1_MODEL | CLAUDE_FABLE_5_MODEL) => {
            Some(PFTERMINAL_PLAN_FABLE_CONTEXT_WINDOW)
        }
        _ => None,
    }
}

/// Return a provider-route output ceiling when it is narrower than the
/// model's shared catalog capability.
pub fn default_model_max_output_tokens_for_provider(
    model_provider_id: &str,
    model: &str,
) -> Option<i64> {
    match (canonical_provider_id(model_provider_id), model.trim()) {
        (PFTERMINAL_PLAN_ANTHROPIC_PROVIDER_ID, CLAUDE_FABLE_5_1_MODEL | CLAUDE_FABLE_5_MODEL) => {
            Some(PFTERMINAL_PLAN_FABLE_MAX_OUTPUT_TOKENS)
        }
        _ => None,
    }
}

fn provider_api_key_vault_instructions() -> String {
    [
        "Run `/providers` and select the matching provider key:",
        "",
        "Providers",
        "  Add or replace provider API keys. Keys are stored in the vault.",
        "",
        "  Search providers",
        "> Provider: Anthropic API Key   Store ANTHROPIC_API_KEY in the vault",
        "  Provider: Ambient API Key     Store AMBIENT_API_KEY in the vault",
        "  Provider: Kimi Code API Key   Store KIMI_API_KEY in the vault",
        "  Provider: Z.AI API Key        Store ZAI_API_KEY in the vault",
        "  Provider: DeepSeek API Key    Store DEEPSEEK_API_KEY in the vault",
        "  Provider: OpenRouter API Key  Store OPENROUTER_API_KEY in the vault",
        "  Provider: Meta API Key        Store MODEL_API_KEY in the vault",
        "  Provider: Baseten API Key     Store BASETEN_API_KEY in the vault",
        "  Provider: Vercel API Key      Store AI_GATEWAY_API_KEY in the vault",
    ]
    .join("\n")
}

const AMAZON_BEDROCK_PROVIDER_NAME: &str = "Amazon Bedrock";
pub const AMAZON_BEDROCK_PROVIDER_ID: &str = "amazon-bedrock";
pub const AMAZON_BEDROCK_GPT_5_5_MODEL_ID: &str = "openai.gpt-5.5";
pub const AMAZON_BEDROCK_GPT_5_4_MODEL_ID: &str = "openai.gpt-5.4";
pub const AMAZON_BEDROCK_GPT_5_6_SOL_MODEL_ID: &str = "openai.gpt-5.6-sol";
pub const AMAZON_BEDROCK_GPT_5_6_TERRA_MODEL_ID: &str = "openai.gpt-5.6-terra";
pub const AMAZON_BEDROCK_GPT_5_6_LUNA_MODEL_ID: &str = "openai.gpt-5.6-luna";
pub const AMAZON_BEDROCK_DEFAULT_BASE_URL: &str =
    "https://bedrock-mantle.us-east-1.api.aws/openai/v1";
const AMAZON_BEDROCK_MANTLE_CLIENT_AGENT_HEADER: &str = "x-amzn-mantle-client-agent";
const AMAZON_BEDROCK_MANTLE_CLIENT_AGENT_VALUE: &str = "codex";
pub const LEGACY_OLLAMA_CHAT_PROVIDER_ID: &str = "ollama-chat";
pub const OLLAMA_CHAT_PROVIDER_REMOVED_ERROR: &str = "`ollama-chat` is no longer supported.\nHow to fix: replace `ollama-chat` with `ollama` in `model_provider`, `oss_provider`, or `--local-provider`.\nMore info: https://github.com/openai/codex/discussions/7782";
const OSS_PROVIDER_NAME: &str = "gpt-oss";
pub const BUILT_IN_MODEL_PROVIDER_NAMES: [&str; 16] = [
    OPENAI_PROVIDER_NAME,
    ANTHROPIC_PROVIDER_NAME,
    CLAUDE_PLAN_PROVIDER_NAME,
    AMBIENT_PROVIDER_NAME,
    KIMI_CODE_PROVIDER_NAME,
    ZAI_PROVIDER_NAME,
    ZAI_ANTHROPIC_PROVIDER_NAME,
    DEEPSEEK_PROVIDER_NAME,
    OPENROUTER_PROVIDER_NAME,
    BASETEN_PROVIDER_NAME,
    BASETEN_ANTHROPIC_PROVIDER_NAME,
    VERCEL_PROVIDER_NAME,
    VERCEL_ANTHROPIC_PROVIDER_NAME,
    VERCEL_ANTHROPIC_FAST_PROVIDER_NAME,
    AMAZON_BEDROCK_PROVIDER_NAME,
    OSS_PROVIDER_NAME,
];

/// Wire protocol that the provider speaks.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum WireApi {
    /// The Responses API exposed by OpenAI at `/v1/responses`.
    #[default]
    Responses,
    /// The OpenAI-compatible Chat Completions API exposed at `/v1/chat/completions`.
    Chat,
    /// The Anthropic-compatible Messages API exposed at `/v1/messages`.
    Anthropic,
}

impl fmt::Display for WireApi {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Responses => "responses",
            Self::Chat => "chat",
            Self::Anthropic => "anthropic",
        };
        f.write_str(value)
    }
}

/// How confidently a provider's text-only `stop` finish reason ends the user's
/// action turn.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ChatStopSemantics {
    /// A text-only `stop` is accepted as the provider's final answer.
    #[default]
    ReliableTerminal,
    /// A text-only `stop` needs a semantic completion decision before the turn
    /// can be considered finished.
    AmbiguousForActionTurns,
}

impl<'de> Deserialize<'de> for WireApi {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "responses" => Ok(Self::Responses),
            "chat" => Ok(Self::Chat),
            "anthropic" | "anthropic_messages" | "anthropic-messages" => Ok(Self::Anthropic),
            _ => Err(serde::de::Error::unknown_variant(
                &value,
                &["responses", "chat", "anthropic"],
            )),
        }
    }
}

/// Typed request and provider-tool budgets for a provider route.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct ProviderRuntimePolicy {
    pub request_body_max_bytes: usize,
    pub retry_request_body_max_bytes: usize,
    pub web_search_max_uses: Option<u32>,
}

impl Default for ProviderRuntimePolicy {
    fn default() -> Self {
        Self {
            request_body_max_bytes: DEFAULT_ANTHROPIC_REQUEST_BODY_MAX_BYTES,
            retry_request_body_max_bytes: DEFAULT_ANTHROPIC_RETRY_BODY_MAX_BYTES,
            web_search_max_uses: None,
        }
    }
}

/// Credential source declared by one runtime model-provider definition.
///
/// Catalog and status consumers use this typed view instead of independently
/// interpreting the provider's authentication fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelProviderCredentialSource<'a> {
    OpenAiAuth,
    EnvironmentApiKey { env_key: &'a str },
    Command,
    Aws,
    None,
}

/// Serializable representation of a provider definition.
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct ModelProviderInfo {
    /// Friendly display name.
    #[serde(default)]
    pub name: String,
    /// Base URL for the provider's OpenAI-compatible API.
    pub base_url: Option<String>,
    /// Environment variable that stores the user's API key for this provider.
    pub env_key: Option<String>,

    /// Optional instructions to help the user get a valid value for the
    /// variable and set it.
    pub env_key_instructions: Option<String>,
    /// Value to use with `Authorization: Bearer <token>` header. Use of this
    /// config is discouraged in favor of `env_key` for security reasons, but
    /// this may be necessary when using this programmatically.
    pub experimental_bearer_token: Option<String>,
    /// Command-backed bearer-token configuration for this provider.
    pub auth: Option<ModelProviderAuthInfo>,
    /// AWS SigV4 auth configuration for this provider.
    pub aws: Option<ModelProviderAwsAuthInfo>,
    /// Which wire protocol this provider expects.
    #[serde(default)]
    pub wire_api: WireApi,
    /// Optional query parameters to append to the base URL.
    pub query_params: Option<HashMap<String, String>>,
    /// Additional HTTP headers to include in requests to this provider where
    /// the (key, value) pairs are the header name and value.
    pub http_headers: Option<HashMap<String, String>>,
    /// Optional HTTP headers to include in requests to this provider where the
    /// (key, value) pairs are the header name and _environment variable_ whose
    /// value should be used. If the environment variable is not set, or the
    /// value is empty, the header will not be included in the request.
    pub env_http_headers: Option<HashMap<String, String>>,
    /// Optional `provider` object to include in Chat Completions request bodies.
    /// This is used by OpenRouter-compatible routes for provider routing
    /// preferences such as `order`, `sort`, `allow_fallbacks`, and
    /// `require_parameters`.
    pub chat_completions_provider: Option<Value>,
    /// Maximum number of times to retry a failed HTTP request to this provider.
    pub request_max_retries: Option<u64>,
    /// Number of times to retry reconnecting a dropped streaming response before failing.
    pub stream_max_retries: Option<u64>,
    /// Idle timeout (in milliseconds) to wait for activity on a streaming response before treating
    /// the connection as lost.
    pub stream_idle_timeout_ms: Option<u64>,
    /// Timeout (in milliseconds) to wait for actionable stream deltas after stream activity
    /// starts. Comment frames and empty data events do not reset this deadline.
    pub stream_actionable_timeout_ms: Option<u64>,
    /// Elapsed stream-attempt duration after which retry policy switches to the long-failure cap.
    pub stream_long_failure_retry_threshold_ms: Option<u64>,
    /// Maximum retry count for long stream failures. Values above one are clamped to one.
    pub stream_long_failure_max_retries: Option<u64>,
    /// Typed request and provider-tool budgets.
    #[serde(default)]
    pub runtime_policy: ProviderRuntimePolicy,
    /// Maximum time (in milliseconds) to wait for a websocket connection attempt before treating
    /// it as failed.
    pub websocket_connect_timeout_ms: Option<u64>,
    /// Does this provider require an OpenAI API Key or ChatGPT login token? If true,
    /// user is presented with login screen on first run, and login preference and token/key
    /// are stored in auth.json. If false (which is the default), login screen is skipped,
    /// and API key (if needed) comes from the "env_key" environment variable.
    #[serde(default)]
    pub requires_openai_auth: bool,
    /// Whether this provider supports the Responses API WebSocket transport.
    #[serde(default)]
    pub supports_websockets: bool,
    /// Whether this provider supports the standalone web-search endpoint.
    #[serde(default)]
    pub supports_standalone_web_search: bool,
}

/// AWS SigV4 auth configuration for a model provider.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct ModelProviderAwsAuthInfo {
    /// AWS profile name to use. When unset, the AWS SDK default chain decides.
    pub profile: Option<String>,
    /// AWS region to use for provider-specific endpoints.
    pub region: Option<String>,
}

impl ModelProviderInfo {
    /// Return the provider's validated credential-source classification.
    pub fn credential_source(&self) -> ModelProviderCredentialSource<'_> {
        if self.requires_openai_auth {
            ModelProviderCredentialSource::OpenAiAuth
        } else if let Some(env_key) = self.env_key.as_deref() {
            ModelProviderCredentialSource::EnvironmentApiKey { env_key }
        } else if self.auth.is_some() {
            ModelProviderCredentialSource::Command
        } else if self.aws.is_some() {
            ModelProviderCredentialSource::Aws
        } else {
            ModelProviderCredentialSource::None
        }
    }

    pub fn validate(&self) -> std::result::Result<(), String> {
        if self.runtime_policy.request_body_max_bytes == 0 {
            return Err("runtime_policy.request_body_max_bytes must be greater than zero".into());
        }
        if self.runtime_policy.retry_request_body_max_bytes == 0
            || self.runtime_policy.retry_request_body_max_bytes
                > self.runtime_policy.request_body_max_bytes
        {
            return Err(
                "runtime_policy.retry_request_body_max_bytes must be nonzero and no greater than request_body_max_bytes"
                    .into(),
            );
        }
        if self.runtime_policy.web_search_max_uses == Some(0) {
            return Err("runtime_policy.web_search_max_uses must be greater than zero".into());
        }
        if let Some(chat_completions_provider) = &self.chat_completions_provider
            && !chat_completions_provider.is_object()
        {
            return Err("chat_completions_provider must be a JSON object".to_string());
        }

        if self.aws.is_some() {
            if self.supports_websockets {
                // TODO(celia-oai): Support AWS SigV4 signing for WebSocket
                // upgrade requests before allowing AWS-authenticated providers
                // to enable Responses-over-WebSocket.
                return Err("provider aws cannot be combined with supports_websockets".to_string());
            }

            let mut conflicts = Vec::new();
            if self.env_key.is_some() {
                conflicts.push("env_key");
            }
            if self.experimental_bearer_token.is_some() {
                conflicts.push("experimental_bearer_token");
            }
            if self.auth.is_some() {
                conflicts.push("auth");
            }
            if self.requires_openai_auth {
                conflicts.push("requires_openai_auth");
            }

            if !conflicts.is_empty() {
                return Err(format!(
                    "provider aws cannot be combined with {}",
                    conflicts.join(", ")
                ));
            }
        }

        let Some(auth) = self.auth.as_ref() else {
            return Ok(());
        };

        if auth.command.trim().is_empty() {
            return Err("provider auth.command must not be empty".to_string());
        }

        let mut conflicts = Vec::new();
        if self.env_key.is_some() {
            conflicts.push("env_key");
        }
        if self.experimental_bearer_token.is_some() {
            conflicts.push("experimental_bearer_token");
        }
        if self.requires_openai_auth {
            conflicts.push("requires_openai_auth");
        }

        if conflicts.is_empty() {
            Ok(())
        } else {
            Err(format!(
                "provider auth cannot be combined with {}",
                conflicts.join(", ")
            ))
        }
    }

    fn build_header_map(&self) -> CodexResult<HeaderMap> {
        let capacity = self.http_headers.as_ref().map_or(0, HashMap::len)
            + self.env_http_headers.as_ref().map_or(0, HashMap::len);
        let mut headers = HeaderMap::with_capacity(capacity);
        if let Some(extra) = &self.http_headers {
            for (k, v) in extra {
                if let (Ok(name), Ok(value)) = (HeaderName::try_from(k), HeaderValue::try_from(v)) {
                    headers.insert(name, value);
                }
            }
        }

        if let Some(env_headers) = &self.env_http_headers {
            for (header, env_var) in env_headers {
                if let Ok(val) = std::env::var(env_var)
                    && !val.trim().is_empty()
                    && let (Ok(name), Ok(value)) =
                        (HeaderName::try_from(header), HeaderValue::try_from(val))
                {
                    headers.insert(name, value);
                }
            }
        }

        Ok(headers)
    }

    pub fn to_api_provider(&self, auth_mode: Option<AuthMode>) -> CodexResult<ApiProvider> {
        let default_base_url = if matches!(
            auth_mode,
            Some(
                AuthMode::Chatgpt
                    | AuthMode::ChatgptAuthTokens
                    | AuthMode::Headers
                    | AuthMode::AgentIdentity
                    | AuthMode::PersonalAccessToken
            )
        ) {
            CHATGPT_CODEX_BASE_URL
        } else {
            "https://api.openai.com/v1"
        };
        let base_url = self
            .base_url
            .clone()
            .unwrap_or_else(|| default_base_url.to_string());

        let headers = self.build_header_map()?;
        let retry = ApiRetryConfig {
            max_attempts: self.request_max_retries(),
            base_delay: Duration::from_millis(200),
            retry_429: self.retries_transient_rate_limits(),
            retry_5xx: true,
            retry_transport: true,
        };

        Ok(ApiProvider {
            name: self.name.clone(),
            base_url,
            query_params: self.query_params.clone(),
            headers,
            retry,
            stream_idle_timeout: self.stream_idle_timeout(),
        })
    }

    /// If `env_key` is Some, returns the API key for this provider if present
    /// (and non-empty) in the environment. If `env_key` is required but
    /// cannot be found, returns an error.
    pub fn api_key(&self) -> CodexResult<Option<String>> {
        match &self.env_key {
            Some(env_key) => {
                let api_key = self
                    .api_key_env_vars()
                    .into_iter()
                    .find_map(|name| std::env::var(name).ok().filter(|v| !v.trim().is_empty()))
                    .ok_or_else(|| {
                        CodexErr::EnvVar(EnvVarError {
                            var: env_key.clone(),
                            instructions: self.env_key_instructions.clone(),
                        })
                    })?;
                Ok(Some(api_key))
            }
            None => Ok(None),
        }
    }

    /// Environment variables accepted for this provider, ordered from the
    /// current public name to compatibility fallbacks.
    pub fn api_key_env_vars(&self) -> Vec<&str> {
        match self.env_key.as_deref() {
            Some(PFTERMINAL_PLAN_API_KEY_ENV_VAR) => vec![
                CORBANU_PLAN_API_KEY_ENV_VAR,
                PFTERMINAL_PLAN_API_KEY_ENV_VAR,
            ],
            Some(env_key) => vec![env_key],
            None => Vec::new(),
        }
    }

    /// Effective maximum number of request retries for this provider.
    pub fn request_max_retries(&self) -> u64 {
        self.request_max_retries
            .unwrap_or(DEFAULT_REQUEST_MAX_RETRIES)
            .min(MAX_REQUEST_MAX_RETRIES)
    }

    /// Effective maximum number of stream reconnection attempts for this provider.
    pub fn stream_max_retries(&self) -> u64 {
        self.stream_max_retries
            .unwrap_or(DEFAULT_STREAM_MAX_RETRIES)
            .min(MAX_STREAM_MAX_RETRIES)
    }

    /// Effective idle timeout for streaming responses.
    pub fn stream_idle_timeout(&self) -> Duration {
        self.stream_idle_timeout_ms
            .map(Duration::from_millis)
            .unwrap_or(Duration::from_millis(DEFAULT_STREAM_IDLE_TIMEOUT_MS))
    }

    /// Effective actionable-silence timeout for streaming responses.
    pub fn stream_actionable_timeout(&self) -> Duration {
        self.stream_actionable_timeout_ms
            .map(Duration::from_millis)
            .unwrap_or(Duration::from_millis(DEFAULT_STREAM_ACTIONABLE_TIMEOUT_MS))
    }

    /// Effective threshold for treating retryable stream errors as long failures.
    pub fn stream_long_failure_retry_threshold(&self) -> Duration {
        self.stream_long_failure_retry_threshold_ms
            .map(Duration::from_millis)
            .unwrap_or(Duration::from_millis(
                DEFAULT_STREAM_LONG_FAILURE_RETRY_THRESHOLD_MS,
            ))
    }

    /// Effective retry cap for long stream failures. This intentionally never exceeds one.
    pub fn stream_long_failure_max_retries(&self) -> u64 {
        self.stream_long_failure_max_retries
            .unwrap_or(DEFAULT_STREAM_LONG_FAILURE_MAX_RETRIES)
            .min(DEFAULT_STREAM_LONG_FAILURE_MAX_RETRIES)
    }

    /// Effective timeout for websocket connect attempts.
    pub fn websocket_connect_timeout(&self) -> Duration {
        self.websocket_connect_timeout_ms
            .map(Duration::from_millis)
            .unwrap_or(Duration::from_millis(DEFAULT_WEBSOCKET_CONNECT_TIMEOUT_MS))
    }

    pub fn create_openai_provider(base_url: Option<String>) -> ModelProviderInfo {
        // An overridden OpenAI-compatible endpoint is only guaranteed to implement the HTTP
        // Responses API. Retain first-party websocket prewarming for the default endpoint while
        // falling back conservatively to SSE for custom endpoints.
        let supports_websockets = base_url.is_none();
        ModelProviderInfo {
            name: OPENAI_PROVIDER_NAME.into(),
            base_url,
            env_key: None,
            env_key_instructions: None,
            experimental_bearer_token: None,
            auth: None,
            aws: None,
            wire_api: WireApi::Responses,
            query_params: None,
            http_headers: Some(
                [(
                    "version".to_string(),
                    OPENAI_CODEX_COMPAT_VERSION.to_string(),
                )]
                .into_iter()
                .collect(),
            ),
            env_http_headers: Some(
                [
                    (
                        "OpenAI-Organization".to_string(),
                        "OPENAI_ORGANIZATION".to_string(),
                    ),
                    ("OpenAI-Project".to_string(), "OPENAI_PROJECT".to_string()),
                ]
                .into_iter()
                .collect(),
            ),
            // Use global defaults for retry/timeout unless overridden in config.toml.
            chat_completions_provider: None,
            request_max_retries: None,
            stream_max_retries: None,
            stream_idle_timeout_ms: None,
            stream_actionable_timeout_ms: None,
            stream_long_failure_retry_threshold_ms: None,
            stream_long_failure_max_retries: None,
            runtime_policy: Default::default(),
            websocket_connect_timeout_ms: None,
            requires_openai_auth: true,
            supports_websockets,
            supports_standalone_web_search: true,
        }
    }

    pub fn create_anthropic_provider() -> ModelProviderInfo {
        ModelProviderInfo {
            name: ANTHROPIC_PROVIDER_NAME.into(),
            base_url: Some(ANTHROPIC_BASE_URL.into()),
            env_key: Some(ANTHROPIC_API_KEY_ENV_VAR.into()),
            env_key_instructions: Some(provider_api_key_vault_instructions()),
            experimental_bearer_token: None,
            auth: None,
            aws: None,
            wire_api: WireApi::Anthropic,
            query_params: None,
            http_headers: None,
            env_http_headers: None,
            chat_completions_provider: None,
            request_max_retries: None,
            stream_max_retries: None,
            stream_idle_timeout_ms: None,
            stream_actionable_timeout_ms: None,
            stream_long_failure_retry_threshold_ms: None,
            stream_long_failure_max_retries: None,
            runtime_policy: Default::default(),
            websocket_connect_timeout_ms: None,
            requires_openai_auth: false,
            supports_websockets: false,
            supports_standalone_web_search: false,
        }
    }

    pub fn create_claude_plan_provider() -> ModelProviderInfo {
        ModelProviderInfo {
            name: CLAUDE_PLAN_PROVIDER_NAME.into(),
            base_url: Some(ANTHROPIC_BASE_URL.into()),
            env_key: None,
            env_key_instructions: None,
            experimental_bearer_token: None,
            auth: Some(ModelProviderAuthInfo {
                command: CORBANU_PROVIDER_AUTH_COMMAND.to_string(),
                args: vec!["internal-claude-oauth-token".to_string()],
                timeout_ms: claude_provider_auth_timeout_ms(),
                // The selected Claude source has its own revision marker and
                // 401 recovery path. Proactive reruns add no freshness while
                // making long tool-driven turns contend with unrelated vault
                // readers for the same encrypted store.
                refresh_interval_ms: 0,
                cwd: claude_provider_auth_cwd(),
            }),
            aws: None,
            wire_api: WireApi::Anthropic,
            query_params: None,
            http_headers: Some(HashMap::from([(
                "anthropic-beta".to_string(),
                "claude-code-20250219,oauth-2025-04-20".to_string(),
            )])),
            env_http_headers: None,
            chat_completions_provider: None,
            request_max_retries: None,
            stream_max_retries: None,
            stream_idle_timeout_ms: None,
            stream_actionable_timeout_ms: None,
            stream_long_failure_retry_threshold_ms: None,
            stream_long_failure_max_retries: None,
            runtime_policy: Default::default(),
            websocket_connect_timeout_ms: None,
            requires_openai_auth: false,
            supports_websockets: false,
            supports_standalone_web_search: false,
        }
    }

    pub fn create_ambient_provider() -> ModelProviderInfo {
        ModelProviderInfo {
            name: AMBIENT_PROVIDER_NAME.into(),
            base_url: Some(AMBIENT_BASE_URL.into()),
            env_key: Some(AMBIENT_API_KEY_ENV_VAR.into()),
            env_key_instructions: Some(provider_api_key_vault_instructions()),
            experimental_bearer_token: None,
            auth: None,
            aws: None,
            wire_api: WireApi::Chat,
            query_params: None,
            http_headers: None,
            env_http_headers: None,
            chat_completions_provider: None,
            request_max_retries: None,
            stream_max_retries: None,
            stream_idle_timeout_ms: None,
            stream_actionable_timeout_ms: None,
            stream_long_failure_retry_threshold_ms: None,
            stream_long_failure_max_retries: None,
            runtime_policy: Default::default(),
            websocket_connect_timeout_ms: None,
            requires_openai_auth: false,
            supports_websockets: false,
            supports_standalone_web_search: false,
        }
    }

    pub fn create_pfterminal_plan_provider() -> ModelProviderInfo {
        ModelProviderInfo {
            name: PLAN_NAME.into(),
            base_url: Some(
                std::env::var("PFTERMINAL_PLAN_BASE_URL")
                    .unwrap_or_else(|_| PFTERMINAL_PLAN_DEFAULT_BASE_URL.to_string()),
            ),
            env_key: Some(PFTERMINAL_PLAN_API_KEY_ENV_VAR.into()),
            env_key_instructions: Some(provider_api_key_vault_instructions()),
            experimental_bearer_token: None,
            auth: None,
            aws: None,
            wire_api: WireApi::Chat,
            query_params: None,
            http_headers: None,
            env_http_headers: None,
            chat_completions_provider: None,
            request_max_retries: None,
            stream_max_retries: None,
            stream_idle_timeout_ms: None,
            stream_actionable_timeout_ms: None,
            stream_long_failure_retry_threshold_ms: None,
            stream_long_failure_max_retries: None,
            runtime_policy: Default::default(),
            websocket_connect_timeout_ms: None,
            requires_openai_auth: false,
            supports_websockets: false,
            supports_standalone_web_search: false,
        }
    }

    /// Anthropic-wire sibling of the Corbanu Plan provider: same gateway and
    /// customer key, serving the plan's non-private Fable routes on
    /// `/v1/messages`.
    pub fn create_pfterminal_plan_anthropic_provider() -> ModelProviderInfo {
        ModelProviderInfo {
            wire_api: WireApi::Anthropic,
            name: PLAN_ANTHROPIC_NAME.into(),
            ..Self::create_pfterminal_plan_provider()
        }
    }

    pub fn create_kimi_code_provider() -> ModelProviderInfo {
        ModelProviderInfo {
            name: KIMI_CODE_PROVIDER_NAME.into(),
            base_url: Some(KIMI_CODE_BASE_URL.into()),
            env_key: Some(KIMI_CODE_API_KEY_ENV_VAR.into()),
            env_key_instructions: Some(provider_api_key_vault_instructions()),
            experimental_bearer_token: None,
            auth: None,
            aws: None,
            wire_api: WireApi::Chat,
            query_params: None,
            http_headers: None,
            env_http_headers: None,
            chat_completions_provider: None,
            request_max_retries: None,
            stream_max_retries: None,
            stream_idle_timeout_ms: None,
            stream_actionable_timeout_ms: None,
            stream_long_failure_retry_threshold_ms: None,
            stream_long_failure_max_retries: None,
            runtime_policy: Default::default(),
            websocket_connect_timeout_ms: None,
            requires_openai_auth: false,
            supports_websockets: false,
            supports_standalone_web_search: false,
        }
    }

    pub fn create_zai_provider() -> ModelProviderInfo {
        ModelProviderInfo {
            name: ZAI_PROVIDER_NAME.into(),
            base_url: Some(ZAI_BASE_URL.into()),
            env_key: Some(ZAI_API_KEY_ENV_VAR.into()),
            env_key_instructions: Some(provider_api_key_vault_instructions()),
            experimental_bearer_token: None,
            auth: None,
            aws: None,
            wire_api: WireApi::Chat,
            query_params: None,
            http_headers: None,
            env_http_headers: None,
            chat_completions_provider: None,
            request_max_retries: None,
            stream_max_retries: None,
            stream_idle_timeout_ms: None,
            stream_actionable_timeout_ms: None,
            stream_long_failure_retry_threshold_ms: None,
            stream_long_failure_max_retries: None,
            runtime_policy: Default::default(),
            websocket_connect_timeout_ms: None,
            requires_openai_auth: false,
            supports_websockets: false,
            supports_standalone_web_search: false,
        }
    }

    pub fn create_zai_anthropic_provider() -> ModelProviderInfo {
        ModelProviderInfo {
            name: ZAI_ANTHROPIC_PROVIDER_NAME.into(),
            base_url: Some(ZAI_ANTHROPIC_BASE_URL.into()),
            env_key: Some(ZAI_API_KEY_ENV_VAR.into()),
            env_key_instructions: Some(provider_api_key_vault_instructions()),
            experimental_bearer_token: None,
            auth: None,
            aws: None,
            wire_api: WireApi::Anthropic,
            query_params: None,
            http_headers: None,
            env_http_headers: None,
            chat_completions_provider: None,
            request_max_retries: None,
            stream_max_retries: None,
            stream_idle_timeout_ms: None,
            stream_actionable_timeout_ms: None,
            stream_long_failure_retry_threshold_ms: None,
            stream_long_failure_max_retries: None,
            runtime_policy: Default::default(),
            websocket_connect_timeout_ms: None,
            requires_openai_auth: false,
            supports_websockets: false,
            supports_standalone_web_search: false,
        }
    }

    pub fn create_openrouter_provider() -> ModelProviderInfo {
        ModelProviderInfo {
            name: OPENROUTER_PROVIDER_NAME.into(),
            base_url: Some(OPENROUTER_BASE_URL.into()),
            env_key: Some(OPENROUTER_API_KEY_ENV_VAR.into()),
            env_key_instructions: Some(provider_api_key_vault_instructions()),
            experimental_bearer_token: None,
            auth: None,
            aws: None,
            wire_api: WireApi::Chat,
            query_params: None,
            http_headers: None,
            env_http_headers: None,
            chat_completions_provider: None,
            request_max_retries: None,
            stream_max_retries: None,
            stream_idle_timeout_ms: None,
            stream_actionable_timeout_ms: None,
            stream_long_failure_retry_threshold_ms: None,
            stream_long_failure_max_retries: None,
            runtime_policy: Default::default(),
            websocket_connect_timeout_ms: None,
            requires_openai_auth: false,
            supports_websockets: false,
            supports_standalone_web_search: false,
        }
    }

    pub fn create_deepseek_provider() -> ModelProviderInfo {
        ModelProviderInfo {
            name: DEEPSEEK_PROVIDER_NAME.into(),
            base_url: Some(DEEPSEEK_BASE_URL.into()),
            env_key: Some(DEEPSEEK_API_KEY_ENV_VAR.into()),
            env_key_instructions: Some(provider_api_key_vault_instructions()),
            experimental_bearer_token: None,
            auth: None,
            aws: None,
            wire_api: WireApi::Responses,
            query_params: None,
            http_headers: None,
            env_http_headers: None,
            chat_completions_provider: None,
            request_max_retries: None,
            stream_max_retries: None,
            stream_idle_timeout_ms: None,
            stream_actionable_timeout_ms: None,
            stream_long_failure_retry_threshold_ms: None,
            stream_long_failure_max_retries: None,
            runtime_policy: Default::default(),
            websocket_connect_timeout_ms: None,
            requires_openai_auth: false,
            supports_websockets: false,
            supports_standalone_web_search: false,
        }
    }

    pub fn create_openrouter_anthropic_provider() -> ModelProviderInfo {
        ModelProviderInfo {
            name: OPENROUTER_ANTHROPIC_PROVIDER_NAME.into(),
            base_url: Some(OPENROUTER_BASE_URL.into()),
            env_key: Some(OPENROUTER_API_KEY_ENV_VAR.into()),
            env_key_instructions: Some(provider_api_key_vault_instructions()),
            experimental_bearer_token: None,
            auth: None,
            aws: None,
            wire_api: WireApi::Anthropic,
            query_params: None,
            http_headers: None,
            env_http_headers: None,
            chat_completions_provider: None,
            request_max_retries: None,
            stream_max_retries: None,
            stream_idle_timeout_ms: None,
            stream_actionable_timeout_ms: None,
            stream_long_failure_retry_threshold_ms: None,
            stream_long_failure_max_retries: None,
            runtime_policy: Default::default(),
            websocket_connect_timeout_ms: None,
            requires_openai_auth: false,
            supports_websockets: false,
            supports_standalone_web_search: false,
        }
    }

    pub fn create_meta_provider() -> ModelProviderInfo {
        ModelProviderInfo {
            name: META_PROVIDER_NAME.into(),
            base_url: Some(META_BASE_URL.into()),
            env_key: Some(META_API_KEY_ENV_VAR.into()),
            env_key_instructions: Some(provider_api_key_vault_instructions()),
            experimental_bearer_token: None,
            auth: None,
            aws: None,
            wire_api: WireApi::Responses,
            query_params: None,
            http_headers: None,
            env_http_headers: None,
            chat_completions_provider: None,
            request_max_retries: None,
            stream_max_retries: None,
            stream_idle_timeout_ms: None,
            stream_actionable_timeout_ms: None,
            stream_long_failure_retry_threshold_ms: None,
            stream_long_failure_max_retries: None,
            runtime_policy: Default::default(),
            websocket_connect_timeout_ms: None,
            requires_openai_auth: false,
            supports_websockets: false,
            supports_standalone_web_search: false,
        }
    }

    pub fn create_baseten_provider() -> ModelProviderInfo {
        ModelProviderInfo {
            name: BASETEN_PROVIDER_NAME.into(),
            base_url: Some(BASETEN_BASE_URL.into()),
            env_key: Some(BASETEN_API_KEY_ENV_VAR.into()),
            env_key_instructions: Some(provider_api_key_vault_instructions()),
            experimental_bearer_token: None,
            auth: None,
            aws: None,
            wire_api: WireApi::Chat,
            query_params: None,
            http_headers: None,
            env_http_headers: None,
            chat_completions_provider: None,
            request_max_retries: None,
            stream_max_retries: None,
            stream_idle_timeout_ms: None,
            stream_actionable_timeout_ms: None,
            stream_long_failure_retry_threshold_ms: None,
            stream_long_failure_max_retries: None,
            runtime_policy: Default::default(),
            websocket_connect_timeout_ms: None,
            requires_openai_auth: false,
            supports_websockets: false,
            supports_standalone_web_search: false,
        }
    }

    pub fn create_baseten_anthropic_provider() -> ModelProviderInfo {
        ModelProviderInfo {
            name: BASETEN_ANTHROPIC_PROVIDER_NAME.into(),
            base_url: Some(BASETEN_BASE_URL.into()),
            env_key: Some(BASETEN_API_KEY_ENV_VAR.into()),
            env_key_instructions: Some(provider_api_key_vault_instructions()),
            experimental_bearer_token: None,
            auth: None,
            aws: None,
            wire_api: WireApi::Anthropic,
            query_params: None,
            http_headers: None,
            env_http_headers: None,
            chat_completions_provider: None,
            request_max_retries: None,
            stream_max_retries: None,
            stream_idle_timeout_ms: None,
            stream_actionable_timeout_ms: None,
            stream_long_failure_retry_threshold_ms: None,
            stream_long_failure_max_retries: None,
            runtime_policy: Default::default(),
            websocket_connect_timeout_ms: None,
            requires_openai_auth: false,
            supports_websockets: false,
            supports_standalone_web_search: false,
        }
    }

    pub fn create_vercel_provider() -> ModelProviderInfo {
        ModelProviderInfo {
            name: VERCEL_PROVIDER_NAME.into(),
            base_url: Some(VERCEL_BASE_URL.into()),
            env_key: Some(VERCEL_API_KEY_ENV_VAR.into()),
            env_key_instructions: Some(provider_api_key_vault_instructions()),
            experimental_bearer_token: None,
            auth: None,
            aws: None,
            wire_api: WireApi::Responses,
            query_params: None,
            http_headers: None,
            env_http_headers: None,
            chat_completions_provider: None,
            request_max_retries: None,
            stream_max_retries: None,
            stream_idle_timeout_ms: None,
            stream_actionable_timeout_ms: None,
            stream_long_failure_retry_threshold_ms: None,
            stream_long_failure_max_retries: None,
            runtime_policy: Default::default(),
            websocket_connect_timeout_ms: None,
            requires_openai_auth: false,
            supports_websockets: false,
            supports_standalone_web_search: false,
        }
    }

    pub fn create_vercel_anthropic_provider() -> ModelProviderInfo {
        ModelProviderInfo {
            name: VERCEL_ANTHROPIC_PROVIDER_NAME.into(),
            base_url: Some(VERCEL_BASE_URL.into()),
            env_key: Some(VERCEL_API_KEY_ENV_VAR.into()),
            env_key_instructions: Some(provider_api_key_vault_instructions()),
            experimental_bearer_token: None,
            auth: None,
            aws: None,
            wire_api: WireApi::Anthropic,
            query_params: None,
            http_headers: None,
            env_http_headers: None,
            chat_completions_provider: None,
            request_max_retries: None,
            stream_max_retries: None,
            stream_idle_timeout_ms: None,
            stream_actionable_timeout_ms: None,
            stream_long_failure_retry_threshold_ms: None,
            stream_long_failure_max_retries: None,
            runtime_policy: Default::default(),
            websocket_connect_timeout_ms: None,
            requires_openai_auth: false,
            supports_websockets: false,
            supports_standalone_web_search: false,
        }
    }

    pub fn create_vercel_anthropic_fast_provider() -> ModelProviderInfo {
        ModelProviderInfo {
            name: VERCEL_ANTHROPIC_FAST_PROVIDER_NAME.into(),
            base_url: Some(VERCEL_BASE_URL.into()),
            env_key: Some(VERCEL_API_KEY_ENV_VAR.into()),
            env_key_instructions: Some(provider_api_key_vault_instructions()),
            experimental_bearer_token: None,
            auth: None,
            aws: None,
            wire_api: WireApi::Anthropic,
            query_params: None,
            http_headers: None,
            env_http_headers: None,
            chat_completions_provider: None,
            request_max_retries: None,
            stream_max_retries: None,
            stream_idle_timeout_ms: None,
            stream_actionable_timeout_ms: None,
            stream_long_failure_retry_threshold_ms: None,
            stream_long_failure_max_retries: None,
            runtime_policy: Default::default(),
            websocket_connect_timeout_ms: None,
            requires_openai_auth: false,
            supports_websockets: false,
            supports_standalone_web_search: false,
        }
    }

    pub fn create_amazon_bedrock_provider(
        aws: Option<ModelProviderAwsAuthInfo>,
    ) -> ModelProviderInfo {
        ModelProviderInfo {
            name: AMAZON_BEDROCK_PROVIDER_NAME.into(),
            // The runtime provider derives the regional Mantle endpoint when
            // this is unset. A configured value is therefore unambiguously an
            // endpoint override.
            base_url: None,
            env_key: None,
            env_key_instructions: None,
            experimental_bearer_token: None,
            auth: None,
            aws: Some(aws.unwrap_or(ModelProviderAwsAuthInfo {
                profile: None,
                region: None,
            })),
            wire_api: WireApi::Responses,
            query_params: None,
            http_headers: Some(HashMap::from([(
                AMAZON_BEDROCK_MANTLE_CLIENT_AGENT_HEADER.to_string(),
                AMAZON_BEDROCK_MANTLE_CLIENT_AGENT_VALUE.to_string(),
            )])),
            env_http_headers: None,
            chat_completions_provider: None,
            request_max_retries: None,
            stream_max_retries: None,
            stream_idle_timeout_ms: None,
            stream_actionable_timeout_ms: None,
            stream_long_failure_retry_threshold_ms: None,
            stream_long_failure_max_retries: None,
            runtime_policy: Default::default(),
            websocket_connect_timeout_ms: None,
            requires_openai_auth: false,
            supports_websockets: false,
            supports_standalone_web_search: false,
        }
    }

    pub fn is_openai(&self) -> bool {
        self.name == OPENAI_PROVIDER_NAME
    }

    pub fn is_anthropic(&self) -> bool {
        self.name == ANTHROPIC_PROVIDER_NAME
    }

    pub fn is_claude_plan(&self) -> bool {
        self.name == CLAUDE_PLAN_PROVIDER_NAME
    }

    /// Direct Anthropic API keys are sent as `x-api-key`; other built-in
    /// provider API keys remain Bearer tokens.
    pub fn api_key_header_name(&self) -> Option<&'static str> {
        (self.env_key.as_deref() == Some(ANTHROPIC_API_KEY_ENV_VAR)).then_some("x-api-key")
    }

    pub fn is_ambient(&self) -> bool {
        self.name == AMBIENT_PROVIDER_NAME
    }

    pub fn is_pfterminal_plan(&self) -> bool {
        self.name == PLAN_NAME
    }

    pub fn is_kimi_code(&self) -> bool {
        self.name == KIMI_CODE_PROVIDER_NAME
    }

    pub fn chat_stop_semantics(&self) -> ChatStopSemantics {
        if self.is_kimi_code() {
            ChatStopSemantics::AmbiguousForActionTurns
        } else {
            ChatStopSemantics::ReliableTerminal
        }
    }

    pub fn is_zai(&self) -> bool {
        self.name == ZAI_PROVIDER_NAME
    }

    pub fn is_openrouter(&self) -> bool {
        self.name == OPENROUTER_PROVIDER_NAME
    }

    pub fn is_deepseek(&self) -> bool {
        self.name == DEEPSEEK_PROVIDER_NAME
    }

    pub fn is_meta(&self) -> bool {
        self.name == META_PROVIDER_NAME
    }

    pub fn is_baseten(&self) -> bool {
        self.name == BASETEN_PROVIDER_NAME
    }

    pub fn is_vercel(&self) -> bool {
        self.name == VERCEL_PROVIDER_NAME
    }

    /// Detect Vercel AI Gateway by endpoint so renamed custom providers retain
    /// the gateway's wire behavior without relying on display-name conventions.
    pub fn is_vercel_gateway(&self) -> bool {
        self.base_url
            .as_deref()
            .is_some_and(is_vercel_gateway_base_url)
    }

    pub fn uses_openai_actor_authorization(&self) -> bool {
        !self.requires_openai_auth
            && self.http_headers.as_ref().is_some_and(|headers| {
                headers.iter().any(|(name, value)| {
                    name.eq_ignore_ascii_case(OPENAI_ACTOR_AUTHORIZATION_HEADER)
                        && !value.trim().is_empty()
                })
            })
    }

    pub fn is_amazon_bedrock(&self) -> bool {
        self.name == AMAZON_BEDROCK_PROVIDER_NAME
    }

    fn retries_transient_rate_limits(&self) -> bool {
        self.is_zai()
    }

    pub fn supports_remote_compaction(&self) -> bool {
        self.is_openai() || is_azure_responses_provider(&self.name, self.base_url.as_deref())
    }

    pub fn has_command_auth(&self) -> bool {
        self.auth.is_some()
    }
}

pub const DEFAULT_LMSTUDIO_PORT: u16 = 1234;
pub const DEFAULT_OLLAMA_PORT: u16 = 11434;

pub const LMSTUDIO_OSS_PROVIDER_ID: &str = "lmstudio";
pub const OLLAMA_OSS_PROVIDER_ID: &str = "ollama";

/// Built-in default provider list.
pub fn built_in_model_providers(
    openai_base_url: Option<String>,
) -> HashMap<String, ModelProviderInfo> {
    use ModelProviderInfo as P;
    let openai_provider = P::create_openai_provider(openai_base_url);
    let anthropic_provider = P::create_anthropic_provider();
    let claude_plan_provider = P::create_claude_plan_provider();
    let ambient_provider = P::create_ambient_provider();
    let pfterminal_plan_provider = P::create_pfterminal_plan_provider();
    let kimi_code_provider = P::create_kimi_code_provider();
    let zai_provider = P::create_zai_provider();
    let zai_anthropic_provider = P::create_zai_anthropic_provider();
    let openrouter_provider = P::create_openrouter_provider();
    let openrouter_anthropic_provider = P::create_openrouter_anthropic_provider();
    let deepseek_provider = P::create_deepseek_provider();
    let meta_provider = P::create_meta_provider();
    let baseten_provider = P::create_baseten_provider();
    let baseten_anthropic_provider = P::create_baseten_anthropic_provider();
    let vercel_provider = P::create_vercel_provider();
    let vercel_anthropic_provider = P::create_vercel_anthropic_provider();
    let vercel_anthropic_fast_provider = P::create_vercel_anthropic_fast_provider();
    let amazon_bedrock_provider = P::create_amazon_bedrock_provider(/*aws*/ None);
    let pfterminal_plan_anthropic_provider = P::create_pfterminal_plan_anthropic_provider();

    // Corbanu Terminal bundles the first-party OpenAI provider, the local OSS
    // providers, and the curated third-party coding providers exposed in the
    // login/model picker UX. Users can still add more providers in config.toml.
    [
        (ANTHROPIC_PROVIDER_ID, anthropic_provider),
        (CLAUDE_PLAN_PROVIDER_ID, claude_plan_provider),
        (AMBIENT_PROVIDER_ID, ambient_provider),
        (PFTERMINAL_PLAN_PROVIDER_ID, pfterminal_plan_provider),
        (
            PFTERMINAL_PLAN_ANTHROPIC_PROVIDER_ID,
            pfterminal_plan_anthropic_provider,
        ),
        (KIMI_CODE_PROVIDER_ID, kimi_code_provider),
        (ZAI_PROVIDER_ID, zai_provider),
        (ZAI_ANTHROPIC_PROVIDER_ID, zai_anthropic_provider),
        (OPENROUTER_PROVIDER_ID, openrouter_provider),
        (
            OPENROUTER_ANTHROPIC_PROVIDER_ID,
            openrouter_anthropic_provider,
        ),
        (DEEPSEEK_PROVIDER_ID, deepseek_provider),
        (META_PROVIDER_ID, meta_provider),
        (BASETEN_PROVIDER_ID, baseten_provider),
        (BASETEN_ANTHROPIC_PROVIDER_ID, baseten_anthropic_provider),
        (VERCEL_PROVIDER_ID, vercel_provider),
        (VERCEL_ANTHROPIC_PROVIDER_ID, vercel_anthropic_provider),
        (
            VERCEL_ANTHROPIC_FAST_PROVIDER_ID,
            vercel_anthropic_fast_provider,
        ),
        (OPENAI_PROVIDER_ID, openai_provider),
        (AMAZON_BEDROCK_PROVIDER_ID, amazon_bedrock_provider),
        (
            OLLAMA_OSS_PROVIDER_ID,
            create_oss_provider(DEFAULT_OLLAMA_PORT, WireApi::Responses),
        ),
        (
            LMSTUDIO_OSS_PROVIDER_ID,
            create_oss_provider(DEFAULT_LMSTUDIO_PORT, WireApi::Responses),
        ),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v))
    .collect()
}

/// Merge configured providers into the built-in provider catalog.
///
/// Configured providers extend the built-in set. Built-in providers are not
/// generally overridable, but the built-in Amazon Bedrock provider allows the
/// user to customize its endpoint, authentication, headers, and AWS settings.
pub fn merge_configured_model_providers(
    mut model_providers: HashMap<String, ModelProviderInfo>,
    configured_model_providers: HashMap<String, ModelProviderInfo>,
) -> Result<HashMap<String, ModelProviderInfo>, String> {
    for (key, mut provider) in configured_model_providers {
        if key == AMAZON_BEDROCK_PROVIDER_ID {
            let base_url_override = provider.base_url.take();
            let auth_override = provider.auth.take();
            let aws_override = provider.aws.take();
            let http_headers_override = provider.http_headers.take();
            if provider != ModelProviderInfo::default() {
                return Err(format!(
                    "model_providers.{AMAZON_BEDROCK_PROVIDER_ID} only supports changing \
`base_url`, `auth`, `http_headers`, `aws.profile`, and `aws.region`; other non-default \
provider fields are not supported"
                ));
            }

            if let Some(built_in_provider) = model_providers.get_mut(AMAZON_BEDROCK_PROVIDER_ID) {
                built_in_provider.base_url = base_url_override;
                built_in_provider.auth = auth_override;
                if let Some(aws_override) = aws_override {
                    built_in_provider.aws = Some(aws_override);
                }
                if let Some(http_headers_override) = http_headers_override {
                    built_in_provider
                        .http_headers
                        .get_or_insert_default()
                        .extend(http_headers_override);
                }
            }
        } else if let Some(built_in_provider) = model_providers.get_mut(&key) {
            apply_transport_overrides(built_in_provider, provider);
        } else {
            model_providers.insert(key, provider);
        }
    }

    Ok(model_providers)
}

fn apply_transport_overrides(
    built_in_provider: &mut ModelProviderInfo,
    configured_provider: ModelProviderInfo,
) {
    if configured_provider.request_max_retries.is_some() {
        built_in_provider.request_max_retries = configured_provider.request_max_retries;
    }
    if configured_provider.stream_max_retries.is_some() {
        built_in_provider.stream_max_retries = configured_provider.stream_max_retries;
    }
    if configured_provider.stream_idle_timeout_ms.is_some() {
        built_in_provider.stream_idle_timeout_ms = configured_provider.stream_idle_timeout_ms;
    }
    if configured_provider.stream_actionable_timeout_ms.is_some() {
        built_in_provider.stream_actionable_timeout_ms =
            configured_provider.stream_actionable_timeout_ms;
    }
    if configured_provider
        .stream_long_failure_retry_threshold_ms
        .is_some()
    {
        built_in_provider.stream_long_failure_retry_threshold_ms =
            configured_provider.stream_long_failure_retry_threshold_ms;
    }
    if configured_provider
        .stream_long_failure_max_retries
        .is_some()
    {
        built_in_provider.stream_long_failure_max_retries =
            configured_provider.stream_long_failure_max_retries;
    }
    if configured_provider.websocket_connect_timeout_ms.is_some() {
        built_in_provider.websocket_connect_timeout_ms =
            configured_provider.websocket_connect_timeout_ms;
    }
}

pub fn create_oss_provider(default_provider_port: u16, wire_api: WireApi) -> ModelProviderInfo {
    // These CODEX_OSS_ environment variables are experimental: we may
    // switch to reading values from config.toml instead.
    let default_codex_oss_base_url = format!(
        "http://localhost:{codex_oss_port}/v1",
        codex_oss_port = std::env::var("CODEX_OSS_PORT")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .and_then(|value| value.parse::<u16>().ok())
            .unwrap_or(default_provider_port)
    );

    let codex_oss_base_url = std::env::var("CODEX_OSS_BASE_URL")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .unwrap_or(default_codex_oss_base_url);
    create_oss_provider_with_base_url(&codex_oss_base_url, wire_api)
}

pub fn create_oss_provider_with_base_url(base_url: &str, wire_api: WireApi) -> ModelProviderInfo {
    ModelProviderInfo {
        name: OSS_PROVIDER_NAME.into(),
        base_url: Some(base_url.into()),
        env_key: None,
        env_key_instructions: None,
        experimental_bearer_token: None,
        auth: None,
        aws: None,
        wire_api,
        query_params: None,
        http_headers: None,
        env_http_headers: None,
        chat_completions_provider: None,
        request_max_retries: None,
        stream_max_retries: None,
        stream_idle_timeout_ms: None,
        stream_actionable_timeout_ms: None,
        stream_long_failure_retry_threshold_ms: None,
        stream_long_failure_max_retries: None,
        runtime_policy: Default::default(),
        websocket_connect_timeout_ms: None,
        requires_openai_auth: false,
        supports_websockets: false,
        supports_standalone_web_search: false,
    }
}

#[cfg(test)]
#[path = "model_provider_info_tests.rs"]
mod tests;
