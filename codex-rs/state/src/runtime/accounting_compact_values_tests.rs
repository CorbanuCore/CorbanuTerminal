use super::super::super::super::Attempt;
use super::super::super::super::DisplayAmount;
use super::super::super::super::Observation;
use super::super::super::super::Snapshot;
use super::super::super::super::quote_observations;
use super::*;
use pretty_assertions::assert_eq;
use serde_json::Value;
use serde_json::json;

fn literal() -> Value {
    json!({"version":1, "known":[0,0,0,0,0,0,0], "unknown":[0,0,0,0,0,0,0],
        "known_usd":"0", "unknown_estimates":0, "attempts":1})
}

fn decode(raw: &Value) -> CompactValues {
    CompactValues::decode(&raw.to_string()).unwrap()
}

fn assert_literal(value: &CompactValues, expected: Value) {
    let encoded = value.encode().unwrap();
    assert_eq!(serde_json::from_str::<Value>(&encoded).unwrap(), expected);
    assert_eq!(CompactValues::decode(&encoded).unwrap(), *value);
    assert_eq!(
        CompactValues::from_day_totals(&value.to_day_totals().unwrap()).unwrap(),
        *value
    );
}

#[test]
fn canonical_amounts_round_trip_at_all_required_scales_and_u128_max() {
    for (text, coefficient, scale) in [
        ("0", 0, 0),
        ("7", 7, 0),
        ("0.000153", 153, 6),
        ("0.000000000000000001", 1, 18),
        ("0.000000000000000000000001", 1, 24),
        ("340282366920938463463374607431768211455", u128::MAX, 0),
        ("340282366920938.463463374607431768211455", u128::MAX, 24),
    ] {
        let mut raw = literal();
        raw["known_usd"] = json!(text);
        let value = decode(&raw);
        assert_literal(&value, raw);
        assert_eq!(
            value.to_day_totals().unwrap(),
            DayTotals {
                known_usd: Decimal { coefficient, scale },
                attempts: 1,
                ..DayTotals::default()
            }
        );
    }
}

#[test]
fn malformed_and_noncanonical_stored_amounts_are_rejected() {
    for text in [
        "",
        ".",
        ".1",
        "1.",
        "1.2.3",
        "+1",
        "-1",
        "-0",
        "1e0",
        "1E0",
        " 1",
        "1 ",
        "1\n",
        "NaN",
        "Infinity",
        "١",
        "00",
        "01",
        "00.1",
        "0.0",
        "1.0",
        "1.2300",
        "0.0000000000000000000000001",
        "340282366920938463463374607431768211456",
        "340282366920938.463463374607431768211456",
        "1000000000000000000000000000000000000000",
    ] {
        let mut raw = literal();
        raw["known_usd"] = json!(text);
        assert!(CompactValues::decode(&raw.to_string()).is_err(), "{text}");
    }
    for invalid in [
        json!(0),
        json!(1.5),
        json!(true),
        Value::Null,
        json!([]),
        json!({}),
    ] {
        let mut raw = literal();
        raw["known_usd"] = invalid;
        assert!(CompactValues::decode(&raw.to_string()).is_err(), "{raw}");
    }
}

