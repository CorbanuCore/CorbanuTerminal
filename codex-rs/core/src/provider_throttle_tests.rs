use super::*;
use codex_state::ProviderRequestResult;
use codex_vault::ProviderAccountKind;
use codex_vault::ProviderAccountName;
use codex_vault::Vault;
use pretty_assertions::assert_eq;

fn zai_on(account: Option<&str>) -> ModelProviderInfo {
    let mut provider = ModelProviderInfo::create_zai_provider();
    provider.account = account.map(|name| NamedProviderAccount {
        provider_id: "zai".to_string(),
        name: name.to_string(),
    });
    provider
}

#[test]
fn named_accounts_are_keyed_by_their_credential() {
    let key = |provider: &ModelProviderInfo| provider_request_key_identity(provider, || "unused");
    // The default account is keyed by its environment key (salted per home,
    // #482); a named account by what it holds, never by that key.
    assert!(match key(&zai_on(/*account*/ None)) {
        ProviderKeyIdentity::EnvSecret { env_key, .. } => env_key == "ZAI_API_KEY",
        ProviderKeyIdentity::Label(label) => label == "stored:ZAI_API_KEY",
        ProviderKeyIdentity::NamedAccount(_) => false,
    });
    assert_eq!(
        key(&zai_on(Some("work"))),
        ProviderKeyIdentity::NamedAccount(NamedProviderAccount {
            provider_id: "zai".to_string(),
            name: "work".to_string(),
        })
    );
    assert_eq!(named_account_label("work"), "account:work");
    assert_eq!(
        named_account_key("0123456789ab"),
        "account-key:0123456789ab"
    );
}

/// #482: the same key gives a different throttle identity on every home, and
/// neither identity is the unsalted value older builds stored.
#[test]
fn env_key_throttle_identity_is_salted_per_home() {
    let secret = "fixture-key-not-a-secret";
    let home_a = salted_env_key_fingerprint(&[1; 32], "ZAI_API_KEY", secret);
    let home_b = salted_env_key_fingerprint(&[2; 32], "ZAI_API_KEY", secret);
    let legacy = legacy_env_key_fingerprint("ZAI_API_KEY", secret);
    assert!(home_a.starts_with("env-salted:ZAI_API_KEY:"), "{home_a}");
    assert_eq!(home_a.len(), "env-salted:ZAI_API_KEY:".len() + 16);
    assert_ne!(home_a, home_b);
    assert_eq!(
        home_a,
        salted_env_key_fingerprint(&[1; 32], "ZAI_API_KEY", secret)
    );
    let unsalted: String = Sha256::digest(secret.as_bytes())
        .iter()
        .take(8)
        .map(|byte| format!("{byte:02x}"))
        .collect();
    assert_eq!(legacy, format!("env:ZAI_API_KEY:{unsalted}"));
    for salted in [&home_a, &home_b] {
        assert!(!salted.contains(&unsalted), "{salted}");
        assert!(!salted.contains(secret), "{salted}");
    }
}

async fn open_state_db(home: &Path) -> StateDbHandle {
    codex_state::StateRuntime::init(
        codex_state::SqliteConfig::from_sqlite_home(
            codex_utils_absolute_path::AbsolutePathBuf::try_from(home).expect("absolute"),
        ),
        "zai".to_string(),
    )
    .await
    .expect("state db")
}

struct Home {
    dir: tempfile::TempDir,
    state_db: StateDbHandle,
}

impl Home {
    async fn new() -> Self {
        let dir = tempfile::tempdir().expect("tempdir");
        let state_db = open_state_db(dir.path()).await;
        Self { dir, state_db }
    }

    /// Another process on the same home.
    async fn other_process(&self) -> StateDbHandle {
        open_state_db(self.dir.path()).await
    }

    fn enroll(&self, name: &str, key: &str) {
        Vault::new(self.dir.path().to_path_buf())
            .write_provider_account(
                "zai",
                &ProviderAccountName::parse(name).expect("name"),
                ProviderAccountKind::ApiKey,
                key,
            )
            .expect("enroll");
    }

    async fn key(&self, account: &str) -> ProviderRequestKey {
        provider_request_key(
            &self.state_db,
            self.dir.path(),
            "zai",
            &zai_on(Some(account)),
            "glm-5.3-flash",
            || "unused",
        )
        .await
    }

    async fn blocked(&self, key: &ProviderRequestKey) -> bool {
        blocked_in(&self.state_db, key).await
    }
}

async fn blocked_in(state_db: &StateDbHandle, key: &ProviderRequestKey) -> bool {
    let preflight = ProviderRequestPreflight {
        input_tokens: 1,
        cached_input_tokens: 0,
        request_bytes: 1,
        thread_id: None,
        turn_id: None,
    };
    state_db
        .check_provider_request_throttle(key, &preflight, now_unix_timestamp_ms())
        .await
        .decision
        .is_some()
}

fn rate_limited() -> ProviderRequestResult {
    ProviderRequestResult::Failed {
        status: Some(429),
        request_id: None,
        retry_after_ms: Some(120_000),
    }
}

