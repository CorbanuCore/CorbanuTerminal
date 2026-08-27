use pretty_assertions::assert_eq;
use serde_json::json;

use super::*;
use crate::models::ResponseItem;

fn epoch() -> AuthorityEpoch {
    AuthorityEpoch::new(
        [1; 16], /*policy_revision*/ 0, /*revocation_generation*/ 0,
    )
    .unwrap()
}

#[test]
fn security_requests_round_trip_without_human_authentication_fields() {
    for action in [
        SecurityControlAction::SetLevel {
            level: SecurityLevel::Aggressive,
        },
        SecurityControlAction::Revoke {
            target: RevocationTarget::KillSwitch { active: true },
            reason: RevocationReason::KillSwitch,
        },
    ] {
        let request = SecurityControlRequest::new(epoch(), action).unwrap();
        let wire = json!(request);
        assert_eq!(
            serde_json::from_value::<SecurityControlRequest>(wire.clone()).unwrap(),
            request
        );
        for field in ["human", "issuer", "authenticated", "confirmation"] {
            let mut forged = wire.clone();
            forged[field] = json!(true);
            assert!(serde_json::from_value::<SecurityControlRequest>(forged).is_err());
        }
    }
}

#[test]
fn security_requests_reject_unknown_versions_and_invalid_nested_contracts() {
    let request = SecurityControlRequest::new(
        epoch(),
        SecurityControlAction::SetLevel {
            level: SecurityLevel::Moderate,
        },
    )
    .unwrap();
    for (path, value) in [
        ("/schema_version", json!(2)),
        ("/expected_epoch/runtime_nonce", json!(vec![0; 16])),
        ("/action/level", json!("trusted")),
    ] {
        let mut wire = json!(request);
        *wire.pointer_mut(path).unwrap() = value;
        assert!(serde_json::from_value::<SecurityControlRequest>(wire).is_err());
    }
    let mut wire = json!(request);
    wire.as_object_mut().unwrap().remove("expected_epoch");
    assert!(serde_json::from_value::<SecurityControlRequest>(wire).is_err());
}

#[test]
fn grant_intent_reuses_existing_scope_validation() {
    let action: SecurityControlAction = serde_json::from_value(json!({
        "kind": "create_grant",
        "actor_chain": [{"kind": "human", "id": "human:1"}, {"kind": "agent", "id": "agent:1"}],
        "scope": {"resource": {"kind": "tool", "id": "tool:1"}, "actions": ["execute"], "context": {"session_id": "session:1", "task_id": "task:1", "purpose": "test", "operation": "tool.execute"}},
        "expires_at_unix_seconds": 200
    })).unwrap();
    let request = SecurityControlRequest::new(epoch(), action).unwrap();
    assert_eq!(
        serde_json::from_value::<SecurityControlRequest>(json!(request)).unwrap(),
        request
    );
    for (path, value) in [
        ("/action/scope/actions", json!([])),
        ("/action/expires_at_unix_seconds", json!(-1)),
    ] {
        let mut wire = json!(request);
        *wire.pointer_mut(path).unwrap() = value;
        assert!(serde_json::from_value::<SecurityControlRequest>(wire).is_err());
    }
}

#[test]
fn security_requests_are_not_provider_calls_and_leave_native_wire_unchanged() {
    let request = SecurityControlRequest::new(
        epoch(),
        SecurityControlAction::SetLevel {
            level: SecurityLevel::Permissive,
        },
    )
    .unwrap();
    let mut wire = json!(request);
    wire["type"] = json!("security_control");
    assert_eq!(
        serde_json::from_value::<ResponseItem>(wire).unwrap(),
        ResponseItem::Other
    );
    let provider_call =
        json!({"type": "function_call", "name": "shell", "arguments": "{}", "call_id": "call_1"});
    let native: ResponseItem = serde_json::from_value(provider_call.clone()).unwrap();
    // A separate security proposal cannot add fields to the native provider wire.
    let _side_channel = json!(request);
    assert_eq!(serde_json::to_value(native).unwrap(), provider_call);
}
