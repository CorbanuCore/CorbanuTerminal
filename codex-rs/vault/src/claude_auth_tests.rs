use std::sync::Arc;

use codex_keyring_store::tests::MockKeyringStore;
use pretty_assertions::assert_eq;

use super::*;
use crate::AddCredential;
use crate::scoped_credential_callback_active;

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
fn macos_keychain_identity_matches_claude_code_profiles() {
    let config_dir = std::path::Path::new("/fixture/config");
    assert_eq!(
        claude_code_macos_keychain_service(config_dir, false, false),
        "Claude Code-credentials"
    );
    assert_eq!(
        claude_code_macos_keychain_service(config_dir, true, false),
        "Claude Code-credentials-9e67da4d"
    );
    let custom = claude_code_macos_keychain_service(config_dir, true, true);
    assert_eq!(custom, "Claude Code-custom-oauth-credentials-9e67da4d");
    assert_eq!(
        macos_keychain_claude_auth_source_id(&custom),
        "claude-login:macos-keychain:Claude Code-custom-oauth-credentials-9e67da4d"
    );
    assert_eq!(
        claude_code_macos_keychain_service(std::path::Path::new("/fixture/café"), true, false,),
        claude_code_macos_keychain_service(std::path::Path::new("/fixture/café"), true, false,)
    );
}

#[test]
fn credentials_file_identity_binds_the_exact_profile_without_exposing_its_path() {
    let profile_a = std::path::Path::new("/fixture/claude-work");
    let profile_b = std::path::Path::new("/fixture/claude-personal");
    let source_a = credentials_file_claude_auth_source_id(profile_a).unwrap();
    let source_b = credentials_file_claude_auth_source_id(profile_b).unwrap();

    assert_ne!(source_a, source_b);
    assert!(source_a.starts_with("claude-login:credentials-file:"));
    assert!(!source_a.contains("claude-work"));
    assert_eq!(
        credentials_file_claude_auth_source_id(std::path::Path::new("/fixture/café")).unwrap(),
        credentials_file_claude_auth_source_id(std::path::Path::new("/fixture/café")).unwrap()
    );
}

#[test]
fn relative_credentials_file_identity_is_bound_to_the_callers_working_directory() {
    let base = std::path::Path::new("/fixture/project");
    let relative =
        credentials_file_claude_auth_source_id_against(std::path::Path::new(".claude-work"), base)
            .unwrap();
    let absolute = credentials_file_claude_auth_source_id_against(
        std::path::Path::new("/fixture/project/.claude-work"),
        std::path::Path::new("/different-caller"),
    )
    .unwrap();

    assert_eq!(relative, absolute);
}

#[test]
fn managed_token_callback_activates_the_secret_bearing_panic_guard() {
    let (_directory, vault) = test_vault();
    vault
        .store_managed_claude_subscription_token("fixture-managed-token".to_string())
        .unwrap();

    let guard_was_active = vault
        .with_managed_claude_subscription_token(|_| scoped_credential_callback_active())
        .unwrap();

    assert!(guard_was_active);
    assert!(!scoped_credential_callback_active());
}

#[test]
fn managed_token_callback_panic_is_contained_without_formatting_its_payload() {
    let (_directory, vault) = test_vault();
    vault
        .store_managed_claude_subscription_token("fixture-managed-token".to_string())
        .unwrap();

    let error = vault
        .with_managed_claude_subscription_token(|_| -> () { panic!("managed-token-panic-canary") })
        .unwrap_err();

    assert!(error.to_string().contains("callback panicked"));
    assert!(!error.to_string().contains("managed-token-panic-canary"));
    assert!(!scoped_credential_callback_active());
}

#[test]
fn absent_selection_preserves_existing_installations() {
    let (_directory, vault) = test_vault();
    assert_eq!(vault.load_claude_auth_selection().unwrap(), None);
}