#[test]
fn schema_requires_exact_fields_version_object_and_array_arity() {
    for field in literal().as_object().unwrap().keys() {
        let mut raw = literal();
        raw.as_object_mut().unwrap().remove(field);
        assert!(CompactValues::decode(&raw.to_string()).is_err(), "{field}");
    }
    for (field, value) in [
        ("extra", json!(0)),
        ("version", json!(0)),
        ("version", json!(2)),
        ("version", json!(256)),
        ("version", json!(1.0)),
        ("version", json!("1")),
        ("version", json!(-1)),
        ("version", json!(true)),
        ("version", Value::Null),
    ] {
        let mut raw = literal();
        raw[field] = value;
        assert!(CompactValues::decode(&raw.to_string()).is_err(), "{raw}");
    }
    for field in ["known", "unknown"] {
        for value in [
            json!([]),
            json!([0, 0, 0, 0, 0, 0]),
            json!([0, 0, 0, 0, 0, 0, 0, 0]),
            Value::Null,
            json!({}),
        ] {
            let mut raw = literal();
            raw[field] = value;
            assert!(CompactValues::decode(&raw.to_string()).is_err(), "{raw}");
        }
    }
    for text in [
        "null",
        "true",
        "0",
        "[]",
        "{}",
        "{",
        "[1,[0,0,0,0,0,0,0],[0,0,0,0,0,0,0],\"0\",0,1]",
    ] {
        assert!(CompactValues::decode(text).is_err(), "{text}");
    }
    let raw = literal().to_string();
    assert!(CompactValues::decode(&raw.replacen('{', "{\"attempts\":1,", 1)).is_err());
    assert!(CompactValues::decode(&format!("{raw} false")).is_err());
}

#[test]
fn every_count_rejects_wrong_types_negative_and_overflow() {
    for invalid in [
        json!(-1),
        json!(1.0),
        json!(true),
        json!("1"),
        Value::Null,
        json!(9_223_372_036_854_775_808_u64),
    ] {
        for field in ["attempts", "unknown_estimates", "known", "unknown"] {
            let indices = if field == "known" || field == "unknown" {
                7
            } else {
                1
            };
            for index in 0..indices {
                let mut raw = literal();
                if indices == 7 {
                    raw[field][index] = invalid.clone();
                } else {
                    raw[field] = invalid.clone();
                }
                assert!(CompactValues::decode(&raw.to_string()).is_err(), "{raw}");
            }
        }
    }
}

#[test]
fn empty_and_unknown_populations_cannot_invent_known_values() {
    for field in ["unknown", "known"] {
        for index in 0..7 {
            let mut raw = literal();
            raw["attempts"] = json!(0);
            raw[field][index] = json!(1);
            assert!(CompactValues::decode(&raw.to_string()).is_err());
            raw["attempts"] = json!(1);
            raw["unknown"][index] = json!(2);
            assert!(CompactValues::decode(&raw.to_string()).is_err());
            raw["unknown"][index] = json!(1);
            raw["known"][index] = json!(1);
            assert!(CompactValues::decode(&raw.to_string()).is_err());
        }
    }
    for (attempts, unknown, usd) in [(0, 1, "0"), (1, 2, "0"), (0, 0, "0.1")] {
        let mut raw = literal();
        raw["attempts"] = json!(attempts);
        raw["unknown_estimates"] = json!(unknown);
        raw["known_usd"] = json!(usd);
        assert!(CompactValues::decode(&raw.to_string()).is_err());
    }
    let empty = CompactValues::from_day_totals(&DayTotals::default()).unwrap();
    let zero = decode(&literal());
    let unknown = decode(&json!({"version":1, "known":[0,0,0,0,0,0,0],
        "unknown":[1,1,1,1,1,1,1], "known_usd":"0", "unknown_estimates":1, "attempts":1}));
    assert_eq!(
        [empty, zero, unknown].map(|value| {
            let totals = value.to_day_totals().unwrap();
            (
                totals.full_usd(),
                totals.measured.map(|metric| metric.full()),
            )
        }),
        [
            (None, [Some(0); 7]),
            (Some(Decimal::default()), [Some(0); 7]),
            (None, [None; 7])
        ]
    );
}

