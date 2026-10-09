use super::*;
use codex_protocol::openai_models::ModelOrchestrationMetadata;
use pretty_assertions::assert_eq;

#[test]
fn accounting_chat_prices_exact_source_and_unknown() -> anyhow::Result<()> {
    let scope = Uuid::new_v4();
    let source = "openai-chat-api-key-bundled-v1";
    let first = chat_original("gpt-5.3-codex", "openai", scope, /*accepted_at*/ 1000)?.remove(0);
    let tuple = serde_json::to_vec(&(
        source,
        "openai",
        "gpt-5.3-codex",
        "api_key",
        "default",
        "USD/million",
        1750,
        14000,
        Some(175),
    ))?;
    assert_eq!(
        first.source_reference,
        Uuid::new_v5(&Uuid::NAMESPACE_OID, &tuple)
    );
    assert_eq!(
        first.rates,
        Rates {
            noncached: Some(rate(/*milli*/ 1750)?),
            output: Some(rate(/*milli*/ 14000)?),
            read: Some(rate(/*milli*/ 175)?),
            write: None
        }
    );
    assert_eq!(
        (
            first.observed_at_ms,
            first.approved_at_ms,
            first.effective_from_ms,
            first.effective_end_ms
        ),
        (1000.try_into()?, 1000.try_into()?, 1000.try_into()?, None)
    );
    let second = chat_original("gpt-5.3-codex", "openai", scope, /*accepted_at*/ 2000)?.remove(0);
    assert_ne!(first.id, second.id);
    assert_eq!(first.source_reference, second.source_reference);
    for model in [
        "gpt-6.1-sol",
        "remote-only",
        "Gpt-5.6-sol",
        "openai/gpt-5.6-sol",
        "claude-opus-5",
    ] {
        assert!(chat_original(model, "openai", scope, /*accepted_at*/ 1000)?.is_empty());
    }
    let catalog = codex_models_manager::bundled_models_response()?;
    let row = catalog
        .models
        .iter()
        .find(|r| r.slug == "gpt-5.6-sol")
        .unwrap()
        .clone();
    assert_eq!(
        billing_for(&[row.clone(), row.clone()], &row.slug, "openai"),
        None
    );
    assert_eq!(
        billing_for(std::slice::from_ref(&row), &row.slug, "openrouter"),
        None
    );
    let mut disabled = row.clone();
    disabled.orchestration = Some(ModelOrchestrationMetadata::Disabled {
        provider_id: "openai".into(),
        capability: codex_protocol::openai_models::ModelCapabilityTier::Frontier,
        reason: "fixture".into(),
        billing: None,
    });
    assert_eq!(billing_for(&[disabled], &row.slug, "openai"), None);
    for billing in [
        ModelBilling::Local,
        ModelBilling::Plan {
            relative_burn_millis: 1000,
        },
        ModelBilling::PlanSchedule {
            off_peak_relative_burn_millis: 1000,
            peak_relative_burn_millis: 2000,
            peak_start_utc_hour: 6,
            peak_end_utc_hour: 10,
            peak_weekdays: None,
            promotional_off_peak_relative_burn_millis: None,
            promotion_valid_through_utc: None,
        },
    ] {
        assert!(
            billed(
                "fixture", "openai", &billing, scope, /*accepted_at*/ 1000, source
            )?
            .is_empty()
        );
    }
    for read in [None, Some(0)] {
        let billing = ModelBilling::Metered {
            input_milli_usd_per_million_tokens: 1,
            output_milli_usd_per_million_tokens: 2,
            cached_input_milli_usd_per_million_tokens: read,
            cache_write_milli_usd_per_million_tokens: None,
            long_context: None,
            valid_through_utc: None,
        };
        let projected = billed(
            "fixture", "openai", &billing, scope, /*accepted_at*/ 1000, source,
        )?
        .remove(0);
        assert_eq!(
            (projected.rates.read, projected.rates.write),
            (read.map(rate).transpose()?, None)
        );
    }
    // Preserve the literal Responses and Anthropic source byte regressions.
    accounting_responses_prices_exact_and_unknown();
    Ok(())
}

