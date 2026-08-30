use super::*;

#[test]
fn operations_use_the_backend_typed_wire_shape() {
    assert_eq!(
        serde_json::to_value(CorbanuApiOperation::TopUpIntent {
            amount_usd: "7.25".to_string(),
        })
        .expect("serialize top-up operation"),
        serde_json::json!({ "kind": "top_up_intent", "amountUsd": "7.25" }),
    );
    assert_eq!(
        serde_json::to_value(CorbanuApiOperation::RevokeKey {
            key_id: "2f9350c1-0cf6-4af1-bb90-cc693c923bb3".to_string(),
        })
        .expect("serialize revoke operation"),
        serde_json::json!({
            "kind": "revoke_key",
            "keyId": "2f9350c1-0cf6-4af1-bb90-cc693c923bb3",
        }),
    );
}

#[test]
fn top_up_amounts_are_exact_microdollars_without_floating_point() {
    let valid = [
        ("1", "1000000"),
        ("0.000001", "1"),
        ("7.25", "7250000"),
        ("7.250000", "7250000"),
    ];
    for (value, expected) in valid {
        assert_eq!(
            parse_usd_micros(value).expect("valid amount"),
            expected,
            "{value}"
        );
    }
    for invalid in ["", "0", ".5", "1.", "-1", " 1", "1 ", "1e3", "1.0000001"] {
        assert!(parse_usd_micros(invalid).is_err(), "{invalid}");
    }
}
