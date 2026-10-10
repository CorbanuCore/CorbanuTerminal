#[cfg(any(unix, windows))]
mod env_scrub;
#[cfg(any(unix, windows))]
pub(crate) mod isolated;
#[cfg(all(test, any(target_os = "linux", target_os = "macos", windows)))]
mod memory_scan_tests;
#[cfg(any(unix, windows))]
pub mod model_auth;
mod providers;
mod resolver;
pub(crate) mod response_scrub;

#[cfg(any(unix, windows))]
pub use isolated::CODEX_CREDENTIAL_BROKER_ARG1;
#[cfg(any(unix, windows))]
pub use isolated::StoredKeyAccount;
pub use isolated::StoredKeyResolver;
#[cfg(any(unix, windows))]
pub use isolated::run_credential_broker_main;
#[cfg(any(unix, windows))]
pub use isolated::run_credential_broker_main_with;

pub use resolver::IsolatedCredentialDispatchError;
pub use resolver::IsolatedCredentialDispatcher;
pub use resolver::IsolatedCredentialReceipt;
pub use resolver::IsolatedCredentialRoute;
pub use resolver::IsolatedCredentialUse;
pub use resolver::ScopedCredentialCallbackError;
pub use resolver::ScopedCredentialInjectionError;
pub use resolver::ScopedCredentialResolver;
pub use resolver::ScopedCredentialResolverError;
pub use resolver::ScopedCredentialRoute;
pub use resolver::ScopedCredentialRouteError;
pub use resolver::ScopedCredentialUse;

use crate::policy::normalize_host;
use codex_secret_broker::response_gate::ResponseGate;
use rama_http::HeaderMap;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;
use zeroize::Zeroizing;

#[cfg(any(unix, windows))]
use codex_secret_broker::CredentialReference as BrokerCredentialReference;
#[cfg(any(unix, windows))]
use codex_secret_broker::ProviderRequestOperation;
#[cfg(any(unix, windows))]
use isolated::IsolatedBrokerClient;
#[cfg(any(unix, windows))]
use isolated::IsolatedBrokerError;
#[cfg(any(unix, windows))]
use isolated::IsolatedBrokerLauncher;
#[cfg(any(unix, windows))]
pub(crate) use isolated::IsolatedBrokerOptions;
#[cfg(any(unix, windows))]
use isolated::protocol::HostBindingWire;
#[cfg(not(any(unix, windows)))]
#[allow(dead_code)]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct IsolatedBrokerOptions {
    pub(crate) allow_local_binding: bool,
    pub(crate) allow_upstream_proxy: bool,
    pub(crate) runtime_dir: Option<std::path::PathBuf>,
    pub(crate) require_containment: bool,
    pub(crate) scrub_responses: bool,
    pub(crate) pin_connections: bool,
}

pub const CREDENTIAL_BROKER_ACTIVE_ENV_KEY: &str = "CODEX_NETWORK_PROXY_CREDENTIAL_BROKER_ACTIVE";
pub(crate) const BROKERED_CREDENTIALS_ENV_KEY: &str = "CODEX_NETWORK_PROXY_BROKERED_CREDENTIALS";

#[derive(Clone)]
pub(crate) struct CredentialBroker {
    state: Arc<RwLock<CredentialBrokerState>>,
    isolation: Isolation,
    /// PF-27-S02: agent environments arrive without provider tokens, so raw
    /// values are read from Core's own process environment instead.
    process_env_source: Option<ProcessEnvSource>,
    /// PF-28-S02: bind credentials to HTTPS, method and path, and scrub
    /// injected values from responses (feature `secret_output_gate`).
    response_gate: bool,
}

/// Which variables may be read from Core's environment, and how.
#[derive(Clone)]
struct ProcessEnvSource {
    lookup: ProcessEnvLookup,
    /// Only the brokered variables the user's shell environment policy would
    /// have passed to agent commands before the launch allowlist ran.
    keys: Arc<[String]>,
}

/// Reads one variable from Core's environment (replaceable in tests).
type ProcessEnvLookup = fn(&str) -> Option<String>;

fn core_process_env(key: &str) -> Option<String> {
    std::env::var(key).ok()
}

/// Where raw provider credential values live while the broker is enabled.
#[derive(Clone)]
enum Isolation {
    /// Legacy in-process injection: the proxy state holds raw values.
    InProcess,
    /// PF-27-S04: raw values live only in a separate broker process.
    #[cfg(any(unix, windows))]
    Process(Arc<IsolatedMode>),
    /// Isolation was requested on a platform without the broker process.
    /// Credentials are virtualized but never injected (fail closed).
    #[cfg(not(any(unix, windows)))]
    Unsupported,
}

