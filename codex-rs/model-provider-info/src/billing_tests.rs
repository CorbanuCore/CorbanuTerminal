use super::*;
use crate::built_in_model_providers;
use codex_protocol::config_types::ModelProviderAuthInfo;
use pretty_assertions::assert_eq;
use std::collections::BTreeMap;

/// The declared default for every built-in provider, at its own route and with
/// the credential it is built to send. Written out from the defaults table
/// (`qa/portfolio/agent-cost-accounting/pf-60-s05/billing-basis-defaults.md`),
/// not derived from `BUILT_IN`.
#[test]
fn every_built_in_provider_declares_its_default_basis() {
    let expected = BTreeMap::from([
        ("amazon-bedrock", PayPerUse),
        ("ambient", PayPerUse),
        ("anthropic", PayPerUse),
        ("baseten", PayPerUse),
        ("baseten-anthropic", PayPerUse),
        ("claude-plan", Subscription),
        ("deepseek", PayPerUse),
        ("kimi-code", Subscription),
        ("lmstudio", Local),
        ("meta", PayPerUse),
        ("ollama", Local),
        ("openai", Subscription),
        ("openrouter", PayPerUse),
        ("openrouter-anthropic", PayPerUse),
        ("pfterminal-plan", PayPerUse),
        ("pfterminal-plan-anthropic", PayPerUse),
        ("vercel", PayPerUse),
        ("vercel-anthropic", PayPerUse),
        ("vercel-anthropic-fast", PayPerUse),
        ("zai", PayPerUse),
        ("zai-anthropic", Subscription),
    ]);
    let providers = built_in_model_providers(/*openai_base_url*/ None);
    let declared: BTreeMap<&str, BillingBasis> = providers
        .iter()
        .map(|(id, provider)| {
            // The OpenAI provider's default credential is a ChatGPT sign-in.
            let credential = BillingCredential::of(provider, Some(AuthMode::Chatgpt));
            let endpoint = provider.base_url.as_deref().unwrap_or_default();
            let declaration = declared_billing(
                id, provider, credential, endpoint, /*at_built_in_route*/ true,
            );
            let BillingDeclaration::BuiltIn(basis) = declaration else {
                panic!("{id} has no built-in declaration: {declaration:?}");
            };
            (id.as_str(), basis)
        })
        .collect();
    assert_eq!(declared, expected);
    assert_eq!(declared.len(), 21);
}

/// Requirement (a): the basis follows the declared table, not the auth type.
#[test]
fn basis_does_not_follow_the_auth_type() {
    let providers = built_in_model_providers(/*openai_base_url*/ None);
    let declared = |id: &str, provider: &ModelProviderInfo, auth_mode| {
        declared_billing(
            id,
            provider,
            BillingCredential::of(provider, auth_mode),
            provider.base_url.as_deref().unwrap_or_default(),
            /*at_built_in_route*/ true,
        )
    };
    // A command login does not make Bedrock a subscription (review Minor 10).
    let mut bedrock = providers[AMAZON_BEDROCK_PROVIDER_ID].clone();
    bedrock.auth =
        Some(toml::from_str::<ModelProviderAuthInfo>("command = \"print-token\"").unwrap());
    assert_eq!(
        declared(AMAZON_BEDROCK_PROVIDER_ID, &bedrock, None),
        BillingDeclaration::BuiltIn(PayPerUse)
    );
    // An environment API key does not make Kimi Code pay per use (the Blocker).
    let kimi = &providers[KIMI_CODE_PROVIDER_ID];
    assert_eq!(BillingCredential::of(kimi, None), BillingCredential::ApiKey);
    assert_eq!(
        declared(KIMI_CODE_PROVIDER_ID, kimi, Some(AuthMode::ApiKey)),
        BillingDeclaration::BuiltIn(Subscription)
    );
    // OpenAI follows its per-credential rows.
    let openai = &providers[OPENAI_PROVIDER_ID];
    for (auth_mode, expected) in [
        (
            Some(AuthMode::Chatgpt),
            BillingDeclaration::BuiltIn(Subscription),
        ),
        (
            Some(AuthMode::ApiKey),
            BillingDeclaration::BuiltIn(PayPerUse),
        ),
        (None, BillingDeclaration::NotDeclared),
    ] {
        assert_eq!(declared(OPENAI_PROVIDER_ID, openai, auth_mode), expected);
    }
    // A provider's own login is declared only for that provider.
    let claude = &providers[CLAUDE_PLAN_PROVIDER_ID];
    assert_eq!(
        declared("my-claude", claude, None),
        BillingDeclaration::NotDeclared
    );
}

