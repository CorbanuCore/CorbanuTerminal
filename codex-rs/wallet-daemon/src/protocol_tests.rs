use super::*;

#[test]
fn legacy_timed_unlock_requests_remain_compatible() {
    let request = serde_json::from_value::<Request>(serde_json::json!({
        "type": "unlock",
        "passcode": "test-passcode",
        "duration_seconds": 900
    }))
    .expect("legacy timed unlock request");
    let Request::Unlock {
        duration_seconds,
        one_action,
        ..
    } = request
    else {
        panic!("expected unlock request");
    };
    assert_eq!((duration_seconds, one_action), (900, false));
}

#[test]
fn new_one_action_request_is_accepted_by_the_legacy_wire_shape() {
    #[derive(Deserialize)]
    #[serde(tag = "type", rename_all = "snake_case")]
    enum LegacyRequest {
        Unlock {
            passcode: String,
            duration_seconds: u64,
        },
    }

    let encoded = serde_json::to_value(Request::Unlock {
        passcode: "test-passcode".to_string(),
        duration_seconds: 300,
        one_action: true,
    })
    .expect("serialize one-action request");
    let LegacyRequest::Unlock {
        passcode,
        duration_seconds,
    } = serde_json::from_value(encoded).expect("legacy daemon wire shape");
    assert_eq!(
        (passcode.as_str(), duration_seconds),
        ("test-passcode", 300)
    );
}

#[test]
fn corbanu_api_operations_round_trip_without_erasing_the_operation_boundary() {
    let operations = [
        codex_wallet::CorbanuApiOperation::Account,
        codex_wallet::CorbanuApiOperation::TopUpIntent {
            amount_usd: "7.25".to_string(),
        },
        codex_wallet::CorbanuApiOperation::CreateKey,
        codex_wallet::CorbanuApiOperation::RevokeKey {
            key_id: "2f9350c1-0cf6-4af1-bb90-cc693c923bb3".to_string(),
        },
    ];
    for operation in operations {
        let request = Request::CorbanuApiOperation {
            capability: "secret-capability".to_string(),
            gateway_origin: "https://api.corbanu.example".to_string(),
            operation: operation.clone(),
        };
        let encoded = serde_json::to_value(&request).expect("serialize operation request");
        let decoded: Request =
            serde_json::from_value(encoded).expect("deserialize operation request");
        let Request::CorbanuApiOperation {
            capability,
            gateway_origin,
            operation: decoded_operation,
        } = decoded
        else {
            panic!("expected Corbanu API operation");
        };
        assert_eq!(
            (capability, gateway_origin, decoded_operation),
            (
                "secret-capability".to_string(),
                "https://api.corbanu.example".to_string(),
                operation,
            )
        );
    }
}
