mod workspace_routing;
mod workspace_switching;
pub use workspace_switching::WorkspaceLogin;
pub use workspace_switching::WorkspaceSwitch;

#[path = "account_grant.rs"]
mod account_grant;
pub use account_grant::AccountOAuthGrant;
pub use account_grant::AccountOAuthGrantLease;
pub use account_grant::AccountOAuthGrantRevision;
use account_grant::same_account_grant;
use account_grant::same_account_login;

use chrono::Utc;
use http::StatusCode;
use serde::Deserialize;
use serde::Serialize;
#[cfg(test)]
use serial_test::serial;
use std::collections::HashMap;
use std::collections::HashSet;
use std::env;
use std::ffi::OsString;
use std::fmt::Debug;
use std::future::Future;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::OnceLock;
use std::sync::RwLock;
use std::sync::Weak;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::time::Duration;
use std::time::Instant;
use tokio::sync::Semaphore;
use tokio::sync::watch;
use tracing::instrument;

use codex_agent_identity::ChatGptEnvironment;
use codex_protocol::auth::AuthMode;
use codex_protocol::config_types::ForcedLoginMethod;
use codex_protocol::config_types::ModelProviderAuthInfo;

use super::access_token::CodexAccessToken;
use super::access_token::classify_codex_access_token;
use super::agent_identity::ManagedChatGptAgentIdentityBinding;
use super::agent_identity::agent_identity_authapi_base_url;
use super::agent_identity::classify_bootstrap_error;
use super::agent_identity::record_matches_managed_chatgpt_binding;
use super::agent_identity::record_needs_task_registration;
use super::agent_identity::register_managed_chatgpt_agent_identity;
use super::agent_identity::require_agent_identity_authapi_base_url;
use super::agent_identity::verified_record_from_jwt;
use super::api_key::is_fedramp_api_key;
use super::change_state::AuthChangeState;
use super::change_state::same_login;
use super::change_state::same_owner;
use super::external_bearer::BearerTokenRefresher;
use super::external_bearer::ExternalBearerCachePolicy;
use super::revoke::revoke_auth_tokens;
use super::workload_identity::WorkloadIdentityExternalAuth;
use super::workload_identity::WorkloadIdentitySessionError;
use crate::auth::AuthHeaders;
pub use crate::auth::agent_identity::AgentIdentityAuth;
pub use crate::auth::agent_identity::AgentIdentityAuthError;
pub use crate::auth::bedrock_access_keys::BedrockAccessKeysAuth;
pub use crate::auth::bedrock_api_key::BedrockApiKeyAuth;
use crate::auth::named_chatgpt::NamedChatgptAccount;
pub use crate::auth::personal_access_token::PersonalAccessTokenAuth;
pub use crate::auth::storage::AgentIdentityAuthRecord;
pub use crate::auth::storage::AgentIdentityStorage;
pub use crate::auth::storage::AuthDotJson;
pub use crate::auth::storage::AuthKeyringBackendKind;
use crate::auth::storage::AuthStorageBackend;
use crate::auth::storage::auth_storage_commit_guard;
use crate::auth::storage::create_auth_storage;
use crate::default_client::create_client;
use crate::default_client::create_default_auth_client;
use crate::oauth::ErrorBodyLimit;
use crate::oauth::OAuthClient;
use crate::oauth::OAuthError;
use crate::oauth::RefreshReason;
use crate::oauth::RefreshTokenGrant;
use crate::oauth::TokenEncoding;
use crate::oauth::TokenEndpoint;
use crate::outbound_proxy::AuthRouteConfig;
use crate::token_data::AccountOAuthProvenance;
use crate::token_data::TokenData;
use crate::token_data::parse_chatgpt_account_user_id;
use crate::token_data::parse_chatgpt_jwt_claims;
use crate::token_data::parse_jwt_expiration;
use codex_config::ManagedAuthPolicy;
use codex_config::types::AuthCredentialsStoreMode;
use codex_http_client::CHATGPT_ACCOUNT_ID_HEADER;
use codex_http_client::HttpClient;
use codex_http_client::HttpClientFactory;
use codex_http_client::OutboundProxyPolicy;
pub use codex_model_provider_info::AMBIENT_API_KEY_ENV_VAR;
pub use codex_model_provider_info::KIMI_CODE_API_KEY_ENV_VAR;
use codex_protocol::account::PlanType as AccountPlanType;
use codex_protocol::auth::PlanType as InternalPlanType;
use codex_protocol::auth::RefreshTokenFailedError;
use codex_protocol::auth::RefreshTokenFailedReason;
use codex_protocol::protocol::SessionSource;
use thiserror::Error;
pub use workspace_routing::WorkspaceRouting;
pub use workspace_routing::WorkspaceRoutingRequest;
pub use workspace_routing::WorkspaceRoutingResolver;
pub use workspace_routing::WorkspaceRoutingSession;

/// Authentication mechanism used by the current user.
#[derive(Debug, Clone)]
pub enum CodexAuth {
    ApiKey(ApiKeyAuth),
    Chatgpt(ChatgptAuth),
    ChatgptAuthTokens(ChatgptAuthTokens),
    Headers(AuthHeaders),
    AgentIdentity(AgentIdentityAuth),
    PersonalAccessToken(PersonalAccessTokenAuth),
    BedrockApiKey(BedrockApiKeyAuth),
    BedrockAccessKeys(BedrockAccessKeysAuth),
}

/// Metadata-only location of a stored provider API key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderApiKeyStorageSource {
    EncryptedVault,
    LegacyPlaintext,
}

/// Metadata-only state of a provider API key in Corbanu-managed storage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderApiKeyStorageMetadata {
    Stored { source: ProviderApiKeyStorageSource },
    Missing,
    Suppressed,
}

/// Secret-free provider-key metadata loaded with one vault-index and one legacy metadata read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderApiKeyStorageMetadataSnapshot {
    entries: HashMap<String, ProviderApiKeyStorageMetadata>,
}

impl ProviderApiKeyStorageMetadataSnapshot {
    pub fn get(&self, provider_key_id: &str) -> Option<ProviderApiKeyStorageMetadata> {
        self.entries.get(provider_key_id).copied()
    }
}

/// Metadata-only state of the primary OpenAI authentication boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenAiAuthMetadata {
    Missing,
    Account,
    ApiKey,
    EnvironmentApiKey,
    ExternallyManaged,
    RecoveryRequired,
    Unsupported,
}

/// Policy for resolving Agent Identity auth from a broader Codex auth snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentIdentityAuthPolicy {
    /// Use Agent Identity auth only when the current auth is already Agent Identity.
    JwtOnly,
    /// Allow managed ChatGPT auth to register or reuse Agent Identity auth.
    ChatGptAuth,
}

const AGENT_IDENTITY_BOOTSTRAP_FAILURE_COOLDOWN: Duration = Duration::from_secs(60 * 60);

#[derive(Debug)]
struct CachedAgentIdentityBootstrapFailure {
    account_id: String,
    authapi_base_url: String,
    retry_at: Instant,
    error: AgentIdentityAuthError,
}

#[derive(Debug, Default)]
struct AgentIdentityBootstrapCooldown {
    failure: Option<CachedAgentIdentityBootstrapFailure>,
}

impl AgentIdentityBootstrapCooldown {
    fn error_for(
        &mut self,
        account_id: &str,
        authapi_base_url: &str,
        now: Instant,
    ) -> Option<AgentIdentityAuthError> {
        let error = self
            .failure
            .as_ref()
            .filter(|failure| {
                failure.account_id == account_id
                    && failure.authapi_base_url == authapi_base_url
                    && failure.retry_at > now
            })
            .map(|failure| failure.error.clone());
        if error.is_none() {
            self.clear();
        }
        error
    }

    fn record_failure(
        &mut self,
        account_id: String,
        authapi_base_url: String,
        error: AgentIdentityAuthError,
        now: Instant,
    ) {
        self.failure = Some(CachedAgentIdentityBootstrapFailure {
            account_id,
            authapi_base_url,
            retry_at: now + AGENT_IDENTITY_BOOTSTRAP_FAILURE_COOLDOWN,
            error,
        });
    }

    fn clear(&mut self) {
        self.failure = None;
    }
}

