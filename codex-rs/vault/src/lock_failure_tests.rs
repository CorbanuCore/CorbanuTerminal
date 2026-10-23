#![allow(
    clippy::disallowed_methods,
    reason = "#[traced_test] installs tracing-test's in-memory global subscriber"
)]
//! #855: a vault write that cannot reach its storage stores nothing and says
//! why, and the text of a vault error message is never stored as a value.

use std::cell::Cell;
use std::sync::Arc;

use codex_keyring_store::tests::MockKeyringStore;
use pretty_assertions::assert_eq;

use super::*;

thread_local! {
    static INJECTED_LOCK_OPEN_ERRNO: Cell<Option<i32>> = const { Cell::new(None) };
}

/// The errno the next lock-file open on this thread fails with, if any.
pub(super) fn injected_lock_open_errno() -> Option<i32> {
    INJECTED_LOCK_OPEN_ERRNO.with(Cell::get)
}

fn test_vault() -> (tempfile::TempDir, Vault) {
    let dir = tempfile::tempdir().expect("tempdir");
    let keyring = Arc::new(MockKeyringStore::default());
    let vault = Vault::new_with_keyring_store(dir.path().to_path_buf(), keyring);
    (dir, vault)
}

fn entry(label: &str, secret: &str) -> AddCredential {
    AddCredential {
        label: label.to_string(),
        credential_type: CredentialType::ApiKey,
        provider: None,
        notes: None,
        revocation_notes: None,
        secret: secret.to_string(),
    }
}

fn labels(vault: &Vault) -> Vec<String> {
    vault
        .list()
        .expect("list")
        .into_iter()
        .map(|meta| meta.label)
        .collect()
}

fn vault_file(dir: &tempfile::TempDir) -> Vec<u8> {
    std::fs::read(dir.path().join("secrets").join("local.age")).expect("vault file")
}

#[test]
fn an_unopenable_lock_file_stores_nothing() {
    let (dir, vault) = test_vault();
    vault
        .add(entry("existing", "lock-test-value-0001"))
        .expect("first add");
    let before = vault_file(&dir);
    let lock = dir.path().join("secrets").join(".vault.lock");
    std::fs::remove_file(&lock).expect("remove lock file");
    std::fs::create_dir(&lock).expect("a directory where the lock file belongs");

    let error = vault
        .add(entry("moonshot_api_key", "lock-test-value-0002"))
        .expect_err("the add must fail");
    assert!(
        error
            .to_string()
            .starts_with(&format!("failed to open vault lock {}: ", lock.display())),
        "{error}"
    );

    std::fs::remove_dir(&lock).expect("restore lock file");
    assert_eq!(vault_file(&dir), before);
    assert_eq!(labels(&vault), vec!["existing".to_string()]);
}

#[test]
fn the_text_of_a_vault_error_is_never_stored_as_a_value() {
    let (dir, vault) = test_vault();
    vault
        .add(entry("existing", "lock-test-value-0001"))
        .expect("first add");
    let before = vault_file(&dir);
    let lock = dir.path().join("secrets").join(".vault.lock");
    // What the TUI showed in #855, as copied from the transcript.
    let shown = format!(
        "Failed to add vault credential \"moonshot_api_key\": failed to open vault lock {}",
        lock.display()
    );

    for value in [
        shown.clone(),
        format!("■ {shown}\n"),
        // Copied from a narrow terminal: wrapped onto several lines.
        shown.replace(" vault ", "\nvault\n"),
        format!("{shown}: Operation not permitted (os error 1)"),
        format!("FAILED TO OPEN VAULT LOCK {}", lock.display()),
        "Failed to replace vault credential \"existing\": failed to lock vault".to_string(),
    ] {
        let error = vault
            .add(entry("kimi_api", &value))
            .expect_err("error text must be refused");
        assert!(error.to_string().contains("nothing was stored"), "{error}");
        assert!(!error.to_string().contains("moonshot_api_key"), "{error}");
        let error = vault
            .update(
                "existing",
                Some(value),
                /*provider*/ None,
                /*notes*/ None,
                /*revocation_notes*/ None,
            )
            .expect_err("error text must be refused");
        assert!(error.to_string().contains("nothing was stored"), "{error}");
    }

    let name = ProviderAccountName::parse("work").expect("account name");
    let error = vault
        .write_provider_account("zai", &name, ProviderAccountKind::ApiKey, &shown)
        .expect_err("error text must be refused for a provider account");
    assert!(error.to_string().contains("nothing was stored"), "{error}");

    assert_eq!(vault_file(&dir), before);
    assert_eq!(labels(&vault), vec!["existing".to_string()]);
    assert_eq!(
        vault.reveal("existing").expect("reveal"),
        "lock-test-value-0001"
    );
}

