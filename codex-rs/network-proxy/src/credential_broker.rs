#[cfg(unix)]
pub(crate) mod isolated;
mod providers;
mod resolver;

#[cfg(unix)]
pub use isolated::CODEX_CREDENTIAL_BROKER_ARG1;
#[cfg(unix)]
pub use isolated::run_credential_broker_main;

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
use rama_http::HeaderMap;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;
use zeroize::Zeroizing;

#[cfg(unix)]
use codex_secret_broker::CredentialReference as BrokerCredentialReference;
#[cfg(unix)]
use codex_secret_broker::ProviderRequestOperation;
#[cfg(unix)]
use isolated::IsolatedBrokerClient;
#[cfg(unix)]
use isolated::IsolatedBrokerError;
#[cfg(unix)]
use isolated::IsolatedBrokerLauncher;
#[cfg(unix)]
pub(crate) use isolated::IsolatedBrokerOptions;
#[cfg(unix)]
use isolated::protocol::HostBindingWire;
#[cfg(not(unix))]
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct IsolatedBrokerOptions {
    pub(crate) allow_local_binding: bool,
    pub(crate) allow_upstream_proxy: bool,
}

pub const CREDENTIAL_BROKER_ACTIVE_ENV_KEY: &str = "CODEX_NETWORK_PROXY_CREDENTIAL_BROKER_ACTIVE";
pub(crate) const BROKERED_CREDENTIALS_ENV_KEY: &str = "CODEX_NETWORK_PROXY_BROKERED_CREDENTIALS";

#[derive(Clone)]
pub(crate) struct CredentialBroker {
    state: Arc<RwLock<CredentialBrokerState>>,
    isolation: Isolation,
}

/// Where raw provider credential values live while the broker is enabled.
#[derive(Clone)]
enum Isolation {
    /// Legacy in-process injection: the proxy state holds raw values.
    InProcess,
    /// PF-27-S04: raw values live only in a separate broker process.
    #[cfg(unix)]
    Process(Arc<IsolatedMode>),
    /// Isolation was requested on a platform without the broker process.
    /// Credentials are virtualized but never injected (fail closed).
    #[cfg(not(unix))]
    Unsupported,
}

#[cfg(unix)]
struct IsolatedMode {
    launcher: IsolatedBrokerLauncher,
    options: IsolatedBrokerOptions,
    client: std::sync::Mutex<Option<Arc<IsolatedBrokerClient>>>,
    fingerprint_key: Zeroizing<[u8; 32]>,
}

/// Routing decision for one outbound request after credential policy.
pub(crate) enum CredentialRouting {
    /// Send upstream from the proxy (any legacy injection already applied).
    Direct,
    /// Send through the broker, which substitutes the credential itself.
    #[cfg(unix)]
    Brokered(BrokeredCredentialRoute),
}

#[cfg(unix)]
pub(crate) struct BrokeredCredentialRoute {
    client: Arc<IsolatedBrokerClient>,
    reference: BrokerCredentialReference,
    operation: ProviderRequestOperation,
}

