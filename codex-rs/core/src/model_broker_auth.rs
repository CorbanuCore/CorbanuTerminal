//! PF-27-S05: Core's own model-provider credentials held by the isolated
//! credential broker (feature `broker_model_auth`).
//!
//! [`install_for_config`] starts one contained broker per process, hands it
//! every provider-key environment variable (removing them from Core's
//! environment), and installs the process's [`ModelKeyBroker`]. From then on
//! every model-provider request this process makes (model client, web
//! search, image generation, model catalog) is signed for the broker, which
//! attaches the credential and performs the HTTPS request; the HTTP transport
//! sends such a request only to the broker's socket (on Windows, PF-27-S09,
//! its data pipe, every connection checked to be served by the broker). For
//! these requests, provider keys stored in the vault are read by the broker,
//! not by Core (other features may still open the vault; see the sprint's
//! known limits).
//!
//! Nothing falls back to sending a credential directly: a broker that cannot
//! start or has died, a provider URL the broker cannot bind (plain HTTP, an
//! IPv6 literal, a query), a missing key, and platforms without the broker
//! all fail the request. The broker is never restarted within a process.
//!
//! #391: security level Aggressive turns the feature on by default where the
//! broker runs (macOS, Linux, Windows); the person's own setting still wins
//! (see [`level_broker_setting`]).

use crate::config::Config;
use codex_api::AuthError;
use codex_api::AuthProvider;
use codex_api::AuthProviderFuture;
use codex_api::SharedAuthProvider;
use codex_http_client::Request;
use codex_model_provider::BrokeredAuthRequest;
use codex_model_provider::BrokeredKeySource;
use codex_model_provider::ModelKeyBroker;
use codex_model_provider::ProviderApiKeyHeader;
use codex_protocol::error::CodexErr;
use codex_security_policy::SecurityLevel;
use http::HeaderMap;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use zeroize::Zeroizing;

#[cfg(any(unix, windows))]
type Credential = codex_network_proxy::model_auth::ModelCredential;
#[cfg(not(any(unix, windows)))]
#[derive(Clone)]
enum Credential {}

/// What turned `broker_model_auth` on for a configuration (#391).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BrokerModelAuthOrigin {
    /// Off, or set by config or a launch flag.
    #[default]
    Config,
    /// Security level Aggressive turned it on; nothing sets it explicitly.
    AggressiveLevel,
}

/// Operating systems where Aggressive turns the broker on by default: the
/// model-auth broker is qualified on macOS and Linux (PF-27-S05) and Windows
/// (PF-27-S09). Elsewhere it stays off unless config turns it on.
pub(crate) const LEVEL_DEFAULT_SUPPORTED: bool =
    cfg!(any(target_os = "macos", target_os = "linux", windows));

/// #391: the `broker_model_auth` value `level` sets, or `None` to keep
/// config's merged value. Aggressive on a `supported` OS sets it on unless
/// the person's own configuration sets it (`explicit`, which leaves out
/// project layers, so a repository cannot turn it off).
pub(crate) fn level_broker_setting(
    level: SecurityLevel,
    supported: bool,
    explicit: Option<bool>,
) -> Option<bool> {
    (level == SecurityLevel::Aggressive && supported).then(|| explicit.unwrap_or(true))
}

/// #391: the level that decides the default for `codex_home`: the stricter of
/// Core's level and the level `/security` stored, as this process first saw
/// them. A level chosen while Corbanu runs takes effect at the next start,
/// so a config rebuilt mid-session never makes a running session brokered
/// without a broker.
pub(crate) fn level_for_defaults(
    codex_home: &std::path::Path,
    core_level: SecurityLevel,
) -> SecurityLevel {
    let level = core_level.max(stored_security_level(codex_home));
    if cfg!(test) {
        // Core's unit tests load many levels for one home path pattern.
        return level;
    }
    static SEEN: std::sync::OnceLock<Mutex<HashMap<std::path::PathBuf, SecurityLevel>>> =
        std::sync::OnceLock::new();
    *SEEN
        .get_or_init(Default::default)
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .entry(codex_home.to_path_buf())
        .or_insert(level)
}

