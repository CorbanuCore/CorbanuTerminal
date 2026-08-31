//! Refresh-capable access to Claude Code subscription credentials.

use std::fs::File;
use std::fs::OpenOptions;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use codex_vault::ClaudeAuthHealth;
use codex_vault::ClaudeAuthResolution;
use codex_vault::ClaudeAuthSelection;
use codex_vault::ClaudeAuthSource;
use codex_vault::ClaudeAuthSourceMetadata;
use codex_vault::ClaudeAuthStoreKind;
use codex_vault::ENVIRONMENT_CLAUDE_AUTH_SOURCE_ID;
use codex_vault::MANAGED_CLAUDE_AUTH_SOURCE_ID;
use codex_vault::ManagedClaudeTokenStatus;
use codex_vault::Vault;
use codex_vault::claude_code_macos_keychain_service;
use codex_vault::credentials_file_claude_auth_source_id;
use codex_vault::macos_keychain_claude_auth_source_id;
use codex_vault::resolve_claude_auth_source;
use serde::Deserialize;
use tokio::process::Command;

const CLAUDE_CREDENTIALS_FILE: &str = ".credentials.json";
const CLAUDE_REFRESH_LOCK_FILE: &str = ".pfterminal-oauth-refresh.lock";
const CLAUDE_CUSTOM_OAUTH_REFRESH_LOCK_FILE: &str = ".pfterminal-custom-oauth-refresh.lock";
const MIN_TOKEN_VALIDITY_MS: u64 = 60_000;
const CLAUDE_REFRESH_TIMEOUT: Duration = Duration::from_secs(30);
#[cfg(all(target_os = "macos", not(test)))]
const CLAUDE_KEYCHAIN_TIMEOUT: Duration = Duration::from_secs(5);
#[cfg(all(target_os = "macos", test))]
const CLAUDE_KEYCHAIN_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Clone, Debug, Deserialize)]
struct ClaudeCodeCredentials {
    #[serde(rename = "claudeAiOauth")]
    claude_ai_oauth: Option<ClaudeCodeOauthCredentials>,
}

#[derive(Clone, Debug, Deserialize)]
struct ClaudeCodeOauthCredentials {
    #[serde(rename = "accessToken")]
    access_token: Option<String>,
    #[serde(rename = "refreshToken")]
    refresh_token: Option<String>,
    #[serde(rename = "expiresAt")]
    expires_at: Option<u64>,
    #[serde(default)]
    scopes: Vec<String>,
}

#[allow(dead_code)] // The other variant is exercised by cross-platform fixtures.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PlatformCredentialStore {
    MacosKeychain,
    CredentialsFile,
}

impl PlatformCredentialStore {
    fn source_id(self, config_dir: &Path) -> Result<String> {
        match self {
            Self::MacosKeychain => Ok(macos_keychain_claude_auth_source_id(
                &claude_keychain_service(config_dir),
            )),
            Self::CredentialsFile => credentials_file_claude_auth_source_id(config_dir)
                .context("failed to identify Claude Code's credentials-file profile"),
        }
    }

    fn metadata_store(self) -> ClaudeAuthStoreKind {
        match self {
            Self::MacosKeychain => ClaudeAuthStoreKind::MacosKeychain,
            Self::CredentialsFile => ClaudeAuthStoreKind::CredentialsFile,
        }
    }
}

#[cfg(target_os = "macos")]
const CURRENT_PLATFORM_STORE: PlatformCredentialStore = PlatformCredentialStore::MacosKeychain;
#[cfg(not(target_os = "macos"))]
const CURRENT_PLATFORM_STORE: PlatformCredentialStore = PlatformCredentialStore::CredentialsFile;

pub(crate) async fn resolve_claude_oauth_access_token(codex_home: &Path) -> Result<String> {
    let vault = Vault::new(codex_home.to_path_buf());
    let selection = vault
        .load_claude_auth_selection()
        .context("failed to load the selected Claude authentication method")?;
    resolve_claude_oauth_access_token_for_selection(selection.as_ref(), &vault).await
}

async fn resolve_claude_oauth_access_token_for_selection(
    selection: Option<&ClaudeAuthSelection>,
    vault: &Vault,
) -> Result<String> {
    if let Some(selection) = selection {
        let discovered = discover_selected_claude_auth_source(selection.source, vault).await?;
        let metadata = resolve_selected_claude_auth_source(selection, &discovered)?;
        return match metadata.source {
            ClaudeAuthSource::ManagedSubscriptionToken => {
                vault
                    .with_managed_claude_subscription_token(ToString::to_string)
                    .context(
                        "the selected managed Claude subscription token is unavailable; run Claude authentication setup again",
                    )
            }
            ClaudeAuthSource::EnvironmentToken => {
                nonempty_env("CLAUDE_CODE_OAUTH_TOKEN").ok_or_else(|| {
                    anyhow!(
                        "the selected CLAUDE_CODE_OAUTH_TOKEN source is missing or blank; set it again or explicitly choose another Claude authentication method"
                    )
                })
            }
            ClaudeAuthSource::ClaudeCodeLogin => {
                resolve_current_platform_claude_oauth_access_token().await
            }
        };
    }

    // Existing installations have no persisted selection. Preserve their exact
    // historical env-first behavior until the user confirms a method.
    if let Some(token) = nonempty_env("CLAUDE_CODE_OAUTH_TOKEN") {
        return Ok(token);
    }

    resolve_current_platform_claude_oauth_access_token().await
}