#[cfg(any(unix, windows))]
struct IsolatedMode {
    client: Option<Arc<IsolatedBrokerClient>>,
    fingerprint_key: Zeroizing<[u8; 32]>,
}

/// Routing decision for one outbound request after credential policy.
pub(crate) enum CredentialRouting {
    /// Send upstream from the proxy (any legacy injection already applied).
    /// PF-28-S02: the gate for the value injected here, if armed.
    Direct(Option<ResponseGate>),
    /// Send through the broker, which substitutes the credential itself.
    #[cfg(any(unix, windows))]
    Brokered(BrokeredCredentialRoute),
}

#[cfg(any(unix, windows))]
pub(crate) struct BrokeredCredentialRoute {
    client: Arc<IsolatedBrokerClient>,
    reference: BrokerCredentialReference,
    operation: ProviderRequestOperation,
}

#[cfg(any(unix, windows))]
impl BrokeredCredentialRoute {
    pub(crate) async fn forward(
        &self,
        request: rama_http::Request,
    ) -> Result<rama_http::Response, IsolatedBrokerError> {
        self.client
            .forward(&self.reference, &self.operation, request)
            .await
    }
}

#[derive(Default)]
struct CredentialBrokerState {
    enabled: bool,
    credentials: Vec<CredentialRecord>,
    scoped_openai: Option<ScopedCredentialRecord>,
    isolated_openai: Option<IsolatedCredentialRecord>,
}

struct CredentialRecord {
    env_var: String,
    provider: &'static providers::CredentialProvider,
    host_binding: providers::CredentialHostBinding,
    secret: RecordSecret,
    dummy_value: String,
}

enum RecordSecret {
    Raw(Zeroizing<String>),
    /// Opaque broker reference; the raw value is not retained in this process.
    #[cfg(any(unix, windows))]
    Brokered {
        fingerprint: [u8; 32],
        reference: BrokerCredentialReference,
        client: Arc<IsolatedBrokerClient>,
    },
    /// Isolation is active but the broker could not hold this value.
    Unavailable {
        fingerprint: [u8; 32],
    },
}

struct ScopedCredentialRecord {
    route: ScopedCredentialRoute,
    dummy_value: String,
    used: bool,
}

#[derive(Clone)]
struct IsolatedCredentialRecord {
    route: IsolatedCredentialRoute,
    dummy_value: String,
}

impl CredentialBroker {
    pub(crate) fn new(enabled: bool) -> Self {
        Self::with_isolation(enabled, Isolation::InProcess)
    }

    pub(crate) fn new_isolated(enabled: bool, options: IsolatedBrokerOptions) -> Self {
        #[cfg(any(unix, windows))]
        {
            Self::new_isolated_with_launcher(
                enabled,
                options,
                IsolatedBrokerLauncher::current_exe(),
            )
        }
        #[cfg(not(any(unix, windows)))]
        {
            let _ = options;
            Self::with_isolation(enabled, Isolation::Unsupported)
        }
    }

    #[cfg(any(unix, windows))]
    pub(crate) fn new_isolated_with_launcher(
        enabled: bool,
        options: IsolatedBrokerOptions,
        launcher: IsolatedBrokerLauncher,
    ) -> Self {
        let mut fingerprint_key = Zeroizing::new([0_u8; 32]);
        rand::Rng::fill(&mut rand::rng(), fingerprint_key.as_mut_slice());
        // Start the broker once per proxy (session), before this session spawns
        // agent processes, to narrow the window in which a concurrent spawn could
        // inherit its control pipes. It is never respawned: after it dies,
        // brokered credentials fail closed until a new session starts its proxy.
        let client = enabled
            .then(|| IsolatedBrokerClient::spawn(&launcher, options))
            .and_then(|spawned| match spawned {
                Ok(client) => Some(Arc::new(client)),
                Err(error) => {
                    tracing::warn!("isolated credential broker unavailable: {error}");
                    None
                }
            });
        Self::with_isolation(
            enabled,
            Isolation::Process(Arc::new(IsolatedMode {
                client,
                fingerprint_key,
            })),
        )
    }

