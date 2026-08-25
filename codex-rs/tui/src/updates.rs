#![cfg(any(not(debug_assertions), test))]

use crate::legacy_core::config::Config;
use crate::update_action;
use crate::update_action::UpdateAction;
use crate::update_versions::extract_version_from_latest_tag;
use crate::update_versions::is_newer;
use crate::update_versions::is_source_build_version;
use crate::updates_cache::VersionInfo;
use crate::updates_cache::read_version_info;
use crate::updates_cache::version_filepath;
use chrono::Duration;
use chrono::Utc;
use codex_http_client::ClientRouteClass;
use codex_http_client::HttpClientFactory;
use codex_http_client::RouteAwareClientPool;
use codex_login::default_client::default_headers;
use serde::Deserialize;
use std::path::Path;

use crate::version::CODEX_CLI_VERSION;

pub fn get_upgrade_version(config: &Config) -> Option<String> {
    if !config.check_for_update_on_startup || is_source_build_version(CODEX_CLI_VERSION) {
        return None;
    }

    let action = update_action::get_update_action();
    let version_file = version_filepath(config);
    let info = read_version_info(&version_file).ok();

    if match &info {
        None => true,
        Some(info) => info.last_checked_at < Utc::now() - Duration::hours(20),
    } {
        let http_client_factory = config.http_client_factory();
        // Refresh the cached latest version in the background so TUI startup
        // isn’t blocked by a network call. The UI reads the previously cached
        // value (if any) for this run; the next run shows the banner if needed.
        tokio::spawn(async move {
            check_for_update(&version_file, action, http_client_factory)
                .await
                .inspect_err(|e| tracing::error!("Failed to update version: {e}"))
        });
    }

    info.and_then(|info| {
        if is_newer(&info.latest_version, CODEX_CLI_VERSION).unwrap_or(false) {
            Some(info.latest_version)
        } else {
            None
        }
    })
}

const CORBANU_LATEST_RELEASE_URL: &str =
    "https://api.github.com/repos/CorbanuCore/CorbanuTerminal/releases/latest";
const CORBANU_RELEASE_BY_TAG_URL: &str =
    "https://api.github.com/repos/CorbanuCore/CorbanuTerminal/releases/tags";
const LEGACY_PFTERMINAL_LATEST_RELEASE_URL: &str =
    "https://api.github.com/repos/agtico/PfTerminal/releases/latest";
const LEGACY_PFTERMINAL_RELEASE_BY_TAG_URL: &str =
    "https://api.github.com/repos/agtico/PfTerminal/releases/tags";
const RELEASE_VALIDATION_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(8);

#[derive(Deserialize, Debug, Clone)]
struct ReleaseInfo {
    tag_name: String,
}

async fn check_for_update(
    version_file: &Path,
    action: Option<UpdateAction>,
    http_client_factory: HttpClientFactory,
) -> anyhow::Result<()> {
    let client_pool = RouteAwareClientPool::with_chatgpt_cloudflare_cookies(
        http_client_factory,
        ClientRouteClass::Other,
    )
    .with_legacy_custom_ca_fallback();
    let latest_version = fetch_latest_version_for_action_with_fallback(
        action,
        &client_pool,
        CORBANU_LATEST_RELEASE_URL,
        Some(LEGACY_PFTERMINAL_LATEST_RELEASE_URL),
    )
    .await?;

    persist_version_info(version_file, latest_version).await
}

async fn fetch_latest_version_for_action(
    action: Option<UpdateAction>,
    client_pool: &RouteAwareClientPool,
    github_latest_release_url: &str,
) -> anyhow::Result<String> {
    match action {
        Some(UpdateAction::StandaloneUnix) | Some(UpdateAction::StandaloneWindows) | None => {
            fetch_latest_github_release_version(client_pool, github_latest_release_url).await
        }
    }
}

async fn fetch_latest_version_for_action_with_fallback(
    action: Option<UpdateAction>,
    client_pool: &RouteAwareClientPool,
    primary_url: &str,
    fallback_url: Option<&str>,
) -> anyhow::Result<String> {
    match fetch_latest_version_for_action(action, client_pool, primary_url).await {
        Ok(version) => Ok(version),
        Err(primary_error) => {
            let Some(fallback_url) = fallback_url else {
                return Err(primary_error);
            };
            tracing::warn!(
                %primary_error,
                "canonical Corbanu release endpoint failed; trying the legacy PFTerminal redirect"
            );
            fetch_latest_version_for_action(action, client_pool, fallback_url).await
        }
    }
}

