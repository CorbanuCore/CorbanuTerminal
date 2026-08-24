use std::sync::Arc;
use std::sync::Barrier;

use chrono::Utc;
use codex_keyring_store::tests::MockKeyringStore;
use codex_security_policy::ActorChain;
use codex_security_policy::AuthorizationContext;
use codex_security_policy::AuthorizationRequest;
use codex_security_policy::BoundedText;
use codex_security_policy::PolicyAction;
use codex_security_policy::PolicyPrincipal;
use codex_security_policy::PrincipalKind;
use codex_security_policy::ProtectedResource;
use codex_security_policy::ResourceKind;
use codex_security_policy::compose_existing_decision;
use codex_security_policy::permissive_decision;
use pretty_assertions::assert_eq;

use super::*;

fn test_vault() -> (tempfile::TempDir, Arc<MockKeyringStore>, Vault) {
    let dir = tempfile::tempdir().expect("tempdir");
    let keyring = Arc::new(MockKeyringStore::default());
    let vault = Vault::new_with_keyring_store(dir.path().to_path_buf(), keyring.clone());
    (dir, keyring, vault)
}

fn api_key_entry(label: &str, secret: &str) -> AddCredential {
    AddCredential {
        label: label.to_string(),
        credential_type: CredentialType::ApiKey,
        provider: Some("ambient".to_string()),
        notes: Some("primary key".to_string()),
        revocation_notes: Some("rotate at https://console.example.com".to_string()),
        secret: secret.to_string(),
    }
}

#[test]
fn permissive_security_composition_preserves_vault_programmatic_use_decisions() {
    let human = PolicyPrincipal::new(PrincipalKind::Human, "human:jim").expect("human");
    let request = AuthorizationRequest::new(
        ActorChain::new(vec![human]).expect("actor chain"),
        ProtectedResource::new(ResourceKind::VaultCredential, "vault:representative")
            .expect("resource"),
        PolicyAction::Use,
        AuthorizationContext {
            now_unix_seconds: 1,
            session_id: BoundedText::new("session:vault-test").expect("session id"),
            task_id: BoundedText::new("task:vault-use").expect("task id"),
            purpose: BoundedText::new("permissive-compatibility").expect("purpose"),
            operation: BoundedText::new("vault.use").expect("operation"),
            destination: None,
            quantity: None,
            grant_id: None,
        },
    )
    .expect("request");
    let permissive = permissive_decision(&request).expect("Permissive decision");

    for (credential_type, expected_allow) in [
        (CredentialType::ApiKey, true),
        (CredentialType::BearerToken, true),
        (CredentialType::BasicAuth, true),
        (CredentialType::OauthClient, true),
        (CredentialType::CryptoPrivateKey, false),
        (CredentialType::SeedPhrase, false),
        (CredentialType::KeystoreJson, false),
        (CredentialType::RpcKey, true),
        (CredentialType::ExchangeKey, true),
        (CredentialType::DeploymentKey, true),
        (CredentialType::ManualSecret, true),
    ] {
        let existing_allow = credential_type.permits_programmatic_use();
        assert_eq!(existing_allow, expected_allow);
        assert_eq!(
            compose_existing_decision(existing_allow, &permissive),
            expected_allow,
            "Permissive changed the vault decision for {credential_type:?}"
        );
    }
}

#[test]
fn add_then_reveal_round_trips() {
    let (_dir, _keyring, vault) = test_vault();
    vault
        .add(api_key_entry("ambient/prod", "sk-secret-123"))
        .unwrap();

    assert!(vault.exists("ambient/prod").unwrap());
    let meta = vault.show("ambient/prod").unwrap();
    assert_eq!(meta.label, "ambient/prod");
    assert_eq!(meta.credential_type, CredentialType::ApiKey);
    assert_eq!(meta.provider.as_deref(), Some("ambient"));
    assert_eq!(meta.storage_backend, StorageBackend::EncryptedSecrets);

    // Listing must NOT include the raw secret.
    let listed = vault.list().unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0], meta);
    let listed_json = serde_json::to_string(&listed).unwrap();
    assert!(!listed_json.contains("sk-secret-123"));

    // Reveal returns the raw value.
    assert_eq!(vault.reveal("ambient/prod").unwrap(), "sk-secret-123");
}