#[test]
fn composition_preserves_all_seven_independent_populations() {
    let left = decode(&json!({"version":1, "known":[10,20,30,40,50,60,70],
        "unknown":[0,1,2,3,4,5,6], "known_usd":"0.000153", "unknown_estimates":1, "attempts":7}));
    let right = decode(&json!({"version":1, "known":[7,6,5,4,3,2,1],
        "unknown":[6,5,4,3,2,1,0], "known_usd":"0", "unknown_estimates":7, "attempts":7}));
    let sum = left.checked_add(&right).unwrap();
    assert_literal(
        &sum,
        json!({"version":1, "known":[17,26,35,44,53,62,71],
        "unknown":[6,6,6,6,6,6,6], "known_usd":"0.000153", "unknown_estimates":8, "attempts":14}),
    );
    assert_eq!(sum.to_day_totals().unwrap().full_usd(), None);
    assert_eq!(right.checked_add(&left).unwrap(), sum);
    let empty = CompactValues::from_day_totals(&DayTotals::default()).unwrap();
    assert_eq!(empty.checked_add(&left).unwrap(), left);
    assert_eq!(left.checked_add(&empty).unwrap(), left);
}

#[test]
fn exact_addition_precedes_six_place_half_even_display() {
    let mut raw = literal();
    raw["known_usd"] = json!("0.0000004");
    let value = decode(&raw);
    let sum = value.checked_add(&value).unwrap();
    raw["attempts"] = json!(2);
    raw["known_usd"] = json!("0.0000008");
    assert_literal(&sum, raw);
    assert_eq!(
        sum.to_day_totals().unwrap().full_usd().unwrap().display(),
        DisplayAmount {
            text: "0.000001".into(),
            rounded: true,
            nonzero_sub_micro: true
        }
    );
    for (amount, expected) in [("0.0000005", "0.000000"), ("0.0000015", "0.000002")] {
        let mut raw = literal();
        raw["known_usd"] = json!(amount);
        assert_eq!(
            decode(&raw)
                .to_day_totals()
                .unwrap()
                .known_usd
                .display()
                .text,
            expected
        );
    }
}

#[test]
fn mixed_scales_and_known_plus_unknown_keep_exact_subtotals() {
    let mut raw = literal();
    raw["known_usd"] = json!("0.00015");
    let left = decode(&raw);
    raw["known_usd"] = json!("0.000003");
    let known = left.checked_add(&decode(&raw)).unwrap();
    raw["known_usd"] = json!("0.000153");
    raw["attempts"] = json!(2);
    assert_literal(&known, raw);
    assert_eq!(
        known.to_day_totals().unwrap().full_usd(),
        Some(Decimal {
            coefficient: 153,
            scale: 6
        })
    );
    let unknown = decode(&json!({"version":1, "known":[0,0,0,0,0,0,0],
        "unknown":[1,1,1,1,1,1,1], "known_usd":"0", "unknown_estimates":1, "attempts":1}));
    let combined = known.checked_add(&unknown).unwrap();
    assert_literal(
        &combined,
        json!({"version":1, "known":[0,0,0,0,0,0,0],
        "unknown":[1,1,1,1,1,1,1], "known_usd":"0.000153", "unknown_estimates":1, "attempts":3}),
    );
    assert_eq!(combined.to_day_totals().unwrap().full_usd(), None);
}

#[test]
fn every_checked_addition_rejects_overflow_without_mutating_operands() {
    for field in [
        "known",
        "unknown",
        "unknown_estimates",
        "attempts",
        "known_usd",
    ] {
        let indices = if field == "known" || field == "unknown" {
            7
        } else {
            1
        };
        for index in 0..indices {
            let mut lhs = literal();
            let mut rhs = literal();
            lhs["attempts"] = json!(i64::MAX);
            let expected = match field {
                "known" | "unknown" => {
                    lhs[field][index] = json!(i64::MAX);
                    rhs[field][index] = json!(1);
                    format!("{field} overflow")
                }
                "known_usd" => {
                    lhs[field] = json!(u128::MAX.to_string());
                    rhs[field] = json!("1");
                    "decimal sum overflow".into()
                }
                "unknown_estimates" => {
                    lhs[field] = json!(i64::MAX);
                    rhs[field] = json!(1);
                    "unknown estimate overflow".into()
                }
                _ => "attempt overflow".into(),
            };
            let left = decode(&lhs);
            let right = decode(&rhs);
            assert_eq!(left.checked_add(&right).unwrap_err().to_string(), expected);
            assert_literal(&left, lhs);
            assert_literal(&right, rhs);
        }
    }
    let mut raw = literal();
    raw["known_usd"] = json!(u128::MAX.to_string());
    let left = decode(&raw);
    raw["known_usd"] = json!("0.000000000000000000000001");
    let right = decode(&raw);
    let before = (left.clone(), right.clone());
    assert_eq!(
        left.checked_add(&right).unwrap_err().to_string(),
        "decimal alignment overflow"
    );
    assert_eq!((left, right), before);
}

