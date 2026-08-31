//! Fail-closed composition seam for protected runtime prerequisites.
//!
//! This module does not activate protected mode or own an ingress, egress, or
//! broker adapter. It binds the completed policy, authoritative-state,
//! revocation-fence, and durable-event contracts so later adapters cannot use a
//! stale measurement or an unregistered route as authority.
#![cfg_attr(not(test), allow(dead_code))]

use std::collections::BTreeSet;
use std::sync::Arc;
use std::sync::RwLock;
use std::sync::atomic::AtomicI64;
use std::sync::atomic::Ordering;

use codex_config::AuthoritativeSecurityState;
use codex_security_audit::AppendAcknowledgement;
use codex_security_audit::AuthorityIdentity;
use codex_security_audit::DispatchPermit;
use codex_security_audit::DispatchResolution;
use codex_security_audit::EventContext;
use codex_security_audit::JournalError;
use codex_security_audit::RecoveryReport;
use codex_security_audit::RecoveryState;
use codex_security_audit::ReferenceJournal;
use codex_security_audit::SecurityEventError;
use codex_security_policy::ActorChain;
use codex_security_policy::AuthorizationRequest;
use codex_security_policy::BoundedGrant;
use codex_security_policy::BoundedText;
use codex_security_policy::DispatchFence;
use codex_security_policy::DispatchPhase;
use codex_security_policy::MandateOutcome;
use codex_security_policy::PolicyPrincipal;
use codex_security_policy::PrincipalKind;
use codex_security_policy::ProtectedActionMandate;
use codex_security_policy::ProtectedActionPreview;
use codex_security_policy::ProtectedDispatchStep;
use codex_security_policy::RevocationError;
use codex_security_policy::RevocationState;
use codex_security_policy::SecurityLevel;
use sha2::Digest as _;
use sha2::Sha256;
use thiserror::Error;
use uuid::Uuid;

use super::effective_policy::EffectivePolicySnapshot;

