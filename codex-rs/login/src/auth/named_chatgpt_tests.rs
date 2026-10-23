use std::sync::Arc;

use codex_keyring_store::tests::MockKeyringStore;
use codex_protocol::auth::AuthMode;
use pretty_assertions::assert_eq;

use super::*;

fn account(name: &str) -> NamedChatgptAccount {
    NamedChatgptAccount {
        provider_id: "openai".to_string(),
        name: name.to_string(),
    }
}

fn api_key_login(key: &str) -> AuthDotJson {
    AuthDotJson {
        auth_mode: Some(AuthMode::ApiKey),
        openai_api_key: Some(key.to_string()),
        tokens: None,
        last_refresh: None,
        agent_identity: None,
        personal_access_token: None,
        bedrock_api_key: None,
    }
}

#[test]
fn each_account_reads_and_writes_only_its_own_label() {
    let home = tempfile::tempdir().expect("home");
    let vault = Vault::new_with_keyring_store(
        home.path().to_path_buf(),
        Arc::new(MockKeyringStore::default()),
    );
    let work = VaultAccountAuthStorage::with_vault(vault.clone(), &account("work")).expect("work");
    let play = VaultAccountAuthStorage::with_vault(vault, &account("play")).expect("play");
    assert_eq!(work.load().expect("empty"), None);

    work.save(&api_key_login("sk-canary-work"))
        .expect("save work");
    play.save(&api_key_login("sk-canary-play"))
        .expect("save play");
    assert_eq!(
        (work.load().expect("work"), play.load().expect("play")),
        (
            Some(api_key_login("sk-canary-work")),
            Some(api_key_login("sk-canary-play"))
        )
    );
    // The default account's file store is never written.
    assert!(!home.path().join("auth.json").exists());

    assert!(work.delete().expect("delete work"));
    assert_eq!(
        (work.load().expect("gone"), play.load().expect("kept")),
        (None, Some(api_key_login("sk-canary-play")))
    );
    assert!(!work.delete().expect("already gone"));
}

#[test]
fn an_unreadable_sign_in_never_quotes_its_value() {
    let home = tempfile::tempdir().expect("home");
    let vault = Vault::new_with_keyring_store(
        home.path().to_path_buf(),
        Arc::new(MockKeyringStore::default()),
    );
    let name = ProviderAccountName::parse("work").expect("name");
    vault
        .write_provider_account(
            "openai",
            &name,
            ProviderAccountKind::ChatgptAuth,
            "sk-canary-not-json",
        )
        .expect("write");
    let storage = VaultAccountAuthStorage::with_vault(vault, &account("work")).expect("storage");
    let error = storage.load().expect_err("not json").to_string();
    assert!(
        error.starts_with("the stored sign-in of account `work` is unreadable")
            && !error.contains("canary"),
        "{error}"
    );
    assert!(!format!("{storage:?}").contains("canary"));
}

fn chatgpt_sign_in(user: &str, workspace: &str) -> AuthDotJson {
    let claims = serde_json::json!({
        "email": format!("{user}@example.com"),
        "https://api.openai.com/auth": {
            "chatgpt_user_id": format!("chatgpt-{user}"),
            "chatgpt_account_id": workspace,
            "chatgpt_plan_type": "plus",
        },
    });
    let b64 = |data: &[u8]| {
        use base64::Engine;
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(data)
    };
    let jwt = format!(
        "{}.{}.{}",
        b64(b"{}"),
        b64(claims.to_string().as_bytes()),
        b64(b"sig")
    );
    AuthDotJson {
        auth_mode: Some(AuthMode::Chatgpt),
        openai_api_key: None,
        tokens: Some(crate::token_data::TokenData {
            id_token: crate::token_data::parse_chatgpt_jwt_claims(&jwt).expect("claims"),
            access_token: "access-canary".to_string(),
            refresh_token: "refresh-canary".to_string(),
            account_id: Some(workspace.to_string()),
        }),
        last_refresh: None,
        agent_identity: None,
        personal_access_token: None,
        bedrock_api_key: None,
    }
}

#[test]
fn summaries_show_kind_email_hint_and_plan_only() {
    assert_eq!(email_hint("travis@example.com"), "t…@example.com");
    assert_eq!(email_hint("no-at-sign"), "…");
    let summary = summarize(&chatgpt_sign_in("work", "ws-1"));
    assert_eq!(
        summary,
        NamedSignInSummary {
            kind: "ChatGPT sign-in",
            email_hint: Some("w…@example.com".to_string()),
            plan: Some("Plus".to_string()),
        }
    );
    assert!(!format!("{summary:?}").contains("canary"));
    assert_eq!(
        summarize(&api_key_login("sk-canary")).kind,
        "OpenAI API key"
    );
}