fn resolve_selected_claude_auth_source(
    selection: &ClaudeAuthSelection,
    discovered: &[ClaudeAuthSourceMetadata],
) -> Result<ClaudeAuthSourceMetadata> {
    match resolve_claude_auth_source(Some(selection), discovered) {
        ClaudeAuthResolution::Selected(metadata) => Ok(metadata),
        ClaudeAuthResolution::UnhealthySelected(metadata) => {
            Err(unhealthy_selected_source_error(&metadata))
        }
        ClaudeAuthResolution::MissingSelected(_) => Err(anyhow!(
            "the selected Claude authentication source no longer matches the current platform or profile; explicitly choose the intended method again"
        )),
        ClaudeAuthResolution::Conflict { .. } => Err(anyhow!(
            "the selected Claude authentication source is ambiguous; explicitly choose one current source again"
        )),
        ClaudeAuthResolution::SelectionRequired { .. } => {
            Err(anyhow!("Claude authentication selection is required"))
        }
    }
}

async fn discover_selected_claude_auth_source(
    source: ClaudeAuthSource,
    vault: &Vault,
) -> Result<Vec<ClaudeAuthSourceMetadata>> {
    let metadata = match source {
        ClaudeAuthSource::ManagedSubscriptionToken => ClaudeAuthSourceMetadata {
            source: ClaudeAuthSource::ManagedSubscriptionToken,
            source_id: MANAGED_CLAUDE_AUTH_SOURCE_ID.to_string(),
            store: ClaudeAuthStoreKind::CorbanuVault,
            health: match vault.managed_claude_subscription_token_status() {
                Ok(ManagedClaudeTokenStatus::Stored { .. }) => ClaudeAuthHealth::Healthy,
                Ok(ManagedClaudeTokenStatus::Missing) => ClaudeAuthHealth::NeedsEnrollment,
                Err(_) => ClaudeAuthHealth::Unavailable,
            },
            account_hint: None,
        },
        ClaudeAuthSource::EnvironmentToken => ClaudeAuthSourceMetadata {
            source: ClaudeAuthSource::EnvironmentToken,
            source_id: ENVIRONMENT_CLAUDE_AUTH_SOURCE_ID.to_string(),
            store: ClaudeAuthStoreKind::Environment,
            health: if nonempty_env("CLAUDE_CODE_OAUTH_TOKEN").is_some() {
                ClaudeAuthHealth::Healthy
            } else {
                ClaudeAuthHealth::Missing
            },
            account_hint: None,
        },
        ClaudeAuthSource::ClaudeCodeLogin => {
            let config_dir = claude_config_dir()?;
            let credentials_path = config_dir.join(CLAUDE_CREDENTIALS_FILE);
            let health = match read_credentials(
                CURRENT_PLATFORM_STORE,
                &config_dir,
                &credentials_path,
                /*security*/ None,
            )
            .await
            {
                Ok(credentials) => credentials_health(&credentials),
                Err(error) => credentials_error_health(&error),
            };
            ClaudeAuthSourceMetadata {
                source: ClaudeAuthSource::ClaudeCodeLogin,
                source_id: CURRENT_PLATFORM_STORE.source_id(&config_dir)?,
                store: CURRENT_PLATFORM_STORE.metadata_store(),
                health,
                account_hint: None,
            }
        }
    };
    Ok(vec![metadata])
}

fn credentials_error_health(error: &anyhow::Error) -> ClaudeAuthHealth {
    if error.downcast_ref::<serde_json::Error>().is_some() {
        ClaudeAuthHealth::Malformed
    } else if error
        .downcast_ref::<io::Error>()
        .is_some_and(|error| error.kind() == io::ErrorKind::NotFound)
    {
        ClaudeAuthHealth::Missing
    } else {
        ClaudeAuthHealth::Unavailable
    }
}

fn unhealthy_selected_source_error(metadata: &ClaudeAuthSourceMetadata) -> anyhow::Error {
    match metadata.health {
        ClaudeAuthHealth::NeedsEnrollment => anyhow!(
            "the selected managed Claude subscription token is missing; run Claude authentication setup again"
        ),
        ClaudeAuthHealth::NeedsReauthorization => anyhow!(
            "the selected Claude Code login needs reauthorization; run `claude auth login` again or explicitly choose another method"
        ),
        ClaudeAuthHealth::Missing => anyhow!(
            "the selected Claude authentication source is missing; restore it or explicitly choose another method"
        ),
        ClaudeAuthHealth::Malformed => anyhow!(
            "the selected Claude Code credential record is malformed; run `claude auth login` again"
        ),
        ClaudeAuthHealth::Unavailable => anyhow!(
            "the selected Claude authentication source is unavailable; restore access or explicitly choose another method"
        ),
        ClaudeAuthHealth::Healthy => {
            anyhow!("the selected Claude authentication source is healthy")
        }
    }
}