#[test]
fn all_entry_points_validate_in_memory_values_without_repair() {
    let valid = decode(&literal());
    for amount in [
        Decimal {
            coefficient: 10,
            scale: 1,
        },
        Decimal {
            coefficient: 0,
            scale: 1,
        },
        Decimal {
            coefficient: 1,
            scale: 25,
        },
        Decimal {
            coefficient: 1,
            scale: u32::MAX,
        },
    ] {
        let totals = DayTotals {
            known_usd: amount,
            attempts: 1,
            ..DayTotals::default()
        };
        assert!(CompactValues::from_day_totals(&totals).is_err());
        let invalid = CompactValues { totals };
        let before = invalid.clone();
        assert!(invalid.encode().is_err());
        assert!(invalid.to_day_totals().is_err());
        assert!(invalid.checked_add(&valid).is_err());
        assert!(valid.checked_add(&invalid).is_err());
        assert_eq!(invalid, before);
    }
}

#[test]
fn actual_quote_reduction_converts_partial_and_twenty_four_place_amounts() {
    let attempt: Attempt = serde_json::from_value(json!({
        "attempt_id":"00000000-0000-0000-0000-000000000001",
        "request_id":"00000000-0000-0000-0000-000000000002",
        "thread_id":"00000000-0000-0000-0000-000000000003", "turn":"fixture",
        "retry_of":null, "provider":"synthetic", "model":"fixture",
        "scope":"00000000-0000-0000-0000-000000000004",
        "dialect":"NativeAnthropic", "dispatched_at_ms":100
    }))
    .unwrap();
    let mut snapshot: Snapshot = serde_json::from_value(json!({
        "id":"00000000-0000-0000-0000-000000000010", "provider":"synthetic",
        "model":"fixture", "scope":attempt.scope, "currency":"USD", "unit":"PerMillionTokens",
        "rates":{"noncached":"3","read":"0.3","write":null,"output":null},
        "source_reference":"00000000-0000-0000-0000-000000000011",
        "source_kind":"ProviderPublished", "observed_at_ms":0, "approved_at_ms":0,
        "effective_from_ms":0, "effective_end_ms":null
    }))
    .unwrap();
    for (patch, expected) in [
        (
            json!({"input":50,"read":10}),
            json!({"version":1, "known":[0,50,10,0,0,0,0],
            "unknown":[1,0,0,1,1,1,1], "known_usd":"0.000153", "unknown_estimates":1, "attempts":1}),
        ),
        (
            json!({"input":1,"read":0,"write":0,"output":0}),
            json!({"version":1, "known":[1,1,0,0,0,0,1],
            "unknown":[0,0,0,0,0,1,0], "known_usd":"0.000000000000000000000001", "unknown_estimates":0, "attempts":1}),
        ),
    ] {
        let row: Observation = serde_json::from_value(json!({"revision":1,
            "source":attempt.attempt_id, "sequence":1, "patch":patch}))
        .unwrap();
        let quote = quote_observations(&attempt, &[row], &[snapshot.clone()]).unwrap();
        let mut totals = DayTotals::default();
        totals.add(&quote).unwrap();
        assert_literal(&CompactValues::from_day_totals(&totals).unwrap(), expected);
        snapshot.rates.noncached =
            Some(Decimal::try_from("0.000000000000000001".to_owned()).unwrap());
    }
    assert!(Decimal::try_from("0.000000000000000000000001".to_owned()).is_err());
    assert!(Decimal::try_from(u128::MAX.to_string()).is_err());
}
