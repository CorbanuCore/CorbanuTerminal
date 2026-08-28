use pretty_assertions::assert_eq;
use serde_json::json;

use super::SecurityControlHealth;
use super::SecurityControlHealthSnapshot;
use super::SecurityDegradationReason;
use super::SecurityInspectorError;
use super::SecurityInspectorSnapshot;
use super::SecurityLevel;

fn snapshot_json() -> serde_json::Value {
    json!({
        "schema_version": 1,
        "requested_level": "moderate",
        "effective_level": "aggressive",
        "controls": {
            "browser_isolation": {"state": "enforcing"},
            "content_firewall": {"state": "unavailable"},
            "confidentiality": {"state": "inactive"},
            "protected_actions": {"state": "degraded", "reason": "health_check_failed"}
        }
    })
}

#[test]
fn inspector_wire_round_trip_preserves_independent_health_and_policy_facts() {
    let snapshot = SecurityInspectorSnapshot::new(
        SecurityLevel::Moderate,
        SecurityLevel::Aggressive,
        SecurityControlHealthSnapshot {
            browser_isolation: SecurityControlHealth::Enforcing {},
            content_firewall: SecurityControlHealth::Unavailable {},
            confidentiality: SecurityControlHealth::Inactive {},
            protected_actions: SecurityControlHealth::Degraded {
                reason: SecurityDegradationReason::HealthCheckFailed,
            },
        },
    )
    .expect("valid independent facts");
    assert_eq!(serde_json::to_value(&snapshot).unwrap(), snapshot_json());
    assert_eq!(
        serde_json::from_value::<SecurityInspectorSnapshot>(snapshot_json()).unwrap(),
        snapshot
    );
    assert_eq!(
        (snapshot.requested_level(), snapshot.effective_level()),
        (SecurityLevel::Moderate, SecurityLevel::Aggressive)
    );
}

#[test]
fn requested_and_effective_level_matrix_never_widens() {
    let levels = [
        SecurityLevel::Permissive,
        SecurityLevel::Moderate,
        SecurityLevel::Aggressive,
    ];
    for requested in levels {
        for effective in levels {
            let result = SecurityInspectorSnapshot::new(
                requested,
                effective,
                SecurityControlHealthSnapshot::default(),
            );
            let mut wire = snapshot_json();
            wire["requested_level"] = json!(requested);
            wire["effective_level"] = json!(effective);
            wire["controls"] = json!(SecurityControlHealthSnapshot::default());
            let decoded = serde_json::from_value::<SecurityInspectorSnapshot>(wire);
            if effective < requested {
                assert_eq!(result, Err(SecurityInspectorError::WeakerEffectiveLevel));
                assert!(decoded.is_err());
            } else {
                let snapshot = result.unwrap();
                assert_eq!(decoded.unwrap(), snapshot);
                // Selecting a stronger floor must not fabricate backend availability.
                assert_eq!(
                    snapshot.controls(),
                    &SecurityControlHealthSnapshot::default()
                );
            }
        }
    }
}

#[test]
fn permissive_cannot_claim_any_added_control_is_enforcing() {
    for control in [
        "browser_isolation",
        "content_firewall",
        "confidentiality",
        "protected_actions",
    ] {
        let mut controls = json!(SecurityControlHealthSnapshot::default());
        controls[control] = json!({"state": "enforcing"});
        assert_eq!(
            SecurityInspectorSnapshot::new(
                SecurityLevel::Permissive,
                SecurityLevel::Permissive,
                serde_json::from_value(controls.clone()).unwrap(),
            ),
            Err(SecurityInspectorError::PermissiveEnforcementClaim)
        );
        let mut wire = snapshot_json();
        wire["requested_level"] = json!("permissive");
        wire["effective_level"] = json!("permissive");
        wire["controls"] = controls;
        assert!(serde_json::from_value::<SecurityInspectorSnapshot>(wire).is_err());
    }
}

