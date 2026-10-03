/// The current Codex CLI version as embedded at compile time.
#[cfg(not(test))]
pub const CODEX_CLI_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Unit tests render as a source build, matching Bazel (which compiles every
/// crate as 0.0.0) so version snapshots do not churn with release bumps.
#[cfg(test)]
pub const CODEX_CLI_VERSION: &str = "0.0.0";
