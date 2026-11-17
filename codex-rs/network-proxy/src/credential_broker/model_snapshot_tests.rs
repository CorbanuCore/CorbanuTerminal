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
fn d2502_eight_model_keys_allow_three_mib_and_find_trailing_copy() {
    let key = [42; 32];
    let values: Vec<_> = (0..8)
        .map(|n| format!("synthetic-2502-{}", "k".repeat(16 + n)))
        .collect();
    let records: Vec<_> = values
        .iter()
        .map(|value| (value.len(), fingerprint(&key, value.as_bytes())))
        .collect();
    let text = "x".repeat(/*n*/ 3 * 1024 * 1024);
    assert_eq!(scan_fingerprints(&text, &key, &records), Ok(false));
    assert_eq!(
        scan_fingerprints(&format!("{text}{}", values[7]), &key, &records),
        Ok(true)
    );
}

#[cfg(any(unix, windows))]
#[test]
fn d690_model_snapshot_budget_refuses_large_isolated_scan() {
    let key = [42; 32];
    let records = vec![(16, fingerprint(&key, b"synthetic-secret"))];
    let text = "x".repeat(codex_secret_broker::output_gate::MAX_SCAN_BYTES + 1);
    assert_eq!(
        scan_fingerprints(&text, &key, &records),
        Err(ModelSnapshotScanError)
    );
}

#[cfg(any(unix, windows))]
#[test]
fn d690_model_snapshot_without_scoped_routes_is_clean() {
    assert_eq!(
        scan_fingerprints("ordinary configuration", &[42; 32], &[]),
        Ok(false)
    );
}
