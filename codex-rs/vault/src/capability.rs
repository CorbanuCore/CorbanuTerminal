use std::collections::HashMap;
use std::fmt;
use std::panic::AssertUnwindSafe;
use std::panic::catch_unwind;
use std::sync::Arc;
use std::sync::RwLock;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use codex_secret_broker::BackendDispatchError;
use codex_secret_broker::CancellationFence;
use codex_secret_broker::CredentialReference as BrokerCredentialReference;
use codex_secret_broker::OpenAiResponsesOperation;
use codex_secret_broker::TypedCredentialBackend;
use codex_secret_broker::TypedOperationReceipt;
use codex_security_policy::CapabilityId;
use codex_security_policy::CredentialCapabilityError;
use codex_security_policy::CredentialCapabilityRequest;
use codex_security_policy::CredentialTransport;
use codex_security_policy::RevocationState;
use thiserror::Error;
use zeroize::Zeroizing;

use super::Vault;
use super::VaultError;
use crate::credential_panic::ScopedCredentialPanicGuard;

/// Secret-free reference emitted by the trusted Core capability store after it
/// has validated the opaque bearer and the complete requested authority.
///
/// This type deliberately implements neither serialization, cloning, nor
/// display. Its debug form is redacted. The public constructor is a
/// trusted-runtime boundary: callers must only invoke it after Core has
/// authorized the corresponding opaque capability.
pub struct VaultCredentialRef {
    capability_id: CapabilityId,
    request: CredentialCapabilityRequest,
}

impl VaultCredentialRef {
    /// Convert a Core-authorized capability into the only reference accepted by
    /// Vault::with_scoped_credential.
    ///
    /// This validates the secret-free request again. Runtime expiry and
    /// revocation are revalidated immediately before vault access.
    pub fn from_authorized(
        capability_id: CapabilityId,
        request: CredentialCapabilityRequest,
    ) -> Result<Self, ScopedCredentialError> {
        validate_binding(&request)?;
        request.validate().map_err(map_capability_error)?;
        Ok(Self {
            capability_id,
            request,
        })
    }

    /// Public, non-authorizing identifier safe for audit and receipt linkage.
    pub fn capability_id(&self) -> &CapabilityId {
        &self.capability_id
    }

    /// Approved vault label. This is metadata, never credential material.
    pub fn label(&self) -> &str {
        self.request.credential.label.as_str()
    }

    /// Approved use scope. This is metadata, never credential material.
    pub fn scope(&self) -> &str {
        self.request.credential.scope.as_str()
    }

    fn authorizes_openai_responses(&self, operation: &OpenAiResponsesOperation) -> bool {
        self.request.method.as_str() == operation.method()
            && self.request.destination.transport == CredentialTransport::Https
            && self.request.destination.host.as_str() == operation.host()
            && self.request.destination.port == operation.port()
            && self.request.path.as_str() == operation.path()
    }

    fn validate_at(
        &self,
        now_unix_seconds: i64,
        revocations: &RevocationState,
    ) -> Result<(), ScopedCredentialError> {
        validate_binding(&self.request)?;
        self.request
            .validate_at(now_unix_seconds, revocations)
            .map_err(map_capability_error)
    }
}

impl fmt::Debug for VaultCredentialRef {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("VaultCredentialRef(<redacted>)")
    }
}

/// Stable callback outcomes that cannot carry credential material.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScopedCredentialCallbackError {
    /// The trusted operation failed.
    Failed,
    /// The trusted operation was cancelled before completion.
    Cancelled,
}

