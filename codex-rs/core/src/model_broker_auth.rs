//! PF-27-S05: Core's own model-provider credentials held by the isolated
//! credential broker (feature `broker_model_auth`).
//!
//! [`install_for_config`] starts one contained broker per process, hands it
//! every provider-key environment variable (removing them from Core's
//! environment), and installs the process's [`ModelKeyBroker`]. From then on
//! every model-provider request this process makes (model client, web
//! search, image generation, model catalog) is signed for the broker, which
//! attaches the credential and performs the HTTPS request; the HTTP transport
//! sends such a request only to the broker's socket. Provider keys stored in
//! the vault are read by the broker, never by Core.
//!
//! Nothing falls back to sending a credential directly: a broker that cannot
//! start or has died, a provider URL the broker cannot bind (plain HTTP, an
//! IPv6 literal, a query), a missing key, and platforms without the broker
//! all fail the request. The broker is never restarted within a process.

use crate::config::Config;
use codex_api::AuthError;
use codex_api::AuthProvider;
use codex_api::AuthProviderFuture;
use codex_api::SharedAuthProvider;
use codex_http_client::MODEL_BROKER_FRAME_HEADER;
use codex_http_client::Request;
use codex_model_provider::BrokeredAuthRequest;
use codex_model_provider::BrokeredKeySource;
use codex_model_provider::ModelKeyBroker;
use codex_model_provider::ProviderApiKeyHeader;
use codex_protocol::error::CodexErr;
use http::HeaderMap;
use http::HeaderValue;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use zeroize::Zeroizing;

#[cfg(unix)]
type Credential = codex_network_proxy::model_auth::ModelCredential;
#[cfg(not(unix))]
#[derive(Clone)]
enum Credential {}

/// Installs the process's broker when `config` enables `broker_model_auth`.
/// Idempotent; the first enabled configuration's settings win.
pub async fn install_for_config(config: &Config) {
    // Also when an earlier configuration made this process brokered.
    if !(config
        .features
        .enabled(codex_features::Feature::BrokerModelAuth)
        || codex_model_provider::model_key_broker_required())
        || codex_model_provider::model_key_broker_installed()
    {
        return;
    }
    static INSTALL: tokio::sync::OnceCell<()> = tokio::sync::OnceCell::const_new();
    let settings = BrokerSettings::for_config(config);
    INSTALL
        .get_or_init(|| async move {
            // Starting the broker and handing over keys are blocking calls.
            let broker = tokio::task::spawn_blocking(move || CoreModelKeyBroker::start(settings))
                .await
                .unwrap_or_else(|_| CoreModelKeyBroker::new(BrokerHandle::Failed));
            codex_model_provider::install_model_key_broker(Arc::new(broker));
        })
        .await;
}

#[derive(Clone, Debug)]
struct BrokerSettings {
    runtime_dir: std::path::PathBuf,
    scrub_responses: bool,
    program: Option<std::path::PathBuf>,
    store_home: std::path::PathBuf,
    /// Every provider-key variable a configured provider may read.
    env_names: Vec<String>,
}

impl BrokerSettings {
    fn for_config(config: &Config) -> Self {
        let mut env_names: Vec<String> = config
            .model_providers
            .values()
            .chain(std::iter::once(&config.model_provider))
            .flat_map(|provider| provider.api_key_env_vars())
            .map(str::to_string)
            .collect();
        env_names.sort();
        env_names.dedup();
        Self {
            runtime_dir: config.codex_home.join("run").to_path_buf(),
            scrub_responses: config
                .features
                .enabled(codex_features::Feature::SecretOutputGate),
            program: config.codex_self_exe.clone(),
            store_home: config.codex_home.to_path_buf(),
            env_names,
        }
    }
}

#[derive(Clone)]
enum BrokerHandle {
    #[cfg(unix)]
    Running(codex_network_proxy::model_auth::ModelCredentialBroker),
    /// The broker could not start; it is not retried.
    Failed,
    /// No broker on this platform (PF-27-S06).
    #[cfg_attr(unix, allow(dead_code))]
    Unsupported,
}