/// Owner decision 2026-10-11: providers limit by key, so two names holding
/// one key share a cooldown, and two keys of one provider never do.
#[tokio::test]
async fn names_on_one_key_share_a_cooldown_and_other_keys_do_not() {
    let home = Home::new().await;
    home.enroll("alpha", "fixture-shared-key");
    home.enroll("beta", "fixture-shared-key");
    home.enroll("gamma", "fixture-other-key");
    let (alpha, beta, gamma) = (
        home.key("alpha").await,
        home.key("beta").await,
        home.key("gamma").await,
    );
    assert_eq!(alpha, beta);
    assert_ne!(alpha, gamma);
    for key in [&alpha, &gamma] {
        assert!(key.key_fingerprint.starts_with("account-key:"), "{key:?}");
        for leaked in ["alpha", "gamma", "fixture"] {
            assert!(!key.key_fingerprint.contains(leaked), "{key:?}");
        }
    }

    record_unleased_provider_request_result(&home.state_db, &alpha, rate_limited()).await;
    assert!(home.blocked(&home.key("beta").await).await);
    assert!(!home.blocked(&gamma).await);
}

/// Every file under `dir` but the state DB's, with its bytes.
fn files_outside_the_state_db(dir: &Path) -> std::collections::BTreeMap<PathBuf, Vec<u8>> {
    let mut files = std::collections::BTreeMap::new();
    let mut pending = vec![dir.to_path_buf()];
    while let Some(dir) = pending.pop() {
        for entry in std::fs::read_dir(&dir)
            .expect("read dir")
            .filter_map(Result::ok)
        {
            let path = entry.path();
            if path.is_dir() {
                pending.push(path);
            } else if !path.to_string_lossy().contains("sqlite") {
                files.insert(path.clone(), std::fs::read(&path).expect("read"));
            }
        }
    }
    files
}

/// #474: keying a request reads the vault and never writes it.
#[tokio::test]
async fn keying_a_named_account_never_writes_the_vault() {
    let home = Home::new().await;
    home.enroll("alpha", "fixture-shared-key");
    let before = files_outside_the_state_db(home.dir.path());
    let key = home.key("alpha").await;
    assert!(key.key_fingerprint.starts_with("account-key:"), "{key:?}");
    assert_eq!(files_outside_the_state_db(home.dir.path()), before);
}

/// An account with no fingerprint (no vault here; a vault whose accounts all
/// predate S05 has no fingerprint salt either, and the request path never
/// adds one, #474) is keyed by name, and no vault is created for it.
#[tokio::test]
async fn an_account_without_a_fingerprint_is_keyed_by_name() {
    let home = Home::new().await;
    let before = std::fs::read_dir(home.dir.path())
        .expect("home")
        .filter_map(Result::ok)
        .map(|entry| entry.file_name())
        .collect::<std::collections::BTreeSet<_>>();
    let key = home.key("never-enrolled").await;
    assert_eq!(key.key_fingerprint, "account:never-enrolled");
    let after = std::fs::read_dir(home.dir.path())
        .expect("home")
        .filter_map(Result::ok)
        .map(|entry| entry.file_name())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(before, after);
}

/// Rows kept by name before throttling went by credential carry an active
/// cooldown over to the credential's row, and the name's rows are dropped.
#[tokio::test]
async fn an_active_cooldown_kept_by_name_moves_to_the_credential() {
    let home = Home::new().await;
    let by_name = |name: &str| ProviderRequestKey {
        provider_id: "zai".to_string(),
        model: "glm-5.3-flash".to_string(),
        key_fingerprint: named_account_label(name),
    };
    // What an older build left behind: `beta` cooling down, `alpha` idle.
    let older = home.other_process().await;
    record_unleased_provider_request_result(&older, &by_name("beta"), rate_limited()).await;
    assert!(!blocked_in(&older, &by_name("alpha")).await);
    home.enroll("alpha", "fixture-shared-key");
    home.enroll("beta", "fixture-shared-key");

    // `alpha` adopts its idle row first; `beta`'s cooldown then merges in.
    let alpha = home.key("alpha").await;
    assert!(!home.blocked(&alpha).await);
    let beta = home.key("beta").await;
    assert_eq!(alpha, beta);
    assert!(home.blocked(&alpha).await);
    assert!(blocked_in(&home.other_process().await, &alpha).await);

    // The rows kept by name are gone: a fresh process sees no cooldown there.
    let fresh = home.other_process().await;
    assert!(!blocked_in(&fresh, &by_name("beta")).await);
}

/// A session on a mock OpenAI-compatible endpoint whose every request is
/// answered with a 429 asking for two minutes, with a state DB of its own.
pub(crate) struct RateLimitedSession {
    pub(crate) server: wiremock::MockServer,
    pub(crate) session: crate::session::session::Session,
    pub(crate) turn_context: TurnContext,
    pub(crate) home: tempfile::TempDir,
    pub(crate) cwd: tempfile::TempDir,
}