async fn resolve_current_platform_claude_oauth_access_token() -> Result<String> {
    let config_dir = claude_config_dir()?;

    let force_refresh =
        nonempty_env("PFTERMINAL_PROVIDER_AUTH_FORCE_REFRESH").as_deref() == Some("1");
    resolve_stored_claude_oauth_access_token(
        &config_dir,
        /*claude_executable*/ None,
        current_time_ms(),
        force_refresh,
        CURRENT_PLATFORM_STORE,
        /*security*/ None,
    )
    .await
}

/// Validate the exact platform credential record used by provider resolution without
/// returning or printing credential material. The TUI calls this before persisting a
/// compatibility selection so a superficial CLI status cannot commit an unhealthy source.
pub(crate) async fn verify_current_platform_claude_login_health() -> Result<()> {
    let config_dir = claude_config_dir()?;
    let security = claude_test_security_executable();
    verify_claude_login_health(&config_dir, CURRENT_PLATFORM_STORE, security.as_deref()).await
}

#[cfg(all(target_os = "macos", debug_assertions))]
fn claude_test_security_executable() -> Option<PathBuf> {
    std::env::var_os("CORBANU_TEST_CLAUDE_SECURITY_EXECUTABLE")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

#[cfg(not(all(target_os = "macos", debug_assertions)))]
fn claude_test_security_executable() -> Option<PathBuf> {
    None
}

async fn verify_claude_login_health(
    config_dir: &Path,
    store: PlatformCredentialStore,
    security: Option<&Path>,
) -> Result<()> {
    let credentials_path = config_dir.join(CLAUDE_CREDENTIALS_FILE);
    let health = match read_credentials(store, config_dir, &credentials_path, security).await {
        Ok(credentials) => credentials_health(&credentials),
        Err(error) => credentials_error_health(&error),
    };
    let metadata = ClaudeAuthSourceMetadata {
        source: ClaudeAuthSource::ClaudeCodeLogin,
        source_id: store.source_id(config_dir)?,
        store: store.metadata_store(),
        health,
        account_hint: None,
    };
    if health == ClaudeAuthHealth::Healthy {
        Ok(())
    } else {
        Err(unhealthy_selected_source_error(&metadata))
    }
}

fn claude_config_dir() -> Result<PathBuf> {
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("HOME is not set; cannot read Claude Code credentials"))?;
    Ok(std::env::var_os("CLAUDE_CONFIG_DIR")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join(".claude")))
}

async fn resolve_stored_claude_oauth_access_token(
    config_dir: &Path,
    claude_executable: Option<&Path>,
    now_ms: u64,
    force_refresh: bool,
    store: PlatformCredentialStore,
    security: Option<&Path>,
) -> Result<String> {
    let credentials_path = config_dir.join(CLAUDE_CREDENTIALS_FILE);
    let credentials = read_credentials(store, config_dir, &credentials_path, security).await?;
    if !force_refresh && let Some(access_token) = usable_access_token(&credentials, now_ms) {
        return Ok(access_token);
    }
    let original_access_token = credentials
        .claude_ai_oauth
        .as_ref()
        .and_then(|oauth| oauth.access_token.clone());

    let _refresh_lock = acquire_refresh_lock(config_dir, store).await?;

    // Another PFTerminal process may have refreshed while this process waited.
    let credentials = read_credentials(store, config_dir, &credentials_path, security).await?;
    if let Some(access_token) = usable_access_token(&credentials, current_time_ms().max(now_ms))
        && (!force_refresh || Some(access_token.clone()) != original_access_token)
    {
        return Ok(access_token);
    }

    let refresh_status =
        refresh_with_claude_cli(config_dir, &credentials, claude_executable).await?;

    let refreshed = read_credentials(store, config_dir, &credentials_path, security).await?;
    if let Some(access_token) = usable_access_token(&refreshed, current_time_ms().max(now_ms))
        && Some(access_token.clone()) != original_access_token
    {
        return Ok(access_token);
    }
    if !refresh_status.success() {
        return Err(anyhow!(
            "Claude Code OAuth refresh failed with status {refresh_status}. Run `claude /login` again."
        ));
    }
    Err(anyhow!(
        "Claude Code completed OAuth refresh but did not persist a new usable access token. Run `claude /login` again."
    ))
}

