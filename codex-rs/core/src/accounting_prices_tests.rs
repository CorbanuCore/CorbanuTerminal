use super::*;
use pretty_assertions::assert_eq;

#[test]
fn accounting_chat_prices_exact_source_and_unknown() -> anyhow::Result<()> {
    let scope = Uuid::new_v4();
    let source = "openai-chat-api-key-bundled-v1";
    let first = chat_original("gpt-5.6-sol", "openai", scope, 1000)?.remove(0);
    let tuple = serde_json::to_vec(&(
        source,
        "openai",
        "gpt-5.6-sol",
        "api_key",
        "default",
        "USD/million",
        5000,
        30000,
        Some(500),
    ))?;
    assert_eq!(
        first.source_reference,
        Uuid::new_v5(&Uuid::NAMESPACE_OID, &tuple)
    );
    assert_eq!(
        first.rates,
        Rates {
            noncached: Some(rate(5000)?),
            output: Some(rate(30000)?),
            read: Some(rate(500)?),
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
    let second = chat_original("gpt-5.6-sol", "openai", scope, 2000)?.remove(0);
    assert_ne!(first.id, second.id);
    assert_eq!(first.source_reference, second.source_reference);
    for model in [
        "gpt-6-astra",
        "remote-only",
        "Gpt-5.6-sol",
        "openai/gpt-5.6-sol",
        "claude-opus-5",
    ] {
        assert!(chat_original(model, "openai", scope, 1000)?.is_empty());
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
    assert_eq!(billing_for(&[row.clone()], &row.slug, "openrouter"), None);
    let mut disabled = row.clone();
    disabled.orchestration = Some(ModelOrchestrationMetadata::Disabled {
        provider_id: "openai".into(),
        capability: codex_protocol::openai_models::ModelCapabilityTier::Frontier,
        reason: "fixture".into(),
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
        assert!(billed("fixture", "openai", &billing, scope, 1000, source)?.is_empty());
    }
    for read in [None, Some(0)] {
        let billing = ModelBilling::Metered {
            input_milli_usd_per_million_tokens: 1,
            output_milli_usd_per_million_tokens: 2,
            cached_input_milli_usd_per_million_tokens: read,
        };
        let projected = billed("fixture", "openai", &billing, scope, 1000, source)?.remove(0);
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
    let first = responses_original("gpt-5.6-sol", "openai", scope, 1000, None)
        .unwrap()
        .remove(0);
    let later = responses_original("gpt-5.6-sol", "openai", scope, 2000, Some("default"))
        .unwrap()
        .remove(0);
    assert_eq!(
        first.rates,
        Rates {
            noncached: Some(rate(5000).unwrap()),
            output: Some(rate(30000).unwrap()),
            read: Some(rate(500).unwrap()),
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
        "gpt-5.6-sol",
        "api_key",
        "default",
        "USD/million",
        5000,
        30000,
        Some(500),
    ))
    .unwrap();
    assert_eq!(
        first.source_reference,
        Uuid::new_v5(&Uuid::NAMESPACE_OID, &source)
    );
    for model in [
        "gpt-6-astra",
        "remote-only",
        "Gpt-5.6-sol",
        "openai/gpt-5.6-sol",
        "claude-opus-5",
    ] {
        assert!(
            responses_original(model, "openai", scope, 1000, None)
                .unwrap()
                .is_empty()
        );
    }
    for tier in ["priority", "flex", "auto", "unknown"] {
        assert!(
            responses_original("gpt-5.6-sol", "openai", scope, 1000, Some(tier))
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
            responses_project("fixture", scope, &billing, 1000)
                .unwrap()
                .is_empty()
        );
    }
    for read in [None, Some(0)] {
        let billing = ModelBilling::Metered {
            input_milli_usd_per_million_tokens: 0,
            output_milli_usd_per_million_tokens: 0,
            cached_input_milli_usd_per_million_tokens: read,
        };
        let value = responses_project("fixture", scope, &billing, 1000)
            .unwrap()
            .remove(0);
        assert_eq!(value.rates.read, read.map(|value| rate(value).unwrap()));
        assert_eq!(value.rates.write, None);
    }
    let anthropic = anthropic_original("claude-opus-5", "anthropic", scope, 1000)
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
    assert_eq!(
        anthropic.source_reference,
        Uuid::new_v5(&Uuid::NAMESPACE_OID, &source)
    );
}

#[test]
fn accounting_bundled_prices_are_exact_prospective_and_content_identified() {
    let scope = Uuid::new_v4();
    let first = anthropic_original("claude-opus-5", "anthropic", scope, 1000)
        .unwrap()
        .remove(0);
    let second = anthropic_original("claude-opus-5", "anthropic", scope, 2000)
        .unwrap()
        .remove(0);
    assert_ne!(first.id, second.id);
    assert_eq!(first.source_reference, second.source_reference);
    assert_eq!(
        first.rates,
        Rates {
            noncached: Some(rate(5000).unwrap()),
            read: Some(rate(500).unwrap()),
            output: Some(rate(25000).unwrap()),
            write: None,
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
    let fable = anthropic_original("claude-fable-5-1", "anthropic", scope, 1000)
        .unwrap()
        .remove(0);
    assert_ne!(fable.source_reference, first.source_reference);
    assert_eq!(
        fable.rates,
        Rates {
            noncached: Some(rate(10000).unwrap()),
            read: Some(rate(1000).unwrap()),
            output: Some(rate(50000).unwrap()),
            write: None,
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
            anthropic_original(model, "anthropic", Uuid::nil(), 10).unwrap(),
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
                10,
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
    };
    let priced = billed(
        "claude-opus-5",
        "anthropic",
        &auth_dependent,
        Uuid::nil(),
        10,
        "anthropic-bundled-v1",
    )
    .unwrap()
    .remove(0);
    assert_eq!(priced.basis, Basis::Billed);
    assert_eq!(priced.plan_burn_millis, None);
    assert_eq!(priced.rates.noncached, Some(rate(5000).unwrap()));
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
    };
    let quote = billed(
        "fixture",
        "anthropic",
        &billing,
        Uuid::nil(),
        10,
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
            10,
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
            -1,
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
    // A burn-only row: the plan rate is stated, and no API price is invented.
    let plan = plan_original("claude-opus-5-plan", "claude-plan", scope, now, None)
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
        (None, None, None, None)
    );
    assert_eq!(plan.provider, "claude-plan");

    // An auth-dependent row states both: the plan rate that applied and the API
    // rates the same tokens would have cost.
    let both = plan_original("gpt-5.6-luna", "openai", scope, now, Some("default"))
        .unwrap()
        .remove(0);
    assert_eq!(both.plan_burn_millis, Some(200));
    assert_eq!(both.rates.noncached, Some(rate(1000).unwrap()));
    assert_eq!(both.rates.output, Some(rate(6000).unwrap()));
    assert_eq!(both.rates.read, Some(rate(100).unwrap()));

    // A scheduled row is resolved at the dispatch instant, not read as a range.
    let peak = chrono::DateTime::parse_from_rfc3339("2026-09-21T07:00:00Z")
        .unwrap()
        .timestamp_millis();
    assert_eq!(
        plan_original("glm-5.3", "zai", scope, peak, None)
            .unwrap()
            .remove(0)
            .plan_burn_millis,
        Some(3000)
    );
    assert_eq!(
        plan_original("glm-5.3", "zai", scope, now, None)
            .unwrap()
            .remove(0)
            .plan_burn_millis,
        Some(1000)
    );

    // Nothing is stated for a metered row, another provider's row, an unknown
    // slug, or a tier the catalogue does not quote.
    for (model, provider, tier) in [
        ("claude-opus-5", "anthropic", None),
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
        .to_api_provider(None)
        .expect("claude-plan route")
        .base_url;
    let mode = AccountingMode::Provider {
        scope: Uuid::new_v4(),
        provider_id: "claude-plan".into(),
        wire_api: provider.wire_api,
        approved_endpoint: endpoint.clone(),
        approved_query: None,
        pricing: PriceAuthority::Unavailable,
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
    // And the plan side of that provider's own rows is burn with no invented price.
    let plan = super::plan_original(
        "claude-opus-5-plan",
        "claude-plan",
        Uuid::new_v4(),
        10,
        None,
    )
    .unwrap()
    .remove(0);
    assert_eq!(plan.plan_burn_millis, Some(1000));
    assert_eq!(plan.rates.noncached, None);
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
        None,
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
            None,
        )
        .expect("lookup succeeds")
        .is_empty(),
        "the wire name states no plan rate under this provider, so recording \
         it is how a plan turn became unpriceable"
    );
}
