use codex_model_provider_info::CLAUDE_FABLE_5_PLAN_MODEL;
use codex_model_provider_info::CLAUDE_PLAN_MODEL;
use codex_model_provider_info::CLAUDE_PLAN_PROVIDER_ID;
use codex_model_provider_info::OPENAI_PROVIDER_ID;
use pretty_assertions::assert_eq;

use super::CatalogModel;
use super::ModelResolution;
use super::ModelResolutionSource;
use super::provider_for_model;
use super::resolve_model;

#[test]
fn aliases_resolve_before_catalog_matches() {
    let catalog = vec![CatalogModel {
        id: "fable".to_string(),
        model: "catalog-fable".to_string(),
        display_name: "Fable".to_string(),
    }];

    assert_eq!(
        resolve_model("fable", &catalog),
        ModelResolution {
            model: CLAUDE_FABLE_5_PLAN_MODEL.to_string(),
            source: ModelResolutionSource::Alias,
        }
    );
    assert_eq!(
        resolve_model("OPUS", &catalog),
        ModelResolution {
            model: CLAUDE_PLAN_MODEL.to_string(),
            source: ModelResolutionSource::Alias,
        }
    );
    assert_eq!(
        resolve_model("gpt", &catalog),
        ModelResolution {
            model: "gpt-5.5".to_string(),
            source: ModelResolutionSource::Alias,
        }
    );
    assert_eq!(
        resolve_model("gpt-5.5", &catalog),
        ModelResolution {
            model: "gpt-5.5".to_string(),
            source: ModelResolutionSource::Alias,
        }
    );
}

#[test]
fn catalog_matching_uses_id_or_display_name_case_insensitively() {
    let catalog = vec![CatalogModel {
        id: "gpt-catalog".to_string(),
        model: "gpt-real-slug".to_string(),
        display_name: "GPT Catalog".to_string(),
    }];

    assert_eq!(
        resolve_model("GPT-CATALOG", &catalog),
        ModelResolution {
            model: "gpt-real-slug".to_string(),
            source: ModelResolutionSource::Catalog,
        }
    );
    assert_eq!(
        resolve_model("gpt catalog", &catalog),
        ModelResolution {
            model: "gpt-real-slug".to_string(),
            source: ModelResolutionSource::Catalog,
        }
    );
}

#[test]
fn unknown_model_is_passed_through_verbatim_after_trim() {
    assert_eq!(
        resolve_model("  vendor/new-model  ", &[]),
        ModelResolution {
            model: "vendor/new-model".to_string(),
            source: ModelResolutionSource::PassThrough,
        }
    );
}

#[test]
fn provider_correction_matches_alias_families() {
    assert_eq!(
        provider_for_model(CLAUDE_FABLE_5_PLAN_MODEL, OPENAI_PROVIDER_ID),
        CLAUDE_PLAN_PROVIDER_ID
    );
    assert_eq!(
        provider_for_model(CLAUDE_PLAN_MODEL, OPENAI_PROVIDER_ID),
        CLAUDE_PLAN_PROVIDER_ID
    );
    assert_eq!(
        provider_for_model("gpt-5.5", CLAUDE_PLAN_PROVIDER_ID),
        OPENAI_PROVIDER_ID
    );
}
