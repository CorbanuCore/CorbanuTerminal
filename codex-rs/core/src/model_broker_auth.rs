//! PF-27-S05: Core's own model-provider API keys held by the isolated
//! credential broker (feature `broker_model_auth`).
//!
//! When a provider's auth is one plain API key, Core registers the key with a
//! contained broker process once and keeps only an opaque [`ModelCredential`].
//! Each model request is then sent as plain HTTP over the broker's private
//! Unix socket with a signed frame for its exact origin, method and path; the
//! broker attaches the key and performs the HTTPS request. Nothing falls back
//! to sending the key directly: a broker that cannot start, or has died,
//! fails the request.
//!
//! Not brokered (sent as before): sign-in tokens, agent identity, command or
//! header auth, AWS auth and plain-HTTP providers. Responses websockets are
//! off under the flag; a websocket handshake cannot carry a signed frame.

use crate::client::BrokerModelAuthConfig;
use codex_api::AuthError;
use codex_api::AuthProvider;
use codex_api::AuthProviderFuture;
use codex_api::SharedAuthProvider;
use codex_http_client::HttpClient;
use codex_http_client::Request;
use codex_model_provider::ProviderApiKey;
use codex_model_provider::ProviderApiKeyHeader;
use codex_network_proxy::model_auth::MODEL_BROKER_FRAME_HEADER;
use codex_network_proxy::model_auth::ModelAuthHeader;
use codex_network_proxy::model_auth::ModelCredential;
use codex_network_proxy::model_auth::ModelCredentialBinding;
use codex_network_proxy::model_auth::ModelCredentialBroker;
use codex_network_proxy::model_auth::ModelCredentialBrokerOptions;
use http::HeaderMap;
use http::HeaderValue;
use sha2::Digest as _;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::LazyLock;
use std::sync::Mutex;

/// One broker per Core process, shared by every session and sub-agent; each
/// distinct key and binding is registered once.
#[derive(Default)]
struct ProcessBroker {
    /// `None` until first use; `Some(None)` after a failed start (never
    /// retried, like the PF-27-S04 broker).
    broker: Option<Option<ModelCredentialBroker>>,
    credentials: HashMap<(BindingKey, [u8; 32]), ModelCredential>,
    fingerprint_salt: [u8; 32],
}

type BindingKey = (String, u16, String, ProviderApiKeyHeader);

static PROCESS_BROKER: LazyLock<Mutex<ProcessBroker>> = LazyLock::new(|| {
    let mut salt = [0_u8; 32];
    rand::Rng::fill(&mut rand::rng(), salt.as_mut_slice());
    Mutex::new(ProcessBroker {
        fingerprint_salt: salt,
        ..ProcessBroker::default()
    })
});

/// Why a model request could not be brokered.
#[derive(Debug, thiserror::Error)]
pub(crate) enum BrokerModelAuthError {
    #[error("the isolated credential broker is unavailable; restart Corbanu to start a new one")]
    Unavailable,
    #[error("the isolated credential broker refused the model provider key")]
    Rejected,
}

impl From<BrokerModelAuthError> for std::io::Error {
    fn from(error: BrokerModelAuthError) -> Self {
        std::io::Error::other(error.to_string())
    }
}

/// Where a key for `base_url` may be sent, or `None` when the URL cannot be
/// brokered (not HTTPS, or a host the broker does not accept).
pub(crate) fn binding_for_base_url(
    base_url: &str,
    header: ProviderApiKeyHeader,
) -> Option<ModelCredentialBinding> {
    let url = url::Url::parse(base_url).ok()?;
    if url.scheme() != "https" || url.query().is_some() || url.fragment().is_some() {
        return None;
    }
    let host = match url.host()? {
        url::Host::Domain(host) => host.to_ascii_lowercase(),
        url::Host::Ipv4(ip) => ip.to_string(),
        url::Host::Ipv6(_) => return None,
    };
    let path = url.path().trim_end_matches('/');
    Some(ModelCredentialBinding {
        host,
        port: url.port_or_known_default()?,
        path_prefix: if path.is_empty() {
            "/".to_string()
        } else {
            path.to_string()
        },
        header: match header {
            ProviderApiKeyHeader::Bearer => ModelAuthHeader::Bearer,
            ProviderApiKeyHeader::XApiKey => ModelAuthHeader::XApiKey,
        },
    })
}

/// Registers `key` (once per process) and returns the opaque credential.
pub(crate) async fn credential_for(
    config: &BrokerModelAuthConfig,
    binding: ModelCredentialBinding,
    key: ProviderApiKey,
) -> Result<ModelCredential, BrokerModelAuthError> {
    let config = config.clone();
    // Starting the broker and registering are blocking control-channel calls.
    tokio::task::spawn_blocking(move || credential_for_blocking(&config, binding, key))
        .await
        .map_err(|_| BrokerModelAuthError::Unavailable)?
}

