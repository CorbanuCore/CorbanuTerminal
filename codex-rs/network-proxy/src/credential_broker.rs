mod providers;
mod resolver;

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

pub const CREDENTIAL_BROKER_ACTIVE_ENV_KEY: &str = "CODEX_NETWORK_PROXY_CREDENTIAL_BROKER_ACTIVE";
pub(crate) const BROKERED_CREDENTIALS_ENV_KEY: &str = "CODEX_NETWORK_PROXY_BROKERED_CREDENTIALS";

#[derive(Clone)]
pub(crate) struct CredentialBroker {
    state: Arc<RwLock<CredentialBrokerState>>,
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
    real_value: Zeroizing<String>,
    dummy_value: String,
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
        Self {
            state: Arc::new(RwLock::new(CredentialBrokerState {
                enabled,
                ..CredentialBrokerState::default()
            })),
        }
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
        self.read_state().scoped_openai.is_some()
    }

    pub(crate) fn scoped_openai_matches_host(&self, host: &str) -> bool {
        self.read_state().scoped_openai.is_some()
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
        let normalized_host = normalize_host(host);
        let mut state = self.write_state();
        if !state.enabled {
            return Ok(());
        }

        if let Some(scoped) = state.scoped_openai.as_mut() {
            let carries_reference = scoped.matches_reference(headers);
            if normalized_host == resolver::OPENAI_API_HOST || carries_reference {
                return scoped.inject(scheme, &normalized_host, port, method, path, headers);
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
            return Ok(());
        };
        let Some(header_value) = credential
            .provider
            .request_header_value(credential.real_value.as_str())
        else {
            return Ok(());
        };
        credential
            .provider
            .insert_request_header(headers, header_value);
        Ok(())
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
    env_var: &str,
    provider: &'static providers::CredentialProvider,
    host_binding: providers::CredentialHostBinding,
) {
    let Some(real_value) = brokerable_credential_value(env, state, env_var, provider) else {
        return;
    };

    let dummy_value = state.register(env_var, provider, host_binding, real_value);
    env.insert(env_var.to_string(), dummy_value);
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
                && credential.real_value.as_str() == real_value
        }) {
            return existing.dummy_value.clone();
        }

        let dummy_value = loop {
            let candidate = provider.dummy_value(real_value);
            if candidate != real_value && !self.is_dummy_value(&candidate) {
                break candidate;
            }
        };
        self.credentials.push(CredentialRecord {
            env_var: env_var.to_string(),
            provider,
            host_binding,
            real_value: Zeroizing::new(real_value.to_string()),
            dummy_value: dummy_value.clone(),
        });
        dummy_value
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
#[path = "credential_broker_tests.rs"]
mod tests;
