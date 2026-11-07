<<<<<<< HEAD
#[cfg(any(unix, windows))]
mod env_scrub;
#[cfg(any(unix, windows))]
pub(crate) mod isolated;
#[cfg(all(test, any(target_os = "linux", target_os = "macos", windows)))]
mod memory_scan_tests;
#[cfg(any(unix, windows))]
pub mod model_auth;
||||||| 413492cd6c
=======
mod configured;
mod destination;
mod environment;
mod matching;
mod provider_config;
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
mod providers;
<<<<<<< HEAD
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
||||||| 413492cd6c
=======
mod registry;
mod replacement;
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a

use crate::config::NetworkProxyConfig;
use crate::policy::normalize_host;
<<<<<<< HEAD
use codex_secret_broker::response_gate::ResponseGate;
||||||| 413492cd6c
=======
use environment::update_brokered_credentials_marker;
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
use rama_http::HeaderMap;
use registry::ActiveCredentialSource;
use registry::BrokeredCredentialProvider;
use registry::active_credential_sources;
use registry::is_builtin_shaped_credential;
use registry::prioritized_credentials;
use registry::registered_credential_source;
use registry::select_credentials;
use replacement::Replacements;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::RwLock;
<<<<<<< HEAD
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
||||||| 413492cd6c
=======
use url::Url;

pub use environment::CredentialBrokerContext;
pub use environment::CredentialBrokerEnvironment;
pub use environment::brokered_credential_binding_env_keys;
pub use environment::brokered_credential_dummy_env_keys;
pub use environment::brokered_credential_env_keys;
pub use environment::brokered_credential_marker_env_keys;
pub use environment::brokered_credential_value_env_keys;
pub use environment::credential_broker_provider_context_env_keys;
pub use environment::credential_broker_provider_sources_allowed;
pub use environment::is_credential_broker_provider_env_key;
pub(crate) use environment::marked_credential_dummy_env_keys;
pub use provider_config::CredentialAuthMethod;
pub use provider_config::CredentialProviderConfig;
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a

pub const CREDENTIAL_BROKER_ACTIVE_ENV_KEY: &str = "CODEX_NETWORK_PROXY_CREDENTIAL_BROKER_ACTIVE";
pub(crate) const BROKERED_CREDENTIALS_ENV_KEY: &str = "CODEX_NETWORK_PROXY_BROKERED_CREDENTIALS";
const MIN_EMBEDDED_CREDENTIAL_LENGTH: usize = 16;
const BROKERED_CREDENTIAL_ALIAS_MARKER_PREFIX: &str = "@alias:";

/// How to handle recognized credentials left raw in a prepared command environment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CredentialMaskingFailureMode {
    /// Warn and keep the unmasked values.
    Warn,
    /// Remove variables containing unmasked values, including copied aliases.
    Deny,
    /// Reject the command environment.
    Error,
}

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
    config_revision: u64,
    enabled: bool,
    allow_local_binding: bool,
    openai_api_host: Option<String>,
    context: CredentialBrokerContext,
    configured_provider_configs: BTreeMap<String, CredentialProviderConfig>,
    configured_providers: Vec<Arc<configured::ConfiguredCredentialProvider>>,
    credentials: Vec<CredentialRecord>,
<<<<<<< HEAD
    scoped_openai: Option<ScopedCredentialRecord>,
    isolated_openai: Option<IsolatedCredentialRecord>,
||||||| 413492cd6c
=======
    credential_owners: Vec<CredentialOwner>,
    credential_aliases: Vec<CredentialAlias>,
}

struct CredentialOwner {
    env_var: String,
    real_value: String,
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
}

struct CredentialRecord {
    env_var: String,
    provider: BrokeredCredentialProvider,
    host_binding: providers::CredentialHostBinding,
<<<<<<< HEAD
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
||||||| 413492cd6c
    real_value: String,
=======
    additional_host_bindings: Vec<providers::CredentialHostBinding>,
    fallback_host_bindings: Vec<providers::CredentialHostBinding>,
    environment_id: Option<String>,
    real_value: String,
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
    dummy_value: String,
    // Canonical identities accompanying discovery, distinct from an alias's own credential.
    source_values: Vec<String>,
    generated_aliases: Arc<RwLock<Vec<replacement::GeneratedAlias>>>,
}

impl CredentialRecord {
    fn has_carried_identity(
        &self,
        env: &HashMap<String, String>,
        credentials: &[CredentialRecord],
        sources: &[ActiveCredentialSource],
    ) -> bool {
        let mut source_present = false;
        let source_matches = sources
            .iter()
            .filter(|source| source_accepts_credential(source, self))
            .flat_map(|source| &source.env_vars)
            .filter_map(|key| env_value(env, key))
            .any(|value| {
                source_present = true;
                let value = match &self.provider {
                    BrokeredCredentialProvider::Builtin(_) => value.trim(),
                    BrokeredCredentialProvider::Configured(_) => value,
                };
                let value = credentials
                    .iter()
                    .find(|source| source.dummy_value == value)
                    .map_or(value, |source| source.real_value.as_str());
                value == self.real_value
                    || value == self.dummy_value
                    || self.source_values.iter().any(|source| source == value)
            });
        let source_changed = source_present && !source_matches;
        env_contains_credential_value(env, &self.dummy_value)
            || (source_changed || self.invalidates_host_binding(env))
                && env.iter().any(|(key, value)| {
                    !env_key_matches(key, CREDENTIAL_BROKER_ACTIVE_ENV_KEY)
                        && !env_key_matches(key, BROKERED_CREDENTIALS_ENV_KEY)
                        && !crate::is_managed_proxy_env_var(key, value)
                        && (value == &self.real_value
                            || self.contains_embedded_value(value, &self.real_value))
                })
    }

    fn invalidates_host_binding(&self, env: &HashMap<String, String>) -> bool {
        match &self.provider {
            BrokeredCredentialProvider::Builtin(provider) => {
                provider.sources().iter().any(|source| {
                    source
                        .env_vars
                        .iter()
                        .any(|key| env_key_matches(key, &self.env_var))
                        && (source.invalidates_host_binding)(env)
                })
            }
            BrokeredCredentialProvider::Configured(provider) => {
                provider
                    .config
                    .url_prefix_from_env
                    .as_deref()
                    .is_some_and(|key| env_value(env, key).is_some())
                    && provider.dynamic_destination(env).is_none()
            }
        }
    }

    fn observe_host_binding(
        &mut self,
        host_binding: providers::CredentialHostBinding,
        env: &HashMap<String, String>,
    ) {
        if self.invalidates_host_binding(env) {
            self.additional_host_bindings.clear();
            self.fallback_host_bindings.clear();
            self.host_binding = host_binding;
            return;
        }
        if self.host_binding == host_binding {
            return;
        }
        // Track the latest source separately, but keep prior destinations usable by
        // concurrent commands sharing this dummy in the same environment.
        self.additional_host_bindings
            .retain(|binding| binding != &host_binding);
        self.additional_host_bindings
            .push(std::mem::replace(&mut self.host_binding, host_binding));
    }

    fn host_bindings(&self) -> impl Iterator<Item = &providers::CredentialHostBinding> {
        std::iter::once(&self.host_binding).chain(&self.additional_host_bindings)
    }

    fn belongs_to_environment(&self, environment_id: Option<&str>) -> bool {
        self.environment_id.as_deref() == environment_id
    }
}

fn source_accepts_credential(
    source: &ActiveCredentialSource,
    credential: &CredentialRecord,
) -> bool {
    credential.provider.same_provider(&source.provider)
        && source
            .env_vars
            .iter()
            .any(|env_var| env_key_matches(env_var, &credential.env_var))
}

