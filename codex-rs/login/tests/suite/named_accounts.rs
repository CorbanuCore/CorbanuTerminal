//! PF-84-S06 slice B: named ChatGPT sign-in accounts refresh, reload and log
//! out through their own vault store only. Runs with
//! `CORBANU_TEST_NO_NATIVE_KEYRING=1` (the isolated test runner), so the vault
//! key stays in the disposable home.

use std::ffi::OsString;
use std::sync::Arc;

use anyhow::Context;
use anyhow::Result;
use base64::Engine;
use chrono::Duration;
use chrono::Utc;
use codex_config::types::AuthCredentialsStoreMode;
use codex_login::AuthDotJson;
use codex_login::AuthKeyringBackendKind;
use codex_login::AuthManager;
use codex_login::NamedChatgptAccount;
use codex_login::REFRESH_TOKEN_URL_OVERRIDE_ENV_VAR;
use codex_login::save_auth;
use codex_login::token_data::IdTokenInfo;
use codex_login::token_data::TokenData;
use codex_protocol::auth::AuthMode;
use codex_vault::ProviderAccountKind;
use codex_vault::ProviderAccountName;
use codex_vault::Vault;
use core_test_support::skip_if_no_network;
use pretty_assertions::assert_eq;
use serde_json::json;
use tempfile::TempDir;
use wiremock::Mock;
use wiremock::MockServer;
use wiremock::ResponseTemplate;
use wiremock::matchers::body_partial_json;
use wiremock::matchers::method;
use wiremock::matchers::path;

fn jwt(payload: serde_json::Value) -> String {
    let b64 = |data: &[u8]| base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(data);
    format!(
        "{}.{}.{}",
        b64(br#"{"alg":"none","typ":"JWT"}"#),
        b64(payload.to_string().as_bytes()),
        b64(b"sig")
    )
}

/// A ChatGPT sign-in whose access token has expired a day ago.
fn sign_in(user: &str, access: &str, refresh: &str) -> AuthDotJson {
    AuthDotJson {
        auth_mode: Some(AuthMode::Chatgpt),
        openai_api_key: None,
        tokens: Some(TokenData {
            id_token: IdTokenInfo {
                raw_jwt: jwt(json!({ "sub": user })),
                ..Default::default()
            },
            access_token: access.to_string(),
            refresh_token: refresh.to_string(),
            account_id: Some(format!("{user}-workspace")),
        }),
        last_refresh: Some(Utc::now() - Duration::days(30)),
        agent_identity: None,
        personal_access_token: None,
        bedrock_api_key: None,
    }
}

fn account(name: &str) -> NamedChatgptAccount {
    NamedChatgptAccount {
        provider_id: "openai".to_string(),
        name: name.to_string(),
    }
}

fn write_account(home: &TempDir, name: &str, auth: &AuthDotJson) -> Result<()> {
    Vault::new(home.path().to_path_buf()).write_provider_account(
        "openai",
        &ProviderAccountName::parse(name)?,
        ProviderAccountKind::ChatgptAuth,
        &serde_json::to_string(auth)?,
    )?;
    Ok(())
}

fn read_account(home: &TempDir, name: &str) -> Result<Option<AuthDotJson>> {
    Vault::new(home.path().to_path_buf())
        .read_provider_account(
            "openai",
            &ProviderAccountName::parse(name)?,
            ProviderAccountKind::ChatgptAuth,
        )?
        .map(|raw| serde_json::from_str(&raw).context("stored sign-in"))
        .transpose()
}

fn tokens(auth: Option<AuthDotJson>) -> Option<(String, String)> {
    auth.and_then(|auth| auth.tokens)
        .map(|tokens| (tokens.access_token, tokens.refresh_token))
}

struct Fixture {
    home: TempDir,
    base: Arc<AuthManager>,
    default_bytes: Vec<u8>,
    _env: EnvGuard,
}

/// A home with a default ChatGPT login in `auth.json` and two named
/// accounts, `work` and `play`, in the vault.
async fn fixture(server: &MockServer) -> Result<Fixture> {
    let home = TempDir::new()?;
    let env = EnvGuard::set(
        REFRESH_TOKEN_URL_OVERRIDE_ENV_VAR,
        format!("{}/oauth/token", server.uri()),
    );
    save_auth(
        home.path(),
        &sign_in("default-user", "default-access-1", "default-refresh-1"),
        AuthCredentialsStoreMode::File,
        AuthKeyringBackendKind::default(),
    )?;
    write_account(
        &home,
        "work",
        &sign_in("work-user", "work-access-1", "work-refresh-1"),
    )?;
    write_account(
        &home,
        "play",
        &sign_in("play-user", "play-access-1", "play-refresh-1"),
    )?;
    let default_bytes = std::fs::read(home.path().join("auth.json"))?;
    let base = AuthManager::shared(
        home.path().to_path_buf(),
        /*enable_codex_api_key_env*/ false,
        AuthCredentialsStoreMode::File,
        /*forced_chatgpt_workspace_id*/ None,
        /*chatgpt_base_url*/ None,
        AuthKeyringBackendKind::default(),
        codex_login::test_support::transport_default_auth_route_config(),
    )
    .await;
    Ok(Fixture {
        home,
        base,
        default_bytes,
        _env: env,
    })
}

async fn mount_refresh(server: &MockServer, from: &str, access: &str, refresh: &str) {
    Mock::given(method("POST"))
        .and(path("/oauth/token"))
        .and(body_partial_json(json!({ "refresh_token": from })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "access_token": access,
            "refresh_token": refresh,
        })))
        .expect(1)
        .mount(server)
        .await;
}