/// The level `/security` stored for this home (Corbanu Terminal enforces it
/// at launch); unreadable state reads as Aggressive, never as Permissive.
fn stored_security_level(codex_home: &std::path::Path) -> SecurityLevel {
    match codex_security_level::level::load(codex_home).enforced() {
        codex_security_level::level::ChosenLevel::Permissive => SecurityLevel::Permissive,
        codex_security_level::level::ChosenLevel::Aggressive => SecurityLevel::Aggressive,
    }
}

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
    if cfg!(test) && config.broker_model_auth_origin == BrokerModelAuthOrigin::AggressiveLevel {
        // Core's unit tests load Aggressive configs in one shared process
        // (see `Config::load_config_with_layer_stack`).
        return;
    }
    static INSTALL: tokio::sync::OnceCell<()> = tokio::sync::OnceCell::const_new();
    let settings = BrokerSettings::for_config(config);
    let origin = settings.origin;
    INSTALL
        .get_or_init(|| async move {
            // Starting the broker and handing over keys are blocking calls.
            let broker = tokio::task::spawn_blocking(move || CoreModelKeyBroker::start(settings))
                .await
                .unwrap_or_else(|_| {
                    CoreModelKeyBroker::not_started(StartFailure::Unavailable, origin)
                });
            codex_model_provider::install_model_key_broker(Arc::new(broker));
        })
        .await;
}

#[derive(Clone, Debug)]
#[cfg_attr(
    not(any(unix, windows)),
    allow(dead_code, reason = "only the broker reads these settings")
)]
struct BrokerSettings {
    runtime_dir: std::path::PathBuf,
    scrub_responses: bool,
    program: Option<std::path::PathBuf>,
    store_home: std::path::PathBuf,
    /// Every provider-key variable a configured provider may read.
    env_names: Vec<String>,
    /// For the message when the broker cannot start.
    origin: BrokerModelAuthOrigin,
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
            origin: config.broker_model_auth_origin,
        }
    }
}

#[derive(Clone)]
enum BrokerHandle {
    #[cfg(any(unix, windows))]
    Running(codex_network_proxy::model_auth::ModelCredentialBroker),
    /// The broker could not start, for this reason; it is not retried.
    Failed(BrokerModelAuthError),
    /// No broker on this platform (PF-27-S06).
    #[cfg_attr(any(unix, windows), allow(dead_code))]
    Unsupported,
}

/// Why the broker did not start (#391).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum StartFailure {
    Unavailable,
    /// #390 (Windows): another process holds or serves its named pipe.
    #[cfg_attr(not(any(unix, windows)), allow(dead_code))]
    PipeSquatted,
}

/// How to send requests without the broker, for this origin.
fn remedy(origin: BrokerModelAuthOrigin) -> &'static str {
    match origin {
        BrokerModelAuthOrigin::AggressiveLevel => {
            "Security level Aggressive turns broker_model_auth on. To send model requests \
             without the broker, choose Permissive in /security, or set \
             `broker_model_auth = false` under [features] in config.toml, then restart"
        }
        BrokerModelAuthOrigin::Config => {
            "broker_model_auth is on in your configuration. To send model requests without the \
             broker, set `broker_model_auth = false` under [features] in config.toml, then restart"
        }
    }
}

fn not_started_message(cause: StartFailure, origin: BrokerModelAuthOrigin) -> String {
    let cause = match cause {
        StartFailure::Unavailable => "the isolated credential broker did not start",
        StartFailure::PipeSquatted => {
            "the isolated credential broker's named pipe is held or served by another process \
             (possible pipe squatting), so Corbanu refused it and sent nothing"
        }
    };
    format!(
        "model requests are refused: {cause}; nothing is sent without it. Restart Corbanu to try \
         again. {}.",
        remedy(origin)
    )
}

