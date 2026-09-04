use codex_model_provider_info::PFTERMINAL_PLAN_GATEWAY_ORIGIN;
use std::time::Duration;

const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(15);

pub(crate) struct WalletGatewayClient {
    pub(crate) origin: String,
    pub(crate) client: reqwest::Client,
}

/// Match model transport precedence without persisting environment credentials.
pub(crate) fn corbanu_account_key(
    home: &std::path::Path,
    store_mode: codex_config::types::AuthCredentialsStoreMode,
    keyring_backend: codex_config::types::AuthKeyringBackendKind,
) -> Result<Option<zeroize::Zeroizing<String>>, String> {
    if let Some(key) = codex_model_provider_info::corbanu_api_key_from_env() {
        return Ok(Some(zeroize::Zeroizing::new(key)));
    }
    codex_login::provider_api_key_from_auth_storage(
        home,
        codex_model_provider_info::PFTERMINAL_PLAN_API_KEY_ENV_VAR,
        store_mode,
        keyring_backend,
    )
    .map(|key| key.map(zeroize::Zeroizing::new))
    .map_err(|error| error.to_string())
}

pub(crate) fn gateway_origin() -> String {
    std::env::var("PFTERMINAL_PLAN_GATEWAY_URL")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| PFTERMINAL_PLAN_GATEWAY_ORIGIN.to_string())
        .trim_end_matches('/')
        .to_string()
}

pub(crate) fn gateway_client() -> Result<WalletGatewayClient, String> {
    let origin = gateway_origin();
    gateway_client_for_origin(origin)
}

fn gateway_client_for_origin(origin: String) -> Result<WalletGatewayClient, String> {
    codex_wallet::validate_gateway_origin(&origin).map_err(|error| error.to_string())?;
    let url = reqwest::Url::parse(&origin).map_err(|error| error.to_string())?;
    let loopback = matches!(url.host_str(), Some("127.0.0.1" | "localhost" | "::1"));
    let client = reqwest::Client::builder()
        .https_only(!loopback)
        .connect_timeout(CONNECT_TIMEOUT)
        .timeout(REQUEST_TIMEOUT)
        .build()
        .map_err(|error| error.to_string())?;
    Ok(WalletGatewayClient { origin, client })
}

#[cfg(test)]
#[path = "wallet_http_tests.rs"]
mod tests;