/// Stable, secret-free failures from scoped credential resolution.
#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum ScopedCredentialError {
    #[error("credential capability is invalid")]
    InvalidCapability,
    #[error("credential capability label does not match its authority")]
    LabelMismatch,
    #[error("credential capability scope does not match its authority")]
    ScopeMismatch,
    #[error("credential capability is expired or not yet valid")]
    Expired,
    #[error("credential capability is revoked or stale")]
    Revoked,
    #[error("credential is unavailable")]
    NotFound,
    #[error("credential type is not eligible for scoped use")]
    CredentialTypeDenied,
    #[error("credential storage is unavailable")]
    Storage,
    #[error("scoped credential callback failed")]
    CallbackFailed,
    #[error("scoped credential callback was cancelled")]
    CallbackCancelled,
    #[error("scoped credential callback panicked")]
    CallbackPanicked,
}

impl Vault {
    /// Resolve a Core-authorized credential only for the duration of one
    /// trusted, synchronous callback.
    ///
    /// The callback cannot return a value and receives only a borrowed view.
    /// The backing allocation is wrapped in Zeroizing before the storage lock is
    /// released and is explicitly dropped before any outcome returns. Callback
    /// errors are reduced to stable variants, and panic payloads are discarded
    /// without formatting. Host panic hooks installed after first use must
    /// honor [`crate::scoped_credential_callback_active`] before logging.
    pub fn with_scoped_credential(
        &self,
        credential: &VaultCredentialRef,
        now_unix_seconds: i64,
        revocations: &RevocationState,
        callback: impl FnOnce(&str) -> Result<(), ScopedCredentialCallbackError>,
    ) -> Result<(), ScopedCredentialError> {
        credential.validate_at(now_unix_seconds, revocations)?;
        let secret = self.read_scoped_secret(credential.label())?;
        let _panic_guard = ScopedCredentialPanicGuard::enter();
        let outcome = catch_unwind(AssertUnwindSafe(|| callback(secret.as_str())));
        drop(secret);

        match outcome {
            Ok(Ok(())) => Ok(()),
            Ok(Err(ScopedCredentialCallbackError::Failed)) => {
                Err(ScopedCredentialError::CallbackFailed)
            }
            Ok(Err(ScopedCredentialCallbackError::Cancelled)) => {
                Err(ScopedCredentialError::CallbackCancelled)
            }
            Err(_) => Err(ScopedCredentialError::CallbackPanicked),
        }
    }

    fn read_scoped_secret(&self, label: &str) -> Result<Zeroizing<String>, ScopedCredentialError> {
        let normalized =
            super::normalize_label(label).map_err(|_| ScopedCredentialError::InvalidCapability)?;
        if normalized == crate::MANAGED_CLAUDE_TOKEN_LABEL {
            return Err(ScopedCredentialError::CredentialTypeDenied);
        }
        self.with_storage_lock(|| {
            let index = self.load_index()?;
            let metadata =
                index
                    .credentials
                    .get(&normalized)
                    .ok_or_else(|| VaultError::NotFound {
                        label: normalized.clone(),
                    })?;
            if !metadata.credential_type.permits_programmatic_use() {
                return Err(VaultError::ProgrammaticUseDenied {
                    label: normalized.clone(),
                    credential_type: metadata.credential_type,
                });
            }
            self.read_secret(&normalized)?
                .map(Zeroizing::new)
                .ok_or_else(|| VaultError::NotFound {
                    label: normalized.clone(),
                })
        })
        .map_err(map_vault_error)
    }
}

fn validate_binding(request: &CredentialCapabilityRequest) -> Result<(), ScopedCredentialError> {
    if request.authorization.resource.id != request.credential.label {
        return Err(ScopedCredentialError::LabelMismatch);
    }
    if request.authorization.context.operation != request.credential.scope {
        return Err(ScopedCredentialError::ScopeMismatch);
    }
    Ok(())
}

fn map_vault_error(error: VaultError) -> ScopedCredentialError {
    match error {
        VaultError::NotFound { .. } => ScopedCredentialError::NotFound,
        VaultError::ProviderManagedCredential { .. }
        | VaultError::ProgrammaticUseDenied { .. }
        | VaultError::ProgrammaticUseSecurityLevelDenied { .. } => {
            ScopedCredentialError::CredentialTypeDenied
        }
        VaultError::Storage(_) => ScopedCredentialError::Storage,
        VaultError::CredentialExists { .. }
        | VaultError::InvalidLabel(_)
        | VaultError::EmptySecret => ScopedCredentialError::InvalidCapability,
    }
}

