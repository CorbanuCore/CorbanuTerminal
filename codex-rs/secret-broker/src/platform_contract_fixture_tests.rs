use super::platform_contract::Capability;
use super::platform_contract::CapabilityResult;
use super::platform_contract::CapabilityStatus;
use super::platform_contract::Observation;
use super::platform_contract::PlatformReport;
use super::platform_contract::REQUIRED_CAPABILITIES;
use super::platform_contract::ResultRejection;
use super::platform_contract::validate_protected_mode_report;
use chrono::DateTime;
use pretty_assertions::assert_eq;
use serde_json::Value;
use std::collections::BTreeSet;

const SCHEMA: &str =
    include_str!("../../../qa/security-levels/platform/capability-result-v1.schema.json");
const FIXTURES: &[(&str, &str)] = &[
    (
        "linux",
        include_str!("../../../qa/security-levels/sprints/PF-27-S03/results/linux.json"),
    ),
    (
        "macos",
        include_str!("../../../qa/security-levels/sprints/PF-27-S03/results/macos.json"),
    ),
    (
        "windows",
        include_str!("../../../qa/security-levels/sprints/PF-27-S03/results/windows.json"),
    ),
];

#[test]
fn schema_enums_match_the_rust_contract() {
    let schema: Value = serde_json::from_str(SCHEMA).expect("schema must be JSON");
    let items = &schema["properties"]["capabilities"]["items"]["properties"];

    assert_eq!(
        enum_values(&items["capability"]),
        REQUIRED_CAPABILITIES
            .iter()
            .map(|capability| capability.as_str())
            .collect()
    );
    assert_eq!(
        enum_values(&items["status"]),
        BTreeSet::from(["supported", "unsupported", "untested"])
    );
    assert_eq!(
        enum_values(&items["observation"]),
        BTreeSet::from([
            "not_applicable",
            "not_tested",
            "observed_allowed",
            "observed_denied",
            "observed_verified",
            "probe_error",
        ])
    );
}

#[test]
fn frozen_result_fixtures_are_consumed_by_the_rust_gate() {
    for (target, fixture) in FIXTURES {
        let wire: Value = serde_json::from_str(fixture).expect("fixture must be JSON");
        let wire_capabilities = wire["capabilities"]
            .as_array()
            .expect("capabilities must be an array");
        let capabilities = wire_capabilities
            .iter()
            .map(parse_capability_result)
            .collect::<Vec<_>>();
        let measured_at = timestamp(&wire["measured_at"]);
        let expires_at = timestamp(&wire["expires_at"]);
        let probe_sha256 = string(&wire["probe_sha256"]);
        let target_id = string(&wire["target_id"]);
        let report = PlatformReport {
            contract_version: string(&wire["contract_version"]),
            fixture_protocol: string(&wire["fixture_protocol"]),
            probe_sha256,
            target_id,
            measured_at_unix_seconds: measured_at,
            expires_at_unix_seconds: expires_at,
            capabilities: &capabilities,
            protected_mode_eligible: wire["protected_mode_eligible"]
                .as_bool()
                .expect("eligibility must be a bool"),
        };

        assert_eq!(wire_capabilities.len(), REQUIRED_CAPABILITIES.len());
        assert_eq!(
            validate_protected_mode_report(&report, target_id, probe_sha256, measured_at,),
            Err(ResultRejection::UnsupportedCapability(
                Capability::ProcessIdentity
            )),
            "{target} must remain ineligible",
        );
    }
}

fn enum_values(value: &Value) -> BTreeSet<&str> {
    value["enum"]
        .as_array()
        .expect("enum must be an array")
        .iter()
        .map(string)
        .collect()
}

fn parse_capability_result(value: &Value) -> CapabilityResult<'_> {
    CapabilityResult {
        capability: match string(&value["capability"]) {
            "process_identity" => Capability::ProcessIdentity,
            "filesystem_boundary" => Capability::FilesystemBoundary,
            "config_boundary" => Capability::ConfigBoundary,
            "inherited_handles" => Capability::InheritedHandles,
            "ipc_peer_identity" => Capability::IpcPeerIdentity,
            "network_boundary" => Capability::NetworkBoundary,
            "process_memory_debug" => Capability::ProcessMemoryDebug,
            "signing_entitlements" => Capability::SigningEntitlements,
            "elevation_boundary" => Capability::ElevationBoundary,
            "protected_store" => Capability::ProtectedStore,
            unknown => panic!("unknown capability: {unknown}"),
        },
        status: match string(&value["status"]) {
            "supported" => CapabilityStatus::Supported,
            "unsupported" => CapabilityStatus::Unsupported,
            "untested" => CapabilityStatus::Untested,
            unknown => panic!("unknown status: {unknown}"),
        },
        observation: match string(&value["observation"]) {
            "observed_denied" => Observation::Denied,
            "observed_allowed" => Observation::Allowed,
            "observed_verified" => Observation::Verified,
            "not_tested" => Observation::Unavailable,
            "not_applicable" => Observation::NotApplicable,
            "probe_error" => Observation::Error,
            unknown => panic!("unknown observation: {unknown}"),
        },
        mechanism: string(&value["mechanism"]),
        detail_code: string(&value["detail_code"]),
    }
}

fn string(value: &Value) -> &str {
    value.as_str().expect("value must be a string")
}

fn timestamp(value: &Value) -> u64 {
    DateTime::parse_from_rfc3339(string(value))
        .expect("timestamp must be RFC 3339")
        .timestamp()
        .try_into()
        .expect("timestamp must be non-negative")
}