impl PartialEq for CodexAuth {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Headers(a), Self::Headers(b)) => a == b,
            (Self::PersonalAccessToken(a), Self::PersonalAccessToken(b)) => a == b,
            (Self::BedrockApiKey(a), Self::BedrockApiKey(b)) => a == b,
            (Self::BedrockAccessKeys(a), Self::BedrockAccessKeys(b)) => a == b,
            _ => self.api_auth_mode() == other.api_auth_mode(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ApiKeyAuth {
    api_key: String,
    /// The environment variable the key was read from, when it did not come
    /// from Corbanu's own storage.
    env_var: Option<&'static str>,
}

#[derive(Debug, Clone)]
pub struct ChatgptAuth {
    state: ChatgptAuthState,
    storage: Arc<dyn AuthStorageBackend>,
}

#[derive(Debug, Clone)]
pub struct ChatgptAuthTokens {
    state: ChatgptAuthState,
}

#[derive(Debug, Clone)]
struct ChatgptAuthState {
    auth_dot_json: Arc<Mutex<Option<AuthDotJson>>>,
    client: HttpClient,
}

const TOKEN_REFRESH_INTERVAL: i64 = 8;
const CHATGPT_ACCESS_TOKEN_REFRESH_WINDOW_MINUTES: i64 = 5;

const REFRESH_TOKEN_EXPIRED_MESSAGE: &str = "Your access token could not be refreshed because your refresh token has expired. Please log out and sign in again.";
const REFRESH_TOKEN_REUSED_MESSAGE: &str = "Your access token could not be refreshed because your refresh token was already used. Please log out and sign in again.";
const REFRESH_TOKEN_INVALIDATED_MESSAGE: &str = "Your access token could not be refreshed because your refresh token was revoked. Please log out and sign in again.";
const REFRESH_TOKEN_UNKNOWN_MESSAGE: &str =
    "Your access token could not be refreshed. Please log out and sign in again.";
const REFRESH_TOKEN_ACCOUNT_MISMATCH_MESSAGE: &str = "Your access token could not be refreshed because you have since logged out or signed in to another account. Please sign in again.";
const REFRESH_TOKEN_URL: &str = "https://auth.openai.com/oauth/token";
pub(super) const REVOKE_TOKEN_URL: &str = "https://auth.openai.com/oauth/revoke";
pub const REFRESH_TOKEN_URL_OVERRIDE_ENV_VAR: &str = "CODEX_REFRESH_TOKEN_URL_OVERRIDE";
pub const REVOKE_TOKEN_URL_OVERRIDE_ENV_VAR: &str = "CODEX_REVOKE_TOKEN_URL_OVERRIDE";
pub const CLIENT_ID_OVERRIDE_ENV_VAR: &str = "CODEX_APP_SERVER_LOGIN_CLIENT_ID";
static NEXT_DUMMY_AUTH_ID: AtomicU64 = AtomicU64::new(1);
static PROVIDER_API_KEY_STORAGE_REVISION: AtomicU64 = AtomicU64::new(1);
static NEXT_PROVIDER_AUTH_TEMP_ID: AtomicU64 = AtomicU64::new(1);

/// In-process revision of stored provider keys; bumped by every write or delete.
pub fn provider_api_key_storage_revision() -> u64 {
    PROVIDER_API_KEY_STORAGE_REVISION.load(Ordering::Acquire)
}

fn mark_provider_api_key_storage_changed() {
    PROVIDER_API_KEY_STORAGE_REVISION.fetch_add(1, Ordering::AcqRel);
}

#[derive(Debug, Error)]
pub enum RefreshTokenError {
    #[error("{0}")]
    Permanent(#[from] RefreshTokenFailedError),
    #[error(transparent)]
    Transient(#[from] std::io::Error),
    /// Denied by application policy; neither retry nor cache as a credential failure.
    #[error(transparent)]
    Policy(#[from] codex_http_client::NetworkPolicyDenied),
}

/// Error returned when constructing an [`AuthManager`] from resolved configuration.
#[derive(Debug, Error)]
#[error(transparent)]
pub struct AuthManagerInitializationError(AuthManagerInitializationErrorSource);

#[derive(Debug, Error)]
enum AuthManagerInitializationErrorSource {
    #[error(transparent)]
    WorkloadIdentityConfiguration(WorkloadIdentitySessionError),
    #[error(transparent)]
    InitialAuth(RefreshTokenError),
}

impl From<WorkloadIdentitySessionError> for AuthManagerInitializationError {
    fn from(error: WorkloadIdentitySessionError) -> Self {
        Self(AuthManagerInitializationErrorSource::WorkloadIdentityConfiguration(error))
    }
}

impl From<RefreshTokenError> for AuthManagerInitializationError {
    fn from(error: RefreshTokenError) -> Self {
        Self(AuthManagerInitializationErrorSource::InitialAuth(error))
    }
}

impl From<AuthManagerInitializationError> for std::io::Error {
    fn from(error: AuthManagerInitializationError) -> Self {
        Self::other(error)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExternalAuthRefreshReason {
    Unauthorized,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExternalAuthRefreshContext {
    pub reason: ExternalAuthRefreshReason,
    pub previous_account_id: Option<String>,
}

/// Pluggable auth provider used by `AuthManager` for externally managed auth flows.
///
/// Implementations own the current auth value and any source-specific refresh mechanism.
pub trait ExternalAuth: Send + Sync {
    /// Returns the provider's current auth value.
    fn resolve(&self) -> ExternalAuthFuture<'_, CodexAuth>;

    /// Refreshes auth and makes the returned value current for future `resolve()` calls.
    fn refresh(&self, context: ExternalAuthRefreshContext) -> ExternalAuthFuture<'_, CodexAuth>;

    /// Maps a provider error into the retry policy used by external-auth reload and recovery.
    fn classify_error(&self, error: std::io::Error) -> RefreshTokenError {
        RefreshTokenError::Transient(error)
    }
}

pub type ExternalAuthFuture<'a, T> = Pin<Box<dyn Future<Output = std::io::Result<T>> + Send + 'a>>;

fn permanent_external_auth_error(message: impl Into<String>) -> RefreshTokenError {
    RefreshTokenError::Permanent(RefreshTokenFailedError::new(
        RefreshTokenFailedReason::Other,
        message,
    ))
}

impl RefreshTokenError {
    pub fn failed_reason(&self) -> Option<RefreshTokenFailedReason> {
        match self {
            Self::Permanent(error) => Some(error.reason),
            Self::Transient(_) | Self::Policy(_) => None,
        }
    }
}

impl From<codex_http_client::HttpError> for RefreshTokenError {
    fn from(error: codex_http_client::HttpError) -> Self {
        match error {
            codex_http_client::HttpError::Policy(denied) => Self::Policy(denied),
            error => Self::Transient(std::io::Error::other(error)),
        }
    }
}

impl From<RefreshTokenError> for std::io::Error {
    fn from(err: RefreshTokenError) -> Self {
        match err {
            RefreshTokenError::Permanent(failed) => std::io::Error::other(failed),
            RefreshTokenError::Transient(inner) => inner,
            RefreshTokenError::Policy(error) => {
                std::io::Error::new(std::io::ErrorKind::PermissionDenied, error)
            }
        }
    }
}

impl CodexAuth {
    #[allow(clippy::too_many_arguments)]
    async fn from_auth_dot_json(
        codex_home: &Path,
        auth_dot_json: AuthDotJson,
        auth_credentials_store_mode: AuthCredentialsStoreMode,
        chatgpt_base_url: Option<&str>,
        keyring_backend_kind: AuthKeyringBackendKind,
        agent_identity_authapi_base_url: Option<&str>,
        auth_route_config: &AuthRouteConfig,
        cached_auth: Option<&CodexAuth>,
    ) -> std::io::Result<Self> {
        Self::from_auth_dot_json_with_storage(
            codex_home,
            auth_dot_json,
            auth_credentials_store_mode,
            chatgpt_base_url,
            keyring_backend_kind,
            agent_identity_authapi_base_url,
            auth_route_config,
            cached_auth,
            /*storage*/ None,
        )
        .await
    }

    /// Like [`Self::from_auth_dot_json`]; `storage` (a named account's vault
    /// store, PF-84-S06) replaces the store a ChatGPT login refreshes into.
    #[allow(clippy::too_many_arguments)]
    async fn from_auth_dot_json_with_storage(
        codex_home: &Path,
        auth_dot_json: AuthDotJson,
        auth_credentials_store_mode: AuthCredentialsStoreMode,
        chatgpt_base_url: Option<&str>,
        keyring_backend_kind: AuthKeyringBackendKind,
        agent_identity_authapi_base_url: Option<&str>,
        auth_route_config: &AuthRouteConfig,
        cached_auth: Option<&CodexAuth>,
        storage: Option<Arc<dyn AuthStorageBackend>>,
    ) -> std::io::Result<Self> {
        let auth_mode = auth_dot_json.resolved_mode();
        if auth_mode == AuthMode::ApiKey {
            let Some(api_key) = auth_dot_json.openai_api_key.as_deref() else {
                return Err(std::io::Error::other("API key auth is missing a key."));
            };
            return Ok(Self::from_api_key(api_key));
        }
        if auth_mode == AuthMode::AgentIdentity {
            let Some(agent_identity) = auth_dot_json.agent_identity.clone() else {
                return Err(std::io::Error::other(
                    "agent identity auth is missing agent identity auth material.",
                ));
            };
            let base_url = chatgpt_base_url
                .unwrap_or(ChatGptEnvironment::default().chatgpt_base_url())
                .trim_end_matches('/')
                .to_string();
            let agent_identity_authapi_base_url =
                require_agent_identity_authapi_base_url(agent_identity_authapi_base_url)?;
            match agent_identity {
                AgentIdentityStorage::Jwt(jwt) => {
                    let auth = AgentIdentityAuth::from_jwt(
                        &jwt,
                        &base_url,
                        agent_identity_authapi_base_url,
                        auth_route_config,
                    )
                    .await?;
                    return Ok(Self::AgentIdentity(auth));
                }
                AgentIdentityStorage::Record(record) => {
                    let auth = AgentIdentityAuth::from_record(
                        record,
                        agent_identity_authapi_base_url,
                        auth_route_config,
                    )
                    .await?;
                    return Ok(Self::AgentIdentity(auth));
                }
            }
        }
        if auth_mode == AuthMode::PersonalAccessToken {
            let Some(personal_access_token) = auth_dot_json.personal_access_token.as_deref() else {
                return Err(std::io::Error::other(
                    "personal access token auth is missing a personal access token.",
                ));
            };
            return Ok(Self::PersonalAccessToken(
                PersonalAccessTokenAuth::load_with_cached(
                    personal_access_token,
                    auth_route_config,
                    cached_auth,
                )
                .await?,
            ));
        }
        if auth_mode == AuthMode::BedrockApiKey {
            let Some(auth) = auth_dot_json.bedrock_api_key else {
                return Err(std::io::Error::other(
                    "Bedrock API key auth is missing a Bedrock API key.",
                ));
            };
            return Ok(Self::BedrockApiKey(auth));
        }
        if auth_mode == AuthMode::BedrockAccessKeys {
            let Some(auth) = auth_dot_json.bedrock_access_keys else {
                return Err(std::io::Error::other(
                    "Bedrock access keys auth is missing AWS access keys.",
                ));
            };
            return Ok(Self::BedrockAccessKeys(auth));
        }
        if auth_mode == AuthMode::Headers {
            return Err(std::io::Error::other(
                "externally provided auth cannot be loaded from auth storage.",
            ));
        }

        let storage_mode = auth_dot_json.storage_mode(auth_credentials_store_mode);
        let provenance = auth_dot_json
            .tokens
            .as_ref()
            .and_then(|tokens| tokens.oauth_provenance.as_ref());
        // Reuse only the transport, not the credential state or storage backend: the
        // selected source can change even when its credential document is identical.
        let cached_client = cached_auth
            .filter(|auth| auth.get_current_auth_json().as_ref() == Some(&auth_dot_json))
            .and_then(|auth| match auth {
                Self::Chatgpt(auth) => Some(auth.state.client.clone()),
                Self::ChatgptAuthTokens(auth) => Some(auth.state.client.clone()),
                _ => None,
            });
        let client = match cached_client {
            Some(client) => client,
            None => {
                create_default_auth_client(&refresh_token_endpoint(provenance), auth_route_config)?
            }
        };
        let state = ChatgptAuthState {
            auth_dot_json: Arc::new(Mutex::new(Some(auth_dot_json))),
            client,
        };

        match auth_mode {
            AuthMode::Chatgpt => {
                let storage = storage.unwrap_or_else(|| {
                    create_auth_storage(
                        codex_home.to_path_buf(),
                        storage_mode,
                        keyring_backend_kind,
                    )
                });
                Ok(Self::Chatgpt(ChatgptAuth { state, storage }))
            }
            AuthMode::ChatgptAuthTokens => Ok(Self::ChatgptAuthTokens(ChatgptAuthTokens { state })),
            AuthMode::ApiKey => unreachable!("api key mode is handled above"),
            AuthMode::Headers => {
                unreachable!("externally provided auth is never loaded from auth storage")
            }
            AuthMode::AgentIdentity => unreachable!("agent identity mode is handled above"),
            AuthMode::PersonalAccessToken => {
                unreachable!("personal access token mode is handled above")
            }
            AuthMode::BedrockApiKey => unreachable!("bedrock api key mode is handled above"),
            AuthMode::BedrockAccessKeys => {
                unreachable!("bedrock access keys mode is handled above")
            }
        }
    }

    pub async fn from_auth_storage(
        codex_home: &Path,
        auth_credentials_store_mode: AuthCredentialsStoreMode,
        chatgpt_base_url: Option<&str>,
        keyring_backend_kind: AuthKeyringBackendKind,
        auth_route_config: &AuthRouteConfig,
    ) -> std::io::Result<Option<Self>> {
        let agent_identity_authapi_base_url =
            agent_identity_authapi_base_url(chatgpt_base_url).ok();
        load_auth(
            codex_home,
            /*enable_codex_api_key_env*/ false,
            auth_credentials_store_mode,
            /*allowed_login_methods*/ None,
            /*forced_chatgpt_workspace_id*/ None,
            chatgpt_base_url,
            keyring_backend_kind,
            agent_identity_authapi_base_url.as_deref(),
            auth_route_config,
            /*cached_auth*/ None,
            /*allowed_auth_modes*/ None,
        )
        .await
    }

    pub async fn from_agent_identity_jwt(
        jwt: &str,
        chatgpt_base_url: Option<&str>,
        auth_route_config: &AuthRouteConfig,
    ) -> std::io::Result<Self> {
        let agent_identity_authapi_base_url = agent_identity_authapi_base_url(chatgpt_base_url)?;
        Self::from_agent_identity_jwt_with_authapi_base_url(
            jwt,
            chatgpt_base_url,
            &agent_identity_authapi_base_url,
            auth_route_config,
        )
        .await
    }

    async fn from_agent_identity_jwt_with_authapi_base_url(
        jwt: &str,
        chatgpt_base_url: Option<&str>,
        agent_identity_authapi_base_url: &str,
        auth_route_config: &AuthRouteConfig,
    ) -> std::io::Result<Self> {
        let base_url = chatgpt_base_url
            .unwrap_or(ChatGptEnvironment::default().chatgpt_base_url())
            .trim_end_matches('/')
            .to_string();
        Ok(Self::AgentIdentity(
            AgentIdentityAuth::from_jwt(
                jwt,
                &base_url,
                agent_identity_authapi_base_url,
                auth_route_config,
            )
            .await?,
        ))
    }

    pub async fn from_personal_access_token(
        access_token: &str,
        auth_route_config: &AuthRouteConfig,
    ) -> std::io::Result<Self> {
        Ok(Self::PersonalAccessToken(
            PersonalAccessTokenAuth::load(access_token, auth_route_config).await?,
        ))
    }

    /// Returns the effective backend auth mode.
    ///
    /// Externally managed ChatGPT tokens are normalized to [`AuthMode::Chatgpt`].
    pub fn auth_mode(&self) -> AuthMode {
        match self {
            Self::ApiKey(_) => AuthMode::ApiKey,
            Self::Chatgpt(_) | Self::ChatgptAuthTokens(_) => AuthMode::Chatgpt,
            Self::Headers(_) => AuthMode::Headers,
            Self::AgentIdentity(_) => AuthMode::AgentIdentity,
            Self::PersonalAccessToken(_) => AuthMode::PersonalAccessToken,
            Self::BedrockApiKey(_) => AuthMode::BedrockApiKey,
            Self::BedrockAccessKeys(_) => AuthMode::BedrockAccessKeys,
        }
    }

    /// Returns the precise kind of credentials backing this authentication.
    pub fn api_auth_mode(&self) -> AuthMode {
        match self {
            Self::ApiKey(_) => AuthMode::ApiKey,
            Self::Chatgpt(_) => AuthMode::Chatgpt,
            Self::ChatgptAuthTokens(_) => AuthMode::ChatgptAuthTokens,
            Self::Headers(_) => AuthMode::Headers,
            Self::AgentIdentity(_) => AuthMode::AgentIdentity,
            Self::PersonalAccessToken(_) => AuthMode::PersonalAccessToken,
            Self::BedrockApiKey(_) => AuthMode::BedrockApiKey,
            Self::BedrockAccessKeys(_) => AuthMode::BedrockAccessKeys,
        }
    }

    pub fn is_api_key_auth(&self) -> bool {
        self.auth_mode() == AuthMode::ApiKey
    }

    pub fn is_personal_access_token_auth(&self) -> bool {
        self.auth_mode() == AuthMode::PersonalAccessToken
    }

    pub fn is_chatgpt_auth(&self) -> bool {
        self.api_auth_mode().has_chatgpt_account()
    }

    pub fn uses_codex_backend(&self) -> bool {
        self.api_auth_mode().uses_codex_backend()
    }

    pub fn is_external_chatgpt_tokens(&self) -> bool {
        matches!(self, Self::ChatgptAuthTokens(_))
    }

    fn supports_unauthorized_recovery(&self) -> bool {
        matches!(
            self,
            Self::Chatgpt(_) | Self::ChatgptAuthTokens(_) | Self::Headers(_)
        )
    }

    /// Returns `None` if `auth_mode() != AuthMode::ApiKey`.
    pub fn api_key(&self) -> Option<&str> {
        match self {
            Self::ApiKey(auth) => Some(auth.api_key.as_str()),
            Self::Chatgpt(_)
            | Self::ChatgptAuthTokens(_)
            | Self::Headers(_)
            | Self::AgentIdentity(_)
            | Self::PersonalAccessToken(_)
            | Self::BedrockApiKey(_)
            | Self::BedrockAccessKeys(_) => None,
        }
    }

    /// Returns `Err` if token-backed ChatGPT auth is unavailable.
    pub fn get_token_data(&self) -> Result<TokenData, std::io::Error> {
        let auth_dot_json: Option<AuthDotJson> = self.get_current_auth_json();
        match auth_dot_json {
            Some(AuthDotJson {
                tokens: Some(tokens),
                last_refresh: Some(_),
                ..
            }) => Ok(workspace_switching::selected_workspace_tokens(tokens)),
            _ => Err(std::io::Error::other("Token data is not available.")),
        }
    }

    /// Returns the token string used for bearer authentication.
    pub fn get_token(&self) -> Result<String, std::io::Error> {
        match self {
            Self::ApiKey(auth) => Ok(auth.api_key.clone()),
            Self::Chatgpt(_) | Self::ChatgptAuthTokens(_) => {
                let access_token = self.get_token_data()?.access_token;
                Ok(access_token)
            }
            Self::AgentIdentity(_) => Err(std::io::Error::other(
                "agent identity auth does not expose a bearer token",
            )),
            Self::Headers(_) => Err(std::io::Error::other(
                "header auth does not expose a bearer token",
            )),
            Self::PersonalAccessToken(auth) => Ok(auth.access_token().to_string()),
            Self::BedrockApiKey(_) | Self::BedrockAccessKeys(_) => Err(std::io::Error::other(
                "Bedrock API key auth does not expose a Codex bearer token",
            )),
        }
    }

    /// Returns `None` if Codex backend auth does not expose an account id.
    pub fn get_account_id(&self) -> Option<String> {
        match self {
            Self::Headers(headers) => headers
                .headers()
                .get(CHATGPT_ACCOUNT_ID_HEADER)
                .and_then(|value| value.to_str().ok())
                .filter(|account_id| !account_id.is_empty() && account_id.trim() == *account_id)
                .map(ToOwned::to_owned),
            Self::AgentIdentity(auth) => Some(auth.account_id().to_string()),
            Self::PersonalAccessToken(auth) => Some(auth.account_id().to_string()),
            _ => self.get_current_token_data().and_then(|t| t.account_id),
        }
    }

    /// Returns whether auth carries a FedRAMP routing hint from a claim or API-key prefix.
    pub fn is_fedramp_account(&self) -> bool {
        match self {
            Self::ApiKey(auth) => is_fedramp_api_key(&auth.api_key),
            Self::Headers(_) => false,
            Self::AgentIdentity(auth) => auth.is_fedramp_account(),
            Self::PersonalAccessToken(auth) => auth.is_fedramp_account(),
            _ => self
                .get_current_token_data()
                .is_some_and(|t| t.id_token.is_fedramp_account()),
        }
    }

    /// Returns `None` if Codex backend auth does not expose an account email.
    pub fn get_account_email(&self) -> Option<String> {
        match self {
            Self::Headers(_) => None,
            Self::AgentIdentity(auth) => auth.email().map(str::to_string),
            Self::PersonalAccessToken(auth) => auth.email().map(str::to_string),
            _ => self.get_current_token_data().and_then(|t| t.id_token.email),
        }
    }

    /// Returns `None` if Codex backend auth does not expose a ChatGPT user id.
    pub fn get_chatgpt_user_id(&self) -> Option<String> {
        match self {
            Self::Headers(_) => None,
            Self::AgentIdentity(auth) => Some(auth.chatgpt_user_id().to_string()),
            Self::PersonalAccessToken(auth) => Some(auth.chatgpt_user_id().to_string()),
            _ => self
                .get_current_token_data()
                .and_then(|t| t.id_token.chatgpt_user_id),
        }
    }

    /// Returns the access token's opaque account-user identity only when its workspace
    /// matches the selected account. Missing claims never fall back to a user id.
    /// Unlike `get_chatgpt_user_id`, this identifies one workspace membership, so keys
    /// are not shared across a person's workspaces. Workload-identity exchange claims
    /// describe an agent identity and cannot select a human verification credential.
    /// This is local identity selection, not access-token or proof validation.
    pub fn get_chatgpt_account_user_id(&self) -> Option<String> {
        let tokens = self.get_current_token_data()?;
        parse_chatgpt_account_user_id(&tokens.access_token, tokens.account_id.as_deref()?)
            .ok()
            .flatten()
    }

    /// Account-facing plan classification derived from the current auth.
    /// Returns a high-level `AccountPlanType` (e.g., Free/Plus/Pro/Team/…)
    /// for UI or product decisions based on the user's subscription.
    pub fn account_plan_type(&self) -> Option<AccountPlanType> {
        if matches!(self, Self::Headers(_)) {
            return None;
        }
        if let Self::AgentIdentity(auth) = self {
            return Some(auth.plan_type());
        }
        if let Self::PersonalAccessToken(auth) = self {
            return Some(auth.plan_type());
        }

        self.get_current_token_data().map(|t| {
            t.id_token
                .chatgpt_plan_type
                .map(AccountPlanType::from)
                .unwrap_or(AccountPlanType::Unknown)
        })
    }

    pub fn is_workspace_account(&self) -> bool {
        self.account_plan_type()
            .is_some_and(AccountPlanType::is_workspace_account)
    }

    /// Returns `None` if token-backed ChatGPT auth is unavailable.
    fn get_current_auth_json(&self) -> Option<AuthDotJson> {
        let state = match self {
            Self::Chatgpt(auth) => &auth.state,
            Self::ChatgptAuthTokens(auth) => &auth.state,
            Self::ApiKey(_)
            | Self::Headers(_)
            | Self::AgentIdentity(_)
            | Self::PersonalAccessToken(_)
            | Self::BedrockApiKey(_)
            | Self::BedrockAccessKeys(_) => return None,
        };
        #[expect(clippy::unwrap_used)]
        state.auth_dot_json.lock().unwrap().clone()
    }

    /// Returns `None` if token-backed ChatGPT auth is unavailable.
    fn get_current_token_data(&self) -> Option<TokenData> {
        self.get_current_auth_json()
            .and_then(|t| t.tokens)
            .map(workspace_switching::selected_workspace_tokens)
    }

    fn stored_managed_chatgpt_agent_identity_record(
        &self,
        account_id: &str,
    ) -> Option<AgentIdentityAuthRecord> {
        self.get_current_auth_json()
            .and_then(|auth| auth.agent_identity)
            .and_then(|identity| identity.as_record().cloned())
            .filter(|identity| identity.account_id == account_id)
    }

    fn persist_managed_chatgpt_agent_identity_record(
        &self,
        record: AgentIdentityAuthRecord,
    ) -> std::io::Result<()> {
        if let Self::Chatgpt(chatgpt_auth) = self {
            chatgpt_auth.persist_agent_identity_record(record)?;
        }
        Ok(())
    }

    async fn agent_identity_auth(
        &self,
        policy: AgentIdentityAuthPolicy,
        agent_identity_authapi_base_url: Option<&str>,
        forced_chatgpt_workspace_id: Option<Vec<String>>,
        auth_route_config: &AuthRouteConfig,
        session_source: SessionSource,
    ) -> std::io::Result<Option<AgentIdentityAuth>> {
        match self {
            Self::AgentIdentity(auth) => Ok(Some(auth.clone())),
            Self::ApiKey(_)
            | Self::ChatgptAuthTokens(_)
            | Self::Headers(_)
            | Self::PersonalAccessToken(_)
            | Self::BedrockApiKey(_)
            | Self::BedrockAccessKeys(_) => Ok(None),
            Self::Chatgpt(_) => {
                if policy == AgentIdentityAuthPolicy::JwtOnly {
                    return Ok(None);
                }
                self.ensure_managed_chatgpt_agent_identity(
                    require_agent_identity_authapi_base_url(agent_identity_authapi_base_url)?,
                    forced_chatgpt_workspace_id,
                    auth_route_config,
                    session_source,
                )
                .await
                .map(Some)
            }
        }
    }

    async fn ensure_managed_chatgpt_agent_identity(
        &self,
        agent_identity_authapi_base_url: &str,
        forced_chatgpt_workspace_id: Option<Vec<String>>,
        auth_route_config: &AuthRouteConfig,
        session_source: SessionSource,
    ) -> std::io::Result<AgentIdentityAuth> {
        let binding =
            ManagedChatGptAgentIdentityBinding::from_auth(self, forced_chatgpt_workspace_id)
                .ok_or_else(|| std::io::Error::other("ChatGPT auth is unavailable"))?;

        // JWT auth is loaded as CodexAuth::AgentIdentity; this path only reuses
        // records created by the managed ChatGPT Agent Identity bootstrap.
        if let Some(record) = self.stored_managed_chatgpt_agent_identity_record(&binding.account_id)
            && record_matches_managed_chatgpt_binding(&record, &binding)
        {
            let should_persist = record_needs_task_registration(&record);
            let auth = AgentIdentityAuth::from_record(
                record,
                agent_identity_authapi_base_url,
                auth_route_config,
            )
            .await
            .map_err(|err| classify_bootstrap_error("agent task registration", err))?;
            if should_persist {
                self.persist_managed_chatgpt_agent_identity_record(auth.record().clone())?;
            }
            return Ok(auth);
        }

        let auth = register_managed_chatgpt_agent_identity(
            binding,
            agent_identity_authapi_base_url,
            session_source,
            auth_route_config,
        )
        .await?;
        self.persist_managed_chatgpt_agent_identity_record(auth.record().clone())?;
        Ok(auth)
    }

    /// Consider this private to integration tests.
    pub fn create_dummy_chatgpt_auth_for_testing() -> Self {
        let auth_dot_json = AuthDotJson {
            auth_mode: Some(AuthMode::Chatgpt),
            openai_api_key: None,
            tokens: Some(TokenData {
                id_token: Default::default(),
                access_token: "Access Token".to_string(),
                refresh_token: "test".to_string(),
                oauth_provenance: None,
                login_owner_id: None,
                account_id: Some("account_id".to_string()),
            }),
            last_refresh: Some(Utc::now()),
            agent_identity: None,
            personal_access_token: None,
            bedrock_api_key: None,
            bedrock_access_keys: None,
        };

        let state = ChatgptAuthState {
            auth_dot_json: Arc::new(Mutex::new(Some(auth_dot_json))),
            client: create_client(),
        };
        let dummy_auth_id = NEXT_DUMMY_AUTH_ID.fetch_add(1, Ordering::Relaxed);
        let storage = create_auth_storage(
            PathBuf::from(format!("dummy-chatgpt-auth-{dummy_auth_id}")),
            AuthCredentialsStoreMode::Ephemeral,
            AuthKeyringBackendKind::default(),
        );
        Self::Chatgpt(ChatgptAuth { state, storage })
    }

    /// Constructs in-memory ChatGPT auth from externally managed tokens.
    pub fn from_external_chatgpt_tokens(
        access_token: &str,
        chatgpt_account_id: &str,
        chatgpt_plan_type: Option<&str>,
    ) -> std::io::Result<Self> {
        let auth_dot_json = AuthDotJson::from_external_access_token(
            access_token,
            chatgpt_account_id,
            chatgpt_plan_type,
        )?;
        let state = ChatgptAuthState {
            auth_dot_json: Arc::new(Mutex::new(Some(auth_dot_json))),
            client: create_client(),
        };
        Ok(Self::ChatgptAuthTokens(ChatgptAuthTokens { state }))
    }

    pub fn from_api_key(api_key: &str) -> Self {
        Self::ApiKey(ApiKeyAuth {
            api_key: api_key.to_owned(),
            env_var: None,
        })
    }

    /// An API key read from `env_var` rather than from stored credentials.
    pub fn from_api_key_env(api_key: &str, env_var: &'static str) -> Self {
        Self::ApiKey(ApiKeyAuth {
            api_key: api_key.to_owned(),
            env_var: Some(env_var),
        })
    }

    /// The environment variable an API key login was read from, if any.
    pub fn api_key_env_var(&self) -> Option<&'static str> {
        match self {
            Self::ApiKey(auth) => auth.env_var,
            _ => None,
        }
    }

    /// Whether this is the ambient `OPENAI_API_KEY` fallback. Model providers
    /// send it only on the first-party OpenAI auth path.
    pub fn is_openai_api_key_env_fallback(&self) -> bool {
        self.api_key_env_var() == Some(OPENAI_API_KEY_ENV_VAR)
    }
}

impl ManagedChatGptAgentIdentityBinding {
    fn from_auth(auth: &CodexAuth, forced_workspace_id: Option<Vec<String>>) -> Option<Self> {
        if !auth.is_chatgpt_auth() {
            return None;
        }

        let token_data = auth.get_token_data().ok()?;
        let forced_workspace_id =
            forced_workspace_id
                .as_deref()
                .and_then(|workspace_ids| match workspace_ids {
                    [workspace_id] if !workspace_id.is_empty() => Some(workspace_id.clone()),
                    _ => None,
                });
        let account_id = forced_workspace_id
            .or(token_data
                .account_id
                .clone()
                .filter(|value| !value.is_empty()))
            .or(token_data.id_token.chatgpt_account_id.clone())?;
        let chatgpt_user_id = token_data
            .id_token
            .chatgpt_user_id
            .clone()
            .filter(|value| !value.is_empty())?;

        Some(Self {
            account_id,
            chatgpt_user_id,
            email: token_data.id_token.email.clone(),
            plan_type: auth.account_plan_type().unwrap_or(AccountPlanType::Unknown),
            chatgpt_account_is_fedramp: auth.is_fedramp_account(),
            access_token: token_data.access_token,
        })
    }
}

impl ChatgptAuth {
    fn current_auth_json(&self) -> Option<AuthDotJson> {
        #[expect(clippy::unwrap_used)]
        self.state.auth_dot_json.lock().unwrap().clone()
    }

    fn current_token_data(&self) -> Option<TokenData> {
        self.current_auth_json().and_then(|auth| auth.tokens)
    }

    fn storage(&self) -> &Arc<dyn AuthStorageBackend> {
        &self.storage
    }

    fn client(&self) -> &HttpClient {
        &self.state.client
    }

    fn persist_agent_identity_record(
        &self,
        record: AgentIdentityAuthRecord,
    ) -> std::io::Result<()> {
        persist_agent_identity_record(&self.state.auth_dot_json, &self.storage, record)
    }
}

fn persist_agent_identity_record(
    auth_dot_json: &Arc<Mutex<Option<AuthDotJson>>>,
    storage: &Arc<dyn AuthStorageBackend>,
    record: AgentIdentityAuthRecord,
) -> std::io::Result<()> {
    let _commit = storage.lock_for_write()?;
    let mut guard = auth_dot_json
        .lock()
        .map_err(|_| std::io::Error::other("failed to lock auth state"))?;
    // D171 (#1311): a named account's record (and its private key) is written
    // only onto the stored sign-in of the ChatGPT account and user it was made
    // for, in one vault step, and never brings back an account another process
    // removed. The default login is written only onto the sign-in it was made
    // for too (upstream: the account may change during registration).
    let named = storage.is_named_account();
    let previous_tokens = guard.as_ref().and_then(|auth| auth.tokens.clone());
    let updated = storage.update(&mut |auth| {
        let stored_binding = auth.tokens.as_ref().map(|tokens| {
            (
                tokens.account_id.as_deref(),
                tokens.id_token.chatgpt_user_id.as_deref(),
            )
        });
        let record_binding = (
            Some(record.account_id.as_str()),
            Some(record.chatgpt_user_id.as_str()),
        );
        if named && stored_binding != Some(record_binding) {
            return Err(std::io::Error::other(
                "the named account's sign-in changed; its agent identity was not saved",
            ));
        }
        if !named
            && let Some(previous) = previous_tokens.as_ref()
            && !auth
                .tokens
                .as_ref()
                .is_some_and(|current| same_login_for_persistence(previous, current))
        {
            return Err(std::io::Error::other(
                "Account changed during agent identity registration.",
            ));
        }
        auth.agent_identity = Some(AgentIdentityStorage::Record(record.clone()));
        Ok(())
    })?;
    let auth = match updated {
        Some(auth) => auth,
        None if named || previous_tokens.is_some() => {
            return Err(std::io::Error::other(
                "the sign-in was removed or signed out; its agent identity was not saved",
            ));
        }
        None => {
            let mut auth = guard
                .clone()
                .ok_or_else(|| std::io::Error::other("auth data is not available"))?;
            auth.agent_identity = Some(AgentIdentityStorage::Record(record));
            storage.save(&auth)?;
            auth
        }
    };
    // Publishing credentials belongs to AuthManager. Persisting identity metadata must
    // not reload different on-disk credentials into already selected auth values.
    match guard.as_mut() {
        Some(cached) => cached.agent_identity = auth.agent_identity,
        None => *guard = Some(auth),
    }
    Ok(())
}

pub const OPENAI_API_KEY_ENV_VAR: &str = "OPENAI_API_KEY";
pub const CODEX_API_KEY_ENV_VAR: &str = "CODEX_API_KEY";
pub const CODEX_ACCESS_TOKEN_ENV_VAR: &str = "CODEX_ACCESS_TOKEN";

/// Shown once per session when a request is authorised by the `OPENAI_API_KEY`
/// fallback, and again whenever the credential in use switches back to it.
pub const OPENAI_API_KEY_ENV_FALLBACK_NOTICE: &str = "Using OPENAI_API_KEY from your \
    environment — billed per use to that API key. Sign in or unset it to change.";

/// Whether `OPENAI_API_KEY` is the last-resort OpenAI credential.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OpenAiApiKeyEnv {
    /// Used only when no external or stored OpenAI credential exists.
    Fallback,
    /// Not read: storage-only loads, and configs that require ChatGPT login.
    Ignore,
}

impl OpenAiApiKeyEnv {
    fn for_forced_login_method(forced_login_method: Option<ForcedLoginMethod>) -> Self {
        if forced_login_method == Some(ForcedLoginMethod::Chatgpt) {
            Self::Ignore
        } else {
            Self::Fallback
        }
    }
}

pub fn read_openai_api_key_from_env() -> Option<String> {
    env::var(OPENAI_API_KEY_ENV_VAR)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

pub fn read_ambient_api_key_from_env() -> Option<String> {
    read_non_empty_env_var(AMBIENT_API_KEY_ENV_VAR)
}

pub fn read_codex_api_key_from_env() -> Option<String> {
    read_non_empty_env_var(CODEX_API_KEY_ENV_VAR)
}

pub fn read_codex_access_token_from_env() -> Option<String> {
    read_non_empty_env_var(CODEX_ACCESS_TOKEN_ENV_VAR)
}

fn auth_uses_api_key_from_env(auth: &CodexAuth) -> bool {
    if auth.api_key_env_var().is_some() {
        return true;
    }
    let Some(api_key) = auth.api_key() else {
        return false;
    };

    read_codex_api_key_from_env().as_deref() == Some(api_key)
        || read_ambient_api_key_from_env().as_deref() == Some(api_key)
}

fn read_non_empty_env_var(key: &str) -> Option<String> {
    env::var(key)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

/// Delete credentials for `codex_home` through the selected storage backend.
/// Returns `Ok(true)` if credentials were removed, `Ok(false)` if none were present.
pub fn logout(
    codex_home: &Path,
    auth_credentials_store_mode: AuthCredentialsStoreMode,
    keyring_backend_kind: AuthKeyringBackendKind,
) -> std::io::Result<bool> {
    if auth_credentials_store_mode == AuthCredentialsStoreMode::File && !codex_home.try_exists()? {
        return Ok(false);
    }
    let storage = create_auth_storage(
        codex_home.to_path_buf(),
        auth_credentials_store_mode,
        keyring_backend_kind,
    );
    let _commit = storage.lock_for_write()?;
    storage.delete()
}

/// Delete Codex auth plus provider credentials from legacy storage and the vault.
///
/// This is intentionally separate from `logout`: normal account logout must not erase
/// provider API keys that the user stored under `/providers`.
pub fn logout_all_credentials(
    codex_home: &Path,
    auth_credentials_store_mode: AuthCredentialsStoreMode,
    keyring_backend_kind: AuthKeyringBackendKind,
) -> std::io::Result<bool> {
    let removed_auth = logout(
        codex_home,
        auth_credentials_store_mode,
        keyring_backend_kind,
    )?;
    let removed_provider = delete_provider_auth_if_exists(codex_home)?;
    // Also wipe any provider keys stored in the encrypted vault (best-effort: a missing
    // keyring in CI must not fail logout).
    let removed_vault_provider =
        super::provider_key_vault::delete_all_provider_keys(codex_home).unwrap_or(false);
    if removed_provider || removed_vault_provider {
        mark_provider_api_key_storage_changed();
    }
    Ok(removed_auth || removed_provider || removed_vault_provider)
}

pub async fn logout_with_revoke(
    codex_home: &Path,
    auth_credentials_store_mode: AuthCredentialsStoreMode,
    keyring_backend_kind: AuthKeyringBackendKind,
    auth_route_config: &AuthRouteConfig,
) -> std::io::Result<bool> {
    logout_with_revoke_common(
        codex_home,
        auth_credentials_store_mode,
        keyring_backend_kind,
        /*include_provider_credentials*/ false,
        auth_route_config,
    )
    .await
}

pub async fn logout_with_revoke_all_credentials(
    codex_home: &Path,
    auth_credentials_store_mode: AuthCredentialsStoreMode,
    keyring_backend_kind: AuthKeyringBackendKind,
    auth_route_config: &AuthRouteConfig,
) -> std::io::Result<bool> {
    logout_with_revoke_common(
        codex_home,
        auth_credentials_store_mode,
        keyring_backend_kind,
        /*include_provider_credentials*/ true,
        auth_route_config,
    )
    .await
}

async fn logout_with_revoke_common(
    codex_home: &Path,
    auth_credentials_store_mode: AuthCredentialsStoreMode,
    keyring_backend_kind: AuthKeyringBackendKind,
    include_provider_credentials: bool,
    auth_route_config: &AuthRouteConfig,
) -> std::io::Result<bool> {
    let auth_dot_json = match load_auth_dot_json(
        codex_home,
        auth_credentials_store_mode,
        keyring_backend_kind,
    ) {
        Ok(auth_dot_json) => auth_dot_json,
        Err(err) => {
            tracing::warn!("failed to load stored auth during logout: {err}");
            None
        }
    };
    if let Err(err) = revoke_auth_tokens(auth_dot_json.as_ref(), auth_route_config).await {
        tracing::warn!("failed to revoke auth tokens during logout: {err}");
    }
    logout_all_stores(
        codex_home,
        auth_credentials_store_mode,
        keyring_backend_kind,
        include_provider_credentials,
    )
}

/// Store API-key credentials through the selected storage backend.
pub fn login_with_api_key(
    codex_home: &Path,
    api_key: &str,
    auth_credentials_store_mode: AuthCredentialsStoreMode,
    keyring_backend_kind: AuthKeyringBackendKind,
) -> std::io::Result<()> {
    let auth_dot_json = AuthDotJson {
        auth_mode: Some(AuthMode::ApiKey),
        openai_api_key: Some(api_key.to_string()),
        tokens: None,
        last_refresh: None,
        agent_identity: None,
        personal_access_token: None,
        bedrock_api_key: None,
        bedrock_access_keys: None,
    };
    save_auth(
        codex_home,
        &auth_dot_json,
        auth_credentials_store_mode,
        keyring_backend_kind,
    )
}

pub fn login_with_provider_api_key(
    codex_home: &Path,
    provider_key_id: &str,
    api_key: &str,
    _auth_credentials_store_mode: AuthCredentialsStoreMode,
    _keyring_backend_kind: AuthKeyringBackendKind,
) -> std::io::Result<()> {
    let provider_key_id = provider_key_id.trim();
    if provider_key_id.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "provider key id cannot be empty",
        ));
    }

    let api_key = api_key.trim();
    if api_key.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "provider API key cannot be empty",
        ));
    }

    save_provider_api_key(codex_home, provider_key_id, api_key)?;
    mark_provider_api_key_storage_changed();
    Ok(())
}

/// Return the provider key identifier represented by a namespaced vault label.
///
/// Provider keys are normalized to lowercase in the vault, while the auth manager indexes them
/// by their environment-style uppercase identifier. Keeping the reverse mapping here ensures
/// generic `/vault` mutations use the same cache-invalidation boundary as `/providers`.
pub fn provider_api_key_id_from_vault_label(label: &str) -> Option<String> {
    let provider_key_id = label
        .trim()
        .strip_prefix(super::provider_key_vault::PROVIDER_LABEL_PREFIX)?;
    if provider_key_id.is_empty()
        || !provider_key_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
    {
        return None;
    }
    Some(provider_key_id.to_ascii_uppercase())
}

/// Remove one provider API key from both encrypted and legacy storage.
///
/// This is intentionally narrower than logout: account authentication and every other provider
/// credential remain untouched.
pub fn delete_provider_api_key(codex_home: &Path, provider_key_id: &str) -> std::io::Result<bool> {
    let provider_key_id = provider_key_id.trim();
    if provider_key_id.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "provider key id cannot be empty",
        ));
    }
    let removed_vault = super::provider_key_vault::delete_provider_key(codex_home, provider_key_id);
    let removed_legacy = legacy_delete_provider_key(codex_home, provider_key_id)?;
    let removed = match removed_vault {
        Ok(removed_vault) => {
            legacy_clear_provider_key_tombstone(codex_home, provider_key_id)?;
            removed_vault || removed_legacy
        }
        Err(error) => {
            // An encrypted vault can become temporarily unavailable (for example after a
            // copied CODEX_HOME loses its keyring binding). Persist a non-secret tombstone so
            // the inaccessible copy cannot silently become active again if the vault later
            // recovers. A subsequent explicit login clears the tombstone.
            tracing::warn!(
                provider_key_id,
                %error,
                "provider vault was unavailable during credential deletion; suppressing the stale vault entry"
            );
            legacy_mark_provider_key_deleted(codex_home, provider_key_id)? || removed_legacy
        }
    };
    if removed {
        mark_provider_api_key_storage_changed();
    }
    Ok(removed)
}

