//! Versioned platform-containment contract frozen by PF-27-S03.
//!
//! G1 integration registers this contract without adding a runtime consumer or
//! enabling a protected-mode route.

use std::fmt;

pub const CONTRACT_VERSION: &str = "corbanu.platform-containment/v1";
pub const FIXTURE_PROTOCOL_VERSION: &str = "corbanu.platform-probe/v1";
pub const MAX_RESULT_AGE_SECONDS: u64 = 86_400;
pub const MAX_FUTURE_SKEW_SECONDS: u64 = 300;

pub const REQUIRED_CAPABILITIES: &[Capability] = &[
    Capability::ProcessIdentity,
    Capability::FilesystemBoundary,
    Capability::ConfigBoundary,
    Capability::InheritedHandles,
    Capability::IpcPeerIdentity,
    Capability::NetworkBoundary,
    Capability::ProcessMemoryDebug,
    Capability::SigningEntitlements,
    Capability::ElevationBoundary,
    Capability::ProtectedStore,
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Capability {
    ProcessIdentity,
    FilesystemBoundary,
    ConfigBoundary,
    InheritedHandles,
    IpcPeerIdentity,
    NetworkBoundary,
    ProcessMemoryDebug,
    SigningEntitlements,
    ElevationBoundary,
    ProtectedStore,
}

impl Capability {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ProcessIdentity => "process_identity",
            Self::FilesystemBoundary => "filesystem_boundary",
            Self::ConfigBoundary => "config_boundary",
            Self::InheritedHandles => "inherited_handles",
            Self::IpcPeerIdentity => "ipc_peer_identity",
            Self::NetworkBoundary => "network_boundary",
            Self::ProcessMemoryDebug => "process_memory_debug",
            Self::SigningEntitlements => "signing_entitlements",
            Self::ElevationBoundary => "elevation_boundary",
            Self::ProtectedStore => "protected_store",
        }
    }

    const fn index(self) -> usize {
        match self {
            Self::ProcessIdentity => 0,
            Self::FilesystemBoundary => 1,
            Self::ConfigBoundary => 2,
            Self::InheritedHandles => 3,
            Self::IpcPeerIdentity => 4,
            Self::NetworkBoundary => 5,
            Self::ProcessMemoryDebug => 6,
            Self::SigningEntitlements => 7,
            Self::ElevationBoundary => 8,
            Self::ProtectedStore => 9,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CapabilityStatus {
    Supported,
    Unsupported,
    Untested,
}

impl CapabilityStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Supported => "supported",
            Self::Unsupported => "unsupported",
            Self::Untested => "untested",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Observation {
    Denied,
    Allowed,
    Verified,
    Unavailable,
    NotApplicable,
    Error,
}

impl Observation {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Denied => "observed_denied",
            Self::Allowed => "observed_allowed",
            Self::Verified => "observed_verified",
            Self::Unavailable => "not_tested",
            Self::NotApplicable => "not_applicable",
            Self::Error => "probe_error",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CapabilityResult<'a> {
    pub capability: Capability,
    pub status: CapabilityStatus,
    pub observation: Observation,
    pub mechanism: &'a str,
    pub detail_code: &'a str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlatformReport<'a> {
    pub contract_version: &'a str,
    pub fixture_protocol: &'a str,
    pub probe_sha256: &'a str,
    pub target_id: &'a str,
    pub measured_at_unix_seconds: u64,
    pub expires_at_unix_seconds: u64,
    pub capabilities: &'a [CapabilityResult<'a>],
    pub protected_mode_eligible: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ResultRejection {
    UnknownContract,
    UnknownFixtureProtocol,
    WrongProbeIdentity,
    WrongTargetIdentity,
    Stale,
    FutureDated,
    InvalidExpiry,
    DuplicateCapability,
    MissingCapability(Capability),
    UnsupportedCapability(Capability),
    UntestedCapability(Capability),
    EligibilityClaimMismatch,
    StatusObservationMismatch(Capability),
}

/// Proof that a complete, current platform report passed every activation gate.
///
/// Callers cannot construct this witness. Protected-mode consumers must require
/// it rather than trusting the report's self-asserted eligibility bit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProtectedModeAuthorization(());

impl fmt::Display for ResultRejection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

/// Validates the complete activation envelope. A returned authorization witness
/// is the only outcome that permits protected mode; every malformed or incomplete
/// input fails closed.
pub fn validate_protected_mode_report(
    report: &PlatformReport<'_>,
    expected_target_id: &str,
    expected_probe_sha256: &str,
    now_unix_seconds: u64,
) -> Result<ProtectedModeAuthorization, ResultRejection> {
    if report.contract_version != CONTRACT_VERSION {
        return Err(ResultRejection::UnknownContract);
    }
    if report.fixture_protocol != FIXTURE_PROTOCOL_VERSION {
        return Err(ResultRejection::UnknownFixtureProtocol);
    }
    if !is_lower_hex_64(expected_probe_sha256)
        || !is_lower_hex_64(report.probe_sha256)
        || report.probe_sha256 != expected_probe_sha256
    {
        return Err(ResultRejection::WrongProbeIdentity);
    }
    if !is_lower_hex_64(expected_target_id)
        || !is_lower_hex_64(report.target_id)
        || report.target_id != expected_target_id
    {
        return Err(ResultRejection::WrongTargetIdentity);
    }
    if report.measured_at_unix_seconds > now_unix_seconds.saturating_add(MAX_FUTURE_SKEW_SECONDS) {
        return Err(ResultRejection::FutureDated);
    }
    if report.expires_at_unix_seconds <= now_unix_seconds {
        return Err(ResultRejection::Stale);
    }
    let Some(lifetime) = report
        .expires_at_unix_seconds
        .checked_sub(report.measured_at_unix_seconds)
    else {
        return Err(ResultRejection::InvalidExpiry);
    };
    if lifetime == 0 || lifetime > MAX_RESULT_AGE_SECONDS {
        return Err(ResultRejection::InvalidExpiry);
    }

    let mut seen = [false; REQUIRED_CAPABILITIES.len()];
    let mut statuses = [CapabilityStatus::Untested; REQUIRED_CAPABILITIES.len()];
    for result in report.capabilities {
        let index = result.capability.index();
        if seen[index] {
            return Err(ResultRejection::DuplicateCapability);
        }
        seen[index] = true;
        if !matches!(
            (result.status, result.observation),
            (
                CapabilityStatus::Supported,
                Observation::Denied | Observation::Verified
            ) | (
                CapabilityStatus::Unsupported,
                Observation::Allowed | Observation::NotApplicable
            ) | (
                CapabilityStatus::Untested,
                Observation::Unavailable | Observation::Error
            )
        ) {
            return Err(ResultRejection::StatusObservationMismatch(
                result.capability,
            ));
        }
        statuses[index] = result.status;
    }
    for capability in REQUIRED_CAPABILITIES {
        let index = capability.index();
        if !seen[index] {
            return Err(ResultRejection::MissingCapability(*capability));
        }
        match statuses[index] {
            CapabilityStatus::Supported => {}
            CapabilityStatus::Unsupported => {
                return Err(ResultRejection::UnsupportedCapability(*capability));
            }
            CapabilityStatus::Untested => {
                return Err(ResultRejection::UntestedCapability(*capability));
            }
        }
    }
    if !report.protected_mode_eligible {
        return Err(ResultRejection::EligibilityClaimMismatch);
    }
    Ok(ProtectedModeAuthorization(()))
}

fn is_lower_hex_64(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    const TARGET_ID: &str = concat!(
        "1111111111111111",
        "1111111111111111",
        "1111111111111111",
        "1111111111111111"
    );
    const PROBE_SHA256: &str = concat!(
        "aaaaaaaaaaaaaaaa",
        "aaaaaaaaaaaaaaaa",
        "aaaaaaaaaaaaaaaa",
        "aaaaaaaaaaaaaaaa"
    );
    const OTHER_TARGET_ID: &str = concat!(
        "0000000000000000",
        "0000000000000000",
        "0000000000000000",
        "0000000000000000"
    );
    const OTHER_PROBE_SHA256: &str = concat!(
        "bbbbbbbbbbbbbbbb",
        "bbbbbbbbbbbbbbbb",
        "bbbbbbbbbbbbbbbb",
        "bbbbbbbbbbbbbbbb"
    );

    const SUPPORTED: CapabilityResult<'static> = CapabilityResult {
        capability: Capability::ProcessIdentity,
        status: CapabilityStatus::Supported,
        observation: Observation::Denied,
        mechanism: "test",
        detail_code: "denied",
    };

    fn supported_results() -> [CapabilityResult<'static>; 10] {
        std::array::from_fn(|index| CapabilityResult {
            capability: REQUIRED_CAPABILITIES[index],
            ..SUPPORTED
        })
    }

    fn report<'a>(capabilities: &'a [CapabilityResult<'a>]) -> PlatformReport<'a> {
        PlatformReport {
            contract_version: CONTRACT_VERSION,
            fixture_protocol: FIXTURE_PROTOCOL_VERSION,
            probe_sha256: PROBE_SHA256,
            target_id: TARGET_ID,
            measured_at_unix_seconds: 1_000,
            expires_at_unix_seconds: 2_000,
            capabilities,
            protected_mode_eligible: true,
        }
    }

    #[test]
    fn accepts_complete_current_supported_report() {
        let results = supported_results();
        assert_eq!(
            validate_protected_mode_report(&report(&results), TARGET_ID, PROBE_SHA256, 1_500),
            Ok(ProtectedModeAuthorization(()))
        );
    }

    #[test]
    fn rejects_unknown_contract_and_fixture_protocol() {
        let results = supported_results();
        let mut candidate = report(&results);
        candidate.contract_version = "corbanu.platform-containment/unknown";
        assert_eq!(
            validate_protected_mode_report(&candidate, TARGET_ID, PROBE_SHA256, 1_500),
            Err(ResultRejection::UnknownContract)
        );

        candidate.contract_version = CONTRACT_VERSION;
        candidate.fixture_protocol = "corbanu.platform-probe/unknown";
        assert_eq!(
            validate_protected_mode_report(&candidate, TARGET_ID, PROBE_SHA256, 1_500),
            Err(ResultRejection::UnknownFixtureProtocol)
        );
    }

    #[test]
    fn rejects_future_dated_report() {
        let results = supported_results();
        let mut candidate = report(&results);
        candidate.measured_at_unix_seconds = 2_000;
        candidate.expires_at_unix_seconds = 2_500;
        assert_eq!(
            validate_protected_mode_report(&candidate, TARGET_ID, PROBE_SHA256, 1_500),
            Err(ResultRejection::FutureDated)
        );
    }

    #[test]
    fn rejects_invalid_expiry() {
        let results = supported_results();
        let mut candidate = report(&results);
        candidate.expires_at_unix_seconds = candidate.measured_at_unix_seconds;
        assert_eq!(
            validate_protected_mode_report(&candidate, TARGET_ID, PROBE_SHA256, 900),
            Err(ResultRejection::InvalidExpiry)
        );
    }

    #[test]
    fn rejects_untested_capability() {
        let mut results = supported_results();
        results[0].status = CapabilityStatus::Untested;
        results[0].observation = Observation::Unavailable;
        assert_eq!(
            validate_protected_mode_report(&report(&results), TARGET_ID, PROBE_SHA256, 1_500),
            Err(ResultRejection::UntestedCapability(
                Capability::ProcessIdentity
            ))
        );
    }

    #[test]
    fn rejects_malformed_digest_identity() {
        let results = supported_results();
        for malformed in [
            "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        ] {
            assert_eq!(
                validate_protected_mode_report(&report(&results), TARGET_ID, malformed, 1_500),
                Err(ResultRejection::WrongProbeIdentity)
            );
        }
    }

    #[test]
    fn rejects_wrong_target() {
        let results = supported_results();
        assert_eq!(
            validate_protected_mode_report(&report(&results), OTHER_TARGET_ID, PROBE_SHA256, 1_500,),
            Err(ResultRejection::WrongTargetIdentity)
        );
    }

    #[test]
    fn rejects_stale_report() {
        let results = supported_results();
        assert_eq!(
            validate_protected_mode_report(&report(&results), TARGET_ID, PROBE_SHA256, 2_000),
            Err(ResultRejection::Stale)
        );
    }

    #[test]
    fn rejects_duplicate_capability() {
        let mut results = supported_results();
        results[9].capability = Capability::ProcessIdentity;
        assert_eq!(
            validate_protected_mode_report(&report(&results), TARGET_ID, PROBE_SHA256, 1_500),
            Err(ResultRejection::DuplicateCapability)
        );
    }

    #[test]
    fn rejects_missing_capability() {
        let results = supported_results();
        assert_eq!(
            validate_protected_mode_report(&report(&results[..9]), TARGET_ID, PROBE_SHA256, 1_500,),
            Err(ResultRejection::MissingCapability(
                Capability::ProtectedStore
            ))
        );
    }

    #[test]
    fn rejects_claimed_eligibility_with_unsupported_capability() {
        let mut results = supported_results();
        results[0].status = CapabilityStatus::Unsupported;
        results[0].observation = Observation::Allowed;
        assert_eq!(
            validate_protected_mode_report(&report(&results), TARGET_ID, PROBE_SHA256, 1_500),
            Err(ResultRejection::UnsupportedCapability(
                Capability::ProcessIdentity
            ))
        );
    }

    #[test]
    fn rejects_wrong_probe_identity() {
        let results = supported_results();
        assert_eq!(
            validate_protected_mode_report(&report(&results), TARGET_ID, OTHER_PROBE_SHA256, 1_500,),
            Err(ResultRejection::WrongProbeIdentity)
        );
    }

    #[test]
    fn rejects_false_ineligibility_claim() {
        let results = supported_results();
        let mut candidate = report(&results);
        candidate.protected_mode_eligible = false;
        assert_eq!(
            validate_protected_mode_report(&candidate, TARGET_ID, PROBE_SHA256, 1_500),
            Err(ResultRejection::EligibilityClaimMismatch)
        );
    }

    #[test]
    fn rejects_inconsistent_status_and_observation() {
        let mut results = supported_results();
        results[0].observation = Observation::Error;
        assert_eq!(
            validate_protected_mode_report(&report(&results), TARGET_ID, PROBE_SHA256, 1_500),
            Err(ResultRejection::StatusObservationMismatch(
                Capability::ProcessIdentity
            ))
        );
    }
}