#[test]
fn accounting_responses_prices_exact_and_unknown() {
    let scope = Uuid::new_v4();
    let first = responses_original(
        "gpt-5.3-codex",
        "openai",
        scope,
        /*accepted_at*/ 1000,
        /*tier*/ None,
    )
    .unwrap()
    .remove(0);
    let later = responses_original(
        "gpt-5.3-codex",
        "openai",
        scope,
        /*accepted_at*/ 2000,
        Some("default"),
    )
    .unwrap()
    .remove(0);
    assert_eq!(
        first.rates,
        Rates {
            noncached: Some(rate(/*milli*/ 1750).unwrap()),
            output: Some(rate(/*milli*/ 14000).unwrap()),
            read: Some(rate(/*milli*/ 175).unwrap()),
            write: None,
        }
    );
    assert_eq!(first.provider, "openai");
    assert_eq!(first.scope, scope);
    assert_eq!(first.source_reference, later.source_reference);
    assert_ne!(first.id, later.id);
    assert_eq!(i64::from(first.effective_from_ms), 1000);
    let source = serde_json::to_vec(&(
        "openai-responses-api-key-bundled-v1",
        "openai",
        "gpt-5.3-codex",
        "api_key",
        "default",
        "USD/million",
        1750,
        14000,
        Some(175),
    ))
    .unwrap();
    assert_eq!(
        first.source_reference,
        Uuid::new_v5(&Uuid::NAMESPACE_OID, &source)
    );
    for model in [
        "gpt-6.1-sol",
        "remote-only",
        "Gpt-5.6-sol",
        "openai/gpt-5.6-sol",
        "claude-opus-5",
    ] {
        assert!(
            responses_original(
                model, "openai", scope, /*accepted_at*/ 1000, /*tier*/ None
            )
            .unwrap()
            .is_empty()
        );
    }
    for tier in ["priority", "flex", "auto", "unknown"] {
        assert!(
            responses_original(
                "gpt-5.6-sol",
                "openai",
                scope,
                /*accepted_at*/ 1000,
                Some(tier)
            )
            .unwrap()
            .is_empty()
        );
    }
    for billing in [
        ModelBilling::Local,
        ModelBilling::Plan {
            relative_burn_millis: 1000,
        },
    ] {
        assert!(
            responses_project("fixture", scope, &billing, /*accepted_at*/ 1000)
                .unwrap()
                .is_empty()
        );
    }
    for read in [None, Some(0)] {
        let billing = ModelBilling::Metered {
            input_milli_usd_per_million_tokens: 0,
            output_milli_usd_per_million_tokens: 0,
            cached_input_milli_usd_per_million_tokens: read,
            cache_write_milli_usd_per_million_tokens: None,
            long_context: None,
            valid_through_utc: None,
        };
        let value = responses_project("fixture", scope, &billing, /*accepted_at*/ 1000)
            .unwrap()
            .remove(0);
        assert_eq!(value.rates.read, read.map(|value| rate(value).unwrap()));
        assert_eq!(value.rates.write, None);
    }
    let anthropic = anthropic_original(
        "claude-opus-5",
        "anthropic",
        scope,
        /*accepted_at*/ 1000,
    )
    .unwrap()
    .remove(0);
    // Every billed projection now states provenance in one shape, including the
    // authentication and tier the rates are quoted for.
    let source = serde_json::to_vec(&(
        "anthropic-bundled-v1",
        "anthropic",
        "claude-opus-5",
        "api_key",
        "default",
        "USD/million",
        5000,
        25000,
        Some(500),
    ))
    .unwrap();
    // Opus 5 on an API key: the five-minute cache write at 1.25x input.
    let source = serde_json::to_vec(&(
        serde_json::from_slice::<serde_json::Value>(&source).unwrap(),
        "cache_write_5m",
        6250,
    ))
    .unwrap();
    assert_eq!(
        anthropic.source_reference,
        Uuid::new_v5(&Uuid::NAMESPACE_OID, &source)
    );
}

