//! Assertion-boundary normalization for headers containing the build version.

/// Replace only the exact package version in a rendered application header.
/// Other versions (including empty or malformed ones) remain visible to snapshots.
pub(crate) fn normalize_snapshot_version(text: &str) -> String {
    normalize_header_version(text, env!("CARGO_PKG_VERSION"))
}

fn normalize_header_version(text: &str, version: &str) -> String {
    const PLACEHOLDER: &str = "<VERSION>";
    let header = format!(">_ Corbanu Terminal (v{version})");
    text.split_inclusive('\n')
        .map(|line| {
            let Some((prefix, suffix)) = line.split_once(&header) else {
                return line.to_string();
            };
            let padding = suffix.len() - suffix.trim_start_matches(' ').len();
            // Keep the frame column stable even when a release adds/removes a digit.
            let normalized_padding = (padding + version.len()).saturating_sub(PLACEHOLDER.len());
            format!(
                "{prefix}>_ Corbanu Terminal (v{PLACEHOLDER}){}{}",
                " ".repeat(normalized_padding),
                &suffix[padding..]
            )
        })
        .collect()
}

#[path = "snapshot_helpers_tests.rs"]
mod tests;
