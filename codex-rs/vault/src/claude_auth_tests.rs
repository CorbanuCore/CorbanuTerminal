use std::sync::Arc;

use codex_keyring_store::tests::MockKeyringStore;
use pretty_assertions::assert_eq;

use super::*;

fn test_vault() -> (tempfile::TempDir, Vault) {
    let directory = tempfile::tempdir().expect("tempdir");
    let vault = Vault::new_with_keyring_store(
        directory.path().to_path_buf(),
        Arc::new(MockKeyringStore::default()),
    );
    (directory, vault)
}

fn source(
    source: ClaudeAuthSource,
    source_id: &str,
    store: ClaudeAuthStoreKind,
    health: ClaudeAuthHealth,
) -> ClaudeAuthSourceMetadata {
    ClaudeAuthSourceMetadata {
        source,
        source_id: source_id.to_string(),
        store,
        health,
        account_hint: None,
    }
}

#[test]
fn absent_selection_preserves_existing_installations() {
    let (_directory, vault) = test_vault();
    assert_eq!(vault.load_claude_auth_selection().unwrap(), None);
}

#[test]
fn selection_round_trips_across_vault_instances_and_can_be_cleared() {
    let (directory, vault) = test_vault();
    let selection = ClaudeAuthSelection::new_at(
        ClaudeAuthSource::ManagedSubscriptionToken,
        "corbanu-vault:claude-plan",
        1_777_777_777,
    )
    .unwrap();
    vault.save_claude_auth_selection(&selection).unwrap();

    assert_eq!(vault.load_claude_auth_selection().unwrap(), Some(selection));
    assert!(vault.clear_claude_auth_selection().unwrap());
    assert_eq!(vault.load_claude_auth_selection().unwrap(), None);
    assert!(directory.path().join("secrets").join("local.age").is_file());
}

#[test]
fn missing_selection_never_auto_selects_an_available_source() {
    let available = vec![source(
        ClaudeAuthSource::ClaudeCodeLogin,
        "macos-keychain:default",
        ClaudeAuthStoreKind::MacosKeychain,
        ClaudeAuthHealth::Healthy,
    )];
    assert_eq!(
        resolve_claude_auth_source(None, &available),
        ClaudeAuthResolution::SelectionRequired { available }
    );
}

#[test]
fn exact_healthy_selection_resolves_without_falling_through() {
    let selection = ClaudeAuthSelection::new_at(
        ClaudeAuthSource::ClaudeCodeLogin,
        "macos-keychain:default",
        10,
    )
    .unwrap();
    let selected = source(
        ClaudeAuthSource::ClaudeCodeLogin,
        "macos-keychain:default",
        ClaudeAuthStoreKind::MacosKeychain,
        ClaudeAuthHealth::Healthy,
    );
    let other = source(
        ClaudeAuthSource::ManagedSubscriptionToken,
        "corbanu-vault:claude-plan",
        ClaudeAuthStoreKind::CorbanuVault,
        ClaudeAuthHealth::Healthy,
    );
    assert_eq!(
        resolve_claude_auth_source(Some(&selection), &[other, selected.clone()]),
        ClaudeAuthResolution::Selected(selected)
    );
}

#[test]
fn unhealthy_or_missing_selection_does_not_fall_back() {
    let selection = ClaudeAuthSelection::new_at(
        ClaudeAuthSource::ClaudeCodeLogin,
        "credentials-file:default",
        10,
    )
    .unwrap();
    let unhealthy = source(
        ClaudeAuthSource::ClaudeCodeLogin,
        "credentials-file:default",
        ClaudeAuthStoreKind::CredentialsFile,
        ClaudeAuthHealth::NeedsReauthorization,
    );
    let healthy_other = source(
        ClaudeAuthSource::ManagedSubscriptionToken,
        "corbanu-vault:claude-plan",
        ClaudeAuthStoreKind::CorbanuVault,
        ClaudeAuthHealth::Healthy,
    );
    assert_eq!(
        resolve_claude_auth_source(
            Some(&selection),
            &[healthy_other.clone(), unhealthy.clone()]
        ),
        ClaudeAuthResolution::UnhealthySelected(unhealthy)
    );

    let missing = ClaudeAuthSelection::new_at(
        ClaudeAuthSource::EnvironmentToken,
        "environment:CLAUDE_CODE_OAUTH_TOKEN",
        11,
    )
    .unwrap();
    assert_eq!(
        resolve_claude_auth_source(Some(&missing), &[healthy_other]),
        ClaudeAuthResolution::MissingSelected(missing)
    );
}