/// B2: two accounts refreshed at the same time each get their own rotated
/// tokens; the default login is untouched, byte for byte.
#[serial_test::serial(auth_env)]
#[tokio::test]
async fn concurrent_refreshes_write_back_to_their_own_account_only() -> Result<()> {
    skip_if_no_network!(Ok(()));
    let server = MockServer::start().await;
    mount_refresh(&server, "work-refresh-1", "work-access-2", "work-refresh-2").await;
    mount_refresh(&server, "play-refresh-1", "play-access-2", "play-refresh-2").await;
    let fixture = fixture(&server).await?;

    let work = fixture.base.for_named_account_loaded(account("work")).await;
    let play = fixture.base.for_named_account_loaded(account("play")).await;
    // One manager, so one refresh lock, per account in a process.
    assert!(Arc::ptr_eq(
        &work,
        &fixture.base.for_named_account(account("work"))
    ));
    let (work_result, play_result) = tokio::join!(
        work.refresh_token_from_authority(),
        play.refresh_token_from_authority()
    );
    work_result.context("work refresh")?;
    play_result.context("play refresh")?;

    assert_eq!(
        (
            tokens(read_account(&fixture.home, "work")?),
            tokens(read_account(&fixture.home, "play")?),
        ),
        (
            Some(("work-access-2".to_string(), "work-refresh-2".to_string())),
            Some(("play-access-2".to_string(), "play-refresh-2".to_string())),
        )
    );
    assert_eq!(
        (
            work.auth().await.and_then(|auth| auth.get_token().ok()),
            play.auth().await.and_then(|auth| auth.get_token().ok()),
            fixture
                .base
                .auth()
                .await
                .and_then(|auth| auth.get_token().ok()),
        ),
        (
            Some("work-access-2".to_string()),
            Some("play-access-2".to_string()),
            Some("default-access-1".to_string()),
        )
    );
    assert_eq!(
        std::fs::read(fixture.home.path().join("auth.json"))?,
        fixture.default_bytes
    );
    Ok(())
}