#[test]
fn add_duplicate_label_is_rejected() {
    let (_dir, _keyring, vault) = test_vault();
    vault.add(api_key_entry("dupe", "one")).unwrap();
    let err = vault
        .add(api_key_entry("dupe", "two"))
        .expect_err("duplicate label should be rejected");
    assert!(
        matches!(err, VaultError::CredentialExists { ref label } if label == "dupe"),
        "unexpected error: {err:?}"
    );
    // Original secret is preserved.
    assert_eq!(vault.reveal("dupe").unwrap(), "one");
}

#[test]
fn add_empty_secret_is_rejected() {
    let (_dir, _keyring, vault) = test_vault();
    let mut entry = api_key_entry("empty", "x");
    entry.secret = "   ".to_string();
    let err = vault
        .add(entry)
        .expect_err("empty secret should be rejected");
    assert!(
        matches!(err, VaultError::EmptySecret),
        "unexpected: {err:?}"
    );
}

#[test]
fn invalid_labels_are_rejected() {
    let (_dir, _keyring, vault) = test_vault();
    for bad in ["", "    ", "has space", "has@symbol", &"x".repeat(129)] {
        let err = vault
            .add(api_key_entry(bad, "secret"))
            .err()
            .unwrap_or_else(|| panic!("expected label {bad:?} to be rejected"));
        assert!(
            matches!(err, VaultError::InvalidLabel(_)),
            "unexpected: {err:?}"
        );
    }
}

#[test]
fn label_is_trimmed_and_case_preserved() {
    let (_dir, _keyring, vault) = test_vault();
    vault.add(api_key_entry("  MyLabel  ", "secret")).unwrap();
    assert!(vault.exists("  MyLabel  ").unwrap());
    // Lookup uses the normalized (trimmed) form.
    assert!(vault.exists("MyLabel").unwrap());
    assert_eq!(vault.reveal("MyLabel").unwrap(), "secret");
}

#[test]
fn update_changes_secret_and_metadata() {
    let (_dir, _keyring, vault) = test_vault();
    vault.add(api_key_entry("k1", "old-secret")).unwrap();
    let before = vault.show("k1").unwrap();
    let updated = vault
        .update(
            "k1",
            Some("new-secret".to_string()),
            Some(Some("openrouter".to_string())),
            /*notes*/ None,
            /*revocation_notes*/ None,
        )
        .unwrap();
    assert_eq!(vault.reveal("k1").unwrap(), "new-secret");
    assert_eq!(updated.provider.as_deref(), Some("openrouter"));
    // created_at preserved, updated_at advanced (or equal if same second).
    assert_eq!(updated.created_at, before.created_at);
    assert!(updated.updated_at >= before.updated_at);
}

#[test]
fn update_missing_label_errors() {
    let (_dir, _keyring, vault) = test_vault();
    let err = vault
        .update(
            "nope",
            Some("x".to_string()),
            /*provider*/ None,
            /*notes*/ None,
            /*revocation_notes*/ None,
        )
        .expect_err("missing label should error");
    assert!(
        matches!(err, VaultError::NotFound { ref label } if label == "nope"),
        "unexpected: {err:?}"
    );
}

#[test]
fn delete_removes_credential_and_secret() {
    let (_dir, _keyring, vault) = test_vault();
    vault.add(api_key_entry("to-remove", "secret")).unwrap();
    assert!(vault.delete("to-remove").unwrap());
    assert!(!vault.exists("to-remove").unwrap());
    // Deleting again reports false (already gone).
    assert!(!vault.delete("to-remove").unwrap());
}

#[test]
fn delete_many_removes_present_credentials_and_ignores_missing_labels() {
    let (_dir, _keyring, vault) = test_vault();
    vault.add(api_key_entry("first", "secret-1")).unwrap();

    assert_eq!(
        vault
            .delete_many(&["first".into(), "missing".into()])
            .unwrap(),
        1
    );
    assert!(!vault.exists("first").unwrap());
}

#[test]
fn reveal_missing_label_errors() {
    let (_dir, _keyring, vault) = test_vault();
    let err = vault
        .reveal("ghost")
        .expect_err("missing label should error");
    assert!(
        matches!(err, VaultError::NotFound { ref label } if label == "ghost"),
        "unexpected: {err:?}"
    );
}

