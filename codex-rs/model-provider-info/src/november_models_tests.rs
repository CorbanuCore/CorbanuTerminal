use super::*;
use pretty_assertions::assert_eq;

#[test]
fn november_models_resolve_without_changing_identity_or_provider_defaults() {
    for (model, provider) in [
        ("gpt-6.2-astra", OPENAI_PROVIDER_ID),
        ("gpt-6.2-sol", OPENAI_PROVIDER_ID),
        (CLAUDE_FABLE_5_2_MODEL, ANTHROPIC_PROVIDER_ID),
    ] {
        assert_eq!(canonical_catalog_provider(model), Some(provider));
        assert_eq!(corrected_catalog_provider(model, provider), None);
        assert_eq!(
            resolve_model_for_provider(Some(model.to_string()), provider),
            Some(model.to_string())
        );
    }
    assert_eq!(
        corrected_catalog_provider(CLAUDE_FABLE_5_2_MODEL, CLAUDE_PLAN_PROVIDER_ID),
        Some(ANTHROPIC_PROVIDER_ID)
    );
    assert_eq!(claude_plan_translation(CLAUDE_FABLE_5_2_MODEL), None);
    assert_eq!(
        resolve_model_for_provider(/*model*/ None, ANTHROPIC_PROVIDER_ID),
        Some(ANTHROPIC_DEFAULT_MODEL.to_string())
    );
    assert_eq!(
        resolve_model_for_provider(/*model*/ None, CLAUDE_PLAN_PROVIDER_ID),
        Some(CLAUDE_PLAN_MODEL.to_string())
    );
}