async fn read_credentials(
    store: PlatformCredentialStore,
    config_dir: &Path,
    path: &Path,
    security: Option<&Path>,
) -> Result<ClaudeCodeCredentials> {
    let (contents, source) = match store {
        PlatformCredentialStore::CredentialsFile => {
            let contents = tokio::fs::read(path).await.with_context(|| {
                format!(
                    "Claude Code credentials were not available at {}. Run `claude auth login` with your Claude subscription.",
                    path.display()
                )
            })?;
            (contents, path.display().to_string())
        }
        PlatformCredentialStore::MacosKeychain => {
            #[cfg(target_os = "macos")]
            {
                let contents = read_macos_keychain_credentials(config_dir, security).await?;
                (contents, "the macOS Keychain".to_string())
            }
            #[cfg(not(target_os = "macos"))]
            {
                let _ = (config_dir, security);
                return Err(anyhow!(
                    "macOS Keychain Claude credentials are unavailable on this platform"
                ));
            }
        }
    };
    serde_json::from_slice(&contents).with_context(|| {
        format!(
            "Claude Code credentials for {source} are not valid JSON. Run `claude auth login` again."
        )
    })
}

#[cfg(target_os = "macos")]
async fn read_macos_keychain_credentials(
    config_dir: &Path,
    security: Option<&Path>,
) -> Result<Vec<u8>> {
    let account = std::env::var("USER")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(whoami::username);
    let service = claude_keychain_service(config_dir);
    let executable = security.unwrap_or_else(|| Path::new("/usr/bin/security"));
    let output = tokio::time::timeout(
        CLAUDE_KEYCHAIN_TIMEOUT,
        Command::new(executable)
            .args([
                "find-generic-password",
                "-a",
                account.as_str(),
                "-w",
                "-s",
                service.as_str(),
            ])
            .stdin(Stdio::null())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .output(),
    )
    .await
    .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "macOS Keychain lookup timed out"))?
    .with_context(|| format!("failed to start `{}`", executable.display()))?;
    if !output.status.success() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "macOS Keychain did not contain current Claude Code credentials",
        )
        .into());
    }
    Ok(output.stdout)
}

fn claude_keychain_service(config_dir: &Path) -> String {
    claude_code_macos_keychain_service(
        config_dir,
        std::env::var_os("CLAUDE_CONFIG_DIR")
            .filter(|value| !value.is_empty())
            .is_some(),
        nonempty_env("CLAUDE_CODE_CUSTOM_OAUTH_URL").is_some(),
    )
}

fn usable_access_token(credentials: &ClaudeCodeCredentials, now_ms: u64) -> Option<String> {
    let oauth = credentials.claude_ai_oauth.as_ref()?;
    let expires_at = oauth.expires_at?;
    if expires_at <= now_ms.saturating_add(MIN_TOKEN_VALIDITY_MS) {
        return None;
    }
    oauth
        .access_token
        .as_deref()
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .map(ToString::to_string)
}

fn credentials_health(credentials: &ClaudeCodeCredentials) -> ClaudeAuthHealth {
    let Some(oauth) = credentials.claude_ai_oauth.as_ref() else {
        return ClaudeAuthHealth::NeedsReauthorization;
    };
    let has_refresh = oauth
        .refresh_token
        .as_deref()
        .map(str::trim)
        .is_some_and(|token| !token.is_empty());
    if has_refresh && !oauth.scopes.is_empty() {
        ClaudeAuthHealth::Healthy
    } else {
        ClaudeAuthHealth::NeedsReauthorization
    }
}

async fn refresh_with_claude_cli(
    config_dir: &Path,
    credentials: &ClaudeCodeCredentials,
    claude_executable: Option<&Path>,
) -> Result<std::process::ExitStatus> {
    let oauth = credentials.claude_ai_oauth.as_ref().ok_or_else(|| {
        anyhow!(
            "Claude Code OAuth credentials are missing. Run `claude /login` and choose a Claude subscription account."
        )
    })?;
    let refresh_token = oauth
        .refresh_token
        .as_deref()
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .ok_or_else(|| {
            anyhow!("Claude Code OAuth refresh token is missing. Run `claude /login` again.")
        })?;
    if oauth.scopes.is_empty() {
        return Err(anyhow!(
            "Claude Code OAuth scopes are missing. Run `claude /login` again."
        ));
    }

    let executable = match claude_executable {
        Some(path) => path.to_path_buf(),
        None => which::which("claude")
            .context("Claude Code executable was not found on PATH; cannot refresh OAuth token")?,
    };
    let mut command = Command::new(&executable);
    command.args(["auth", "login"]);
    if claude_config_dir_is_default(config_dir) {
        // Claude Code's default profile has no config hash in its Keychain
        // service. Avoid creating a different service for the same directory.
        command.env_remove("CLAUDE_CONFIG_DIR");
    } else {
        command.env("CLAUDE_CONFIG_DIR", config_dir);
    }
    command
        .env("CLAUDE_CODE_OAUTH_REFRESH_TOKEN", refresh_token)
        .env("CLAUDE_CODE_OAUTH_SCOPES", oauth.scopes.join(" "))
        .env_remove("CLAUDE_CODE_OAUTH_TOKEN")
        .env_remove("ANTHROPIC_API_KEY")
        .env_remove("ANTHROPIC_AUTH_TOKEN")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(true);

    tokio::time::timeout(CLAUDE_REFRESH_TIMEOUT, command.status())
        .await
        .map_err(|_| anyhow!("Claude Code OAuth refresh timed out after 30 seconds"))?
        .with_context(|| {
            format!(
                "failed to start `{}` for OAuth refresh",
                executable.display()
            )
        })
}