fn map_capability_error(error: CredentialCapabilityError) -> ScopedCredentialError {
    match error {
        CredentialCapabilityError::CredentialAuthorityMismatch => {
            ScopedCredentialError::LabelMismatch
        }
        CredentialCapabilityError::CredentialScopeMismatch => ScopedCredentialError::ScopeMismatch,
        CredentialCapabilityError::ExpiredOrNotYetValid => ScopedCredentialError::Expired,
        CredentialCapabilityError::StaleRevocationGeneration
        | CredentialCapabilityError::Revoked
        | CredentialCapabilityError::Revocation(_) => ScopedCredentialError::Revoked,
        CredentialCapabilityError::UnsupportedSchemaVersion { .. }
        | CredentialCapabilityError::NegativeTimestamp
        | CredentialCapabilityError::InvalidExpiry
        | CredentialCapabilityError::AuthorizationTimeMismatch
        | CredentialCapabilityError::AgentActorRequired
        | CredentialCapabilityError::DestinationMismatch
        | CredentialCapabilityError::GrantMismatch
        | CredentialCapabilityError::InvalidDestinationHost
        | CredentialCapabilityError::NonCanonicalDestinationHost
        | CredentialCapabilityError::InvalidDestinationPort
        | CredentialCapabilityError::InvalidOriginPath
        | CredentialCapabilityError::InvalidIdentifier(_)
        | CredentialCapabilityError::InvalidCapabilityId
        | CredentialCapabilityError::Authorization(_)
        | CredentialCapabilityError::Grant(_)
        | CredentialCapabilityError::Mandate(_)
        | CredentialCapabilityError::BoundedText(_)
        | CredentialCapabilityError::Serialization(_) => ScopedCredentialError::InvalidCapability,
    }
}

const MAX_BROKER_CREDENTIALS: usize = 64;

/// In-broker HTTP adapter. The raw value is borrowed only for this call and
/// must never be returned, logged, or retained by the implementation.
pub trait VaultBrokerTransport: Send + Sync + 'static {
    fn execute_openai_responses(
        &self,
        raw_credential: &str,
        operation: &OpenAiResponsesOperation,
        cancellation: &CancellationFence,
    ) -> Result<TypedOperationReceipt, BackendDispatchError>;
}

pub trait VaultBrokerClock: Send + Sync + 'static {
    fn now_unix_seconds(&self) -> Result<i64, BackendDispatchError>;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SystemVaultBrokerClock;

impl VaultBrokerClock for SystemVaultBrokerClock {
    fn now_unix_seconds(&self) -> Result<i64, BackendDispatchError> {
        let seconds = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| BackendDispatchError::Failed)?
            .as_secs();
        i64::try_from(seconds).map_err(|_| BackendDispatchError::Failed)
    }
}

/// A trusted-backend adapter intended to be linked only into the constrained
/// broker service. Its map is keyed by opaque digest and cannot be enumerated
/// through the broker protocol.
pub struct VaultBrokerBackend<T, C = SystemVaultBrokerClock> {
    vault: Arc<Vault>,
    credentials: HashMap<BrokerCredentialReference, VaultCredentialRef>,
    revocations: Arc<RwLock<RevocationState>>,
    transport: T,
    clock: C,
}

impl<T> VaultBrokerBackend<T, SystemVaultBrokerClock>
where
    T: VaultBrokerTransport,
{
    pub fn new(
        vault: Arc<Vault>,
        credentials: Vec<(BrokerCredentialReference, VaultCredentialRef)>,
        revocations: Arc<RwLock<RevocationState>>,
        transport: T,
    ) -> Result<Self, VaultBrokerBackendError> {
        Self::with_clock(
            vault,
            credentials,
            revocations,
            transport,
            SystemVaultBrokerClock,
        )
    }
}