/// Why a brokered request could not be authorized.
#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub(crate) enum BrokerModelAuthError {
    #[error("the isolated credential broker is unavailable; restart Corbanu to start a new one")]
    Unavailable,
    /// #391: the broker did not start; every model request is refused.
    #[error("{}", not_started_message(*.cause, *.origin))]
    NotStarted {
        cause: StartFailure,
        origin: BrokerModelAuthOrigin,
    },
    #[error(
        "broker_model_auth: the credential broker is not available on this platform; \
         turn broker_model_auth off to use provider keys and sign-in"
    )]
    Unsupported,
    #[error(
        "the provider base URL `{url}` cannot be brokered (it needs HTTPS with a DNS name or IPv4 \
         address and no query). {}.",
        remedy(*.origin)
    )]
    Unbindable {
        url: String,
        origin: BrokerModelAuthOrigin,
    },
    #[error(
        "no API key for this provider: set {env} or save the key with /providers \
         (broker_model_auth reads it inside the credential broker)"
    )]
    #[cfg_attr(
        not(any(unix, windows)),
        allow(dead_code, reason = "only the broker reports it")
    )]
    MissingKey { env: String },
    #[error("the credential broker could not read the stored provider key: {0}")]
    #[cfg_attr(
        not(any(unix, windows)),
        allow(dead_code, reason = "only the broker reports it")
    )]
    Store(&'static str),
    #[error("the isolated credential broker refused the provider credential")]
    #[cfg_attr(
        not(any(unix, windows)),
        allow(dead_code, reason = "only the broker reports it")
    )]
    Rejected,
    /// #390 (Windows): another process holds or serves the broker's named
    /// pipe; it was refused before anything was sent.
    #[error(
        "the isolated credential broker's named pipe is held or served by another process \
         (possible pipe squatting), so Corbanu refused it and sent nothing; no request is sent \
         without the broker. Close that process and restart Corbanu"
    )]
    #[cfg_attr(
        not(any(unix, windows)),
        allow(dead_code, reason = "only the broker reports it")
    )]
    PipeSquatted,
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
    /// What turned the broker on, for refusal messages.
    origin: BrokerModelAuthOrigin,
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
            origin: BrokerModelAuthOrigin::Config,
        }
    }

    fn with_origin(mut self, origin: BrokerModelAuthOrigin) -> Self {
        self.origin = origin;
        self
    }

    /// A broker that did not start: every brokered request fails (#391).
    fn not_started(cause: StartFailure, origin: BrokerModelAuthOrigin) -> Self {
        Self::new(BrokerHandle::Failed(BrokerModelAuthError::NotStarted {
            cause,
            origin,
        }))
        .with_origin(origin)
    }

    /// The broker for platforms without one: every brokered request fails.
    #[cfg_attr(all(any(unix, windows), not(test)), allow(dead_code))]
    pub(crate) fn unsupported() -> Self {
        Self::new(BrokerHandle::Unsupported)
    }

    #[cfg(not(any(unix, windows)))]
    fn start(_settings: BrokerSettings) -> Self {
        // No fallback: the broker does not run on this platform (PF-27-S06).
        Self::unsupported()
    }

    #[cfg(any(unix, windows))]
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
                return Self::not_started(
                    match error {
                        codex_network_proxy::model_auth::ModelCredentialBrokerError::PipeSquatted => {
                            StartFailure::PipeSquatted
                        }
                        _ => StartFailure::Unavailable,
                    },
                    settings.origin,
                );
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
                return Self::not_started(StartFailure::Unavailable, settings.origin);
            }
        }
        #[cfg(unix)]
        match codex_http_client::HttpClient::unix_socket(
            broker.socket_path(),
            codex_login::default_client::default_headers(),
        ) {
            Ok(client) => codex_http_client::install_model_broker_client(client),
            // Brokered requests then fail in the transport.
            Err(error) => tracing::warn!("credential broker client: {error}"),
        }
        // PF-27-S09: the broker's data pipe, every connection checked to be
        // served by the broker process.
        #[cfg(windows)]
        codex_http_client::install_model_broker_sender(Arc::new(PipeSender {
            broker: broker.clone(),
            default_headers: codex_login::default_client::default_headers(),
        }));
        Self::new(BrokerHandle::Running(broker)).with_origin(settings.origin)
    }
}

impl ModelKeyBroker for CoreModelKeyBroker {
    fn unavailable_reason(&self) -> Option<&'static str> {
        let state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        match state.handle {
            #[cfg(any(unix, windows))]
            BrokerHandle::Running(_) => None,
            BrokerHandle::Failed(_) => {
                Some("the broker did not start; model requests are refused, never sent directly")
            }
            BrokerHandle::Unsupported => {
                Some("the broker does not run on this system; model requests are refused")
            }
        }
    }

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
        let binding = binding_for_base_url(&base_url, header).ok_or_else(|| {
            fatal(BrokerModelAuthError::Unbindable {
                url: base_url.clone(),
                origin: self.origin,
            })
        })?;
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
        #[cfg_attr(
            not(any(unix, windows)),
            allow(dead_code, reason = "only the broker reads it")
        )]
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
///
/// Without a broker (neither Unix nor Windows) `Credential` is uninhabited, so everything after
/// a successful registration is statically unreachable there.
#[cfg_attr(
    not(any(unix, windows)),
    allow(
        unreachable_code,
        unused_variables,
        clippy::redundant_clone,
        reason = "`Credential` is uninhabited without the broker"
    )
)]
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

#[cfg(any(unix, windows))]
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
        BrokerHandle::Failed(error) => return Err(error.clone()),
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
        ModelCredentialBrokerError::PipeSquatted => BrokerModelAuthError::PipeSquatted,
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

#[cfg(not(any(unix, windows)))]
fn register(
    handle: &BrokerHandle,
    _binding: &Binding,
    _source: &Source,
) -> Result<Credential, BrokerModelAuthError> {
    match handle {
        BrokerHandle::Failed(error) => Err(error.clone()),
        BrokerHandle::Unsupported => Err(BrokerModelAuthError::Unsupported),
    }
}