async fn persist_version_info(version_file: &Path, latest_version: String) -> anyhow::Result<()> {
    // Preserve any previously dismissed version if present.
    let prev_info = read_version_info(version_file).ok();
    let info = VersionInfo {
        latest_version,
        last_checked_at: Utc::now(),
        dismissed_version: prev_info.and_then(|p| p.dismissed_version),
    };

    let json_line = format!("{}\n", serde_json::to_string(&info)?);
    if let Some(parent) = version_file.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    tokio::fs::write(version_file, json_line).await?;
    Ok(())
}

async fn fetch_latest_github_release_version(
    client_pool: &RouteAwareClientPool,
    latest_release_url: &str,
) -> anyhow::Result<String> {
    let ReleaseInfo {
        tag_name: latest_tag_name,
    } = client_pool
        .get(latest_release_url)
        .headers(default_headers())
        .send()
        .await?
        .error_for_status()?
        .json::<ReleaseInfo>()
        .await?;
    extract_version_from_latest_tag(&latest_tag_name)
}

/// Returns the latest version to show in a popup, if it should be shown.
/// This respects the user's dismissal choice for the current latest version.
pub async fn get_upgrade_version_for_popup(config: &Config) -> Option<String> {
    if !config.check_for_update_on_startup || is_source_build_version(CODEX_CLI_VERSION) {
        return None;
    }

    let action = update_action::get_update_action();
    let latest = match tokio::time::timeout(
        RELEASE_VALIDATION_TIMEOUT,
        revalidated_upgrade_version(
            config,
            action,
            CODEX_CLI_VERSION,
            CORBANU_LATEST_RELEASE_URL,
            CORBANU_RELEASE_BY_TAG_URL,
            Some((
                LEGACY_PFTERMINAL_LATEST_RELEASE_URL,
                LEGACY_PFTERMINAL_RELEASE_BY_TAG_URL,
            )),
        ),
    )
    .await
    {
        Ok(Ok(latest)) => latest?,
        Ok(Err(err)) => {
            tracing::warn!("release revalidation failed; suppressing update prompt: {err}");
            return None;
        }
        Err(_) => {
            tracing::warn!("release revalidation timed out; suppressing update prompt");
            return None;
        }
    };
    let version_file = version_filepath(config);
    // If the user dismissed this exact version previously, do not show the popup.
    if let Ok(info) = read_version_info(&version_file)
        && info.dismissed_version.as_deref() == Some(latest.as_str())
    {
        return None;
    }
    Some(latest)
}

async fn revalidated_upgrade_version(
    config: &Config,
    action: Option<UpdateAction>,
    current_version: &str,
    latest_release_url: &str,
    release_by_tag_url: &str,
    fallback_urls: Option<(&str, &str)>,
) -> anyhow::Result<Option<String>> {
    let client_pool = RouteAwareClientPool::with_chatgpt_cloudflare_cookies(
        config.http_client_factory(),
        ClientRouteClass::Other,
    )
    .with_legacy_custom_ca_fallback();
    let latest = fetch_latest_version_for_action_with_fallback(
        action,
        &client_pool,
        latest_release_url,
        fallback_urls.map(|urls| urls.0),
    )
    .await?;
    let version_file = version_filepath(config);
    persist_version_info(&version_file, latest.clone()).await?;

    if is_newer(&latest, current_version).unwrap_or(false) {
        return Ok(Some(latest));
    }
    if latest == current_version {
        return Ok(None);
    }

    let installed_exists = github_release_exists_with_fallback(
        &client_pool,
        &format!("{release_by_tag_url}/rust-v{current_version}"),
        fallback_urls.map(|urls| format!("{}/rust-v{current_version}", urls.1)),
    )
    .await?;
    if installed_exists {
        Ok(None)
    } else {
        // The installed release has been recalled. The latest published release
        // is the recovery target even when its version is numerically lower.
        Ok(Some(latest))
    }
}

