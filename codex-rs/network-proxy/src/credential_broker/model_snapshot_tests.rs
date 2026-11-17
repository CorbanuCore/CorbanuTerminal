use super::*;
use pretty_assertions::assert_eq;

#[cfg(any(unix, windows))]
#[test]
fn d690_model_snapshot_fingerprints_match_copies_and_short_words() {
    let key = [42; 32];
    for (value, text, expected) in [
        (
            "d690-synthetic-model-value",
            "prefixd690-synthetic-model-valuesuffix",
            true,
        ),
        (
            "d690-synthetic-model-value",
            "ordinary configuration",
            false,
        ),
        ("abc", "prefixabc", false),
        ("abc", "prefix:abc:suffix", true),
        ("1234567", "x1234567y", false),
        ("1234567", "'1234567'", true),
        ("12345678", "x12345678y", true),
    ] {
        let records = vec![(value.len(), fingerprint(&key, value.as_bytes()))];
        assert_eq!(scan_fingerprints(text, &key, &records), Ok(expected));
    }
    assert_eq!(
        scan_fingerprints("safe", &key, &[(2, fingerprint(&key, b"xy"))]),
        Err(ModelSnapshotScanError)
    );
}

#[cfg(any(unix, windows))]
#[test]
fn d690_model_snapshot_budget_refuses_large_isolated_scan() {
    let key = [42; 32];
    let records = vec![(16, fingerprint(&key, b"synthetic-secret"))];
    let text = "x".repeat((1 << 21) + 16);
    assert_eq!(
        scan_fingerprints(&text, &key, &records),
        Err(ModelSnapshotScanError)
    );
}

#[test]
fn d690_model_snapshot_without_scoped_routes_is_clean() {
    let holder = RwLock::new(CredentialBrokerState::default());
    assert_eq!(holder.occurs_in("ordinary configuration"), Ok(false));
}