/// Why a brokered request could not be authorized.
#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub(crate) enum BrokerModelAuthError {
    #[error("the isolated credential broker is unavailable; restart Corbanu to start a new one")]
    Unavailable,
    #[error(
        "broker_model_auth: the credential broker is not available on this platform; \
         turn broker_model_auth off to use provider keys and sign-in"
    )]
    Unsupported,
    #[error(
        "broker_model_auth: the provider base URL `{0}` cannot be brokered (it needs HTTPS with a \
         DNS name or IPv4 address and no query); turn broker_model_auth off to use it"
    )]
    Unbindable(String),
    #[error(
        "no API key for this provider: set {env} or save the key with /providers \
         (broker_model_auth reads it inside the credential broker)"
    )]
    MissingKey { env: String },
    #[error("the credential broker could not read the stored provider key: {0}")]
    Store(&'static str),
    #[error("the isolated credential broker refused the provider credential")]
    Rejected,
}

/// Where a credential may be sent: one HTTPS origin and a path prefix.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct Binding {
    pub(crate) host: String,
    pub(crate) port: u16,
    /// `/` or a plain path without a trailing slash, e.g. `/v1`.
    pub(crate) path_prefix: String,
    pub(crate) header: ProviderApiKeyHeader,
}

/// Where a credential for `base_url` may be sent, or `None` when the URL
/// cannot be brokered (not HTTPS, or a host the broker does not accept).
pub(crate) fn binding_for_base_url(
    base_url: &str,
    header: ProviderApiKeyHeader,
) -> Option<Binding> {
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
    Some(Binding {
        host,
        port: url.port_or_known_default()?,
        path_prefix: if path.is_empty() {
            "/".to_string()
        } else {
            path.to_string()
        },
        header,
    })
}

/// A cached registration: which version of its source it holds.
struct Registered {
    credential: Credential,
    version: [u8; 32],
}

struct BrokerState {
    handle: BrokerHandle,
    /// Keyed by binding and source slot (a provider key id, a sign-in slot,
    /// or a value fingerprint).
    credentials: HashMap<(Binding, String), Registered>,
    fingerprint_salt: [u8; 32],
}

/// The process's [`ModelKeyBroker`].
pub(crate) struct CoreModelKeyBroker {
    state: Arc<Mutex<BrokerState>>,
}

impl CoreModelKeyBroker {
    fn new(handle: BrokerHandle) -> Self {
        let mut salt = [0_u8; 32];
        rand::Rng::fill(&mut rand::rng(), salt.as_mut_slice());
        Self {
            state: Arc::new(Mutex::new(BrokerState {
                handle,
                credentials: HashMap::new(),
                fingerprint_salt: salt,
            })),
        }
    }

    /// The broker for platforms without one: every brokered request fails.
    #[cfg_attr(all(unix, not(test)), allow(dead_code))]
    pub(crate) fn unsupported() -> Self {
        Self::new(BrokerHandle::Unsupported)
    }

    #[cfg(not(unix))]
    fn start(_settings: BrokerSettings) -> Self {
        // No fallback: the broker does not run on this platform (PF-27-S06).
        Self::unsupported()
    }

    #[cfg(unix)]
    fn start(settings: BrokerSettings) -> Self {
        use codex_network_proxy::model_auth::ModelCredentialBroker;
        use codex_network_proxy::model_auth::ModelCredentialBrokerOptions;
        let spawned = ModelCredentialBroker::spawn(ModelCredentialBrokerOptions {
            runtime_dir: Some(settings.runtime_dir),
            scrub_responses: settings.scrub_responses,
            program: settings.program,
            store_home: Some(settings.store_home),
            withheld_env: settings.env_names.clone(),
        });
        let broker = match spawned {
            Ok(broker) => broker,
            Err(error) => {
                tracing::warn!("model credential broker did not start: {error}");
                // Core no longer uses these keys; child processes must not
                // inherit them either.
                codex_network_proxy::model_auth::scrub_env_keys(&settings.env_names);
                return Self::new(BrokerHandle::Failed);
            }
        };
        match broker.take_env_keys(&settings.env_names) {
            Ok(taken) if !taken.is_empty() => tracing::info!(
                "provider keys handed to the credential broker and removed from the environment: {}",
                taken.join(", ")
            ),
            Ok(_) => {}
            Err(error) => {
                // The keys are gone from the environment either way; a
                // broker that refused one is not trusted with any.
                tracing::warn!("credential broker refused an environment key: {error}");
                return Self::new(BrokerHandle::Failed);
            }
        }
        match codex_http_client::HttpClient::unix_socket(
            broker.socket_path(),
            codex_login::default_client::default_headers(),
        ) {
            Ok(client) => codex_http_client::install_model_broker_client(client),
            // Brokered requests then fail in the transport.
            Err(error) => tracing::warn!("credential broker client: {error}"),
        }
        Self::new(BrokerHandle::Running(broker))
    }
}