impl RateLimitedSession {
    pub(crate) async fn start() -> Self {
        let server = wiremock::MockServer::start().await;
        wiremock::Mock::given(wiremock::matchers::method("POST"))
            .respond_with(
                wiremock::ResponseTemplate::new(429)
                    .insert_header("retry-after", "120")
                    .set_body_json(serde_json::json!({
                        "error": {"message": "Rate limit reached", "type": "requests",
                            "code": "rate_limit_exceeded"}
                    })),
            )
            .mount(&server)
            .await;
        let home = tempfile::tempdir().expect("tempdir");
        let test_cwd = tempfile::tempdir().expect("create test cwd");
        let mut config =
            crate::session::tests::build_test_config(home.path(), test_cwd.path()).await;
        config.model_provider = ModelProviderInfo {
            request_max_retries: Some(0),
            stream_max_retries: Some(0),
            supports_websockets: false,
            ..ModelProviderInfo::create_openai_provider(Some(format!("{}/v1", server.uri())))
        };
        config.model_provider_id = "openai".into();
        let (mut session, turn_context) =
            crate::session::tests::make_session_and_context_for_config(config).await;
        session.services.state_db = Some(open_state_db(home.path()).await);
        Self {
            server,
            session,
            turn_context,
            home,
            cwd: test_cwd,
        }
    }

    pub(crate) async fn requests(&self) -> usize {
        self.server
            .received_requests()
            .await
            .unwrap_or_default()
            .len()
    }

    pub(crate) fn metadata(&self) -> crate::responses_metadata::CodexResponsesMetadata {
        crate::responses_metadata::CodexResponsesMetadata::new(
            "fixture".into(),
            "fixture".into(),
            self.session.thread_id.to_string(),
            "fixture:0".into(),
        )
    }
}

/// The first request reaches the provider and is refused with a 429; the
/// second, after it, is held back locally. Returns both errors.
pub(crate) fn assert_rate_limited_then_blocked(first: &CodexErr, second: &CodexErr) {
    assert_eq!(first.http_status_code_value(), Some(429), "{first}");
    assert!(is_provider_request_block(second), "{second}");
    assert!(second.to_string().contains("Last status: 429"), "{second}");
}

/// A vault read that hangs costs one `FINGERPRINT_BUDGET`, not one per
/// request: later requests go on at once, with an older fingerprint if one was
/// read, else by name. No second read starts while one runs.
#[tokio::test]
async fn a_hung_vault_read_falls_back_within_the_budget() {
    let home = tempfile::tempdir().expect("tempdir");
    let account = NamedProviderAccount {
        provider_id: "zai".to_string(),
        name: "hung".to_string(),
    };
    let key: FingerprintKey = (
        home.path().to_path_buf(),
        account.provider_id.clone(),
        account.name.clone(),
    );
    // A read that never finishes, as if the vault lock were held.
    let (_never_done, reading) = tokio::sync::watch::channel(());
    fingerprints().insert(
        key.clone(),
        FingerprintEntry {
            reading: Some((Instant::now(), reading)),
            ..Default::default()
        },
    );
    let started = Instant::now();
    assert_eq!(named_account_fingerprint(home.path(), &account).await, None);
    let waited = started.elapsed();
    assert!(waited >= FINGERPRINT_BUDGET, "{waited:?}");
    assert!(waited < FINGERPRINT_BUDGET * 2, "{waited:?}");

    // The next request does not wait again.
    let started = Instant::now();
    assert_eq!(named_account_fingerprint(home.path(), &account).await, None);
    assert!(started.elapsed() < FINGERPRINT_BUDGET / 2);

    // An older fingerprint is used at once meanwhile.
    let stale = Instant::now()
        .checked_sub(FINGERPRINT_FRESH_FOR * 2)
        .expect("an instant two minutes ago");
    fingerprints().get_mut(&key).expect("entry").read =
        Some((stale, Some("0123456789ab".to_string())));
    let started = Instant::now();
    assert_eq!(
        named_account_fingerprint(home.path(), &account).await,
        Some("0123456789ab".to_string())
    );
    assert!(started.elapsed() < FINGERPRINT_BUDGET / 2);
    assert!(fingerprints().get(&key).expect("entry").reading.is_some());
    fingerprints().remove(&key);
}

/// A read whose task ends without reading (it failed, or never ran) leaves
/// the fingerprint read before, and the account can be read again.
#[test]
fn a_failed_read_keeps_the_older_fingerprint() {
    let key: FingerprintKey = (
        PathBuf::from("/nonexistent/failed-read-home"),
        "zai".to_string(),
        "failed".to_string(),
    );
    let (done, _reading) = tokio::sync::watch::channel(());
    fingerprints().insert(
        key.clone(),
        FingerprintEntry {
            read: Some((Instant::now(), Some("0123456789ab".to_string()))),
            ..Default::default()
        },
    );
    drop(FinishRead {
        key: key.clone(),
        found: None,
        done,
    });
    let entry = fingerprints().remove(&key).expect("entry");
    assert!(entry.reading.is_none());
    assert_eq!(
        entry.read.and_then(|(_, fingerprint)| fingerprint),
        Some("0123456789ab".to_string())
    );
}