fn credential_for_blocking(
    config: &BrokerModelAuthConfig,
    binding: ModelCredentialBinding,
    key: ProviderApiKey,
) -> Result<ModelCredential, BrokerModelAuthError> {
    let mut process = PROCESS_BROKER
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let fingerprint: [u8; 32] = sha2::Sha256::new()
        .chain_update(process.fingerprint_salt)
        .chain_update(key.value.as_bytes())
        .finalize()
        .into();
    let cache_key = (
        (
            binding.host.clone(),
            binding.port,
            binding.path_prefix.clone(),
            key.header,
        ),
        fingerprint,
    );
    if let Some(credential) = process.credentials.get(&cache_key) {
        return if credential.is_alive() {
            Ok(credential.clone())
        } else {
            Err(BrokerModelAuthError::Unavailable)
        };
    }
    let broker = process
        .broker
        .get_or_insert_with(|| {
            ModelCredentialBroker::spawn(ModelCredentialBrokerOptions {
                runtime_dir: Some(config.runtime_dir.clone()),
                scrub_responses: config.scrub_responses,
            })
            .inspect_err(|error| tracing::warn!("model credential broker did not start: {error}"))
            .ok()
        })
        .clone()
        .ok_or(BrokerModelAuthError::Unavailable)?;
    let credential = broker
        .register(binding, &key.value)
        .map_err(|error| match error {
            codex_network_proxy::model_auth::ModelCredentialBrokerError::Rejected => {
                BrokerModelAuthError::Rejected
            }
            codex_network_proxy::model_auth::ModelCredentialBrokerError::Unavailable => {
                BrokerModelAuthError::Unavailable
            }
        })?;
    drop(key);
    let binding = credential.binding();
    tracing::info!(
        "model provider key held by the credential broker (host={}, port={}, path={}, header={:?})",
        binding.host,
        binding.port,
        binding.path_prefix,
        binding.header
    );
    process.credentials.insert(cache_key, credential.clone());
    Ok(credential)
}

/// Builds the HTTP client for brokered model requests.
pub(crate) fn broker_http_client(
    credential: &ModelCredential,
) -> Result<HttpClient, BrokerModelAuthError> {
    HttpClient::unix_socket(
        credential.socket_path(),
        codex_login::default_client::default_headers(),
    )
    .map_err(|_| BrokerModelAuthError::Unavailable)
}

/// Request auth that names the key by reference only.
#[derive(Debug)]
pub(crate) struct BrokeredModelAuthProvider {
    credential: ModelCredential,
}

impl BrokeredModelAuthProvider {
    pub(crate) fn shared(credential: ModelCredential) -> SharedAuthProvider {
        Arc::new(Self { credential })
    }
}

impl AuthProvider for BrokeredModelAuthProvider {
    /// Header-only paths (websockets, telemetry, uploads) get no credential.
    fn add_auth_headers(&self, _headers: &mut HeaderMap) {}

    fn apply_auth(&self, request: Request) -> AuthProviderFuture<'_> {
        Box::pin(async move { broker_request(&self.credential, request) })
    }
}

/// Signs `request` for the broker and points it at the broker's socket.
fn broker_request(
    credential: &ModelCredential,
    mut request: Request,
) -> Result<Request, AuthError> {
    let rewrite = BrokerRewrite::for_url(&request.url)
        .ok_or_else(|| AuthError::Build("model request URL cannot be brokered".to_string()))?;
    let frame = credential
        .sign(
            request.method.as_str(),
            &rewrite.host,
            rewrite.port,
            &rewrite.path_and_query,
        )
        .map_err(|error| AuthError::Build(format!("credential broker: {error}")))?;
    let frame = HeaderValue::from_str(&frame)
        .map_err(|_| AuthError::Build("credential broker frame".to_string()))?;
    request.headers.remove(http::header::AUTHORIZATION);
    request.headers.remove("x-api-key");
    request.headers.insert(MODEL_BROKER_FRAME_HEADER, frame);
    request.url = rewrite.broker_url;
    Ok(request)
}

/// The signed parts of an HTTPS URL and the plain-HTTP URL sent to the broker.
#[derive(Debug, PartialEq, Eq)]
struct BrokerRewrite {
    host: String,
    port: u16,
    path_and_query: String,
    broker_url: String,
}

impl BrokerRewrite {
    fn for_url(url: &str) -> Option<Self> {
        let mut url = url::Url::parse(url).ok()?;
        if url.scheme() != "https" || url.fragment().is_some() {
            return None;
        }
        let host = match url.host()? {
            url::Host::Domain(host) => host.to_ascii_lowercase(),
            url::Host::Ipv4(ip) => ip.to_string(),
            url::Host::Ipv6(_) => return None,
        };
        let port = url.port_or_known_default()?;
        let path_and_query = match url.query() {
            Some(query) => format!("{}?{query}", url.path()),
            None => url.path().to_string(),
        };
        url.set_scheme("http").ok()?;
        url.set_port(Some(port)).ok()?;
        Some(Self {
            host,
            port,
            path_and_query,
            broker_url: url.to_string(),
        })
    }
}

#[cfg(test)]
#[path = "model_broker_auth_tests.rs"]
mod tests;