#[test]
fn duplicate_selected_identity_fails_as_a_deterministic_conflict() {
    let selection = ClaudeAuthSelection::new_at(
        ClaudeAuthSource::ClaudeCodeLogin,
        "claude-login:default",
        10,
    )
    .unwrap();
    let file = source(
        ClaudeAuthSource::ClaudeCodeLogin,
        "claude-login:default",
        ClaudeAuthStoreKind::LegacyCredentialsFile,
        ClaudeAuthHealth::Healthy,
    );
    let keychain = source(
        ClaudeAuthSource::ClaudeCodeLogin,
        "claude-login:default",
        ClaudeAuthStoreKind::MacosKeychain,
        ClaudeAuthHealth::Healthy,
    );
    assert_eq!(
        resolve_claude_auth_source(Some(&selection), &[keychain.clone(), file.clone()]),
        ClaudeAuthResolution::Conflict {
            selection,
            matches: vec![keychain, file],
        }
    );
}

#[test]
fn serialized_selection_contains_metadata_only() {
    let canary = "must-never-serialize";
    let selection = ClaudeAuthSelection::new_at(
        ClaudeAuthSource::ManagedSubscriptionToken,
        "corbanu-vault:claude-plan",
        10,
    )
    .unwrap();
    let serialized = serde_json::to_string(&selection).unwrap();
    assert!(!serialized.contains(canary));
    assert!(!serialized.to_ascii_lowercase().contains("token_value"));
    assert_eq!(
        serde_json::from_str::<ClaudeAuthSelection>(&serialized).unwrap(),
        selection
    );
}

#[test]
fn invalid_source_ids_are_rejected() {
    for source_id in ["", "line\nbreak"] {
        assert!(
            ClaudeAuthSelection::new_at(ClaudeAuthSource::ClaudeCodeLogin, source_id, 10).is_err()
        );
    }
}

#[test]
fn managed_token_round_trip_status_and_replace_are_metadata_only() {
    let (directory, vault) = test_vault();
    let first = "synthetic-oauth-token-first";
    let second = "synthetic-oauth-token-second";

    assert_eq!(
        vault.managed_claude_subscription_token_status().unwrap(),
        ManagedClaudeTokenStatus::Missing
    );
    let stored = vault
        .store_managed_claude_subscription_token(first.to_string())
        .unwrap();
    assert_eq!(
        vault.managed_claude_subscription_token_status().unwrap(),
        stored
    );
    let metadata = vault.show(MANAGED_CLAUDE_TOKEN_LABEL).unwrap();
    let metadata_json = serde_json::to_string(&metadata).unwrap();
    assert!(!metadata_json.contains(first));
    assert!(!format!("{metadata:?}").contains(first));
    assert!(matches!(
        vault.reveal(MANAGED_CLAUDE_TOKEN_LABEL),
        Err(VaultError::ProviderManagedCredential { .. })
    ));
    assert!(matches!(
        vault.reveal_for_programmatic_use(
            MANAGED_CLAUDE_TOKEN_LABEL,
            codex_security_policy::SecurityLevel::Permissive,
        ),
        Err(VaultError::ProviderManagedCredential { .. })
    ));

    vault
        .store_managed_claude_subscription_token(second.to_string())
        .unwrap();
    let resolved = vault
        .with_managed_claude_subscription_token(ToString::to_string)
        .unwrap();
    assert_eq!(resolved, second);
    let encrypted = std::fs::read(directory.path().join("secrets").join("local.age")).unwrap();
    assert!(!String::from_utf8_lossy(&encrypted).contains(first));
    assert!(!String::from_utf8_lossy(&encrypted).contains(second));
}

