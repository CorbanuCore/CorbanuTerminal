//! D319: vault files written by earlier Corbanu builds must keep opening.
//!
//! The fixtures under `tests/fixtures/d319_vault/` were generated once from
//! `main`'s code (age 0.11.2) before the upstream sync touched `age`, and must
//! never be regenerated (see the README there). Each test copies one fixture
//! into a fresh home, opens it through the public backend with the fixture key,
//! checks every decrypted entry against `expected.json`, and checks that files
//! above the current scrypt work factor come back re-encrypted at 16.

use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context;
use anyhow::Result;
use codex_keyring_store::CredentialStoreError;
use codex_keyring_store::KeyringStore;
use codex_secrets::LocalSecretsBackend;
use codex_secrets::LocalSecretsNamespace;
use codex_secrets::SecretName;
use codex_secrets::SecretScope;
use pretty_assertions::assert_eq;
use serde_json::Value;

/// Fixture-only key; it protects nothing but the test values below.
const FIXTURE_PASSPHRASE: &str = "corbanu-d319-vault-fixture-passphrase-not-a-secret";
const CURRENT_WORK_FACTOR: u8 = 16;

#[derive(Debug)]
struct FixtureKeyring {
    passphrase: &'static str,
}

impl KeyringStore for FixtureKeyring {
    fn load(&self, _service: &str, _account: &str) -> Result<Option<String>, CredentialStoreError> {
        Ok(Some(self.passphrase.to_string()))
    }

    fn save(
        &self,
        _service: &str,
        _account: &str,
        _value: &str,
    ) -> Result<(), CredentialStoreError> {
        Err(CredentialStoreError::from_message(
            "the fixture keyring is read-only",
        ))
    }

    fn delete(&self, _service: &str, _account: &str) -> Result<bool, CredentialStoreError> {
        Err(CredentialStoreError::from_message(
            "the fixture keyring is read-only",
        ))
    }
}

fn fixtures_dir() -> PathBuf {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/d319_vault");
    if manifest.is_dir() {
        manifest
    } else {
        // Bazel runs integration tests from a Cargo-like crate directory.
        PathBuf::from("tests/fixtures/d319_vault")
    }
}

fn namespace_for(file: &str) -> LocalSecretsNamespace {
    match file {
        "local.age" => LocalSecretsNamespace::ManagedSecrets,
        "codex_auth.age" => LocalSecretsNamespace::CodexAuth,
        "mcp_oauth.age" => LocalSecretsNamespace::McpOAuth,
        other => panic!("unknown fixture file {other}"),
    }
}

fn parse_canonical_key(key: &str) -> (SecretScope, SecretName) {
    let parts: Vec<&str> = key.split('/').collect();
    match parts.as_slice() {
        ["global", name] => (SecretScope::Global, SecretName::new(name).expect("name")),
        ["env", environment, name] => (
            SecretScope::Environment((*environment).to_string()),
            SecretName::new(name).expect("name"),
        ),
        _ => panic!("unexpected canonical key {key}"),
    }
}

fn scrypt_work_factor(ciphertext: &[u8]) -> Option<u8> {
    let header = String::from_utf8_lossy(&ciphertext[..ciphertext.len().min(512)]).into_owned();
    header
        .lines()
        .find_map(|line| line.strip_prefix("-> scrypt "))
        .and_then(|fields| fields.split_whitespace().nth(1))
        .and_then(|work_factor| work_factor.parse().ok())
}

fn backend(home: &Path, file: &str, passphrase: &'static str) -> LocalSecretsBackend {
    LocalSecretsBackend::new_with_namespace(
        home.to_path_buf(),
        Arc::new(FixtureKeyring { passphrase }),
        namespace_for(file),
    )
}

fn assert_contents(
    backend: &LocalSecretsBackend,
    expected: &serde_json::Map<String, Value>,
) -> Result<()> {
    for (key, value) in expected {
        let (scope, name) = parse_canonical_key(key);
        let expected_value = value.as_str().context("expected value is a string")?;
        assert_eq!(
            backend.get(&scope, &name)?.as_deref(),
            Some(expected_value),
            "entry {key}"
        );
    }
    let mut listed: Vec<String> = backend
        .list(None)?
        .into_iter()
        .map(|entry| entry.scope.canonical_key(&entry.name))
        .collect();
    listed.sort();
    let mut keys: Vec<String> = expected.keys().cloned().collect();
    keys.sort();
    assert_eq!(listed, keys);
    Ok(())
}

fn check_fixture(work_factor: u8, file: &str) -> Result<()> {
    let fixtures = fixtures_dir();
    let expected: Value = serde_json::from_slice(&std::fs::read(fixtures.join("expected.json"))?)?;
    let expected = expected[file]["secrets"]
        .as_object()
        .context("expected entries for fixture")?;
    let original = std::fs::read(fixtures.join(format!("wf{work_factor}/secrets/{file}")))?;
    assert_eq!(
        scrypt_work_factor(&original),
        Some(work_factor),
        "fixture header"
    );

    let home = tempfile::tempdir()?;
    let stored = home.path().join("secrets").join(file);
    std::fs::create_dir_all(stored.parent().context("parent")?)?;
    std::fs::write(&stored, &original)?;

    assert_contents(&backend(home.path(), file, FIXTURE_PASSPHRASE), expected)?;

    let after_open = std::fs::read(&stored)?;
    if work_factor > CURRENT_WORK_FACTOR {
        assert_eq!(
            scrypt_work_factor(&after_open),
            Some(CURRENT_WORK_FACTOR),
            "a file above work factor {CURRENT_WORK_FACTOR} is re-encrypted after a successful open"
        );
        // The re-encrypted file still holds the same entries.
        assert_contents(&backend(home.path(), file, FIXTURE_PASSPHRASE), expected)?;
    } else {
        assert_eq!(
            after_open, original,
            "an up-to-date file is not rewritten on open"
        );
    }
    Ok(())
}

macro_rules! fixture_tests {
    ($($test:ident => ($work_factor:expr, $file:expr),)*) => {
        $(
            #[test]
            fn $test() -> Result<()> {
                check_fixture($work_factor, $file)
            }
        )*
    };
}

fixture_tests! {
    local_wf16 => (16, "local.age"),
    local_wf18 => (18, "local.age"),
    local_wf20 => (20, "local.age"),
    codex_auth_wf16 => (16, "codex_auth.age"),
    codex_auth_wf18 => (18, "codex_auth.age"),
    codex_auth_wf20 => (20, "codex_auth.age"),
    mcp_oauth_wf16 => (16, "mcp_oauth.age"),
    mcp_oauth_wf18 => (18, "mcp_oauth.age"),
    mcp_oauth_wf20 => (20, "mcp_oauth.age"),
}

#[test]
fn wrong_key_fails_cleanly_and_leaves_the_file_unchanged() -> Result<()> {
    let file = "local.age";
    let original = std::fs::read(fixtures_dir().join(format!("wf16/secrets/{file}")))?;
    let home = tempfile::tempdir()?;
    let stored = home.path().join("secrets").join(file);
    std::fs::create_dir_all(stored.parent().context("parent")?)?;
    std::fs::write(&stored, &original)?;

    let backend = backend(home.path(), file, "not-the-fixture-passphrase");
    let error = backend
        .list(None)
        .expect_err("a wrong key must not open the vault");
    assert!(
        format!("{error:#}").contains("cannot be decrypted"),
        "unexpected error: {error:#}"
    );
    assert_eq!(std::fs::read(&stored)?, original);
    Ok(())
}
