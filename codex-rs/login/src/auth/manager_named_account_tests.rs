//! Runs under the isolated test runner (`CORBANU_TEST_NO_NATIVE_KEYRING=1`).

use std::sync::Arc;

use codex_protocol::config_types::ForcedLoginMethod;
use pretty_assertions::assert_eq;

use super::*;
use crate::auth::named_chatgpt::login_named_account_with_api_key;

fn account(name: &str) -> NamedChatgptAccount {
    NamedChatgptAccount {
        provider_id: "openai".to_string(),
        name: name.to_string(),
    }
}

/// The manager itself refuses a sign-in the login restrictions rule out, and
/// a named manager hands default-account providers back to its base.
#[tokio::test]
#[serial_test::serial(codex_auth_env)]
async fn restricted_sign_ins_never_load_and_the_base_stays_the_default() {
    let home = tempfile::tempdir().expect("home");
    login_named_account_with_api_key(home.path(), &account("work"), "sk-canary-restricted")
        .expect("sign-in");
    let mut base = AuthManager::new(
        home.path().to_path_buf(),
        /*enable_codex_api_key_env*/ false,
        crate::AuthCredentialsStoreMode::File,
        /*forced_chatgpt_workspace_id*/ None,
        /*chatgpt_base_url*/ None,
        crate::AuthKeyringBackendKind::default(),
        crate::test_support::transport_default_auth_route_config(),
    )
    .await;
    base.forced_login_method = Some(ForcedLoginMethod::Chatgpt);
    let base = Arc::new(base);
    let work = base.for_named_account_loaded(account("work")).await;
    assert_eq!(
        (
            work.auth().await.is_none(),
            Arc::ptr_eq(&work.default_account_manager(), &base),
        ),
        (true, true)
    );
}

/// An unreadable stored sign-in can still be signed out (no revoke).
#[tokio::test]
async fn an_unreadable_sign_in_is_still_removed() {
    let home = tempfile::tempdir().expect("home");
    let name = codex_vault::ProviderAccountName::parse("work").expect("name");
    codex_vault::Vault::new(home.path().to_path_buf())
        .write_provider_account(
            "openai",
            &name,
            codex_vault::ProviderAccountKind::ChatgptAuth,
            "not json",
        )
        .expect("write");
    let base = AuthManager::from_auth_for_testing_with_home(
        CodexAuth::from_api_key("sk-default"),
        home.path().to_path_buf(),
    );
    let work = base.for_named_account(account("work"));
    assert_eq!(work.logout_with_revoke().await.expect("logout"), true);
    assert!(
        codex_vault::Vault::new(home.path().to_path_buf())
            .provider_account_kinds("openai", &name)
            .expect("kinds")
            .is_empty()
    );
}