fn source_tracks_credential(
    source: &ActiveCredentialSource,
    credential: &CredentialRecord,
    env: &HashMap<String, String>,
) -> bool {
    if !source_accepts_credential(source, credential) {
        return false;
    }
    // Aliases may hold a different credential from the canonical source.
    if source.host_binding == credential.host_binding {
        return true;
    }
    let mut source_present = false;
    let contains_credential = source
        .env_vars
        .iter()
        .filter_map(|env_var| env_value(env, env_var))
        .any(|value| {
            source_present = true;
            let value = match &source.provider {
                BrokeredCredentialProvider::Builtin(_) => value.trim(),
                BrokeredCredentialProvider::Configured(_) => value,
            };
            value == credential.dummy_value
                || value == credential.real_value
                || credential
                    .source_values
                    .iter()
                    .any(|source| source == value)
        });
    !source_present || contains_credential
}

struct CredentialAlias {
    env_var: String,
    dummy_value: String,
}

enum CredentialEnvironment {
    Child,
    Snapshot,
}

fn env_key_matches(candidate: &str, expected: &str) -> bool {
    if cfg!(windows) {
        candidate.eq_ignore_ascii_case(expected)
    } else {
        candidate == expected
    }
}

fn env_entry<'a>(env: &'a HashMap<String, String>, key: &str) -> Option<(&'a str, &'a str)> {
    env.iter()
        .find(|(candidate, _)| env_key_matches(candidate, key))
        .map(|(key, value)| (key.as_str(), value.as_str()))
}

pub(super) fn env_value<'a>(env: &'a HashMap<String, String>, key: &str) -> Option<&'a str> {
    env_entry(env, key).map(|(_, value)| value)
}

pub(super) fn set_env_value(env: &mut HashMap<String, String>, key: &str, value: String) {
    if cfg!(windows) {
        env.retain(|candidate, _| !env_key_matches(candidate, key));
    }
    env.insert(key.to_string(), value);
}

pub(super) fn remove_env_value(env: &mut HashMap<String, String>, key: &str) {
    if cfg!(windows) {
        env.retain(|candidate, _| !env_key_matches(candidate, key));
    } else {
        env.remove(key);
    }
}

