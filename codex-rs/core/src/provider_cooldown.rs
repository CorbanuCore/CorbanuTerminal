//! User-requested recovery of the selected provider/model/credential's throttle.

use crate::StateDbHandle;
use crate::config::Config;
use crate::provider_throttle::provider_request_key;

/// Clear the same row that requests using this configuration consult. This
/// does not invoke an authentication command or send a request to the provider.
/// The caller supplies its auth mode: TUI and exec differ in whether they honor
/// CODEX_API_KEY, so reconstructing it here can select the wrong row.
pub async fn clear_provider_cooldown(
    state_db: &StateDbHandle,
    config: &Config,
    model: &str,
    auth_mode: Option<codex_protocol::auth::AuthMode>,
) -> anyhow::Result<()> {
    let key = provider_request_key(
        state_db,
        &config.codex_home,
        &config.model_provider_id,
        &config.model_provider,
        model,
        || auth_mode,
    )
    .await;
    state_db.clear_provider_request_cooldown(&key).await
}
