use codex_core::config::Config;
use tempfile::TempDir;

/// Default test config rooted at `codex_home`.
///
/// Main's `core_test_support` has no loader without an explicit session folder
/// (#1120: a config built for the test process's own folder would let sandboxed
/// commands write into the source tree). These app-server tests only need the
/// home's config, so the temporary home also serves as the session folder.
pub async fn load_default_config_for_test(codex_home: &TempDir) -> Config {
    core_test_support::load_default_config_for_test_in(
        codex_home,
        Default::default(),
        codex_home.path(),
    )
    .await
}