/// B3: logging out of `work` revokes only its token and clears only its
/// store; `play` and the default login stay. An account without a sign-in
/// never falls back to the default login.
#[serial_test::serial(auth_env)]
#[tokio::test]
async fn logout_clears_only_that_account_and_missing_accounts_never_fall_back() -> Result<()> {
    skip_if_no_network!(Ok(()));
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/oauth/revoke"))
        .and(body_partial_json(json!({ "token": "work-refresh-1" })))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&server)
        .await;
    let fixture = fixture(&server).await?;

    let work = fixture.base.for_named_account_loaded(account("work")).await;
    assert!(work.logout_with_revoke().await?);
    assert_eq!(
        (
            read_account(&fixture.home, "work")?,
            tokens(read_account(&fixture.home, "play")?),
            work.auth().await.is_none(),
        ),
        (
            None,
            Some(("play-access-1".to_string(), "play-refresh-1".to_string())),
            true,
        )
    );
    assert_eq!(
        std::fs::read(fixture.home.path().join("auth.json"))?,
        fixture.default_bytes
    );
    let missing = fixture.base.for_named_account(account("missing"));
    assert!(missing.auth().await.is_none());
    Ok(())
}

/// B2: another process's refresh (a newer sign-in saved to the vault) is
/// picked up by the guarded reload instead of refreshing again; a refresh the
/// server rejects names the recovery for that account.
#[serial_test::serial(auth_env)]
#[tokio::test]
async fn refreshes_from_another_process_are_picked_up_and_rejections_name_the_account() -> Result<()>
{
    skip_if_no_network!(Ok(()));
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/oauth/token"))
        .and(body_partial_json(
            json!({ "refresh_token": "play-refresh-1" }),
        ))
        .respond_with(ResponseTemplate::new(401).set_body_json(json!({
            "error": { "code": "refresh_token_expired" }
        })))
        .expect(1)
        .mount(&server)
        .await;
    let fixture = fixture(&server).await?;

    let work = fixture.base.for_named_account_loaded(account("work")).await;
    write_account(
        &fixture.home,
        "work",
        &sign_in("work-user", "work-access-9", "work-refresh-9"),
    )?;
    work.refresh_token().await.context("guarded reload")?;
    assert_eq!(
        work.auth().await.and_then(|auth| auth.get_token().ok()),
        Some("work-access-9".to_string())
    );

    let play = fixture.base.for_named_account_loaded(account("play")).await;
    let error = play
        .refresh_token_from_authority()
        .await
        .expect_err("rejected refresh")
        .to_string();
    assert!(
        error.contains("sign in again with `corbanu login --account play`"),
        "{error}"
    );
    assert_eq!(
        std::fs::read(fixture.home.path().join("auth.json"))?,
        fixture.default_bytes
    );
    Ok(())
}

/// A named account without a sign-in never falls back to `OPENAI_API_KEY`,
/// `CODEX_API_KEY` or the default login, even when they are set.
#[serial_test::serial(auth_env)]
#[tokio::test]
async fn a_named_account_without_a_sign_in_ignores_environment_keys() -> Result<()> {
    let server = MockServer::start().await;
    let fixture = fixture(&server).await?;
    let _openai = EnvGuard::set("OPENAI_API_KEY", "sk-env-openai-canary".to_string());
    let _codex = EnvGuard::set("CODEX_API_KEY", "sk-env-codex-canary".to_string());
    let base = AuthManager::shared(
        fixture.home.path().to_path_buf(),
        /*enable_codex_api_key_env*/ true,
        AuthCredentialsStoreMode::File,
        /*forced_chatgpt_workspace_id*/ None,
        /*chatgpt_base_url*/ None,
        AuthKeyringBackendKind::default(),
        codex_login::test_support::transport_default_auth_route_config(),
    )
    .await;
    let missing = base.for_named_account_loaded(account("missing")).await;
    assert!(missing.auth().await.is_none());
    Ok(())
}