async fn github_release_exists(
    client_pool: &RouteAwareClientPool,
    release_url: &str,
) -> anyhow::Result<bool> {
    let response = client_pool
        .get(release_url)
        .headers(default_headers())
        .send()
        .await?;
    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(false);
    }
    response.error_for_status()?;
    Ok(true)
}

async fn github_release_exists_with_fallback(
    client_pool: &RouteAwareClientPool,
    primary_url: &str,
    fallback_url: Option<String>,
) -> anyhow::Result<bool> {
    match github_release_exists(client_pool, primary_url).await {
        Ok(exists) => Ok(exists),
        Err(primary_error) => {
            let Some(fallback_url) = fallback_url else {
                return Err(primary_error);
            };
            tracing::warn!(
                %primary_error,
                "canonical Corbanu tag endpoint failed; trying the legacy PFTerminal redirect"
            );
            github_release_exists(client_pool, &fallback_url).await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::legacy_core::config::ConfigBuilder;
    use wiremock::Mock;
    use wiremock::MockServer;
    use wiremock::ResponseTemplate;
    use wiremock::matchers::method;
    use wiremock::matchers::path;

    async fn test_config() -> (tempfile::TempDir, Config) {
        let home = tempfile::tempdir().expect("temp home");
        let config = ConfigBuilder::default()
            .codex_home(home.path().to_path_buf())
            .build()
            .await
            .expect("config");
        (home, config)
    }

    #[tokio::test]
    async fn deleted_cached_target_is_revalidated_before_prompting() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/releases/latest"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "tag_name": "rust-v1.0.0"
            })))
            .expect(1)
            .mount(&server)
            .await;
        let (_home, config) = test_config().await;
        persist_version_info(&version_filepath(&config), "1.2.0".to_string())
            .await
            .expect("seed stale cache");

        let result = revalidated_upgrade_version(
            &config,
            /*action*/ None,
            "1.0.0",
            &format!("{}/releases/latest", server.uri()),
            &format!("{}/releases/tags", server.uri()),
            /*fallback_urls*/ None,
        )
        .await
        .expect("revalidate");

        assert_eq!(result, None);
        assert_eq!(
            read_version_info(&version_filepath(&config))
                .expect("refreshed cache")
                .latest_version,
            "1.0.0"
        );
    }

    #[tokio::test]
    async fn deleted_installed_release_prompts_recall_downgrade() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/releases/latest"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "tag_name": "rust-v1.1.0"
            })))
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/releases/tags/rust-v1.2.0"))
            .respond_with(ResponseTemplate::new(404))
            .expect(1)
            .mount(&server)
            .await;
        let (_home, config) = test_config().await;

        let result = revalidated_upgrade_version(
            &config,
            /*action*/ None,
            "1.2.0",
            &format!("{}/releases/latest", server.uri()),
            &format!("{}/releases/tags", server.uri()),
            /*fallback_urls*/ None,
        )
        .await
        .expect("revalidate");

        assert_eq!(result.as_deref(), Some("1.1.0"));
    }

    #[tokio::test]
    async fn existing_installed_release_does_not_prompt_downgrade() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/releases/latest"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "tag_name": "rust-v1.1.0"
            })))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/releases/tags/rust-v1.2.0"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&server)
            .await;
        let (_home, config) = test_config().await;

        let result = revalidated_upgrade_version(
            &config,
            /*action*/ None,
            "1.2.0",
            &format!("{}/releases/latest", server.uri()),
            &format!("{}/releases/tags", server.uri()),
            /*fallback_urls*/ None,
        )
        .await
        .expect("revalidate");

        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn canonical_release_failure_uses_legacy_redirect_fallback() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/canonical/releases/latest"))
            .respond_with(ResponseTemplate::new(503))
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/legacy/releases/latest"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "tag_name": "rust-v1.3.0"
            })))
            .expect(1)
            .mount(&server)
            .await;
        let (_home, config) = test_config().await;

        let result = revalidated_upgrade_version(
            &config,
            /*action*/ None,
            "1.2.0",
            &format!("{}/canonical/releases/latest", server.uri()),
            &format!("{}/canonical/releases/tags", server.uri()),
            Some((
                &format!("{}/legacy/releases/latest", server.uri()),
                &format!("{}/legacy/releases/tags", server.uri()),
            )),
        )
        .await
        .expect("fallback revalidation");

        assert_eq!(result.as_deref(), Some("1.3.0"));
    }
}