#[test]
fn accounting_bundled_prices_are_exact_prospective_and_content_identified() {
    let scope = Uuid::new_v4();
    let first = anthropic_original(
        "claude-opus-5",
        "anthropic",
        scope,
        /*accepted_at*/ 1000,
    )
    .unwrap()
    .remove(0);
    let second = anthropic_original(
        "claude-opus-5",
        "anthropic",
        scope,
        /*accepted_at*/ 2000,
    )
    .unwrap()
    .remove(0);
    assert_ne!(first.id, second.id);
    assert_eq!(first.source_reference, second.source_reference);
    assert_eq!(
        first.rates,
        Rates {
            noncached: Some(rate(/*milli*/ 5000).unwrap()),
            read: Some(rate(/*milli*/ 500).unwrap()),
            output: Some(rate(/*milli*/ 25000).unwrap()),
            write: Some(rate(/*milli*/ 6250).unwrap()),
        }
    );
    assert_eq!(
        (
            first.scope,
            first.provider.as_str(),
            first.model.as_str(),
            first.currency,
            first.unit,
            first.source_kind
        ),
        (
            scope,
            "anthropic",
            "claude-opus-5",
            Currency::Usd,
            Unit::PerMillionTokens,
            SourceKind::NativeCatalog
        )
    );
    assert_eq!(
        (
            i64::from(first.observed_at_ms),
            i64::from(first.approved_at_ms),
            i64::from(first.effective_from_ms),
            first.effective_end_ms
        ),
        (1000, 1000, 1000, None)
    );
    let fable = anthropic_original(
        "claude-fable-5-1",
        "anthropic",
        scope,
        /*accepted_at*/ 1000,
    )
    .unwrap()
    .remove(0);
    assert_ne!(fable.source_reference, first.source_reference);
    assert_eq!(
        fable.rates,
        Rates {
            noncached: Some(rate(/*milli*/ 10000).unwrap()),
            read: Some(rate(/*milli*/ 250).unwrap()),
            output: Some(rate(/*milli*/ 50000).unwrap()),
            write: Some(rate(/*milli*/ 12500).unwrap()),
        }
    );
}

#[test]
fn accounting_price_authority_rejects_aliases_remote_and_non_metered_rows() {
    for model in [
        "claude-opus-5-unknown",
        "namespace/claude-opus-5",
        "CLAUDE-OPUS-5",
        "remote-only",
    ] {
        assert_eq!(
            anthropic_original(model, "anthropic", Uuid::nil(), /*accepted_at*/ 10).unwrap(),
            vec![]
        );
    }
    // Rows with no per-token price state none, whoever serves them.
    for billing in [
        ModelBilling::Local,
        ModelBilling::Plan {
            relative_burn_millis: 1000,
        },
    ] {
        assert_eq!(
            billed(
                "claude-opus-5",
                "anthropic",
                &billing,
                Uuid::nil(),
                /*accepted_at*/ 10,
                "anthropic-bundled-v1"
            )
            .unwrap(),
            vec![]
        );
    }
    // An auth-dependent row does state one, and under API-key authentication it
    // is what the provider charges. Refusing it, as this projection used to,
    // left those turns priceless on the very auth mode that is billed per token.
    let auth_dependent = ModelBilling::AuthDependent {
        plan_relative_burn_millis: 1000,
        api_key_input_milli_usd_per_million_tokens: 5000,
        api_key_output_milli_usd_per_million_tokens: 25000,
        api_key_cached_input_milli_usd_per_million_tokens: None,
        api_key_cache_write_milli_usd_per_million_tokens: None,
        api_key_long_context: None,
        api_key_valid_through_utc: None,
    };
    let priced = billed(
        "claude-opus-5",
        "anthropic",
        &auth_dependent,
        Uuid::nil(),
        /*accepted_at*/ 10,
        "anthropic-bundled-v1",
    )
    .unwrap()
    .remove(0);
    assert_eq!(priced.basis, Basis::Billed);
    assert_eq!(priced.plan_burn_millis, None);
    assert_eq!(priced.rates.noncached, Some(rate(/*milli*/ 5000).unwrap()));
}

