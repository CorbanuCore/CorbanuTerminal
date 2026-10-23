//! Connections ("routes") that serve the same model (#653).
//!
//! Some models are served by more than one built-in provider: the same slug
//! over OpenAI-style Chat Completions and over Anthropic-style Messages, e.g.
//! `glm-5.3-flash` on `zai` and `zai-anthropic`. `/model` lets the person
//! choose the connection.
//!
//! Owner decision (Travis, 2026-10-13, binding): when a model is available on
//! more than one connection, the OpenAI-style one is the default. Each family
//! below lists it first, and [`crate::canonical_catalog_provider`] and
//! [`crate::corrected_catalog_provider`] return it.

use crate::ZAI_ANTHROPIC_PROVIDER_ID;
use crate::ZAI_PROVIDER_ID;

/// The request format a connection speaks, in plain words.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModelRouteStyle {
    /// OpenAI-style requests (Chat Completions or Responses).
    OpenAi,
    /// Anthropic-style (Messages) requests.
    Claude,
}

impl ModelRouteStyle {
    pub fn label(self) -> &'static str {
        match self {
            Self::OpenAi => "OpenAI-style",
            Self::Claude => "Claude-style",
        }
    }
}

/// One connection that serves a model.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ModelRoute {
    pub provider_id: &'static str,
    /// The provider's plain name, shared by every connection in the family.
    pub provider_label: &'static str,
    pub style: ModelRouteStyle,
    /// What `/model` says about this connection instead of the catalog
    /// description, which describes the default connection.
    pub note: Option<&'static str>,
}

impl ModelRoute {
    /// "Z.AI (Claude-style)".
    pub fn label(self) -> String {
        format!("{} ({})", self.provider_label, self.style.label())
    }

    /// "GLM 5.3 Flash · Z.AI (Claude-style)", from a catalog display name that
    /// may already start with the provider name ("Z.AI GLM 5.3 Flash").
    pub fn model_label(self, display_name: &str) -> String {
        let display_name = display_name.trim();
        let model_name = display_name
            .strip_prefix(self.provider_label)
            .and_then(|rest| rest.strip_prefix(' '))
            .map(str::trim)
            .filter(|rest| !rest.is_empty())
            .unwrap_or(display_name);
        format!("{model_name} · {}", self.label())
    }
}

struct ModelRouteFamily {
    /// Default (OpenAI-style) connection first.
    routes: [ModelRoute; 2],
    serves: fn(&str) -> bool,
}

/// Only provider pairs checked live end to end are listed (Z.AI: GLM 5.3 Flash
/// on both, #653). A family covers every model its providers already accept
/// (`resolve_model_for_provider` keeps any `glm-*` on both Z.AI connections),
/// not only the checked one. Baseten, OpenRouter and Vercel also have
/// Anthropic-style siblings, still marked experimental; add a family here once
/// one is checked.
const MODEL_ROUTE_FAMILIES: [ModelRouteFamily; 1] = [ModelRouteFamily {
    routes: [
        ModelRoute {
            provider_id: ZAI_PROVIDER_ID,
            provider_label: "Z.AI",
            style: ModelRouteStyle::OpenAi,
            note: None,
        },
        ModelRoute {
            provider_id: ZAI_ANTHROPIC_PROVIDER_ID,
            provider_label: "Z.AI",
            style: ModelRouteStyle::Claude,
            // qa/portfolio/agent-cost-accounting/pf-60-s05/billing-basis-defaults.md
            note: Some(
                "The same model over Z.AI's Anthropic-compatible connection, the GLM Coding Plan route.",
            ),
        },
    ],
    serves: |model| model.starts_with("glm-"),
}];

/// Every built-in connection that serves `model`, default (OpenAI-style)
/// first. Empty when the model has a single connection.
pub fn model_routes(model: &str) -> &'static [ModelRoute] {
    let model = model.trim();
    MODEL_ROUTE_FAMILIES
        .iter()
        .find(|family| (family.serves)(model))
        .map_or(&[], |family| family.routes.as_slice())
}

/// The connection `provider_id` is for `model`, when the model has several.
pub fn model_route(model: &str, provider_id: &str) -> Option<ModelRoute> {
    let provider_id = provider_id.trim();
    model_routes(model)
        .iter()
        .copied()
        .find(|route| route.provider_id == provider_id)
}

/// The default connection for `model` when it has several: the OpenAI-style one.
pub fn default_model_route(model: &str) -> Option<ModelRoute> {
    model_routes(model).first().copied()
}

#[cfg(test)]
#[path = "model_routes_tests.rs"]
mod tests;
