use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use keyring::credential::Credential;
use keyring::credential::CredentialBuilderApi;

use crate::DefaultKeyringStore;
use crate::KeyringStore;

static CALLS: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug)]
struct EmptyFixture;

impl CredentialBuilderApi for EmptyFixture {
    fn build(&self, _: Option<&str>, _: &str, _: &str) -> keyring::Result<Box<Credential>> {
        CALLS.fetch_add(1, Ordering::SeqCst);
        Ok(Box::new(keyring::mock::MockCredential::default()))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[derive(Debug)]
struct ForbiddenGlobalBuilder;

impl CredentialBuilderApi for ForbiddenGlobalBuilder {
    fn build(&self, _: Option<&str>, _: &str, _: &str) -> keyring::Result<Box<Credential>> {
        panic!("fixture operation must never call the global keyring builder");
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[test]
fn explicit_fixture_never_uses_global_builder_or_native_fallback() {
    const CHILD: &str = "CORBANU_KEYRING_FIXTURE_CANARY_CHILD";
    if std::env::var_os(CHILD).is_none() {
        let output = std::process::Command::new(std::env::current_exe().expect("test executable"))
            .args([
                "--exact",
                "test_support::tests::explicit_fixture_never_uses_global_builder_or_native_fallback",
                "--nocapture",
            ])
            .env(CHILD, "1")
            .env("CORBANU_TEST_NO_NATIVE_KEYRING", "1")
            .output()
            .expect("isolated fixture child");
        assert!(
            output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        return;
    }
    keyring::set_default_credential_builder(Box::new(ForbiddenGlobalBuilder));
    super::install(Box::new(EmptyFixture));
    assert_eq!(DefaultKeyringStore.load("fixture", "canary").unwrap(), None);
    assert!(
        DefaultKeyringStore
            .save("fixture", "canary", "synthetic-value")
            .is_ok()
    );
    assert!(!DefaultKeyringStore.delete("fixture", "canary").unwrap());
    assert_eq!(CALLS.load(Ordering::SeqCst), 3);
}
