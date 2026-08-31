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
const RESERVATION_ID_DOMAIN: &[u8] = b"corbanu-credential-reservation-v1\0";
const MAX_USAGE_RESERVATIONS_PER_CAPABILITY: usize = 1_024;

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

/// Opaque reservation bearer transferred only over the later authenticated
/// Core-to-broker IPC channel. It has no public/model serialization surface.
struct ReservationToken([u8; CAPABILITY_TOKEN_BYTES]);

impl fmt::Debug for ReservationToken {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ReservationToken(<redacted>)")
    }
}

impl Drop for ReservationToken {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct CredentialUsage {
    pub(crate) requests: u64,
    pub(crate) tokens: u64,
    pub(crate) bytes: u64,
    pub(crate) spend_microunits: u64,
}

impl CredentialUsage {
    pub(crate) fn new(requests: u64, tokens: u64, bytes: u64, spend_microunits: u64) -> Self {
        Self {
            requests,
            tokens,
            bytes,
            spend_microunits,
        }
    }

    fn checked_add(self, other: Self) -> Result<Self, CredentialCapabilityStoreError> {
        Ok(Self {
            requests: self
                .requests
                .checked_add(other.requests)
                .ok_or(CredentialCapabilityStoreError::UsageOverflow)?,
            tokens: self
                .tokens
                .checked_add(other.tokens)
                .ok_or(CredentialCapabilityStoreError::UsageOverflow)?,
            bytes: self
                .bytes
                .checked_add(other.bytes)
                .ok_or(CredentialCapabilityStoreError::UsageOverflow)?,
            spend_microunits: self
                .spend_microunits
                .checked_add(other.spend_microunits)
                .ok_or(CredentialCapabilityStoreError::UsageOverflow)?,
        })
    }

    fn checked_sub(self, other: Self) -> Result<Self, CredentialCapabilityStoreError> {
        Ok(Self {
            requests: self
                .requests
                .checked_sub(other.requests)
                .ok_or(CredentialCapabilityStoreError::UsageInvariant)?,
            tokens: self
                .tokens
                .checked_sub(other.tokens)
                .ok_or(CredentialCapabilityStoreError::UsageInvariant)?,
            bytes: self
                .bytes
                .checked_sub(other.bytes)
                .ok_or(CredentialCapabilityStoreError::UsageInvariant)?,
            spend_microunits: self
                .spend_microunits
                .checked_sub(other.spend_microunits)
                .ok_or(CredentialCapabilityStoreError::UsageInvariant)?,
        })
    }

    fn fits_within(self, limit: Self) -> bool {
        self.requests <= limit.requests
            && self.tokens <= limit.tokens
            && self.bytes <= limit.bytes
            && self.spend_microunits <= limit.spend_microunits
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CredentialUsageOutcome {
    Completed,
    Cancelled,
    Partial,
    Unknown,
}

/// Metering facts may only be constructed by trusted runtime/broker adapters.
/// The type deliberately has no serde implementation or arbitrary fields.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct TrustedCredentialMetering {
    outcome: CredentialUsageOutcome,
    measured: Option<CredentialUsage>,
}

impl TrustedCredentialMetering {
    pub(crate) fn completed(measured: CredentialUsage) -> Self {
        Self {
            outcome: CredentialUsageOutcome::Completed,
            measured: Some(measured),
        }
    }

    pub(crate) fn partial(measured: CredentialUsage) -> Self {
        Self {
            outcome: CredentialUsageOutcome::Partial,
            measured: Some(measured),
        }
    }

    pub(crate) fn cancelled() -> Self {
        Self {
            outcome: CredentialUsageOutcome::Cancelled,
            measured: None,
        }
    }

    pub(crate) fn unknown() -> Self {
        Self {
            outcome: CredentialUsageOutcome::Unknown,
            measured: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CredentialUsageSettlement {
    pub(crate) reservation_id: CapabilityId,
    pub(crate) outcome: CredentialUsageOutcome,
    pub(crate) reserved: CredentialUsage,
    pub(crate) charged: CredentialUsage,
}

/// Authenticated opaque handoff for one worst-case reservation. The bearer is
/// never serialized publicly and its debug representation is redacted.
pub(crate) struct IssuedCredentialReservation {
    capability_id: CapabilityId,
    reservation_id: CapabilityId,
    token: ReservationToken,
    request: CredentialCapabilityRequest,
    reserved: CredentialUsage,
}

impl fmt::Debug for IssuedCredentialReservation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("IssuedCredentialReservation")
            .field("capability_id", &self.capability_id)
            .field("reservation_id", &self.reservation_id)
            .field("token", &self.token)
            .field("reserved", &self.reserved)
            .finish()
    }
}

impl IssuedCredentialReservation {
    pub(crate) fn reservation_id(&self) -> &CapabilityId {
        &self.reservation_id
    }

