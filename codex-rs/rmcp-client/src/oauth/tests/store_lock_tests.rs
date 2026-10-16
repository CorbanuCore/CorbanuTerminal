use std::process::Command;
use std::sync::mpsc;
use std::time::Duration;
use std::time::Instant;

use anyhow::Context;
use anyhow::Result;
use codex_config::types::AuthKeyringBackendKind;
use codex_keyring_store::KeyringStore;
use codex_keyring_store::tests::MockKeyringStore;
use oauth2::AccessToken;
use oauth2::RefreshToken;
use oauth2::Scope;
use oauth2::TokenResponse;
use oauth2::basic::BasicTokenType;
use pretty_assertions::assert_eq;
use rmcp::transport::auth::OAuthTokenResponse;
use rmcp::transport::auth::VendorExtraTokenFields;
use tracing::Event;
use tracing::Id;
use tracing::Metadata;
use tracing::Subscriber;
use tracing::span::Attributes;
use tracing::span::Record;
use tracing::subscriber::Interest;

use super::OAuthStore;
use super::OAuthStoreLock;
use super::OAuthStoreLockFailure;
use crate::oauth::StoredOAuthTokens;
use crate::oauth::WrappedOAuthTokenResponse;
use crate::oauth::fallback_file_path;
use crate::oauth::load_oauth_tokens_from_file;
use crate::oauth::load_oauth_tokens_from_keyring;
use crate::oauth::resolve_oauth_tokens_from_store_policy;
use crate::oauth::save_oauth_tokens_to_file;
use crate::oauth::save_oauth_tokens_to_file_with_lock_held;
use crate::oauth::save_oauth_tokens_to_secrets_keyring_with_lock_held;
use crate::oauth::save_oauth_tokens_with_keyring;
use crate::oauth::save_oauth_tokens_with_keyring_with_fallback_to_file;
use crate::oauth::test_support::TempCodexHome;
use codex_config::types::OAuthCredentialsStoreMode;

const STORE_LOCK_CONTENTION_EVENT_TARGET: &str = "codex_rmcp_client::oauth::store_lock::contention";
// Contention is proven by the tracing event emitted after a real WouldBlock. Keep the timeout
// generous because it only bounds a failed test; it must not turn worker scheduling latency into
// a false failure on loaded CI hosts.
const STORE_LOCK_CONTENTION_EVENT_TIMEOUT: Duration = Duration::from_secs(/*secs*/ 10);

fn assert_tokens_match_without_expiry(actual: &StoredOAuthTokens, expected: &StoredOAuthTokens) {
    assert_eq!(actual.server_name, expected.server_name);
    assert_eq!(actual.url, expected.url);
    assert_eq!(actual.client_id, expected.client_id);
    assert_eq!(actual.expires_at, expected.expires_at);
    assert_token_response_match_without_expiry(&actual.token_response, &expected.token_response);
}

fn assert_token_response_match_without_expiry(
    actual: &WrappedOAuthTokenResponse,
    expected: &WrappedOAuthTokenResponse,
) {
    let actual_response = &actual.0;
    let expected_response = &expected.0;

    assert_eq!(
        actual_response.access_token().secret(),
        expected_response.access_token().secret()
    );
    assert_eq!(actual_response.token_type(), expected_response.token_type());
    assert_eq!(
        actual_response.refresh_token().map(RefreshToken::secret),
        expected_response.refresh_token().map(RefreshToken::secret),
    );
    assert_eq!(actual_response.scopes(), expected_response.scopes());
    assert_eq!(
        actual_response.extra_fields().0,
        expected_response.extra_fields().0
    );
    assert_eq!(
        actual_response.expires_in().is_some(),
        expected_response.expires_in().is_some()
    );
}

