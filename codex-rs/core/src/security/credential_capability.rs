use std::collections::HashMap;
use std::fmt;
use std::sync::RwLock;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use codex_security_policy::AuthorizationDecision;
use codex_security_policy::CapabilityId;
use codex_security_policy::CredentialCapabilityError;
use codex_security_policy::CredentialCapabilityRequest;
use codex_security_policy::RevocationState;
use codex_vault::ScopedCredentialError;
use codex_vault::VaultCredentialRef;
use rand::TryRngCore;
use rand::rngs::OsRng;
use sha2::Digest as _;
use sha2::Sha256;
use thiserror::Error;
use zeroize::Zeroize;

pub(crate) const MAX_CREDENTIAL_CAPABILITIES: usize = 1_024;
const CAPABILITY_TOKEN_BYTES: usize = 32;
const MAX_TOKEN_GENERATION_ATTEMPTS: usize = 4;
const CAPABILITY_ID_DOMAIN: &[u8] = b"corbanu-credential-capability-v1\0";

pub(crate) trait CredentialClock: Send + Sync {
    fn now_unix_seconds(&self) -> Result<i64, CredentialCapabilityStoreError>;
}

pub(crate) trait CredentialEntropy: Send + Sync {
    fn fill_token(
        &self,
        token: &mut [u8; CAPABILITY_TOKEN_BYTES],
    ) -> Result<(), CredentialCapabilityStoreError>;
}

#[derive(Clone, Copy, Default)]
pub(crate) struct SystemCredentialClock;

impl CredentialClock for SystemCredentialClock {
    fn now_unix_seconds(&self) -> Result<i64, CredentialCapabilityStoreError> {
        let seconds = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| CredentialCapabilityStoreError::ClockBeforeEpoch)?
            .as_secs();
        i64::try_from(seconds).map_err(|_| CredentialCapabilityStoreError::ClockOverflow)
    }
}

#[derive(Clone, Copy, Default)]
pub(crate) struct OsCredentialEntropy;

impl CredentialEntropy for OsCredentialEntropy {
    fn fill_token(
        &self,
        token: &mut [u8; CAPABILITY_TOKEN_BYTES],
    ) -> Result<(), CredentialCapabilityStoreError> {
        OsRng
            .try_fill_bytes(token)
            .map_err(|_| CredentialCapabilityStoreError::EntropyUnavailable)
    }
}

/// Opaque bearer material retained only inside trusted Core runtime objects.
///
/// It deliberately implements neither serialization nor cloning. Debug output is
/// always redacted and the bytes are cleared on drop.
struct CapabilityToken([u8; CAPABILITY_TOKEN_BYTES]);

impl fmt::Debug for CapabilityToken {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("CapabilityToken(<redacted>)")
    }
}

impl Drop for CapabilityToken {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

/// Trusted runtime handle. Model/tool protocol surfaces receive only the
/// CapabilityId digest, never this object.
pub(crate) struct IssuedCredentialCapability {
    capability_id: CapabilityId,
    token: CapabilityToken,
    decision: AuthorizationDecision,
}

impl fmt::Debug for IssuedCredentialCapability {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("IssuedCredentialCapability")
            .field("capability_id", &self.capability_id)
            .field("token", &self.token)
            .field("decision", &self.decision)
            .finish()
    }
}

impl IssuedCredentialCapability {
    pub(crate) fn capability_id(&self) -> &CapabilityId {
        &self.capability_id
    }

    pub(crate) fn decision(&self) -> &AuthorizationDecision {
        &self.decision
    }
}

/// Secret-free authority returned to the later trusted vault resolver.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct AuthorizedCredentialCapability {
    pub(crate) capability_id: CapabilityId,
    pub(crate) request: CredentialCapabilityRequest,
}

impl AuthorizedCredentialCapability {
    /// Cross the trusted Core-to-vault boundary after opaque bearer
    /// authorization has succeeded.
    pub(crate) fn into_vault_ref(self) -> Result<VaultCredentialRef, ScopedCredentialError> {
        VaultCredentialRef::from_authorized(self.capability_id, self.request)
    }
}

#[derive(Clone, Debug)]
struct StoredCapability {
    request: CredentialCapabilityRequest,
}

/// Hard-bounded concurrent lifecycle store for opaque credential capabilities.
///
/// The store never holds a credential value. It binds bearer entropy to one
/// complete secret-free request, revalidates time and revocation state, and
/// atomically removes valid authority before returning it for one use.
pub(crate) struct CredentialCapabilityStore<C = SystemCredentialClock, E = OsCredentialEntropy> {
    entries: RwLock<HashMap<CapabilityId, StoredCapability>>,
    capacity: usize,
    clock: C,
    entropy: E,
}

impl CredentialCapabilityStore {
    pub(crate) fn new(capacity: usize) -> Result<Self, CredentialCapabilityStoreError> {
        Self::with_sources(capacity, SystemCredentialClock, OsCredentialEntropy)
    }
}