/// B3: a device-code sign-in for a named account lands in its vault store,
/// never in `auth.json`; a first sign-in never replaces an existing account.
#[serial_test::serial(auth_env)]
#[tokio::test]
async fn device_code_sign_in_is_saved_to_the_named_account_only() -> Result<()> {
    skip_if_no_network!(Ok(()));
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/accounts/deviceauth/usercode"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "device_auth_id": "device-auth-1", "user_code": "CODE-1", "interval": "0"
        })))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/accounts/deviceauth/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "authorization_code": "code-1", "code_challenge": "c", "code_verifier": "v"
        })))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/oauth/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id_token": jwt(json!({ "sub": "user-new" })),
            "access_token": "new-access",
            "refresh_token": "new-refresh",
        })))
        .mount(&server)
        .await;
    // The refused second add revokes the tokens it could not save.
    Mock::given(method("POST"))
        .and(path("/oauth/revoke"))
        .and(body_partial_json(json!({ "token": "new-refresh" })))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&server)
        .await;
    let _revoke = EnvGuard::set(
        codex_login::REVOKE_TOKEN_URL_OVERRIDE_ENV_VAR,
        format!("{}/oauth/revoke", server.uri()),
    );
    let home = TempDir::new()?;
    let options = |name: &str, must_be_new: bool| {
        let mut opts = codex_login::ServerOptions::new(
            home.path().to_path_buf(),
            "client-id".to_string(),
            /*forced_chatgpt_workspace_id*/ None,
            AuthCredentialsStoreMode::File,
            AuthKeyringBackendKind::default(),
            codex_login::test_support::transport_default_auth_route_config(),
        );
        opts.issuer = server.uri();
        opts.open_browser = false;
        opts.named_account = Some(account(name));
        opts.named_account_must_be_new = must_be_new;
        opts
    };
    codex_login::run_device_code_login(options("fresh", true)).await?;
    assert_eq!(
        (
            tokens(read_account(&home, "fresh")?),
            home.path().join("auth.json").exists(),
        ),
        (
            Some(("new-access".to_string(), "new-refresh".to_string())),
            false,
        )
    );
    assert!(
        codex_login::run_device_code_login(options("fresh", true))
            .await
            .is_err(),
        "adding an existing account is refused"
    );
    Ok(())
}

/// A replaced sign-in is revoked only when the new one is another ChatGPT
/// user: revoking a same-user sign-in could end the new session too.
#[serial_test::serial(auth_env)]
#[tokio::test]
async fn replaced_sign_ins_are_revoked_only_for_another_user() -> Result<()> {
    skip_if_no_network!(Ok(()));
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/oauth/revoke"))
        .and(body_partial_json(json!({ "token": "work-refresh-1" })))
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/oauth/revoke"))
        .and(body_partial_json(json!({ "token": "work-refresh-2" })))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&server)
        .await;
    let fixture = fixture(&server).await?;
    let route = codex_login::test_support::transport_default_auth_route_config();
    // A sign-in whose ID token names its ChatGPT user.
    let as_user = |user: &str, refresh: &str| {
        let mut auth = sign_in("work-user", "access", refresh);
        if let Some(tokens) = auth.tokens.as_mut() {
            tokens.id_token = codex_login::token_data::parse_chatgpt_jwt_claims(&jwt(json!({
                "https://api.openai.com/auth": { "chatgpt_user_id": user },
            })))
            .expect("claims");
        }
        auth
    };
    write_account(&fixture.home, "work", &as_user("u-1", "work-refresh-1"))?;
    for (user, refresh) in [("u-1", "work-refresh-2"), ("u-2", "other-refresh-1")] {
        let previous = codex_login::current_named_sign_in(fixture.home.path(), &account("work"))
            .context("prior sign-in")?;
        write_account(&fixture.home, "work", &as_user(user, refresh))?;
        codex_login::revoke_replaced_sign_in(
            fixture.home.path(),
            &account("work"),
            previous,
            &route,
        )
        .await;
    }
    Ok(())
}

struct EnvGuard {
    key: &'static str,
    original: Option<OsString>,
}

impl EnvGuard {
    fn set(key: &'static str, value: String) -> Self {
        let original = std::env::var_os(key);
        // SAFETY: these tests run serially (`auth_env`), so updating the process environment is safe.
        unsafe {
            std::env::set_var(key, &value);
        }
        Self { key, original }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        // SAFETY: the guard restores the original value before other tests run.
        unsafe {
            match &self.original {
                Some(value) => std::env::set_var(self.key, value),
                None => std::env::remove_var(self.key),
            }
        }
    }
}