impl ModelKeyBroker for CoreModelKeyBroker {
    /// Registers the credential now (a blocking control-channel call, like the
    /// stored-key read it replaces) so a failure ends the turn at once.
    fn auth(
        &self,
        request: BrokeredAuthRequest,
    ) -> codex_protocol::error::Result<SharedAuthProvider> {
        let BrokeredAuthRequest {
            base_url,
            header,
            source,
            extra_headers,
        } = request;
        let fatal = |error: BrokerModelAuthError| CodexErr::Fatal(error.to_string());
        let binding = binding_for_base_url(&base_url, header)
            .ok_or_else(|| fatal(BrokerModelAuthError::Unbindable(base_url.clone())))?;
        let source = match source {
            BrokeredKeySource::ProviderKey {
                provider_key_id,
                env_vars,
            } => Source::ProviderKey {
                provider_key_id,
                env_vars,
            },
            BrokeredKeySource::Value { key, slot } => Source::Value {
                value: Zeroizing::new(key.value),
                slot,
            },
        };
        let credential = credential_for(&self.state, binding, &source).map_err(fatal)?;
        Ok(Arc::new(BrokeredAuth {
            credential,
            extra_headers,
        }))
    }
}

enum Source {
    ProviderKey {
        provider_key_id: String,
        env_vars: Vec<String>,
    },
    Value {
        value: Zeroizing<String>,
        slot: Option<String>,
    },
}

/// Request auth that names its credential by reference only.
struct BrokeredAuth {
    credential: Credential,
    extra_headers: HeaderMap,
}

impl AuthProvider for BrokeredAuth {
    /// Header-only paths (websockets, uploads, telemetry) get no credential.
    fn add_auth_headers(&self, _headers: &mut HeaderMap) {}

    fn apply_auth(&self, request: Request) -> AuthProviderFuture<'_> {
        Box::pin(async move {
            let mut request = broker_request(&self.credential, request)?;
            for (name, value) in &self.extra_headers {
                request.headers.insert(name.clone(), value.clone());
            }
            Ok(request)
        })
    }
}

/// The registered credential for `binding` and `source`, registering it (or
/// its refreshed value) when needed. The state lock is not held while the
/// broker registers (a blocking control-channel call), so other requests are
/// not serialized behind it.
fn credential_for(
    state: &Mutex<BrokerState>,
    binding: Binding,
    source: &Source,
) -> Result<Credential, BrokerModelAuthError> {
    let lock = || {
        state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    };
    let (key, version, seen, handle) = {
        let state = lock();
        let (slot, version) = match source {
            Source::ProviderKey {
                provider_key_id, ..
            } => {
                // A key saved or deleted in this process takes effect.
                let mut version = [0_u8; 32];
                version[..8].copy_from_slice(
                    &codex_login::provider_api_key_storage_revision().to_le_bytes(),
                );
                (format!("provider:{provider_key_id}"), version)
            }
            Source::Value { value, slot } => {
                use sha2::Digest as _;
                let fingerprint: [u8; 32] = sha2::Sha256::new()
                    .chain_update(state.fingerprint_salt)
                    .chain_update(value.as_bytes())
                    .finalize()
                    .into();
                let slot = slot
                    .clone()
                    .unwrap_or_else(|| format!("value:{}", hex(&fingerprint)));
                (slot, fingerprint)
            }
        };
        let key = (binding, slot);
        let seen = state
            .credentials
            .get(&key)
            .map(|registered| registered.version);
        if let Some(registered) = state.credentials.get(&key)
            && registered.version == version
        {
            return if alive(&registered.credential) {
                Ok(registered.credential.clone())
            } else {
                Err(BrokerModelAuthError::Unavailable)
            };
        }
        (key, version, seen, state.handle.clone())
    };
    let credential = match register(&handle, &key.0, source) {
        Ok(credential) => credential,
        Err(error) => {
            if matches!(error, BrokerModelAuthError::MissingKey { .. })
                && let Some(stale) = lock().credentials.remove(&key)
            {
                // The key was deleted: drop the broker's old copy too.
                unregister(&stale.credential);
            }
            return Err(error);
        }
    };
    let mut state = lock();
    let current = state
        .credentials
        .get(&key)
        .map(|registered| (registered.version, registered.credential.clone()));
    if let Some((current_version, current)) = current
        && Some(current_version) != seen
    {
        // Another request registered meanwhile (the same value, or a newer
        // one): keep it, drop ours, so stale snapshots never undo a refresh.
        drop(state);
        unregister(&credential);
        return Ok(current);
    }
    let binding = &key.0;
    tracing::info!(
        "model provider credential held by the credential broker (host={}, port={}, path={}, header={:?})",
        binding.host,
        binding.port,
        binding.path_prefix,
        binding.header
    );
    let replaced = state.credentials.insert(
        key,
        Registered {
            credential: credential.clone(),
            version,
        },
    );
    drop(state);
    if let Some(replaced) = replaced {
        // A refreshed token or re-saved key: drop the broker's old copy. A
        // request already signed with it fails once (not retried with a key).
        unregister(&replaced.credential);
    }
    Ok(credential)
}

