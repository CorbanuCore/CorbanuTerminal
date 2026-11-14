use super::DefaultKeyringStore;
use super::FIXTURE;
use codex_keyring_store::KeyringStore;
use pretty_assertions::assert_eq;

#[test]
fn uninstalled_fixture_keeps_native_store_denied() {
    // Fail before any credential operation if not in the ordinary isolated lane.
    // keyring-store honors the denial only in debug builds; a release-mode run
    // would reach the real OS keychain, so refuse it outright.
    if !cfg!(debug_assertions) {
        panic!("run in the isolated debug test lane (`just test`)");
    }
    assert_eq!(
        std::env::var("CORBANU_TEST_NO_NATIVE_KEYRING").as_deref(),
        Ok("1")
    );
    assert!(
        FIXTURE.get().is_none(),
        "fixture installs belong in isolated children"
    );
    for result in [
        DefaultKeyringStore
            .load("fixture-only", "canary")
            .map(|_| ()),
        DefaultKeyringStore.save("fixture-only", "canary", "synthetic-value"),
        DefaultKeyringStore
            .delete("fixture-only", "canary")
            .map(|_| ()),
    ] {
        assert_eq!(
            result
                .expect_err("native store must remain denied")
                .message(),
            "Native keyring is unavailable in an isolated test fixture",
        );
    }
}