fn chatgpt_sign_in(access_token: &str) -> crate::auth::storage::AuthDotJson {
    let claims = serde_json::json!({
        "email": "work@example.com",
        "https://api.openai.com/auth": {
            "chatgpt_user_id": "chatgpt-work",
            "chatgpt_account_id": "ws-work",
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
    crate::auth::storage::AuthDotJson {
        auth_mode: Some(AuthMode::Chatgpt),
        openai_api_key: None,
        // Not a JWT and no last refresh: no refresh is attempted.
        tokens: Some(crate::token_data::TokenData {
            id_token: crate::token_data::parse_chatgpt_jwt_claims(&jwt).expect("claims"),
            access_token: access_token.to_string(),
            refresh_token: "refresh-canary".to_string(),
            account_id: Some("ws-work".to_string()),
        }),
        last_refresh: None,
        agent_identity: None,
        personal_access_token: None,
        bedrock_api_key: None,
    }
}

fn access_token(auth: Option<CodexAuth>) -> Option<String> {
    auth.and_then(|auth| auth.get_current_auth_json())
        .and_then(|auth| auth.tokens)
        .map(|tokens| tokens.access_token)
}

/// D171 (#1286): a ChatGPT sign-in held in memory is dropped once another
/// process removes the account: the next `auth()` has none (so the request is
/// refused with the recovery), it never falls back to the default login, and
/// signing the account in again is picked up.
#[tokio::test]
async fn a_removed_accounts_sign_in_is_dropped_from_memory() {
    let home = tempfile::tempdir().expect("home");
    crate::auth::named_chatgpt::save_named_account_auth(
        home.path(),
        &account("work"),
        &chatgpt_sign_in("access-before"),
        /*must_be_new*/ true,
    )
    .expect("sign-in");
    let base = AuthManager::from_auth_for_testing_with_home(
        CodexAuth::from_api_key("sk-default-canary"),
        home.path().to_path_buf(),
    );
    let work = base.for_named_account(account("work"));
    assert_eq!(
        access_token(work.auth().await),
        Some("access-before".to_string())
    );

    // What `corbanu account remove openai work` does in another process (its
    // own vault handle; this process's managers are not told).
    let name = codex_vault::ProviderAccountName::parse("work").expect("name");
    assert!(
        codex_vault::Vault::new(home.path().to_path_buf())
            .remove_provider_account("openai", &name)
            .expect("remove")
    );

    // Synchronous readers (connector headers, MCP refresh) see it too.
    assert!(work.auth_cached().is_none());
    assert!(work.auth().await.is_none());
    let reason = work.named_sign_in_unavailable().expect("a reason");
    assert!(
        reason.contains("account `work` has no sign-in (it may have been removed or signed out)")
            && reason.contains("corbanu login --account work")
            && reason.contains("`/providers`"),
        "{reason}"
    );
    assert!(!reason.contains("sk-default-canary"), "{reason}");
    // The default login is untouched and never lent to the account.
    assert_eq!(
        base.auth()
            .await
            .and_then(|auth| auth.api_key().map(str::to_string)),
        Some("sk-default-canary".to_string())
    );

    crate::auth::named_chatgpt::save_named_account_auth(
        home.path(),
        &account("work"),
        &chatgpt_sign_in("access-after"),
        /*must_be_new*/ true,
    )
    .expect("sign in again");
    assert_eq!(
        access_token(work.auth().await),
        Some("access-after".to_string())
    );
}

/// D171 (#1286): an account removed and signed in again (here with other
/// tokens) by another process before the session's next request is not
/// served from the old sign-in: synchronous readers never see it again and
/// `auth()` returns the new one.
#[tokio::test]
async fn a_replaced_accounts_old_sign_in_is_not_used() {
    let home = tempfile::tempdir().expect("home");
    let save = |access: &str| {
        crate::auth::named_chatgpt::save_named_account_auth(
            home.path(),
            &account("work"),
            &chatgpt_sign_in(access),
            /*must_be_new*/ true,
        )
        .expect("sign-in");
    };
    save("access-old");
    let base = AuthManager::from_auth_for_testing_with_home(
        CodexAuth::from_api_key("sk-default-canary"),
        home.path().to_path_buf(),
    );
    let work = base.for_named_account(account("work"));
    assert_eq!(
        access_token(work.auth().await),
        Some("access-old".to_string())
    );

    let name = codex_vault::ProviderAccountName::parse("work").expect("name");
    assert!(
        codex_vault::Vault::new(home.path().to_path_buf())
            .remove_provider_account("openai", &name)
            .expect("remove")
    );
    let mut replacement = chatgpt_sign_in("access-new");
    if let Some(tokens) = replacement.tokens.as_mut() {
        tokens.refresh_token = "refresh-new".to_string();
    }
    crate::auth::named_chatgpt::save_named_account_auth(
        home.path(),
        &account("work"),
        &replacement,
        /*must_be_new*/ true,
    )
    .expect("sign in again");

    assert_ne!(
        access_token(work.auth_cached()),
        Some("access-old".to_string())
    );
    assert_eq!(
        access_token(work.auth().await),
        Some("access-new".to_string())
    );
    // Unchanged store: the same sign-in stays.
    assert_eq!(
        access_token(work.auth_cached()),
        Some("access-new".to_string())
    );
}
