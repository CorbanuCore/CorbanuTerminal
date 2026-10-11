use std::sync::Arc;

use codex_keyring_store::tests::MockKeyringStore;
use pretty_assertions::assert_eq;

use super::*;
use crate::AddCredential;

fn test_vault() -> (tempfile::TempDir, Vault) {
    let dir = tempfile::tempdir().expect("tempdir");
    let vault = Vault::new_with_keyring_store(
        dir.path().to_path_buf(),
        Arc::new(MockKeyringStore::default()),
    );
    (dir, vault)
}

fn name(raw: &str) -> ProviderAccountName {
    ProviderAccountName::parse(raw).expect("valid name")
}

#[test]
fn names_follow_the_frozen_grammar() {
    for valid in [
        "work",
        "a",
        "acct-2",
        "0",
        "abcdefghijklmnopqrstuvwxyz012345",
    ] {
        assert_eq!(
            ProviderAccountName::parse(valid).map(|n| n.0),
            Ok(valid.to_string())
        );
    }
    for invalid in [
        "",
        "-work",
        "Work",
        "a_b",
        "a/b",
        "a.b",
        "abcdefghijklmnopqrstuvwxyz0123456",
    ] {
        assert_eq!(
            ProviderAccountName::parse(invalid),
            Err(ProviderAccountError::InvalidName)
        );
    }
    assert_eq!(
        ProviderAccountName::parse("default"),
        Err(ProviderAccountError::ReservedName)
    );
    assert_eq!(parse_provider_account_selection("default"), Ok(None));
    assert_eq!(parse_provider_account_selection(""), Ok(None));
    assert_eq!(
        parse_provider_account_selection("work"),
        Ok(Some(name("work")))
    );
}

#[test]
fn labels_round_trip_and_reject_other_labels() {
    let label =
        provider_account_label("zai", &name("work"), ProviderAccountKind::ApiKey).expect("label");
    assert_eq!(label, "provider/zai/accounts/work/api_key");
    assert_eq!(
        parse_provider_account_label(&label),
        Some(("zai".to_string(), name("work"), ProviderAccountKind::ApiKey))
    );
    for other in [
        "provider/zai_api_key",
        "provider/claude-code-oauth-token",
        "provider/zai/accounts/default/api_key",
        "provider/zai/accounts/work/unknown",
        "provider/zai/accounts/work/api_key/extra",
        "provider/ZAI/accounts/work/api_key",
    ] {
        assert_eq!(parse_provider_account_label(other), None, "{other}");
    }
    assert_eq!(
        provider_account_label("Bad Id", &name("work"), ProviderAccountKind::ApiKey),
        Err(ProviderAccountError::InvalidProviderId)
    );
}

#[test]
fn accounts_are_isolated_per_provider_name_and_kind() {
    let (_dir, vault) = test_vault();
    vault
        .write_provider_account(
            "zai",
            &name("a"),
            ProviderAccountKind::ApiKey,
            "canary-zai-a",
        )
        .expect("write a");
    vault
        .write_provider_account(
            "zai",
            &name("b"),
            ProviderAccountKind::ApiKey,
            "canary-zai-b",
        )
        .expect("write b");
    vault
        .write_provider_account(
            "claude-plan",
            &name("a"),
            ProviderAccountKind::ClaudeOauthToken,
            "canary-claude-a",
        )
        .expect("write claude a");

    let read = |provider: &str, account: &str, kind| {
        vault
            .read_provider_account(provider, &name(account), kind)
            .expect("read")
            .map(|value| value.to_string())
    };
    assert_eq!(
        read("zai", "a", ProviderAccountKind::ApiKey).as_deref(),
        Some("canary-zai-a")
    );
    assert_eq!(
        read("zai", "b", ProviderAccountKind::ApiKey).as_deref(),
        Some("canary-zai-b")
    );
    assert_eq!(read("zai", "c", ProviderAccountKind::ApiKey), None);
    assert_eq!(
        read("zai", "a", ProviderAccountKind::ClaudeOauthToken),
        None
    );
    assert_eq!(read("kimi", "a", ProviderAccountKind::ApiKey), None);

    assert_eq!(
        vault.list_provider_accounts().expect("list"),
        vec![
            ProviderAccountMeta {
                provider_id: "claude-plan".to_string(),
                name: name("a"),
                kinds: vec![ProviderAccountKind::ClaudeOauthToken],
            },
            ProviderAccountMeta {
                provider_id: "zai".to_string(),
                name: name("a"),
                kinds: vec![ProviderAccountKind::ApiKey],
            },
            ProviderAccountMeta {
                provider_id: "zai".to_string(),
                name: name("b"),
                kinds: vec![ProviderAccountKind::ApiKey],
            },
        ]
    );

    assert!(
        vault
            .remove_provider_account("zai", &name("a"))
            .expect("remove")
    );
    assert!(
        !vault
            .remove_provider_account("zai", &name("a"))
            .expect("remove again")
    );
    assert_eq!(read("zai", "a", ProviderAccountKind::ApiKey), None);
    assert_eq!(
        read("zai", "b", ProviderAccountKind::ApiKey).as_deref(),
        Some("canary-zai-b")
    );
}

#[test]
fn rewriting_an_account_replaces_its_value() {
    let (_dir, vault) = test_vault();
    let work = name("work");
    for value in ["first", "second"] {
        vault
            .write_provider_account("zai", &work, ProviderAccountKind::ApiKey, value)
            .expect("write");
    }
    assert_eq!(
        vault
            .read_provider_account("zai", &work, ProviderAccountKind::ApiKey)
            .expect("read")
            .map(|value| value.to_string())
            .as_deref(),
        Some("second")
    );
    assert!(matches!(
        vault.write_provider_account("zai", &work, ProviderAccountKind::ApiKey, "  "),
        Err(VaultError::EmptySecret)
    ));
}