#[test]
fn accounting_price_projection_retains_absent_read_and_write_and_exact_milli() {
    for (milli, expected) in [
        (0, "0"),
        (1, "0.001"),
        (999, "0.999"),
        (1000, "1"),
        (u32::MAX, "4294967.295"),
    ] {
        let value = rate(milli).unwrap();
        let expected: Decimal = serde_json::from_value(serde_json::json!(expected)).unwrap();
        assert_eq!(value, expected);
    }
    let billing = ModelBilling::Metered {
        input_milli_usd_per_million_tokens: 1,
        output_milli_usd_per_million_tokens: 2,
        cached_input_milli_usd_per_million_tokens: None,
        cache_write_milli_usd_per_million_tokens: None,
        long_context: None,
        valid_through_utc: None,
    };
    let quote = billed(
        "fixture",
        "anthropic",
        &billing,
        Uuid::nil(),
        /*accepted_at*/ 10,
        "anthropic-bundled-v1",
    )
    .unwrap()
    .remove(0);
    assert_eq!((quote.rates.read, quote.rates.write), (None, None));
    let mut changed = billing.clone();
    if let ModelBilling::Metered {
        input_milli_usd_per_million_tokens,
        ..
    } = &mut changed
    {
        *input_milli_usd_per_million_tokens = 3;
    }
    assert_ne!(
        quote.source_reference,
        billed(
            "fixture",
            "anthropic",
            &changed,
            Uuid::nil(),
            /*accepted_at*/ 10,
            "anthropic-bundled-v1"
        )
        .unwrap()[0]
            .source_reference
    );
    assert!(
        billed(
            "fixture",
            "anthropic",
            &billing,
            Uuid::nil(),
            /*accepted_at*/ -1,
            "anthropic-bundled-v1"
        )
        .is_err()
    );
}

/// Plan rows are the ones the old projection could not state at all: it had no
/// plan basis, so every subscription turn recorded tokens and nothing else.
#[test]
fn accounting_plan_projection_states_the_rate_and_only_stated_equivalents() {
    let scope = Uuid::new_v4();
    let now = chrono::DateTime::parse_from_rfc3339("2026-09-21T12:00:00Z")
        .unwrap()
        .timestamp_millis();
    // A Claude subscription row states its plan rate, and its API equivalent
    // is the same model's Anthropic API row with the one-hour cache write
    // (2x input) the subscription route requests.
    let plan = plan_original(
        "claude-opus-5-plan",
        "claude-plan",
        scope,
        now,
        /*tier*/ None,
    )
    .unwrap()
    .remove(0);
    assert_eq!(plan.basis, Basis::PlanEquivalent);
    assert_eq!(plan.plan_burn_millis, Some(1000));
    assert_eq!(
        (
            plan.rates.noncached,
            plan.rates.output,
            plan.rates.read,
            plan.rates.write
        ),
        (
            Some(rate(/*milli*/ 5000).unwrap()),
            Some(rate(/*milli*/ 25000).unwrap()),
            Some(rate(/*milli*/ 500).unwrap()),
            Some(rate(/*milli*/ 10000).unwrap())
        )
    );
    assert_eq!(plan.provider, "claude-plan");
    let opus_5_5 = plan_original(
        "claude-opus-5-5-plan",
        "claude-plan",
        scope,
        now,
        /*tier*/ None,
    )
    .unwrap()
    .remove(0);
    assert_eq!(
        (opus_5_5.rates.noncached, opus_5_5.rates.write),
        (
            Some(rate(/*milli*/ 4000).unwrap()),
            Some(rate(/*milli*/ 8000).unwrap())
        )
    );

    // An auth-dependent row states both: the plan rate that applied and the API
    // rates the same tokens would have cost.
    let both = plan_original("gpt-5.6-luna", "openai", scope, now, Some("default"))
        .unwrap()
        .remove(0);
    assert_eq!(both.plan_burn_millis, Some(200));
    assert_eq!(both.rates.noncached, Some(rate(/*milli*/ 200).unwrap()));
    assert_eq!(both.rates.output, Some(rate(/*milli*/ 1200).unwrap()));
    assert_eq!(both.rates.read, Some(rate(/*milli*/ 20).unwrap()));
    assert_eq!(both.rates.write, Some(rate(/*milli*/ 250).unwrap()));

    // A metered row reached through a subscription: the vendor published API
    // rates and no plan figure, so the equivalent is stated and the plan rate
    // is not invented.
    let metered = plan_original("gpt-6-sol", "openai", scope, now, Some("default"))
        .unwrap()
        .remove(0);
    assert_eq!(metered.basis, Basis::PlanEquivalent);
    assert_eq!(metered.plan_burn_millis, None);
    assert_eq!(metered.rates.noncached, Some(rate(/*milli*/ 2000).unwrap()));
    assert_eq!(metered.rates.output, Some(rate(/*milli*/ 10000).unwrap()));
    assert_eq!(metered.rates.read, Some(rate(/*milli*/ 200).unwrap()));

    // Nothing is stated for another vendor's metered row (its own API key may
    // pay for it), another provider's row, an unknown slug, or a tier the
    // catalogue does not quote.
    for (model, provider, tier) in [
        ("claude-opus-5", "anthropic", None),
        ("deepseek-flash", "deepseek", None),
        ("glm-5.3", "zai", None),
        ("claude-opus-5-plan", "anthropic", None),
        ("no-such-model", "claude-plan", None),
        ("gpt-5.6-luna", "openai", Some("priority")),
    ] {
        assert_eq!(
            plan_original(model, provider, scope, now, tier).unwrap(),
            vec![],
            "{provider}/{model}"
        );
    }
}