/// Immediately suppress one stored provider key without waiting for encrypted-vault cleanup.
///
/// This durable tombstone is useful at destructive UI boundaries: readers stop returning the
/// credential as soon as this function succeeds, while the potentially slower encrypted-vault
/// deletion can finish in the background. A later explicit provider login clears the tombstone.
pub fn suppress_provider_api_key(
    codex_home: &Path,
    provider_key_id: &str,
) -> std::io::Result<bool> {
    let provider_key_id = provider_key_id.trim();
    if provider_key_id.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "provider key id cannot be empty",
        ));
    }
    let inserted = legacy_mark_provider_key_deleted(codex_home, provider_key_id)?;
    if inserted {
        mark_provider_api_key_storage_changed();
    }
    Ok(inserted)
}

/// Validate and store personal-access-token or Agent Identity credentials through
/// the selected storage backend.
pub async fn login_with_access_token(
    codex_home: &Path,
    access_token: &str,
    auth_credentials_store_mode: AuthCredentialsStoreMode,
    forced_chatgpt_workspace_id: Option<&[String]>,
    chatgpt_base_url: Option<&str>,
    keyring_backend_kind: AuthKeyringBackendKind,
    auth_route_config: &AuthRouteConfig,
) -> std::io::Result<()> {
    let auth_dot_json = match classify_codex_access_token(access_token) {
        CodexAccessToken::PersonalAccessToken(access_token) => {
            let auth = PersonalAccessTokenAuth::load(access_token, auth_route_config).await?;
            ensure_auth_workspace_allowed(forced_chatgpt_workspace_id, auth.account_id())?;
            AuthDotJson {
                // Infer PAT auth from the credential field so older Codex builds can still
                // deserialize auth.json after a rollback.
                auth_mode: None,
                openai_api_key: None,
                tokens: None,
                last_refresh: None,
                agent_identity: None,
                personal_access_token: Some(access_token.to_string()),
                bedrock_api_key: None,
                bedrock_access_keys: None,
            }
        }
        CodexAccessToken::AgentIdentityJwt(jwt) => {
            let record = AgentIdentityAuthRecord::from_agent_identity_jwt(jwt)?;
            ensure_auth_workspace_allowed(forced_chatgpt_workspace_id, &record.account_id)?;
            let base_url = chatgpt_base_url
                .unwrap_or(ChatGptEnvironment::default().chatgpt_base_url())
                .trim_end_matches('/')
                .to_string();
            verified_record_from_jwt(jwt, &base_url, auth_route_config).await?;
            AuthDotJson {
                auth_mode: Some(AuthMode::AgentIdentity),
                openai_api_key: None,
                tokens: None,
                last_refresh: None,
                agent_identity: Some(AgentIdentityStorage::Jwt(jwt.to_string())),
                personal_access_token: None,
                bedrock_api_key: None,
                bedrock_access_keys: None,
            }
        }
    };
    save_auth(
        codex_home,
        &auth_dot_json,
        auth_credentials_store_mode,
        keyring_backend_kind,
    )
}

fn ensure_auth_workspace_allowed(
    expected_workspace_ids: Option<&[String]>,
    account_id: &str,
) -> std::io::Result<()> {
    crate::server::ensure_workspace_account_allowed(expected_workspace_ids, account_id)
        .map_err(|message| std::io::Error::new(std::io::ErrorKind::PermissionDenied, message))
}

fn ensure_agent_identity_workspace_allowed(
    expected_workspace_ids: Option<&[String]>,
    agent_identity: &AgentIdentityStorage,
) -> std::io::Result<()> {
    let Some(expected_workspace_ids) = expected_workspace_ids else {
        return Ok(());
    };

    match agent_identity {
        AgentIdentityStorage::Jwt(jwt) => {
            let record = AgentIdentityAuthRecord::from_agent_identity_jwt(jwt)?;
            ensure_auth_workspace_allowed(Some(expected_workspace_ids), &record.account_id)
        }
        AgentIdentityStorage::Record(record) => {
            ensure_auth_workspace_allowed(Some(expected_workspace_ids), &record.account_id)
        }
    }
}

/// Writes an in-memory auth payload for externally managed ChatGPT tokens.
pub fn login_with_chatgpt_auth_tokens(
    codex_home: &Path,
    access_token: &str,
    chatgpt_account_id: &str,
    chatgpt_plan_type: Option<&str>,
) -> std::io::Result<()> {
    let auth_dot_json = AuthDotJson::from_external_access_token(
        access_token,
        chatgpt_account_id,
        chatgpt_plan_type,
    )?;
    save_auth(
        codex_home,
        &auth_dot_json,
        AuthCredentialsStoreMode::Ephemeral,
        AuthKeyringBackendKind::default(),
    )
}

/// Persist the provided auth payload using the specified backend.
pub fn save_auth(
    codex_home: &Path,
    auth: &AuthDotJson,
    auth_credentials_store_mode: AuthCredentialsStoreMode,
    keyring_backend_kind: AuthKeyringBackendKind,
) -> std::io::Result<()> {
    let storage = create_auth_storage(
        codex_home.to_path_buf(),
        auth_credentials_store_mode,
        keyring_backend_kind,
    );
    let _commit = storage.lock_for_write()?;
    storage.save(auth)
}

/// Load the raw stored auth payload without applying environment overrides.
///
/// Returns `None` when no credentials are stored. Prefer `AuthManager` for
/// ordinary production reads; this helper is for tests and write-side
/// maintenance that must inspect the exact payload in storage.
pub fn load_auth_dot_json(
    codex_home: &Path,
    auth_credentials_store_mode: AuthCredentialsStoreMode,
    keyring_backend_kind: AuthKeyringBackendKind,
) -> std::io::Result<Option<AuthDotJson>> {
    let _commit = auth_storage_commit_guard()?;
    let storage = create_auth_storage(
        codex_home.to_path_buf(),
        auth_credentials_store_mode,
        keyring_backend_kind,
    );
    storage.load()
}

pub fn provider_api_key_from_auth_storage(
    codex_home: &Path,
    provider_key_id: &str,
    _auth_credentials_store_mode: AuthCredentialsStoreMode,
    _keyring_backend_kind: AuthKeyringBackendKind,
) -> std::io::Result<Option<String>> {
    if legacy_provider_key_is_deleted(codex_home, provider_key_id)? {
        return Ok(None);
    }
    // Prefer the encrypted vault; fall back to the legacy plaintext store for compatibility.
    if let Some(key) = super::provider_key_vault::read_provider_key(codex_home, provider_key_id)? {
        return Ok(Some(key));
    }
    legacy_provider_key(codex_home, provider_key_id)
}

/// Classify the configured OpenAI credential without exposing its secret or
/// handing callers a long-lived [`AuthManager`].
pub async fn openai_auth_metadata_from_config(
    config: &impl AuthManagerConfig,
    enable_codex_api_key_env: bool,
) -> OpenAiAuthMetadata {
    match AuthManager::shared_from_config(config, enable_codex_api_key_env).await {
        Ok(manager) => manager.openai_auth_metadata(),
        // Workload identity was selected but could not start.
        Err(_) => OpenAiAuthMetadata::RecoveryRequired,
    }
}

/// Inspect provider-key storage without revealing or decrypting the credential.
pub fn provider_api_key_metadata_from_auth_storage(
    codex_home: &Path,
    provider_key_id: &str,
) -> std::io::Result<ProviderApiKeyStorageMetadata> {
    if legacy_provider_key_is_deleted(codex_home, provider_key_id)? {
        return Ok(ProviderApiKeyStorageMetadata::Suppressed);
    }
    super::provider_key_vault::provider_key_metadata(codex_home, provider_key_id)
}

/// Inspect several provider keys without repeatedly opening the encrypted vault or legacy file.
///
/// Tombstones retain precedence over encrypted and legacy storage, matching the single-key API.
/// Only storage metadata is loaded; credential values are never read.
pub fn provider_api_key_metadata_snapshot_from_auth_storage(
    codex_home: &Path,
    provider_key_ids: impl IntoIterator<Item = impl AsRef<str>>,
) -> std::io::Result<ProviderApiKeyStorageMetadataSnapshot> {
    let provider_key_ids = provider_key_ids
        .into_iter()
        .map(|provider_key_id| provider_key_id.as_ref().to_string())
        .collect::<Vec<_>>();
    if provider_key_ids.is_empty() {
        return Ok(ProviderApiKeyStorageMetadataSnapshot {
            entries: HashMap::new(),
        });
    }
    let legacy = load_provider_auth(codex_home)?;
    let vault_ids = super::provider_key_vault::stored_provider_key_ids(codex_home);
    let entries = provider_key_ids
        .into_iter()
        .map(|provider_key_id| {
            let metadata = if legacy.deleted_api_keys.contains(&provider_key_id) {
                ProviderApiKeyStorageMetadata::Suppressed
            } else if vault_ids.contains(&provider_key_id.trim().to_ascii_uppercase()) {
                ProviderApiKeyStorageMetadata::Stored {
                    source: ProviderApiKeyStorageSource::EncryptedVault,
                }
            } else if legacy
                .api_keys
                .get(&provider_key_id)
                .is_some_and(|value| !value.trim().is_empty())
            {
                ProviderApiKeyStorageMetadata::Stored {
                    source: ProviderApiKeyStorageSource::LegacyPlaintext,
                }
            } else {
                ProviderApiKeyStorageMetadata::Missing
            };
            (provider_key_id, metadata)
        })
        .collect();
    Ok(ProviderApiKeyStorageMetadataSnapshot { entries })
}

/// Read a provider key from the legacy plaintext `provider_auth.json` store only.
pub(super) fn legacy_provider_key(
    codex_home: &Path,
    provider_key_id: &str,
) -> std::io::Result<Option<String>> {
    Ok(load_provider_auth(codex_home)?
        .api_keys
        .get(provider_key_id)
        .map(String::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(str::to_string))
}

pub(super) fn legacy_provider_key_is_present(
    codex_home: &Path,
    provider_key_id: &str,
) -> std::io::Result<bool> {
    Ok(load_provider_auth(codex_home)?
        .api_keys
        .get(provider_key_id)
        .is_some_and(|value| !value.trim().is_empty()))
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct ProviderAuthDotJson {
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    api_keys: HashMap<String, String>,
    #[serde(default, skip_serializing_if = "HashSet::is_empty")]
    deleted_api_keys: HashSet<String>,
}

fn provider_auth_file(codex_home: &Path) -> PathBuf {
    codex_home.join("provider_auth.json")
}

fn delete_provider_auth_if_exists(codex_home: &Path) -> std::io::Result<bool> {
    match std::fs::remove_file(provider_auth_file(codex_home)) {
        Ok(()) => Ok(true),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(err) => Err(err),
    }
}

fn load_provider_auth(codex_home: &Path) -> std::io::Result<ProviderAuthDotJson> {
    let provider_auth_file = provider_auth_file(codex_home);
    let contents = match std::fs::read_to_string(provider_auth_file) {
        Ok(contents) => contents,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            return Ok(ProviderAuthDotJson::default());
        }
        Err(err) => return Err(err),
    };

    serde_json::from_str(&contents).map_err(std::io::Error::other)
}

fn save_provider_auth(
    codex_home: &Path,
    provider_auth: &ProviderAuthDotJson,
) -> std::io::Result<()> {
    let provider_auth_file = provider_auth_file(codex_home);
    if let Some(parent) = provider_auth_file.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut json_data = serde_json::to_string_pretty(provider_auth)?;
    json_data.push('\n');
    let temporary = provider_auth_file.with_file_name(format!(
        ".provider_auth.json.{}.{}.tmp",
        std::process::id(),
        NEXT_PROVIDER_AUTH_TEMP_ID.fetch_add(1, Ordering::Relaxed)
    ));
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(&temporary)?;
    file.write_all(json_data.as_bytes())?;
    file.sync_all()?;
    if let Err(error) = std::fs::rename(&temporary, &provider_auth_file) {
        let _ = std::fs::remove_file(&temporary);
        return Err(error);
    }
    #[cfg(unix)]
    if let Some(parent) = provider_auth_file.parent() {
        std::fs::File::open(parent)?.sync_all()?;
    }
    Ok(())
}

fn save_provider_api_key(
    codex_home: &Path,
    provider_key_id: &str,
    api_key: &str,
) -> std::io::Result<()> {
    legacy_clear_provider_key_tombstone(codex_home, provider_key_id)?;
    super::provider_key_vault::write_provider_key(codex_home, provider_key_id, api_key)
}

/// Write a provider key to the legacy plaintext `provider_auth.json` store only.
pub(super) fn legacy_save_provider_key(
    codex_home: &Path,
    provider_key_id: &str,
    api_key: &str,
) -> std::io::Result<()> {
    let mut provider_auth = load_provider_auth(codex_home)?;
    provider_auth.deleted_api_keys.remove(provider_key_id);
    provider_auth
        .api_keys
        .insert(provider_key_id.to_string(), api_key.to_string());
    save_provider_auth(codex_home, &provider_auth)
}

/// Remove a single provider key from the legacy plaintext `provider_auth.json`.
///
/// Used after a vault write succeeds so re-saving a provider key migrates it off plaintext
/// storage. Removes the file entirely when it becomes empty. Returns `true` if a key was removed.
pub(super) fn legacy_delete_provider_key(
    codex_home: &Path,
    provider_key_id: &str,
) -> std::io::Result<bool> {
    let mut provider_auth = load_provider_auth(codex_home)?;
    let removed = provider_auth.api_keys.remove(provider_key_id).is_some();
    if removed {
        if provider_auth.api_keys.is_empty() && provider_auth.deleted_api_keys.is_empty() {
            delete_provider_auth_if_exists(codex_home)?;
        } else {
            save_provider_auth(codex_home, &provider_auth)?;
        }
    }
    Ok(removed)
}

fn legacy_provider_key_is_deleted(
    codex_home: &Path,
    provider_key_id: &str,
) -> std::io::Result<bool> {
    Ok(load_provider_auth(codex_home)?
        .deleted_api_keys
        .contains(provider_key_id))
}

fn legacy_mark_provider_key_deleted(
    codex_home: &Path,
    provider_key_id: &str,
) -> std::io::Result<bool> {
    let mut provider_auth = load_provider_auth(codex_home)?;
    provider_auth.api_keys.remove(provider_key_id);
    let inserted = provider_auth
        .deleted_api_keys
        .insert(provider_key_id.to_string());
    save_provider_auth(codex_home, &provider_auth)?;
    Ok(inserted)
}

fn legacy_clear_provider_key_tombstone(
    codex_home: &Path,
    provider_key_id: &str,
) -> std::io::Result<bool> {
    let mut provider_auth = load_provider_auth(codex_home)?;
    let removed = provider_auth.deleted_api_keys.remove(provider_key_id);
    if removed {
        if provider_auth.api_keys.is_empty() && provider_auth.deleted_api_keys.is_empty() {
            delete_provider_auth_if_exists(codex_home)?;
        } else {
            save_provider_auth(codex_home, &provider_auth)?;
        }
    }
    Ok(removed)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthConfig {
    pub codex_home: PathBuf,
    pub auth_credentials_store_mode: AuthCredentialsStoreMode,
    pub keyring_backend_kind: AuthKeyringBackendKind,
    pub forced_login_method: Option<ForcedLoginMethod>,
    pub chatgpt_base_url: Option<String>,
    pub forced_chatgpt_workspace_id: Option<Vec<String>>,
    pub managed_auth_policy: ManagedAuthPolicy,
    pub auth_route_config: AuthRouteConfig,
}

impl AuthConfig {
    pub fn is_login_method_allowed(&self, method: ForcedLoginMethod) -> bool {
        self.managed_auth_policy.allows_login_method(
            method,
            self.forced_login_method,
            self.forced_chatgpt_workspace_id.as_deref(),
        )
    }

    pub fn effective_chatgpt_workspaces(&self) -> Option<Vec<String>> {
        self.managed_auth_policy
            .effective_chatgpt_workspaces(self.forced_chatgpt_workspace_id.as_deref())
    }

    pub fn validate(&self) -> std::io::Result<()> {
        if self.is_login_method_allowed(ForcedLoginMethod::Api)
            || self.is_login_method_allowed(ForcedLoginMethod::Chatgpt)
        {
            Ok(())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "authentication requirements do not permit any usable login method",
            ))
        }
    }

    pub fn allows_auth(&self, auth: &CodexAuth) -> bool {
        let allowed_login_methods = self.allowed_login_methods();
        let workspaces = self.effective_chatgpt_workspaces();
        validate_auth_restrictions(Some(&allowed_login_methods), workspaces.as_deref(), auth)
            .is_ok()
    }

    pub async fn load_auth(
        &self,
        enable_codex_api_key_env: bool,
    ) -> std::io::Result<Option<CodexAuth>> {
        self.load_auth_with_modes(
            enable_codex_api_key_env,
            OpenAiApiKeyEnv::Ignore,
            /*allowed_auth_modes*/ None,
        )
        .await
    }

    async fn load_auth_with_modes(
        &self,
        enable_codex_api_key_env: bool,
        openai_api_key_env: OpenAiApiKeyEnv,
        allowed_auth_modes: Option<&[AuthMode]>,
    ) -> std::io::Result<Option<CodexAuth>> {
        let allowed_login_methods = self.allowed_login_methods();
        let workspaces = self.effective_chatgpt_workspaces();
        let agent_identity_authapi_base_url =
            agent_identity_authapi_base_url(self.chatgpt_base_url.as_deref()).ok();
        let auth = load_auth_with_openai_api_key_env(
            &self.codex_home,
            enable_codex_api_key_env,
            openai_api_key_env,
            self.auth_credentials_store_mode,
            Some(&allowed_login_methods),
            workspaces.as_deref(),
            self.chatgpt_base_url.as_deref(),
            self.keyring_backend_kind,
            agent_identity_authapi_base_url.as_deref(),
            &self.auth_route_config,
            /*cached_auth*/ None,
            allowed_auth_modes,
        )
        .await?;
        Ok(auth.filter(|auth| self.allows_auth(auth)))
    }

    fn allowed_login_methods(&self) -> Vec<ForcedLoginMethod> {
        self.managed_auth_policy.allowed_login_methods(
            self.forced_login_method,
            self.forced_chatgpt_workspace_id.as_deref(),
        )
    }
}

