use super::*;
use pretty_assertions::assert_eq;

#[test]
fn accounting_bundled_prices_are_exact_prospective_and_content_identified() {
    let scope = Uuid::new_v4();
    let first = original("claude-opus-5", scope, 1000).unwrap().remove(0);
    let second = original("claude-opus-5", scope, 2000).unwrap().remove(0);
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
    let fable = original("claude-fable-5-1", scope, 1000).unwrap().remove(0);
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
        assert_eq!(original(model, Uuid::nil(), 10).unwrap(), vec![]);
    }
    for billing in [
        ModelBilling::Local,
        ModelBilling::Plan {
            relative_burn_millis: 1000,
        },
        ModelBilling::AuthDependent {
            plan_relative_burn_millis: 1000,
            api_key_input_milli_usd_per_million_tokens: 5000,
            api_key_output_milli_usd_per_million_tokens: 25000,
            api_key_cached_input_milli_usd_per_million_tokens: None,
        },
    ] {
        assert_eq!(
            project("claude-opus-5", Uuid::nil(), &billing, 10).unwrap(),
            vec![]
        );
    }
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
    let quote = project("fixture", Uuid::nil(), &billing, 10)
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
        project("fixture", Uuid::nil(), &changed, 10).unwrap()[0].source_reference
    );
    assert!(project("fixture", Uuid::nil(), &billing, -1).is_err());
}