fn sample_tokens() -> StoredOAuthTokens {
    let mut response = OAuthTokenResponse::new(
        AccessToken::new("access-token".to_string()),
        BasicTokenType::Bearer,
        VendorExtraTokenFields::default(),
    );
    response.set_refresh_token(Some(RefreshToken::new("refresh-token".to_string())));
    response.set_scopes(Some(vec![
        Scope::new("scope-a".to_string()),
        Scope::new("scope-b".to_string()),
    ]));
    let expires_in = Duration::from_secs(3600);
    response.set_expires_in(Some(&expires_in));
    let expires_at = crate::oauth::compute_expires_at_millis(&response);

    StoredOAuthTokens {
        server_name: "test-server".to_string(),
        url: "https://example.test".to_string(),
        client_id: "client-id".to_string(),
        token_response: WrappedOAuthTokenResponse(response),
        expires_at,
    }
}

#[test]
fn legacy_rmcp_oauth_keyring_credentials_remain_readable() -> Result<()> {
    let _env = TempCodexHome::new();
    let keyring_store = MockKeyringStore::default();
    let mut expected = sample_tokens();
    expected.expires_at = None;
    expected.token_response.0.set_expires_in(None);

    let serialized = serde_json::json!({
        "server_name": "test-server",
        "url": "https://example.test",
        "client_id": "client-id",
        "token_response": {
            "access_token": "access-token",
            "token_type": "Bearer",
            "refresh_token": "refresh-token",
            "scope": "scope-a scope-b",
        },
    })
    .to_string();
    let key = crate::oauth::compute_store_key(&expected.server_name, &expected.url)?;
    keyring_store.save(crate::oauth::KEYRING_SERVICE, &key, &serialized)?;

    let resolved = resolve_oauth_tokens_from_store_policy(
        &keyring_store,
        &expected.server_name,
        &expected.url,
        OAuthCredentialsStoreMode::Auto,
        AuthKeyringBackendKind::Direct,
    )?
    .expect("OAuth credentials written before the rmcp upgrade should remain readable");

    assert_eq!(
        resolved.store,
        crate::oauth::ResolvedOAuthCredentialStore::Keyring(AuthKeyringBackendKind::Direct)
    );
    assert_tokens_match_without_expiry(&resolved.tokens, &expected);
    assert!(crate::oauth::oauth_tokens_are_usable(&resolved.tokens));
    Ok(())
}

const LOCK_HOLDER_CHILD_TEST: &str =
    "oauth::store_lock::tests::store_lock_is_released_when_holder_process_exits_child";
const LOCK_HOLDER_READY_PATH_ENV: &str = "CODEX_OAUTH_STORE_LOCK_CHILD_READY_PATH";

#[test]
fn store_lock_is_released_when_holder_process_exits() -> Result<()> {
    let env = TempCodexHome::new();
    let ready_file = env.path().join("lock-holder-ready");
    let mut child = Command::new(std::env::current_exe()?)
        .arg("--exact")
        .arg(LOCK_HOLDER_CHILD_TEST)
        .arg("--ignored")
        .env("CODEX_HOME", env.path())
        .env(LOCK_HOLDER_READY_PATH_ENV, &ready_file)
        .spawn()
        .context("spawn OAuth store lock holder test process")?;

    let test_result = (|| -> Result<()> {
        let started = Instant::now();
        while !ready_file.exists() {
            if started.elapsed() > Duration::from_secs(/*secs*/ 5) {
                anyhow::bail!("timed out waiting for child process to acquire OAuth store lock");
            }
            std::thread::sleep(Duration::from_millis(/*millis*/ 20));
        }

        let error = match OAuthStoreLock::acquire_in(
            env.path(),
            OAuthStore::File,
            Duration::from_millis(/*millis*/ 100),
        ) {
            Ok(_) => {
                anyhow::bail!("live holder process should keep the OAuth store lock unavailable")
            }
            Err(error) => error,
        };
        assert!(matches!(error, OAuthStoreLockFailure::Timeout { .. }));

        child
            .kill()
            .context("kill OAuth store lock holder process")?;
        let status = child
            .wait()
            .context("wait for killed OAuth store lock holder process")?;
        assert!(!status.success());
        let _lock = OAuthStoreLock::acquire_in(
            env.path(),
            OAuthStore::File,
            Duration::from_secs(/*secs*/ 1),
        )?;
        Ok(())
    })();

    if let Ok(None) = child.try_wait() {
        let _ = child.kill();
        let _ = child.wait();
    }

    test_result
}