fn assert_programmatic_use_round_trip(credential_type: CredentialType, label: &str) {
    let (_dir, _keyring, vault) = test_vault();
    let secret = "programmatic-use-fixture";
    vault
        .add(AddCredential {
            label: label.to_string(),
            credential_type,
            provider: None,
            notes: None,
            revocation_notes: None,
            secret: secret.to_string(),
        })
        .unwrap();

    assert_eq!(vault.reveal_for_programmatic_use(label).unwrap(), secret);
}

#[test]
fn programmatic_use_accepts_manual_secrets_under_arbitrary_labels() {
    assert_programmatic_use_round_trip(CredentialType::ManualSecret, "infrastructure/primary");
}

#[test]
fn programmatic_use_accepts_api_keys_under_arbitrary_labels() {
    assert_programmatic_use_round_trip(CredentialType::ApiKey, "external-service/key");
}

#[test]
fn programmatic_use_accepts_deployment_credentials_under_arbitrary_labels() {
    assert_programmatic_use_round_trip(CredentialType::DeploymentKey, "hosting/deploy");
}

#[test]
fn programmatic_use_rejects_key_custody_material_without_deleting_it() {
    let (_dir, _keyring, vault) = test_vault();
    let credential_types = [CredentialType::CryptoPrivateKey, CredentialType::SeedPhrase];

    for (index, credential_type) in credential_types.into_iter().enumerate() {
        let label = format!("custody-{index}");
        let secret = format!("custody-material-{index}");
        vault
            .add(AddCredential {
                label: label.clone(),
                credential_type,
                provider: None,
                notes: None,
                revocation_notes: None,
                secret: secret.clone(),
            })
            .unwrap();

        let error = vault
            .reveal_for_programmatic_use(&label)
            .expect_err("key custody material must require explicit user access");
        assert!(matches!(
            error,
            VaultError::ProgrammaticUseDenied {
                label: denied_label,
                credential_type: denied_type,
            } if denied_label == label && denied_type == credential_type
        ));
        assert_eq!(vault.reveal(&label).unwrap(), secret);
    }
}

#[test]
fn multiple_credential_types_are_supported() {
    let (_dir, _keyring, vault) = test_vault();
    vault
        .add(AddCredential {
            label: "bearer".to_string(),
            credential_type: CredentialType::BearerToken,
            provider: None,
            notes: None,
            revocation_notes: None,
            secret: "token-abc".to_string(),
        })
        .unwrap();
    vault
        .add(AddCredential {
            label: "seed".to_string(),
            credential_type: CredentialType::SeedPhrase,
            provider: None,
            notes: None,
            revocation_notes: None,
            secret: "abandon amount bridge".to_string(),
        })
        .unwrap();
    let listed = vault.list().unwrap();
    assert_eq!(listed.len(), 2);
    // Sorted by label.
    assert_eq!(listed[0].label, "bearer");
    assert_eq!(listed[1].label, "seed");
    assert_eq!(vault.reveal("seed").unwrap(), "abandon amount bridge");
}

#[test]
fn timestamp_format_is_iso8601() {
    let now = Utc::now().timestamp();
    let formatted = format_timestamp(now);
    assert!(
        formatted.ends_with('Z'),
        "expected UTC 'Z' suffix: {formatted}"
    );
    assert!(
        formatted.contains('T'),
        "expected ISO-8601 'T' separator: {formatted}"
    );
}

#[test]
fn vault_index_is_encrypted_at_rest() {
    let (dir, _keyring, vault) = test_vault();
    vault
        .add(api_key_entry("secret-label", "plaintext-should-not-appear"))
        .unwrap();

    // The managed-secrets file must exist and must NOT contain the plaintext secret or label.
    let secrets_file = dir.path().join("secrets").join("local.age");
    assert!(
        secrets_file.exists(),
        "expected encrypted secrets file to exist"
    );
    let bytes = std::fs::read(&secrets_file).unwrap();
    let contents = String::from_utf8_lossy(&bytes);
    assert!(
        !contents.contains("plaintext-should-not-appear"),
        "raw secret leaked into the secrets file"
    );
}