fn auth_mode_is_allowed(
    allowed_login_methods: Option<&[ForcedLoginMethod]>,
    mode: AuthMode,
) -> bool {
    let method = if mode.uses_codex_backend() {
        ForcedLoginMethod::Chatgpt
    } else {
        ForcedLoginMethod::Api
    };
    allowed_login_methods.is_none_or(|allowed| allowed.contains(&method))
}

fn validate_auth_restrictions(
    allowed_login_methods: Option<&[ForcedLoginMethod]>,
    expected_workspaces: Option<&[String]>,
    auth: &CodexAuth,
) -> Result<(), String> {
    if !auth_mode_is_allowed(allowed_login_methods, auth.auth_mode()) {
        return Err(match allowed_login_methods {
            Some(methods) if methods.contains(&ForcedLoginMethod::Api) => {
                "API key login is required".to_string()
            }
            Some(_) => "ChatGPT login is required".to_string(),
            None => unreachable!("unrestricted login methods accept every auth mode"),
        });
    }

    let Some(expected_workspaces) = expected_workspaces else {
        return Ok(());
    };
    if matches!(
        auth,
        CodexAuth::ApiKey(_) | CodexAuth::BedrockApiKey(_) | CodexAuth::BedrockAccessKeys(_)
    ) {
        return Ok(());
    }

    let actual_workspace = auth.get_account_id().or_else(|| {
        auth.get_token_data()
            .ok()
            .and_then(|tokens| tokens.id_token.chatgpt_account_id)
    });
    if actual_workspace
        .as_ref()
        .is_some_and(|workspace| expected_workspaces.contains(workspace))
    {
        Ok(())
    } else {
        let actual = actual_workspace.unwrap_or_else(|| "unknown".to_string());
        Err(format!(
            "Login is restricted to workspace(s) {}, but current credentials belong to {actual}",
            expected_workspaces.join(", ")
        ))
    }
}

/// Enforces configured login restrictions using auth-owned HTTP settings.
pub async fn enforce_login_restrictions(config: &AuthConfig) -> std::io::Result<()> {
    let agent_identity_authapi_base_url =
        agent_identity_authapi_base_url(config.chatgpt_base_url.as_deref()).ok();
    enforce_login_restrictions_with_agent_identity_authapi_base_url(
        config,
        agent_identity_authapi_base_url.as_deref(),
    )
    .await
}

async fn enforce_login_restrictions_with_agent_identity_authapi_base_url(
    config: &AuthConfig,
    agent_identity_authapi_base_url: Option<&str>,
) -> std::io::Result<()> {
    // Managed-only restrictions are enforced by AuthManager.
    if config.forced_login_method.is_none() && config.forced_chatgpt_workspace_id.is_none() {
        return Ok(());
    }

    let Some(auth) = load_auth_for_restrictions(config, agent_identity_authapi_base_url).await?
    else {
        return Ok(());
    };
    match login_restriction_violation(
        &auth,
        config.forced_login_method,
        config.forced_chatgpt_workspace_id.as_deref(),
    ) {
        None => Ok(()),
        Some(LoginRestrictionViolation::Method(reason)) => {
            let message = format!("{reason} Logging out.");
            if auth_uses_api_key_from_env(&auth) {
                return Err(std::io::Error::other(message));
            }
            logout_and_continue(
                &config.codex_home,
                message,
                config.auth_credentials_store_mode,
                config.keyring_backend_kind,
            )
        }
        Some(LoginRestrictionViolation::Workspace(reason)) => logout_with_message(
            &config.codex_home,
            format!("{reason} Logging out."),
            config.auth_credentials_store_mode,
            config.keyring_backend_kind,
        ),
    }
}

/// PF-84-S06 (#570): set when this process skipped the startup enforcement
/// of the login restrictions on the default login, because its session runs
/// (or a resumed or forked thread may run) on a named account.
static DEFAULT_LOGIN_LEFT_UNENFORCED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// PF-84-S06 (#570): records that this process left the default login in
/// place instead of enforcing the login restrictions on it at startup. Every
/// default-login manager [`AuthManager::shared_from_config`] builds afterwards
/// treats a login the restrictions rule out as absent, and a thread on the
/// default login is refused while it is ruled out. Call before any manager
/// is built.
pub fn leave_default_login_unenforced() {
    DEFAULT_LOGIN_LEFT_UNENFORCED.store(true, std::sync::atomic::Ordering::SeqCst);
}

/// PF-84-S06 (#595): whether this process left the default login in place
/// (see [`leave_default_login_unenforced`]). An app server reports it at
/// initialize, so a client reusing it knows a ruled-out default login is
/// neither logged out nor used there.
pub fn default_login_left_unenforced() -> bool {
    DEFAULT_LOGIN_LEFT_UNENFORCED.load(std::sync::atomic::Ordering::SeqCst)
}

fn default_login_unusable_message(reason: &str) -> String {
    format!(
        "The default OpenAI login cannot be used: {reason} It was left unchanged; sign in with \
         `corbanu login`, or choose a named account with `--account` or /providers."
    )
}

async fn load_auth_for_restrictions(
    config: &AuthConfig,
    agent_identity_authapi_base_url: Option<&str>,
) -> std::io::Result<Option<CodexAuth>> {
    load_auth_with_openai_api_key_env(
        &config.codex_home,
        /*enable_codex_api_key_env*/ true,
        OpenAiApiKeyEnv::for_forced_login_method(config.forced_login_method),
        config.auth_credentials_store_mode,
        /*allowed_login_methods*/ None,
        /*forced_chatgpt_workspace_id*/ None,
        config.chatgpt_base_url.as_deref(),
        config.keyring_backend_kind,
        agent_identity_authapi_base_url,
        &config.auth_route_config,
        /*cached_auth*/ None,
        /*allowed_auth_modes*/ None,
    )
    .await
}

/// How a login breaks the configured login restrictions. A wrong method is
/// logged out and the session continues; a wrong workspace ends it.
enum LoginRestrictionViolation {
    Method(String),
    Workspace(String),
}

impl LoginRestrictionViolation {
    fn reason(&self) -> &str {
        match self {
            Self::Method(reason) | Self::Workspace(reason) => reason,
        }
    }
}

fn login_restriction_violation(
    auth: &CodexAuth,
    forced_login_method: Option<ForcedLoginMethod>,
    forced_chatgpt_workspace_id: Option<&[String]>,
) -> Option<LoginRestrictionViolation> {
    if let Some(required_method) = forced_login_method {
        let method_violation = match (required_method, auth.auth_mode()) {
            (ForcedLoginMethod::Api, AuthMode::ApiKey)
            | (ForcedLoginMethod::Api, AuthMode::BedrockApiKey)
            | (ForcedLoginMethod::Api, AuthMode::BedrockAccessKeys) => None,
            (ForcedLoginMethod::Chatgpt, AuthMode::Chatgpt)
            | (ForcedLoginMethod::Chatgpt, AuthMode::ChatgptAuthTokens)
            | (ForcedLoginMethod::Chatgpt, AuthMode::Headers)
            | (ForcedLoginMethod::Chatgpt, AuthMode::AgentIdentity)
            | (ForcedLoginMethod::Chatgpt, AuthMode::PersonalAccessToken) => None,
            (ForcedLoginMethod::Api, AuthMode::Chatgpt)
            | (ForcedLoginMethod::Api, AuthMode::ChatgptAuthTokens)
            | (ForcedLoginMethod::Api, AuthMode::Headers)
            | (ForcedLoginMethod::Api, AuthMode::AgentIdentity)
            | (ForcedLoginMethod::Api, AuthMode::PersonalAccessToken) => {
                Some("API key login is required, but ChatGPT is currently being used.")
            }
            (ForcedLoginMethod::Chatgpt, AuthMode::ApiKey)
            | (ForcedLoginMethod::Chatgpt, AuthMode::BedrockApiKey)
            | (ForcedLoginMethod::Chatgpt, AuthMode::BedrockAccessKeys) => {
                Some("ChatGPT login is required, but an API key is currently being used.")
            }
        };
        if let Some(reason) = method_violation {
            return Some(LoginRestrictionViolation::Method(reason.to_string()));
        }
    }

    let expected_account_ids = forced_chatgpt_workspace_id?;
    let chatgpt_account_id = match auth {
        CodexAuth::ApiKey(_) | CodexAuth::BedrockApiKey(_) | CodexAuth::BedrockAccessKeys(_) => {
            return None;
        }
        CodexAuth::Headers(_) | CodexAuth::AgentIdentity(_) | CodexAuth::PersonalAccessToken(_) => {
            auth.get_account_id()
        }
        CodexAuth::Chatgpt(_) | CodexAuth::ChatgptAuthTokens(_) => match auth.get_token_data() {
            Ok(data) => data.id_token.chatgpt_account_id,
            Err(err) => {
                return Some(LoginRestrictionViolation::Workspace(format!(
                    "Failed to load ChatGPT credentials while enforcing workspace restrictions: {err}."
                )));
            }
        },
    };

    // workspace is the external identifier for account id.
    let chatgpt_account_id = chatgpt_account_id.as_deref();
    if chatgpt_account_id.is_some_and(|actual| {
        expected_account_ids
            .iter()
            .any(|expected| expected == actual)
    }) {
        return None;
    }
    let expected_workspaces = expected_account_ids.join(", ");
    Some(LoginRestrictionViolation::Workspace(
        match chatgpt_account_id {
            Some(actual) => format!(
                "Login is restricted to workspace(s) {expected_workspaces}, but current credentials belong to {actual}."
            ),
            None => format!(
                "Login is restricted to workspace(s) {expected_workspaces}, but current credentials lack a workspace identifier."
            ),
        },
    ))
}

fn logout_with_message(
    codex_home: &Path,
    message: String,
    auth_credentials_store_mode: AuthCredentialsStoreMode,
    keyring_backend_kind: AuthKeyringBackendKind,
) -> std::io::Result<()> {
    // External auth tokens live in the ephemeral store, but persistent auth may still exist
    // from earlier logins. Clear both so a forced logout truly removes all active account auth.
    let removal_result = logout_all_stores(
        codex_home,
        auth_credentials_store_mode,
        keyring_backend_kind,
        /*include_provider_credentials*/ false,
    );
    let error_message = match removal_result {
        Ok(_) => message,
        Err(err) => format!("{message}. Failed to remove stored credentials: {err}"),
    };
    Err(std::io::Error::other(error_message))
}

fn logout_and_continue(
    codex_home: &Path,
    message: String,
    auth_credentials_store_mode: AuthCredentialsStoreMode,
    keyring_backend_kind: AuthKeyringBackendKind,
) -> std::io::Result<()> {
    match logout_all_stores(
        codex_home,
        auth_credentials_store_mode,
        keyring_backend_kind,
        /*include_provider_credentials*/ false,
    ) {
        Ok(_) => {
            tracing::warn!("{message}");
            Ok(())
        }
        Err(err) => Err(std::io::Error::other(format!(
            "{message}. Failed to remove auth.json: {err}"
        ))),
    }
}

fn logout_all_stores(
    codex_home: &Path,
    auth_credentials_store_mode: AuthCredentialsStoreMode,
    keyring_backend_kind: AuthKeyringBackendKind,
    include_provider_credentials: bool,
) -> std::io::Result<bool> {
    if auth_credentials_store_mode == AuthCredentialsStoreMode::Ephemeral {
        let removed_ephemeral = logout(
            codex_home,
            AuthCredentialsStoreMode::Ephemeral,
            AuthKeyringBackendKind::default(),
        )?;
        let removed_provider = include_provider_credentials
            && logout_all_credentials(
                codex_home,
                AuthCredentialsStoreMode::File,
                keyring_backend_kind,
            )?;
        return Ok(removed_ephemeral || removed_provider);
    }
    let removed_ephemeral = logout(
        codex_home,
        AuthCredentialsStoreMode::Ephemeral,
        AuthKeyringBackendKind::default(),
    )?;
    let removed_managed = logout(
        codex_home,
        auth_credentials_store_mode,
        keyring_backend_kind,
    )?;
    let removed_provider = include_provider_credentials
        && logout_all_credentials(
            codex_home,
            auth_credentials_store_mode,
            keyring_backend_kind,
        )?;
    Ok(removed_ephemeral || removed_managed || removed_provider)
}

/// Loads auth without the `OPENAI_API_KEY` fallback.
#[allow(clippy::too_many_arguments)]
async fn load_auth(
    codex_home: &Path,
    enable_codex_api_key_env: bool,
    auth_credentials_store_mode: AuthCredentialsStoreMode,
    allowed_login_methods: Option<&[ForcedLoginMethod]>,
    forced_chatgpt_workspace_id: Option<&[String]>,
    chatgpt_base_url: Option<&str>,
    keyring_backend_kind: AuthKeyringBackendKind,
    agent_identity_authapi_base_url: Option<&str>,
    auth_route_config: &AuthRouteConfig,
    cached_auth: Option<&CodexAuth>,
    allowed_auth_modes: Option<&[AuthMode]>,
) -> std::io::Result<Option<CodexAuth>> {
    load_auth_with_openai_api_key_env(
        codex_home,
        enable_codex_api_key_env,
        OpenAiApiKeyEnv::Ignore,
        auth_credentials_store_mode,
        allowed_login_methods,
        forced_chatgpt_workspace_id,
        chatgpt_base_url,
        keyring_backend_kind,
        agent_identity_authapi_base_url,
        auth_route_config,
        cached_auth,
        allowed_auth_modes,
    )
    .await
}

#[allow(clippy::too_many_arguments)]
async fn load_auth_with_openai_api_key_env(
    codex_home: &Path,
    enable_codex_api_key_env: bool,
    openai_api_key_env: OpenAiApiKeyEnv,
    auth_credentials_store_mode: AuthCredentialsStoreMode,
    allowed_login_methods: Option<&[ForcedLoginMethod]>,
    forced_chatgpt_workspace_id: Option<&[String]>,
    chatgpt_base_url: Option<&str>,
    keyring_backend_kind: AuthKeyringBackendKind,
    agent_identity_authapi_base_url: Option<&str>,
    auth_route_config: &AuthRouteConfig,
    cached_auth: Option<&CodexAuth>,
    allowed_auth_modes: Option<&[AuthMode]>,
) -> std::io::Result<Option<CodexAuth>> {
    let mode_allowed = |mode| allowed_auth_modes.is_none_or(|modes| modes.contains(&mode));
    let api_key_allowed = mode_allowed(AuthMode::ApiKey)
        && auth_mode_is_allowed(allowed_login_methods, AuthMode::ApiKey);
    // Precedence (docs/authentication.md, "OpenAI API keys"): CODEX_API_KEY
    // (exec only), then external or stored OpenAI credentials, then
    // OPENAI_API_KEY. A shell-exported OPENAI_API_KEY therefore never replaces
    // a ChatGPT sign-in or a saved key, so it can't switch the billing basis.
    if enable_codex_api_key_env
        && api_key_allowed
        && let Some(api_key) = read_codex_api_key_from_env()
    {
        return Ok(Some(CodexAuth::from_api_key_env(
            api_key.as_str(),
            CODEX_API_KEY_ENV_VAR,
        )));
    }
    if enable_codex_api_key_env
        && api_key_allowed
        && let Some(api_key) = read_ambient_api_key_from_env()
    {
        return Ok(Some(CodexAuth::from_api_key_env(
            api_key.as_str(),
            AMBIENT_API_KEY_ENV_VAR,
        )));
    }
    let openai_api_key_env_auth = || match openai_api_key_env {
        OpenAiApiKeyEnv::Fallback if !api_key_allowed => None,
        OpenAiApiKeyEnv::Fallback => read_openai_api_key_from_env()
            .map(|api_key| CodexAuth::from_api_key_env(api_key.as_str(), OPENAI_API_KEY_ENV_VAR)),
        OpenAiApiKeyEnv::Ignore => None,
    };

    // External ChatGPT auth tokens live in the in-memory (ephemeral) store. Always check this
    // first so external auth takes precedence over any persisted credentials.
    let ephemeral_storage = create_auth_storage(
        codex_home.to_path_buf(),
        AuthCredentialsStoreMode::Ephemeral,
        AuthKeyringBackendKind::default(),
    );
    if let Some(auth_dot_json) = load_stored_auth(ephemeral_storage).await?
        && auth_dot_json.has_primary_auth_material()
        && auth_mode_is_allowed(allowed_login_methods, auth_dot_json.resolved_mode())
    {
        if !mode_allowed(auth_dot_json.resolved_mode()) {
            return Ok(None);
        }
        if let Some(agent_identity) = auth_dot_json.agent_identity.as_ref() {
            ensure_agent_identity_workspace_allowed(forced_chatgpt_workspace_id, agent_identity)?;
        }
        let auth = CodexAuth::from_auth_dot_json(
            codex_home,
            auth_dot_json,
            AuthCredentialsStoreMode::Ephemeral,
            chatgpt_base_url,
            keyring_backend_kind,
            agent_identity_authapi_base_url,
            auth_route_config,
            cached_auth,
        )
        .await?;
        if let CodexAuth::PersonalAccessToken(auth) = &auth {
            ensure_auth_workspace_allowed(forced_chatgpt_workspace_id, auth.account_id())?;
        }
        return Ok(Some(auth));
    }

    if auth_mode_is_allowed(allowed_login_methods, AuthMode::AgentIdentity)
        && let Some(access_token) = read_codex_access_token_from_env()
    {
        return match classify_codex_access_token(&access_token) {
            CodexAccessToken::PersonalAccessToken(access_token) => {
                if !mode_allowed(AuthMode::PersonalAccessToken) {
                    return Ok(None);
                }
                let auth = PersonalAccessTokenAuth::load_with_cached(
                    access_token,
                    auth_route_config,
                    cached_auth,
                )
                .await?;
                ensure_auth_workspace_allowed(forced_chatgpt_workspace_id, auth.account_id())?;
                Ok(Some(CodexAuth::PersonalAccessToken(auth)))
            }
            CodexAccessToken::AgentIdentityJwt(jwt) => {
                if !mode_allowed(AuthMode::AgentIdentity) {
                    return Ok(None);
                }
                let record = AgentIdentityAuthRecord::from_agent_identity_jwt(jwt)?;
                ensure_auth_workspace_allowed(forced_chatgpt_workspace_id, &record.account_id)?;
                CodexAuth::from_agent_identity_jwt_with_authapi_base_url(
                    jwt,
                    chatgpt_base_url,
                    require_agent_identity_authapi_base_url(agent_identity_authapi_base_url)?,
                    auth_route_config,
                )
            }
            .await
            .map(Some),
        };
    }

    // If the caller explicitly requested ephemeral auth, there is no persisted fallback.
    if auth_credentials_store_mode == AuthCredentialsStoreMode::Ephemeral {
        return Ok(openai_api_key_env_auth());
    }

    // Fall back to the configured persistent store (file/keyring/auto) for managed auth.
    let storage = create_auth_storage(
        codex_home.to_path_buf(),
        auth_credentials_store_mode,
        keyring_backend_kind,
    );
    let auth_dot_json = match load_stored_auth(storage).await? {
        Some(auth) if auth.has_primary_auth_material() => auth,
        Some(_) | None => match load_pfterminal_legacy_codex_auth(
            codex_home,
            auth_credentials_store_mode,
            keyring_backend_kind,
        )? {
            Some(auth) => auth,
            None => return Ok(openai_api_key_env_auth()),
        },
    };
    if !mode_allowed(auth_dot_json.resolved_mode())
        || !auth_mode_is_allowed(allowed_login_methods, auth_dot_json.resolved_mode())
    {
        return Ok(None);
    }
    if let Some(agent_identity) = auth_dot_json.agent_identity.as_ref() {
        ensure_agent_identity_workspace_allowed(forced_chatgpt_workspace_id, agent_identity)?;
    }

    let auth = CodexAuth::from_auth_dot_json(
        codex_home,
        auth_dot_json,
        auth_credentials_store_mode,
        chatgpt_base_url,
        keyring_backend_kind,
        agent_identity_authapi_base_url,
        auth_route_config,
        cached_auth,
    )
    .await?;
    if let CodexAuth::PersonalAccessToken(auth) = &auth {
        ensure_auth_workspace_allowed(forced_chatgpt_workspace_id, auth.account_id())?;
    }
    Ok(Some(auth))
}

fn load_pfterminal_legacy_codex_auth(
    codex_home: &Path,
    auth_credentials_store_mode: AuthCredentialsStoreMode,
    keyring_backend_kind: AuthKeyringBackendKind,
) -> std::io::Result<Option<AuthDotJson>> {
    let Some(legacy_codex_home) = legacy_codex_home_for_product_home(codex_home) else {
        return Ok(None);
    };
    let storage = create_auth_storage(
        legacy_codex_home.clone(),
        auth_credentials_store_mode,
        keyring_backend_kind,
    );
    let auth = storage
        .load()?
        .filter(AuthDotJson::has_primary_auth_material);
    if auth.is_some() {
        tracing::info!(
            product_home = %codex_home.display(),
            legacy_codex_home = %legacy_codex_home.display(),
            "loaded legacy Codex auth for Corbanu Terminal"
        );
    }
    Ok(auth)
}