#[test]
fn ordinary_values_are_not_mistaken_for_error_text() {
    for value in [
        "sk-lock-test-value-0001",
        "correct horse battery staple vault lock",
        "failed-to-open",
        "Failed to add a vault",
    ] {
        assert!(!is_vault_error_text(value), "{value}");
    }
}

/// The EPERM cases inject the failure; on Windows a refused read-write open
/// falls back to a read-only one (PF-27-S09), so they run on Unix only.
#[cfg(unix)]
mod eperm {
    use std::sync::Mutex;

    use pretty_assertions::assert_eq;
    use tracing_test::traced_test;

    use super::*;

    const EPERM: i32 = 1;

    /// Makes every lock-file open on this thread fail with `errno` until dropped.
    struct LockOpenFails;

    impl LockOpenFails {
        fn with(errno: i32) -> Self {
            INJECTED_LOCK_OPEN_ERRNO.with(|injected| injected.set(Some(errno)));
            Self
        }
    }

    impl Drop for LockOpenFails {
        fn drop(&mut self) {
            INJECTED_LOCK_OPEN_ERRNO.with(|injected| injected.set(None));
        }
    }

    #[test]
    fn a_lock_open_failure_stores_nothing_and_names_the_cause() {
        let (dir, vault) = test_vault();
        vault
            .add(entry("existing", "lock-test-value-0001"))
            .expect("first add");
        let before = vault_file(&dir);
        let lock = dir.path().join("secrets").join(".vault.lock");

        let error = {
            let _eperm = LockOpenFails::with(EPERM);
            vault
                .add(entry("moonshot_api_key", "lock-test-value-0002"))
                .expect_err("the add must fail")
        };
        assert_eq!(
            error.to_string(),
            format!(
                "failed to open vault lock {}: {}",
                lock.display(),
                std::io::Error::from_raw_os_error(EPERM)
            )
        );
        assert!(error.to_string().ends_with("(os error 1)"), "{error}");
        let error = {
            let _eperm = LockOpenFails::with(EPERM);
            vault
                .update(
                    "existing",
                    Some("lock-test-value-0003".to_string()),
                    /*provider*/ None,
                    /*notes*/ None,
                    /*revocation_notes*/ None,
                )
                .expect_err("the update must fail")
        };
        assert!(
            error.to_string().starts_with("failed to open vault lock"),
            "{error}"
        );

        assert_eq!(vault_file(&dir), before);
        assert_eq!(labels(&vault), vec!["existing".to_string()]);
        assert_eq!(
            vault.reveal("existing").expect("reveal"),
            "lock-test-value-0001"
        );
        assert!(
            matches!(
                vault.reveal("moonshot_api_key"),
                Err(VaultError::NotFound { .. })
            ),
            "the failed add must not leave the label behind"
        );
    }

    static REPORTED: Mutex<Vec<(String, PathBuf, Option<i32>)>> = Mutex::new(Vec::new());

    fn record_failure(operation: &str, path: &Path, error: &std::io::Error) {
        REPORTED.lock().expect("reported failures").push((
            operation.to_string(),
            path.to_path_buf(),
            error.raw_os_error(),
        ));
    }

    #[test]
    #[traced_test]
    fn a_permission_failure_is_reported_and_logged_without_the_value() {
        set_storage_failure_reporter(record_failure);
        let (dir, vault) = test_vault();
        let lock = dir.path().join("secrets").join(".vault.lock");

        {
            let _eperm = LockOpenFails::with(EPERM);
            vault
                .add(entry("moonshot_api_key", "lock-test-value-0004"))
                .expect_err("the add must fail");
        }

        let reported = REPORTED.lock().expect("reported failures").clone();
        assert!(
            reported.contains(&("open vault lock".to_string(), lock.clone(), Some(EPERM))),
            "{reported:?}"
        );
        assert!(logs_contain("vault storage unavailable"));
        assert!(logs_contain("operation=\"open vault lock\""));
        assert!(logs_contain(&format!("path={}", lock.display())));
        assert!(logs_contain("errno=1"));
        assert!(!logs_contain("lock-test-value-0004"));
        assert!(!logs_contain("moonshot_api_key"));
    }
}