    fn with_isolation(enabled: bool, isolation: Isolation) -> Self {
        Self {
            state: Arc::new(RwLock::new(CredentialBrokerState {
                enabled,
                ..CredentialBrokerState::default()
            })),
            isolation,
            process_env_source: None,
            response_gate: false,
        }
    }

    /// PF-28-S02: bind injected credentials to HTTPS, method and path and
    /// scrub them from responses.
    pub(crate) fn with_response_gate(mut self, enabled: bool) -> Self {
        self.response_gate = enabled;
        self
    }

    pub(crate) fn response_gate_enabled(&self) -> bool {
        self.response_gate
    }

    /// PF-27-S02: source brokered values from Core's environment rather than
    /// the (stripped) child environment.
    pub(crate) fn with_process_env_source(self, keys: Option<&[String]>) -> Self {
        self.with_process_env_lookup(keys, core_process_env)
    }

    pub(crate) fn with_process_env_lookup(
        mut self,
        keys: Option<&[String]>,
        lookup: ProcessEnvLookup,
    ) -> Self {
        self.process_env_source = keys.map(|keys| ProcessEnvSource {
            lookup,
            keys: keys.into(),
        });
        self
    }

    pub(crate) fn isolated(&self) -> bool {
        !matches!(self.isolation, Isolation::InProcess)
    }

    /// Revokes every brokered reference: the broker advances its generation,
    /// closes in-flight channels, and this process forgets the references.
    /// Returns false when isolation is not active or the broker is unavailable.
    pub(crate) fn revoke_isolated_credentials(&self) -> bool {
        let mut state = self.write_state();
        state
            .credentials
            .retain(|credential| matches!(credential.secret, RecordSecret::Raw(_)));
        #[cfg(any(unix, windows))]
        if let Isolation::Process(mode) = &self.isolation
            && let Some(client) = mode.live_client()
        {
            return client.revoke().is_ok();
        }
        false
    }

    pub(crate) fn enabled(&self) -> bool {
        self.read_state().enabled
    }

    pub(crate) fn virtualize_child_env(&self, env: &mut HashMap<String, String>) {
        let mut state = self.write_state();
        if !state.enabled {
            env.remove(CREDENTIAL_BROKER_ACTIVE_ENV_KEY);
            env.remove(BROKERED_CREDENTIALS_ENV_KEY);
            return;
        }
        env.insert(
            CREDENTIAL_BROKER_ACTIVE_ENV_KEY.to_string(),
            "1".to_string(),
        );

        // With secretless launch the child environment no longer carries the
        // provider tokens; read them (and host context) from Core's own
        // environment, letting any value already in the child win.
        let source_env = self
            .process_env_source
            .as_ref()
            .map(|source| process_credential_env(env, source));
        let source_env = source_env.as_ref().unwrap_or(env).clone();
        for provider in providers::credential_providers() {
            if (state.scoped_openai.is_some() || state.isolated_openai.is_some())
                && std::ptr::eq(provider, providers::openai_provider())
            {
                continue;
            }
            for source in provider.sources() {
                if let Some(host_binding) = (source.host_binding)(&source_env) {
                    for env_var in source.env_vars {
                        virtualize_env_var(
                            &source_env,
                            env,
                            &mut state,
                            &self.isolation,
                            env_var,
                            provider,
                            host_binding.clone(),
                        );
                    }
                }
            }
        }
        if let Some(scoped) = state.scoped_openai.as_ref() {
            env.insert(
                resolver::OPENAI_API_KEY_ENV_VAR.to_string(),
                scoped.dummy_value.clone(),
            );
        }
        if let Some(isolated) = state.isolated_openai.as_ref() {
            env.insert(
                resolver::OPENAI_API_KEY_ENV_VAR.to_string(),
                isolated.dummy_value.clone(),
            );
        }
        update_brokered_credentials_marker(&state, env);
    }

    pub(crate) fn install_scoped_openai_route(
        &self,
        route: ScopedCredentialRoute,
    ) -> Result<(), ScopedCredentialRouteError> {
        let mut state = self.write_state();
        if !state.enabled {
            return Err(ScopedCredentialRouteError::BrokerDisabled);
        }
        if state.scoped_openai.is_some() || state.isolated_openai.is_some() {
            return Err(ScopedCredentialRouteError::AlreadyConfigured);
        }
        state
            .credentials
            .retain(|credential| !std::ptr::eq(credential.provider, providers::openai_provider()));
        let dummy_value = providers::openai_provider().dummy_value(route.capability_id().as_str());
        state.scoped_openai = Some(ScopedCredentialRecord {
            route,
            dummy_value,
            used: false,
        });
        Ok(())
    }