#[cfg(unix)]
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
    #[cfg(unix)]
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
        #[cfg(unix)]
        {
            Self::new_isolated_with_launcher(
                enabled,
                options,
                IsolatedBrokerLauncher::current_exe(),
            )
        }
        #[cfg(not(unix))]
        {
            let _ = options;
            Self::with_isolation(enabled, Isolation::Unsupported)
        }
    }

    #[cfg(unix)]
    pub(crate) fn new_isolated_with_launcher(
        enabled: bool,
        options: IsolatedBrokerOptions,
        launcher: IsolatedBrokerLauncher,
    ) -> Self {
        let mut fingerprint_key = Zeroizing::new([0_u8; 32]);
        rand::Rng::fill(&mut rand::rng(), fingerprint_key.as_mut_slice());
        Self::with_isolation(
            enabled,
            Isolation::Process(Arc::new(IsolatedMode {
                launcher,
                options,
                client: std::sync::Mutex::new(None),
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
        }
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
        #[cfg(unix)]
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

        for provider in providers::credential_providers() {
            if (state.scoped_openai.is_some() || state.isolated_openai.is_some())
                && std::ptr::eq(provider, providers::openai_provider())
            {
                continue;
            }
            for source in provider.sources() {
                if let Some(host_binding) = (source.host_binding)(env) {
                    for env_var in source.env_vars {
                        virtualize_env_var(
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
            return Ok(CredentialRouting::Direct);
        }

        if let Some(scoped) = state.scoped_openai.as_mut() {
            let carries_reference = scoped.matches_reference(headers);
            if normalized_host == resolver::OPENAI_API_HOST || carries_reference {
                return scoped
                    .inject(scheme, &normalized_host, port, method, path, headers)
                    .map(|()| CredentialRouting::Direct);
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
            return Ok(CredentialRouting::Direct);
        };
        match &credential.secret {
            RecordSecret::Raw(real_value) => {
                if let Some(header_value) = credential
                    .provider
                    .request_header_value(real_value.as_str())
                {
                    credential
                        .provider
                        .insert_request_header(headers, header_value);
                }
                Ok(CredentialRouting::Direct)
            }
            #[cfg(unix)]
            RecordSecret::Brokered {
                reference, client, ..
            } => {
                if scheme != "https" {
                    return Err(ScopedCredentialInjectionError::SchemeDenied);
                }
                if !client.is_alive() {
                    return Err(ScopedCredentialInjectionError::IsolatedBrokerUnavailable);
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

    pub(crate) fn inject_request_headers(&self, host: &str, headers: &mut HeaderMap) {
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

fn virtualize_env_var(
    env: &mut HashMap<String, String>,
    state: &mut CredentialBrokerState,
    isolation: &Isolation,
    env_var: &str,
    provider: &'static providers::CredentialProvider,
    host_binding: providers::CredentialHostBinding,
) {
    let Some(real_value) = brokerable_credential_value(env, state, env_var, provider) else {
        return;
    };

    let dummy_value = match isolation {
        Isolation::InProcess => state.register(env_var, provider, host_binding, real_value),
        #[cfg(unix)]
        Isolation::Process(mode) => {
            state.register_isolated(mode, env_var, provider, host_binding, real_value)
        }
        #[cfg(not(unix))]
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

#[cfg(unix)]
impl IsolatedMode {
    /// Returns the current broker, starting a fresh one when the previous
    /// broker died. A fresh broker never inherits earlier references.
    fn live_client(&self) -> Option<Arc<IsolatedBrokerClient>> {
        let mut client = self
            .client
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(current) = client.as_ref()
            && current.is_alive()
        {
            return Some(current.clone());
        }
        *client = None;
        match IsolatedBrokerClient::spawn(&self.launcher, self.options) {
            Ok(spawned) => {
                let spawned = Arc::new(spawned);
                *client = Some(spawned.clone());
                Some(spawned)
            }
            Err(error) => {
                tracing::warn!("isolated credential broker unavailable: {error}");
                None
            }
        }
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

    #[cfg(unix)]
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
        let Some(client) = client else {
            return self.register_unavailable(
                fingerprint,
                env_var,
                provider,
                host_binding,
                real_value,
            );
        };
        let reference = match client.register(
            providers::provider_id(provider),
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

    fn fingerprint(&self) -> Option<[u8; 32]> {
        match &self.secret {
            RecordSecret::Raw(_) => None,
            #[cfg(unix)]
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
    ) -> Result<(), ScopedCredentialInjectionError> {
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
        let mut callback = |secret: &str| {
            let Some(header_value) = providers::scoped_openai_header_value(secret) else {
                return Err(ScopedCredentialCallbackError::Failed);
            };
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
        Ok(())
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

    #[cfg(unix)]
    pub(crate) fn current_isolated_client(&self) -> Option<Arc<IsolatedBrokerClient>> {
        match &self.isolation {
            Isolation::Process(mode) => mode
                .client
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone(),
            Isolation::InProcess => None,
        }
    }
}

#[cfg(test)]
#[path = "credential_broker_tests.rs"]
mod tests;