#[test]
fn persistence_across_vault_instances() {
    // A vault persists into the age-encrypted secrets file. A second Vault instance
    // pointed at the same codex_home AND the same OS keyring store reads it back.
    // (The keyring holds the encryption passphrase, so it must be shared.)
    let (dir, keyring, vault) = test_vault();
    vault.add(api_key_entry("persist", "value-1")).unwrap();

    let vault2 = Vault::new_with_keyring_store(dir.path().to_path_buf(), keyring);
    assert_eq!(vault2.reveal("persist").unwrap(), "value-1");
}

#[test]
fn distinct_labels_do_not_collide_at_secret_layer() {
    // Regression: separator and case variants must map to DISTINCT secret records so adding one
    // never overwrites another's stored secret.
    let labels = ["a/b", "a.b", "a_b", "a-b", "MyLabel", "mylabel"];
    let secret_names = labels
        .into_iter()
        .map(|label| secret_name_for(label).expect("valid secret name"))
        .collect::<std::collections::HashSet<_>>();
    assert_eq!(secret_names.len(), labels.len());

    // Exercise the original separator-collision failure end to end through encrypted storage.
    // The exhaustive uniqueness assertion above keeps the expensive age integration below the
    // repository's per-test timeout while still covering every collision class.
    let (_dir, _keyring, vault) = test_vault();
    for (label, secret) in [("a/b", "slash"), ("a.b", "dot")] {
        vault
            .add(api_key_entry(label, secret))
            .unwrap_or_else(|e| panic!("adding {label:?} failed: {e:?}"));
    }

    // Each label independently resolves to its own secret — no clobbering.
    assert_eq!(vault.reveal("a/b").unwrap(), "slash");
    assert_eq!(vault.reveal("a.b").unwrap(), "dot");

    // Both metadata entries survive.
    assert_eq!(vault.list().unwrap().len(), 2);
}

#[test]
fn concurrent_unique_adds_preserve_every_credential() {
    const WRITERS: usize = 2;
    let (_dir, _keyring, vault) = test_vault();
    let vault = Arc::new(vault);
    let barrier = Arc::new(Barrier::new(WRITERS));
    let handles = (0..WRITERS)
        .map(|index| {
            let vault = Arc::clone(&vault);
            let barrier = Arc::clone(&barrier);
            std::thread::spawn(move || {
                barrier.wait();
                let label = format!("concurrent/{index:02}");
                let secret = format!("secret-{index:02}");
                vault.add(api_key_entry(&label, &secret))
            })
        })
        .collect::<Vec<_>>();

    for handle in handles {
        handle.join().expect("writer thread").expect("vault add");
    }

    let listed = vault.list().expect("list concurrent credentials");
    assert_eq!(
        listed
            .into_iter()
            .map(|credential| credential.label)
            .collect::<Vec<_>>(),
        vec!["concurrent/00".to_string(), "concurrent/01".to_string()]
    );
}

#[test]
fn wrong_key_fails_closed_without_replacing_encrypted_state() {
    let (dir, _keyring, vault) = test_vault();
    vault
        .add(api_key_entry("protected", "original-secret"))
        .expect("seed vault");
    let encrypted_path = dir.path().join("secrets").join("local.age");
    let encrypted_before = std::fs::read(&encrypted_path).expect("encrypted vault");

    let wrong_key_vault = Vault::new_with_keyring_store(
        dir.path().to_path_buf(),
        Arc::new(MockKeyringStore::default()),
    );
    assert!(wrong_key_vault.list().is_err());
    assert_eq!(
        std::fs::read(&encrypted_path).expect("encrypted vault after failed read"),
        encrypted_before
    );
    assert_eq!(
        vault.reveal("protected").expect("original key still works"),
        "original-secret"
    );
}

#[test]
fn corrupt_encrypted_state_fails_closed() {
    let (dir, keyring, vault) = test_vault();
    vault
        .add(api_key_entry("protected", "original-secret"))
        .expect("seed vault");
    let encrypted_path = dir.path().join("secrets").join("local.age");
    let mut bytes = std::fs::read(&encrypted_path).expect("encrypted vault");
    let payload_offset = bytes.len() / 2;
    bytes[payload_offset] = b'!';
    std::fs::write(&encrypted_path, bytes).expect("corrupt encrypted vault fixture");

    let reopened = Vault::new_with_keyring_store(dir.path().to_path_buf(), keyring);
    assert!(reopened.list().is_err());
    assert!(reopened.reveal("protected").is_err());
}
