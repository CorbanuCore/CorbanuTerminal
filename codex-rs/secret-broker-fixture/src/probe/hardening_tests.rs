use super::*;
use pretty_assertions::assert_eq;

#[test]
fn pf_27_s01_probe_identity_requires_nonroot_and_all_saved_ids() {
    assert!(valid_ids([1000; 3], [1000; 3]));
    for (uid, gid) in [
        ([0; 3], [1000; 3]),
        ([1000, 1000, 0], [1000; 3]),
        ([1000; 3], [0; 3]),
        ([1000; 3], [1000, 1000, 0]),
    ] {
        assert!(!valid_ids(uid, gid));
    }
}

#[test]
fn pf_27_s01_probe_missing_malformed_or_duplicate_ambient_state_denies() {
    assert!(ambient_is_empty(&format!("{}\nCapAmb: 0000000000000000", "x".repeat(8192))).is_err());
    assert_eq!(
        ambient_is_empty("CapAmb:\t0000000000000000\n").ok(),
        Some(true)
    );
    assert_eq!(
        ambient_is_empty("CapAmb:\t0000000000000001\n").ok(),
        Some(false)
    );
    for value in [
        "",
        "CapAmb: xyz",
        "CapAmb: 0",
        "CapAmb: 00000000000000000",
        "CapAmb: 0000000000000000\nCapAmb: 0000000000000000",
    ] {
        assert!(ambient_is_empty(value).is_err());
    }
}

#[test]
fn pf_27_s01_probe_each_failed_kernel_observation_prevents_success() {
    for index in 0..5 {
        let mut checks = [true; 5];
        checks[index] = false;
        let report = Report {
            no_new_privileges: checks[0],
            nondumpable: checks[1],
            keepcaps_disabled: checks[2],
            capabilities_empty: checks[3],
            ambient_empty: checks[4],
            groups: 0,
            descriptors_allowed: true,
        };
        assert!(!report.verified());
    }
}