/// A provider whose own credential is a plan login carries no Codex auth mode.
/// Keying the plan side only on `AuthMode` dropped exactly that shape.
#[test]
fn accounting_provider_held_plan_login_takes_the_plan_side() {
    use crate::config::AccountingMode;
    use crate::config::PriceAuthority;
    let provider = codex_model_provider_info::ModelProviderInfo::create_claude_plan_provider();
    assert!(
        provider.auth.is_some(),
        "fixture must be a command-auth provider"
    );
    let endpoint = provider
        .to_api_provider(/*auth_mode*/ None)
        .expect("claude-plan route")
        .base_url;
    let mode = AccountingMode::Provider {
        scope: Uuid::new_v4(),
        provider_id: "claude-plan".into(),
        wire_api: provider.wire_api,
        approved_endpoint: endpoint.clone(),
        approved_query: None,
        pricing: PriceAuthority::Unavailable,
        basis_source: Default::default(),
    };
    for auth in [None, Some(codex_protocol::auth::AuthMode::Chatgpt)] {
        let bound = super::super::turn_mode(&mode, "claude-plan", &provider, auth, &endpoint);
        assert!(
            matches!(
                bound,
                AccountingMode::Provider {
                    pricing: PriceAuthority::PlanRate,
                    ..
                }
            ),
            "claude-plan must take the plan side for {auth:?}"
        );
    }
    // And the plan side of that provider's own rows is its burn, with the same
    // model's published Anthropic API price as the equivalent.
    let plan = super::plan_original(
        "claude-opus-5-plan",
        "claude-plan",
        Uuid::new_v4(),
        /*accepted_at*/ 10,
        /*tier*/ None,
    )
    .unwrap()
    .remove(0);
    assert_eq!(plan.plan_burn_millis, Some(1000));
    assert_eq!(
        plan.rates.noncached,
        Some(super::rate(/*milli*/ 5000).unwrap())
    );
}

/// The wire name is not the catalogue identity, and pricing is a catalogue
/// lookup. A Claude Plan turn goes out as `claude-opus-5` because that is what
/// Anthropic is asked for, while the row that states its plan rate is keyed
/// `claude-opus-5-plan` - so recording the wire name left every turn on that
/// provider with no rate at all, which is what an operator saw as a ledger
/// full of unknowns.
#[test]
fn pf_60_s03_claude_plan_prices_by_catalogue_identity_not_wire_name() {
    let scope = Uuid::new_v4();
    let now = chrono::DateTime::parse_from_rfc3339("2026-09-22T12:00:00Z")
        .unwrap()
        .timestamp_millis();

    // What the catalogue keys, and what the ledger must therefore record.
    let priced = plan_original(
        codex_model_provider_info::CLAUDE_PLAN_MODEL,
        codex_model_provider_info::CLAUDE_PLAN_PROVIDER_ID,
        scope,
        now,
        /*tier*/ None,
    )
    .expect("the catalogue states a plan rate for this row");
    assert_eq!(priced.len(), 1, "the plan row is priced");
    assert_eq!(priced[0].plan_burn_millis, Some(1000));

    // What this client sends on the wire for that same turn. It is a real
    // catalogue slug - under a different provider - so the lookup does not
    // merely miss, it could have matched the wrong row.
    let wire = codex_model_provider_info::CLAUDE_PLAN_UPSTREAM_MODEL;
    assert_ne!(
        wire,
        codex_model_provider_info::CLAUDE_PLAN_MODEL,
        "this test only means anything while the two differ"
    );
    assert!(
        plan_original(
            wire,
            codex_model_provider_info::CLAUDE_PLAN_PROVIDER_ID,
            scope,
            now,
            /*tier*/ None,
        )
        .expect("lookup succeeds")
        .is_empty(),
        "the wire name states no plan rate under this provider, so recording \
         it is how a plan turn became unpriceable"
    );
}

