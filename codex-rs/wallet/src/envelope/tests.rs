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
fn short_passcodes_require_machine_secret_and_secret_never_appears_on_disk() {
    let home = tempfile::tempdir().expect("tempdir");
    let keyring = Arc::new(MockKeyringStore::default());
    let wallet = Wallet::new_with_keyring(home.path().to_path_buf(), keyring);
    let created = wallet.create("123456", Network::Mainnet).expect("create");
    assert!(wallet.unlock("123456").is_ok());
    let disk = fs::read_to_string(home.path().join("wallet/wallet.json")).expect("read");
    assert!(!disk.contains(&created.recovery_material[..]));
}