/// Option B: a provider id the table doesn't know gets the basis of the route
/// and credential it used; an unknown route is not declared.
#[test]
fn custom_providers_follow_their_route() {
    let custom = ModelProviderInfo {
        name: "custom".into(),
        env_key: Some("CUSTOM_KEY".into()),
        ..ModelProviderInfo::default()
    };
    let at = |endpoint: &str| {
        declared_billing(
            "custom",
            &custom,
            BillingCredential::of(&custom, None),
            endpoint,
            /*at_built_in_route*/ false,
        )
    };
    for (endpoint, expected) in [
        (
            "https://api.z.ai/api/coding/paas/v4",
            BillingDeclaration::BuiltIn(Subscription),
        ),
        (
            "https://api.z.ai/api/paas/v4/",
            BillingDeclaration::BuiltIn(PayPerUse),
        ),
        (
            "https://api.kimi.com/coding/v1",
            BillingDeclaration::BuiltIn(Subscription),
        ),
        (
            "https://api.moonshot.ai/v1",
            BillingDeclaration::BuiltIn(PayPerUse),
        ),
        (
            "https://api.openai.com/v1",
            BillingDeclaration::BuiltIn(PayPerUse),
        ),
        (
            "https://llm.example.com/v1",
            BillingDeclaration::NotDeclared,
        ),
    ] {
        assert_eq!(at(endpoint), expected, "{endpoint}");
    }
    // A built-in provider moved off its own route is judged by the route.
    let providers = built_in_model_providers(/*openai_base_url*/ None);
    assert_eq!(
        declared_billing(
            ZAI_PROVIDER_ID,
            &providers[ZAI_PROVIDER_ID],
            BillingCredential::ApiKey,
            "https://api.z.ai/api/coding/paas/v4",
            /*at_built_in_route*/ false,
        ),
        BillingDeclaration::BuiltIn(Subscription)
    );
}

#[test]
fn a_configured_basis_wins_and_says_so() {
    let provider = ModelProviderInfo {
        billing: Some(Subscription),
        ..built_in_model_providers(/*openai_base_url*/ None)[ZAI_PROVIDER_ID].clone()
    };
    assert_eq!(
        declared_billing(
            ZAI_PROVIDER_ID,
            &provider,
            BillingCredential::ApiKey,
            "https://api.z.ai/api/paas/v4",
            /*at_built_in_route*/ true,
        ),
        BillingDeclaration::UserConfig(Subscription)
    );
    assert!(not_declared_next_step("my-llm").contains("model_providers.my-llm.billing"));
}

#[test]
fn billing_parses_and_names_bad_values() {
    let parsed: ModelProviderInfo = toml::from_str("billing = \"pay_per_use\"").unwrap();
    assert_eq!(parsed.billing, Some(PayPerUse));
    assert!(parsed.is_billing_override_only());
    let error = toml::from_str::<ModelProviderInfo>("billing = \"free\"")
        .unwrap_err()
        .to_string();
    assert!(error.contains("billing"), "{error}");
}

/// No two built-in default routes declare different bases for one credential:
/// such a route would be undeclared for custom providers.
#[test]
fn default_routes_are_unambiguous() {
    for (route, credential, basis) in default_routes() {
        assert_eq!(
            route_basis(route, *credential),
            Some(*basis),
            "{route} {credential:?}"
        );
    }
}