/// Baseten's sheet has no cache-write charge either, so its snapshot states a
/// zero cache-write rate; a provider not on `NO_CACHE_WRITE_CHARGE` keeps the
/// null cache-write of projection v1, and with it its content identity.
#[test]
fn accounting_baseten_states_a_free_cache_write() -> anyhow::Result<()> {
    let scope = Uuid::from_u128(8);
    let decimal = |text: &str| Decimal::try_from(text.to_owned());
    let snapshot = original(
        "zai-org/GLM-5.2",
        "baseten",
        scope,
        /*accepted_at*/ 1,
        "bundled-models-v1",
    )?
    .remove(0);
    assert_eq!(
        snapshot.rates,
        Rates {
            noncached: Some(decimal("1.4")?),
            read: Some(decimal("0.14")?),
            write: Some(Decimal::default()),
            output: Some(decimal("4.4")?),
        }
    );
    // A provider not on the list keeps the unknown cache-write rate.
    let routed = original(
        "z-ai/glm-5.2",
        "ambient",
        scope,
        /*accepted_at*/ 1,
        "bundled-models-v1",
    )?
    .remove(0);
    assert_eq!(routed.rates.write, None);
    Ok(())
}

/// DeepSeek's price sheet has cache hits and misses and no cache-write charge,
/// so its snapshot states a zero cache-write rate; that is what lets pricing
/// charge the miss when DeepSeek's usage omits cache writes. Its rates change
/// by time of day, and a snapshot states the rate in force at dispatch. Every
/// provider not on `NO_CACHE_WRITE_CHARGE` keeps the null cache-write of
/// projection v1, and with it its content identity.
#[test]
fn accounting_deepseek_states_the_rate_in_force_and_a_free_cache_write() -> anyhow::Result<()> {
    let scope = Uuid::from_u128(7);
    let decimal = |text: &str| Decimal::try_from(text.to_owned());
    let at = |text: &str| -> anyhow::Result<i64> {
        Ok(chrono::DateTime::parse_from_rfc3339(text)?.timestamp_millis())
    };
    let peak_at = at("2026-09-22T02:00:00Z")?; // Tuesday, inside 01:00-04:00 UTC
    let off_peak_at = at("2026-09-22T05:00:00Z")?; // between the peak windows
    for (models, accepted_at, input, read, output) in [
        (
            &["deepseek-flash", "deepseek-v4-flash"][..],
            peak_at,
            "0.3",
            "0.006",
            "1.2",
        ),
        (
            &["deepseek-flash", "deepseek-v4-flash"][..],
            off_peak_at,
            "0.15",
            "0.003",
            "0.6",
        ),
        (&["deepseek-v4-pro"][..], peak_at, "1.32", "0.044", "3.96"),
        (
            &["deepseek-v4-pro"][..],
            off_peak_at,
            "0.66",
            "0.022",
            "1.98",
        ),
    ] {
        for model in models {
            let snapshot =
                original(model, "deepseek", scope, accepted_at, "bundled-models-v1")?.remove(0);
            assert_eq!(
                snapshot.rates,
                Rates {
                    noncached: Some(decimal(input)?),
                    read: Some(decimal(read)?),
                    write: Some(Decimal::default()),
                    output: Some(decimal(output)?),
                },
                "{model} at {accepted_at}"
            );
        }
    }
    let peak = original(
        "deepseek-flash",
        "deepseek",
        scope,
        peak_at,
        "bundled-models-v1",
    )?;
    let off_peak = original(
        "deepseek-flash",
        "deepseek",
        scope,
        off_peak_at,
        "bundled-models-v1",
    )?;
    assert_ne!(peak[0].source_reference, off_peak[0].source_reference);

    // A row whose sheet states no cache-write price leaves writes unpriced.
    let openai = chat_original("gpt-5.4-mini", "openai", scope, /*accepted_at*/ 1000)?.remove(0);
    assert_eq!(openai.rates.write, None);
    Ok(())
}

