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