    pub(crate) fn install_isolated_openai_route(
        &self,
        route: IsolatedCredentialRoute,
    ) -> Result<(), ScopedCredentialRouteError> {
        let mut state = self.write_state();
        if !state.enabled {
            return Err(ScopedCredentialRouteError::BrokerDisabled);
        }
        if state.scoped_openai.is_some() || state.isolated_openai.is_some() {
            return Err(ScopedCredentialRouteError::AlreadyConfigured);
        }
        state
            .credentials
            .retain(|credential| !std::ptr::eq(credential.provider, providers::openai_provider()));
        let dummy_value = providers::openai_provider().dummy_value(route.capability_id.as_str());
        state.isolated_openai = Some(IsolatedCredentialRecord { route, dummy_value });
        Ok(())
    }

    pub(crate) fn scoped_openai_enabled(&self) -> bool {
        let state = self.read_state();
        state.scoped_openai.is_some() || state.isolated_openai.is_some()
    }

    pub(crate) fn scoped_openai_matches_host(&self, host: &str) -> bool {
        let state = self.read_state();
        (state.scoped_openai.is_some() || state.isolated_openai.is_some())
            && normalize_host(host) == resolver::OPENAI_API_HOST
    }

    pub(crate) fn host_requires_mitm(&self, host: &str) -> bool {
        let normalized_host = normalize_host(host);
        let state = self.read_state();
        state.enabled
            && ((state.scoped_openai.is_some() || state.isolated_openai.is_some())
                && normalized_host == resolver::OPENAI_API_HOST
                || state
                    .credentials
                    .iter()
                    .any(|credential| credential.matches_host(&normalized_host)))
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn inject_request_headers_for_request(
        &self,
        scheme: &str,
        host: &str,
        port: u16,
        method: &str,
        path: &str,
        headers: &mut HeaderMap,
    ) -> Result<(), ScopedCredentialInjectionError> {
        // A brokered route cannot be honored by in-process callers; the request
        // proceeds with its dummy value and never receives the raw credential.
        self.route_request_credentials(scheme, host, port, method, path, headers)
            .map(|_| ())
    }

    /// Applies credential policy and decides whether the request must travel
    /// through the isolated broker process.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn route_request_credentials(
        &self,
        scheme: &str,
        host: &str,
        port: u16,
        method: &str,
        path: &str,
        headers: &mut HeaderMap,
    ) -> Result<CredentialRouting, ScopedCredentialInjectionError> {
        let normalized_host = normalize_host(host);
        let mut state = self.write_state();
        if !state.enabled {
            return Ok(CredentialRouting::Direct(None));
        }

        if let Some(scoped) = state.scoped_openai.as_mut() {
            let carries_reference = scoped.matches_reference(headers);
            if normalized_host == resolver::OPENAI_API_HOST || carries_reference {
                return scoped
                    .inject(
                        scheme,
                        &normalized_host,
                        port,
                        method,
                        path,
                        headers,
                        self.response_gate,
                    )
                    .map(CredentialRouting::Direct);
            }
        }
        if let Some(isolated) = state.isolated_openai.as_ref() {
            let carries_reference = isolated.matches_reference(headers);
            if normalized_host == resolver::OPENAI_API_HOST || carries_reference {
                return Err(ScopedCredentialInjectionError::IsolatedBrokerRequired);
            }
        }

        let matching_credentials = state
            .credentials
            .iter()
            .filter(|credential| credential.matches_host(&normalized_host))
            .collect::<Vec<_>>();
        let Some(credential) = select_credential(headers, &matching_credentials) else {
            return Ok(CredentialRouting::Direct(None));
        };
        if self.response_gate {
            credential
                .provider
                .allows_request(scheme, &normalized_host, port, method, path)?;
        }
        match &credential.secret {
            RecordSecret::Raw(real_value) => {
                let gate = self
                    .response_gate
                    .then(|| {
                        let label = format!("broker:{}", credential.env_var);
                        ResponseGate::new([(label.as_str(), real_value.as_str())])
                    })
                    .transpose()
                    .map_err(|_| ScopedCredentialInjectionError::ResolutionFailed)?;
                if let Some(header_value) = credential
                    .provider
                    .request_header_value(real_value.as_str())
                {
                    credential
                        .provider
                        .insert_request_header(headers, header_value);
                }
                Ok(CredentialRouting::Direct(gate))
            }
            #[cfg(any(unix, windows))]
            RecordSecret::Brokered {
                reference, client, ..
            } => {
                if scheme != "https" {
                    return Err(ScopedCredentialInjectionError::SchemeDenied);
                }
                if !client.is_alive() {
                    return Err(ScopedCredentialInjectionError::IsolatedBrokerUnavailable);
                }
                if !matches!(method, "GET" | "HEAD" | "POST" | "PUT" | "PATCH" | "DELETE") {
                    return Err(ScopedCredentialInjectionError::MethodDenied);
                }
                let operation =
                    ProviderRequestOperation::new(normalized_host.as_str(), port, method, path)
                        .map_err(|_| ScopedCredentialInjectionError::PathDenied)?;
                Ok(CredentialRouting::Brokered(BrokeredCredentialRoute {
                    client: client.clone(),
                    reference: reference.clone(),
                    operation,
                }))
            }
            RecordSecret::Unavailable { .. } => {
                Err(ScopedCredentialInjectionError::IsolatedBrokerUnavailable)
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn dispatch_isolated_openai(
        &self,
        scheme: &str,
        host: &str,
        port: u16,
        method: &str,
        path: &str,
    ) -> Result<IsolatedCredentialReceipt, IsolatedCredentialDispatchError> {
        let normalized_host = normalize_host(host);
        let isolated = self
            .read_state()
            .isolated_openai
            .clone()
            .ok_or(IsolatedCredentialDispatchError::Unavailable)?;
        isolated.dispatch(scheme, &normalized_host, port, method, path)
    }

    /// Plain-HTTP injection. PF-28-S02: never while credentials are bound to
    /// HTTPS.
    pub(crate) fn inject_request_headers(&self, host: &str, headers: &mut HeaderMap) {
        if self.response_gate {
            return;
        }
        let _ = self.inject_request_headers_for_request(
            "https",
            host,
            resolver::OPENAI_API_PORT,
            "POST",
            "/v1/responses",
            headers,
        );
    }

    fn read_state(&self) -> std::sync::RwLockReadGuard<'_, CredentialBrokerState> {
        self.state
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn write_state(&self) -> std::sync::RwLockWriteGuard<'_, CredentialBrokerState> {
        self.state
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

/// Core's own values for every broker-managed variable, overridden by any
/// value the child environment already carries (for example a dummy).
fn process_credential_env(
    child_env: &HashMap<String, String>,
    source: &ProcessEnvSource,
) -> HashMap<String, String> {
    let mut values = HashMap::new();
    for key in providers::credential_broker_env_keys() {
        let permitted = source.keys.iter().any(|permitted| permitted == key);
        let value = child_env
            .get(key)
            .cloned()
            .or_else(|| permitted.then(|| (source.lookup)(key)).flatten());
        if let Some(value) = value {
            values.insert(key.to_string(), value);
        }
    }
    values
}

fn virtualize_env_var(
    source_env: &HashMap<String, String>,
    env: &mut HashMap<String, String>,
    state: &mut CredentialBrokerState,
    isolation: &Isolation,
    env_var: &str,
    provider: &'static providers::CredentialProvider,
    host_binding: providers::CredentialHostBinding,
) {
    let Some(real_value) = brokerable_credential_value(source_env, state, env_var, provider) else {
        return;
    };

    let dummy_value = match isolation {
        Isolation::InProcess => state.register(env_var, provider, host_binding, real_value),
        #[cfg(any(unix, windows))]
        Isolation::Process(mode) => {
            state.register_isolated(mode, env_var, provider, host_binding, real_value)
        }
        #[cfg(not(any(unix, windows)))]
        Isolation::Unsupported => state.register_unavailable(
            fingerprint(&[0; 32], env_var, real_value),
            env_var,
            provider,
            host_binding,
            real_value,
        ),
    };
    env.insert(env_var.to_string(), dummy_value);
}

fn fingerprint(key: &[u8; 32], env_var: &str, value: &str) -> [u8; 32] {
    use sha2::Digest as _;
    let mut hasher = sha2::Sha256::new();
    hasher.update(key);
    hasher.update(env_var.as_bytes());
    hasher.update([0]);
    hasher.update(value.as_bytes());
    hasher.finalize().into()
}

#[cfg(any(unix, windows))]
impl IsolatedMode {
    /// Returns the broker while it is alive. A dead broker is not replaced.
    fn live_client(&self) -> Option<Arc<IsolatedBrokerClient>> {
        self.client
            .as_ref()
            .filter(|client| client.is_alive())
            .cloned()
    }
}

fn brokerable_credential_value<'a>(
    env: &'a HashMap<String, String>,
    state: &CredentialBrokerState,
    env_var: &str,
    provider: &providers::CredentialProvider,
) -> Option<&'a str> {
    let real_value = env.get(env_var)?.trim();
    (!real_value.is_empty()
        && !state.is_dummy_value(real_value)
        && provider.request_header_value(real_value).is_some())
    .then_some(real_value)
}

impl CredentialBrokerState {
    fn register(
        &mut self,
        env_var: &str,
        provider: &'static providers::CredentialProvider,
        host_binding: providers::CredentialHostBinding,
        real_value: &str,
    ) -> String {
        if let Some(existing) = self.credentials.iter().find(|credential| {
            credential.env_var == env_var
                && std::ptr::eq(credential.provider, provider)
                && credential.host_binding == host_binding
                && matches!(&credential.secret, RecordSecret::Raw(value) if value.as_str() == real_value)
        }) {
            return existing.dummy_value.clone();
        }

        let dummy_value = self.fresh_dummy(provider, real_value);
        self.credentials.push(CredentialRecord {
            env_var: env_var.to_string(),
            provider,
            host_binding,
            secret: RecordSecret::Raw(Zeroizing::new(real_value.to_string())),
            dummy_value: dummy_value.clone(),
        });
        dummy_value
    }

    #[cfg(any(unix, windows))]
    fn register_isolated(
        &mut self,
        mode: &IsolatedMode,
        env_var: &str,
        provider: &'static providers::CredentialProvider,
        host_binding: providers::CredentialHostBinding,
        real_value: &str,
    ) -> String {
        let fingerprint = fingerprint(&mode.fingerprint_key, env_var, real_value);
        let client = mode.live_client();
        // References from a dead or replaced broker are never reused.
        self.credentials
            .retain(|credential| match &credential.secret {
                RecordSecret::Brokered { client: owner, .. } => client
                    .as_ref()
                    .is_some_and(|current| Arc::ptr_eq(current, owner)),
                RecordSecret::Raw(_) | RecordSecret::Unavailable { .. } => true,
            });
        if let Some(existing) = self.credentials.iter().position(|credential| {
            credential.env_var == env_var
                && std::ptr::eq(credential.provider, provider)
                && credential.host_binding == host_binding
                && credential.fingerprint() == Some(fingerprint)
        }) {
            if matches!(
                self.credentials[existing].secret,
                RecordSecret::Brokered { .. }
            ) || client.is_none()
            {
                return self.credentials[existing].dummy_value.clone();
            }
            self.credentials.remove(existing);
        }
        let (Some(client), Some(provider_id)) = (client, providers::provider_id(provider)) else {
            return self.register_unavailable(
                fingerprint,
                env_var,
                provider,
                host_binding,
                real_value,
            );
        };
        let reference = match client.register(
            provider_id,
            HostBindingWire::from_binding(&host_binding),
            real_value,
        ) {
            Ok(reference) => reference,
            Err(error) => {
                tracing::warn!("isolated credential broker rejected a credential: {error}");
                return self.register_unavailable(
                    fingerprint,
                    env_var,
                    provider,
                    host_binding,
                    real_value,
                );
            }
        };
        let dummy_value = self.fresh_dummy(provider, real_value);
        self.credentials.push(CredentialRecord {
            env_var: env_var.to_string(),
            provider,
            host_binding,
            secret: RecordSecret::Brokered {
                fingerprint,
                reference,
                client,
            },
            dummy_value: dummy_value.clone(),
        });
        dummy_value
    }

    fn register_unavailable(
        &mut self,
        fingerprint: [u8; 32],
        env_var: &str,
        provider: &'static providers::CredentialProvider,
        host_binding: providers::CredentialHostBinding,
        real_value: &str,
    ) -> String {
        let dummy_value = self.fresh_dummy(provider, real_value);
        self.credentials.push(CredentialRecord {
            env_var: env_var.to_string(),
            provider,
            host_binding,
            secret: RecordSecret::Unavailable { fingerprint },
            dummy_value: dummy_value.clone(),
        });
        dummy_value
    }

    fn fresh_dummy(
        &self,
        provider: &'static providers::CredentialProvider,
        real_value: &str,
    ) -> String {
        loop {
            let candidate = provider.dummy_value(real_value);
            if candidate != real_value && !self.is_dummy_value(&candidate) {
                break candidate;
            }
        }
    }

    fn is_dummy_value(&self, value: &str) -> bool {
        self.credentials
            .iter()
            .any(|credential| credential.dummy_value == value)
            || self
                .scoped_openai
                .as_ref()
                .is_some_and(|credential| credential.dummy_value == value)
            || self
                .isolated_openai
                .as_ref()
                .is_some_and(|credential| credential.dummy_value == value)
    }
}

impl CredentialRecord {
    fn matches_host(&self, host: &str) -> bool {
        self.host_binding.matches_host(host)
    }

    #[cfg(any(unix, windows))]
    fn fingerprint(&self) -> Option<[u8; 32]> {
        match &self.secret {
            RecordSecret::Raw(_) => None,
            #[cfg(any(unix, windows))]
            RecordSecret::Brokered { fingerprint, .. } => Some(*fingerprint),
            RecordSecret::Unavailable { fingerprint } => Some(*fingerprint),
        }
    }
}

impl IsolatedCredentialRecord {
    fn matches_reference(&self, headers: &HeaderMap) -> bool {
        providers::openai_provider()
            .request_header(headers)
            .and_then(|value| value.to_str().ok())
            .is_some_and(|value| value == format!("Bearer {}", self.dummy_value))
    }

    fn dispatch(
        &self,
        scheme: &str,
        host: &str,
        port: u16,
        method: &str,
        path: &str,
    ) -> Result<IsolatedCredentialReceipt, IsolatedCredentialDispatchError> {
        if scheme != "https"
            || host != resolver::OPENAI_API_HOST
            || port != resolver::OPENAI_API_PORT
            || method != "POST"
            || path != self.route.authority.path.as_str()
        {
            return Err(IsolatedCredentialDispatchError::Denied);
        }
        self.route.dispatcher.dispatch(&IsolatedCredentialUse {
            scheme,
            host,
            port,
            method,
            path,
            capability_id: &self.route.capability_id,
            authority: &self.route.authority,
        })
    }
}

impl ScopedCredentialRecord {
    fn matches_reference(&self, headers: &HeaderMap) -> bool {
        providers::openai_provider()
            .request_header(headers)
            .and_then(|value| value.to_str().ok())
            .is_some_and(|value| value == format!("Bearer {}", self.dummy_value))
    }

    #[allow(clippy::too_many_arguments)]
    fn inject(
        &mut self,
        scheme: &str,
        host: &str,
        port: u16,
        method: &str,
        path: &str,
        headers: &mut HeaderMap,
        response_gate: bool,
    ) -> Result<Option<ResponseGate>, ScopedCredentialInjectionError> {
        if scheme != "https" {
            return Err(ScopedCredentialInjectionError::SchemeDenied);
        }
        if host != resolver::OPENAI_API_HOST {
            return Err(ScopedCredentialInjectionError::HostDenied);
        }
        if port != resolver::OPENAI_API_PORT {
            return Err(ScopedCredentialInjectionError::PortDenied);
        }
        if method != "POST" {
            return Err(ScopedCredentialInjectionError::MethodDenied);
        }
        if !path.starts_with(resolver::OPENAI_API_PATH_PREFIX)
            || path != self.route.authority().path.as_str()
        {
            return Err(ScopedCredentialInjectionError::PathDenied);
        }
        let Some(authorization) = providers::openai_provider()
            .request_header(headers)
            .and_then(|value| value.to_str().ok())
        else {
            return Err(ScopedCredentialInjectionError::MissingReference);
        };
        if authorization != format!("Bearer {}", self.dummy_value) {
            return Err(ScopedCredentialInjectionError::AuthorizationConflict);
        }
        if self.used {
            return Err(ScopedCredentialInjectionError::AlreadyUsed);
        }

        let use_request = ScopedCredentialUse {
            scheme,
            host,
            port,
            method,
            path,
            capability_id: self.route.capability_id(),
            authority: self.route.authority(),
        };
        let mut inserted = false;
        let mut gate = None;
        let mut callback = |secret: &str| {
            let Some(header_value) = providers::scoped_openai_header_value(secret) else {
                return Err(ScopedCredentialCallbackError::Failed);
            };
            if response_gate {
                gate = Some(
                    ResponseGate::new([("broker:OPENAI_API_KEY", secret)])
                        .map_err(|_| ScopedCredentialCallbackError::Failed)?,
                );
            }
            providers::openai_provider().insert_request_header(headers, header_value);
            inserted = true;
            Ok(())
        };
        self.route
            .resolver()
            .resolve(&use_request, &mut callback)
            .map_err(|_| ScopedCredentialInjectionError::ResolutionFailed)?;
        if !inserted {
            return Err(ScopedCredentialInjectionError::ResolutionFailed);
        }
        self.used = true;
        Ok(gate)
    }
}

fn select_credential<'a>(
    headers: &HeaderMap,
    matching_credentials: &[&'a CredentialRecord],
) -> Option<&'a CredentialRecord> {
    let dummy_matches = matching_credentials
        .iter()
        .copied()
        .filter(|credential| {
            credential
                .provider
                .request_header(headers)
                .and_then(|value| value.to_str().ok())
                .is_some_and(|value| value.contains(&credential.dummy_value))
        })
        .collect::<Vec<_>>();
    match dummy_matches.as_slice() {
        [credential] => Some(*credential),
        [] | [_, _, ..] => None,
    }
}

fn update_brokered_credentials_marker(
    state: &CredentialBrokerState,
    env: &mut HashMap<String, String>,
) {
    let brokered = providers::credential_broker_env_keys()
        .filter_map(|key| {
            let value = env.get(key)?;
            state.is_dummy_value(value).then_some((key, value.as_str()))
        })
        .collect::<Vec<_>>();
    match serde_json::to_string(&brokered) {
        Ok(marker) => {
            env.insert(BROKERED_CREDENTIALS_ENV_KEY.to_string(), marker);
        }
        Err(_) => {
            env.remove(BROKERED_CREDENTIALS_ENV_KEY);
        }
    }
}

/// Returns supported environment keys whose current values still match the child-scoped dummy
/// values recorded by the credential broker.
///
/// The broker marker is treated as untrusted: malformed metadata, unsupported keys, and values
/// replaced by the user are ignored. The environment is not mutated; callers own the decision to
/// remove the returned keys.
pub fn brokered_credential_dummy_env_keys(env: &HashMap<String, String>) -> Vec<String> {
    env.get(BROKERED_CREDENTIALS_ENV_KEY)
        .and_then(|marker| serde_json::from_str::<Vec<(String, String)>>(marker).ok())
        .unwrap_or_default()
        .into_iter()
        .filter_map(|(key, dummy_value)| {
            (providers::credential_broker_env_keys().any(|candidate| candidate == key.as_str())
                && env.get(&key) == Some(&dummy_value))
            .then_some(key)
        })
        .collect()
}

/// Every environment variable the credential broker manages (tokens and host
/// context), for callers that decide which ones a launch may source.
pub fn credential_broker_env_var_names() -> Vec<&'static str> {
    providers::credential_broker_env_keys().collect()
}