/// Z.AI's pay-as-you-go API (docs.z.ai pricing) bills input, cached input and
/// output per token, with no cache-write charge.
#[test]
fn accounting_zai_api_states_published_rates_and_a_free_cache_write() -> anyhow::Result<()> {
    let scope = Uuid::from_u128(9);
    let decimal = |text: &str| Decimal::try_from(text.to_owned());
    let now = chrono::DateTime::parse_from_rfc3339("2026-09-25T08:00:00Z")?.timestamp_millis();
    for (model, input, read, output) in [
        ("glm-5.3", "1.4", "0.26", "4.4"),
        ("glm-5.2", "1.4", "0.26", "4.4"),
        ("glm-5.3-flash", "0.15", "0.03", "0.5"),
    ] {
        let snapshot = original(model, "zai", scope, now, "bundled-models-v1")?.remove(0);
        assert_eq!(snapshot.basis, Basis::Billed);
        assert_eq!(
            snapshot.rates,
            Rates {
                noncached: Some(decimal(input)?),
                read: Some(decimal(read)?),
                write: Some(Decimal::default()),
                output: Some(decimal(output)?),
            },
            "{model}"
        );
    }
    Ok(())
}

/// Milliseconds at a UTC instant.
fn utc(text: &str) -> i64 {
    chrono::DateTime::parse_from_rfc3339(text)
        .expect("fixture instant")
        .timestamp_millis()
}

