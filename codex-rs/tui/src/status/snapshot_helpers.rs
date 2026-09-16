//! Assertion-boundary normalization for rendered build versions.

/// Replace only the exact package version in application headers and update notices.
/// Other versions (including empty or malformed ones) remain visible to snapshots.
pub(crate) fn normalize_snapshot_version(text: &str) -> String {
    normalize_header_version(text, env!("CARGO_PKG_VERSION"))
}

fn normalize_header_version(text: &str, version: &str) -> String {
    // Shorter than every Cargo package version (at least "0.0.0"), so even a
    // flush header can retain its frame column without truncating the marker.
    const PLACEHOLDER: &str = "<V>";
    let extra_padding = version.len() - PLACEHOLDER.len();
    let header = format!(">_ Corbanu Terminal (v{version})");
    let update = format!("Update available! {version} -> ");
    text.split_inclusive('\n')
        .map(|line| {
            if let Some((prefix, suffix)) = line.split_once(&header) {
                return format!(
                    "{prefix}>_ Corbanu Terminal (v{PLACEHOLDER}){}{suffix}",
                    " ".repeat(extra_padding),
                );
            }
            if let Some((prefix, suffix)) = line.split_once(&update) {
                // Keep the target version and arrow intact; put reclaimed
                // columns at the end of the notice, before any frame/quotes.
                let content = suffix.trim_end_matches(['│', '"', '\r', '\n']);
                let ending = &suffix[content.len()..];
                return format!(
                    "{prefix}Update available! {PLACEHOLDER} -> {content}{}{ending}",
                    " ".repeat(extra_padding),
                );
            }
            line.to_string()
        })
        .collect()
}

#[path = "snapshot_helpers_tests.rs"]
mod tests;