/// PF-27-S02: the per-user directory the isolated broker falls back to when
/// `CODEX_HOME/run` is too long for a socket path.
pub fn credential_broker_user_runtime_dir() -> Option<std::path::PathBuf> {
    #[cfg(any(unix, windows))]
    {
        isolated::user_runtime_dir()
    }
    #[cfg(not(any(unix, windows)))]
    {
        None
    }
}

/// Returns supported credential keys only for an environment with an active broker.
pub fn brokered_credential_env_keys(
    env: &HashMap<String, String>,
) -> impl Iterator<Item = &'static str> {
    let active = env
        .get(CREDENTIAL_BROKER_ACTIVE_ENV_KEY)
        .is_some_and(|value| value == "1");
    providers::credential_broker_env_keys().filter(move |_| active)
}

#[cfg(test)]
impl CredentialBroker {
    /// True when this process still retains `value` as raw credential material.
    pub(crate) fn holds_raw_value(&self, value: &str) -> bool {
        self.read_state()
            .credentials
            .iter()
            .any(|credential| matches!(&credential.secret, RecordSecret::Raw(raw) if raw.as_str() == value))
    }

    #[cfg(any(unix, windows))]
    pub(crate) fn current_isolated_client(&self) -> Option<Arc<IsolatedBrokerClient>> {
        match &self.isolation {
            Isolation::Process(mode) => mode.client.clone(),
            Isolation::InProcess => None,
        }
    }
}

#[cfg(test)]
#[path = "credential_broker_tests.rs"]
mod tests;