fn claude_config_dir_is_default(config_dir: &Path) -> bool {
    if std::env::var_os("CLAUDE_CONFIG_DIR")
        .filter(|value| !value.is_empty())
        .is_some()
    {
        return false;
    }
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .is_some_and(|home| config_dir == home.join(".claude"))
}

async fn acquire_refresh_lock(config_dir: &Path, store: PlatformCredentialStore) -> Result<File> {
    let lock_path = refresh_lock_path(config_dir, store)?;
    let lock_dir = lock_path
        .parent()
        .ok_or_else(|| anyhow!("Claude OAuth refresh lock has no parent directory"))?;
    tokio::fs::create_dir_all(lock_dir).await.with_context(|| {
        format!(
            "failed to create Claude Code config directory {}",
            lock_dir.display()
        )
    })?;
    tokio::task::spawn_blocking(move || -> io::Result<File> {
        let lock_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(lock_path)?;
        lock_file.lock()?;
        Ok(lock_file)
    })
    .await
    .context("Claude OAuth refresh lock task failed")?
    .context("failed to acquire Claude OAuth refresh lock")
}

fn refresh_lock_path(config_dir: &Path, store: PlatformCredentialStore) -> Result<PathBuf> {
    Ok(refresh_lock_path_for_store(
        config_dir,
        store,
        nonempty_env("CLAUDE_CODE_CUSTOM_OAUTH_URL").is_some(),
    ))
}

fn refresh_lock_path_for_store(
    config_dir: &Path,
    store: PlatformCredentialStore,
    custom_oauth: bool,
) -> PathBuf {
    match store {
        PlatformCredentialStore::CredentialsFile => config_dir.join(CLAUDE_REFRESH_LOCK_FILE),
        PlatformCredentialStore::MacosKeychain => config_dir.join(if custom_oauth {
            CLAUDE_CUSTOM_OAUTH_REFRESH_LOCK_FILE
        } else {
            CLAUDE_REFRESH_LOCK_FILE
        }),
    }
}