    pub(crate) fn reserved(&self) -> CredentialUsage {
        self.reserved
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
    committed_usage: CredentialUsage,
    pending_usage: CredentialUsage,
    reservations: HashMap<CapabilityId, StoredReservation>,
}

#[derive(Clone, Debug)]
struct StoredReservation {
    reserved: CredentialUsage,
    settlement: Option<CredentialUsageSettlement>,
    dispatch_authorized: bool,
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
        retain_valid(&mut entries, now, revocations)?;
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
            entries.insert(
                capability_id.clone(),
                StoredCapability {
                    request,
                    committed_usage: CredentialUsage::default(),
                    pending_usage: CredentialUsage::default(),
                    reservations: HashMap::new(),
                },
            );
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
        retain_valid(&mut entries, now, revocations)?;
        let stored = entries
            .get(capability.capability_id())
            .ok_or(CredentialCapabilityStoreError::UnknownCapability)?;
        authenticate_capability(capability, stored)?;
        if &stored.request != presented_request {
            return Err(CredentialCapabilityStoreError::AuthorityMismatch);
        }
        if stored.request.has_usage_limits() {
            return Err(CredentialCapabilityStoreError::UsageReservationRequired);
        }
        if stored
            .reservations
            .values()
            .any(|reservation| reservation.settlement.is_none())
        {
            return Err(CredentialCapabilityStoreError::ActiveReservations);
        }
        let request = stored.request.clone();
        entries.remove(capability.capability_id());
        Ok(AuthorizedCredentialCapability {
            capability_id: capability.capability_id.clone(),
            request,
        })
    }

    pub(crate) fn reserve(
        &self,
        capability: &IssuedCredentialCapability,
        presented_request: &CredentialCapabilityRequest,
        worst_case: CredentialUsage,
        revocations: &RevocationState,
    ) -> Result<IssuedCredentialReservation, CredentialCapabilityStoreError> {
        let now = self.clock.now_unix_seconds()?;
        presented_request.validate_at(now, revocations)?;
        if worst_case.requests != 1 {
            return Err(CredentialCapabilityStoreError::InvalidReservationRequestCount);
        }
        let mut entries = self
            .entries
            .write()
            .map_err(|_| CredentialCapabilityStoreError::StorePoisoned)?;
        retain_valid(&mut entries, now, revocations)?;
        let stored = entries
            .get_mut(capability.capability_id())
            .ok_or(CredentialCapabilityStoreError::UnknownCapability)?;
        authenticate_capability(capability, stored)?;
        if &stored.request != presented_request {
            return Err(CredentialCapabilityStoreError::AuthorityMismatch);
        }
        let (per_request, aggregate) = usage_limits(&stored.request)?;
        if !worst_case.fits_within(per_request) {
            return Err(CredentialCapabilityStoreError::PerRequestUsageExceeded);
        }
        let next_usage = stored
            .committed_usage
            .checked_add(stored.pending_usage)?
            .checked_add(worst_case)?;
        if !next_usage.fits_within(aggregate) {
            return Err(CredentialCapabilityStoreError::AggregateUsageExceeded);
        }
        if stored
            .reservations
            .values()
            .filter(|reservation| reservation.settlement.is_none())
            .count()
            >= MAX_USAGE_RESERVATIONS_PER_CAPABILITY
        {
            return Err(CredentialCapabilityStoreError::ReservationCapacityReached);
        }

        let request_digest = stored.request.digest()?;
        for _ in 0..MAX_TOKEN_GENERATION_ATTEMPTS {
            let mut token_bytes = [0u8; CAPABILITY_TOKEN_BYTES];
            self.entropy.fill_token(&mut token_bytes)?;
            let token = ReservationToken(token_bytes);
            let reservation_id = derive_reservation_id(
                &token,
                capability.capability_id(),
                &request_digest,
                worst_case,
            )?;
            if stored.reservations.contains_key(&reservation_id) {
                continue;
            }
            stored.pending_usage = stored.pending_usage.checked_add(worst_case)?;
            stored.reservations.insert(
                reservation_id.clone(),
                StoredReservation {
                    reserved: worst_case,
                    settlement: None,
                    dispatch_authorized: false,
                },
            );
            return Ok(IssuedCredentialReservation {
                capability_id: capability.capability_id().clone(),
                reservation_id,
                token,
                request: stored.request.clone(),
                reserved: worst_case,
            });
        }
        Err(CredentialCapabilityStoreError::TokenCollision)
    }