#[test]
fn unknown_versions_levels_states_and_diagnostics_fail_closed() {
    for (pointer, value) in [
        ("/schema_version", json!(0)),
        ("/schema_version", json!(2)),
        ("/schema_version", json!("1")),
        ("/requested_level", json!("unknown")),
        ("/effective_level", json!("unknown")),
        ("/controls/browser_isolation/state", json!("ready")),
        (
            "/controls/protected_actions/reason",
            json!("backend-secret"),
        ),
    ] {
        let mut wire = snapshot_json();
        *wire.pointer_mut(pointer).unwrap() = value;
        assert!(serde_json::from_value::<SecurityInspectorSnapshot>(wire).is_err());
    }
}

#[test]
fn missing_facts_and_extra_authority_or_secret_fields_are_rejected() {
    let mut missing_version = snapshot_json();
    missing_version
        .as_object_mut()
        .unwrap()
        .remove("schema_version");
    let mut missing_control = snapshot_json();
    missing_control["controls"]
        .as_object_mut()
        .unwrap()
        .remove("content_firewall");
    let mut human_claim = snapshot_json();
    human_claim["human"] = json!(true);
    let mut error_payload = snapshot_json();
    error_payload["controls"]["protected_actions"]["message"] = json!("backend-secret");
    let mut extra_control = snapshot_json();
    extra_control["controls"]["new_control"] = json!({"state": "enforcing"});
    let mut missing_reason = snapshot_json();
    missing_reason["controls"]["protected_actions"]
        .as_object_mut()
        .unwrap()
        .remove("reason");
    for wire in [
        missing_version,
        missing_control,
        human_claim,
        error_payload,
        extra_control,
        missing_reason,
    ] {
        assert!(serde_json::from_value::<SecurityInspectorSnapshot>(wire).is_err());
    }
}

#[test]
fn degraded_controls_preserve_reason_without_converting_to_enforcement() {
    for reason in [
        SecurityDegradationReason::BackendUnavailable,
        SecurityDegradationReason::UnsupportedPlatform,
        SecurityDegradationReason::PolicyMismatch,
        SecurityDegradationReason::HealthCheckFailed,
        SecurityDegradationReason::ResourceLimit,
    ] {
        let degraded = SecurityControlHealth::Degraded { reason };
        let snapshot = SecurityInspectorSnapshot::new(
            SecurityLevel::Aggressive,
            SecurityLevel::Aggressive,
            SecurityControlHealthSnapshot {
                browser_isolation: degraded,
                content_firewall: degraded,
                confidentiality: degraded,
                protected_actions: degraded,
            },
        )
        .unwrap();
        assert_eq!(
            serde_json::from_value::<SecurityInspectorSnapshot>(json!(snapshot)).unwrap(),
            snapshot
        );
    }
}

#[test]
fn inspector_schema_exposes_versioned_facts_without_authority_fields() {
    let schema = serde_json::to_value(schemars::schema_for!(SecurityInspectorSnapshot)).unwrap();
    assert_eq!(schema["additionalProperties"], json!(false));
    assert_eq!(
        schema["properties"]
            .as_object()
            .unwrap()
            .keys()
            .collect::<Vec<_>>(),
        vec![
            "controls",
            "effective_level",
            "requested_level",
            "schema_version"
        ]
    );
    assert_eq!(
        schema["required"],
        json!([
            "controls",
            "effective_level",
            "requested_level",
            "schema_version"
        ])
    );
    assert_eq!(
        schema["properties"]["schema_version"],
        json!({"type": "integer", "format": "uint32", "minimum": 1.0, "maximum": 1.0})
    );
}

#[test]
fn every_health_variant_rejects_payload_extensions() {
    for state in ["unavailable", "inactive", "enforcing", "degraded"] {
        let mut wire = snapshot_json();
        wire["controls"]["browser_isolation"] = json!({"state": state, "payload": "secret"});
        if state == "degraded" {
            wire["controls"]["browser_isolation"]["reason"] = json!("resource_limit");
        }
        assert!(serde_json::from_value::<SecurityInspectorSnapshot>(wire).is_err());
    }
}