/// B5 (Travis 2026-10-11: warn, don't refuse): other sign-ins of the same
/// ChatGPT user are named, the default login included.
#[test]
fn same_user_sign_ins_name_the_default_and_other_accounts() {
    let home = tempfile::tempdir().expect("home");
    for (name, user) in [("work", "ann"), ("twin", "ann"), ("play", "bob")] {
        save_named_account_auth(
            home.path(),
            &account(name),
            &chatgpt_sign_in(user, "ws-1"),
            /*must_be_new*/ true,
        )
        .expect("save");
    }
    let default_login = chatgpt_sign_in("ann", "ws-2");
    assert_eq!(
        same_user_sign_ins(home.path(), &account("work"), Some(&default_login)).expect("same"),
        vec!["default".to_string(), "twin".to_string()]
    );
    assert_eq!(
        same_user_sign_ins(home.path(), &account("play"), Some(&default_login)).expect("same"),
        Vec::<String>::new()
    );
    // A first sign-in never replaces an existing account.
    assert!(
        save_named_account_auth(
            home.path(),
            &account("work"),
            &chatgpt_sign_in("bob", "ws-1"),
            /*must_be_new*/ true,
        )
        .is_err()
    );
}

#[test]
fn login_restrictions_refuse_the_other_kind_and_other_workspaces() {
    let work = account("work");
    let chatgpt = chatgpt_sign_in("ann", "ws-1");
    let api_key = api_key_login("sk-canary");
    assert_eq!(
        [
            sign_in_restriction(
                &work, &chatgpt, /*forced_login_method*/ None, /*forced_workspace*/ None
            ),
            sign_in_restriction(
                &work,
                &chatgpt,
                Some(ForcedLoginMethod::Chatgpt),
                Some(&["ws-1".to_string()])
            ),
            sign_in_restriction(
                &work,
                &api_key,
                Some(ForcedLoginMethod::Api),
                /*forced_workspace*/ None
            ),
        ],
        [None, None, None]
    );
    for refused in [
        sign_in_restriction(
            &work,
            &api_key,
            Some(ForcedLoginMethod::Chatgpt),
            /*forced_workspace*/ None,
        ),
        sign_in_restriction(
            &work,
            &chatgpt,
            Some(ForcedLoginMethod::Api),
            /*forced_workspace*/ None,
        ),
        sign_in_restriction(
            &work,
            &chatgpt,
            /*forced_login_method*/ None,
            Some(&["ws-other".to_string()]),
        ),
    ] {
        let reason = refused.expect("refused");
        assert!(
            reason.contains("account `work`") && !reason.contains("canary"),
            "{reason}"
        );
    }
}

/// #609: what a named sign-in holds (a ChatGPT sign-in or an OpenAI API key)
/// is recorded in its vault metadata when it is saved, never its value. One
/// saved before it was recorded gets it on its next read, once.
#[test]
fn what_a_sign_in_holds_is_recorded_in_its_metadata() {
    let home = tempfile::tempdir().expect("home");
    let vault = Vault::new_with_keyring_store(
        home.path().to_path_buf(),
        Arc::new(MockKeyringStore::default()),
    );
    let recorded = |name: &str| {
        vault
            .provider_account_sign_in("openai", &ProviderAccountName::parse(name).expect("name"))
            .expect("recorded")
    };
    let work = VaultAccountAuthStorage::with_vault(vault.clone(), &account("work")).expect("work");
    work.create(&api_key_login("sk-canary-work"))
        .expect("create");
    assert_eq!(recorded("work"), Some(ProviderAccountSignIn::ApiKey));
    work.save(&chatgpt_sign_in("work", "ws-1")).expect("save");
    assert_eq!(recorded("work"), Some(ProviderAccountSignIn::Chatgpt));

    // Saved by an earlier build: nothing recorded until it is read.
    let old_name = ProviderAccountName::parse("old").expect("name");
    let raw = serde_json::to_string(&api_key_login("sk-canary-old")).expect("json");
    vault
        .write_provider_account("openai", &old_name, ProviderAccountKind::ChatgptAuth, &raw)
        .expect("write");
    assert_eq!(recorded("old"), None);
    let old = VaultAccountAuthStorage::with_vault(vault.clone(), &account("old")).expect("old");
    assert_eq!(
        old.load().expect("load"),
        Some(api_key_login("sk-canary-old"))
    );
    assert_eq!(recorded("old"), Some(ProviderAccountSignIn::ApiKey));
    // Once recorded, nothing more is written.
    assert!(
        !vault
            .record_provider_account_sign_in("openai", &old_name, ProviderAccountSignIn::ApiKey)
            .expect("record")
    );

    let metadata = serde_json::to_string(&vault.list().expect("list")).expect("json");
    assert!(
        metadata.contains("\"sign_in\":\"api_key\"") && !metadata.contains("canary"),
        "{metadata}"
    );
}