#[cfg(any(unix, windows))]
fn alive(credential: &Credential) -> bool {
    credential.is_alive()
}

#[cfg(not(any(unix, windows)))]
fn alive(credential: &Credential) -> bool {
    match *credential {}
}

#[cfg(any(unix, windows))]
fn unregister(credential: &Credential) {
    if let Err(error) = credential.unregister() {
        tracing::debug!("credential broker unregister: {error}");
    }
}

#[cfg(not(any(unix, windows)))]
fn unregister(credential: &Credential) {
    match *credential {}
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// Signs `request` for the broker and points it at the broker (the HTTP
/// transport sends a frame-bearing request only to the broker's socket).
#[cfg(any(unix, windows))]
fn broker_request(credential: &Credential, mut request: Request) -> Result<Request, AuthError> {
    use codex_http_client::MODEL_BROKER_FRAME_HEADER;
    use http::HeaderValue;
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

#[cfg(not(any(unix, windows)))]
fn broker_request(credential: &Credential, _request: Request) -> Result<Request, AuthError> {
    match *credential {}
}

/// The signed parts of an HTTPS URL and the plain-HTTP URL sent to the broker.
#[cfg(any(unix, windows, test))]
#[derive(Debug, PartialEq, Eq)]
struct BrokerRewrite {
    host: String,
    port: u16,
    path_and_query: String,
    broker_url: String,
}

#[cfg(any(unix, windows, test))]
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

/// PF-27-S09: sends frame-bearing requests to the Windows broker's data
/// pipe ([`ModelCredentialBroker::send`]).
///
/// [`ModelCredentialBroker::send`]: codex_network_proxy::model_auth::ModelCredentialBroker::send
#[cfg(windows)]
struct PipeSender {
    broker: codex_network_proxy::model_auth::ModelCredentialBroker,
    /// What the Unix socket client sends by default (user agent, originator).
    default_headers: HeaderMap,
}

#[cfg(windows)]
impl codex_http_client::ModelBrokerSender for PipeSender {
    fn send(
        &self,
        request: codex_http_client::ModelBrokerRequest,
    ) -> codex_http_client::ModelBrokerFuture {
        use codex_http_client::TransportError;
        use futures::StreamExt as _;
        let broker = self.broker.clone();
        let default_headers = self.default_headers.clone();
        Box::pin(async move {
            let codex_http_client::ModelBrokerRequest {
                method,
                url,
                mut headers,
                body,
            } = request;
            let (host, path_and_query) = pipe_request_target(&url)
                .ok_or_else(|| TransportError::Build("brokered request URL".to_string()))?;
            // What the Unix socket client sends by default, reqwest's
            // `Accept` included.
            for (name, value) in &default_headers {
                if !headers.contains_key(name) {
                    headers.insert(name.clone(), value.clone());
                }
            }
            if !headers.contains_key(http::header::ACCEPT) {
                headers.insert(http::header::ACCEPT, http::HeaderValue::from_static("*/*"));
            }
            let response = broker
                .send(codex_network_proxy::model_auth::ModelBrokerRequest {
                    method,
                    host,
                    path_and_query,
                    headers,
                    body,
                })
                .await
                .map_err(|error| TransportError::Network(format!("credential broker: {error}")))?;
            Ok(codex_http_client::ModelBrokerResponse {
                status: response.status,
                headers: response.headers,
                bytes: response
                    .body
                    .map(|chunk| chunk.map_err(|error| TransportError::Network(error.to_string())))
                    .boxed(),
            })
        })
    }
}

/// PF-27-S09: the `Host` value and origin-form target of a brokered request
/// URL (`http://host:port/path?query`, as [`broker_request`] rewrote it).
/// `None` for anything else, including a URL with user info, which the pipe
/// could not carry (reqwest would turn it into an `Authorization` header).
#[cfg_attr(
    not(windows),
    allow(dead_code, reason = "used by the Windows sender and tests")
)]
fn pipe_request_target(url: &str) -> Option<(String, String)> {
    let url = url::Url::parse(url).ok()?;
    if url.scheme() != "http"
        || !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
    {
        return None;
    }
    let host = match (url.host_str()?, url.port()) {
        (host, Some(port)) => format!("{host}:{port}"),
        (host, None) => host.to_string(),
    };
    let path_and_query = match url.query() {
        Some(query) => format!("{}?{query}", url.path()),
        None => url.path().to_string(),
    };
    Some((host, path_and_query))
}

#[cfg(test)]
#[path = "model_broker_auth_tests.rs"]
mod tests;
