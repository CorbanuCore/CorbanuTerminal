//! How a runtime is paid for, and roughly what it costs.
//!
//! Agents allocating work need two facts the model catalog does not carry: how a
//! runtime is billed, and how expensive it is relative to its peers. Without them
//! an agent picks a runtime from pretraining priors about model names, which
//! silently fails for same-family tiers and for anything added after training.
//!
//! Prices are list rates in USD per 1M tokens as of 2026-07-26 and are guidance
//! for allocation, not billing truth.

use crate::ModelProviderInfo;

/// How the user pays for a runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BillingClass {
    /// Covered by a subscription the user already pays for.
    ///
    /// Not free: plan capacity is a finite shared pool, models draw against it at
    /// different rates, and exhausting it can overflow to metered billing on the
    /// same provider. Prefer plan routes, but still prefer the cheapest plan
    /// runtime that can do the job.
    Plan,
    /// User-owned rented compute, already paid for by the hour.
    Local,
    /// Pay-per-token API key. Every token is new spend.
    Metered,
    /// Provider shape did not match a known billing arrangement.
    Unknown,
}

impl BillingClass {
    pub fn as_str(self) -> &'static str {
        match self {
            BillingClass::Plan => "plan",
            BillingClass::Local => "local",
            BillingClass::Metered => "metered",
            BillingClass::Unknown => "unknown",
        }
    }
}

/// Rough capability class, for matching model strength to task difficulty.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityTier {
    /// Hardest reasoning, research-grade work. Most expensive.
    Frontier,
    /// Competent general engineering. The default choice.
    Balanced,
    /// High-volume, mechanical, or well-specified work.
    Fast,
}

impl CapabilityTier {
    pub fn as_str(self) -> &'static str {
        match self {
            CapabilityTier::Frontier => "frontier",
            CapabilityTier::Balanced => "balanced",
            CapabilityTier::Fast => "fast",
        }
    }
}

/// Allocation guidance for one runtime.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ModelEconomics {
    pub tier: CapabilityTier,
    /// USD per 1M input tokens. `None` for plan and local routes.
    pub input_usd_per_mtok: Option<f64>,
    /// USD per 1M output tokens. `None` for plan and local routes.
    pub output_usd_per_mtok: Option<f64>,
    /// How fast this runtime drains a subscription pool, relative to a
    /// baseline frontier plan session (1.0). `None` when not plan-routed.
    pub plan_burn_weight: Option<f64>,
}

const fn metered(tier: CapabilityTier, input: f64, output: f64) -> ModelEconomics {
    ModelEconomics {
        tier,
        input_usd_per_mtok: Some(input),
        output_usd_per_mtok: Some(output),
        plan_burn_weight: None,
    }
}

const fn plan(tier: CapabilityTier, burn: f64) -> ModelEconomics {
    ModelEconomics {
        tier,
        input_usd_per_mtok: None,
        output_usd_per_mtok: None,
        plan_burn_weight: Some(burn),
    }
}

/// Keyed by exact `(provider_id, model)`. The same model slug can be plan-routed
/// on one provider and metered on another; that pairing is the whole point.
const ECONOMICS: &[(&str, &str, ModelEconomics)] = &[
    // Subscription-backed. Burn weights are relative to an Opus-class session:
    // Fable draws roughly double, and is additionally capped at ~50% of the pool.
    (
        "claude-plan",
        "claude-fable-5-plan",
        plan(CapabilityTier::Frontier, 2.0),
    ),
    (
        "claude-plan",
        "claude-opus-5-plan",
        plan(CapabilityTier::Frontier, 1.0),
    ),
    (
        "claude-plan",
        "claude-opus-4-8-plan",
        plan(CapabilityTier::Balanced, 1.0),
    ),
    ("openai", "gpt-5.6-sol", plan(CapabilityTier::Frontier, 1.0)),
    (
        "openai",
        "gpt-5.6-terra",
        plan(CapabilityTier::Balanced, 0.5),
    ),
    ("openai", "gpt-5.6-luna", plan(CapabilityTier::Fast, 0.2)),
    ("openai", "gpt-5.5", plan(CapabilityTier::Frontier, 1.0)),
    // Metered.
    (
        "anthropic",
        "claude-fable-5",
        metered(CapabilityTier::Frontier, 10.0, 50.0),
    ),
    (
        "anthropic",
        "claude-opus-5",
        metered(CapabilityTier::Frontier, 5.0, 25.0),
    ),
    (
        "anthropic",
        "claude-fable-5-plan",
        metered(CapabilityTier::Frontier, 10.0, 50.0),
    ),
    (
        "kimi-code",
        "k3",
        metered(CapabilityTier::Frontier, 3.0, 15.0),
    ),
    (
        "openrouter",
        "moonshotai/kimi-k3",
        metered(CapabilityTier::Frontier, 3.0, 15.0),
    ),
    (
        "openrouter",
        "x-ai/grok-4.5",
        metered(CapabilityTier::Frontier, 2.0, 6.0),
    ),
    (
        "openrouter",
        "minimax/minimax-m3",
        metered(CapabilityTier::Balanced, 0.6, 2.4),
    ),
    (
        "openrouter",
        "deepseek/deepseek-v4-pro",
        metered(CapabilityTier::Balanced, 0.435, 0.87),
    ),
    (
        "openrouter",
        "tencent/hy3:free",
        metered(CapabilityTier::Fast, 0.0, 0.0),
    ),
    (
        "ambient",
        "moonshotai/kimi-k2.7-code",
        metered(CapabilityTier::Balanced, 0.95, 4.0),
    ),
    (
        "zai",
        "glm-5.2",
        metered(CapabilityTier::Balanced, 1.4, 4.4),
    ),
    (
        "ambient",
        "z-ai/glm-5.2",
        metered(CapabilityTier::Balanced, 1.4, 4.4),
    ),
    (
        "vercel",
        "zai/glm-5.2",
        metered(CapabilityTier::Balanced, 1.4, 4.4),
    ),
    (
        "vercel-anthropic-fast",
        "zai/glm-5.2-fast",
        metered(CapabilityTier::Balanced, 1.4, 4.4),
    ),
    (
        "baseten",
        "zai-org/GLM-5.2",
        metered(CapabilityTier::Balanced, 1.4, 4.4),
    ),
];

/// Allocation guidance for an exact runtime pair, when known.
pub fn economics_for(provider_id: &str, model: &str) -> Option<ModelEconomics> {
    ECONOMICS
        .iter()
        .find(|(p, m, _)| *p == provider_id && *m == model)
        .map(|(_, _, economics)| *economics)
}

/// Classify how a provider is paid for, from its configured auth shape.
///
/// Rented GPU endpoints are user-owned capacity. A command-backed bearer token is
/// a subscription handshake. An API-key environment variable is pay-per-token.
/// OpenAI is plan-routed when it uses account auth rather than an API key.
pub fn billing_class_for_provider(provider_id: &str, provider: &ModelProviderInfo) -> BillingClass {
    if provider_id.starts_with("gpu-") {
        return BillingClass::Local;
    }
    if provider.requires_openai_auth {
        return BillingClass::Plan;
    }
    if provider.env_key.is_some() {
        return BillingClass::Metered;
    }
    if provider.auth.is_some() {
        return BillingClass::Plan;
    }
    BillingClass::Unknown
}

#[cfg(test)]
#[path = "model_economics_tests.rs"]
mod tests;
