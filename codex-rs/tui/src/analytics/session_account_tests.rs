//! D562 (#2381): Analytics runs on the account the session runs on. A session
//! on a named ChatGPT account never sends the default login's token.
//! Runs under the isolated test runner (`CORBANU_TEST_NO_NATIVE_KEYRING=1`).

use super::*;
use crate::legacy_core::config::ConfigBuilder;
use base64::Engine;
use codex_config::LoaderOverrides;
use pretty_assertions::assert_eq;
use serde_json::json;
use wiremock::Mock;
use wiremock::MockServer;
use wiremock::ResponseTemplate;
use wiremock::matchers::method;
use wiremock::matchers::path;

/// A ChatGPT sign-in for `account` and `user`; its access token is returned.
fn chatgpt_sign_in(account: &str, user: &str) -> (codex_login::AuthDotJson, String) {
    let claims = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(
        json!({
            "exp": 4102444800_i64, "email": format!("{user}@example.test"),
            "https://api.openai.com/auth": {
                "chatgpt_account_id": account, "chatgpt_user_id": user, "chatgpt_plan_type": "plus",
            },
        })
        .to_string(),
    );
    let token = format!("e30.{claims}.test");
    let auth = serde_json::from_value(json!({
        "auth_mode": "chatgpt", "tokens": {"id_token": token, "access_token": token,
            "refresh_token": "test-refresh", "account_id": account},
        "last_refresh": chrono::Utc::now(),
    }))
    .unwrap();
    (auth, token)
}

#[tokio::test]
async fn analytics_uses_the_named_account_the_session_runs_on() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/backend-api/wham/accounts/check"))
        .respond_with(ResponseTemplate::new(/*s*/ 200).set_body_json(json!({
            "accounts": [
                {"id": "account-default", "plan_type": "plus"},
                {"id": "account-work", "plan_type": "plus"},
            ],
        })))
        .mount(&server)
        .await;
    let home = tempfile::tempdir().unwrap();
    let (default_auth, default_token) = chatgpt_sign_in("account-default", "user-default");
    codex_login::save_auth(
        home.path(),
        &default_auth,
        codex_login::AuthCredentialsStoreMode::File,
        codex_login::AuthKeyringBackendKind::default(),
    )
    .unwrap();
    let (work_auth, work_token) = chatgpt_sign_in("account-work", "user-work");
    codex_login::save_named_account_auth(
        home.path(),
        &codex_login::NamedChatgptAccount {
            provider_id: "openai".to_string(),
            name: "work".to_string(),
        },
        &work_auth,
        /*must_be_new*/ false,
    )
    .unwrap();
    std::fs::write(
        home.path().join("config.toml"),
        "[features]\nnamed_accounts = true\n\n[provider_accounts]\nopenai = \"work\"\n",
    )
    .unwrap();
    let mut config = ConfigBuilder::default()
        .codex_home(home.path().to_path_buf())
        .loader_overrides(LoaderOverrides::without_managed_config_for_tests())
        .build()
        .await
        .unwrap();
    config.chatgpt_base_url = format!("{}/backend-api", server.uri());
    config.cli_auth_credentials_store_mode = codex_login::AuthCredentialsStoreMode::File;
    assert!(config.runs_on_named_sign_in_account());
    let live = Live::new(Arc::new(config), "2026-01-15".parse().unwrap());

    let session = live.session().await.unwrap();
    assert_eq!(session.backend.account().id, "account-work");
    assert_eq!(
        live.account_label().as_deref(),
        Some("user-work@example.test")
    );

    let requests = server.received_requests().await.unwrap();
    assert!(!requests.is_empty());
    for request in &requests {
        let header = |name: &str| {
            request
                .headers
                .get(name)
                .and_then(|value| value.to_str().ok())
                .unwrap_or_default()
                .to_string()
        };
        assert_eq!(header("authorization"), format!("Bearer {work_token}"));
        assert_eq!(header("chatgpt-account-id"), "account-work");
        assert!(!header("authorization").contains(&default_token));
    }
}