#[test]
#[ignore = "child process for store_lock_is_released_when_holder_process_exits"]
fn store_lock_is_released_when_holder_process_exits_child() -> Result<()> {
    let ready_file = match std::env::var_os(LOCK_HOLDER_READY_PATH_ENV) {
        Some(path) => std::path::PathBuf::from(path),
        None => return Ok(()),
    };
    let _lock = OAuthStoreLock::acquire(OAuthStore::File)?;
    std::fs::write(ready_file, b"ready")?;
    loop {
        std::thread::sleep(Duration::from_secs(/*secs*/ 60));
    }
}

#[test]
fn auto_save_secrets_lock_failure_does_not_fall_back_to_file() -> Result<()> {
    let env = TempCodexHome::new();
    let lock_dir = env.path().join("mcp-oauth-locks");
    std::fs::create_dir_all(&lock_dir)?;
    // Break only the Secrets lock path. The distinct File lock remains usable, so Auto would
    // successfully write fallback credentials if it mistook coordination failure for backend
    // unavailability.
    std::fs::create_dir(lock_dir.join("secrets-store.lock"))?;
    let keyring_store = MockKeyringStore::default();
    let tokens = sample_tokens();

    let error = save_oauth_tokens_with_keyring_with_fallback_to_file(
        &keyring_store,
        AuthKeyringBackendKind::Secrets,
        &tokens.server_name,
        &tokens,
    )
    .expect_err("aggregate-store lock failure must abort Auto persistence");

    assert!(error.downcast_ref::<OAuthStoreLockFailure>().is_some());
    assert!(!fallback_file_path()?.exists());
    save_oauth_tokens_to_file(&tokens)?;
    let loaded = load_oauth_tokens_from_file(&tokens.server_name, &tokens.url)?
        .expect("fallback File should remain independently writable");
    assert_tokens_match_without_expiry(&loaded, &tokens);
    Ok(())
}

#[test]
fn auto_load_secrets_lock_failure_does_not_fall_back_to_file() -> Result<()> {
    let env = TempCodexHome::new();
    let keyring_store = MockKeyringStore::default();
    let tokens = sample_tokens();
    save_oauth_tokens_to_file(&tokens)?;

    let lock_dir = env.path().join("mcp-oauth-locks");
    std::fs::create_dir(lock_dir.join("secrets-store.lock"))?;
    let error = resolve_oauth_tokens_from_store_policy(
        &keyring_store,
        &tokens.server_name,
        &tokens.url,
        OAuthCredentialsStoreMode::Auto,
        AuthKeyringBackendKind::Secrets,
    )
    .expect_err("aggregate-store lock failure must abort Auto resolution");

    assert!(error.downcast_ref::<OAuthStoreLockFailure>().is_some());
    let loaded = load_oauth_tokens_from_file(&tokens.server_name, &tokens.url)?
        .expect("fallback File should remain independently readable");
    assert_tokens_match_without_expiry(&loaded, &tokens);
    Ok(())
}

struct LockContentionSubscriber {
    contended_tx: mpsc::Sender<()>,
}

impl Subscriber for LockContentionSubscriber {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        metadata.target() == STORE_LOCK_CONTENTION_EVENT_TARGET
    }

    fn register_callsite(&self, metadata: &'static Metadata<'static>) -> Interest {
        if self.enabled(metadata) {
            Interest::always()
        } else {
            Interest::never()
        }
    }

    fn max_level_hint(&self) -> Option<tracing::level_filters::LevelFilter> {
        Some(tracing::level_filters::LevelFilter::DEBUG)
    }

    fn new_span(&self, _span: &Attributes<'_>) -> Id {
        Id::from_u64(/*u*/ 1)
    }

    fn record(&self, _span: &Id, _values: &Record<'_>) {}

    fn record_follows_from(&self, _span: &Id, _follows_from: &Id) {}

    fn event(&self, event: &Event<'_>) {
        if self.enabled(event.metadata()) {
            self.contended_tx
                .send(())
                .expect("signal actual OAuth store lock contention");
        }
    }

    fn enter(&self, _span: &Id) {}

    fn exit(&self, _span: &Id) {}
}