#[cfg(unix)]
fn register(
    handle: &BrokerHandle,
    binding: &Binding,
    source: &Source,
) -> Result<Credential, BrokerModelAuthError> {
    use codex_network_proxy::model_auth::ModelAuthHeader;
    use codex_network_proxy::model_auth::ModelCredentialBinding;
    use codex_network_proxy::model_auth::ModelCredentialBrokerError;
    let broker = match handle {
        BrokerHandle::Running(broker) => broker,
        BrokerHandle::Failed => return Err(BrokerModelAuthError::Unavailable),
        BrokerHandle::Unsupported => return Err(BrokerModelAuthError::Unsupported),
    };
    let wire = ModelCredentialBinding {
        host: binding.host.clone(),
        port: binding.port,
        path_prefix: binding.path_prefix.clone(),
        header: match binding.header {
            ProviderApiKeyHeader::Bearer => ModelAuthHeader::Bearer,
            ProviderApiKeyHeader::XApiKey => ModelAuthHeader::XApiKey,
        },
    };
    let map_error = |error| match error {
        ModelCredentialBrokerError::Rejected => BrokerModelAuthError::Rejected,
        ModelCredentialBrokerError::Unavailable => BrokerModelAuthError::Unavailable,
        ModelCredentialBrokerError::StoreUnavailable => {
            BrokerModelAuthError::Store("the vault or the OS keyring is unavailable")
        }
    };
    match source {
        Source::ProviderKey {
            provider_key_id,
            env_vars,
        } => broker
            .register_stored(wire, provider_key_id, env_vars)
            .map_err(map_error)?
            .ok_or_else(|| BrokerModelAuthError::MissingKey {
                env: env_vars
                    .first()
                    .cloned()
                    .unwrap_or_else(|| provider_key_id.clone()),
            }),
        Source::Value { value, .. } => broker.register(wire, value).map_err(map_error),
    }
}

#[cfg(not(unix))]
fn register(
    handle: &BrokerHandle,
    _binding: &Binding,
    _source: &Source,
) -> Result<Credential, BrokerModelAuthError> {
    match handle {
        BrokerHandle::Failed => Err(BrokerModelAuthError::Unavailable),
        BrokerHandle::Unsupported => Err(BrokerModelAuthError::Unsupported),
    }
}

#[cfg(unix)]
fn alive(credential: &Credential) -> bool {
    credential.is_alive()
}

#[cfg(not(unix))]
fn alive(credential: &Credential) -> bool {
    match *credential {}
}

#[cfg(unix)]
fn unregister(credential: &Credential) {
    if let Err(error) = credential.unregister() {
        tracing::debug!("credential broker unregister: {error}");
    }
}

#[cfg(not(unix))]
fn unregister(credential: &Credential) {
    match *credential {}
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// Signs `request` for the broker and points it at the broker (the HTTP
/// transport sends a frame-bearing request only to the broker's socket).
#[cfg(unix)]
fn broker_request(credential: &Credential, mut request: Request) -> Result<Request, AuthError> {
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

#[cfg(not(unix))]
fn broker_request(credential: &Credential, _request: Request) -> Result<Request, AuthError> {
    match *credential {}
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