fn nonempty_env(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn current_time_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

    fn source_metadata(
        source: ClaudeAuthSource,
        source_id: &str,
        health: ClaudeAuthHealth,
    ) -> ClaudeAuthSourceMetadata {
        ClaudeAuthSourceMetadata {
            source,
            source_id: source_id.to_string(),
            store: ClaudeAuthStoreKind::Environment,
            health,
            account_hint: None,
        }
    }

    #[test]
    fn production_selection_path_uses_exact_typed_resolution() {
        let selection = ClaudeAuthSelection::new_at(
            ClaudeAuthSource::EnvironmentToken,
            ENVIRONMENT_CLAUDE_AUTH_SOURCE_ID,
            10,
        )
        .expect("selection");
        let selected = source_metadata(
            ClaudeAuthSource::EnvironmentToken,
            ENVIRONMENT_CLAUDE_AUTH_SOURCE_ID,
            ClaudeAuthHealth::Healthy,
        );
        assert_eq!(
            resolve_selected_claude_auth_source(&selection, std::slice::from_ref(&selected))
                .expect("exact healthy source"),
            selected
        );

        let drifted = source_metadata(
            ClaudeAuthSource::EnvironmentToken,
            "environment:different-account-slot",
            ClaudeAuthHealth::Healthy,
        );
        let error = resolve_selected_claude_auth_source(&selection, &[drifted])
            .expect_err("profile drift must fail closed");
        assert!(error.to_string().contains("no longer matches"));

        let error = resolve_selected_claude_auth_source(&selection, &[selected.clone(), selected])
            .expect_err("duplicate identities must fail as a conflict");
        assert!(error.to_string().contains("ambiguous"));
    }

    #[test]
    fn credentials_file_profile_change_fails_closed_in_production_resolution() {
        let work_profile = tempfile::tempdir().expect("work profile");
        let personal_profile = tempfile::tempdir().expect("personal profile");
        let selected_source_id = PlatformCredentialStore::CredentialsFile
            .source_id(work_profile.path())
            .expect("work source id");
        let selection =
            ClaudeAuthSelection::new_at(ClaudeAuthSource::ClaudeCodeLogin, selected_source_id, 10)
                .expect("selection");
        let discovered = ClaudeAuthSourceMetadata {
            source: ClaudeAuthSource::ClaudeCodeLogin,
            source_id: PlatformCredentialStore::CredentialsFile
                .source_id(personal_profile.path())
                .expect("personal source id"),
            store: ClaudeAuthStoreKind::CredentialsFile,
            health: ClaudeAuthHealth::Healthy,
            account_hint: None,
        };

        let error = resolve_selected_claude_auth_source(&selection, &[discovered])
            .expect_err("changing credentials-file profiles must fail closed");
        assert!(error.to_string().contains("no longer matches"));
    }

    #[tokio::test]
    async fn metadata_only_login_health_rejects_blank_refresh_credentials() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let now_ms = current_time_ms();
        write_credentials(temp_dir.path(), "live-access", "", now_ms + 120_000);

        let error = verify_claude_login_health(
            temp_dir.path(),
            PlatformCredentialStore::CredentialsFile,
            /*security*/ None,
        )
        .await
        .expect_err("blank refresh token must not be selected");

        assert!(error.to_string().contains("needs reauthorization"));
    }

    #[tokio::test]
    async fn valid_access_token_does_not_require_claude_cli() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let now_ms = current_time_ms();
        write_credentials(
            temp_dir.path(),
            "valid-access",
            "valid-refresh",
            now_ms + 120_000,
        );

        let token = resolve_stored_claude_oauth_access_token(
            temp_dir.path(),
            Some(Path::new("/does/not/exist")),
            now_ms,
            /*force_refresh*/ false,
            PlatformCredentialStore::CredentialsFile,
            /*security*/ None,
        )
        .await
        .expect("valid token should be returned without starting Claude");

        assert_eq!(token, "valid-access");
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn expiring_access_token_is_refreshed_by_claude_cli() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let now_ms = current_time_ms();
        write_credentials(
            temp_dir.path(),
            "expiring-access",
            "rotating-refresh",
            now_ms + 30_000,
        );
        let claude = fake_refreshing_claude(
            temp_dir.path(),
            now_ms + 600_000,
            /*sleep_seconds*/ 0,
            /*exit_code*/ 0,
        );

        let token = resolve_stored_claude_oauth_access_token(
            temp_dir.path(),
            Some(&claude),
            now_ms,
            /*force_refresh*/ false,
            PlatformCredentialStore::CredentialsFile,
            /*security*/ None,
        )
        .await
        .expect("Claude CLI should refresh expiring credentials");

        assert_eq!(token, "refreshed-access");
        let refresh_log =
            std::fs::read_to_string(temp_dir.path().join("refresh.log")).expect("refresh log");
        assert_eq!(
            refresh_log,
            "rotating-refresh|user:profile user:inference\n"
        );
        let persisted = read_credentials(
            PlatformCredentialStore::CredentialsFile,
            temp_dir.path(),
            &temp_dir.path().join(CLAUDE_CREDENTIALS_FILE),
            /*security*/ None,
        )
        .await
        .expect("persisted credentials");
        assert_eq!(
            persisted
                .claude_ai_oauth
                .and_then(|oauth| oauth.refresh_token),
            Some("refreshed-refresh".to_string())
        );
    }

    #[cfg(unix)]
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn concurrent_refreshes_share_one_claude_exchange() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let now_ms = current_time_ms();
        write_credentials(
            temp_dir.path(),
            "expired-access",
            "single-use-refresh",
            now_ms.saturating_sub(1),
        );
        let claude = fake_refreshing_claude(
            temp_dir.path(),
            now_ms + 600_000,
            /*sleep_seconds*/ 1,
            /*exit_code*/ 0,
        );
        let config_a = temp_dir.path().to_path_buf();
        let config_b = config_a.clone();
        let claude_a = claude.clone();
        let claude_b = claude.clone();

        let (first, second) = tokio::join!(
            resolve_stored_claude_oauth_access_token(
                &config_a,
                Some(&claude_a),
                now_ms,
                /*force_refresh*/ false,
                PlatformCredentialStore::CredentialsFile,
                /*security*/ None,
            ),
            resolve_stored_claude_oauth_access_token(
                &config_b,
                Some(&claude_b),
                now_ms,
                /*force_refresh*/ false,
                PlatformCredentialStore::CredentialsFile,
                /*security*/ None,
            ),
        );

        assert_eq!(first.expect("first refresh"), "refreshed-access");
        assert_eq!(second.expect("second refresh"), "refreshed-access");
        let refresh_log =
            std::fs::read_to_string(temp_dir.path().join("refresh.log")).expect("refresh log");
        assert_eq!(refresh_log.lines().count(), 1);
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn forced_refresh_replaces_an_unexpired_rejected_token() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let now_ms = current_time_ms();
        write_credentials(
            temp_dir.path(),
            "rejected-but-unexpired-access",
            "refresh-after-401",
            now_ms + 600_000,
        );
        let claude = fake_refreshing_claude(
            temp_dir.path(),
            now_ms + 900_000,
            /*sleep_seconds*/ 0,
            /*exit_code*/ 0,
        );

        let token = resolve_stored_claude_oauth_access_token(
            temp_dir.path(),
            Some(&claude),
            now_ms,
            /*force_refresh*/ true,
            PlatformCredentialStore::CredentialsFile,
            /*security*/ None,
        )
        .await
        .expect("401 recovery should force Claude credential rotation");

        assert_eq!(token, "refreshed-access");
        assert_eq!(
            std::fs::read_to_string(temp_dir.path().join("refresh.log"))
                .expect("refresh log")
                .lines()
                .count(),
            1
        );
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn rotated_credentials_win_over_nonzero_claude_exit() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let now_ms = current_time_ms();
        write_credentials(
            temp_dir.path(),
            "expired-access",
            "refresh-before-nonzero-exit",
            now_ms.saturating_sub(1),
        );
        let claude = fake_refreshing_claude(
            temp_dir.path(),
            now_ms + 600_000,
            /*sleep_seconds*/ 0,
            /*exit_code*/ 1,
        );

        let token = resolve_stored_claude_oauth_access_token(
            temp_dir.path(),
            Some(&claude),
            now_ms,
            /*force_refresh*/ false,
            PlatformCredentialStore::CredentialsFile,
            /*security*/ None,
        )
        .await
        .expect("persisted credential rotation is the refresh authority");

        assert_eq!(token, "refreshed-access");
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn failed_refresh_preserves_credentials_and_redacts_refresh_token() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let now_ms = current_time_ms();
        write_credentials(
            temp_dir.path(),
            "expired-access",
            "never-print-this-refresh-token",
            now_ms.saturating_sub(1),
        );
        let credentials_path = temp_dir.path().join(CLAUDE_CREDENTIALS_FILE);
        let before = std::fs::read(&credentials_path).expect("credentials before refresh");
        let claude = fake_failing_claude(temp_dir.path());

        let error = resolve_stored_claude_oauth_access_token(
            temp_dir.path(),
            Some(&claude),
            now_ms,
            /*force_refresh*/ false,
            PlatformCredentialStore::CredentialsFile,
            /*security*/ None,
        )
        .await
        .expect_err("failed Claude exchange should surface");

        assert!(error.to_string().contains("OAuth refresh failed"));
        assert!(!error.to_string().contains("never-print-this-refresh-token"));
        assert_eq!(
            std::fs::read(credentials_path).expect("credentials after refresh"),
            before
        );
    }

    #[tokio::test]
    async fn blank_refresh_token_requires_reauthorization_without_starting_claude() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let now_ms = current_time_ms();
        write_credentials(
            temp_dir.path(),
            "expired-access",
            "",
            now_ms.saturating_sub(1),
        );

        let error = resolve_stored_claude_oauth_access_token(
            temp_dir.path(),
            Some(Path::new("/does/not/exist")),
            now_ms,
            /*force_refresh*/ false,
            PlatformCredentialStore::CredentialsFile,
            /*security*/ None,
        )
        .await
        .expect_err("blank rotating refresh token must require login");

        assert!(error.to_string().contains("refresh token is missing"));
        assert!(!error.to_string().contains("expired-access"));
    }

    #[test]
    fn platform_fixture_health_classifies_stale_live_and_blank_refresh_records() {
        let now_ms = 1_000_000;
        let refreshable: ClaudeCodeCredentials = serde_json::from_value(json!({
            "claudeAiOauth": {
                "accessToken": "stale-access",
                "refreshToken": "rotating-refresh",
                "expiresAt": now_ms - 1,
                "scopes": ["user:inference"]
            }
        }))
        .expect("refreshable fixture");
        let blank_refresh: ClaudeCodeCredentials = serde_json::from_value(json!({
            "claudeAiOauth": {
                "accessToken": "stale-access",
                "refreshToken": "  ",
                "expiresAt": now_ms - 1,
                "scopes": ["user:inference"]
            }
        }))
        .expect("blank refresh fixture");
        let live_access_blank_refresh: ClaudeCodeCredentials = serde_json::from_value(json!({
            "claudeAiOauth": {
                "accessToken": "live-access",
                "refreshToken": "  ",
                "expiresAt": now_ms + MIN_TOKEN_VALIDITY_MS + 1,
                "scopes": ["user:inference"]
            }
        }))
        .expect("live access with blank refresh fixture");

        assert_eq!(credentials_health(&refreshable), ClaudeAuthHealth::Healthy);
        assert_eq!(
            credentials_health(&blank_refresh),
            ClaudeAuthHealth::NeedsReauthorization
        );
        assert_eq!(
            credentials_health(&live_access_blank_refresh),
            ClaudeAuthHealth::NeedsReauthorization
        );
    }

    #[tokio::test]
    async fn credentials_file_fixture_distinguishes_missing_and_malformed_records() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let path = temp_dir.path().join(CLAUDE_CREDENTIALS_FILE);
        let missing = read_credentials(
            PlatformCredentialStore::CredentialsFile,
            temp_dir.path(),
            &path,
            /*security*/ None,
        )
        .await
        .expect_err("missing fixture");
        assert_eq!(
            credentials_error_health(&missing),
            ClaudeAuthHealth::Missing
        );

        std::fs::write(&path, b"not-json").expect("malformed fixture");
        let malformed = read_credentials(
            PlatformCredentialStore::CredentialsFile,
            temp_dir.path(),
            &path,
            /*security*/ None,
        )
        .await
        .expect_err("malformed fixture");
        assert_eq!(
            credentials_error_health(&malformed),
            ClaudeAuthHealth::Malformed
        );
    }

    #[test]
    fn refresh_lock_follows_the_authoritative_store_identity() {
        let config_a = Path::new("/fixture/config-a");
        let config_b = Path::new("/fixture/config-b");

        assert_ne!(
            refresh_lock_path_for_store(config_a, PlatformCredentialStore::CredentialsFile, false),
            refresh_lock_path_for_store(config_b, PlatformCredentialStore::CredentialsFile, false)
        );
        let keychain_lock =
            refresh_lock_path_for_store(config_a, PlatformCredentialStore::MacosKeychain, false);
        assert_ne!(
            keychain_lock,
            refresh_lock_path_for_store(config_b, PlatformCredentialStore::MacosKeychain, false,)
        );
        assert_ne!(
            keychain_lock,
            refresh_lock_path_for_store(config_a, PlatformCredentialStore::MacosKeychain, true,)
        );
    }

    #[cfg(target_os = "macos")]
    #[tokio::test]
    async fn current_claude_keychain_credentials_are_supported() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let now_ms = current_time_ms();
        let security = fake_keychain_security(temp_dir.path(), now_ms + 600_000);
        write_credentials(
            temp_dir.path(),
            "legacy-file-access",
            "legacy-file-refresh",
            now_ms + 600_000,
        );

        let credentials = read_credentials(
            PlatformCredentialStore::MacosKeychain,
            temp_dir.path(),
            &temp_dir.path().join(CLAUDE_CREDENTIALS_FILE),
            Some(&security),
        )
        .await
        .expect("keychain credentials");

        assert_eq!(
            usable_access_token(&credentials, now_ms).as_deref(),
            Some("keychain-access")
        );
    }

    fn write_credentials(
        config_dir: &Path,
        access_token: &str,
        refresh_token: &str,
        expires_at: u64,
    ) {
        let credentials = json!({
            "claudeAiOauth": {
                "accessToken": access_token,
                "refreshToken": refresh_token,
                "expiresAt": expires_at,
                "scopes": ["user:profile", "user:inference"]
            }
        });
        std::fs::write(
            config_dir.join(CLAUDE_CREDENTIALS_FILE),
            serde_json::to_vec(&credentials).expect("serialize credentials"),
        )
        .expect("write credentials");
    }

    #[cfg(unix)]
    fn fake_refreshing_claude(
        config_dir: &Path,
        expires_at: u64,
        sleep_seconds: u64,
        exit_code: i32,
    ) -> PathBuf {
        let executable = config_dir.join("fake-claude");
        let script = format!(
            r#"#!/bin/sh
set -eu
[ "$1 $2" = "auth login" ]
printf '%s|%s\n' "$CLAUDE_CODE_OAUTH_REFRESH_TOKEN" "$CLAUDE_CODE_OAUTH_SCOPES" >> "$CLAUDE_CONFIG_DIR/refresh.log"
sleep {sleep_seconds}
printf '%s\n' '{{"claudeAiOauth":{{"accessToken":"refreshed-access","refreshToken":"refreshed-refresh","expiresAt":{expires_at},"scopes":["user:profile","user:inference"]}}}}' > "$CLAUDE_CONFIG_DIR/{CLAUDE_CREDENTIALS_FILE}"
exit {exit_code}
"#
        );
        std::fs::write(&executable, script).expect("write fake Claude");
        let mut permissions = std::fs::metadata(&executable)
            .expect("fake Claude metadata")
            .permissions();
        permissions.set_mode(0o700);
        std::fs::set_permissions(&executable, permissions).expect("make fake Claude executable");
        executable
    }

    #[cfg(unix)]
    fn fake_failing_claude(config_dir: &Path) -> PathBuf {
        let executable = config_dir.join("failing-claude");
        std::fs::write(&executable, "#!/bin/sh\nexit 7\n").expect("write failing Claude");
        let mut permissions = std::fs::metadata(&executable)
            .expect("failing Claude metadata")
            .permissions();
        permissions.set_mode(0o700);
        std::fs::set_permissions(&executable, permissions).expect("make fake Claude executable");
        executable
    }

    #[cfg(target_os = "macos")]
    fn fake_keychain_security(config_dir: &Path, expires_at: u64) -> PathBuf {
        let executable = config_dir.join("fake-security");
        let script = format!(
            r#"#!/bin/sh
set -eu
[ "$1" = "find-generic-password" ]
printf '%s\n' '{{"claudeAiOauth":{{"accessToken":"keychain-access","refreshToken":"keychain-refresh","expiresAt":{expires_at},"scopes":["user:profile","user:inference"]}}}}'
"#
        );
        std::fs::write(&executable, script).expect("write fake security");
        let mut permissions = std::fs::metadata(&executable)
            .expect("fake security metadata")
            .permissions();
        permissions.set_mode(0o700);
        std::fs::set_permissions(&executable, permissions).expect("make fake security executable");
        executable
    }
}