impl<T, C> VaultBrokerBackend<T, C>
where
    T: VaultBrokerTransport,
    C: VaultBrokerClock,
{
    pub fn with_clock(
        vault: Arc<Vault>,
        credentials: Vec<(BrokerCredentialReference, VaultCredentialRef)>,
        revocations: Arc<RwLock<RevocationState>>,
        transport: T,
        clock: C,
    ) -> Result<Self, VaultBrokerBackendError> {
        if credentials.is_empty() || credentials.len() > MAX_BROKER_CREDENTIALS {
            return Err(VaultBrokerBackendError::InvalidCredentials);
        }
        for (reference, credential) in &credentials {
            if reference.as_str() != credential.capability_id().as_str() {
                return Err(VaultBrokerBackendError::InvalidCredentials);
            }
        }
        let expected_len = credentials.len();
        let credentials = credentials.into_iter().collect::<HashMap<_, _>>();
        if credentials.len() != expected_len {
            return Err(VaultBrokerBackendError::InvalidCredentials);
        }
        Ok(Self {
            vault,
            credentials,
            revocations,
            transport,
            clock,
        })
    }
}

impl<T, C> TypedCredentialBackend for VaultBrokerBackend<T, C>
where
    T: VaultBrokerTransport,
    C: VaultBrokerClock,
{
    fn execute_openai_responses(
        &self,
        reference: &BrokerCredentialReference,
        operation: &OpenAiResponsesOperation,
        cancellation: &CancellationFence,
    ) -> Result<TypedOperationReceipt, BackendDispatchError> {
        cancellation.ensure_active()?;
        let credential = self
            .credentials
            .get(reference)
            .ok_or(BackendDispatchError::Failed)?;
        if !credential.authorizes_openai_responses(operation) {
            return Err(BackendDispatchError::Failed);
        }
        let now = self.clock.now_unix_seconds()?;
        let revocations = self
            .revocations
            .read()
            .map_err(|_| BackendDispatchError::Failed)?;
        let mut transport_result = None;
        let vault_result =
            self.vault
                .with_scoped_credential(credential, now, &revocations, |raw_credential| {
                    let result = self.transport.execute_openai_responses(
                        raw_credential,
                        operation,
                        cancellation,
                    );
                    let callback_result = match &result {
                        Ok(_) => Ok(()),
                        Err(BackendDispatchError::Cancelled) => {
                            Err(ScopedCredentialCallbackError::Cancelled)
                        }
                        Err(
                            BackendDispatchError::Failed | BackendDispatchError::OutcomeUnknown,
                        ) => Err(ScopedCredentialCallbackError::Failed),
                    };
                    transport_result = Some(result);
                    callback_result
                });
        if let Some(result) = transport_result {
            return result;
        }
        match vault_result {
            Err(ScopedCredentialError::CallbackCancelled) => Err(BackendDispatchError::Cancelled),
            Ok(())
            | Err(
                ScopedCredentialError::InvalidCapability
                | ScopedCredentialError::LabelMismatch
                | ScopedCredentialError::ScopeMismatch
                | ScopedCredentialError::Expired
                | ScopedCredentialError::Revoked
                | ScopedCredentialError::NotFound
                | ScopedCredentialError::CredentialTypeDenied
                | ScopedCredentialError::Storage
                | ScopedCredentialError::CallbackFailed
                | ScopedCredentialError::CallbackPanicked,
            ) => Err(BackendDispatchError::Failed),
        }
    }
}

#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum VaultBrokerBackendError {
    #[error("broker credential grants are invalid")]
    InvalidCredentials,
}

#[cfg(test)]
#[path = "capability_tests.rs"]
mod tests;
