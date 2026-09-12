use codex_security_policy::CapabilityId;
use codex_security_policy::CredentialCapabilityRequest;
use codex_security_policy::CredentialHttpMethod;
use codex_security_policy::CredentialTransport;
use std::fmt;
use std::sync::Arc;
use thiserror::Error;

pub(super) const OPENAI_API_HOST: &str = "api.openai.com";
pub(super) const OPENAI_API_PORT: u16 = 443;
pub(super) const OPENAI_API_PATH_PREFIX: &str = "/v1/";
pub(super) const OPENAI_API_KEY_ENV_VAR: &str = "OPENAI_API_KEY";

/// Complete trusted and transport-derived context checked immediately before
/// scoped credential resolution.
pub struct ScopedCredentialUse<'a> {
    pub scheme: &'a str,
    pub host: &'a str,
    pub port: u16,
    pub method: &'a str,
    pub path: &'a str,
    pub capability_id: &'a CapabilityId,
    pub authority: &'a CredentialCapabilityRequest,
}

/// Stable callback failure that cannot carry credential material.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScopedCredentialCallbackError {
    Failed,
}

/// Stable, secret-free result from the trusted credential resolver.
#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum ScopedCredentialResolverError {
    #[error("scoped credential authority was denied")]
    Denied,
    #[error("scoped credential authority is expired")]
    Expired,
    #[error("scoped credential authority is revoked")]
    Revoked,
    #[error("scoped credential is unavailable")]
    Unavailable,
    #[error("scoped credential callback failed")]
    CallbackFailed,
}

/// Trusted callback boundary implemented by Core.
///
/// Implementations must compare the complete use context with their authorized
/// capability before resolving and must not retain or return credential
/// material.
pub trait ScopedCredentialResolver: Send + Sync + 'static {
    fn resolve(
        &self,
        request: &ScopedCredentialUse<'_>,
        callback: &mut dyn FnMut(&str) -> Result<(), ScopedCredentialCallbackError>,
    ) -> Result<(), ScopedCredentialResolverError>;
}

/// One secret-free OpenAI credential route installed by trusted Core code.
pub struct ScopedCredentialRoute {
    capability_id: CapabilityId,
    authority: CredentialCapabilityRequest,
    resolver: Arc<dyn ScopedCredentialResolver>,
}

impl ScopedCredentialRoute {
    pub fn new(
        capability_id: CapabilityId,
        authority: CredentialCapabilityRequest,
        resolver: Arc<dyn ScopedCredentialResolver>,
    ) -> Result<Self, ScopedCredentialRouteError> {
        validate_openai_authority(&authority)?;
        Ok(Self {
            capability_id,
            authority,
            resolver,
        })
    }

    pub(super) fn capability_id(&self) -> &CapabilityId {
        &self.capability_id
    }

    pub(super) fn authority(&self) -> &CredentialCapabilityRequest {
        &self.authority
    }

    pub(super) fn resolver(&self) -> &dyn ScopedCredentialResolver {
        self.resolver.as_ref()
    }
}

impl fmt::Debug for ScopedCredentialRoute {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ScopedCredentialRoute(<redacted>)")
    }
}

#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum ScopedCredentialRouteError {
    #[error("scoped credential route authority is invalid")]
    InvalidAuthority,
    #[error("scoped credential route authority is not supported")]
    UnsupportedAuthority,
    #[error("scoped credential route is already configured")]
    AlreadyConfigured,
    #[error("credential broker is disabled")]
    BrokerDisabled,
}

/// Stable denial returned before an outbound request can carry a credential.
#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum ScopedCredentialInjectionError {
    #[error("scoped credential scheme is denied")]
    SchemeDenied,
    #[error("scoped credential host is denied")]
    HostDenied,
    #[error("scoped credential port is denied")]
    PortDenied,
    #[error("scoped credential method is denied")]
    MethodDenied,
    #[error("scoped credential path is denied")]
    PathDenied,
    #[error("scoped credential reference is missing")]
    MissingReference,
    #[error("request authorization conflicts with scoped credential")]
    AuthorizationConflict,
    #[error("scoped credential reference has already been used")]
    AlreadyUsed,
    #[error("scoped credential resolution failed")]
    ResolutionFailed,
    #[error("credential request must be dispatched by the isolated broker")]
    IsolatedBrokerRequired,
}

/// Secret-free result returned by the isolated broker transport.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IsolatedCredentialReceipt {
    pub response_status: u16,
    pub uploaded_bytes: u64,
    pub downloaded_bytes: u64,
}

/// Exact request metadata admitted to the isolated broker.
pub struct IsolatedCredentialUse<'a> {
    pub scheme: &'a str,
    pub host: &'a str,
    pub port: u16,
    pub method: &'a str,
    pub path: &'a str,
    pub capability_id: &'a CapabilityId,
    pub authority: &'a CredentialCapabilityRequest,
}

#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum IsolatedCredentialDispatchError {
    #[error("isolated credential broker is unavailable")]
    Unavailable,
    #[error("isolated credential request was denied")]
    Denied,
    #[error("isolated credential request was cancelled")]
    Cancelled,
    #[error("isolated credential outcome is unknown")]
    OutcomeUnknown,
}

/// Client boundary for the separately constrained credential broker.
pub trait IsolatedCredentialDispatcher: Send + Sync + 'static {
    fn dispatch(
        &self,
        request: &IsolatedCredentialUse<'_>,
    ) -> Result<IsolatedCredentialReceipt, IsolatedCredentialDispatchError>;
}

/// Secret-free OpenAI route whose implementation performs the request in the
/// isolated broker rather than injecting a header into this process.
#[derive(Clone)]
pub struct IsolatedCredentialRoute {
    pub(super) capability_id: CapabilityId,
    pub(super) authority: CredentialCapabilityRequest,
    pub(super) dispatcher: Arc<dyn IsolatedCredentialDispatcher>,
}

impl IsolatedCredentialRoute {
    pub fn new(
        capability_id: CapabilityId,
        authority: CredentialCapabilityRequest,
        dispatcher: Arc<dyn IsolatedCredentialDispatcher>,
    ) -> Result<Self, ScopedCredentialRouteError> {
        validate_openai_authority(&authority)?;
        Ok(Self {
            capability_id,
            authority,
            dispatcher,
        })
    }
}

impl fmt::Debug for IsolatedCredentialRoute {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("IsolatedCredentialRoute(<redacted>)")
    }
}

fn validate_openai_authority(
    authority: &CredentialCapabilityRequest,
) -> Result<(), ScopedCredentialRouteError> {
    authority
        .validate()
        .map_err(|_| ScopedCredentialRouteError::InvalidAuthority)?;
    if authority.method != CredentialHttpMethod::Post
        || authority.destination.transport != CredentialTransport::Https
        || authority.destination.host.as_str() != OPENAI_API_HOST
        || authority.destination.port != OPENAI_API_PORT
        || !authority.path.as_str().starts_with(OPENAI_API_PATH_PREFIX)
    {
        return Err(ScopedCredentialRouteError::UnsupportedAuthority);
    }
    Ok(())
}