pub fn legacy_codex_home_for_product_home(codex_home: &Path) -> Option<PathBuf> {
    let home_name = codex_home.file_name()?.to_str()?;
    if home_name != ".corbanu" && home_name != ".pfterminal" {
        return None;
    }
    Some(codex_home.parent()?.join(".codex"))
}

// Blocking keyring access must not prevent an async caller's deadline from firing.
async fn load_stored_auth(
    storage: Arc<dyn AuthStorageBackend>,
) -> std::io::Result<Option<AuthDotJson>> {
    let originator = codex_otel::auth_storage::AuthStorageOriginator::current();
    tokio::task::spawn_blocking(move || originator.sync_scope(|| storage.load()))
        .await
        .map_err(|_| std::io::Error::other("credential storage read failed"))?
}

// Ordinary auth tolerates legacy metadata loss; EMA grant checks remain strict.
fn same_login_for_persistence(expected: &TokenData, stored: &TokenData) -> bool {
    same_account_login(expected, stored)
        || (expected.login_owner_id.is_some()
            && stored.login_owner_id.is_none()
            && stored.account_id == expected.account_id
            && stored.id_token.chatgpt_user_id == expected.id_token.chatgpt_user_id
            && !stored.refresh_token.is_empty()
            && stored.refresh_token == expected.refresh_token)
}

// Persist refreshed tokens into auth storage and update last_refresh.
// D171 (#1311): read and saved in one step, and only onto the stored sign-in
// that was refreshed (`expected`), so a removal or a new sign-in by another
// process meanwhile is kept.
pub(super) fn persist_tokens(
    storage: &Arc<dyn AuthStorageBackend>,
    expected: &TokenData,
    provenance: Option<AccountOAuthProvenance>,
    id_token: Option<String>,
    access_token: Option<String>,
    refresh_token: Option<String>,
) -> std::io::Result<AuthDotJson> {
    let id_token = id_token
        .map(|id_token| parse_chatgpt_jwt_claims(&id_token).map_err(std::io::Error::other))
        .transpose()?;
    let _commit = storage.lock_for_write()?;
    let changed = || std::io::Error::other("Account credentials changed during refresh.");
    let apply = |auth_dot_json: &mut AuthDotJson| -> std::io::Result<()> {
        if auth_dot_json.resolved_mode() != AuthMode::Chatgpt
            || !auth_dot_json.tokens.as_ref().is_some_and(|tokens| {
                same_login_for_persistence(expected, tokens)
                    && tokens.refresh_token == expected.refresh_token
                    && tokens.access_token == expected.access_token
            })
        {
            return Err(changed());
        }
        let tokens = auth_dot_json
            .tokens
            .as_mut()
            .ok_or(std::io::Error::other("Token data is not available."))?;
        tokens.oauth_provenance = provenance
            .clone()
            .filter(|_| refresh_token.is_some() || tokens.refresh_token == expected.refresh_token);
        if let Some(id_token) = id_token.clone() {
            tokens.id_token = id_token;
        }
        if let Some(access_token) = access_token.clone() {
            tokens.access_token = access_token;
        }
        if let Some(refresh_token) = refresh_token.clone() {
            tokens.refresh_token = refresh_token;
        }
        auth_dot_json.last_refresh = Some(Utc::now());
        Ok(())
    };
    if storage.is_named_account() {
        // A named account's vault store reads and saves under one lock.
        let mut apply = apply;
        return storage
            .update(&mut apply)?
            .ok_or_else(|| std::io::Error::other("Token data is not available."));
    }
    let mut auth_dot_json = storage
        .load()?
        .ok_or(std::io::Error::other("Token data is not available."))?;
    let original = auth_dot_json.clone();
    apply(&mut auth_dot_json)?;
    storage.compare_and_save(&original, &auth_dot_json)?;
    Ok(auth_dot_json)
}

// Requests refreshed ChatGPT OAuth tokens from the auth service using a refresh token.
// The caller is responsible for persisting any returned tokens.
async fn request_chatgpt_token_refresh(
    refresh_token: String,
    endpoint: &str,
    client_id: &str,
    client: &HttpClient,
    refresh_reason: RefreshReason,
) -> Result<RefreshResponse, RefreshTokenError> {
    let oauth = OAuthClient::new(
        client,
        TokenEndpoint {
            url: endpoint,
            client_id,
            encoding: TokenEncoding::Json,
            timeout: Some(Duration::from_secs(45)),
            error_body_limit: ErrorBodyLimit::Unlimited,
        },
    );
    match oauth
        .refresh(RefreshTokenGrant {
            refresh_token: &refresh_token,
            resource: None,
            refresh_reason: Some(refresh_reason),
        })
        .await
    {
        Ok(response) => Ok(response),
        Err(OAuthError::Rejected(rejection)) => {
            if let Some(codex_http_client::HttpError::Policy(denied)) = rejection.body_read_error {
                return Err(denied.into());
            }
            let status = rejection.status;
            // The IdP controls the response body and may echo credential material.
            tracing::error!(%status, "Failed to refresh token");
            let code = rejection.detail.error_code();
            let is_invalid_grant_bad_request = status == StatusCode::BAD_REQUEST
                && code.is_some_and(|code| code.eq_ignore_ascii_case("invalid_grant"));
            let failed = classify_refresh_token_failure(code, is_invalid_grant_bad_request);
            if status == StatusCode::UNAUTHORIZED
                || failed.reason != RefreshTokenFailedReason::Other
                || is_invalid_grant_bad_request
            {
                Err(RefreshTokenError::Permanent(failed))
            } else {
                Err(RefreshTokenError::Transient(std::io::Error::other(
                    format!("Failed to refresh token: {status}"),
                )))
            }
        }
        Err(OAuthError::Transport(error)) => Err(error.into()),
        Err(error @ OAuthError::InvalidResponse) => {
            Err(RefreshTokenError::Transient(std::io::Error::other(error)))
        }
    }
}

fn classify_refresh_token_failure(
    code: Option<&str>,
    is_invalid_grant_bad_request: bool,
) -> RefreshTokenFailedError {
    let normalized_code = code.map(str::to_ascii_lowercase);
    let reason = match normalized_code.as_deref() {
        Some("refresh_token_expired") => RefreshTokenFailedReason::Expired,
        Some("refresh_token_reused") => RefreshTokenFailedReason::Exhausted,
        Some("refresh_token_invalidated") => RefreshTokenFailedReason::Revoked,
        _ => RefreshTokenFailedReason::Other,
    };

    if reason == RefreshTokenFailedReason::Other && !is_invalid_grant_bad_request {
        tracing::warn!("Encountered unknown response while refreshing token");
    }

    let message = match reason {
        RefreshTokenFailedReason::Expired => REFRESH_TOKEN_EXPIRED_MESSAGE.to_string(),
        RefreshTokenFailedReason::Exhausted => REFRESH_TOKEN_REUSED_MESSAGE.to_string(),
        RefreshTokenFailedReason::Revoked => REFRESH_TOKEN_INVALIDATED_MESSAGE.to_string(),
        RefreshTokenFailedReason::Other => REFRESH_TOKEN_UNKNOWN_MESSAGE.to_string(),
    };

    RefreshTokenFailedError::new(reason, message)
}

#[derive(Deserialize, Clone)]
struct RefreshResponse {
    id_token: Option<String>,
    access_token: Option<String>,
    refresh_token: Option<String>,
}

// Shared constant for token refresh (client id used for oauth token refresh flow)
pub const CLIENT_ID: &str = "app_EMoamEEZ73f0CkXaXp7hrann";

pub fn oauth_client_id() -> String {
    std::env::var(CLIENT_ID_OVERRIDE_ENV_VAR)
        .ok()
        .filter(|client_id| !client_id.trim().is_empty())
        .unwrap_or_else(|| CLIENT_ID.to_string())
}

fn refresh_token_endpoint(provenance: Option<&AccountOAuthProvenance>) -> String {
    std::env::var(REFRESH_TOKEN_URL_OVERRIDE_ENV_VAR).unwrap_or_else(|_| {
        provenance
            .map(|provenance| format!("{}/oauth/token", provenance.issuer.trim_end_matches('/')))
            .unwrap_or_else(|| REFRESH_TOKEN_URL.to_string())
    })
}

impl AuthDotJson {
    fn has_primary_auth_material(&self) -> bool {
        self.auth_mode.is_some()
            || self.openai_api_key.is_some()
            || self.tokens.is_some()
            || self
                .agent_identity
                .as_ref()
                .is_some_and(AgentIdentityStorage::has_auth_material)
            || self.personal_access_token.is_some()
            || self.bedrock_api_key.is_some()
            || self.bedrock_access_keys.is_some()
    }

    fn from_external_access_token(
        access_token: &str,
        chatgpt_account_id: &str,
        chatgpt_plan_type: Option<&str>,
    ) -> std::io::Result<Self> {
        let mut token_info =
            parse_chatgpt_jwt_claims(access_token).map_err(std::io::Error::other)?;
        token_info.chatgpt_account_id = Some(chatgpt_account_id.to_string());
        token_info.chatgpt_plan_type = chatgpt_plan_type
            .map(InternalPlanType::from_raw_value)
            .or(token_info.chatgpt_plan_type)
            .or(Some(InternalPlanType::Unknown("unknown".to_string())));
        let tokens = TokenData {
            id_token: token_info,
            access_token: access_token.to_string(),
            refresh_token: String::new(),
            oauth_provenance: None,
            login_owner_id: None,
            account_id: Some(chatgpt_account_id.to_string()),
        };

        Ok(Self {
            auth_mode: Some(AuthMode::ChatgptAuthTokens),
            openai_api_key: None,
            tokens: Some(tokens),
            last_refresh: Some(Utc::now()),
            agent_identity: None,
            personal_access_token: None,
            bedrock_api_key: None,
            bedrock_access_keys: None,
        })
    }

    pub(super) fn resolved_mode(&self) -> AuthMode {
        if let Some(mode) = self.auth_mode {
            return mode;
        }
        if self.personal_access_token.is_some() {
            return AuthMode::PersonalAccessToken;
        }
        if self.bedrock_api_key.is_some() {
            return AuthMode::BedrockApiKey;
        }
        if self.bedrock_access_keys.is_some() {
            return AuthMode::BedrockAccessKeys;
        }
        if self.openai_api_key.is_some() {
            return AuthMode::ApiKey;
        }
        AuthMode::Chatgpt
    }

    fn storage_mode(
        &self,
        auth_credentials_store_mode: AuthCredentialsStoreMode,
    ) -> AuthCredentialsStoreMode {
        if self.resolved_mode() == AuthMode::ChatgptAuthTokens {
            AuthCredentialsStoreMode::Ephemeral
        } else {
            auth_credentials_store_mode
        }
    }
}

struct AuthReload {
    changed: bool,
    changed_for_refresh: bool,
}

/// Internal cached auth state.
#[derive(Clone)]
struct CachedAuth {
    auth: Option<CodexAuth>,
    /// Permanent refresh failure cached for the current auth snapshot so
    /// later refresh attempts for the same credentials fail fast without network.
    permanent_refresh_failure: Option<AuthScopedRefreshFailure>,
}

#[derive(Clone)]
struct AuthScopedRefreshFailure {
    auth: CodexAuth,
    error: RefreshTokenFailedError,
}

impl Debug for CachedAuth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CachedAuth")
            .field(
                "auth_mode",
                &self.auth.as_ref().map(CodexAuth::api_auth_mode),
            )
            .field(
                "permanent_refresh_failure",
                &self
                    .permanent_refresh_failure
                    .as_ref()
                    .map(|failure| failure.error.reason),
            )
            .finish()
    }
}

enum UnauthorizedRecoveryStep {
    Reload,
    RefreshToken,
    ExternalRefresh,
    Done,
}

enum ReloadOutcome {
    /// Reload was performed and the cached auth changed
    ReloadedChanged,
    /// Reload was performed and the cached auth remained the same
    ReloadedNoChange,
    /// Reload was skipped (missing or mismatched account id)
    Skipped,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum UnauthorizedRecoveryMode {
    Managed,
    External,
}

// UnauthorizedRecovery is a state machine that handles an attempt to refresh the authentication when requests
// to API fail with 401 status code.
// The client calls next() every time it encounters a 401 error, one time per retry.
// For API key based authentication, we don't do anything and let the error bubble to the user.
//
// For ChatGPT based authentication, we:
// 1. Attempt to reload the auth data from disk. We only reload if the account id matches the one the current process is running as.
// 2. Attempt to refresh the token using OAuth token refresh flow.
// If after both steps the server still responds with 401 we let the error bubble to the user.
//
// For external auth sources, UnauthorizedRecovery retries once by asking the
// configured provider to refresh and caching the returned auth through the same
// path used by other auth sources.
pub struct UnauthorizedRecovery {
    manager: Arc<AuthManager>,
    step: UnauthorizedRecoveryStep,
    expected_account_id: Option<String>,
    mode: UnauthorizedRecoveryMode,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnauthorizedRecoveryStepResult {
    auth_state_changed: Option<bool>,
}

impl UnauthorizedRecoveryStepResult {
    pub fn auth_state_changed(&self) -> Option<bool> {
        self.auth_state_changed
    }
}

impl UnauthorizedRecovery {
    fn new(manager: Arc<AuthManager>) -> Self {
        let cached_auth = manager.auth_cached();
        let expected_account_id = cached_auth.as_ref().and_then(CodexAuth::get_account_id);
        let mode = if manager.has_external_auth() {
            UnauthorizedRecoveryMode::External
        } else {
            UnauthorizedRecoveryMode::Managed
        };
        let step = match mode {
            UnauthorizedRecoveryMode::Managed => UnauthorizedRecoveryStep::Reload,
            UnauthorizedRecoveryMode::External => UnauthorizedRecoveryStep::ExternalRefresh,
        };
        Self {
            manager,
            step,
            expected_account_id,
            mode,
        }
    }

    pub fn has_next(&self) -> bool {
        if self.manager.has_refreshable_external_auth() {
            return !matches!(self.step, UnauthorizedRecoveryStep::Done);
        }

        if !self
            .manager
            .auth_cached()
            .as_ref()
            .is_some_and(CodexAuth::supports_unauthorized_recovery)
        {
            return false;
        }

        if self.mode == UnauthorizedRecoveryMode::External && !self.manager.has_external_auth() {
            return false;
        }

        !matches!(self.step, UnauthorizedRecoveryStep::Done)
    }

    pub fn unavailable_reason(&self) -> &'static str {
        if self.manager.has_refreshable_external_auth() {
            return if matches!(self.step, UnauthorizedRecoveryStep::Done) {
                "recovery_exhausted"
            } else {
                "ready"
            };
        }

        if self
            .manager
            .auth_cached()
            .as_ref()
            .is_some_and(CodexAuth::is_personal_access_token_auth)
        {
            return "not_refreshable_auth";
        }

        if !self
            .manager
            .auth_cached()
            .as_ref()
            .is_some_and(CodexAuth::supports_unauthorized_recovery)
        {
            return "not_chatgpt_auth";
        }

        if self.mode == UnauthorizedRecoveryMode::External && !self.manager.has_external_auth() {
            return "no_external_auth";
        }

        if matches!(self.step, UnauthorizedRecoveryStep::Done) {
            return "recovery_exhausted";
        }

        "ready"
    }

    pub fn mode_name(&self) -> &'static str {
        match self.mode {
            UnauthorizedRecoveryMode::Managed => "managed",
            UnauthorizedRecoveryMode::External => "external",
        }
    }

    pub fn step_name(&self) -> &'static str {
        match self.step {
            UnauthorizedRecoveryStep::Reload => "reload",
            UnauthorizedRecoveryStep::RefreshToken => "refresh_token",
            UnauthorizedRecoveryStep::ExternalRefresh => "external_refresh",
            UnauthorizedRecoveryStep::Done => "done",
        }
    }

    pub async fn next(&mut self) -> Result<UnauthorizedRecoveryStepResult, RefreshTokenError> {
        if !self.has_next() {
            return Err(RefreshTokenError::Permanent(RefreshTokenFailedError::new(
                RefreshTokenFailedReason::Other,
                "No more recovery steps available.",
            )));
        }

        match self.step {
            UnauthorizedRecoveryStep::Reload => {
                match self
                    .manager
                    .reload_if_account_id_matches(self.expected_account_id.as_deref())
                    .await?
                {
                    ReloadOutcome::ReloadedChanged => {
                        self.step = UnauthorizedRecoveryStep::RefreshToken;
                        return Ok(UnauthorizedRecoveryStepResult {
                            auth_state_changed: Some(true),
                        });
                    }
                    ReloadOutcome::ReloadedNoChange => {
                        self.step = UnauthorizedRecoveryStep::RefreshToken;
                        return Ok(UnauthorizedRecoveryStepResult {
                            auth_state_changed: Some(false),
                        });
                    }
                    ReloadOutcome::Skipped => {
                        self.step = UnauthorizedRecoveryStep::Done;
                        return Err(RefreshTokenError::Permanent(RefreshTokenFailedError::new(
                            RefreshTokenFailedReason::Other,
                            REFRESH_TOKEN_ACCOUNT_MISMATCH_MESSAGE.to_string(),
                        )));
                    }
                }
            }
            UnauthorizedRecoveryStep::RefreshToken => {
                self.manager
                    .refresh_token_from_authority_with_reason(RefreshReason::UnauthorizedRecovery)
                    .await?;
                self.step = UnauthorizedRecoveryStep::Done;
                return Ok(UnauthorizedRecoveryStepResult {
                    auth_state_changed: Some(true),
                });
            }
            UnauthorizedRecoveryStep::ExternalRefresh => {
                self.manager
                    .refresh_token_from_authority_with_reason(RefreshReason::UnauthorizedRecovery)
                    .await?;
                self.step = UnauthorizedRecoveryStep::Done;
                return Ok(UnauthorizedRecoveryStepResult {
                    auth_state_changed: Some(true),
                });
            }
            UnauthorizedRecoveryStep::Done => {}
        }
        Ok(UnauthorizedRecoveryStepResult {
            auth_state_changed: None,
        })
    }
}

/// Central manager providing a single source of truth for stored
/// authentication data. It loads once (or on preference change) and then
/// hands out cloned `CodexAuth` values so the rest of the program has a
/// consistent snapshot.
///
/// External modifications to stored credentials will NOT be observed until
/// `reload()` is called explicitly. This matches the design goal of avoiding
/// different parts of the program seeing inconsistent auth data mid‑run.
pub struct AuthManager {
    codex_home: PathBuf,
    inner: RwLock<CachedAuth>,
    auth_change_tx: watch::Sender<u64>,
    auth_change_state_tx: watch::Sender<AuthChangeState>,
    workspace_routing_resolver: OnceLock<Weak<dyn WorkspaceRoutingResolver>>,
    enable_codex_api_key_env: bool,
    openai_api_key_env: OpenAiApiKeyEnv,
    auth_credentials_store_mode: AuthCredentialsStoreMode,
    keyring_backend_kind: AuthKeyringBackendKind,
    forced_login_method: Option<ForcedLoginMethod>,
    forced_chatgpt_workspace_id: RwLock<Option<Vec<String>>>,
    managed_auth_policy: ManagedAuthPolicy,
    chatgpt_base_url: Option<String>,
    agent_identity_authapi_base_url: Option<String>,
    refresh_lock: Arc<Semaphore>,
    // Writers hold refresh_lock; cancellation retains the attempted generation.
    unsettled_chatgpt_refresh: Mutex<Option<TokenData>>,
    agent_identity_lock: Semaphore,
    agent_identity_bootstrap_cooldown: Mutex<AgentIdentityBootstrapCooldown>,
    external_auth: RwLock<Option<Arc<dyn ExternalAuth>>>,
    provider_api_key_cache: RwLock<ProviderApiKeyCache>,
    workload_identity_selected: bool,
    auth_route_config: AuthRouteConfig,
    /// PF-84-S06: the named sign-in account this manager serves. It loads,
    /// refreshes and logs out only that account's vault store, never the
    /// default login, the environment or the ephemeral store.
    named_account: Option<NamedChatgptAccount>,
    /// D171 (#1286): for a named-account manager, its vault handle (kept so
    /// the decrypted store stays cached) and the store marker last seen
    /// holding the account.
    named_store: Option<named_account::NamedAccountStore>,
    /// For a named-account manager, the manager of the default login it was
    /// created from (providers without a named account go back to it).
    base_manager: Option<Arc<AuthManager>>,
    /// PF-84-S06 (#570): the session runs on a named sign-in account, so the
    /// default login was not checked (and logged out) at startup; one the
    /// login restrictions rule out is treated as absent instead.
    hide_restricted_default_login: bool,
    /// Why the default login is hidden right now, if it is.
    hidden_default_login_reason: RwLock<Option<String>>,
    allowed_auth_modes: RwLock<Option<Vec<AuthMode>>>,
}

#[derive(Debug)]
struct ProviderApiKeyCache {
    storage_revision: u64,
    entries: HashMap<String, Option<String>>,
}

impl ProviderApiKeyCache {
    fn new() -> Self {
        Self {
            storage_revision: provider_api_key_storage_revision(),
            entries: HashMap::new(),
        }
    }

    fn clear_to_current_revision(&mut self) {
        self.storage_revision = provider_api_key_storage_revision();
        self.entries.clear();
    }
}

/// Configuration view required to construct a shared [`AuthManager`].
///
/// Implementations should return the auth-related config values for the
/// already-resolved runtime configuration. The primary implementation is
/// `codex_core::config::Config`, but this trait keeps `codex-login` independent
/// from `codex-core`.
pub trait AuthManagerConfig {
    /// Returns the Codex home directory used for auth storage.
    fn codex_home(&self) -> PathBuf;

    /// Returns the CLI auth credential storage mode for auth loading.
    fn cli_auth_credentials_store_mode(&self) -> AuthCredentialsStoreMode;

    /// Returns the backend to use when CLI auth keyring storage is selected.
    fn auth_keyring_backend_kind(&self) -> AuthKeyringBackendKind;

    /// Returns the resolved login-method restriction, if any. A required
    /// ChatGPT login stops `OPENAI_API_KEY` from being used as a fallback.
    fn forced_login_method(&self) -> Option<ForcedLoginMethod>;

    /// Returns the workspace IDs that ChatGPT auth should be restricted to, if any.
    fn forced_chatgpt_workspace_id(&self) -> Option<Vec<String>>;

    /// Returns administrator-managed authentication restrictions.
    fn managed_auth_policy(&self) -> ManagedAuthPolicy;

