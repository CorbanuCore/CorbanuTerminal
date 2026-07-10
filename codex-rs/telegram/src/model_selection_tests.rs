use std::collections::HashMap;

use codex_model_provider_info::AMBIENT_PROVIDER_ID;
use codex_model_provider_info::CLAUDE_FABLE_5_PLAN_MODEL;
use codex_model_provider_info::CLAUDE_PLAN_MODEL;
use codex_model_provider_info::CLAUDE_PLAN_PROVIDER_ID;
use codex_model_provider_info::ModelProviderInfo;
use codex_model_provider_info::OPENAI_PROVIDER_ID;
use codex_model_provider_info::ZAI_PROVIDER_ID;
use codex_model_provider_info::built_in_model_providers;
use pretty_assertions::assert_eq;

use super::CatalogModel;
use super::ModelResolution;
use super::ModelResolutionSource;
use super::missing_provider_credential_with;
use super::provider_for_model;
use super::resolve_model;

const ZAI: &str = "zai";

fn provider_needing_env(env_key: &str) -> ModelProviderInfo {
    ModelProviderInfo {
        env_key: Some(env_key.to_string()),
        ..ModelProviderInfo::default()
    }
}

fn providers(entries: [(&str, ModelProviderInfo); 1]) -> HashMap<String, ModelProviderInfo> {
    entries
        .into_iter()
        .map(|(id, info)| (id.to_string(), info))
        .collect()
}

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
    let fable = provider_for_model(CLAUDE_FABLE_5_PLAN_MODEL, OPENAI_PROVIDER_ID);
    assert_eq!(fable.provider, CLAUDE_PLAN_PROVIDER_ID);
    assert!(fable.changed);

    let opus = provider_for_model(CLAUDE_PLAN_MODEL, OPENAI_PROVIDER_ID);
    assert_eq!(opus.provider, CLAUDE_PLAN_PROVIDER_ID);
    assert!(opus.changed);

    let gpt = provider_for_model("gpt-5.5", CLAUDE_PLAN_PROVIDER_ID);
    assert_eq!(gpt.provider, OPENAI_PROVIDER_ID);
    assert!(gpt.changed);
}

/// The regression the original suite missed: it only ever asserted the four families
/// `corrected_catalog_provider` repairs, so the inherit branch — every other model —
/// was never exercised. `ambient/large` on a `zai` session stays on `zai`.
#[test]
fn provider_is_inherited_for_models_outside_the_corrected_families() {
    let ambient = provider_for_model("ambient/large", ZAI);
    assert_eq!(ambient.provider, ZAI);
    assert!(
        !ambient.changed,
        "no provider label exists to route by; the switch must not claim one"
    );

    let glm = provider_for_model("z-ai/glm-5.2", ZAI);
    assert_eq!(glm.provider, ZAI);
    assert!(!glm.changed);
}

#[test]
fn same_provider_correction_is_not_reported_as_a_change() {
    let choice = provider_for_model(CLAUDE_PLAN_MODEL, CLAUDE_PLAN_PROVIDER_ID);
    assert_eq!(choice.provider, CLAUDE_PLAN_PROVIDER_ID);
    assert!(!choice.changed);
}

#[test]
fn missing_credential_is_reported_when_no_key_is_reachable() {
    let map = providers([(ZAI, provider_needing_env("ZAI_API_KEY"))]);
    let missing = missing_provider_credential_with(ZAI, &map, |_| false)
        .expect("an unreachable key must be reported");
    assert_eq!(missing.provider, ZAI);
    assert_eq!(missing.env_key, "ZAI_API_KEY");
}

/// A key stored via `/providers` lives in the vault, not the environment. Reporting it
/// missing would refuse a switch that would in fact have worked.
#[test]
fn credential_reachable_from_the_vault_is_not_reported_missing() {
    let map = providers([(ZAI, provider_needing_env("ZAI_API_KEY"))]);
    assert!(missing_provider_credential_with(ZAI, &map, |_| true).is_none());
}

/// Providers that authenticate by token command (`claude-plan`) or subscription login
/// hold no env var. A missing variable proves nothing about them and must never block
/// a switch — this is the guard that keeps the fix from breaking the primary harness.
#[test]
fn providers_authenticating_without_an_env_key_are_never_blocked() {
    let command_auth = ModelProviderInfo {
        env_key: Some("IGNORED_KEY".to_string()),
        requires_openai_auth: true,
        ..ModelProviderInfo::default()
    };
    let map = providers([(CLAUDE_PLAN_PROVIDER_ID, command_auth)]);
    assert!(missing_provider_credential_with(CLAUDE_PLAN_PROVIDER_ID, &map, |_| false).is_none());

    let no_env = providers([(OPENAI_PROVIDER_ID, ModelProviderInfo::default())]);
    assert!(missing_provider_credential_with(OPENAI_PROVIDER_ID, &no_env, |_| false).is_none());
}

#[test]
fn unknown_provider_is_not_reported_as_missing_a_credential() {
    let map = providers([(ZAI, provider_needing_env("ZAI_API_KEY"))]);
    assert!(missing_provider_credential_with("nope", &map, |_| false).is_none());
}

/// Asserted against the real provider table, not a hand-built one: `claude-plan`
/// authenticates via a token command and must stay switchable with no env vars set,
/// while `ambient` and `zai` declare an `env_key` and must be caught.
#[test]
fn builtin_providers_split_correctly_on_credential_requirements() {
    let builtin = built_in_model_providers(None);

    assert!(
        missing_provider_credential_with(CLAUDE_PLAN_PROVIDER_ID, &builtin, |_| false).is_none(),
        "claude-plan authenticates via a token command; a missing key must not block it"
    );
    assert!(
        missing_provider_credential_with(OPENAI_PROVIDER_ID, &builtin, |_| false).is_none(),
        "openai declares no env_key"
    );
    assert!(
        missing_provider_credential_with(AMBIENT_PROVIDER_ID, &builtin, |_| false).is_some(),
        "ambient requires AMBIENT_API_KEY"
    );
    assert!(
        missing_provider_credential_with(ZAI_PROVIDER_ID, &builtin, |_| false).is_some(),
        "zai requires ZAI_API_KEY — the QA failure this fix catches at /model time"
    );
}
