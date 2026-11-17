use super::*;

#[test]
fn november_models_switch_provider_without_translating_to_older_models() {
    for (model, provider) in [
        ("gpt-6.2-astra", "openai"),
        ("gpt-6.2-sol", "openai"),
        ("claude-fable-5-2", "anthropic"),
    ] {
        assert_eq!(
            provider_for_model(model, "claude-plan"),
            ProviderChoice {
                provider: provider.to_string(),
                changed: true,
                model: model.to_string(),
            }
        );
        assert_eq!(
            provider_for_model(model, provider),
            ProviderChoice {
                provider: provider.to_string(),
                changed: false,
                model: model.to_string(),
            }
        );
    }
    assert_eq!(resolve_model("fable", &[]).model, CLAUDE_FABLE_5_PLAN_MODEL);
    assert_eq!(resolve_model("gpt", &[]).model, GPT_5_5_MODEL);
}
