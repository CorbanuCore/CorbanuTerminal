use super::*;
use crate::DefaultKeyringStore;
use crate::KeyringStore;
use crate::default_keyring_operation_gate;
use crate::run_bounded_keyring_operation;
use std::time::Duration;

#[test]
#[ignore = "uses a synthetic Keychain item trusted only to /usr/bin/security"]
fn native_denied_item_fails_without_repeated_ui() {
    let service = "Corbanu Keychain denied regression test";
    let account = format!("synthetic-{}", std::process::id());
    let created = std::process::Command::new("/usr/bin/security")
        .args([
            "add-generic-password",
            "-s",
            service,
            "-a",
            &account,
            "-w",
            "synthetic-denied",
            "-T",
            "/usr/bin/security",
        ])
        .output()
        .unwrap();
    assert!(created.status.success(), "synthetic item creation failed");
    let account_for_worker = account.clone();
    let outcome = run_bounded_keyring_operation(
        default_keyring_operation_gate(),
        "native-denial-test",
        Duration::from_secs(5),
        move || {
            // Consume the interactive permit without displaying UI. The actual
            // protected item is then read twenty times under the production
            // non-interactive path; no user credential is involved.
            run(service, &account_for_worker, || Ok(()))?;
            for _ in 0..20 {
                let result = run(service, &account_for_worker, || {
                    keyring::Entry::new(service, &account_for_worker)
                        .map_err(CredentialStoreError::new)?
                        .get_password()
                        .map_err(CredentialStoreError::new)
                });
                let error = result.expect_err("untrusted native reader must be denied");
                assert!(
                    error
                        .message()
                        .contains("repeated Keychain prompts are suppressed")
                );
                assert!(!error.message().contains("synthetic-denied"));
            }
            Ok(())
        },
    );
    let removed = std::process::Command::new("/usr/bin/security")
        .args(["delete-generic-password", "-s", service, "-a", &account])
        .output()
        .unwrap();
    assert!(removed.status.success(), "synthetic item cleanup failed");
    outcome.unwrap();
}

#[test]
#[ignore = "creates and removes only its own synthetic item in the real login Keychain"]
fn native_store_repeated_read_update_delete_remains_fresh() {
    let service = "Corbanu Keychain regression test";
    let account = format!("synthetic-{}", std::process::id());
    let store = DefaultKeyringStore;
    store.save(service, &account, "synthetic-first").unwrap();
    let outcome = std::panic::catch_unwind(|| {
        for _ in 0..20 {
            assert_eq!(
                store.load(service, &account).unwrap().as_deref(),
                Some("synthetic-first")
            );
        }
        store.save(service, &account, "synthetic-updated").unwrap();
        assert_eq!(
            store.load(service, &account).unwrap().as_deref(),
            Some("synthetic-updated")
        );
    });
    // Exact synthetic item only; no existing user credentials are accessed.
    assert!(store.delete(service, &account).unwrap());
    assert_eq!(store.load(service, &account).unwrap(), None);
    assert!(!store.delete(service, &account).unwrap());
    if let Err(error) = outcome {
        std::panic::resume_unwind(error);
    }
}

fn interaction_state() -> u8 {
    let mut value = 0;
    // SAFETY: Valid one-byte output pointer; read-only native state query.
    assert_eq!(
        unsafe { SecKeychainGetUserInteractionAllowed(&mut value) },
        0
    );
    value
}

#[test]
fn native_ui_suppression_restores_state_and_does_not_cache_results() {
    run_bounded_keyring_operation(
        default_keyring_operation_gate(),
        "native-ui-test",
        Duration::from_secs(5),
        || {
            let before = interaction_state();
            for (attempt, value) in ["first", "updated", "latest"].into_iter().enumerate() {
                let result = run("corbanu-synthetic-ui-test", "freshness", || {
                    assert_eq!(interaction_state(), if attempt == 0 { before } else { 0 });
                    Ok(value)
                })?;
                assert_eq!(result, value);
                assert_eq!(interaction_state(), before);
            }
            let error = run::<()>("corbanu-synthetic-ui-test", "freshness", || {
                assert_eq!(interaction_state(), 0);
                Err(CredentialStoreError::from_message("synthetic denial"))
            })
            .unwrap_err();
            assert!(error.message().contains("restart Corbanu"));
            assert_eq!(interaction_state(), before);
            let panic = std::panic::catch_unwind(|| {
                let _ = run::<()>("corbanu-synthetic-ui-test", "freshness", || {
                    assert_eq!(interaction_state(), 0);
                    panic!("synthetic worker panic");
                });
            });
            assert!(panic.is_err());
            assert_eq!(interaction_state(), before);
            Ok(())
        },
    )
    .unwrap();
}