    /// Authorize exactly one trusted broker dispatch for an active reservation.
    /// The returned reference remains secret-free; vault resolution still occurs
    /// only inside the later broker boundary.
    pub(crate) fn authorize_reservation_dispatch(
        &self,
        reservation: &IssuedCredentialReservation,
        revocations: &RevocationState,
    ) -> Result<VaultCredentialRef, CredentialCapabilityStoreError> {
        let now = self.clock.now_unix_seconds()?;
        reservation.request.validate_at(now, revocations)?;
        let mut entries = self
            .entries
            .write()
            .map_err(|_| CredentialCapabilityStoreError::StorePoisoned)?;
        retain_valid(&mut entries, now, revocations)?;
        let stored = entries
            .get_mut(&reservation.capability_id)
            .ok_or(CredentialCapabilityStoreError::UnknownCapability)?;
        if stored.request != reservation.request {
            return Err(CredentialCapabilityStoreError::AuthorityMismatch);
        }
        let request_digest = stored.request.digest()?;
        let stored_reservation = stored
            .reservations
            .get_mut(&reservation.reservation_id)
            .ok_or(CredentialCapabilityStoreError::UnknownReservation)?;
        authenticate_reservation(reservation, stored_reservation, &request_digest)?;
        if stored_reservation.settlement.is_some() {
            return Err(CredentialCapabilityStoreError::ReservationSettled);
        }
        if stored_reservation.dispatch_authorized {
            return Err(CredentialCapabilityStoreError::ReservationAlreadyAuthorized);
        }
        stored_reservation.dispatch_authorized = true;
        VaultCredentialRef::from_authorized(
            reservation.capability_id.clone(),
            stored.request.clone(),
        )
        .map_err(CredentialCapabilityStoreError::VaultReference)
    }