    /// Returns the ChatGPT backend base URL used for first-party backend authorization.
    fn chatgpt_base_url(&self) -> String;

    /// Returns route-selection settings for auth-owned clients.
    fn auth_route_config(&self) -> AuthRouteConfig;

    /// The login method the configuration itself requires (`forced_login_method`
    /// as configured). Unlike [`AuthManagerConfig::forced_login_method`] it
    /// omits the API-key-only default a third-party session provider implies
    /// for onboarding, which must not refuse a named OpenAI account (#571).
    fn configured_forced_login_method(&self) -> Option<ForcedLoginMethod> {
        self.forced_login_method()
    }

    /// PF-84-S06 (#570): whether the session runs on a named OpenAI sign-in
    /// account. Its startup then leaves the default login alone, so the
    /// default login's manager must not use one the restrictions rule out.
    fn runs_on_named_sign_in_account(&self) -> bool {
        false
    }
}

/// Runtime storage and network policy shared by independent credential managers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuthRuntimeConfig {
    pub codex_home: PathBuf,
    pub auth_route_config: AuthRouteConfig,
}

impl Debug for AuthManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AuthManager")
            .field("codex_home", &self.codex_home)
            .field("inner", &self.inner)
            .field("enable_codex_api_key_env", &self.enable_codex_api_key_env)
            .field(
                "auth_credentials_store_mode",
                &self.auth_credentials_store_mode,
            )
            .field("keyring_backend_kind", &self.keyring_backend_kind)
            .field("forced_login_method", &self.forced_login_method)
            .field(
                "forced_chatgpt_workspace_id",
                &self.forced_chatgpt_workspace_id,
            )
            .field("managed_auth_policy", &self.managed_auth_policy)
            .field("chatgpt_base_url", &self.chatgpt_base_url)
            .field("auth_route_config", &self.auth_route_config)
            .field("has_external_auth", &self.has_external_auth())
            .field(
                "workload_identity_selected",
                &self.workload_identity_selected,
            )
            .finish_non_exhaustive()
    }
}

fn default_agent_identity_authapi_base_url() -> Option<String> {
    agent_identity_authapi_base_url(/*chatgpt_base_url*/ None).ok()
}

impl AuthManager {
    /// Auth storage root used by provider-scoped credential caches.
    pub fn codex_home(&self) -> &Path {
        &self.codex_home
    }

    /// Returns the application policy associated with this account owner.
    pub fn application_network_policy(&self) -> codex_http_client::NetworkPolicy {
        self.auth_route_config.application_network_policy().clone()
    }