#[test]
fn managed_enrollment_commits_token_and_exact_selection_together() {
    let (_directory, vault) = test_vault();
    let selection = vault
        .enroll_managed_claude_subscription_token("synthetic-enrollment-token".to_string())
        .unwrap();

    assert_eq!(selection.source, ClaudeAuthSource::ManagedSubscriptionToken);
    assert_eq!(selection.source_id, MANAGED_CLAUDE_AUTH_SOURCE_ID);
    assert_eq!(vault.load_claude_auth_selection().unwrap(), Some(selection));
    assert_eq!(
        vault
            .with_managed_claude_subscription_token(ToString::to_string)
            .unwrap(),
        "synthetic-enrollment-token"
    );
}

#[test]
fn managed_enrollment_rolls_back_an_injected_token_write_failure() {
    assert_managed_enrollment_rollback(EnrollmentFailurePoint::AfterTokenWrite);
}

#[test]
fn managed_enrollment_rolls_back_an_injected_index_write_failure() {
    assert_managed_enrollment_rollback(EnrollmentFailurePoint::AfterIndexWrite);
}

fn assert_managed_enrollment_rollback(failure_point: EnrollmentFailurePoint) {
    let (_directory, vault) = test_vault();
    let previous_selection = ClaudeAuthSelection::new_at(
        ClaudeAuthSource::ClaudeCodeLogin,
        MACOS_KEYCHAIN_CLAUDE_AUTH_SOURCE_ID,
        10,
    )
    .unwrap();
    vault
        .save_claude_auth_selection(&previous_selection)
        .unwrap();
    vault
        .store_managed_claude_subscription_token("previous-managed-token".to_string())
        .unwrap();
    let error = vault
        .enroll_managed_claude_subscription_token_at(
            "replacement-managed-token".to_string(),
            20,
            failure_point,
        )
        .unwrap_err();

    assert!(
        error
            .to_string()
            .contains("injected managed-token enrollment failure")
    );
    assert_eq!(
        vault.load_claude_auth_selection().unwrap(),
        Some(previous_selection)
    );
    assert_eq!(
        vault
            .with_managed_claude_subscription_token(ToString::to_string)
            .unwrap(),
        "previous-managed-token"
    );
    assert!(!format!("{error:?}").contains("replacement-managed-token"));
}

#[test]
fn invalid_managed_tokens_fail_without_echoing_input() {
    let (_directory, vault) = test_vault();
    for token in ["", " leading", "two lines\nsecret"] {
        let error = vault
            .store_managed_claude_subscription_token(token.to_string())
            .unwrap_err();
        if !token.is_empty() {
            assert!(!error.to_string().contains(token));
            assert!(!format!("{error:?}").contains(token));
        }
    }
}

#[test]
fn managed_token_removal_preserves_unrelated_credentials() {
    let (_directory, vault) = test_vault();
    vault
        .add(crate::AddCredential {
            label: "provider/unrelated".to_string(),
            credential_type: crate::CredentialType::ApiKey,
            provider: Some("unrelated".to_string()),
            notes: None,
            revocation_notes: None,
            secret: "unrelated-secret".to_string(),
        })
        .unwrap();
    vault
        .store_managed_claude_subscription_token("synthetic-token".to_string())
        .unwrap();

    assert!(vault.remove_managed_claude_subscription_token().unwrap());
    let labels = vault
        .list()
        .unwrap()
        .into_iter()
        .map(|metadata| metadata.label)
        .collect::<Vec<_>>();
    assert_eq!(labels, vec!["provider/unrelated"]);
}
