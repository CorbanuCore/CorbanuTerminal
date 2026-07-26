use super::*;
use crate::ANTHROPIC_PROVIDER_ID;
use crate::CLAUDE_PLAN_PROVIDER_ID;
use crate::OPENAI_PROVIDER_ID;
use crate::built_in_model_providers;

#[test]
fn billing_class_is_derived_from_provider_auth_shape() {
    let providers = built_in_model_providers(None);

    // Subscription handshake via a command-backed bearer token.
    let claude_plan = &providers[CLAUDE_PLAN_PROVIDER_ID];
    assert_eq!(
        billing_class_for_provider(CLAUDE_PLAN_PROVIDER_ID, claude_plan),
        BillingClass::Plan
    );

    // Account auth rather than an API key.
    let openai = &providers[OPENAI_PROVIDER_ID];
    assert_eq!(
        billing_class_for_provider(OPENAI_PROVIDER_ID, openai),
        BillingClass::Plan
    );

    // Every API-key provider is pay-per-token, not just the one that caused an
    // incident.
    for provider_id in ["anthropic", "openrouter", "kimi-code", "zai", "baseten"] {
        let Some(provider) = providers.get(provider_id) else {
            continue;
        };
        assert_eq!(
            billing_class_for_provider(provider_id, provider),
            BillingClass::Metered,
            "{provider_id} bills per token and must not be classified as plan"
        );
    }

    // Rented GPU capacity is already paid for by the hour.
    let rented = &providers[OPENAI_PROVIDER_ID];
    assert_eq!(
        billing_class_for_provider("gpu-gpu-1234", rented),
        BillingClass::Local
    );
}

#[test]
fn plan_routes_carry_burn_weight_instead_of_a_zero_price() {
    // Plan is not free. It is a finite shared pool, so plan runtimes express a
    // relative drain rate and must never present a dollar price of zero, which
    // would read as costless to an allocating agent.
    for model in ["claude-opus-5-plan", "claude-fable-5-plan"] {
        let economics =
            economics_for(CLAUDE_PLAN_PROVIDER_ID, model).expect("plan runtime should be known");
        assert!(economics.input_usd_per_mtok.is_none());
        assert!(economics.output_usd_per_mtok.is_none());
        assert!(
            economics.plan_burn_weight.unwrap_or_default() > 0.0,
            "{model} must declare a nonzero plan burn weight"
        );
    }

    // Within a plan there is still a cost gradient: Fable draws roughly double
    // an Opus session, which is why grunt work should not go to it.
    let fable = economics_for(CLAUDE_PLAN_PROVIDER_ID, "claude-fable-5-plan").unwrap();
    let opus = economics_for(CLAUDE_PLAN_PROVIDER_ID, "claude-opus-5-plan").unwrap();
    assert!(fable.plan_burn_weight > opus.plan_burn_weight);

    // And the cheap plan tier really is cheaper than the frontier plan tier.
    let luna = economics_for(OPENAI_PROVIDER_ID, "gpt-5.6-luna").unwrap();
    let sol = economics_for(OPENAI_PROVIDER_ID, "gpt-5.6-sol").unwrap();
    assert!(luna.plan_burn_weight < sol.plan_burn_weight);
    assert_eq!(luna.tier, CapabilityTier::Fast);
    assert_eq!(sol.tier, CapabilityTier::Frontier);
}

#[test]
fn the_plan_suffix_footgun_is_visible_in_the_data() {
    // `claude-fable-5-plan` on claude-plan is subscription-routed; the same model
    // family on `anthropic` is metered at $10/$50. One word apart, and an agent
    // that cannot see the difference will spend real money by accident.
    let planned = economics_for(CLAUDE_PLAN_PROVIDER_ID, "claude-fable-5-plan").unwrap();
    let metered = economics_for(ANTHROPIC_PROVIDER_ID, "claude-fable-5").unwrap();

    assert!(planned.plan_burn_weight.is_some());
    assert!(planned.input_usd_per_mtok.is_none());
    assert_eq!(metered.input_usd_per_mtok, Some(10.0));
    assert_eq!(metered.output_usd_per_mtok, Some(50.0));
    assert!(metered.plan_burn_weight.is_none());
}

#[test]
fn metered_tiers_order_by_real_cost() {
    let fable = economics_for(ANTHROPIC_PROVIDER_ID, "claude-fable-5").unwrap();
    let grok = economics_for("openrouter", "x-ai/grok-4.5").unwrap();
    let deepseek = economics_for("openrouter", "deepseek/deepseek-v4-pro").unwrap();

    // Output dominates agent cost, so the ordering that matters is output price.
    assert!(fable.output_usd_per_mtok > grok.output_usd_per_mtok);
    assert!(grok.output_usd_per_mtok > deepseek.output_usd_per_mtok);
    assert_eq!(deepseek.tier, CapabilityTier::Balanced);
}

#[test]
fn unknown_runtimes_are_absent_rather_than_guessed() {
    assert!(economics_for("openrouter", "openrouter/owl-alpha").is_none());
    assert!(economics_for("meta", "muse-spark-1.1").is_none());
    assert!(economics_for("openai", "not-a-real-model").is_none());
}