    /// Creates content clients bound to the account currently owned by this manager.
    pub fn http_client_factory(&self) -> HttpClientFactory {
        self.auth_route_config
            .http_client_factory()
            .clone()
            .with_network_policy(self.application_network_policy().for_current_account())
    }
    /// Create a new manager loading the initial auth using the provided
    /// preferred auth method. Errors loading auth are swallowed; `auth()` will
    /// simply return `None` in that case so callers can treat it as an
    /// unauthenticated state. `OPENAI_API_KEY` is not read; model sessions use
    /// [`AuthManager::shared_from_config`], which adds it as a fallback.
    pub async fn new(
        codex_home: PathBuf,
        enable_codex_api_key_env: bool,
        auth_credentials_store_mode: AuthCredentialsStoreMode,
        forced_chatgpt_workspace_id: Option<Vec<String>>,
        chatgpt_base_url: Option<String>,
        keyring_backend_kind: AuthKeyringBackendKind,
        auth_route_config: AuthRouteConfig,
    ) -> Self {
        Self::new_with_openai_api_key_env(
            codex_home,
            enable_codex_api_key_env,
            OpenAiApiKeyEnv::Ignore,
            auth_credentials_store_mode,
            forced_chatgpt_workspace_id,
            chatgpt_base_url,
            keyring_backend_kind,
            auth_route_config,
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    async fn new_with_openai_api_key_env(
        codex_home: PathBuf,
        enable_codex_api_key_env: bool,
        openai_api_key_env: OpenAiApiKeyEnv,
        auth_credentials_store_mode: AuthCredentialsStoreMode,
        forced_chatgpt_workspace_id: Option<Vec<String>>,
        chatgpt_base_url: Option<String>,
        keyring_backend_kind: AuthKeyringBackendKind,
        auth_route_config: AuthRouteConfig,
    ) -> Self {
        Self::new_from_auth_config(
            AuthConfig {
                codex_home,
                auth_credentials_store_mode,
                keyring_backend_kind,
                forced_login_method: None,
                chatgpt_base_url,
                forced_chatgpt_workspace_id,
                managed_auth_policy: ManagedAuthPolicy::default(),
                auth_route_config,
            },
            enable_codex_api_key_env,
            openai_api_key_env,
            /*allowed_auth_modes*/ None,
        )
        .await
    }

    async fn new_from_auth_config(
        auth_config: AuthConfig,
        enable_codex_api_key_env: bool,
        openai_api_key_env: OpenAiApiKeyEnv,
        allowed_auth_modes: Option<Vec<AuthMode>>,
    ) -> Self {
        let managed_auth = auth_config
            .load_auth_with_modes(
                enable_codex_api_key_env,
                openai_api_key_env,
                allowed_auth_modes.as_deref(),
            )
            .await
            .ok()
            .flatten();
        let AuthConfig {
            codex_home,
            auth_credentials_store_mode,
            keyring_backend_kind,
            forced_login_method,
            chatgpt_base_url,
            forced_chatgpt_workspace_id,
            managed_auth_policy,
            auth_route_config,
        } = auth_config;
        let agent_identity_authapi_base_url =
            agent_identity_authapi_base_url(chatgpt_base_url.as_deref()).ok();
        let (auth_change_tx, _auth_change_rx) = watch::channel(0);
        Self {
            codex_home,
            inner: RwLock::new(CachedAuth {
                auth: managed_auth,
                permanent_refresh_failure: None,
            }),
            auth_change_tx,
            auth_change_state_tx: watch::channel(AuthChangeState::default()).0,
            workspace_routing_resolver: OnceLock::new(),
            enable_codex_api_key_env,
            openai_api_key_env,
            auth_credentials_store_mode,
            keyring_backend_kind,
            forced_login_method,
            forced_chatgpt_workspace_id: RwLock::new(forced_chatgpt_workspace_id),
            managed_auth_policy,
            chatgpt_base_url,
            agent_identity_authapi_base_url,
            refresh_lock: Arc::new(Semaphore::new(/*permits*/ 1)),
            unsettled_chatgpt_refresh: Mutex::new(None),
            agent_identity_lock: Semaphore::new(/*permits*/ 1),
            agent_identity_bootstrap_cooldown: Mutex::default(),
            external_auth: RwLock::new(None),
            provider_api_key_cache: RwLock::new(ProviderApiKeyCache::new()),
            named_account: None,
            named_store: None,
            base_manager: None,
            hide_restricted_default_login: false,
            hidden_default_login_reason: RwLock::new(None),
            workload_identity_selected: false,
            auth_route_config,
            allowed_auth_modes: RwLock::new(allowed_auth_modes),
        }
    }

    /// Create an AuthManager with a specific CodexAuth, for testing only.
    pub fn from_auth_for_testing(auth: CodexAuth) -> Arc<Self> {
        Self::from_optional_auth_for_testing(Some(auth))
    }

    /// Create an AuthManager with an optional CodexAuth, for testing only.
    pub(crate) fn from_optional_auth_for_testing(auth: Option<CodexAuth>) -> Arc<Self> {
        let cached = CachedAuth {
            auth,
            permanent_refresh_failure: None,
        };
        let (auth_change_tx, _auth_change_rx) = watch::channel(0);

        Arc::new(Self {
            codex_home: PathBuf::from("non-existent"),
            inner: RwLock::new(cached),
            auth_change_tx,
            auth_change_state_tx: watch::channel(AuthChangeState::default()).0,
            workspace_routing_resolver: OnceLock::new(),
            enable_codex_api_key_env: false,
            openai_api_key_env: OpenAiApiKeyEnv::Ignore,
            auth_credentials_store_mode: AuthCredentialsStoreMode::File,
            keyring_backend_kind: AuthKeyringBackendKind::default(),
            forced_login_method: None,
            forced_chatgpt_workspace_id: RwLock::new(None),
            managed_auth_policy: ManagedAuthPolicy::default(),
            chatgpt_base_url: None,
            agent_identity_authapi_base_url: default_agent_identity_authapi_base_url(),
            refresh_lock: Arc::new(Semaphore::new(/*permits*/ 1)),
            unsettled_chatgpt_refresh: Mutex::new(None),
            agent_identity_lock: Semaphore::new(/*permits*/ 1),
            agent_identity_bootstrap_cooldown: Mutex::default(),
            external_auth: RwLock::new(None),
            provider_api_key_cache: RwLock::new(ProviderApiKeyCache::new()),
            named_account: None,
            named_store: None,
            base_manager: None,
            hide_restricted_default_login: false,
            hidden_default_login_reason: RwLock::new(None),
            workload_identity_selected: false,
            allowed_auth_modes: RwLock::new(None),
            auth_route_config: crate::test_support::transport_default_auth_route_config(),
        })
    }

    /// Create an AuthManager with a specific CodexAuth and codex home, for testing only.
    pub fn from_auth_for_testing_with_home(auth: CodexAuth, codex_home: PathBuf) -> Arc<Self> {
        let cached = CachedAuth {
            auth: Some(auth),
            permanent_refresh_failure: None,
        };
        let (auth_change_tx, _auth_change_rx) = watch::channel(0);
        Arc::new(Self {
            codex_home,
            inner: RwLock::new(cached),
            auth_change_tx,
            auth_change_state_tx: watch::channel(AuthChangeState::default()).0,
            workspace_routing_resolver: OnceLock::new(),
            enable_codex_api_key_env: false,
            openai_api_key_env: OpenAiApiKeyEnv::Ignore,
            auth_credentials_store_mode: AuthCredentialsStoreMode::File,
            keyring_backend_kind: AuthKeyringBackendKind::default(),
            forced_login_method: None,
            forced_chatgpt_workspace_id: RwLock::new(None),
            managed_auth_policy: ManagedAuthPolicy::default(),
            chatgpt_base_url: None,
            agent_identity_authapi_base_url: default_agent_identity_authapi_base_url(),
            refresh_lock: Arc::new(Semaphore::new(/*permits*/ 1)),
            unsettled_chatgpt_refresh: Mutex::new(None),
            agent_identity_lock: Semaphore::new(/*permits*/ 1),
            agent_identity_bootstrap_cooldown: Mutex::default(),
            external_auth: RwLock::new(None),
            provider_api_key_cache: RwLock::new(ProviderApiKeyCache::new()),
            named_account: None,
            named_store: None,
            base_manager: None,
            hide_restricted_default_login: false,
            hidden_default_login_reason: RwLock::new(None),
            workload_identity_selected: false,
            allowed_auth_modes: RwLock::new(None),
            auth_route_config: crate::test_support::transport_default_auth_route_config(),
        })
    }

    /// Create an AuthManager with a specific CodexAuth and Agent Identity AuthAPI base URL, for testing only.
    #[doc(hidden)]
    pub fn from_auth_for_testing_with_agent_identity_authapi_base_url(
        auth: CodexAuth,
        agent_identity_authapi_base_url: String,
    ) -> Arc<Self> {
        let cached = CachedAuth {
            auth: Some(auth),
            permanent_refresh_failure: None,
        };
        let (auth_change_tx, _auth_change_rx) = watch::channel(0);
        Arc::new(Self {
            codex_home: PathBuf::from("non-existent"),
            inner: RwLock::new(cached),
            auth_change_tx,
            auth_change_state_tx: watch::channel(AuthChangeState::default()).0,
            workspace_routing_resolver: OnceLock::new(),
            enable_codex_api_key_env: false,
            openai_api_key_env: OpenAiApiKeyEnv::Ignore,
            auth_credentials_store_mode: AuthCredentialsStoreMode::File,
            keyring_backend_kind: AuthKeyringBackendKind::default(),
            forced_login_method: None,
            forced_chatgpt_workspace_id: RwLock::new(None),
            managed_auth_policy: ManagedAuthPolicy::default(),
            chatgpt_base_url: None,
            agent_identity_authapi_base_url: Some(
                agent_identity_authapi_base_url
                    .trim_end_matches('/')
                    .to_string(),
            ),
            refresh_lock: Arc::new(Semaphore::new(/*permits*/ 1)),
            unsettled_chatgpt_refresh: Mutex::new(None),
            agent_identity_lock: Semaphore::new(/*permits*/ 1),
            agent_identity_bootstrap_cooldown: Mutex::default(),
            external_auth: RwLock::new(None),
            provider_api_key_cache: RwLock::new(ProviderApiKeyCache::new()),
            named_account: None,
            named_store: None,
            base_manager: None,
            hide_restricted_default_login: false,
            hidden_default_login_reason: RwLock::new(None),
            workload_identity_selected: false,
            allowed_auth_modes: RwLock::new(None),
            auth_route_config: crate::test_support::transport_default_auth_route_config(),
        })
    }

    pub fn external_bearer_only(config: ModelProviderAuthInfo) -> Arc<Self> {
        Self::external_bearer_only_with_env(config, HashMap::new())
    }

    /// Creates a provider-scoped token cache whose command environment overrides are
    /// retained for both initial authentication and every subsequent refresh.
    pub fn external_bearer_only_with_env(
        config: ModelProviderAuthInfo,
        env: HashMap<String, OsString>,
    ) -> Arc<Self> {
        Self::external_bearer_only_for_account(
            config,
            ExternalBearerCachePolicy::Timed,
            /*account*/ None,
            env,
        )
    }

    pub fn external_bearer_only_with_cache_policy(
        config: ModelProviderAuthInfo,
        cache_policy: ExternalBearerCachePolicy,
    ) -> Arc<Self> {
        Self::external_bearer_only_for_account(
            config,
            cache_policy,
            /*account*/ None,
            HashMap::new(),
        )
    }

    /// PF-84: like [`Self::external_bearer_only_with_cache_policy`], for a
    /// named account the command receives in `CORBANU_PROVIDER_ACCOUNT`.
    pub fn external_bearer_only_for_account(
        config: ModelProviderAuthInfo,
        cache_policy: ExternalBearerCachePolicy,
        account: Option<super::external_bearer::ExternalBearerAccount>,
        env: HashMap<String, OsString>,
    ) -> Arc<Self> {
        let (auth_change_tx, _auth_change_rx) = watch::channel(0);
        Arc::new(Self {
            codex_home: PathBuf::from("non-existent"),
            inner: RwLock::new(CachedAuth {
                auth: None,
                permanent_refresh_failure: None,
            }),
            auth_change_tx,
            auth_change_state_tx: watch::channel(AuthChangeState::default()).0,
            workspace_routing_resolver: OnceLock::new(),
            enable_codex_api_key_env: false,
            openai_api_key_env: OpenAiApiKeyEnv::Ignore,
            auth_credentials_store_mode: AuthCredentialsStoreMode::File,
            keyring_backend_kind: AuthKeyringBackendKind::default(),
            forced_login_method: None,
            forced_chatgpt_workspace_id: RwLock::new(None),
            managed_auth_policy: ManagedAuthPolicy::default(),
            chatgpt_base_url: None,
            agent_identity_authapi_base_url: default_agent_identity_authapi_base_url(),
            refresh_lock: Arc::new(Semaphore::new(/*permits*/ 1)),
            unsettled_chatgpt_refresh: Mutex::new(None),
            agent_identity_lock: Semaphore::new(/*permits*/ 1),
            agent_identity_bootstrap_cooldown: Mutex::default(),
            external_auth: RwLock::new(Some(Arc::new(BearerTokenRefresher::new(
                config,
                cache_policy,
                account,
                env,
            )) as Arc<dyn ExternalAuth>)),
            provider_api_key_cache: RwLock::new(ProviderApiKeyCache::new()),
            named_account: None,
            named_store: None,
            base_manager: None,
            hide_restricted_default_login: false,
            hidden_default_login_reason: RwLock::new(None),
            workload_identity_selected: false,
            allowed_auth_modes: RwLock::new(None),
            // External bearer auth refreshes by running the provider's command and never makes
            // auth-owned HTTP requests, so this route is intentionally inert.
            auth_route_config: AuthRouteConfig::from_http_client_factory(HttpClientFactory::new(
                OutboundProxyPolicy::ReqwestDefault,
            )),
        })
    }

    /// Current cached auth (clone) without attempting a refresh.
    ///
    /// D171 (#1286): a named account's sign-in is dropped once the account
    /// was removed or signed out (by any process); [`AuthManager::auth`]
    /// picks up a later sign-in.
    pub fn auth_cached(&self) -> Option<CodexAuth> {
        loop {
            match self.held_named_sign_in() {
                named_account::HeldSignIn::Current => {}
                named_account::HeldSignIn::Changed => self.reload_named_sign_in_sync(),
                named_account::HeldSignIn::Gone { epoch } => {
                    if !self.clear_named_sign_in_if_unchanged(epoch) {
                        // The replacement may itself have been removed since
                        // it was installed. Check it before returning it.
                        continue;
                    }
                }
            }
            return self.auth_cached_unchecked();
        }
    }

    /// The cached auth as it is, for the guarded reloads that compare it with
    /// the stored one.
    fn auth_cached_unchecked(&self) -> Option<CodexAuth> {
        self.inner
            .read()
            .ok()
            .and_then(|cached| cached.auth.clone())
    }

    /// Current revisions if the selected credential still belongs to this manager's source.
    pub fn auth_change_state_for(&self, expected: &CodexAuth) -> Option<AuthChangeState> {
        let cached = self.inner.read().ok()?;
        if !super::source::same_source(cached.auth.as_ref(), Some(expected)) {
            return None;
        }
        Some(*self.auth_change_state_tx.borrow())
    }

    /// Subscribes to cached auth changes that can affect request recovery.
    pub fn auth_change_receiver(&self) -> watch::Receiver<u64> {
        self.auth_change_tx.subscribe()
    }

    /// Subscribes to credential and owner revisions published together, including when changes coalesce.
    pub fn auth_change_state_receiver(&self) -> watch::Receiver<AuthChangeState> {
        self.auth_change_state_tx.subscribe()
    }

    pub fn refresh_failure_for_auth(&self, auth: &CodexAuth) -> Option<RefreshTokenFailedError> {
        self.inner.read().ok().and_then(|cached| {
            cached
                .permanent_refresh_failure
                .as_ref()
                .filter(|failure| Self::auths_equal_for_refresh(Some(auth), Some(&failure.auth)))
                .map(|failure| failure.error.clone())
        })
    }

    /// Whether this credential's refresh exchange or persistence started but did not finish.
    /// Cancellation retains the record so callers can withhold an uncertain generation.
    /// Waiting for the refresh guard or reloading storage does not mark a credential.
    pub fn is_chatgpt_refresh_unsettled(&self, auth: &CodexAuth) -> std::io::Result<bool> {
        let unsettled = self
            .unsettled_chatgpt_refresh
            .lock()
            .map_err(|_| std::io::Error::other("Refresh progress is unavailable."))?;
        // Match the stored generation; get_token_data() rewrites workspace claims.
        Ok(unsettled.as_ref().is_some_and(|tokens| {
            auth.get_current_auth_json()
                .and_then(|auth| auth.tokens)
                .as_ref()
                == Some(tokens)
        }))
    }

    /// Current cached auth (clone). May be `None` if not logged in or load failed.
    /// For managed ChatGPT auth that needs a proactive refresh, first performs
    /// a guarded reload and then refreshes only if the on-disk auth is unchanged.
    #[instrument(level = "trace", skip_all)]
    pub async fn auth(&self) -> Option<CodexAuth> {
        // A named account loads lazily, and picks up a sign-in added later.
        // D171 (#1286): `auth_cached` drops a sign-in held in memory once the
        // account was removed or replaced (by any process), so the next
        // request is refused with its recovery or runs on the new sign-in.
        if self.named_account.is_some() && self.auth_cached().is_none() {
            self.reload().await;
        }
        if self.has_external_auth() {
            self.reload().await;
            return self.auth_cached();
        }

        let auth = self.auth_cached()?;
        if let Some(reason) = Self::proactive_refresh_reason(&auth)
            && let Err(err) = self.refresh_token_with_reason(reason, Some(&auth)).await
        {
            tracing::error!("Failed to refresh token: {}", err);
            // D171: a named account removed meanwhile is not used.
            if self.named_account.is_some() {
                return self.auth_cached();
            }
            return Some(auth);
        }
        self.auth_cached()
    }

    /// Refreshes auth, then captures credentials and their account-bound factory together.
    /// The auth read lock prevents an identity change between the two snapshots.
    pub async fn auth_with_http_client_factory(&self) -> Option<(CodexAuth, HttpClientFactory)> {
        self.auth().await;
        let cached = self.inner.read().ok()?;
        Some((cached.auth.clone()?, self.http_client_factory()))
    }

    pub async fn agent_identity_auth(
        &self,
        policy: AgentIdentityAuthPolicy,
        session_source: SessionSource,
    ) -> std::io::Result<Option<AgentIdentityAuth>> {
        let Some(auth) = self.auth().await else {
            return Ok(None);
        };
        if policy == AgentIdentityAuthPolicy::ChatGptAuth && matches!(auth, CodexAuth::Chatgpt(_)) {
            let _bootstrap_permit = self
                .agent_identity_lock
                .acquire()
                .await
                .map_err(std::io::Error::other)?;
            let effective_chatgpt_workspaces = self.effective_chatgpt_workspaces();
            let cooldown_key = ManagedChatGptAgentIdentityBinding::from_auth(
                &auth,
                effective_chatgpt_workspaces.clone(),
            )
            .and_then(|binding| {
                self.agent_identity_authapi_base_url
                    .as_ref()
                    .map(|base_url| (binding.account_id, base_url.clone()))
            });
            if let Some((account_id, authapi_base_url)) = cooldown_key.as_ref()
                && let Ok(mut cooldown) = self.agent_identity_bootstrap_cooldown.lock()
                && let Some(error) =
                    cooldown.error_for(account_id, authapi_base_url, Instant::now())
            {
                tracing::warn!("agent identity bootstrap retry suppressed during shared cooldown");
                return Err(std::io::Error::other(error));
            }

            let result = auth
                .agent_identity_auth(
                    policy,
                    self.agent_identity_authapi_base_url.as_deref(),
                    effective_chatgpt_workspaces,
                    &self.auth_route_config,
                    session_source,
                )
                .await;
            if let Ok(mut cooldown) = self.agent_identity_bootstrap_cooldown.lock() {
                if let (Err(err), Some((account_id, authapi_base_url))) = (&result, cooldown_key)
                    && let Some(error) = AgentIdentityAuthError::bootstrap_unavailable(err).cloned()
                {
                    cooldown.record_failure(account_id, authapi_base_url, error, Instant::now());
                } else {
                    cooldown.clear();
                }
            }
            return result;
        }
        auth.agent_identity_auth(
            policy,
            self.agent_identity_authapi_base_url.as_deref(),
            self.effective_chatgpt_workspaces(),
            &self.auth_route_config,
            session_source,
        )
        .await
    }

    /// Borrows a persisted account grant while excluding this manager's token refreshes.
    /// Release the lease after the IdP exchange, then check the grant before using its result.
    pub async fn lease_account_oauth_grant(
        self: &Arc<Self>,
    ) -> std::io::Result<Option<AccountOAuthGrantLease>> {
        let permit = Arc::clone(&self.refresh_lock)
            .acquire_owned()
            .await
            .map_err(|_| std::io::Error::other("Account refresh coordination is unavailable."))?;
        Ok(self
            .account_oauth_grant()
            .await?
            .map(|grant| AccountOAuthGrantLease { grant, permit }))
    }

    /// Reads an owner-checked grant without waiting for or excluding account refresh.
    /// Use this for inspection; lease the grant again before exchanging its refresh token.
    pub async fn account_oauth_grant(
        self: &Arc<Self>,
    ) -> std::io::Result<Option<AccountOAuthGrant>> {
        let login_generation = self.auth_change_state_tx.borrow().login_generation;
        let Some(tokens) = self.read_account_grant_tokens().await? else {
            return Ok(None);
        };
        let grant = AccountOAuthGrant {
            manager: Arc::clone(self),
            tokens,
            login_generation,
        };
        if !grant.owner_is_current() {
            return Ok(None);
        }
        Ok(Some(grant))
    }

    /// Checks the persisted account owner without requiring an EMA-eligible account grant.
    /// Dedicated enterprise credentials still belong to this account login.
    pub async fn account_login_is_current(&self, expected: &CodexAuth) -> std::io::Result<bool> {
        if !matches!(expected, CodexAuth::Chatgpt(_)) {
            return Ok(self.auth_cached().is_some_and(|current| {
                !matches!(current, CodexAuth::Chatgpt(_))
                    && current.get_account_id() == expected.get_account_id()
                    && current.get_chatgpt_user_id() == expected.get_chatgpt_user_id()
            }));
        }
        let expected = expected.get_token_data()?;
        if expected
            .login_owner_id
            .as_ref()
            .is_none_or(String::is_empty)
        {
            return Ok(false);
        }
        let matches_owner = || {
            self.auth_cached().is_some_and(|auth| {
                matches!(auth, CodexAuth::Chatgpt(_))
                    && auth
                        .get_token_data()
                        .is_ok_and(|tokens| account_grant::same_account_login(&expected, &tokens))
            })
        };
        if !matches_owner() {
            return Ok(false);
        }
        let stored = self.read_persisted_account_tokens().await?;
        Ok(matches_owner()
            && stored
                .as_ref()
                .is_some_and(|tokens| account_grant::same_account_login(&expected, tokens)))
    }

    async fn read_persisted_account_tokens(&self) -> std::io::Result<Option<TokenData>> {
        let home = self.codex_home.clone();
        let mode = self.auth_credentials_store_mode;
        let backend = self.keyring_backend_kind;
        let originator = codex_otel::auth_storage::AuthStorageOriginator::current();
        let stored = tokio::task::spawn_blocking(move || {
            originator.sync_scope(|| {
                let _commit = auth_storage_commit_guard()?;
                create_auth_storage(home, mode, backend).load_account_grant()
            })
        })
        .await
        .map_err(|_| std::io::Error::other("Account credential read failed."))?
        .map_err(|_: std::io::Error| {
            std::io::Error::other("Account credential storage is unavailable.")
        })?;
        Ok(stored
            .filter(|auth| auth.resolved_mode() == AuthMode::Chatgpt)
            .and_then(|auth| auth.tokens))
    }

    async fn read_account_grant_tokens(&self) -> std::io::Result<Option<TokenData>> {
        if self.has_external_auth()
            || self.auth_credentials_store_mode == AuthCredentialsStoreMode::Ephemeral
        {
            return Ok(None);
        }
        let Some(auth @ CodexAuth::Chatgpt(_)) = self.auth_cached() else {
            return Ok(None);
        };
        let allowed = self.allowed_login_methods();
        if validate_auth_restrictions(
            Some(&allowed),
            self.effective_chatgpt_workspaces().as_deref(),
            &auth,
        )
        .is_err()
        {
            return Ok(None);
        }
        let cached = auth.get_token_data()?;
        let stored = self.read_persisted_account_tokens().await?;
        Ok(stored.filter(|tokens| {
            tokens.oauth_provenance.as_ref().is_some_and(|provenance| {
                !provenance.issuer.trim().is_empty() && !provenance.client_id.trim().is_empty()
            }) && tokens
                .login_owner_id
                .as_ref()
                .is_some_and(|id| !id.is_empty())
                && !tokens.refresh_token.is_empty()
                && tokens
                    .id_token
                    .chatgpt_user_id
                    .as_ref()
                    .is_some_and(|id| !id.is_empty())
                && tokens.account_id.as_ref().is_some_and(|id| !id.is_empty())
                && same_account_grant(tokens, &cached)
        }))
    }

    /// Reloads auth from the active source. Returns whether the auth value changed.
    pub async fn reload(&self) -> bool {
        tracing::info!("Reloading auth");
        loop {
            let generation = self.auth_change_state_tx.borrow().generation;
            let new_auth = self.load_auth(/*cached_auth*/ None).await.ok().flatten();
            if let Some(publication) = self.publish_reloaded_auth(generation, new_auth) {
                return publication.changed;
            }
            if self.inner.is_poisoned() {
                return false;
            }
        }
    }

    /// Pins the stored credential kind before any subsequent load or hydration.
    /// This prevents a running client from silently acquiring a different kind of secret.
    pub fn restrict_auth_modes(&self, mut modes: Vec<AuthMode>) -> std::io::Result<()> {
        let mut allowed = self
            .allowed_auth_modes
            .write()
            .map_err(|_| std::io::Error::other("credential policy is unavailable"))?;
        if let Some(previous) = allowed.as_ref() {
            modes.retain(|mode| previous.contains(mode));
        }
        *allowed = Some(modes);
        Ok(())
    }

    /// Reloads stored credentials without hiding storage or hydration failures.
    /// Failed loads retain the last known owner so a temporary outage does not
    /// invalidate application network policy as if the user had logged out.
    /// Callers must serialize reload and refresh and reject cached headers on error.
    pub async fn reload_with_cached_auth(&self) -> std::io::Result<bool> {
        loop {
            let generation = self.auth_change_state_tx.borrow().generation;
            let cached = self.auth_cached();
            // PF-84-S06: a named account's manager reloads only that
            // account's store, never the default login.
            let new_auth = if self.named_account.is_some() {
                self.load_named_account_auth().await
            } else {
                self.try_load_managed_auth(cached.as_ref()).await?
            };
            if let Some(publication) = self.publish_reloaded_auth(generation, new_auth) {
                return Ok(publication.changed);
            }
            if self.inner.is_poisoned() {
                return Err(std::io::Error::other("credential cache is unavailable"));
            }
        }
    }

    async fn reload_if_account_id_matches(
        &self,
        expected_account_id: Option<&str>,
    ) -> std::io::Result<ReloadOutcome> {
        let expected_account_id = match expected_account_id {
            Some(account_id) => account_id,
            None => {
                tracing::info!("Skipping auth reload because no account id is available.");
                return Ok(ReloadOutcome::Skipped);
            }
        };

        loop {
            let generation = self.auth_change_state_tx.borrow().generation;
            let new_auth = self.load_auth(/*cached_auth*/ None).await?;
            let new_account_id = new_auth.as_ref().and_then(CodexAuth::get_account_id);

            if new_account_id.as_deref() != Some(expected_account_id) {
                let found_account_id = new_account_id.as_deref().unwrap_or("unknown");
                tracing::info!(
                    "Skipping auth reload due to account id mismatch (expected: {expected_account_id}, found: {found_account_id})"
                );
                return Ok(ReloadOutcome::Skipped);
            }

            tracing::info!("Reloading auth for account {expected_account_id}");
            if let Some(publication) = self.publish_reloaded_auth(generation, new_auth) {
                return Ok(if publication.changed_for_refresh {
                    ReloadOutcome::ReloadedChanged
                } else {
                    ReloadOutcome::ReloadedNoChange
                });
            }
            if self.inner.is_poisoned() {
                return Ok(ReloadOutcome::Skipped);
            }
        }
    }

    fn auths_equal_for_refresh(a: Option<&CodexAuth>, b: Option<&CodexAuth>) -> bool {
        match (a, b) {
            (None, None) => true,
            (Some(a), Some(b)) => match (a.api_auth_mode(), b.api_auth_mode()) {
                (AuthMode::ApiKey, AuthMode::ApiKey) => a.api_key() == b.api_key(),
                (AuthMode::Chatgpt, AuthMode::Chatgpt) => {
                    a.get_current_auth_json() == b.get_current_auth_json()
                }
                (AuthMode::ChatgptAuthTokens, AuthMode::ChatgptAuthTokens) => {
                    a.get_current_token_data() == b.get_current_token_data()
                }
                (AuthMode::Headers, AuthMode::Headers) => a == b,
                (AuthMode::AgentIdentity, AuthMode::AgentIdentity) => match (a, b) {
                    (CodexAuth::AgentIdentity(a), CodexAuth::AgentIdentity(b)) => {
                        a.record() == b.record()
                    }
                    _ => false,
                },
                (AuthMode::PersonalAccessToken, AuthMode::PersonalAccessToken) => a == b,
                (AuthMode::BedrockApiKey, AuthMode::BedrockApiKey) => a == b,
                (AuthMode::BedrockAccessKeys, AuthMode::BedrockAccessKeys) => a == b,
                _ => false,
            },
            _ => false,
        }
    }

    fn auths_equal(a: Option<&CodexAuth>, b: Option<&CodexAuth>) -> bool {
        match (a, b) {
            (None, None) => true,
            (Some(a), Some(b)) => a == b,
            _ => false,
        }
    }

    /// Records a permanent refresh failure only if the failed refresh was
    /// attempted against the auth snapshot that is still cached.
    fn record_permanent_refresh_failure_if_unchanged(
        &self,
        attempted_auth: &CodexAuth,
        error: &RefreshTokenFailedError,
    ) {
        if let Ok(mut guard) = self.inner.write() {
            let current_auth_matches =
                Self::auths_equal_for_refresh(Some(attempted_auth), guard.auth.as_ref());
            if current_auth_matches {
                guard.permanent_refresh_failure = Some(AuthScopedRefreshFailure {
                    auth: attempted_auth.clone(),
                    error: error.clone(),
                });
            }
        }
    }

    async fn load_auth(
        &self,
        cached_auth: Option<&CodexAuth>,
    ) -> std::io::Result<Option<CodexAuth>> {
        if self.named_account.is_some() {
            return Ok(self.load_named_account_auth().await);
        }
        if let Some(external_auth) = self.external_auth_provider() {
            self.clear_hidden_default_login_reason();
            let cached_auth = self.auth_cached();
            if cached_auth
                .as_ref()
                .is_some_and(|auth| self.refresh_failure_for_auth(auth).is_some())
            {
                return Ok(cached_auth);
            }
            let auth = match self.resolve_external_auth(external_auth.as_ref()).await {
                Ok(auth) => Some(auth),
                Err(err) => {
                    tracing::error!("Failed to resolve external auth: {err}");
                    match err {
                        RefreshTokenError::Permanent(error) => {
                            if let Some(auth) = cached_auth.as_ref() {
                                self.record_permanent_refresh_failure_if_unchanged(auth, &error);
                            }
                            cached_auth
                        }
                        RefreshTokenError::Transient(_) => None,
                        RefreshTokenError::Policy(_) => cached_auth,
                    }
                }
            };
            return Ok(auth);
        }

        self.try_load_managed_auth(cached_auth).await
    }

    async fn try_load_managed_auth(
        &self,
        cached_auth: Option<&CodexAuth>,
    ) -> std::io::Result<Option<CodexAuth>> {
        let allowed_auth_modes = self
            .allowed_auth_modes
            .read()
            .map_err(|_| std::io::Error::other("credential policy is unavailable"))?
            .clone();
        let allowed_login_methods = self.allowed_login_methods();
        let effective_chatgpt_workspaces = self.effective_chatgpt_workspaces();
        // PF-84-S06 (#570): a manager that hides a restricted default login
        // loads it unfiltered, as main did, so the reason it is hidden is
        // recorded before the login policy below drops it.
        let hides_restricted = self.hides_restricted_default_login();
        let auth = load_auth_with_openai_api_key_env(
            &self.codex_home,
            self.enable_codex_api_key_env,
            self.openai_api_key_env,
            self.auth_credentials_store_mode,
            (!hides_restricted).then_some(allowed_login_methods.as_slice()),
            if hides_restricted {
                None
            } else {
                effective_chatgpt_workspaces.as_deref()
            },
            self.chatgpt_base_url.as_deref(),
            self.keyring_backend_kind,
            self.agent_identity_authapi_base_url.as_deref(),
            &self.auth_route_config,
            cached_auth,
            allowed_auth_modes.as_deref(),
        )
        .await?;
        let auth = self.without_restricted_default_login(auth);
        Ok(auth.filter(|auth| {
            validate_auth_restrictions(
                Some(&allowed_login_methods),
                effective_chatgpt_workspaces.as_deref(),
                auth,
            )
            .is_ok()
        }))
    }

    /// PF-84-S06 (#570): whether this default-login manager treats a login
    /// the restrictions rule out as absent, because the process left it in
    /// place at startup (see [`leave_default_login_unenforced`]).
    pub fn hides_restricted_default_login(&self) -> bool {
        self.hide_restricted_default_login && self.named_account.is_none()
    }

    /// PF-84-S06 (#570): why the default login this manager hides cannot be
    /// used, while it hides one. Requests on the default login are refused
    /// with it instead of being sent without credentials.
    pub fn hidden_default_login_reason(&self) -> Option<String> {
        self.hidden_default_login_reason
            .read()
            .ok()
            .and_then(|reason| reason.clone())
    }

    /// PF-84-S06 (#570): drops a default login the login restrictions rule
    /// out when this process left it in place for a named-account session.
    fn without_restricted_default_login(&self, auth: Option<CodexAuth>) -> Option<CodexAuth> {
        if !self.hides_restricted_default_login() {
            return auth;
        }
        let forced_workspace = self.forced_chatgpt_workspace_id();
        let reason = auth.as_ref().and_then(|auth| {
            login_restriction_violation(auth, self.forced_login_method, forced_workspace.as_deref())
                .map(|violation| default_login_unusable_message(violation.reason()))
        });
        if let Some(reason) = &reason {
            tracing::warn!("the default login is not used in this process: {reason}");
        }
        let hidden = reason.is_some();
        if let Ok(mut current) = self.hidden_default_login_reason.write() {
            *current = reason;
        }
        if hidden { None } else { auth }
    }

    fn set_cached_auth(&self, new_auth: Option<CodexAuth>) -> bool {
        let Ok(mut guard) = self.inner.write() else {
            return false;
        };
        self.set_cached_auth_locked(&mut guard, new_auth)
    }

    fn publish_reloaded_auth(
        &self,
        generation: u64,
        auth: Option<CodexAuth>,
    ) -> Option<AuthReload> {
        let mut guard = self.inner.write().ok()?;
        if self.auth_change_state_tx.borrow().generation != generation {
            return None;
        }
        let changed_for_refresh =
            !Self::auths_equal_for_refresh(guard.auth.as_ref(), auth.as_ref());
        let changed = self.set_cached_auth_locked(&mut guard, auth);
        Some(AuthReload {
            changed,
            changed_for_refresh,
        })
    }

    fn invalidate_pending_auth_reads(&self) {
        if let Ok(_guard) = self.inner.write() {
            self.auth_change_state_tx
                .send_modify(|state| state.generation += 1);
        }
    }

    fn set_cached_auth_locked(&self, guard: &mut CachedAuth, new_auth: Option<CodexAuth>) -> bool {
        let previous = guard.auth.as_ref();
        let changed = !AuthManager::auths_equal(previous, new_auth.as_ref());
        let auth_changed_for_refresh = !Self::auths_equal_for_refresh(previous, new_auth.as_ref());
        let owner_changed = auth_changed_for_refresh && !same_owner(previous, new_auth.as_ref());
        let source_changed =
            auth_changed_for_refresh && !super::source::same_source(previous, new_auth.as_ref());
        let login_changed = auth_changed_for_refresh && !same_login(previous, new_auth.as_ref());
        if owner_changed {
            self.auth_route_config
                .application_network_policy()
                .invalidate();
        }
        self.clear_provider_api_key_cache();
        if auth_changed_for_refresh {
            guard.permanent_refresh_failure = None;
        }
        guard.auth = new_auth;
        // #1326: every cache installation invalidates earlier observations,
        // including a reinstall of the same credentials after removal.
        // Equality still controls the return value and refresh notifications.
        if let Some(store) = self.named_store.as_ref() {
            store.forget();
        }
        if auth_changed_for_refresh {
            self.auth_change_state_tx.send_modify(|state| {
                state.generation += 1;
                if owner_changed {
                    state.owner_generation += 1;
                }
                if source_changed {
                    state.source_generation += 1;
                }
                if login_changed {
                    state.login_generation += 1;
                }
            });
            self.auth_change_tx.send_modify(|revision| *revision += 1);
        }
        tracing::info!(
            auth_changed_for_refresh,
            "Reloaded auth, changed: {changed}"
        );
        changed
    }

    /// Replaces auth while the caller holds the cache write lock, including
    /// invalidation and notification for every guarded cache update.
    fn replace_cached_auth(&self, cached: &mut CachedAuth, new_auth: Option<CodexAuth>) -> bool {
        self.set_cached_auth_locked(cached, new_auth)
    }

    pub async fn set_external_auth(
        &self,
        external_auth: Arc<dyn ExternalAuth>,
    ) -> Result<(), RefreshTokenError> {
        if self.workload_identity_selected {
            return Err(permanent_external_auth_error(
                "workload identity auth cannot be replaced at runtime",
            ));
        }
        self.install_external_auth(external_auth).await
    }

    async fn install_external_auth(
        &self,
        external_auth: Arc<dyn ExternalAuth>,
    ) -> Result<(), RefreshTokenError> {
        let auth = self.resolve_external_auth(external_auth.as_ref()).await?;
        let mut external_auth_slot = self.external_auth.write().map_err(|_| {
            RefreshTokenError::Transient(std::io::Error::other("external auth lock is poisoned"))
        })?;
        *external_auth_slot = Some(external_auth);
        drop(external_auth_slot);
        if let Ok(mut guard) = self.inner.write() {
            guard.permanent_refresh_failure = None;
        }
        self.commit_external_auth(auth)
    }

    pub fn clear_external_auth(&self) {
        if self.workload_identity_selected {
            return;
        }
        if let Ok(mut external_auth) = self.external_auth.write()
            && external_auth.take().is_some()
        {
            self.clear_hidden_default_login_reason();
            self.set_cached_auth(/*new_auth*/ None);
        }
    }

    fn clear_hidden_default_login_reason(&self) {
        if let Ok(mut reason) = self.hidden_default_login_reason.write() {
            *reason = None;
        }
    }

    pub fn set_forced_chatgpt_workspace_id(&self, workspace_id: Option<Vec<String>>) {
        if let Ok(mut guard) = self.forced_chatgpt_workspace_id.write()
            && *guard != workspace_id
        {
            *guard = workspace_id;
        }
    }

    pub fn forced_chatgpt_workspace_id(&self) -> Option<Vec<String>> {
        self.forced_chatgpt_workspace_id
            .read()
            .ok()
            .and_then(|guard| guard.clone())
    }

    pub fn effective_chatgpt_workspaces(&self) -> Option<Vec<String>> {
        self.managed_auth_policy
            .effective_chatgpt_workspaces(self.forced_chatgpt_workspace_id().as_deref())
    }

    pub fn is_login_method_allowed(&self, method: ForcedLoginMethod) -> bool {
        self.managed_auth_policy.allows_login_method(
            method,
            self.forced_login_method,
            self.forced_chatgpt_workspace_id().as_deref(),
        )
    }

    /// Returns the login methods permitted by the current effective authentication policy.
    pub fn allowed_login_methods(&self) -> Vec<ForcedLoginMethod> {
        self.managed_auth_policy.allowed_login_methods(
            self.forced_login_method,
            self.forced_chatgpt_workspace_id().as_deref(),
        )
    }

    pub fn has_external_auth(&self) -> bool {
        self.external_auth_provider().is_some()
    }

    pub fn is_workload_identity_selected(&self) -> bool {
        self.workload_identity_selected
    }

    pub fn is_external_chatgpt_auth_active(&self) -> bool {
        self.auth_cached()
            .as_ref()
            .is_some_and(CodexAuth::is_external_chatgpt_tokens)
    }

    pub fn codex_api_key_env_enabled(&self) -> bool {
        self.enable_codex_api_key_env
    }

    /// Returns policy only; independent credential managers own their own state and lifecycle.
    pub fn runtime_config(&self) -> AuthRuntimeConfig {
        AuthRuntimeConfig {
            codex_home: self.codex_home.clone(),
            auth_route_config: self.auth_route_config.clone(),
        }
    }

    /// Convenience constructor returning an `Arc` wrapper.
    pub async fn shared(
        codex_home: PathBuf,
        enable_codex_api_key_env: bool,
        auth_credentials_store_mode: AuthCredentialsStoreMode,
        forced_chatgpt_workspace_id: Option<Vec<String>>,
        chatgpt_base_url: Option<String>,
        keyring_backend_kind: AuthKeyringBackendKind,
        auth_route_config: AuthRouteConfig,
    ) -> Arc<Self> {
        Arc::new(
            Self::new(
                codex_home,
                enable_codex_api_key_env,
                auth_credentials_store_mode,
                forced_chatgpt_workspace_id,
                chatgpt_base_url,
                keyring_backend_kind,
                auth_route_config,
            )
            .await,
        )
    }

    /// Builds a shared manager and activates process-configured workload identity when selected.
    pub async fn shared_from_config(
        config: &impl AuthManagerConfig,
        enable_codex_api_key_env: bool,
    ) -> Result<Arc<Self>, AuthManagerInitializationError> {
        Self::shared_from_auth_config_with_session(
            auth_config_from(config),
            enable_codex_api_key_env,
            /*allowed_auth_modes*/ None,
            SessionAuthOptions {
                openai_api_key_env: OpenAiApiKeyEnv::for_forced_login_method(
                    config.forced_login_method(),
                ),
                hide_restricted_default_login: config.runs_on_named_sign_in_account()
                    || default_login_left_unenforced(),
            },
        )
        .await
    }

    /// Activates workload identity against an auth config resolved before full runtime config.
    pub async fn shared_from_auth_config(
        auth_config: AuthConfig,
        enable_codex_api_key_env: bool,
    ) -> Result<Arc<Self>, AuthManagerInitializationError> {
        Self::shared_from_auth_config_with_modes(
            auth_config,
            enable_codex_api_key_env,
            /*allowed_auth_modes*/ None,
        )
        .await
    }

    /// Restricts credential modes before loading or hydrating the initial credential.
    pub async fn shared_from_config_with_auth_modes(
        config: &impl AuthManagerConfig,
        enable_codex_api_key_env: bool,
        allowed_auth_modes: Vec<AuthMode>,
    ) -> Result<Arc<Self>, AuthManagerInitializationError> {
        Self::shared_from_auth_config_with_auth_modes(
            auth_config_from(config),
            enable_codex_api_key_env,
            allowed_auth_modes,
        )
        .await
    }

    /// Restricts credential modes during bootstrap, before loading runtime configuration.
    pub async fn shared_from_auth_config_with_auth_modes(
        auth_config: AuthConfig,
        enable_codex_api_key_env: bool,
        allowed_auth_modes: Vec<AuthMode>,
    ) -> Result<Arc<Self>, AuthManagerInitializationError> {
        Self::shared_from_auth_config_with_modes(
            auth_config,
            enable_codex_api_key_env,
            Some(allowed_auth_modes),
        )
        .await
    }

    async fn shared_from_auth_config_with_modes(
        auth_config: AuthConfig,
        enable_codex_api_key_env: bool,
        allowed_auth_modes: Option<Vec<AuthMode>>,
    ) -> Result<Arc<Self>, AuthManagerInitializationError> {
        let session = SessionAuthOptions::for_auth_config(&auth_config);
        Self::shared_from_auth_config_with_session(
            auth_config,
            enable_codex_api_key_env,
            allowed_auth_modes,
            session,
        )
        .await
    }

    async fn shared_from_auth_config_with_session(
        auth_config: AuthConfig,
        enable_codex_api_key_env: bool,
        allowed_auth_modes: Option<Vec<AuthMode>>,
        session: SessionAuthOptions,
    ) -> Result<Arc<Self>, AuthManagerInitializationError> {
        let external_auth = WorkloadIdentityExternalAuth::from_process_config(&auth_config)?;
        let mut manager = Self::new_from_auth_config(
            auth_config,
            enable_codex_api_key_env,
            session.openai_api_key_env,
            allowed_auth_modes,
        )
        .await;
        manager.workload_identity_selected = external_auth.is_some();
        // PF-84-S06 (#570): a session on a named sign-in account left the
        // default login in place; one the restrictions rule out is absent here.
        // The initial load applied the login policy, which would drop such a
        // login before its reason is recorded, so reload it the hiding way.
        manager.hide_restricted_default_login = session.hide_restricted_default_login;
        if manager.hide_restricted_default_login {
            let auth = manager
                .try_load_managed_auth(/*cached_auth*/ None)
                .await
                .ok()
                .flatten();
            if let Ok(mut cached) = manager.inner.write() {
                cached.auth = auth;
            }
        }
        let manager = Arc::new(manager);
        if let Some(external_auth) = external_auth {
            manager
                .install_external_auth(Arc::new(external_auth))
                .await?;
        }
        Ok(manager)
    }

    pub fn unauthorized_recovery(self: &Arc<Self>) -> UnauthorizedRecovery {
        UnauthorizedRecovery::new(Arc::clone(self))
    }

    fn external_auth_provider(&self) -> Option<Arc<dyn ExternalAuth>> {
        self.external_auth
            .read()
            .ok()
            .and_then(|external_auth| external_auth.clone())
    }

    fn has_refreshable_external_auth(&self) -> bool {
        self.has_external_auth()
            && self
                .auth_cached()
                .as_ref()
                .is_none_or(|auth| auth.is_api_key_auth() || auth.supports_unauthorized_recovery())
    }

    async fn resolve_external_auth(
        &self,
        external_auth: &dyn ExternalAuth,
    ) -> Result<CodexAuth, RefreshTokenError> {
        let auth = external_auth
            .resolve()
            .await
            .map_err(|error| external_auth.classify_error(error))?;
        self.validate_external_auth(&auth, external_auth)?;
        Ok(auth)
    }

    /// Refreshes external auth through its provider. For managed auth, first
    /// performs a guarded reload: if the stored token differs from the cached
    /// token, reuse it; otherwise, ask the token authority to refresh.
    pub async fn refresh_token(&self) -> Result<(), RefreshTokenError> {
        self.refresh_token_with_reason(RefreshReason::Forced, /*expected_auth*/ None)
            .await
    }

    /// Refreshes only if the expected credential is still current after acquiring
    /// the existing refresh guard, so queued callers reuse a completed refresh.
    pub async fn refresh_token_if_unchanged(
        &self,
        expected_auth: &CodexAuth,
    ) -> Result<(), RefreshTokenError> {
        self.refresh_token_with_reason(RefreshReason::Forced, Some(expected_auth))
            .await
    }

    async fn refresh_token_with_reason(
        &self,
        reason: RefreshReason,
        expected_auth: Option<&CodexAuth>,
    ) -> Result<(), RefreshTokenError> {
        let _refresh_guard = self.refresh_lock.acquire().await.map_err(|_| {
            RefreshTokenError::Permanent(RefreshTokenFailedError::new(
                RefreshTokenFailedReason::Other,
                REFRESH_TOKEN_UNKNOWN_MESSAGE.to_string(),
            ))
        })?;
        // D171: as held, so another process's refresh is picked up below.
        let auth_before_reload = self.auth_cached_unchecked();
        if let Some(expected_auth) = expected_auth
            && !Self::auths_equal_for_refresh(auth_before_reload.as_ref(), Some(expected_auth))
        {
            return Ok(());
        }
        if self.has_external_auth() {
            return self.refresh_token_from_authority_impl(reason).await;
        }
        if auth_before_reload
            .as_ref()
            .is_some_and(|auth| auth.is_api_key_auth() || auth.is_personal_access_token_auth())
        {
            return Ok(());
        }
        let expected_account_id = auth_before_reload
            .as_ref()
            .and_then(CodexAuth::get_account_id);

        match self
            .reload_if_account_id_matches(expected_account_id.as_deref())
            .await?
        {
            ReloadOutcome::ReloadedChanged => {
                tracing::info!("Skipping token refresh because auth changed after guarded reload.");
                Ok(())
            }
            ReloadOutcome::ReloadedNoChange => self.refresh_token_from_authority_impl(reason).await,
            ReloadOutcome::Skipped => {
                Err(RefreshTokenError::Permanent(RefreshTokenFailedError::new(
                    RefreshTokenFailedReason::Other,
                    REFRESH_TOKEN_ACCOUNT_MISMATCH_MESSAGE.to_string(),
                )))
            }
        }
    }

    /// Attempt to refresh the current auth token from the authority that issued
    /// it and update the shared cache. If the token refresh fails, returns the
    /// error to the caller.
    pub async fn refresh_token_from_authority(&self) -> Result<(), RefreshTokenError> {
        self.refresh_token_from_authority_with_reason(RefreshReason::Forced)
            .await
    }

    async fn refresh_token_from_authority_with_reason(
        &self,
        reason: RefreshReason,
    ) -> Result<(), RefreshTokenError> {
        let _refresh_guard = self.refresh_lock.acquire().await.map_err(|_| {
            RefreshTokenError::Permanent(RefreshTokenFailedError::new(
                RefreshTokenFailedReason::Other,
                REFRESH_TOKEN_UNKNOWN_MESSAGE.to_string(),
            ))
        })?;
        self.refresh_token_from_authority_impl(reason).await
    }

    async fn refresh_token_from_authority_impl(
        &self,
        reason: RefreshReason,
    ) -> Result<(), RefreshTokenError> {
        tracing::info!("Refreshing token");

        let attempted_auth = self.auth_cached();
        if let Some(error) = attempted_auth
            .as_ref()
            .and_then(|auth| self.refresh_failure_for_auth(auth))
        {
            return Err(RefreshTokenError::Permanent(error));
        }

        let result = if self.has_external_auth() {
            self.refresh_external_auth(ExternalAuthRefreshReason::Unauthorized)
                .await
        } else {
            match attempted_auth.as_ref() {
                Some(CodexAuth::Chatgpt(chatgpt_auth)) => {
                    let token_data = chatgpt_auth.current_token_data().ok_or_else(|| {
                        RefreshTokenError::Transient(std::io::Error::other(
                            "Token data is not available.",
                        ))
                    })?;
                    Box::pin(self.refresh_and_persist_chatgpt_token(
                        chatgpt_auth,
                        token_data,
                        reason,
                    ))
                    .await
                }
                Some(
                    CodexAuth::ApiKey(_)
                    | CodexAuth::ChatgptAuthTokens(_)
                    | CodexAuth::Headers(_)
                    | CodexAuth::AgentIdentity(_)
                    | CodexAuth::PersonalAccessToken(_)
                    | CodexAuth::BedrockApiKey(_)
                    | CodexAuth::BedrockAccessKeys(_),
                )
                | None => Ok(()),
            }
        };
        // PF-84-S06: a named account's refresh failure names how to recover it.
        let result = match (result, self.named_account.as_ref()) {
            (Err(RefreshTokenError::Permanent(error)), Some(account)) => {
                Err(RefreshTokenError::Permanent(RefreshTokenFailedError::new(
                    error.reason,
                    format!(
                        "{} (account `{}`: sign in again with `corbanu login --account {}`; \
                         the default login is unchanged)",
                        error.message, account.name, account.name
                    ),
                )))
            }
            (result, _) => result,
        };
        if let Some(attempted_auth) = attempted_auth.as_ref()
            && let Err(RefreshTokenError::Permanent(error)) = &result
        {
            self.record_permanent_refresh_failure_if_unchanged(attempted_auth, error);
        }
        result
    }

    /// Log out by deleting credentials from the selected storage backend and the
    /// ephemeral store. Returns `Ok(true)` if credentials were removed, `Ok(false)`
    /// if none were present. On success, reloads the in-memory auth cache so
    /// callers immediately observe the unauthenticated state.
    pub async fn logout(&self) -> std::io::Result<bool> {
        self.ensure_logout_allowed()?;
        if let Some(result) = self.logout_named_account(/*revoke*/ false).await {
            return result;
        }
        let removed = logout_all_stores(
            &self.codex_home,
            self.auth_credentials_store_mode,
            self.keyring_backend_kind,
            /*include_provider_credentials*/ false,
        )?;
        // Always reload to clear cached auth, even if no stored credentials were present.
        self.invalidate_pending_auth_reads();
        self.clear_external_auth();
        self.reload().await;
        Ok(removed)
    }

    pub async fn logout_all_credentials(&self) -> std::io::Result<bool> {
        if let Some(result) = self.logout_named_account(/*revoke*/ false).await {
            return result;
        }
        let removed = logout_all_stores(
            &self.codex_home,
            self.auth_credentials_store_mode,
            self.keyring_backend_kind,
            /*include_provider_credentials*/ true,
        )?;
        self.reload().await;
        Ok(removed)
    }

    pub async fn logout_with_revoke(&self) -> std::io::Result<bool> {
        self.ensure_logout_allowed()?;
        if let Some(result) = self.logout_named_account(/*revoke*/ true).await {
            return result;
        }
        let auth_dot_json = self
            .auth_cached()
            .and_then(|auth| auth.get_current_auth_json());
        if let Err(err) = revoke_auth_tokens(auth_dot_json.as_ref(), &self.auth_route_config).await
        {
            tracing::warn!("failed to revoke auth tokens during logout: {err}");
        }
        let result = logout_all_stores(
            &self.codex_home,
            self.auth_credentials_store_mode,
            self.keyring_backend_kind,
            /*include_provider_credentials*/ false,
        )?;
        // Always reload to clear cached auth, even if no stored credentials were present.
        self.invalidate_pending_auth_reads();
        self.clear_external_auth();
        self.reload().await;
        Ok(result)
    }

    fn ensure_logout_allowed(&self) -> std::io::Result<()> {
        if self.workload_identity_selected {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "workload identity auth is managed by the host and cannot be logged out",
            ));
        }
        Ok(())
    }