fn env_contains_credential_value(env: &HashMap<String, String>, credential: &str) -> bool {
    env.iter().any(|(key, value)| {
        !env_key_matches(key, CREDENTIAL_BROKER_ACTIVE_ENV_KEY)
            && !env_key_matches(key, BROKERED_CREDENTIALS_ENV_KEY)
            && value.contains(credential)
            && !crate::is_managed_proxy_env_var(key, value)
    })
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

<<<<<<< HEAD
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
||||||| 413492cd6c
    pub(crate) fn enabled(&self) -> bool {
        self.read_state().enabled
=======
    pub(crate) fn configure(&self, config: &NetworkProxyConfig) {
        let mut state = self.write_state();
        let previous_context_sources = (state.context != config.credential_broker_context)
            .then(|| active_credential_sources(&state, &HashMap::new()));
        if state.enabled != config.credential_broker
            || state.openai_api_host != config.credential_broker_openai_host
            || state.allow_local_binding != config.allow_local_binding()
            || state.configured_provider_configs != config.credential_providers
            || previous_context_sources.is_some()
        {
            state.config_revision += 1;
        }
        state.context.clone_from(&config.credential_broker_context);
        state.allow_local_binding = config.allow_local_binding();
        if state.enabled != config.credential_broker {
            state.enabled = config.credential_broker;
            state.credentials.clear();
            state.credential_owners.clear();
            state.credential_aliases.clear();
        }
        if state.openai_api_host != config.credential_broker_openai_host {
            state
                .openai_api_host
                .clone_from(&config.credential_broker_openai_host);
            state.credentials.retain(|credential| {
                !matches!(
                    &credential.provider,
                    BrokeredCredentialProvider::Builtin(provider)
                        if provider.reset_on_configuration_change
                )
            });
        }
        if state.configured_provider_configs != config.credential_providers {
            let mut configured_providers: Vec<Arc<configured::ConfiguredCredentialProvider>> =
                Vec::new();
            let mut provider_configs = config.credential_providers.iter().collect::<Vec<_>>();
            provider_configs.sort_by_key(|(id, provider_config)| {
                std::cmp::Reverse(
                    state.configured_provider_configs.get(id.as_str()) == Some(*provider_config)
                        && state
                            .configured_providers
                            .iter()
                            .any(|provider| provider.id == id.as_str()),
                )
            });
            for (id, provider_config) in provider_configs {
                if configured_providers.iter().any(|existing| {
                    existing.config.env.iter().any(|existing_key| {
                        provider_config
                            .env
                            .iter()
                            .any(|key| env_key_matches(existing_key, key))
                    })
                }) {
                    tracing::warn!(
                        provider = %id,
                        "ignoring credential provider with an overlapping environment source"
                    );
                    continue;
                }

                let existing = (state.configured_provider_configs.get(id) == Some(provider_config))
                    .then(|| {
                        state
                            .configured_providers
                            .iter()
                            .find(|provider| provider.id == *id)
                            .cloned()
                    })
                    .flatten();
                match existing {
                    Some(provider) => configured_providers.push(provider),
                    None => {
                        match configured::ConfiguredCredentialProvider::compile(id, provider_config)
                        {
                            Ok(provider) => configured_providers.push(Arc::new(provider)),
                            Err(error) => {
                                tracing::warn!(provider = %id, %error, "ignoring invalid credential provider");
                            }
                        }
                    }
                }
            }
            state
                .credentials
                .retain(|credential| match &credential.provider {
                    BrokeredCredentialProvider::Builtin(_) => true,
                    BrokeredCredentialProvider::Configured(provider) => configured_providers
                        .iter()
                        .any(|current| Arc::ptr_eq(current, provider)),
                });
            state.configured_providers = configured_providers;
            state
                .configured_provider_configs
                .clone_from(&config.credential_providers);
        }
        if let Some(previous_sources) = previous_context_sources {
            // Update registrations that used the old fallback, leaving distinct captured bindings intact.
            let context = state.context.with_fallbacks(&HashMap::new()).into_owned();
            let sources = active_credential_sources(&state, &context);
            state.credentials.retain_mut(|credential| {
                let Some(previous_source) = previous_sources.iter().find(|source| {
                    source_accepts_credential(source, credential)
                        && credential
                            .host_bindings()
                            .any(|binding| binding == &source.host_binding)
                }) else {
                    return true;
                };
                let source = sources
                    .iter()
                    .find(|source| source_accepts_credential(source, credential));
                if source.is_some_and(|source| source.host_binding == previous_source.host_binding)
                {
                    return true;
                }
                if !credential
                    .fallback_host_bindings
                    .contains(&previous_source.host_binding)
                {
                    credential
                        .fallback_host_bindings
                        .push(previous_source.host_binding.clone());
                }
                if source.is_none() || credential.invalidates_host_binding(&context) {
                    // Clearing a fallback revokes its history, not independently captured hosts.
                    credential
                        .additional_host_bindings
                        .retain(|binding| !credential.fallback_host_bindings.contains(binding));
                    if credential
                        .fallback_host_bindings
                        .contains(&credential.host_binding)
                    {
                        let replacement = source
                            .map(|source| source.host_binding.clone())
                            .or_else(|| credential.additional_host_bindings.pop());
                        let Some(replacement) = replacement else {
                            return false;
                        };
                        credential.host_binding = replacement;
                    }
                    credential.fallback_host_bindings.clear();
                }
                if credential.host_binding == previous_source.host_binding {
                    let Some(source) = source else {
                        return false;
                    };
                    credential.observe_host_binding(source.host_binding.clone(), &context);
                } else {
                    // A captured primary destination must stay primary when its older fallback changes.
                    if let Some(source) = source
                        && !credential
                            .host_bindings()
                            .any(|binding| binding == &source.host_binding)
                    {
                        credential
                            .additional_host_bindings
                            .push(source.host_binding.clone());
                    }
                }
                if let Some(source) = source
                    && !credential
                        .fallback_host_bindings
                        .contains(&source.host_binding)
                {
                    credential
                        .fallback_host_bindings
                        .push(source.host_binding.clone());
                }
                true
            });
        }
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
    }

    pub(crate) fn config_revision(&self) -> u64 {
        self.read_state().config_revision
    }

    #[cfg(test)]
    pub(crate) fn discover_parent_credentials(
        &self,
        parent_env: &HashMap<String, String>,
        child_env: &HashMap<String, String>,
    ) {
        self.discover_parent_credentials_for_environment(
            parent_env, child_env, /*environment_id*/ None,
        );
    }

    pub(crate) fn discover_parent_credentials_for_environment(
        &self,
        parent_env: &HashMap<String, String>,
        child_env: &HashMap<String, String>,
        environment_id: Option<&str>,
    ) {
        let mut state = self.write_state();
        if !state.enabled {
            return;
        }
        state.observe_credential_owners(parent_env);

<<<<<<< HEAD
        // With secretless launch the child environment no longer carries the
        // provider tokens; read them (and host context) from Core's own
        // environment, letting any value already in the child win.
        let source_env = self
            .process_env_source
            .as_ref()
            .map(|source| process_credential_env(env, source));
        let source_env = source_env.as_ref().unwrap_or(env).clone();
||||||| 413492cd6c
=======
        let active_sources = active_credential_sources(&state, child_env);
        for source in &active_sources {
            for env_var in &source.env_vars {
                let Some(real_value) =
                    brokerable_credential_value(parent_env, &state, env_var, &source.provider)
                        .map(str::to_string)
                else {
                    continue;
                };
                // Reconcile carried identities in virtualize_env, preserving their registered
                // destinations instead of binding the parent's value to a rotated child source.
                if state.credentials.iter().any(|credential| {
                    env_key_matches(&credential.env_var, env_var)
                        && credential.provider.same_provider(&source.provider)
                        && credential.real_value == real_value
                        && credential.has_carried_identity(
                            child_env,
                            &state.credentials,
                            &active_sources,
                        )
                }) {
                    continue;
                }
                if env_value(child_env, env_var) != Some(real_value.as_str())
                    && child_env.values().any(|value| {
                        value == &real_value
                            || is_builtin_shaped_credential(&real_value)
                                && value.contains(&real_value)
                            || source.provider.contains_embedded_value(value, &real_value)
                    })
                {
                    let _ = state.register(
                        env_var,
                        source.provider.clone(),
                        source.host_binding.clone(),
                        environment_id,
                        &real_value,
                        parent_env,
                    );
                }
            }
        }
    }

    #[cfg(test)]
    pub(crate) fn virtualize_child_env(&self, env: &mut HashMap<String, String>) {
        self.virtualize_child_env_for_environment(env, /*environment_id*/ None);
    }

    pub(crate) fn virtualize_child_env_for_environment(
        &self,
        env: &mut HashMap<String, String>,
        environment_id: Option<&str>,
    ) {
        self.virtualize_env(env, environment_id, CredentialEnvironment::Child);
    }

    pub(crate) fn virtualize_snapshot_env(
        &self,
        env: &mut HashMap<String, String>,
        environment_id: Option<&str>,
    ) {
        self.virtualize_env(env, environment_id, CredentialEnvironment::Snapshot);
    }

    fn virtualize_env(
        &self,
        env: &mut HashMap<String, String>,
        environment_id: Option<&str>,
        destination: CredentialEnvironment,
    ) {
        let mut state = self.write_state();
        if !state.enabled {
            remove_env_value(env, CREDENTIAL_BROKER_ACTIVE_ENV_KEY);
            remove_env_value(env, BROKERED_CREDENTIALS_ENV_KEY);
            return;
        }
        set_env_value(env, CREDENTIAL_BROKER_ACTIVE_ENV_KEY, "1".to_string());
        state.observe_credential_owners(env);

        let active_sources = active_credential_sources(&state, env);
        let carried_dummies = state
            .credentials
            .iter()
            .filter(|credential| {
                credential.has_carried_identity(env, &state.credentials, &active_sources)
            })
            .map(|credential| credential.dummy_value.clone())
            .collect::<HashSet<_>>();
        for credential in state.credentials.iter_mut().filter(|credential| {
            credential.belongs_to_environment(environment_id)
                && carried_dummies.contains(&credential.dummy_value)
        }) {
            if let Some(source) = registered_credential_source(credential, &active_sources, env)
                .filter(|source| {
                    credential.invalidates_host_binding(env)
                        || source_tracks_credential(source, credential, env)
                })
            {
                credential.observe_host_binding(source.host_binding, env);
            }
        }
        let stale_credentials = state
            .credentials
            .iter()
            .filter(|credential| {
                credential.belongs_to_environment(environment_id)
                    && carried_dummies.contains(&credential.dummy_value)
                    && !registered_credential_source(credential, &active_sources, env)
                        .is_some_and(|source| source_tracks_credential(&source, credential, env))
            })
            .map(|credential| {
                (
                    credential.dummy_value.clone(),
                    credential.real_value.clone(),
                )
            })
            .collect::<Vec<_>>();
        state.replace_child_env_dummies(env, &stale_credentials);
        state.credentials.retain(|credential| {
            !credential.belongs_to_environment(environment_id)
                || !stale_credentials
                    .iter()
                    .any(|(dummy_value, _)| credential.dummy_value == *dummy_value)
        });

        let mut owned_dummies = state
            .credentials
            .iter()
            .filter(|credential| {
                credential.belongs_to_environment(environment_id)
                    && carried_dummies.contains(&credential.dummy_value)
            })
            .map(|credential| credential.dummy_value.clone())
            .collect::<HashSet<_>>();
        let unbound_inherited_credentials = state
            .credentials
            .iter()
            .filter(|credential| {
                !credential.belongs_to_environment(environment_id)
                    && !owned_dummies.contains(&credential.dummy_value)
                    && env_contains_credential_value(env, &credential.dummy_value)
                    && registered_credential_source(credential, &active_sources, env).is_none()
            })
            .map(|credential| {
                (
                    credential.dummy_value.clone(),
                    credential.real_value.clone(),
                )
            })
            .collect::<Vec<_>>();
        state.replace_child_env_dummies(env, &unbound_inherited_credentials);

        let inherited_credentials = state
            .credentials
            .iter()
            .filter(|credential| {
                !credential.belongs_to_environment(environment_id)
                    && !owned_dummies.contains(&credential.dummy_value)
                    && carried_dummies.contains(&credential.dummy_value)
            })
            .filter_map(|credential| {
                registered_credential_source(credential, &active_sources, env).map(|source| {
                    (
                        credential.env_var.clone(),
                        source.provider,
                        source.host_binding,
                        credential.real_value.clone(),
                        credential.dummy_value.clone(),
                        credential.source_values.clone(),
                        Arc::clone(&credential.generated_aliases),
                    )
                })
            })
            .collect::<Vec<_>>();
        let mut rebound_dummies = Vec::new();
        for (
            env_var,
            provider,
            host_binding,
            real_value,
            inherited_dummy,
            source_values,
            generated_aliases,
        ) in inherited_credentials
        {
            let current_dummy = if let Some(credential) =
                state.credentials.iter_mut().find(|credential| {
                    credential.belongs_to_environment(environment_id)
                        && credential.provider.same_provider(&provider)
                        && env_key_matches(&credential.env_var, &env_var)
                        && credential.real_value == real_value
                }) {
                credential.observe_host_binding(host_binding, env);
                credential.dummy_value.clone()
            } else {
                state.credentials.push(CredentialRecord {
                    env_var,
                    provider,
                    host_binding,
                    additional_host_bindings: Vec::new(),
                    fallback_host_bindings: Vec::new(),
                    environment_id: environment_id.map(str::to_string),
                    real_value,
                    dummy_value: inherited_dummy.clone(),
                    source_values,
                    generated_aliases,
                });
                inherited_dummy.clone()
            };
            if current_dummy != inherited_dummy {
                rebound_dummies.push((inherited_dummy, current_dummy.clone()));
            }
            owned_dummies.insert(current_dummy);
        }
        state.replace_child_env_dummies(env, &rebound_dummies);
        for credential in state
            .credentials
            .iter_mut()
            .filter(|credential| credential.belongs_to_environment(environment_id))
        {
            for source in &mut credential.source_values {
                if let Some((_, rebound)) =
                    rebound_dummies.iter().find(|(dummy, _)| dummy == source)
                {
                    source.clone_from(rebound);
                }
            }
        }

        for source in &active_sources {
            for env_var in &source.env_vars {
                virtualize_env_var(
                    env,
                    &mut state,
                    env_var,
                    source.provider.clone(),
                    source.host_binding.clone(),
                    environment_id,
                );
            }
        }
        let provider_context_keys = state.environment(env).provider_context_keys;
        let mut known_registrations = Vec::new();
        let binding_env = state.context.with_fallbacks(env);
        let discoverable_values = env
            .iter()
            .filter(|(key, value)| {
                !key.eq_ignore_ascii_case("PATH")
                    && !key.to_ascii_uppercase().ends_with("_PATH")
                    && !crate::is_managed_proxy_env_var(key, value)
                    && !provider_context_keys
                        .iter()
                        .any(|context_key| env_key_matches(key, context_key))
            })
            .map(|(_, value)| {
                let known = matching::known_credential_matches(&state, value, env);
                for matched in &known {
                    if matched.value == matched.real_value
                        && !state.credentials.iter().any(|credential| {
                            owned_dummies.contains(&credential.dummy_value)
                                && credential.real_value == matched.real_value
                                && credential.provider.same_provider(&matched.provider)
                                && env_key_matches(&credential.env_var, matched.env_var)
                        })
                        && let Some(source) = active_sources.iter().find(|source| {
                            source.provider.same_provider(&matched.provider)
                                && source
                                    .env_vars
                                    .iter()
                                    .any(|key| env_key_matches(key, matched.env_var))
                        })
                    {
                        known_registrations.push((
                            matched.env_var.to_string(),
                            matched.provider.clone(),
                            source.host_binding.clone(),
                            matched.real_value.to_string(),
                        ));
                    }
                }
                matching::mask_known_credentials(value, &known)
            })
            .collect::<Vec<_>>();
        // Keep normal destination reconciliation for exact identities before discovering new ones.
        for (env_var, provider, host_binding, real_value) in known_registrations {
            let _ = state.register(
                &env_var,
                provider,
                host_binding,
                environment_id,
                &real_value,
                env,
            );
        }
        let configured_discoveries = state
            .configured_providers
            .iter()
            .flat_map(|provider| {
                discoverable_values.iter().flat_map(move |value| {
                    provider
                        .find_discoverable_credentials(value)
                        .filter(|matched| !value[matched.range.clone()].contains('\0'))
                        .map(move |matched| (Arc::clone(provider), value, matched))
                })
            })
            .collect::<Vec<_>>();
        let mut builtin_spans = Vec::new();
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        for provider in providers::credential_providers() {
<<<<<<< HEAD
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
||||||| 413492cd6c
            for source in provider.sources() {
                if let Some(host_binding) = (source.host_binding)(env) {
                    for env_var in source.env_vars {
                        virtualize_env_var(
                            env,
                            &mut state,
                            env_var,
                            provider,
=======
            let brokered_provider = BrokeredCredentialProvider::Builtin(provider);
            let Some((source, host_binding)) = provider.sources().iter().rev().find_map(|source| {
                (source.host_binding)(&binding_env, state.openai_api_host.as_deref())
                    .map(|binding| (source, binding))
            }) else {
                continue;
            };
            for value in &discoverable_values {
                for prefix in provider.credential_prefixes {
                    for (start, _) in value.match_indices(prefix) {
                        let credential =
                            matching::builtin_credential_candidate(provider, value, start);
                        let configured_start = configured_discoveries
                            .iter()
                            .filter(|(_, candidate_value, matched)| {
                                *candidate_value == value
                                    && matched.has_distinctive_prefix
                                    && matched.range.start > start
                                    && matched.range.start < start + credential.len()
                            })
                            .map(|(_, _, matched)| matched.range.start - start)
                            .filter(|end| {
                                credential[..*end].trim_end_matches(['_', '-']).len()
                                    >= provider.minimum_credential_len
                            })
                            .min();
                        let credential = configured_start.map_or(credential, |end| {
                            credential[..end].trim_end_matches(['_', '-'])
                        });
                        if credential.len() < provider.minimum_credential_len
                            || matching::is_operational_path_match(
                                value,
                                start,
                                start + credential.len(),
                            )
                            || provider.ignored_credential_prefixes.iter().any(|ignored| {
                                credential.starts_with(ignored)
                                    && provider
                                        .credential_watermark
                                        .is_none_or(|watermark| !credential.contains(watermark))
                            })
                            || provider.request_header_value(credential).is_none()
                        {
                            continue;
                        }
                        // Retain ownership spans even when registration fails or is ambiguous.
                        let span = start..start + credential.len();
                        builtin_spans.push((value, span.clone()));
                        if configured_discoveries
                            .iter()
                            .any(|(_, candidate_value, matched)| {
                                *candidate_value == value
                                    && matched.range.start == start
                                    && matched.range.end >= span.end
                            })
                            || state.is_dummy_value(credential)
                            || state.credentials.iter().any(|existing| {
                                existing.belongs_to_environment(environment_id)
                                    && existing.provider.same_provider(&brokered_provider)
                                    && existing.host_binding == host_binding
                                    && existing.real_value == credential
                            })
                            || state.credential_owners.iter().any(|existing| {
                                existing.real_value == credential
                                    && (source.binding_env_vars.is_empty()
                                        && !source
                                            .env_vars
                                            .iter()
                                            .any(|key| env_key_matches(key, &existing.env_var))
                                        || !provider.sources().iter().any(|candidate| {
                                            candidate
                                                .env_vars
                                                .iter()
                                                .any(|key| env_key_matches(key, &existing.env_var))
                                        }))
                            })
                            || provider.sources().iter().any(|candidate| {
                                candidate
                                    .env_vars
                                    .iter()
                                    .any(|key| env_value(env, key) == Some(credential))
                                    && (candidate.host_binding)(
                                        &binding_env,
                                        state.openai_api_host.as_deref(),
                                    )
                                    .is_none()
                            })
                        {
                            continue;
                        }
                        let _ = state.register(
                            source.env_vars[0],
                            brokered_provider.clone(),
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
                            host_binding.clone(),
                            environment_id,
                            credential,
                            env,
                        );
                    }
                }
            }
        }
<<<<<<< HEAD
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
||||||| 413492cd6c
=======
        for (provider, value, matched) in &configured_discoveries {
            let Some(host_binding) = provider.host_binding(&binding_env) else {
                continue;
            };
            let Some(env_var) = provider.config.env.first() else {
                continue;
            };
            let brokered_provider = BrokeredCredentialProvider::Configured(provider.clone());
            let range = &matched.range;
            let credential = &value[range.clone()];
            if matching::is_operational_path_match(value, range.start, range.end)
                || builtin_spans.iter().any(|(candidate_value, span)| {
                    candidate_value == value && span.start < range.end && range.start < span.end
                })
                || configured_discoveries
                    .iter()
                    .any(|(other, candidate_value, matched)| {
                        !Arc::ptr_eq(provider, other)
                            && candidate_value == value
                            && matched.range.start < range.end
                            && range.start < matched.range.end
                    })
                || state.is_dummy_value(credential)
                || state.credentials.iter().any(|existing| {
                    existing.belongs_to_environment(environment_id)
                        && existing.provider.same_provider(&brokered_provider)
                        && existing.host_binding == host_binding
                        && existing.real_value == credential
                })
                || credential_provider_definitions(&state).any(|other| {
                    !other.same_provider(&brokered_provider)
                        && other.recognizes_credential_in(credential)
                })
                || brokered_provider.request_header_value(credential).is_none()
            {
                continue;
            }
            let _ = state.register(
                env_var,
                brokered_provider.clone(),
                host_binding.clone(),
                environment_id,
                credential,
                env,
            );
        }
        let credentials = prioritized_credentials(&state, env)
            .into_iter()
            .filter(|credential| {
                credential.belongs_to_environment(environment_id)
                    && (owned_dummies.contains(&credential.dummy_value)
                        || active_sources.iter().any(|source| {
                            credential.provider.same_provider(&source.provider)
                                && credential.host_binding == source.host_binding
                        }))
            })
            .collect::<Vec<_>>();
        let mut credential_aliases = Vec::new();
        for (key, value) in env.iter_mut() {
            if crate::is_managed_proxy_env_var(key, value) {
                continue;
            }
            let mut replacements = Replacements::default();
            for credential in &credentials {
                for original in [&credential.real_value, &credential.dummy_value] {
                    replacements.add(
                        credential.value_match_ranges(value, original),
                        &credential.dummy_value,
                        Some(credential),
                    );
                }
            }
            if replacements.render(value) {
                credential_aliases.push(CredentialAlias {
                    env_var: key.clone(),
                    dummy_value: value.clone(),
                });
            }
        }
        for alias in credential_aliases {
            if !state.credential_aliases.iter().any(|existing| {
                env_key_matches(&existing.env_var, &alias.env_var)
                    && existing.dummy_value == alias.dummy_value
            }) {
                state.credential_aliases.push(alias);
            }
        }
        if state.allow_local_binding && matches!(destination, CredentialEnvironment::Child) {
            // Clients bypass the proxy for local destinations, so their credentials must stay real.
            state.restore_child_env(env, |credential| {
                credential.belongs_to_environment(environment_id)
                    && credential
                        .host_bindings()
                        .any(providers::CredentialHostBinding::bypasses_proxy_with_local_binding)
            });
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        }
        update_brokered_credentials_marker(&state, env);
    }

<<<<<<< HEAD
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
||||||| 413492cd6c
    pub(crate) fn host_requires_mitm(&self, host: &str) -> bool {
        let normalized_host = normalize_host(host);
=======
    pub(crate) fn restore_child_env(
        &self,
        env: &mut HashMap<String, String>,
        _command: &mut [String],
    ) {
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        let state = self.read_state();
<<<<<<< HEAD
        state.enabled
            && ((state.scoped_openai.is_some() || state.isolated_openai.is_some())
                && normalized_host == resolver::OPENAI_API_HOST
                || state
                    .credentials
                    .iter()
                    .any(|credential| credential.matches_host(&normalized_host)))
||||||| 413492cd6c
        state.enabled
            && state
                .credentials
                .iter()
                .any(|credential| credential.matches_host(&normalized_host))
=======
        if !state.enabled || env_value(env, CREDENTIAL_BROKER_ACTIVE_ENV_KEY) != Some("1") {
            return;
        }
        state.restore_child_env(env, |_| true);
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
    }

<<<<<<< HEAD
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
||||||| 413492cd6c
    pub(crate) fn inject_request_headers(&self, host: &str, headers: &mut HeaderMap) {
=======
    pub(crate) fn restore_and_disable_child_env(
        &self,
        env: &mut HashMap<String, String>,
        command: &mut [String],
    ) {
        self.restore_child_env(env, command);
        remove_env_value(env, CREDENTIAL_BROKER_ACTIVE_ENV_KEY);
        remove_env_value(env, BROKERED_CREDENTIALS_ENV_KEY);
    }

    pub(crate) fn child_alias_matches(
        &self,
        key: &str,
        value: &str,
        snapshot_value: &str,
        environment_id: Option<&str>,
    ) -> bool {
        let state = self.read_state();
        if !state.enabled {
            return false;
        }
        let mut expected = HashMap::from([(key.to_string(), snapshot_value.to_string())]);
        state.restore_child_env(&mut expected, |_| true);
        let referenced_credentials = |text: &str| {
            state
                .credentials
                .iter()
                .filter(|credential| {
                    text == credential.dummy_value
                        || credential.contains_embedded_value(text, &credential.dummy_value)
                })
                .collect::<Vec<_>>()
        };
        let expected_credentials = referenced_credentials(snapshot_value);
        if expected_credentials.is_empty() {
            return false;
        }
        state.credential_aliases.iter().any(|alias| {
            if !env_key_matches(key, &alias.env_var) {
                return false;
            }
            let alias_credentials = referenced_credentials(&alias.dummy_value);
            let same_owner = |left: &&CredentialRecord, right: &&CredentialRecord| {
                left.provider.same_provider(&right.provider)
                    && env_key_matches(&left.env_var, &right.env_var)
                    && left.real_value == right.real_value
            };
            if !expected_credentials.iter().all(|expected| {
                alias_credentials
                    .iter()
                    .any(|actual| same_owner(expected, actual))
            }) || !alias_credentials.iter().all(|actual| {
                expected_credentials
                    .iter()
                    .any(|expected| same_owner(expected, actual))
            }) {
                return false;
            }
            let mut candidate = HashMap::from([(key.to_string(), alias.dummy_value.clone())]);
            let mut identity = candidate.clone();
            if state.allow_local_binding {
                state.restore_child_env(&mut candidate, |credential| {
                    credential.belongs_to_environment(environment_id)
                        && credential.host_bindings().any(
                            providers::CredentialHostBinding::bypasses_proxy_with_local_binding,
                        )
                });
            }
            if env_value(&candidate, key) != Some(value) {
                return false;
            }
            state.restore_child_env(&mut identity, |_| true);
            identity == expected
        })
    }

    #[cfg(test)]
    pub(crate) fn host_requires_mitm(&self, host: &str, port: u16) -> bool {
        self.host_protocols_for_environment(host, port, /*environment_id*/ None)
            .tls
    }

    pub(crate) fn host_protocols_for_environment(
        &self,
        host: &str,
        port: u16,
        environment_id: Option<&str>,
    ) -> crate::brokered_tunnel::BrokeredProtocols {
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        let normalized_host = normalize_host(host);
<<<<<<< HEAD
        let mut state = self.write_state();
||||||| 413492cd6c
        let state = self.read_state();
=======
        let state = self.read_state();
        let mut protocols = crate::brokered_tunnel::BrokeredProtocols::default();
        if state.enabled {
            for credential in state
                .credentials
                .iter()
                .filter(|credential| credential.belongs_to_environment(environment_id))
            {
                for binding in credential.host_bindings() {
                    protocols.tls |= binding.requires_mitm(&normalized_host, port);
                    if let providers::CredentialHostBinding::ConfiguredHosts(destinations) = binding
                    {
                        protocols.http |= destinations.iter().any(|destination| {
                            destination.requires_http_interception(&normalized_host, port)
                        });
                    }
                }
            }
        }
        protocols
    }

    pub(crate) fn virtualize_text(&self, text: &mut String, env: &HashMap<String, String>) -> bool {
        let state = self.read_state();
        matching::virtualize_text(&state, text, env)
    }

    pub(crate) fn handle_unmasked_credentials(
        &self,
        env: &mut HashMap<String, String>,
        mode: CredentialMaskingFailureMode,
    ) -> anyhow::Result<()> {
        let state = self.read_state();
        anyhow::ensure!(
            state.enabled,
            "credential masking requires an enabled credential broker"
        );
        let mut unmasked_keys = Vec::new();
        for (key, value) in env.iter() {
            let value = value.trim();
            let known = matching::known_credential_matches(&state, value, env);
            let contains_real_credential = known
                .iter()
                .any(|matched| matched.value == matched.real_value);
            let remainder = matching::mask_known_credentials(value, &known);
            let mut masked = remainder.clone();
            if contains_real_credential
                || !matching::virtualize_text(&state, &mut masked, env)
                || masked != remainder
            {
                unmasked_keys.push(key.clone());
            }
        }
        drop(state);
        if unmasked_keys.is_empty() {
            return Ok(());
        }
        match mode {
            CredentialMaskingFailureMode::Warn => {
                tracing::warn!(
                    "Credential masking could not protect some environment credentials; continuing with those values unmasked."
                );
            }
            CredentialMaskingFailureMode::Deny => {
                for key in unmasked_keys {
                    env.remove(&key);
                }
                tracing::warn!(
                    "Credential masking could not protect some environment credentials; removed those variables before execution."
                );
            }
            CredentialMaskingFailureMode::Error => {
                anyhow::bail!(
                    "credential masking left an unmasked credential in the command environment"
                );
            }
        }
        Ok(())
    }

    pub(crate) fn restore_text(&self, text: &mut String) -> bool {
        let state = self.read_state();
        if !state.enabled {
            return false;
        }

        let mut credentials = state.credentials.iter().collect::<Vec<_>>();
        credentials
            .sort_unstable_by_key(|credential| std::cmp::Reverse(credential.dummy_value.len()));
        let mut replacements = Replacements::default();
        for credential in credentials {
            replacements.add(
                credential.value_match_ranges(text, &credential.dummy_value),
                &credential.real_value,
                /*dummy*/ None,
            );
        }
        replacements.render(text)
    }

    pub(crate) fn environment(&self, env: &HashMap<String, String>) -> CredentialBrokerEnvironment {
        self.read_state().environment(env)
    }

    pub(crate) fn environment_for_text(
        &self,
        text: &str,
        env: &HashMap<String, String>,
    ) -> CredentialBrokerEnvironment {
        let state = self.read_state();
        let mut scoped_env = env.clone();
        for credential in &state.credentials {
            remove_env_value(&mut scoped_env, &credential.env_var);
        }
        for credential in &state.credentials {
            if text == credential.dummy_value
                || credential.contains_embedded_value(text, &credential.dummy_value)
            {
                set_env_value(
                    &mut scoped_env,
                    &credential.env_var,
                    credential.dummy_value.clone(),
                );
            }
        }
        update_brokered_credentials_marker(&state, &mut scoped_env);
        state.environment(&scoped_env)
    }

    pub(crate) fn source_matches_text(&self, source: &str, source_value: &str, text: &str) -> bool {
        let state = self.read_state();
        state.enabled
            && state.credentials.iter().any(|credential| {
                env_key_matches(&credential.env_var, source)
                    && credential.real_value == source_value
                    && (text == credential.real_value
                        || credential.contains_embedded_value(text, &credential.real_value))
            })
    }

    pub(crate) fn provider_sources_allowed(
        &self,
        value: &str,
        virtualized: &str,
        source_env: &HashMap<String, String>,
        is_allowed: impl Fn(&str) -> bool,
    ) -> bool {
        let state = self.read_state();
        let known = matching::known_credential_matches(&state, value, source_env);
        if known.iter().any(|matched| {
            !known.iter().any(|equivalent| {
                equivalent.range == matched.range
                    && equivalent.provider.same_provider(&matched.provider)
                    && equivalent.real_value == matched.real_value
                    && is_allowed(equivalent.env_var)
            })
        }) {
            return false;
        }
        let uncovered = matching::mask_known_credentials(value, &known);
        let value = uncovered.as_str();
        let builtin_recognized = providers::credential_providers().any(|provider| {
            provider.credential_prefixes.iter().any(|prefix| {
                value.match_indices(*prefix).any(|(start, _)| {
                    matching::recognized_credential_match(provider, value, virtualized, start)
                        .is_some()
                })
            })
        });
        if builtin_recognized
            && !credential_broker_provider_sources_allowed(
                value,
                virtualized,
                source_env,
                &is_allowed,
            )
        {
            return false;
        }

        let mut configured_recognized = false;
        for provider in &state.configured_providers {
            for source in &provider.config.env {
                if let Some(credential) = env_value(source_env, source)
                    && provider.matches_value(credential)
                    && !provider
                        .credential_value_match_ranges(value, credential)
                        .is_empty()
                    && provider
                        .credential_value_match_ranges(virtualized, credential)
                        .is_empty()
                {
                    configured_recognized = true;
                    if !provider.config.env.iter().any(|equivalent| {
                        env_value(source_env, equivalent) == Some(credential)
                            && is_allowed(equivalent)
                    }) {
                        return false;
                    }
                }
            }
            for credential in provider.find_credentials(value).filter(|credential| {
                !credential.contains('\0') && !virtualized.contains(credential)
            }) {
                configured_recognized = true;
                let sources = provider
                    .config
                    .env
                    .iter()
                    .filter(|source| env_value(source_env, source) == Some(credential))
                    .collect::<Vec<_>>();
                let allowed = if sources.is_empty() {
                    provider.config.env.iter().all(|source| is_allowed(source))
                } else {
                    sources.iter().any(|source| is_allowed(source))
                };
                if !allowed {
                    return false;
                }
            }
        }

        !known.is_empty() || builtin_recognized || configured_recognized
    }

    #[cfg(test)]
    pub(crate) fn inject_request_headers(&self, destination: &str, headers: &mut HeaderMap) {
        self.inject_request_headers_for_environment(
            destination,
            headers,
            /*environment_id*/ None,
        );
    }

    pub(crate) fn inject_request_headers_for_environment(
        &self,
        destination: &str,
        headers: &mut HeaderMap,
        environment_id: Option<&str>,
    ) {
        let state = self.read_state();
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
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

        let credentials =
            select_credentials(headers, destination, &state.credentials, environment_id);
        for (credential, header_name, header_value) in credentials {
            credential
                .provider
                .insert_request_header(headers, header_name, header_value);
        }
    }

    pub(crate) fn request_matches_hooked_host_alias(
        &self,
        destination: &str,
        headers: &HeaderMap,
        hooked_hosts: &[String],
        environment_id: Option<&str>,
    ) -> bool {
        let request = if destination.contains("://") {
            Url::parse(destination).ok()
        } else {
            None
        };
        let normalized_host = normalize_host(
            request
                .as_ref()
                .and_then(Url::host_str)
                .unwrap_or(destination),
        );
        let state = self.read_state();
        if !state.enabled {
            return false;
        }

        let selected_headers =
            select_credentials(headers, destination, &state.credentials, environment_id)
                .into_iter()
                .map(|(_, name, _)| name)
                .collect::<Vec<_>>();
        if selected_headers.is_empty() {
            return false;
        }
        let matching_credentials = state
            .credentials
            .iter()
            .filter(|credential| {
                credential.belongs_to_environment(environment_id)
                    && credential
                        .host_bindings()
                        .any(|binding| binding.matches_request(&normalized_host, request.as_ref()))
            })
            .collect::<Vec<_>>();
<<<<<<< HEAD
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
||||||| 413492cd6c
        let Some(credential) = select_credential(headers, &matching_credentials) else {
            return;
        };
        let Some(header_value) = credential
            .provider
            .request_header_value(&credential.real_value)
        else {
            return;
        };
        credential
            .provider
            .insert_request_header(headers, header_value);
=======

        // Ambiguity removes the whole header, so every translation for a surviving header
        // contributed through equivalent deduplication or Basic-field merging.
        matching_credentials.iter().any(|credential| {
            hooked_hosts.iter().any(|hooked_host| {
                hooked_host != &normalized_host
                    && credential
                        .host_bindings()
                        .any(|binding| binding.matches_hostname(hooked_host))
            }) && matching_credentials.iter().any(|candidate| {
                candidate.provider.same_provider(&credential.provider)
                    && candidate.real_value == credential.real_value
                    && credential
                        .provider
                        .translate_request_headers(
                            headers,
                            &candidate.dummy_value,
                            &credential.real_value,
                        )
                        .iter()
                        .any(|(name, _)| selected_headers.contains(name))
            })
        })
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
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
    provider: BrokeredCredentialProvider,
    host_binding: providers::CredentialHostBinding,
    environment_id: Option<&str>,
) {
<<<<<<< HEAD
    let Some(real_value) = brokerable_credential_value(source_env, state, env_var, provider) else {
||||||| 413492cd6c
    let Some(real_value) = brokerable_credential_value(env, state, env_var, provider) else {
=======
    let previous_dummy = env_value(env, env_var)
        .filter(|value| state.is_dummy_value(value))
        .map(str::to_string);
    let Some(real_value) = brokerable_credential_value(env, state, env_var, &provider)
        .map(str::to_string)
        .or_else(|| {
            let dummy = previous_dummy.as_deref()?;
            state
                .credentials
                .iter()
                .find(|credential| {
                    credential.dummy_value == dummy
                        && credential.provider.same_provider(&provider)
                        && env_key_matches(&credential.env_var, env_var)
                })
                .map(|credential| credential.real_value.clone())
        })
    else {
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        return;
    };

<<<<<<< HEAD
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
||||||| 413492cd6c
    let dummy_value = state.register(env_var, provider, host_binding, real_value);
    env.insert(env_var.to_string(), dummy_value);
=======
    if let Some(dummy_value) = state.register(
        env_var,
        provider,
        host_binding,
        environment_id,
        &real_value,
        env,
    ) {
        if let Some(previous_dummy) = previous_dummy
            && previous_dummy != dummy_value
        {
            state.replace_child_env_dummies(env, &[(previous_dummy, dummy_value.clone())]);
        }
        set_env_value(env, env_var, dummy_value);
    }
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
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
    provider: &BrokeredCredentialProvider,
) -> Option<&'a str> {
    let real_value = env_value(env, env_var)?;
    let real_value = match provider {
        BrokeredCredentialProvider::Builtin(_) => real_value.trim(),
        BrokeredCredentialProvider::Configured(_) => real_value,
    };
    (!real_value.is_empty()
        && !state.is_dummy_value(real_value)
        && provider.request_header_value(real_value).is_some())
    .then_some(real_value)
}

impl CredentialBrokerState {
    fn restore_child_env(
        &self,
        env: &mut HashMap<String, String>,
        should_restore: impl Fn(&CredentialRecord) -> bool,
    ) {
        let credentials = self
            .credentials
            .iter()
            .filter(|credential| should_restore(credential))
            .filter(|credential| {
                env.iter().any(|(key, value)| {
                    !env_key_matches(key, CREDENTIAL_BROKER_ACTIVE_ENV_KEY)
                        && !env_key_matches(key, BROKERED_CREDENTIALS_ENV_KEY)
                        && (value == &credential.dummy_value
                            || credential.contains_embedded_value(value, &credential.dummy_value))
                })
            })
            .collect::<Vec<_>>();
        for (key, value) in env.iter_mut() {
            if env_key_matches(key, CREDENTIAL_BROKER_ACTIVE_ENV_KEY)
                || env_key_matches(key, BROKERED_CREDENTIALS_ENV_KEY)
            {
                continue;
            }
            let canonical_credential = self
                .credentials
                .iter()
                .any(|credential| env_key_matches(key, &credential.env_var));
            if !canonical_credential
                && !self.credential_aliases.iter().any(|alias| {
                    env_key_matches(key, &alias.env_var) && value == &alias.dummy_value
                })
            {
                continue;
            }
            let mut replacements = Replacements::default();
            for credential in &credentials {
                if canonical_credential
                    && !self.credentials.iter().any(|candidate| {
                        env_key_matches(key, &candidate.env_var)
                            && candidate.provider.same_provider(&credential.provider)
                            && candidate.host_binding == credential.host_binding
                            && candidate.real_value == credential.real_value
                    })
                {
                    continue;
                }
                replacements.add(
                    credential.value_match_ranges(value, &credential.dummy_value),
                    &credential.real_value,
                    /*dummy*/ None,
                );
            }
            // Carry surviving dummy spans through partial local-destination restoration.
            for credential in &self.credentials {
                replacements.add(
                    credential.value_match_ranges(value, &credential.dummy_value),
                    &credential.dummy_value,
                    Some(credential),
                );
            }
            replacements.render(value);
        }
    }

    fn observe_credential_owners(&mut self, env: &HashMap<String, String>) {
        // Ownership is known even when a destination is missing or invalid.
        let sources = providers::credential_providers()
            .flat_map(|provider| {
                provider.sources().iter().flat_map(move |source| {
                    source
                        .env_vars
                        .iter()
                        .map(move |key| (*key, BrokeredCredentialProvider::Builtin(provider)))
                })
            })
            .chain(self.configured_providers.iter().flat_map(|provider| {
                provider.config.env.iter().map(move |key| {
                    (
                        key.as_str(),
                        BrokeredCredentialProvider::Configured(Arc::clone(provider)),
                    )
                })
            }));
        let owners = sources
            .filter_map(|(key, provider)| {
                brokerable_credential_value(env, self, key, &provider)
                    .map(|value| (key.to_string(), value.to_string()))
            })
            .collect::<Vec<_>>();
        for (key, value) in owners {
            self.remember_credential_owner(&key, &value);
        }
    }

    fn remember_credential_owner(&mut self, env_var: &str, real_value: &str) {
        if !self
            .credential_owners
            .iter()
            .any(|owner| env_key_matches(&owner.env_var, env_var) && owner.real_value == real_value)
        {
            self.credential_owners.push(CredentialOwner {
                env_var: env_var.to_string(),
                real_value: real_value.to_string(),
            });
        }
    }

    fn register(
        &mut self,
        env_var: &str,
        provider: BrokeredCredentialProvider,
        host_binding: providers::CredentialHostBinding,
        environment_id: Option<&str>,
        real_value: &str,
<<<<<<< HEAD
    ) -> String {
        if let Some(existing) = self.credentials.iter().find(|credential| {
            credential.env_var == env_var
                && std::ptr::eq(credential.provider, provider)
                && credential.host_binding == host_binding
                && matches!(&credential.secret, RecordSecret::Raw(value) if value.as_str() == real_value)
||||||| 413492cd6c
    ) -> String {
        if let Some(existing) = self.credentials.iter().find(|credential| {
            credential.env_var == env_var
                && std::ptr::eq(credential.provider, provider)
                && credential.host_binding == host_binding
                && credential.real_value == real_value
=======
        existing_env: &HashMap<String, String>,
    ) -> Option<String> {
        self.remember_credential_owner(env_var, real_value);
        if let Some(existing) = self.credentials.iter_mut().find(|credential| {
            env_key_matches(&credential.env_var, env_var)
                && credential.provider.same_provider(&provider)
                && credential.belongs_to_environment(environment_id)
                && credential.real_value == real_value
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        }) {
            // Existing dummies were reconciled before fresh-value discovery.
            if !env_contains_credential_value(existing_env, &existing.dummy_value) {
                existing.observe_host_binding(host_binding, existing_env);
            }
            return Some(existing.dummy_value.clone());
        }
        if self.credentials.iter().any(|credential| {
            is_builtin_shaped_credential(real_value) && credential.dummy_value.contains(real_value)
                || provider.contains_embedded_value(&credential.dummy_value, real_value)
        }) {
            tracing::warn!(
                env_var,
                "credential brokerage skipped: credential overlaps an existing dummy"
            );
            return None;
        }

<<<<<<< HEAD
        let dummy_value = self.fresh_dummy(provider, real_value);
||||||| 413492cd6c
        let dummy_value = loop {
            let candidate = provider.dummy_value(real_value);
            if candidate != real_value && !self.is_dummy_value(&candidate) {
                break candidate;
            }
        };
=======
        let mut alias_identities = Vec::new();
        if matches!(&provider, BrokeredCredentialProvider::Builtin(_))
            && !self.configured_providers.is_empty()
        {
            let context_keys = self.environment(existing_env).provider_context_keys;
            for (key, value) in existing_env {
                if key.eq_ignore_ascii_case("PATH")
                    || key.to_ascii_uppercase().ends_with("_PATH")
                    || crate::is_managed_proxy_env_var(key, value)
                    || context_keys
                        .iter()
                        .any(|context| env_key_matches(key, context))
                {
                    continue;
                }
                let known = matching::known_credential_matches(self, value, existing_env);
                let uncovered = matching::mask_known_credentials(value, &known);
                for configured in &self.configured_providers {
                    for matched in configured.find_discoverable_credentials(&uncovered) {
                        let value = &uncovered[matched.range.clone()];
                        if value.contains('\0')
                            || matching::is_operational_path_match(
                                &uncovered,
                                matched.range.start,
                                matched.range.end,
                            )
                        {
                            continue;
                        }
                        alias_identities.push((
                            BrokeredCredentialProvider::Configured(Arc::clone(configured)),
                            value.to_string(),
                        ));
                    }
                }
            }
        }

        let Some(dummy_value) = (0..64).find_map(|_| {
            let candidate = provider.dummy_value(real_value)?;
            (candidate != real_value
                // These records restore embedded dummies without configured regex boundaries.
                && (!is_builtin_shaped_credential(real_value)
                    || candidate.len() >= MIN_EMBEDDED_CREDENTIAL_LENGTH)
                && !existing_env
                    .values()
                    .any(|value| value.contains(&candidate))
                && !self.credentials.iter().any(|credential| {
                    credential.dummy_value == candidate
                        || credential.real_value == candidate
                        || credential.contains_embedded_value(&candidate, &credential.dummy_value)
                        || credential.contains_embedded_value(&candidate, &credential.real_value)
                        || is_builtin_shaped_credential(real_value)
                            && credential.dummy_value.contains(&candidate)
                        || provider.contains_embedded_value(&credential.dummy_value, &candidate)
                })
                && match &provider {
                    BrokeredCredentialProvider::Builtin(_) => {
                        // Check observed identities even when their sources have not registered.
                        !credential_provider_definitions(self).any(|owner_provider| {
                            self.credential_owners.iter().any(|owner| {
                                let owns_key = match &owner_provider {
                                    BrokeredCredentialProvider::Builtin(provider) => {
                                        provider.sources().iter().any(|source| {
                                            source.env_vars.iter().any(|key| {
                                                env_key_matches(key, &owner.env_var)
                                            })
                                        })
                                    }
                                    BrokeredCredentialProvider::Configured(provider) => provider
                                        .config
                                        .env
                                        .iter()
                                        .any(|key| env_key_matches(key, &owner.env_var)),
                                };
                                owns_key
                                    && (candidate == owner.real_value
                                        || is_builtin_shaped_credential(&owner.real_value)
                                            && candidate.contains(&owner.real_value)
                                        || owner_provider.contains_embedded_value(
                                            &candidate,
                                            &owner.real_value,
                                        ))
                            })
                        })
                    }
                    BrokeredCredentialProvider::Configured(_) => {
                        !credential_provider_definitions(self).any(|other| {
                            !other.same_provider(&provider)
                                && other.recognizes_strictly_embedded_dummy_collision_in(&candidate)
                        })
                    }
                }
                && !alias_identities.iter().any(|(alias_provider, value)| {
                    candidate == *value
                        || is_builtin_shaped_credential(value) && candidate.contains(value)
                        || alias_provider.contains_embedded_value(&candidate, value)
                }))
            .then_some(candidate)
        }) else {
            tracing::warn!(
                env_var,
                "credential brokerage skipped: unable to generate a unique dummy credential"
            );
            return None;
        };
        let source_values = env_value(existing_env, env_var)
            .map(|value| {
                let value = match &provider {
                    BrokeredCredentialProvider::Builtin(_) => value.trim(),
                    BrokeredCredentialProvider::Configured(_) => value,
                };
                if value == real_value {
                    vec![real_value.to_string(), dummy_value.clone()]
                } else if let Some(source) = self
                    .credentials
                    .iter()
                    .find(|source| source.dummy_value == value)
                {
                    vec![source.real_value.clone(), source.dummy_value.clone()]
                } else {
                    vec![value.to_string()]
                }
            })
            .unwrap_or_default();
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        self.credentials.push(CredentialRecord {
            env_var: env_var.to_string(),
            provider,
            host_binding,
<<<<<<< HEAD
            secret: RecordSecret::Raw(Zeroizing::new(real_value.to_string())),
||||||| 413492cd6c
            real_value: real_value.to_string(),
=======
            additional_host_bindings: Vec::new(),
            fallback_host_bindings: Vec::new(),
            environment_id: environment_id.map(str::to_string),
            real_value: real_value.to_string(),
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
            dummy_value: dummy_value.clone(),
            source_values,
            generated_aliases: Arc::default(),
        });
        Some(dummy_value)
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

<<<<<<< HEAD
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
||||||| 413492cd6c
impl CredentialRecord {
    fn matches_host(&self, host: &str) -> bool {
        self.host_binding.matches_host(host)
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
=======
fn credential_provider_definitions(
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
    state: &CredentialBrokerState,
<<<<<<< HEAD
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
||||||| 413492cd6c
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
=======
) -> impl Iterator<Item = BrokeredCredentialProvider> + '_ {
    providers::credential_providers()
        .map(BrokeredCredentialProvider::Builtin)
        .chain(
            state
                .configured_providers
                .iter()
                .cloned()
                .map(BrokeredCredentialProvider::Configured),
        )
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
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

#[cfg(test)]
#[path = "credential_broker/configured_tests.rs"]
mod configured_tests;
