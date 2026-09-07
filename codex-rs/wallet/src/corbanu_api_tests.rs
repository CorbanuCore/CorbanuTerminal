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

#[test]
fn created_keys_accept_the_public_api_shape_without_changing_daemon_ipc() {
    let created = serde_json::from_value::<CreatedApiKey>(serde_json::json!({
        "id": "2f9350c1-0cf6-4af1-bb90-cc693c923bb3",
        "key": "cbn_live_secret",
        "displayPrefix": "cbn_live_4D7K",
    }))
    .expect("deserialize the public key response");
    let key = GatewayKey::from(created);

    assert_eq!(key.key_id, "2f9350c1-0cf6-4af1-bb90-cc693c923bb3");
    assert_eq!(key.api_key, "cbn_live_secret");
    assert_eq!(key.display_prefix, "cbn_live_4D7K");
    assert_eq!(
        serde_json::to_value(key).expect("serialize the daemon IPC key"),
        serde_json::json!({
            "key_id": "2f9350c1-0cf6-4af1-bb90-cc693c923bb3",
            "api_key": "cbn_live_secret",
            "display_prefix": "cbn_live_4D7K",
        })
    );
}

#[test]
fn gateway_key_debug_output_redacts_plaintext() {
    let key = GatewayKey {
        key_id: "key-id".to_string(),
        api_key: "cbn_live_debug_canary".to_string(),
        display_prefix: "cbn_live_debug".to_string(),
    };

    let debug = format!("{key:?}");
    assert!(!debug.contains("cbn_live_debug_canary"));
    assert!(debug.contains("[REDACTED]"));
    assert!(debug.contains("key-id"));
}

#[test]
fn post_settlement_account_creates_a_key_only_without_an_active_one() {
    let account = |keys| CorbanuApiAccount {
        balance: CorbanuApiBalance {
            balance_microusd: "1000000".to_string(),
            reserved_microusd: "0".to_string(),
            available_microusd: "1000000".to_string(),
            balance_usd: "1".to_string(),
            reserved_usd: "0".to_string(),
            available_usd: "1".to_string(),
        },
        keys,
        models: Vec::new(),
    };
    let key = |revoked_at| CorbanuApiKeySummary {
        id: "2f9350c1-0cf6-4af1-bb90-cc693c923bb3".to_string(),
        display_prefix: "cbn_live_4D7K".to_string(),
        created_at: "2026-08-30T00:00:00.000Z".to_string(),
        revoked_at,
        last_used_at: None,
    };

    assert!(needs_initial_api_key(&account(Vec::new())));
    assert!(needs_initial_api_key(&account(vec![key(Some(
        "2026-08-30T01:00:00.000Z".to_string(),
    ))])));
    assert!(!needs_initial_api_key(&account(vec![key(None)])));
}