fn complete_after_store_lock_contention<T>(
    codex_home: &std::path::Path,
    store: OAuthStore,
    while_locked: impl FnOnce() -> Result<()>,
    operation: impl FnOnce() -> Result<T> + Send + 'static,
) -> Result<T>
where
    T: Send + 'static,
{
    std::thread::scope(|scope| {
        let held_lock =
            OAuthStoreLock::acquire_in(codex_home, store, Duration::from_millis(/*millis*/ 100))?;
        let (contended_tx, contended_rx) = mpsc::channel();
        let worker = scope.spawn(move || {
            tracing::subscriber::with_default(LockContentionSubscriber { contended_tx }, operation)
        });

        // This event is emitted only after `try_lock()` returns WouldBlock, so the test fails if the
        // operation stops acquiring the aggregate-store lock.
        contended_rx
            .recv_timeout(STORE_LOCK_CONTENTION_EVENT_TIMEOUT)
            .context("timed out waiting for actual OAuth store lock contention")?;
        while_locked()?;
        drop(held_lock);
        worker
            .join()
            .expect("contending OAuth store worker should finish")
    })
}

#[test]
fn file_store_lock_preserves_updates_for_different_servers() -> Result<()> {
    let env = TempCodexHome::new();
    let first = sample_tokens();
    let mut second = sample_tokens();
    second.server_name = "second-server".to_string();
    second.url = "https://second.example.test".to_string();

    let second_for_writer = second.clone();
    complete_after_store_lock_contention(
        env.path(),
        OAuthStore::File,
        || save_oauth_tokens_to_file_with_lock_held(&first),
        move || save_oauth_tokens_to_file(&second_for_writer),
    )?;

    let loaded_first = load_oauth_tokens_from_file(&first.server_name, &first.url)?
        .expect("first server tokens should remain stored");
    let loaded_second = load_oauth_tokens_from_file(&second.server_name, &second.url)?
        .expect("second server tokens should be stored");
    assert_tokens_match_without_expiry(&loaded_first, &first);
    assert_tokens_match_without_expiry(&loaded_second, &second);
    Ok(())
}

#[test]
fn file_store_load_and_delete_observe_aggregate_lock() -> Result<()> {
    let env = TempCodexHome::new();
    let tokens = sample_tokens();
    save_oauth_tokens_to_file(&tokens)?;

    let server_name = tokens.server_name.clone();
    let url = tokens.url.clone();
    let loaded = complete_after_store_lock_contention(
        env.path(),
        OAuthStore::File,
        || Ok(()),
        move || load_oauth_tokens_from_file(&server_name, &url),
    )?
    .expect("file credentials should remain readable after contention");
    assert_tokens_match_without_expiry(&loaded, &tokens);

    let key = crate::oauth::compute_store_key(&tokens.server_name, &tokens.url)?;
    let removed = complete_after_store_lock_contention(
        env.path(),
        OAuthStore::File,
        || Ok(()),
        move || crate::oauth::delete_oauth_tokens_from_file(&key),
    )?;
    assert!(removed);
    assert!(load_oauth_tokens_from_file(&tokens.server_name, &tokens.url)?.is_none());
    Ok(())
}