    /// Returns the precise kind of credentials backing the current authentication.
    pub fn get_api_auth_mode(&self) -> Option<AuthMode> {
        self.auth_cached().as_ref().map(CodexAuth::api_auth_mode)
    }

    /// Returns the effective backend auth mode for the current authentication.
    pub fn auth_mode(&self) -> Option<AuthMode> {
        self.auth_cached().as_ref().map(CodexAuth::auth_mode)
    }

    /// Whether `OPENAI_API_KEY` is read as the last-resort OpenAI credential.
    pub fn openai_api_key_env_enabled(&self) -> bool {
        self.openai_api_key_env == OpenAiApiKeyEnv::Fallback
    }

    /// Inspect the primary OpenAI auth boundary without returning credential material.
    pub fn openai_auth_metadata(&self) -> OpenAiAuthMetadata {
        let Some(auth) = self.auth_cached() else {
            return OpenAiAuthMetadata::Missing;
        };
        if auth.api_key_env_var().is_some()
            || (self.enable_codex_api_key_env && auth_uses_api_key_from_env(&auth))
        {
            return OpenAiAuthMetadata::EnvironmentApiKey;
        }
        if auth.is_external_chatgpt_tokens() {
            return OpenAiAuthMetadata::ExternallyManaged;
        }
        if self.refresh_failure_for_auth(&auth).is_some() {
            return OpenAiAuthMetadata::RecoveryRequired;
        }
        match auth.api_auth_mode() {
            AuthMode::Chatgpt | AuthMode::ChatgptAuthTokens | AuthMode::PersonalAccessToken => {
                OpenAiAuthMetadata::Account
            }
            AuthMode::ApiKey => OpenAiAuthMetadata::ApiKey,
            AuthMode::Headers | AuthMode::AgentIdentity => OpenAiAuthMetadata::ExternallyManaged,
            AuthMode::BedrockApiKey | AuthMode::BedrockAccessKeys => {
                OpenAiAuthMetadata::Unsupported
            }
        }
    }

    /// PF-84: the API key of one named account (vault only, uncached, no
    /// environment or default-account fallback).
    pub fn provider_account_api_key(
        &self,
        provider_id: &str,
        account_name: &str,
    ) -> std::io::Result<Option<String>> {
        super::provider_key_vault::provider_account_api_key(
            &self.codex_home,
            provider_id,
            account_name,
        )
    }

    pub fn provider_api_key(&self, provider_key_id: &str) -> std::io::Result<Option<String>> {
        let storage_revision = provider_api_key_storage_revision();
        if let Some(cached) = self.provider_api_key_cache.read().ok().and_then(|cache| {
            (cache.storage_revision == storage_revision)
                .then(|| cache.entries.get(provider_key_id).cloned())
                .flatten()
        }) {
            return Ok(cached);
        }

        let key = provider_api_key_from_auth_storage(
            &self.codex_home,
            provider_key_id,
            self.auth_credentials_store_mode,
            self.keyring_backend_kind,
        )?;
        if let Ok(mut cache) = self.provider_api_key_cache.write() {
            if cache.storage_revision != storage_revision {
                cache.storage_revision = storage_revision;
                cache.entries.clear();
            }
            cache
                .entries
                .insert(provider_key_id.to_string(), key.clone());
        }
        Ok(key)
    }

    fn clear_provider_api_key_cache(&self) {
        if let Ok(mut cache) = self.provider_api_key_cache.write() {
            cache.clear_to_current_revision();
        }
    }

    pub fn current_auth_uses_codex_backend(&self) -> bool {
        self.get_api_auth_mode()
            .is_some_and(AuthMode::uses_codex_backend)
    }

    fn proactive_refresh_reason(auth: &CodexAuth) -> Option<RefreshReason> {
        let chatgpt_auth = match auth {
            CodexAuth::Chatgpt(chatgpt_auth) => chatgpt_auth,
            _ => return None,
        };

        let auth_dot_json = chatgpt_auth.current_auth_json()?;
        if let Some(tokens) = auth_dot_json.tokens.as_ref()
            && let Ok(Some(expires_at)) = parse_jwt_expiration(&tokens.access_token)
        {
            return (expires_at
                <= Utc::now()
                    + chrono::Duration::minutes(CHATGPT_ACCESS_TOKEN_REFRESH_WINDOW_MINUTES))
            .then_some(RefreshReason::ProactiveExpiry);
        }
        let last_refresh = auth_dot_json.last_refresh?;
        (last_refresh < Utc::now() - chrono::Duration::days(TOKEN_REFRESH_INTERVAL))
            .then_some(RefreshReason::Scheduled)
    }

    async fn refresh_external_auth(
        &self,
        reason: ExternalAuthRefreshReason,
    ) -> Result<(), RefreshTokenError> {
        let Some(external_auth) = self.external_auth_provider() else {
            return Err(RefreshTokenError::Transient(std::io::Error::other(
                "external auth is not configured",
            )));
        };
        let previous_account_id = self
            .auth_cached()
            .as_ref()
            .and_then(CodexAuth::get_account_id);
        let context = ExternalAuthRefreshContext {
            reason,
            previous_account_id,
        };

        let refreshed = external_auth
            .refresh(context)
            .await
            .map_err(|error| external_auth.classify_error(error))?;
        self.validate_external_auth(&refreshed, external_auth.as_ref())?;
        self.commit_external_auth(refreshed)?;
        Ok(())
    }

    fn commit_external_auth(&self, auth: CodexAuth) -> Result<(), RefreshTokenError> {
        if auth.is_external_chatgpt_tokens() {
            let auth_dot_json = auth.get_current_auth_json().ok_or_else(|| {
                RefreshTokenError::Transient(std::io::Error::other(
                    "external ChatGPT auth tokens are missing auth state",
                ))
            })?;
            // Independent AuthManagers share external ChatGPT auth through the process-local store.
            save_auth(
                &self.codex_home,
                &auth_dot_json,
                AuthCredentialsStoreMode::Ephemeral,
                AuthKeyringBackendKind::default(),
            )
            .map_err(RefreshTokenError::Transient)?;
        }

        self.clear_hidden_default_login_reason();
        self.set_cached_auth(Some(auth));
        Ok(())
    }

    fn validate_external_auth(
        &self,
        auth: &CodexAuth,
        external_auth: &dyn ExternalAuth,
    ) -> Result<(), RefreshTokenError> {
        let allowed_login_methods = self.allowed_login_methods();
        validate_auth_restrictions(
            Some(&allowed_login_methods),
            self.effective_chatgpt_workspaces().as_deref(),
            auth,
        )
        .map_err(|error| external_auth.classify_error(std::io::Error::other(error)))
    }

    // Refreshes ChatGPT OAuth tokens, persists the updated auth state, and
    // reloads the in-memory cache so callers immediately observe new tokens.
    async fn refresh_and_persist_chatgpt_token(
        &self,
        auth: &ChatgptAuth,
        tokens: TokenData,
        reason: RefreshReason,
    ) -> Result<(), RefreshTokenError> {
        let endpoint = refresh_token_endpoint(tokens.oauth_provenance.as_ref());
        let client_id = tokens
            .oauth_provenance
            .as_ref()
            .map(|provenance| provenance.client_id.clone())
            .unwrap_or_else(oauth_client_id);
        let previous_unsettled = self
            .unsettled_chatgpt_refresh
            .lock()
            .map_err(|_| std::io::Error::other("Refresh progress is unavailable."))?
            .replace(tokens.clone());
        let result: Result<(), RefreshTokenError> = async {
            let refresh_response = request_chatgpt_token_refresh(
                tokens.refresh_token.clone(),
                &endpoint,
                &client_id,
                auth.client(),
                reason,
            )
            .await?;

            // An explicit override may send refresh to another authority. Retain provenance
            // only when the endpoint used by this request matches the recorded issuer.
            let provenance = tokens.oauth_provenance.clone().filter(|provenance| {
                endpoint == format!("{}/oauth/token", provenance.issuer.trim_end_matches('/'))
            });

            let storage = Arc::clone(auth.storage());
            let originator = codex_otel::auth_storage::AuthStorageOriginator::current();
            tokio::task::spawn_blocking(move || {
                originator.sync_scope(|| {
                    persist_tokens(
                        &storage,
                        &tokens,
                        provenance,
                        refresh_response.id_token,
                        refresh_response.access_token,
                        refresh_response.refresh_token,
                    )
                })
            })
            .await
            .map_err(|_| {
                RefreshTokenError::Transient(std::io::Error::other(
                    "credential persistence task failed",
                ))
            })?
            .map_err(RefreshTokenError::from)?;
            Ok(())
        }
        .await;
        // Dropping this future while exchange or persistence is pending leaves
        // the attempted generation recorded. A known failure cannot settle an
        // earlier cancelled attempt; successful persistence can.
        *self
            .unsettled_chatgpt_refresh
            .lock()
            .map_err(|_| std::io::Error::other("Refresh progress is unavailable."))? =
            if result.is_ok() {
                None
            } else {
                previous_unsettled
            };
        result?;
        self.reload().await;

        Ok(())
    }
}

#[path = "manager_named_account.rs"]
mod named_account;

#[path = "manager_session_account.rs"]
mod session_account;
pub use session_account::SessionSignIn;
pub use session_account::SessionSignInRegistration;

/// Corbanu session settings applied while a shared manager is built.
struct SessionAuthOptions {
    /// Whether `OPENAI_API_KEY` is a fallback credential for this session.
    openai_api_key_env: OpenAiApiKeyEnv,
    /// PF-84-S06 (#570): see [`AuthManager::hides_restricted_default_login`].
    hide_restricted_default_login: bool,
}

impl SessionAuthOptions {
    fn for_auth_config(auth_config: &AuthConfig) -> Self {
        Self {
            openai_api_key_env: OpenAiApiKeyEnv::for_forced_login_method(
                auth_config.forced_login_method,
            ),
            hide_restricted_default_login: default_login_left_unenforced(),
        }
    }
}

fn auth_config_from(config: &impl AuthManagerConfig) -> AuthConfig {
    AuthConfig {
        codex_home: config.codex_home(),
        auth_credentials_store_mode: config.cli_auth_credentials_store_mode(),
        keyring_backend_kind: config.auth_keyring_backend_kind(),
        // #571: the configured method only; the API-key-only onboarding
        // default a third-party session provider implies must not hide a
        // ChatGPT login (it only stops the `OPENAI_API_KEY` fallback).
        forced_login_method: config.configured_forced_login_method(),
        chatgpt_base_url: Some(config.chatgpt_base_url()),
        forced_chatgpt_workspace_id: config.forced_chatgpt_workspace_id(),
        managed_auth_policy: config.managed_auth_policy(),
        auth_route_config: config.auth_route_config(),
    }
}

#[cfg(test)]
#[path = "auth_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "change_state_tests.rs"]
mod change_state_tests;

#[cfg(test)]
#[path = "account_user_id_tests.rs"]
mod account_user_id_tests;

#[cfg(test)]
#[path = "source_tests.rs"]
mod source_tests;