/// #361: every OpenAI API-key row states the Standard rates OpenAI publishes,
/// read 2026-10-09 from developers.openai.com/api/docs/pricing and each
/// model's page: input, cached input, cache writes ("-" or unlisted means the
/// sheet states none) and output in milli-USD per million tokens, and, where
/// the page says prompts above 272K input tokens cost 2x input (cache rates
/// included) and 1.5x output for the full request, that tier.
#[test]
fn openai_api_key_rates_match_the_published_sheet() -> anyhow::Result<()> {
    let scope = Uuid::new_v4();
    let read_on = utc("2026-10-09T19:00:00Z");
    // (model, input, cached input, cache writes, output, long-context tier)
    type Row = (&'static str, u32, u32, Option<u32>, u32, bool);
    let sheet: [Row; 11] = [
        ("gpt-6-astra", 10_000, 1_000, Some(12_500), 50_000, true),
        ("gpt-6-sol", 2_000, 200, Some(2_500), 10_000, true),
        ("gpt-6-luna", 100, 10, Some(125), 500, true),
        ("gpt-5.6-sol", 4_000, 400, Some(5_000), 20_000, true),
        ("gpt-5.6-terra", 2_000, 200, Some(2_500), 12_000, true),
        ("gpt-5.6-luna", 200, 20, Some(250), 1_200, true),
        ("gpt-5.5", 5_000, 500, None, 30_000, true),
        ("gpt-5.4", 2_500, 250, None, 15_000, true),
        ("gpt-5.4-mini", 750, 75, None, 4_500, false),
        ("gpt-5.3-codex", 1_750, 175, None, 14_000, false),
        ("gpt-5.2", 1_750, 175, None, 14_000, false),
    ];
    for (model, input, cached, write, output, long) in sheet {
        for price in [
            responses_original(model, "openai", scope, read_on, /*tier*/ None)?,
            chat_original(model, "openai", scope, read_on)?,
        ] {
            let [price] = price.as_slice() else {
                panic!("{model}: exactly one price");
            };
            assert_eq!(
                price.rates,
                Rates {
                    noncached: Some(rate(input)?),
                    read: Some(rate(cached)?),
                    write: write.map(rate).transpose()?,
                    output: Some(rate(output)?),
                },
                "{model}"
            );
            let tier = long
                .then(|| -> anyhow::Result<LongContext> {
                    Ok(LongContext {
                        above_input_tokens: 272_000.try_into()?,
                        rates: Rates {
                            noncached: Some(rate(input * 2)?),
                            // GPT-5.5 and GPT-5.4's pages state 2x input and
                            // 1.5x output only: cache reads above 272K have no
                            // stated price.
                            read: (!matches!(model, "gpt-5.5" | "gpt-5.4"))
                                .then(|| rate(cached * 2))
                                .transpose()?,
                            write: write.map(|write| rate(write * 2)).transpose()?,
                            output: Some(rate(output * 3 / 2)?),
                        },
                    })
                })
                .transpose()?;
            assert_eq!(price.long_context, tier, "{model}");
            assert_eq!(price.basis, Basis::Billed, "{model}");
        }
        // Fast (formerly Priority), Flex and unknown tiers have no Standard
        // price here: no price, never the Standard one.
        for tier in ["priority", "flex", "auto"] {
            assert!(
                responses_original(model, "openai", scope, read_on, Some(tier))?.is_empty(),
                "{model} {tier}"
            );
        }
    }
    // GPT-5.6 Sol's promotional price holds "at least through November 21,
    // 2026"; after that the catalogue states no price until it is re-read.
    assert!(
        !responses_original(
            "gpt-5.6-sol",
            "openai",
            scope,
            utc("2026-11-21T23:59:59.999Z"),
            None
        )?
        .is_empty()
    );
    assert!(
        responses_original(
            "gpt-5.6-sol",
            "openai",
            scope,
            utc("2026-11-22T00:00:00Z"),
            None
        )?
        .is_empty()
    );
    // A stated cache write or tier extends the price identity; changing either
    // changes it, so a record states exactly which sheet priced it.
    let luna = responses_original("gpt-5.6-luna", "openai", scope, read_on, None)?.remove(0);
    let v1 = (
        "openai-responses-api-key-bundled-v1",
        "openai",
        "gpt-5.6-luna",
        "api_key",
        "default",
        "USD/million",
        200,
        1200,
        Some(20),
    );
    let reference = serde_json::to_vec(&(
        v1,
        "stated-v1",
        Some(250),
        Some((272_000, 400, 1800, Some(40), Some(500))),
    ))?;
    assert_eq!(
        luna.source_reference,
        Uuid::new_v5(&Uuid::NAMESPACE_OID, &reference)
    );
    Ok(())
}

/// A ChatGPT-login turn on an OpenAI row states the same published API price,
/// cache writes and long-context tier included, as its not-spent equivalent.
#[test]
fn openai_plan_equivalent_states_cache_write_and_long_context() -> anyhow::Result<()> {
    let scope = Uuid::new_v4();
    let at = utc("2026-10-09T19:00:00Z");
    let plan = plan_original("gpt-5.6-luna", "openai", scope, at, None)?.remove(0);
    let billed = responses_original("gpt-5.6-luna", "openai", scope, at, None)?.remove(0);
    assert_eq!(plan.basis, Basis::PlanEquivalent);
    assert_eq!(plan.plan_burn_millis, Some(200));
    assert_eq!(plan.rates, billed.rates);
    assert_eq!(plan.long_context, billed.long_context);
    Ok(())
}

/// A row that is not spawn-eligible still states its published price for
/// accounting; one that states none stays unpriced.
#[test]
fn disabled_rows_state_their_published_price() -> anyhow::Result<()> {
    let catalog = codex_models_manager::bundled_models_response()?;
    let row = catalog
        .models
        .iter()
        .find(|row| row.slug == "gpt-5.4-mini")
        .expect("bundled row")
        .clone();
    assert!(
        !row.orchestration
            .as_ref()
            .is_some_and(ModelOrchestrationMetadata::is_spawn_eligible)
    );
    assert_eq!(
        billing_for(std::slice::from_ref(&row), "gpt-5.4-mini", "openai")
            .and_then(|billing| billing.api_key_rates()),
        Some((750, 4500, Some(75)))
    );
    let mut unpriced = row;
    unpriced.orchestration = Some(ModelOrchestrationMetadata::Disabled {
        provider_id: "openai".into(),
        capability: codex_protocol::openai_models::ModelCapabilityTier::Legacy,
        reason: "fixture".into(),
        billing: None,
    });
    assert_eq!(billing_for(&[unpriced], "gpt-5.4-mini", "openai"), None);
    Ok(())
}
