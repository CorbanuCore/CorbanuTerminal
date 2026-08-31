//! Fail-closed composition seam for protected runtime prerequisites.
//!
//! This module does not activate protected mode or own an ingress, egress, or
//! broker adapter. It binds the completed policy, authoritative-state,
//! revocation-fence, and durable-event contracts so later adapters cannot use a
//! stale measurement or an unregistered route as authority.
#![allow(dead_code)]

use std::collections::BTreeSet;
use std::sync::Arc;
use std::sync::RwLock;

use codex_config::AuthoritativeSecurityState;
use codex_security_audit::AuthorityIdentity;
use codex_security_audit::DispatchPermit;
use codex_security_audit::EventContext;
use codex_security_audit::JournalError;
use codex_security_audit::RecoveryReport;
use codex_security_audit::ReferenceJournal;
use codex_security_audit::SecurityEventError;
use codex_security_policy::AuthorizationRequest;
use codex_security_policy::BoundedGrant;
use codex_security_policy::BoundedText;
use codex_security_policy::DispatchFence;
use codex_security_policy::PolicyPrincipal;
use codex_security_policy::PrincipalKind;
use codex_security_policy::ProtectedActionMandate;
use codex_security_policy::ProtectedActionPreview;
use codex_security_policy::ProtectedDispatchStep;
use codex_security_policy::RevocationError;
use codex_security_policy::RevocationState;
use codex_security_policy::SecurityLevel;
use thiserror::Error;

use super::effective_policy::EffectivePolicySnapshot;

pub(crate) const PROTECTED_RUNTIME_CONTRACT_VERSION: u32 = 1;

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
    pub(crate) generations: RuntimeGenerationBinding,
    pub(crate) state_owner: codex_config::AuthoritativeStateOwner,
    pub(crate) backend_id: BoundedText,
    pub(crate) identity_id: BoundedText,
}

/// One immutable generation binding plus a live PF-19 revocation source.
pub(crate) struct ProtectedRuntime {
    snapshot: ProtectedRuntimeSnapshot,
    runtime_nonce: [u8; 16],
    configured_kill_switch_active: bool,
    readiness_producer: PolicyPrincipal,
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
        let snapshot = ProtectedRuntimeSnapshot {
            contract_version: PROTECTED_RUNTIME_CONTRACT_VERSION,
            configured_level: current.effective.requested_level,
            creator_required_level: current.effective.creator_required_level,
            effective_level: current.effective.level,
            effective_policy_epoch: current.effective.epoch,
            generations: current.readiness.generations,
            state_owner: current.authoritative.owner.clone(),
            backend_id: current.readiness.backend_id.clone(),
            identity_id: current.readiness.identity_id.clone(),
        };
        Ok(Self {
            snapshot,
            runtime_nonce: current.effective.runtime_nonce,
            configured_kill_switch_active: current.authoritative.kill_switch_active,
            readiness_producer: current.readiness.producer.clone(),
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
        authority.validate_request(request)?;
        let revocations = self.read_revocations()?;
        let (fence, identity) = match authority {
            ProtectedAuthority::Grant(grant) => (
                DispatchFence::queued_for_grant(
                    run_id.clone(),
                    self.snapshot.generations.revocation,
                    request.context.now_unix_seconds,
                    grant,
                    &revocations,
                )?,
                AuthorityIdentity::from_grant(grant)?,
            ),
            ProtectedAuthority::Mandate { mandate, .. } => (
                DispatchFence::queued_for_mandate(
                    run_id.clone(),
                    self.snapshot.generations.revocation,
                    request.context.now_unix_seconds,
                    mandate,
                    &revocations,
                )?,
                AuthorityIdentity::from_mandate(mandate)?,
            ),
        };
        let (permit, _) = journal.reserve_dispatch(
            self.event_context()?,
            None,
            request,
            identity,
            deduplication_key,
            request.context.now_unix_seconds,
        )?;
        Ok(ProtectedDispatch {
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
        let candidate = (
            current.effective.runtime_nonce,
            current.effective.epoch,
            current.effective.requested_level,
            current.effective.creator_required_level,
            current.effective.level,
            current.authoritative.kill_switch_active,
            current.readiness.generations,
            &current.authoritative.owner,
            &current.readiness.backend_id,
            &current.readiness.identity_id,
            &current.readiness.producer,
        );
        let bound = (
            self.runtime_nonce,
            self.snapshot.effective_policy_epoch,
            self.snapshot.configured_level,
            self.snapshot.creator_required_level,
            self.snapshot.effective_level,
            self.configured_kill_switch_active,
            self.snapshot.generations,
            &self.snapshot.state_owner,
            &self.snapshot.backend_id,
            &self.snapshot.identity_id,
            &self.readiness_producer,
        );
        if candidate != bound {
            return Err(ProtectedRuntimeError::StaleRuntimeBinding);
        }
        Ok(())
    }

    fn event_context(&self) -> Result<EventContext, ProtectedRuntimeError> {
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
    fn validate_request(self, request: &AuthorizationRequest) -> Result<(), ProtectedRuntimeError> {
        let matches = match self {
            Self::Grant(grant) => grant.matches_request(request).unwrap_or(false),
            Self::Mandate { mandate, preview } => {
                preview.request == *request
                    && mandate
                        .matches_preview(preview, request.context.now_unix_seconds)
                        .unwrap_or(false)
            }
        };
        if !matches {
            return Err(ProtectedRuntimeError::AuthorityRequestMismatch);
        }
        Ok(())
    }
}

pub(crate) struct ProtectedDispatch {
    run_id: BoundedText,
    fence: DispatchFence,
    permit: DispatchPermit,
}

impl ProtectedDispatch {
    pub(crate) fn authorize<T>(
        &mut self,
        runtime: &ProtectedRuntime,
        current: CurrentProtectedRuntime<'_>,
        authority: ProtectedAuthority<'_>,
        step: ProtectedDispatchStep,
        effect: impl FnOnce() -> T,
    ) -> Result<T, ProtectedRuntimeError> {
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

    pub(crate) fn record_completed(mut self) -> Result<DispatchPermit, ProtectedRuntimeError> {
        self.fence.record_completed()?;
        Ok(self.permit)
    }

    pub(crate) fn record_unknown(mut self) -> Result<DispatchPermit, ProtectedRuntimeError> {
        self.fence.record_unknown_financial_outcome()?;
        Ok(self.permit)
    }
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
    if current.now_unix_seconds < current.readiness.window.measured_at_unix_seconds
        || current.now_unix_seconds >= current.readiness.window.expires_at_unix_seconds
    {
        return Err(ProtectedRuntimeError::ExpiredReadiness);
    }
    if !current.recovery.permits_protected_dispatch() {
        return Err(ProtectedRuntimeError::AuditRecoveryBlocked);
    }
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
    #[error("protected authority does not match the exact authorization request")]
    AuthorityRequestMismatch,
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
