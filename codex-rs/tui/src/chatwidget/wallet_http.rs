use codex_model_provider_info::PFTERMINAL_PLAN_GATEWAY_ORIGIN;
use std::time::Duration;

const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(15);

pub(crate) struct WalletGatewayClient {
    pub(crate) origin: String,
    pub(crate) client: reqwest::Client,
}

/// The Corbanu API key for account calls (wallet balance, plan usage).
pub(crate) enum CorbanuAccountKey {
    /// A key Corbanu reads: the environment's, else the stored one.
    Value(zeroize::Zeroizing<String>),
    /// #641: the environment's key, which the credential broker took out of
    /// this process (Aggressive's `broker_model_auth`). Account calls go
    /// through the broker, which attaches it as it does for model requests;
    /// Corbanu never reads it, and agent commands never see it.
    HeldByBroker,
}

/// Match model transport precedence without persisting environment credentials.
pub(crate) fn corbanu_account_key(
    home: &std::path::Path,
    store_mode: codex_config::types::AuthCredentialsStoreMode,
    keyring_backend: codex_config::types::AuthKeyringBackendKind,
) -> Result<Option<CorbanuAccountKey>, String> {
    account_key_from(
        codex_model_provider_info::corbanu_api_key_from_env(),
        codex_model_provider_info::CORBANU_API_KEY_ENV_VARS
            .iter()
            .any(|name| crate::legacy_core::broker_holds_environment_key(name)),
        || {
            codex_login::provider_api_key_from_auth_storage(
                home,
                codex_model_provider_info::PFTERMINAL_PLAN_API_KEY_ENV_VAR,
                store_mode,
                keyring_backend,
            )
            .map_err(|error| error.to_string())
        },
    )
}

/// [`corbanu_account_key`] from the environment's key, whether the broker
/// holds one (#641: it was the environment's, so it comes first), and the
/// stored key.
fn account_key_from(
    environment: Option<String>,
    held_by_broker: bool,
    stored: impl FnOnce() -> Result<Option<String>, String>,
) -> Result<Option<CorbanuAccountKey>, String> {
    if let Some(key) = environment {
        return Ok(Some(CorbanuAccountKey::Value(zeroize::Zeroizing::new(key))));
    }
    if held_by_broker {
        return Ok(Some(CorbanuAccountKey::HeldByBroker));
    }
    Ok(stored()?.map(|key| CorbanuAccountKey::Value(zeroize::Zeroizing::new(key))))
}

/// An account call's answer.
pub(crate) struct AccountResponse {
    pub(crate) status: reqwest::StatusCode,
    pub(crate) body: Vec<u8>,
}

impl AccountResponse {
    pub(crate) fn json<T: serde::de::DeserializeOwned>(&self) -> Result<T, String> {
        serde_json::from_slice(&self.body).map_err(|error| error.to_string())
    }
}

/// `GET {origin}{path}` with the Corbanu API key `key`.
pub(crate) async fn account_get(
    gateway: &WalletGatewayClient,
    path: &str,
    key: &CorbanuAccountKey,
) -> Result<AccountResponse, String> {
    let url = format!("{}{path}", gateway.origin);
    match key {
        CorbanuAccountKey::Value(key) => {
            let response = gateway
                .client
                .get(url)
                .bearer_auth(key.as_str())
                .send()
                .await
                .map_err(|error| error.to_string())?;
            let status = response.status();
            let body = response
                .bytes()
                .await
                .map_err(|error| error.to_string())?
                .to_vec();
            Ok(AccountResponse { status, body })
        }
        CorbanuAccountKey::HeldByBroker => brokered_get(gateway, url).await,
    }
}

/// #641: the broker binding for the Corbanu API key: the gateway's `/v1`, as
/// for the Corbanu API model provider, with the key it was handed from the
/// environment (else the stored key, which the broker reads itself).
fn brokered_account_auth(origin: &str) -> codex_model_provider::BrokeredAuthRequest {
    codex_model_provider::BrokeredAuthRequest {
        base_url: format!("{origin}/v1"),
        header: codex_model_provider::ProviderApiKeyHeader::Bearer,
        source: codex_model_provider::BrokeredKeySource::ProviderKey {
            provider_key_id: codex_model_provider_info::PFTERMINAL_PLAN_API_KEY_ENV_VAR.to_string(),
            env_vars: codex_model_provider_info::CORBANU_API_KEY_ENV_VARS
                .iter()
                .map(|name| (*name).to_string())
                .collect(),
            account: None,
        },
        extra_headers: reqwest::header::HeaderMap::new(),
    }
}

async fn brokered_get(
    gateway: &WalletGatewayClient,
    url: String,
) -> Result<AccountResponse, String> {
    use codex_http_client::HttpTransport as _;
    // Registering the credential is a blocking call to the broker.
    let binding = brokered_account_auth(&gateway.origin);
    let auth = tokio::task::spawn_blocking(move || codex_model_provider::brokered_auth(binding))
        .await
        .map_err(|error| error.to_string())?
        .map_err(|error| error.to_string())?;
    let mut request = codex_http_client::Request::new(reqwest::Method::GET, url);
    request.timeout = Some(REQUEST_TIMEOUT);
    let request = auth
        .apply_auth(request)
        .await
        .map_err(|error| error.to_string())?;
    match codex_http_client::ReqwestTransport::new(gateway.client.clone())
        .execute(request)
        .await
    {
        Ok(response) => Ok(AccountResponse {
            status: response.status,
            body: response.body.to_vec(),
        }),
        // A refusal still answers the call (401: the key is no longer valid).
        Err(codex_http_client::TransportError::Http { status, body, .. }) => Ok(AccountResponse {
            status,
            body: body.unwrap_or_default().into_bytes(),
        }),
        Err(error) => Err(error.to_string()),
    }
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
