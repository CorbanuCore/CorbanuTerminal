//! Resource lifetime for the session-heavy #826, #874 and #891 fixtures.

use core_test_support::test_codex::TestCodex;

/// Finish background session work and close its SQLite workers before the next
/// scenario. Dropping the fixture alone leaves the submission loop alive until
/// the test's Tokio runtime exits; these tests run many scenarios per runtime.
pub(super) async fn shutdown(test: &TestCodex) -> anyhow::Result<()> {
    test.codex.shutdown_and_wait().await?;
    if let Some(state_db) = test.codex.state_db() {
        state_db.close().await;
    }
    Ok(())
}