#[test]
fn secrets_store_lock_preserves_updates_for_different_servers() -> Result<()> {
    let env = TempCodexHome::new();
    let keyring_store = MockKeyringStore::default();
    let first = sample_tokens();
    let mut second = sample_tokens();
    second.server_name = "second-server".to_string();
    second.url = "https://second.example.test".to_string();

    let store_for_writer = keyring_store.clone();
    let second_for_writer = second.clone();
    complete_after_store_lock_contention(
        env.path(),
        OAuthStore::Secrets,
        || {
            let first_serialized = serde_json::to_string(&first)?;
            save_oauth_tokens_to_secrets_keyring_with_lock_held(
                &keyring_store,
                &first.server_name,
                &first,
                &first_serialized,
            )
        },
        move || {
            save_oauth_tokens_with_keyring(
                &store_for_writer,
                AuthKeyringBackendKind::Secrets,
                &second_for_writer.server_name,
                &second_for_writer,
            )
        },
    )?;

    let loaded_first = load_oauth_tokens_from_keyring(
        &keyring_store,
        AuthKeyringBackendKind::Secrets,
        &first.server_name,
        &first.url,
    )?
    .expect("first server tokens should remain stored");
    let loaded_second = load_oauth_tokens_from_keyring(
        &keyring_store,
        AuthKeyringBackendKind::Secrets,
        &second.server_name,
        &second.url,
    )?
    .expect("second server tokens should be stored");
    assert_tokens_match_without_expiry(&loaded_first, &first);
    assert_tokens_match_without_expiry(&loaded_second, &second);
    Ok(())
}

#[test]
fn secrets_store_load_and_delete_observe_aggregate_lock() -> Result<()> {
    let env = TempCodexHome::new();
    let keyring_store = MockKeyringStore::default();
    let tokens = sample_tokens();
    save_oauth_tokens_with_keyring(
        &keyring_store,
        AuthKeyringBackendKind::Secrets,
        &tokens.server_name,
        &tokens,
    )?;

    let store_for_load = keyring_store.clone();
    let server_name = tokens.server_name.clone();
    let url = tokens.url.clone();
    let loaded = complete_after_store_lock_contention(
        env.path(),
        OAuthStore::Secrets,
        || Ok(()),
        move || {
            Ok(load_oauth_tokens_from_keyring(
                &store_for_load,
                AuthKeyringBackendKind::Secrets,
                &server_name,
                &url,
            )?)
        },
    )?
    .expect("encrypted credentials should remain readable after contention");
    assert_tokens_match_without_expiry(&loaded, &tokens);

    let store_for_delete = keyring_store.clone();
    let server_name = tokens.server_name.clone();
    let url = tokens.url.clone();
    let removed = complete_after_store_lock_contention(
        env.path(),
        OAuthStore::Secrets,
        || Ok(()),
        move || {
            crate::oauth::delete_oauth_tokens_from_secrets_keyring(
                &store_for_delete,
                &server_name,
                &url,
            )
        },
    )?;
    assert!(removed);
    assert!(
        load_oauth_tokens_from_keyring(
            &keyring_store,
            AuthKeyringBackendKind::Secrets,
            &tokens.server_name,
            &tokens.url,
        )?
        .is_none()
    );
    Ok(())
}

// Issue928: exercise the real acquired guard, not a replacement lock implementation.
// An error return must drop the actual guard before the caller observes the error.
fn store_error_return(_guard: OAuthStoreLock) -> std::result::Result<(), &'static str> {
    Err("synthetic transaction error")
}

#[derive(Clone, Copy, Debug)]
enum LockScopeExit {
    Normal,
    Error,
    Unwind,
}

