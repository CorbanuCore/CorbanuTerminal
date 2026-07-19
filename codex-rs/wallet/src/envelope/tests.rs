use std::sync::Arc;

use codex_keyring_store::tests::MockKeyringStore;

use super::*;

#[test]
fn create_unlock_restore_and_tamper_fail_closed() {
    let first = tempfile::tempdir().expect("tempdir");
    let wallet = Wallet::new_with_keyring(
        first.path().to_path_buf(),
        Arc::new(MockKeyringStore::default()),
    );
    let created = wallet
        .create("correct horse battery", Network::Devnet)
        .expect("create");
    assert_eq!(
        wallet
            .unlock("correct horse battery")
            .expect("unlock")
            .manifest()
            .address,
        created.manifest.address
    );
    assert!(matches!(
        wallet.unlock("wrong passphrase"),
        Err(WalletError::UnlockFailed)
    ));

    let second = tempfile::tempdir().expect("tempdir");
    let restored = Wallet::new_with_keyring(
        second.path().to_path_buf(),
        Arc::new(MockKeyringStore::default()),
    )
    .restore(
        &created.recovery_material,
        "another long passphrase",
        Network::Devnet,
    )
    .expect("restore");
    assert_eq!(restored.manifest.address, created.manifest.address);

    let path = first.path().join("wallet/wallet.json");
    let mut envelope: serde_json::Value =
        serde_json::from_slice(&fs::read(&path).expect("read")).expect("json");
    envelope["memory_kib"] = serde_json::json!(999_999_999_u64);
    fs::write(&path, serde_json::to_vec(&envelope).expect("json")).expect("write");
    assert!(matches!(
        wallet.unlock("correct horse battery"),
        Err(WalletError::UnsafeParameters)
    ));
}

#[test]
fn remove_from_device_requires_the_current_address_and_removes_local_access() {
    let root = tempfile::tempdir().expect("tempdir");
    let wallet = Wallet::new(root.path().to_path_buf());
    let created = wallet
        .create("a sufficiently long test passphrase", Network::Mainnet)
        .expect("create wallet");

    assert!(matches!(
        wallet.remove_from_device("11111111111111111111111111111111"),
        Err(WalletError::AddressMismatch)
    ));
    assert!(wallet.exists());

    wallet
        .remove_from_device(&created.manifest.address)
        .expect("remove wallet");
    assert!(!wallet.exists());
    assert!(matches!(wallet.manifest(), Err(WalletError::Missing)));
}

#[test]
fn short_passcodes_require_machine_secret_and_secret_never_appears_on_disk() {
    let home = tempfile::tempdir().expect("tempdir");
    let keyring = Arc::new(MockKeyringStore::default());
    let wallet = Wallet::new_with_keyring(home.path().to_path_buf(), keyring);
    let created = wallet.create("123456", Network::Mainnet).expect("create");
    assert!(wallet.unlock("123456").is_ok());
    let disk = fs::read_to_string(home.path().join("wallet/wallet.json")).expect("read");
    assert!(!disk.contains(&created.recovery_material[..]));
}

#[test]
fn recovery_backup_requires_the_fresh_passcode_and_round_trips() {
    let home = tempfile::tempdir().expect("tempdir");
    let wallet = Wallet::new(home.path().to_path_buf());
    let created = wallet
        .create("a sufficiently long test passphrase", Network::Mainnet)
        .expect("create");

    assert!(matches!(
        wallet.export_recovery("wrong passphrase"),
        Err(WalletError::UnlockFailed)
    ));
    let backup = wallet
        .export_recovery("a sufficiently long test passphrase")
        .expect("export recovery");
    assert_eq!(backup.manifest.address, created.manifest.address);
    assert_eq!(
        &backup.recovery_material[..],
        &created.recovery_material[..]
    );

    let restored_home = tempfile::tempdir().expect("restored tempdir");
    let restored = Wallet::new(restored_home.path().to_path_buf())
        .restore(
            &backup.recovery_material,
            "another sufficiently long passphrase",
            Network::Mainnet,
        )
        .expect("restore backup");
    assert_eq!(restored.manifest.address, created.manifest.address);
}
