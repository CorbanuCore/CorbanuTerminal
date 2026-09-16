use super::normalize_header_version;
use super::normalize_snapshot_version;
use pretty_assertions::assert_eq;

#[test]
fn status_header_version_normalization_preserves_frame_across_releases() {
    let expected = "│ >_ Corbanu Terminal (v<VERSION>)        │\n";
    for version in ["0.1.9", "0.1.42", "0.1.100", "1.0.0-rc.1"] {
        let header = format!(">_ Corbanu Terminal (v{version})");
        let rendered = format!("│ {header:40}│\n");
        assert_eq!(normalize_header_version(&rendered, version), expected);
    }
}

#[test]
fn status_header_version_normalization_only_masks_the_compiled_version() {
    let version = env!("CARGO_PKG_VERSION");
    let rendered = format!(
        "│ >_ Corbanu Terminal (v{version})            │\naccount: {version}\n│ >_ Corbanu Terminal (v{version})            │"
    );
    let normalized = normalize_snapshot_version(&rendered);
    assert_eq!(normalized.matches("(v<VERSION>)").count(), 2);
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