pub(crate) const PROTECTED_RUNTIME_CONTRACT_VERSION: u32 = 4;
const MAX_READINESS_WINDOW_SECONDS: i64 = 300;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ProtectionReadinessStatus {
    Ready,
    Unavailable,
    Unsupported,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct RuntimeGenerationBinding {
    pub(crate) owner: u64,
    pub(crate) policy: u64,
    pub(crate) run: u64,
    pub(crate) revocation: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ReadinessWindow {
    pub(crate) measured_at_unix_seconds: i64,
    pub(crate) expires_at_unix_seconds: i64,
}

/// Controller-supplied measurement of one concrete protected backend.
///
/// The IDs are policy identities, never credentials, paths, environment
/// values, or debug payloads. Construction alone does not authenticate the
/// measurement; [`ProtectedRuntime::compose`] binds it to authenticated PF-20
/// state and the live PF-22 policy snapshot.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct MeasuredProtectionReadiness {
    backend_id: BoundedText,
    identity_id: BoundedText,
    producer: PolicyPrincipal,
    state_owner: codex_config::AuthoritativeStateOwner,
    status: ProtectionReadinessStatus,
    generations: RuntimeGenerationBinding,
    window: ReadinessWindow,
}

impl MeasuredProtectionReadiness {
    pub(crate) fn new(
        backend_id: impl Into<String>,
        identity_id: impl Into<String>,
        producer: PolicyPrincipal,
        state_owner: codex_config::AuthoritativeStateOwner,
        status: ProtectionReadinessStatus,
        generations: RuntimeGenerationBinding,
        window: ReadinessWindow,
    ) -> Result<Self, ProtectedRuntimeError> {
        if producer.kind != PrincipalKind::Service {
            return Err(ProtectedRuntimeError::InvalidReadinessProducer);
        }
        let measurement = Self {
            backend_id: BoundedText::new(backend_id.into())?,
            identity_id: BoundedText::new(identity_id.into())?,
            producer,
            state_owner,
            status,
            generations,
            window,
        };
        measurement.validate_shape()?;
        Ok(measurement)
    }

    fn validate_shape(&self) -> Result<(), ProtectedRuntimeError> {
        if self.generations.owner == 0
            || self.generations.policy == 0
            || self.generations.run == 0
            || self.window.measured_at_unix_seconds < 0
            || self.window.expires_at_unix_seconds <= self.window.measured_at_unix_seconds
            || self.window.expires_at_unix_seconds - self.window.measured_at_unix_seconds
                > MAX_READINESS_WINDOW_SECONDS
        {
            return Err(ProtectedRuntimeError::InvalidReadinessMeasurement);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum ProtectedRouteKind {
    Ingress,
    Egress,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct ProtectedRoute {
    kind: ProtectedRouteKind,
    id: BoundedText,
}

/// Closed-world inventory populated only by hook-owning adapter sprints.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct ProtectedRouteRegistry {
    routes: BTreeSet<ProtectedRoute>,
}

impl ProtectedRouteRegistry {
    pub(crate) fn new(
        routes: impl IntoIterator<Item = (ProtectedRouteKind, String)>,
    ) -> Result<Self, ProtectedRuntimeError> {
        let mut registered = BTreeSet::new();
        for (kind, id) in routes {
            let route = ProtectedRoute {
                kind,
                id: BoundedText::new(id)?,
            };
            if !registered.insert(route) {
                return Err(ProtectedRuntimeError::DuplicateRoute);
            }
        }
        Ok(Self { routes: registered })
    }

    fn contains(&self, kind: ProtectedRouteKind, id: &BoundedText) -> bool {
        self.routes.contains(&ProtectedRoute {
            kind,
            id: id.clone(),
        })
    }
}

#[derive(Clone, Copy)]
pub(crate) struct CurrentProtectedRuntime<'a> {
    pub(crate) effective: &'a EffectivePolicySnapshot,
    pub(crate) authoritative: &'a AuthoritativeSecurityState,
    pub(crate) readiness: &'a MeasuredProtectionReadiness,
    pub(crate) recovery: &'a RecoveryReport,
    /// Expected live run generation supplied by the hook-owning adapter.
    ///
    /// PF-22 cross-checks it against measured readiness, but cannot authenticate
    /// it independently until the adapter binds the session source and the same
    /// value passed to `ReferenceJournal::recover`.
    pub(crate) run_generation: u64,
    pub(crate) now_unix_seconds: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ProtectedRuntimeSnapshot {
    pub(crate) contract_version: u32,
    pub(crate) configured_level: SecurityLevel,
    pub(crate) creator_required_level: SecurityLevel,
    pub(crate) effective_level: SecurityLevel,
    pub(crate) effective_policy_epoch: u64,
    pub(crate) actor_chain: ActorChain,
    pub(crate) session_id: BoundedText,
    pub(crate) task_id: BoundedText,
    pub(crate) generations: RuntimeGenerationBinding,
    pub(crate) readiness_window: ReadinessWindow,
    pub(crate) state_owner: codex_config::AuthoritativeStateOwner,
    pub(crate) backend_id: BoundedText,
    pub(crate) identity_id: BoundedText,
}

/// One immutable generation binding plus a live PF-19 revocation source.
pub(crate) struct ProtectedRuntime {
    snapshot: ProtectedRuntimeSnapshot,
    instance_id: [u8; 16],
    runtime_nonce: [u8; 16],
    configured_kill_switch_active: bool,
    readiness_producer: PolicyPrincipal,
    last_observed_unix_seconds: AtomicI64,
    routes: ProtectedRouteRegistry,
    revocations: Arc<RwLock<RevocationState>>,
}

impl ProtectedRuntime {
    pub(crate) fn compose(
        current: CurrentProtectedRuntime<'_>,
        routes: ProtectedRouteRegistry,
        revocations: Arc<RwLock<RevocationState>>,
    ) -> Result<Self, ProtectedRuntimeError> {
        validate_current_shape(&current, &revocations)?;
        validate_readiness_time(&current)?;
        let snapshot = ProtectedRuntimeSnapshot {
            contract_version: PROTECTED_RUNTIME_CONTRACT_VERSION,
            configured_level: current.effective.requested_level,
            creator_required_level: current.effective.creator_required_level,
            effective_level: current.effective.level,
            effective_policy_epoch: current.effective.epoch,
            actor_chain: current.effective.actor_chain.clone(),
            session_id: current.effective.session_id.clone(),
            task_id: current.effective.task_id.clone(),
            generations: current.readiness.generations,
            readiness_window: current.readiness.window,
            state_owner: current.authoritative.owner.clone(),
            backend_id: current.readiness.backend_id.clone(),
            identity_id: current.readiness.identity_id.clone(),
        };
        Ok(Self {
            snapshot,
            instance_id: *Uuid::new_v4().as_bytes(),
            runtime_nonce: current.effective.runtime_nonce,
            configured_kill_switch_active: current.authoritative.kill_switch_active,
            readiness_producer: current.readiness.producer.clone(),
            last_observed_unix_seconds: AtomicI64::new(current.now_unix_seconds),
            routes,
            revocations,
        })
    }

    pub(crate) fn snapshot(&self) -> &ProtectedRuntimeSnapshot {
        &self.snapshot
    }

    pub(crate) fn authorize_route(
        &self,
        current: CurrentProtectedRuntime<'_>,
        kind: ProtectedRouteKind,
        route_id: impl Into<String>,
    ) -> Result<(), ProtectedRuntimeError> {
        self.validate_current(current)?;
        let route_id = BoundedText::new(route_id.into())?;
        if !self.routes.contains(kind, &route_id) {
            return Err(ProtectedRuntimeError::UnknownRoute { kind, route_id });
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn reserve_dispatch(
        &self,
        current: CurrentProtectedRuntime<'_>,
        egress_route_id: impl Into<String>,
        journal: &mut ReferenceJournal,
        request: &AuthorizationRequest,
        authority: ProtectedAuthority<'_>,
        run_id: BoundedText,
        deduplication_key: BoundedText,
    ) -> Result<ProtectedDispatch, ProtectedRuntimeError> {
        self.authorize_route(current, ProtectedRouteKind::Egress, egress_route_id)?;
        let now_unix_seconds = current.now_unix_seconds;
        self.validate_request_binding(request)?;
        authority.validate_request(request, now_unix_seconds)?;
        let receiptless_never_admitted_completion =
            matches!(authority, ProtectedAuthority::Grant(_));
        let revocations = self.read_revocations()?;
        let (fence, identity, deduplication_key) = match authority {
            ProtectedAuthority::Grant(grant) => (
                DispatchFence::queued_for_grant(
                    run_id.clone(),
                    self.snapshot.generations.revocation,
                    now_unix_seconds,
                    grant,
                    &revocations,
                )?,
                AuthorityIdentity::from_grant(grant)?,
                domain_separated_deduplication_key("grant-effect", &deduplication_key)?,
            ),
            ProtectedAuthority::Mandate { mandate, .. } => (
                DispatchFence::queued_for_mandate(
                    run_id.clone(),
                    self.snapshot.generations.revocation,
                    now_unix_seconds,
                    mandate,
                    &revocations,
                )?,
                AuthorityIdentity::from_mandate(mandate)?,
                BoundedText::new(format!("mandate-preview:{}", mandate.preview_digest))?,
            ),
        };
        drop(revocations);
        let (permit, _) = journal.reserve_dispatch(
            self.event_context()?,
            None,
            request,
            identity,
            deduplication_key,
            now_unix_seconds,
        )?;
        Ok(ProtectedDispatch {
            runtime_instance_id: self.instance_id,
            receiptless_never_admitted_completion,
            run_id,
            fence,
            permit,
        })
    }

    fn validate_current(
        &self,
        current: CurrentProtectedRuntime<'_>,
    ) -> Result<(), ProtectedRuntimeError> {
        validate_current_shape(&current, &self.revocations)?;
        if current.effective.runtime_nonce != self.runtime_nonce
            || current.effective.epoch != self.snapshot.effective_policy_epoch
            || current.effective.requested_level != self.snapshot.configured_level
            || current.effective.creator_required_level != self.snapshot.creator_required_level
            || current.effective.level != self.snapshot.effective_level
            || current.effective.actor_chain != self.snapshot.actor_chain
            || current.effective.session_id != self.snapshot.session_id
            || current.effective.task_id != self.snapshot.task_id
            || current.authoritative.kill_switch_active != self.configured_kill_switch_active
            || current.readiness.generations != self.snapshot.generations
            || current.readiness.window != self.snapshot.readiness_window
            || current.authoritative.owner != self.snapshot.state_owner
            || current.readiness.backend_id != self.snapshot.backend_id
            || current.readiness.identity_id != self.snapshot.identity_id
            || current.readiness.producer != self.readiness_producer
        {
            return Err(ProtectedRuntimeError::StaleRuntimeBinding);
        }
        validate_readiness_time(&current)?;
        self.observe_time(current.now_unix_seconds)?;
        Ok(())
    }

    fn validate_request_binding(
        &self,
        request: &AuthorizationRequest,
    ) -> Result<(), ProtectedRuntimeError> {
        if request.subject != self.snapshot.actor_chain
            || request.context.session_id != self.snapshot.session_id
            || request.context.task_id != self.snapshot.task_id
        {
            return Err(ProtectedRuntimeError::RequestRuntimeMismatch);
        }
        Ok(())
    }

    fn validate_instance(&self, instance_id: [u8; 16]) -> Result<(), ProtectedRuntimeError> {
        if self.instance_id != instance_id {
            return Err(ProtectedRuntimeError::RuntimeInstanceMismatch);
        }
        Ok(())
    }

    fn observe_time(&self, now_unix_seconds: i64) -> Result<(), ProtectedRuntimeError> {
        self.last_observed_unix_seconds
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |previous| {
                (now_unix_seconds >= previous).then_some(now_unix_seconds)
            })
            .map(|_| ())
            .map_err(|_| ProtectedRuntimeError::RuntimeClockRegression)
    }

    pub(crate) fn event_context(&self) -> Result<EventContext, ProtectedRuntimeError> {
        Ok(EventContext::new(
            self.readiness_producer.clone(),
            self.snapshot.generations.policy,
            self.snapshot.generations.run,
        )?)
    }

    fn read_revocations(
        &self,
    ) -> Result<std::sync::RwLockReadGuard<'_, RevocationState>, ProtectedRuntimeError> {
        self.revocations
            .read()
            .map_err(|_| ProtectedRuntimeError::RevocationStatePoisoned)
    }
}

#[derive(Clone, Copy)]
pub(crate) enum ProtectedAuthority<'a> {
    Grant(&'a BoundedGrant),
    Mandate {
        mandate: &'a ProtectedActionMandate,
        preview: &'a ProtectedActionPreview,
    },
}

impl ProtectedAuthority<'_> {
    fn validate_request(
        self,
        request: &AuthorizationRequest,
        now_unix_seconds: i64,
    ) -> Result<(), ProtectedRuntimeError> {
        let matches = match self {
            Self::Grant(grant) => grant.matches_request(request).unwrap_or(false),
            Self::Mandate { mandate, preview } => {
                preview.request == *request
                    && mandate
                        .matches_preview(preview, now_unix_seconds)
                        .unwrap_or(false)
            }
        };
        if !matches {
            return Err(ProtectedRuntimeError::AuthorityRequestMismatch);
        }
        Ok(())
    }
}

#[must_use = "a protected dispatch must be resolved as completed or unknown"]
pub(crate) struct ProtectedDispatch {
    runtime_instance_id: [u8; 16],
    receiptless_never_admitted_completion: bool,
    run_id: BoundedText,
    fence: DispatchFence,
    permit: DispatchPermit,
}

impl ProtectedDispatch {
    /// Authorize and perform one bounded effect while holding the PF-19 read
    /// guard that defines revocation linearization.
    ///
    /// Stream and channel adapters must call this for each individual write;
    /// journal persistence happens outside this guard.
    pub(crate) fn authorize<T>(
        &mut self,
        runtime: &ProtectedRuntime,
        current: CurrentProtectedRuntime<'_>,
        authority: ProtectedAuthority<'_>,
        step: ProtectedDispatchStep,
        effect: impl FnOnce() -> T,
    ) -> Result<T, ProtectedRuntimeError> {
        runtime.validate_instance(self.runtime_instance_id)?;
        let now_unix_seconds = current.now_unix_seconds;
        runtime.validate_current(current)?;
        let revocations = runtime.read_revocations()?;
        match authority {
            ProtectedAuthority::Grant(grant) => self.fence.authorize_grant(
                &self.run_id,
                now_unix_seconds,
                grant,
                &revocations,
                step,
            )?,
            ProtectedAuthority::Mandate { mandate, .. } => self.fence.authorize_mandate(
                &self.run_id,
                now_unix_seconds,
                mandate,
                &revocations,
                step,
            )?,
        }
        Ok(effect())
    }

    pub(crate) fn resolve(
        mut self,
        runtime: &ProtectedRuntime,
        journal: &mut ReferenceJournal,
        resolution: DispatchResolution,
        occurred_at_unix_seconds: i64,
    ) -> Result<AppendAcknowledgement, ProtectedRuntimeError> {
        runtime.validate_instance(self.runtime_instance_id)?;
        let phase = self.fence.phase();
        let terminal_transition = match &resolution {
            DispatchResolution::Completed { .. } => self.fence.record_completed(),
            DispatchResolution::Unknown { .. } => self.fence.record_unknown_financial_outcome(),
        };
        if let Err(error) = terminal_transition
            && !never_admitted_resolution_is_safe(
                phase,
                self.receiptless_never_admitted_completion,
                &resolution,
                &error,
            )
        {
            return Err(error.into());
        }
        Ok(journal.resolve_dispatch(
            self.permit,
            runtime.event_context()?,
            resolution,
            occurred_at_unix_seconds,
        )?)
    }
}

fn domain_separated_deduplication_key(
    domain: &str,
    key: &BoundedText,
) -> Result<BoundedText, ProtectedRuntimeError> {
    Ok(BoundedText::new(format!(
        "{domain}:{:x}",
        Sha256::digest(key.as_str().as_bytes())
    ))?)
}

fn never_admitted_resolution_is_safe(
    phase: DispatchPhase,
    receiptless_completion_allowed: bool,
    resolution: &DispatchResolution,
    error: &RevocationError,
) -> bool {
    matches!(error, RevocationError::InvalidDispatchTransition)
        && matches!(phase, DispatchPhase::Queued | DispatchPhase::Fenced)
        && (matches!(resolution, DispatchResolution::Unknown { .. })
            || receiptless_completion_allowed
                && matches!(
                    resolution,
                    DispatchResolution::Completed {
                        outcome: MandateOutcome::Denied | MandateOutcome::Cancelled,
                        ..
                    }
                ))
}

fn validate_current_shape(
    current: &CurrentProtectedRuntime<'_>,
    revocations: &RwLock<RevocationState>,
) -> Result<(), ProtectedRuntimeError> {
    current.authoritative.validate()?;
    current.readiness.validate_shape()?;
    if current.effective.level == SecurityLevel::Permissive {
        return Err(ProtectedRuntimeError::ProtectedLevelRequired);
    }
    if current.effective.requested_level != current.authoritative.level
        || current.effective.level
            != current
                .effective
                .requested_level
                .max(current.effective.creator_required_level)
        || current.effective.revocation_generation != current.authoritative.revocation_generation
        || current.authoritative.kill_switch_active
            != current.effective.authority_kill_switch_active
    {
        return Err(ProtectedRuntimeError::EffectivePolicyMismatch);
    }
    if current.effective.kill_switch_active {
        return Err(ProtectedRuntimeError::RuntimeRestricted);
    }
    if current.readiness.status != ProtectionReadinessStatus::Ready {
        return Err(ProtectedRuntimeError::ReadinessUnavailable(
            current.readiness.status,
        ));
    }
    let expected = RuntimeGenerationBinding {
        owner: current.authoritative.owner.owner_generation,
        policy: current.authoritative.revision,
        run: current.run_generation,
        revocation: current.authoritative.revocation_generation,
    };
    if current.readiness.generations != expected
        || current.readiness.state_owner != current.authoritative.owner
    {
        return Err(ProtectedRuntimeError::StaleReadinessGeneration);
    }
    if !current.recovery.permits_protected_dispatch() {
        return Err(ProtectedRuntimeError::AuditRecoveryBlocked);
    }
    validate_recovery_binding(current)?;
    let revocations = revocations
        .read()
        .map_err(|_| ProtectedRuntimeError::RevocationStatePoisoned)?;
    revocations.validate()?;
    if revocations.generation != current.authoritative.revocation_generation
        || revocations.kill_switch_active != current.authoritative.kill_switch_active
    {
        return Err(ProtectedRuntimeError::StaleRevocationState);
    }
    Ok(())
}

fn validate_readiness_time(
    current: &CurrentProtectedRuntime<'_>,
) -> Result<(), ProtectedRuntimeError> {
    if current.now_unix_seconds < current.readiness.window.measured_at_unix_seconds
        || current.now_unix_seconds >= current.readiness.window.expires_at_unix_seconds
    {
        return Err(ProtectedRuntimeError::ExpiredReadiness);
    }
    Ok(())
}

fn validate_recovery_binding(
    current: &CurrentProtectedRuntime<'_>,
) -> Result<(), ProtectedRuntimeError> {
    match (&current.recovery.state, &current.recovery.checkpoint) {
        (RecoveryState::Empty, None) if current.recovery.event_count == 0 => Ok(()),
        (RecoveryState::Ready, Some(checkpoint))
            if u64::try_from(current.recovery.event_count) == Ok(checkpoint.sequence)
                && checkpoint.producer == current.readiness.producer
                && checkpoint.owner_generation == current.authoritative.owner.owner_generation
                && checkpoint.policy_generation <= current.authoritative.revision
                && checkpoint.run_generation <= current.run_generation =>
        {
            Ok(())
        }
        _ => Err(ProtectedRuntimeError::AuditRecoveryBindingMismatch),
    }
}

#[derive(Debug, Error)]
pub(crate) enum ProtectedRuntimeError {
    #[error("protected runtime requires Moderate or Aggressive effective policy")]
    ProtectedLevelRequired,
    #[error("effective policy does not match controller-owned state")]
    EffectivePolicyMismatch,
    #[error("effective policy or revocation state currently restricts this runtime")]
    RuntimeRestricted,
    #[error("readiness producer must be a service identity")]
    InvalidReadinessProducer,
    #[error("readiness measurement generations or time window are invalid")]
    InvalidReadinessMeasurement,
    #[error("protected readiness is {0:?}")]
    ReadinessUnavailable(ProtectionReadinessStatus),
    #[error("protected readiness does not match the authoritative generation")]
    StaleReadinessGeneration,
    #[error("protected readiness is not current")]
    ExpiredReadiness,
    #[error("durable audit recovery does not permit protected dispatch")]
    AuditRecoveryBlocked,
    #[error("live revocation state does not match controller-owned state")]
    StaleRevocationState,
    #[error("protected runtime binding is stale")]
    StaleRuntimeBinding,
    #[error("protected dispatch belongs to a different runtime instance")]
    RuntimeInstanceMismatch,
    #[error("protected authority does not match the exact authorization request")]
    AuthorityRequestMismatch,
    #[error("authorization request does not match the protected runtime actor binding")]
    RequestRuntimeMismatch,
    #[error("protected runtime clock moved backwards")]
    RuntimeClockRegression,
    #[error("durable audit recovery does not match the protected runtime identity")]
    AuditRecoveryBindingMismatch,
    #[error("protected revocation state lock is poisoned")]
    RevocationStatePoisoned,
    #[error("protected route is already registered")]
    DuplicateRoute,
    #[error("unregistered protected {kind:?} route {route_id}")]
    UnknownRoute {
        kind: ProtectedRouteKind,
        route_id: BoundedText,
    },
    #[error(transparent)]
    AuthoritativeState(#[from] codex_config::AuthoritativeStateValidationError),
    #[error(transparent)]
    BoundedText(#[from] codex_security_policy::BoundedTextError),
    #[error(transparent)]
    Revocation(#[from] RevocationError),
    #[error(transparent)]
    SecurityEvent(#[from] SecurityEventError),
    #[error(transparent)]
    Journal(#[from] JournalError),
}