impl<C, E> CredentialCapabilityStore<C, E>
where
    C: CredentialClock,
    E: CredentialEntropy,
{
    fn with_sources(
        capacity: usize,
        clock: C,
        entropy: E,
    ) -> Result<Self, CredentialCapabilityStoreError> {
        if capacity == 0 || capacity > MAX_CREDENTIAL_CAPABILITIES {
            return Err(CredentialCapabilityStoreError::InvalidCapacity {
                requested: capacity,
                maximum: MAX_CREDENTIAL_CAPABILITIES,
            });
        }
        Ok(Self {
            entries: RwLock::new(HashMap::with_capacity(capacity)),
            capacity,
            clock,
            entropy,
        })
    }

    pub(crate) fn issue(
        &self,
        request: CredentialCapabilityRequest,
        revocations: &RevocationState,
    ) -> Result<IssuedCredentialCapability, CredentialCapabilityStoreError> {
        let now = self.clock.now_unix_seconds()?;
        request.validate_at(now, revocations)?;
        let decision = request.decision()?;
        let request_digest = request.digest()?;
        let mut entries = self
            .entries
            .write()
            .map_err(|_| CredentialCapabilityStoreError::StorePoisoned)?;
        retain_valid(&mut entries, now, revocations);
        if entries.len() >= self.capacity {
            return Err(CredentialCapabilityStoreError::CapacityReached {
                capacity: self.capacity,
            });
        }

        for _ in 0..MAX_TOKEN_GENERATION_ATTEMPTS {
            let mut token_bytes = [0u8; CAPABILITY_TOKEN_BYTES];
            self.entropy.fill_token(&mut token_bytes)?;
            let token = CapabilityToken(token_bytes);
            let capability_id = derive_capability_id(&token, &request_digest)?;
            if entries.contains_key(&capability_id) {
                continue;
            }
            entries.insert(capability_id.clone(), StoredCapability { request });
            return Ok(IssuedCredentialCapability {
                capability_id,
                token,
                decision,
            });
        }
        Err(CredentialCapabilityStoreError::TokenCollision)
    }

    pub(crate) fn consume(
        &self,
        capability: &IssuedCredentialCapability,
        presented_request: &CredentialCapabilityRequest,
        revocations: &RevocationState,
    ) -> Result<AuthorizedCredentialCapability, CredentialCapabilityStoreError> {
        let now = self.clock.now_unix_seconds()?;
        presented_request.validate_at(now, revocations)?;
        let mut entries = self
            .entries
            .write()
            .map_err(|_| CredentialCapabilityStoreError::StorePoisoned)?;
        retain_valid(&mut entries, now, revocations);
        let stored = entries
            .get(capability.capability_id())
            .ok_or(CredentialCapabilityStoreError::UnknownCapability)?;
        let expected_id = derive_capability_id(&capability.token, &stored.request.digest()?)?;
        if &expected_id != capability.capability_id() {
            return Err(CredentialCapabilityStoreError::ForgedCapability);
        }
        if &stored.request != presented_request {
            return Err(CredentialCapabilityStoreError::AuthorityMismatch);
        }
        let request = stored.request.clone();
        entries.remove(capability.capability_id());
        Ok(AuthorizedCredentialCapability {
            capability_id: capability.capability_id.clone(),
            request,
        })
    }

    pub(crate) fn purge(
        &self,
        revocations: &RevocationState,
    ) -> Result<usize, CredentialCapabilityStoreError> {
        revocations.validate()?;
        let now = self.clock.now_unix_seconds()?;
        let mut entries = self
            .entries
            .write()
            .map_err(|_| CredentialCapabilityStoreError::StorePoisoned)?;
        let previous = entries.len();
        retain_valid(&mut entries, now, revocations);
        Ok(previous - entries.len())
    }

    pub(crate) fn len(&self) -> Result<usize, CredentialCapabilityStoreError> {
        self.entries
            .read()
            .map(|entries| entries.len())
            .map_err(|_| CredentialCapabilityStoreError::StorePoisoned)
    }
}

fn retain_valid(
    entries: &mut HashMap<CapabilityId, StoredCapability>,
    now_unix_seconds: i64,
    revocations: &RevocationState,
) {
    entries.retain(|_, stored| {
        stored
            .request
            .validate_at(now_unix_seconds, revocations)
            .is_ok()
    });
}

fn derive_capability_id(
    token: &CapabilityToken,
    request_digest: &str,
) -> Result<CapabilityId, CredentialCapabilityStoreError> {
    let mut digest = Sha256::new();
    digest.update(CAPABILITY_ID_DOMAIN);
    digest.update(token.0);
    digest.update(request_digest.as_bytes());
    let digest = digest.finalize();
    let mut encoded = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write as _;
        write!(&mut encoded, "{byte:02x}")
            .map_err(|_| CredentialCapabilityStoreError::DigestEncoding)?;
    }
    CapabilityId::from_sha256_hex(encoded).map_err(CredentialCapabilityStoreError::Policy)
}

#[derive(Debug, Error)]
pub(crate) enum CredentialCapabilityStoreError {
    #[error("credential capability capacity {requested} is invalid; maximum is {maximum}")]
    InvalidCapacity { requested: usize, maximum: usize },
    #[error("credential capability store reached its hard capacity of {capacity}")]
    CapacityReached { capacity: usize },
    #[error("credential capability store lock is poisoned")]
    StorePoisoned,
    #[error("system clock is before the Unix epoch")]
    ClockBeforeEpoch,
    #[error("system clock cannot be represented")]
    ClockOverflow,
    #[error("operating-system entropy is unavailable")]
    EntropyUnavailable,
    #[error("could not generate a unique credential capability")]
    TokenCollision,
    #[error("credential capability does not exist or is no longer valid")]
    UnknownCapability,
    #[error("credential capability bearer material is invalid")]
    ForgedCapability,
    #[error("credential capability authority does not match the request")]
    AuthorityMismatch,
    #[error("credential capability digest encoding failed")]
    DigestEncoding,
    #[error(transparent)]
    Policy(#[from] CredentialCapabilityError),
    #[error(transparent)]
    Revocation(#[from] codex_security_policy::RevocationError),
}

#[cfg(test)]
#[path = "credential_capability_tests.rs"]
mod tests;