#[test]
fn selection_round_trips_in_the_encrypted_store() {
    let (directory, vault) = test_vault();
    let selection = ClaudeAuthSelection::new_at(
        ClaudeAuthSource::ManagedSubscriptionToken,
        "corbanu-vault:claude-plan",
        1_777_777_777,
    )
    .unwrap();
    vault.save_claude_auth_selection(&selection).unwrap();

    assert_eq!(vault.load_claude_auth_selection().unwrap(), Some(selection));
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
fn claude_login_authority_is_normalized_bound_and_metadata_only() {
    let authority =
        claude_login_authority_id(" User@Example.COM ", " ORG-Work ", Some(" Max ")).unwrap();
    assert_eq!(
        authority,
        claude_login_authority_id("user@example.com", "org-work", Some("max")).unwrap()
    );
    assert_ne!(
        authority,
        claude_login_authority_id("other@example.com", "org-work", Some("max")).unwrap()
    );
    assert_ne!(
        authority,
        claude_login_authority_id("user@example.com", "org-personal", Some("max")).unwrap()
    );
    assert_ne!(
        authority,
        claude_login_authority_id("user@example.com", "org-work", Some("team")).unwrap()
    );

    let selection =
        ClaudeAuthSelection::new_claude_code_login("claude-login:fixture", authority.clone())
            .unwrap();
    let serialized = serde_json::to_string(&selection).unwrap();
    for raw in [
        "User@Example.COM",
        "user@example.com",
        "ORG-Work",
        "org-work",
        "Max",
    ] {
        assert!(!serialized.contains(raw));
    }
    assert!(serialized.contains("claude-login-authority:sha256:"));
    let debug = format!("{selection:?}");
    assert!(!debug.contains(&authority));
    assert!(debug.contains("authority_bound: true"));

    for missing_subscription in [None, Some(""), Some("   ")] {
        assert!(
            claude_login_authority_id("user@example.com", "org-work", missing_subscription,)
                .is_err()
        );
    }
}

#[test]
fn environment_token_authority_is_normalized_bound_and_metadata_only() {
    let selected = " environment-token-fixture ";
    let selection = ClaudeAuthSelection::new_environment_token(selected).unwrap();
    let authority = selection
        .authority_id
        .as_deref()
        .expect("authority binding");

    assert_eq!(
        authority,
        claude_environment_token_authority_id("environment-token-fixture")
    );
    assert_ne!(
        authority,
        claude_environment_token_authority_id("different-environment-token")
    );
    assert!(ClaudeAuthSelection::new_environment_token(" \t ").is_err());

    let serialized = serde_json::to_string(&selection).unwrap();
    assert!(!serialized.contains(selected));
    assert!(!serialized.contains("environment-token-fixture"));
    assert!(serialized.contains("claude-environment-token-authority:sha256:"));
    assert!(!format!("{selection:?}").contains(authority));
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
fn generic_vault_writes_cannot_create_or_replace_the_managed_token() {
    let (_directory, vault) = test_vault();
    assert!(matches!(
        vault.add(AddCredential {
            label: MANAGED_CLAUDE_TOKEN_LABEL.to_string(),
            credential_type: CredentialType::ManualSecret,
            provider: None,
            notes: None,
            revocation_notes: None,
            secret: "bypass-managed-token-validation".to_string(),
        }),
        Err(VaultError::ProviderManagedCredential { .. })
    ));
    assert_eq!(
        vault.managed_claude_subscription_token_status().unwrap(),
        ManagedClaudeTokenStatus::Missing
    );

    vault
        .store_managed_claude_subscription_token("synthetic-managed-token".to_string())
        .unwrap();
    assert!(matches!(
        vault.update(
            MANAGED_CLAUDE_TOKEN_LABEL,
            Some("bypass-managed-token-replacement".to_string()),
            None,
            None,
            None,
        ),
        Err(VaultError::ProviderManagedCredential { .. })
    ));
    let resolved = vault
        .with_managed_claude_subscription_token(ToString::to_string)
        .unwrap();
    assert_eq!(resolved, "synthetic-managed-token");
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