fn assert_store_scope_releases_retained_descriptor(
    store: OAuthStore,
    exit: LockScopeExit,
) -> Result<()> {
    let home = tempfile::tempdir()?;
    let guard = OAuthStoreLock::acquire_in(home.path(), store, Duration::ZERO)?;
    let retained = guard._file.try_clone()?;
    let path = super::oauth_store_lock_path(home.path(), store);
    let before = std::fs::metadata(&path)?;
    match exit {
        LockScopeExit::Normal => drop(guard),
        LockScopeExit::Error => {
            let result = store_error_return(guard);
            assert_eq!(result, Err("synthetic transaction error"));
        }
        LockScopeExit::Unwind => {
            let result = std::panic::catch_unwind(move || {
                let _owned = guard;
                panic!("synthetic transaction unwind");
            });
            assert!(result.is_err());
        }
    }
    let reacquired = OAuthStoreLock::acquire_in(home.path(), store, Duration::ZERO);
    assert!(
        reacquired.is_ok(),
        "real {store} guard must release on {exit:?} while its duplicate remains open: {:?}",
        reacquired.as_ref().err()
    );
    let _reacquired = reacquired?;
    let after = std::fs::metadata(&path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        assert_eq!((before.dev(), before.ino()), (after.dev(), after.ino()));
        let retained_metadata = retained.metadata()?;
        assert_eq!(
            (before.dev(), before.ino()),
            (retained_metadata.dev(), retained_metadata.ino())
        );
    }
    #[cfg(not(unix))]
    assert_eq!(before.len(), after.len());
    drop(retained);
    Ok(())
}

#[test]
fn issue928_mcp_file_guard_scope_releases_with_retained_descriptor() -> Result<()> {
    for exit in [
        LockScopeExit::Normal,
        LockScopeExit::Error,
        LockScopeExit::Unwind,
    ] {
        assert_store_scope_releases_retained_descriptor(OAuthStore::File, exit)?;
    }
    Ok(())
}

#[test]
fn issue928_mcp_secrets_guard_scope_releases_with_retained_descriptor() -> Result<()> {
    for exit in [
        LockScopeExit::Normal,
        LockScopeExit::Error,
        LockScopeExit::Unwind,
    ] {
        assert_store_scope_releases_retained_descriptor(OAuthStore::Secrets, exit)?;
    }
    Ok(())
}

#[test]
fn issue928_mcp_live_store_guard_exclusion_and_independence() -> Result<()> {
    for store in [OAuthStore::File, OAuthStore::Secrets] {
        let home = tempfile::tempdir()?;
        let held = OAuthStoreLock::acquire_in(home.path(), store, Duration::ZERO)?;
        drop(held._file.try_clone()?);
        let other_store = match store {
            OAuthStore::File => OAuthStore::Secrets,
            OAuthStore::Secrets => OAuthStore::File,
        };
        let _independent = OAuthStoreLock::acquire_in(home.path(), other_store, Duration::ZERO)?;
        let contender = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(super::oauth_store_lock_path(home.path(), store))?;
        assert!(matches!(
            contender.try_lock(),
            Err(std::fs::TryLockError::WouldBlock)
        ));
        let error = OAuthStoreLock::acquire_in(home.path(), store, Duration::ZERO)
            .err()
            .expect("same-resource contender must time out while the guard lives");
        assert!(matches!(error, OAuthStoreLockFailure::Timeout { .. }));
        drop(held);
        let _reacquired = OAuthStoreLock::acquire_in(home.path(), store, Duration::ZERO)?;
    }
    Ok(())
}

#[test]
fn issue928_mcp_store_acquisition_io_errors_are_preserved() -> Result<()> {
    for store in [OAuthStore::File, OAuthStore::Secrets] {
        let home = tempfile::tempdir()?;
        let blocked_home = home.path().join("not-a-directory");
        std::fs::write(&blocked_home, b"synthetic sentinel")?;
        let error = OAuthStoreLock::acquire_in(&blocked_home, store, Duration::ZERO)
            .err()
            .expect("file-as-home must fail directory creation");
        assert!(matches!(error, OAuthStoreLockFailure::CreateDir { .. }));
        assert_eq!(std::fs::read(&blocked_home)?, b"synthetic sentinel");
        let path = super::oauth_store_lock_path(home.path(), store);
        std::fs::create_dir_all(&path)?;
        let error = OAuthStoreLock::acquire_in(home.path(), store, Duration::ZERO)
            .err()
            .expect("directory-as-lock must fail opening the file");
        assert!(matches!(error, OAuthStoreLockFailure::Open { .. }));
        assert!(path.is_dir());
    }
    Ok(())
}
