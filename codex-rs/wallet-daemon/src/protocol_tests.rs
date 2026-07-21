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
