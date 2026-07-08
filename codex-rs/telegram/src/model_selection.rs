use codex_model_provider_info::CLAUDE_FABLE_5_PLAN_MODEL;
use codex_model_provider_info::CLAUDE_PLAN_MODEL;
use codex_model_provider_info::corrected_catalog_provider;

const GPT_5_5_MODEL: &str = "gpt-5.5";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ModelAlias {
    pub alias: &'static str,
    pub model: &'static str,
    pub display_name: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CatalogModel {
    pub id: String,
    pub model: String,
    pub display_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ModelResolutionSource {
    Alias,
    Catalog,
    PassThrough,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ModelResolution {
    pub model: String,
    pub source: ModelResolutionSource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AvailableModel {
    pub model: String,
    pub display_name: String,
    pub aliases: Vec<&'static str>,
    pub in_catalog: bool,
}

pub(crate) fn aliases() -> &'static [ModelAlias] {
    &[
        ModelAlias {
            alias: "fable",
            model: CLAUDE_FABLE_5_PLAN_MODEL,
            display_name: "Claude Fable 5 Plan",
        },
        ModelAlias {
            alias: "opus",
            model: CLAUDE_PLAN_MODEL,
            display_name: "Claude Opus 4.8 Plan",
        },
        ModelAlias {
            alias: "gpt",
            model: GPT_5_5_MODEL,
            display_name: "GPT-5.5",
        },
        ModelAlias {
            alias: "gpt-5.5",
            model: GPT_5_5_MODEL,
            display_name: "GPT-5.5",
        },
    ]
}

pub(crate) fn resolve_model(input: &str, catalog: &[CatalogModel]) -> ModelResolution {
    let trimmed = input.trim();
    if let Some(alias) = aliases()
        .iter()
        .find(|alias| alias.alias.eq_ignore_ascii_case(trimmed))
    {
        return ModelResolution {
            model: alias.model.to_string(),
            source: ModelResolutionSource::Alias,
        };
    }

    if let Some(model) = catalog.iter().find(|model| {
        model.id.eq_ignore_ascii_case(trimmed) || model.display_name.eq_ignore_ascii_case(trimmed)
    }) {
        return ModelResolution {
            model: model.model.clone(),
            source: ModelResolutionSource::Catalog,
        };
    }

    ModelResolution {
        model: trimmed.to_string(),
        source: ModelResolutionSource::PassThrough,
    }
}

pub(crate) fn provider_for_model(model: &str, current_provider: &str) -> String {
    corrected_catalog_provider(model, current_provider)
        .unwrap_or(current_provider)
        .to_string()
}

pub(crate) fn available_models(catalog: &[CatalogModel]) -> Vec<AvailableModel> {
    let mut available = Vec::new();
    for model in catalog {
        let aliases = aliases_for_model(&model.model);
        available.push(AvailableModel {
            model: model.model.clone(),
            display_name: model.display_name.clone(),
            aliases,
            in_catalog: true,
        });
    }

    for alias in aliases() {
        if available.iter().any(|model| model.model == alias.model) {
            continue;
        }
        available.push(AvailableModel {
            model: alias.model.to_string(),
            display_name: alias.display_name.to_string(),
            aliases: aliases_for_model(alias.model),
            in_catalog: false,
        });
    }

    available
}

pub(crate) fn known_model_source(model: &str, catalog: &[CatalogModel]) -> Option<&'static str> {
    if catalog.iter().any(|entry| entry.model == model) {
        return Some("catalog");
    }
    if aliases().iter().any(|alias| alias.model == model) {
        return Some("alias");
    }
    None
}

fn aliases_for_model(model: &str) -> Vec<&'static str> {
    aliases()
        .iter()
        .filter(|alias| alias.model == model)
        .map(|alias| alias.alias)
        .collect()
}

#[cfg(test)]
#[path = "model_selection_tests.rs"]
mod tests;