#[test]
fn subscription_material_is_refused_by_generic_vault_access() {
    let (_dir, vault) = test_vault();
    let work = name("work");
    vault
        .write_provider_account(
            "claude-plan",
            &work,
            ProviderAccountKind::ClaudeOauthToken,
            "canary-token",
        )
        .expect("write token");
    vault
        .write_provider_account("zai", &work, ProviderAccountKind::ApiKey, "canary-key")
        .expect("write key");
    let token_label = "provider/claude-plan/accounts/work/claude_oauth_token";
    assert!(matches!(
        vault.reveal(token_label),
        Err(VaultError::ProviderManagedCredential { .. })
    ));
    assert!(matches!(
        vault.add(AddCredential {
            label: "provider/openai/accounts/work/chatgpt_auth".to_string(),
            credential_type: CredentialType::BearerToken,
            provider: None,
            notes: None,
            revocation_notes: None,
            secret: "x".to_string(),
        }),
        Err(VaultError::ProviderManagedCredential { .. })
    ));
    // Named API keys stay operational credentials, like today's provider keys.
    assert_eq!(
        vault
            .reveal("provider/zai/accounts/work/api_key")
            .expect("reveal api key"),
        "canary-key"
    );
}

#[test]
fn an_entry_of_another_type_is_not_account_material() {
    let (_dir, vault) = test_vault();
    vault
        .add(AddCredential {
            label: "provider/zai/accounts/work/api_key".to_string(),
            credential_type: CredentialType::ManualSecret,
            provider: None,
            notes: None,
            revocation_notes: None,
            secret: "canary-hand-added".to_string(),
        })
        .expect("hand-added entry");
    assert_eq!(
        vault
            .read_provider_account("zai", &name("work"), ProviderAccountKind::ApiKey)
            .expect("read"),
        None
    );
    assert_eq!(vault.list_provider_accounts().expect("list"), Vec::new());
}

#[test]
fn scoped_broker_reads_refuse_named_subscription_material() {
    let (_dir, vault) = test_vault();
    vault
        .write_provider_account(
            "claude-plan",
            &name("work"),
            ProviderAccountKind::ClaudeOauthToken,
            "canary-token",
        )
        .expect("write token");
    assert!(matches!(
        vault.read_scoped_secret("provider/claude-plan/accounts/work/claude_oauth_token"),
        Err(crate::ScopedCredentialError::CredentialTypeDenied)
    ));
}

#[test]
fn rename_moves_every_kind_and_leaves_other_labels_alone() {
    let (_dir, vault) = test_vault();
    let (work, home, other) = (name("work"), name("home"), name("other"));
    vault
        .write_provider_account("claude-plan", &work, ProviderAccountKind::ClaudeOauthToken, "tok")
        .expect("write token");
    vault
        .write_provider_account("claude-plan", &work, ProviderAccountKind::ClaudeConfigDir, "/d")
        .expect("write dir");
    vault
        .write_provider_account("claude-plan", &other, ProviderAccountKind::ClaudeOauthToken, "o")
        .expect("write other");
    let labels_before = vault.list().expect("list").len();

    vault
        .rename_provider_account("claude-plan", &work, &home)
        .expect("rename");

    assert_eq!(vault.list().expect("list").len(), labels_before);
    assert_eq!(
        vault.provider_account_kinds("claude-plan", &work).expect("kinds"),
        Vec::new()
    );
    assert_eq!(
        vault.provider_account_kinds("claude-plan", &home).expect("kinds"),
        vec![
            ProviderAccountKind::ClaudeOauthToken,
            ProviderAccountKind::ClaudeConfigDir
        ]
    );
    assert_eq!(
        vault
            .read_provider_account("claude-plan", &home, ProviderAccountKind::ClaudeOauthToken)
            .expect("read")
            .map(|value| value.to_string())
            .as_deref(),
        Some("tok")
    );
    // An existing target or a missing source is refused and changes nothing.
    assert!(vault.rename_provider_account("claude-plan", &home, &other).is_err());
    assert!(vault.rename_provider_account("claude-plan", &work, &name("x")).is_err());
    assert_eq!(vault.list().expect("list").len(), labels_before);
}

#[test]
fn fingerprints_identify_material_without_revealing_it() {
    let (_dir, vault) = test_vault();
    let (a, b, c) = (name("a"), name("b"), name("c"));
    assert_eq!(vault.provider_account_fingerprint("zai", &a).expect("fp"), None);
    for (account, value) in [(&a, "same-key"), (&b, "same-key"), (&c, "other-key")] {
        vault
            .write_provider_account("zai", account, ProviderAccountKind::ApiKey, value)
            .expect("write");
    }
    let fp = |account| {
        vault
            .provider_account_fingerprint("zai", account)
            .expect("fp")
            .expect("present")
    };
    let (fa, fb, fc) = (fp(&a), fp(&b), fp(&c));
    assert_eq!(fa.len(), 12);
    assert!(fa.bytes().all(|byte| byte.is_ascii_hexdigit()));
    assert_eq!(fa, fb, "same material, same fingerprint");
    assert_ne!(fa, fc);
    assert_eq!(fp(&a), fa, "stable across calls");
    // The salt is per home and is not a listed credential.
    assert!(vault.list().expect("list").iter().all(|entry| !entry.label.contains("SALT")));
    let (_other_dir, other_home) = test_vault();
    other_home
        .write_provider_account("zai", &a, ProviderAccountKind::ApiKey, "same-key")
        .expect("write");
    assert_ne!(
        other_home.provider_account_fingerprint("zai", &a).expect("fp"),
        Some(fa)
    );
}
