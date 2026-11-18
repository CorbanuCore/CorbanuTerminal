//! User-requested recovery of the selected provider/model/credential's throttle.

use crate::StateDbHandle;
use crate::config::Config;
use crate::provider_throttle::provider_request_key;

/// Clear the same row that requests using this configuration consult. This
/// does not invoke an authentication command or send a request to the provider.
/// The caller supplies its API-key environment policy: TUI and exec differ in
/// whether they honor CODEX_API_KEY. Authentication stays behind the core boundary.
pub async fn clear_provider_cooldown(
    state_db: &StateDbHandle,
    config: &Config,
    model: &str,
    enable_codex_api_key_env: bool,
) -> anyhow::Result<()> {
    let auth = codex_login::AuthManager::shared_from_config(config, enable_codex_api_key_env).await;
    let key = provider_request_key(
        state_db,
        &config.codex_home,
        &config.model_provider_id,
        &config.model_provider,
        model,
        || auth.auth_mode(),
    )
    .await;
    state_db.clear_provider_request_cooldown(&key).await
}