    pub(crate) fn settle(
        &self,
        reservation: &IssuedCredentialReservation,
        metering: TrustedCredentialMetering,
        revocations: &RevocationState,
    ) -> Result<CredentialUsageSettlement, CredentialCapabilityStoreError> {
        revocations.validate()?;
        let now = self.clock.now_unix_seconds()?;
        let mut entries = self
            .entries
            .write()
            .map_err(|_| CredentialCapabilityStoreError::StorePoisoned)?;
        retain_valid(&mut entries, now, revocations)?;
        let stored = entries
            .get_mut(&reservation.capability_id)
            .ok_or(CredentialCapabilityStoreError::UnknownCapability)?;
        let request_digest = stored.request.digest()?;
        let stored_reservation = stored
            .reservations
            .get_mut(&reservation.reservation_id)
            .ok_or(CredentialCapabilityStoreError::UnknownReservation)?;
        authenticate_reservation(reservation, stored_reservation, &request_digest)?;
        if let Some(settlement) = &stored_reservation.settlement {
            return Ok(settlement.clone());
        }

        let charged = match (metering.outcome, metering.measured) {
            (CredentialUsageOutcome::Cancelled, None) if stored_reservation.dispatch_authorized => {
                stored_reservation.reserved
            }
            (CredentialUsageOutcome::Cancelled, None) => CredentialUsage::new(1, 0, 0, 0),
            (CredentialUsageOutcome::Unknown, None) => stored_reservation.reserved,
            (
                CredentialUsageOutcome::Completed | CredentialUsageOutcome::Partial,
                Some(measured),
            ) if measured.requests == 1 && measured.fits_within(stored_reservation.reserved) => {
                measured
            }
            (CredentialUsageOutcome::Completed | CredentialUsageOutcome::Partial, Some(_)) => {
                return Err(CredentialCapabilityStoreError::MeteringExceedsReservation);
            }
            _ => return Err(CredentialCapabilityStoreError::InvalidMeteringOutcome),
        };
        stored.pending_usage = stored
            .pending_usage
            .checked_sub(stored_reservation.reserved)?;
        stored.committed_usage = stored.committed_usage.checked_add(charged)?;
        let settlement = CredentialUsageSettlement {
            reservation_id: reservation.reservation_id.clone(),
            outcome: metering.outcome,
            reserved: stored_reservation.reserved,
            charged,
        };
        stored_reservation.settlement = Some(settlement.clone());
        Ok(settlement)
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
        retain_valid(&mut entries, now, revocations)?;
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
) -> Result<(), CredentialCapabilityStoreError> {
    for stored in entries.values_mut() {
        if now_unix_seconds >= stored.request.expires_at_unix_seconds {
            force_unknown_at_expiry(stored)?;
        }
    }
    entries.retain(|_, stored| {
        stored
            .request
            .validate_at(now_unix_seconds, revocations)
            .is_ok()
            || stored
                .reservations
                .values()
                .any(|reservation| reservation.settlement.is_none())
    });
    Ok(())
}

fn force_unknown_at_expiry(
    stored: &mut StoredCapability,
) -> Result<(), CredentialCapabilityStoreError> {
    if stored.pending_usage == CredentialUsage::default() {
        return Ok(());
    }
    let committed_usage = stored.committed_usage.checked_add(stored.pending_usage)?;
    for (reservation_id, reservation) in &mut stored.reservations {
        if reservation.settlement.is_none() {
            reservation.settlement = Some(CredentialUsageSettlement {
                reservation_id: reservation_id.clone(),
                outcome: CredentialUsageOutcome::Unknown,
                reserved: reservation.reserved,
                charged: reservation.reserved,
            });
        }
    }
    stored.committed_usage = committed_usage;
    stored.pending_usage = CredentialUsage::default();
    Ok(())
}

fn authenticate_capability(
    capability: &IssuedCredentialCapability,
    stored: &StoredCapability,
) -> Result<(), CredentialCapabilityStoreError> {
    let expected_id = derive_capability_id(&capability.token, &stored.request.digest()?)?;
    if &expected_id != capability.capability_id() {
        return Err(CredentialCapabilityStoreError::ForgedCapability);
    }
    Ok(())
}

fn authenticate_reservation(
    reservation: &IssuedCredentialReservation,
    stored: &StoredReservation,
    request_digest: &str,
) -> Result<(), CredentialCapabilityStoreError> {
    let expected_id = derive_reservation_id(
        &reservation.token,
        &reservation.capability_id,
        request_digest,
        stored.reserved,
    )?;
    if expected_id != reservation.reservation_id {
        return Err(CredentialCapabilityStoreError::ForgedReservation);
    }
    Ok(())
}

fn usage_limits(
    request: &CredentialCapabilityRequest,
) -> Result<(CredentialUsage, CredentialUsage), CredentialCapabilityStoreError> {
    request.validate()?;
    if !request.has_usage_limits() {
        return Err(CredentialCapabilityStoreError::MissingUsageLimits);
    }
    let read = |aggregate: bool, dimension: &str| {
        if aggregate {
            request.aggregate_usage_limit(dimension)
        } else {
            request.per_request_usage_limit(dimension)
        }
        .ok_or(CredentialCapabilityStoreError::MissingUsageLimits)
    };
    Ok((
        CredentialUsage::new(
            read(false, "requests")?,
            read(false, "tokens")?,
            read(false, "bytes")?,
            read(false, "spend_microunits")?,
        ),
        CredentialUsage::new(
            read(true, "requests")?,
            read(true, "tokens")?,
            read(true, "bytes")?,
            read(true, "spend_microunits")?,
        ),
    ))
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

fn derive_reservation_id(
    token: &ReservationToken,
    capability_id: &CapabilityId,
    request_digest: &str,
    usage: CredentialUsage,
) -> Result<CapabilityId, CredentialCapabilityStoreError> {
    let mut digest = Sha256::new();
    digest.update(RESERVATION_ID_DOMAIN);
    digest.update(token.0);
    digest.update(capability_id.as_str().as_bytes());
    digest.update(request_digest.as_bytes());
    for value in [
        usage.requests,
        usage.tokens,
        usage.bytes,
        usage.spend_microunits,
    ] {
        digest.update(value.to_le_bytes());
    }
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
    #[error("credential usage arithmetic overflowed")]
    UsageOverflow,
    #[error("credential usage ledger invariant was violated")]
    UsageInvariant,
    #[error("metered credential capability requires a usage reservation")]
    UsageReservationRequired,
    #[error("credential capability has active usage reservations")]
    ActiveReservations,
    #[error("credential usage reservation must account for exactly one request")]
    InvalidReservationRequestCount,
    #[error("credential usage reservation exceeds a per-request limit")]
    PerRequestUsageExceeded,
    #[error("credential usage reservation exceeds an aggregate limit")]
    AggregateUsageExceeded,
    #[error("credential usage reservation store reached its hard capacity")]
    ReservationCapacityReached,
    #[error("credential capability does not contain a complete usage policy")]
    MissingUsageLimits,
    #[error("credential usage reservation does not exist")]
    UnknownReservation,
    #[error("credential usage reservation bearer material is invalid")]
    ForgedReservation,
    #[error("credential usage reservation has already authorized a dispatch")]
    ReservationAlreadyAuthorized,
    #[error("credential usage reservation has already been settled")]
    ReservationSettled,
    #[error("credential metering exceeds its authenticated reservation")]
    MeteringExceedsReservation,
    #[error("credential metering outcome is internally inconsistent")]
    InvalidMeteringOutcome,
    #[error("credential vault reference is invalid: {0}")]
    VaultReference(ScopedCredentialError),
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
