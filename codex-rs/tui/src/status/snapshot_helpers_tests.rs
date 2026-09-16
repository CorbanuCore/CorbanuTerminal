use super::normalize_header_version;
use super::normalize_snapshot_version;
use pretty_assertions::assert_eq;

#[test]
fn status_header_version_normalization_preserves_frame_across_releases() {
    let expected = "│ >_ Corbanu Terminal (v<V>)              │\n";
    for version in ["0.1.9", "0.1.42", "0.1.100", "1.0.0-rc.1"] {
        let header = format!(">_ Corbanu Terminal (v{version})");
        let rendered = format!("│ {header:40}│\n");
        assert_eq!(normalize_header_version(&rendered, version), expected);
    }
}

#[test]
fn status_version_normalization_preserves_flush_frames() {
    for version in ["0.1.9", "0.1.42", "0.1.100", "1.0.0-rc.1"] {
        for padding in 0..=3 {
            for body in [
                format!(">_ Corbanu Terminal (v{version})"),
                format!("✨\u{200a}Update available! {version} -> 9.9.9"),
            ] {
                let rendered = format!("│ {body}{}│\n", " ".repeat(padding));
                let normalized = normalize_header_version(&rendered, version);
                assert_eq!(normalized.chars().count(), rendered.chars().count());
                assert!(normalized.contains("<V>"));
                assert!(!normalized.contains(version));
                assert_eq!(normalized.find('│'), rendered.find('│'));
                assert_eq!(normalized.rfind('│'), rendered.rfind('│'));
            }
        }
    }
}

#[test]
fn status_update_version_normalization_preserves_target_and_other_versions() {
    let version = env!("CARGO_PKG_VERSION");
    let rendered = format!(
        "│ ✨\u{200a}Update available! {version} -> {version}          │\naccount: {version}\n"
    );
    let expected = format!(
        "│ ✨\u{200a}Update available! <V> -> {version}{}│\naccount: {version}\n",
        " ".repeat(10 + version.len() - 3),
    );
    assert_eq!(normalize_snapshot_version(&rendered), expected);
    for wrong_version in ["", "garbled", "999.999.999"] {
        let rendered = format!("│ Update available! {wrong_version} -> 9.9.9 │");
        assert_eq!(normalize_snapshot_version(&rendered), rendered);
    }
    let missing = "│ Update available! -> 9.9.9 │";
    assert_eq!(normalize_snapshot_version(missing), missing);
}

#[test]
fn status_header_version_normalization_only_masks_the_compiled_version() {
    let version = env!("CARGO_PKG_VERSION");
    let rendered = format!(
        "│ >_ Corbanu Terminal (v{version})            │\naccount: {version}\n│ >_ Corbanu Terminal (v{version})            │"
    );
    let normalized = normalize_snapshot_version(&rendered);
    assert_eq!(normalized.matches("(v<V>)").count(), 2);
    assert!(normalized.contains(&format!("account: {version}")));
    assert_eq!(normalized.lines().count(), rendered.lines().count());
    assert_eq!(normalized.len(), rendered.len());
}

#[test]
fn status_header_version_normalization_preserves_invalid_or_missing_headers() {
    for rendered in [
        "│ >_ Corbanu Terminal (v)            │",
        "│ >_ Corbanu Terminal (vgarbled)     │",
        "│ >_ Corbanu Terminal (v999.999.999) │",
        "│ >_ Corbanu Terminal               │",
        "│ model: gpt-5                     │",
        "",
    ] {
        assert_eq!(normalize_snapshot_version(rendered), rendered);
    }
}
