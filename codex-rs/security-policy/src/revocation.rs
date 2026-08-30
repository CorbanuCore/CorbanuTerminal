use std::collections::BTreeSet;

use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;
use thiserror::Error;

use crate::BoundedGrant;
use crate::BoundedText;
use crate::PolicyPrincipal;
use crate::PrincipalKind;
use crate::ProtectedActionMandate;
use crate::digest::canonical_sha256;

pub const REVOCATION_SCHEMA_VERSION: u32 = 1;
pub const DISPATCH_FENCE_SCHEMA_VERSION: u32 = 1;

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum DispatchAuthorityKind {
    Grant,
    Mandate,
}

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum DispatchPhase {
    Queued,
    Admitted,
    EstablishedChannel,
    Uploading,
    Fenced,
    Completed,
    UnknownFinancialOutcome,
}

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum ProtectedDispatchStep {
    Admit,
    EstablishChannel,
    BeginUpload,
    UploadWrite,
    FinishUpload,
    ChannelWrite,
}

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum RestrictionAuditStatus {
    Recorded,
    Unavailable,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RestrictionApplication {
    pub event_was_effective: bool,
    pub generation: u64,
    pub audit_status: RestrictionAuditStatus,
}

/// Adapter-neutral fence for one run and one grant or mandate.
///
/// Consumers must call `authorize_*` while holding the same revocation-state
/// read guard through the protected dispatch or write. That check is the
/// linearization point. A stale generation never authorizes work; unaffected
/// authority may continue only after a trusted `refresh_*` revalidation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DispatchFence {
    schema_version: u32,
    run_id: BoundedText,
    authority_kind: DispatchAuthorityKind,
    authority_id: BoundedText,
    generation: u64,
    phase: DispatchPhase,
}

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum RevocationReason {
    HumanRequest,
    SecurityLevelChange,
    RiskSignal,
    GrantExpired,
    KillSwitch,
}

#[derive(
    Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum RevocationTarget {
    Grant { grant_id: BoundedText },
    Mandate { mandate_id: BoundedText },
    Actor { actor_id: BoundedText },
    AllActiveAuthority,
    KillSwitch { active: bool },
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct RevocationOrder {
    created_at_unix_seconds: i64,
    event_id: BoundedText,
}

/// Human-originated invalidation event. No free-form reason or secret payload
/// is accepted.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RevocationEvent {
    pub schema_version: u32,
    pub event_id: BoundedText,
    pub issuer: PolicyPrincipal,
    pub target: RevocationTarget,
    pub reason: RevocationReason,
    pub created_at_unix_seconds: i64,
}

#[derive(Serialize)]
struct RevocationBinding<'a> {
    schema_version: u32,
    issuer: &'a PolicyPrincipal,
    target: &'a RevocationTarget,
    reason: RevocationReason,
    created_at_unix_seconds: i64,
}

impl RevocationEvent {
    pub fn new(
        issuer: PolicyPrincipal,
        target: RevocationTarget,
        reason: RevocationReason,
        created_at_unix_seconds: i64,
    ) -> Result<Self, RevocationError> {
        let mut event = Self {
            schema_version: REVOCATION_SCHEMA_VERSION,
            event_id: BoundedText::new("pending")?,
            issuer,
            target,
            reason,
            created_at_unix_seconds,
        };
        event.validate_fields()?;
        event.event_id = BoundedText::new(event.expected_id()?)?;
        Ok(event)
    }

    pub fn validate(&self) -> Result<(), RevocationError> {
        self.validate_fields()?;
        if self.event_id.as_str() != self.expected_id()? {
            return Err(RevocationError::IntegrityMismatch);
        }
        Ok(())
    }

    fn validate_fields(&self) -> Result<(), RevocationError> {
        if self.schema_version != REVOCATION_SCHEMA_VERSION {
            return Err(RevocationError::UnsupportedSchemaVersion {
                found: self.schema_version,
                supported: REVOCATION_SCHEMA_VERSION,
            });
        }
        if self.issuer.kind != PrincipalKind::Human {
            return Err(RevocationError::IssuerMustBeHuman);
        }
        if self.created_at_unix_seconds < 0 {
            return Err(RevocationError::NegativeTimestamp);
        }
        Ok(())
    }

    fn expected_id(&self) -> Result<String, RevocationError> {
        canonical_sha256(&RevocationBinding {
            schema_version: self.schema_version,
            issuer: &self.issuer,
            target: &self.target,
            reason: self.reason,
            created_at_unix_seconds: self.created_at_unix_seconds,
        })
        .map_err(RevocationError::Serialization)
    }
}

/// Versioned, restart-safe invalidation state.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RevocationState {
    pub schema_version: u32,
    pub generation: u64,
    pub kill_switch_active: bool,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    revoked_grant_ids: BTreeSet<BoundedText>,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    revoked_mandate_ids: BTreeSet<BoundedText>,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    revoked_actor_ids: BTreeSet<BoundedText>,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    applied_event_ids: BTreeSet<BoundedText>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    all_authority_revoked_at_unix_seconds: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    last_kill_switch_event: Option<RevocationOrder>,
}

impl RevocationState {
    pub fn new() -> Self {
        Self {
            schema_version: REVOCATION_SCHEMA_VERSION,
            generation: 0,
            kill_switch_active: false,
            revoked_grant_ids: BTreeSet::new(),
            revoked_mandate_ids: BTreeSet::new(),
            revoked_actor_ids: BTreeSet::new(),
            applied_event_ids: BTreeSet::new(),
            all_authority_revoked_at_unix_seconds: None,
            last_kill_switch_event: None,
        }
    }

    /// Apply an event exactly once. Duplicate delivery is harmless.
    pub fn apply(&mut self, event: &RevocationEvent) -> Result<bool, RevocationError> {
        self.validate()?;
        event.validate()?;
        if self.applied_event_ids.contains(&event.event_id) {
            return Ok(false);
        }

        let next_generation = self
            .generation
            .checked_add(1)
            .ok_or(RevocationError::GenerationOverflow)?;
        let kill_switch_order =
            matches!(&event.target, RevocationTarget::KillSwitch { .. }).then(|| RevocationOrder {
                created_at_unix_seconds: event.created_at_unix_seconds,
                event_id: event.event_id.clone(),
            });
        if let (Some(candidate), Some(previous)) =
            (&kill_switch_order, &self.last_kill_switch_event)
            && candidate <= previous
        {
            if matches!(&event.target, RevocationTarget::KillSwitch { active: true }) {
                self.all_authority_revoked_at_unix_seconds = Some(
                    self.all_authority_revoked_at_unix_seconds
                        .map_or(event.created_at_unix_seconds, |previous| {
                            previous.max(event.created_at_unix_seconds)
                        }),
                );
            }
            self.applied_event_ids.insert(event.event_id.clone());
            self.generation = next_generation;
            return Ok(false);
        }

        match &event.target {
            RevocationTarget::Grant { grant_id } => {
                self.revoked_grant_ids.insert(grant_id.clone());
            }
            RevocationTarget::Mandate { mandate_id } => {
                self.revoked_mandate_ids.insert(mandate_id.clone());
            }
            RevocationTarget::Actor { actor_id } => {
                self.revoked_actor_ids.insert(actor_id.clone());
            }
            RevocationTarget::AllActiveAuthority => {
                self.all_authority_revoked_at_unix_seconds = Some(
                    self.all_authority_revoked_at_unix_seconds
                        .map_or(event.created_at_unix_seconds, |previous| {
                            previous.max(event.created_at_unix_seconds)
                        }),
                );
            }
            RevocationTarget::KillSwitch { active } => {
                self.kill_switch_active = *active;
                self.last_kill_switch_event = kill_switch_order;
                if *active {
                    self.all_authority_revoked_at_unix_seconds = Some(
                        self.all_authority_revoked_at_unix_seconds
                            .map_or(event.created_at_unix_seconds, |previous| {
                                previous.max(event.created_at_unix_seconds)
                            }),
                    );
                }
            }
        }
        self.applied_event_ids.insert(event.event_id.clone());
        self.generation = next_generation;
        Ok(true)
    }

    /// Apply an emergency restriction before recording whether its audit write
    /// succeeded. Audit failure is preserved in the result and cannot delay or
    /// roll back the restriction.
    pub fn apply_restriction(
        &mut self,
        event: &RevocationEvent,
        write_audit: impl FnOnce() -> RestrictionAuditStatus,
    ) -> Result<RestrictionApplication, RevocationError> {
        if matches!(event.target, RevocationTarget::KillSwitch { active: false }) {
            return Err(RevocationError::NotARestriction);
        }
        let event_was_effective = self.apply(event)?;
        let audit_status = write_audit();
        Ok(RestrictionApplication {
            event_was_effective,
            generation: self.generation,
            audit_status,
        })
    }

    pub fn grant_is_revoked(&self, grant: &BoundedGrant) -> bool {
        self.kill_switch_active
            || self.revoked_grant_ids.contains(&grant.grant_id)
            || grant
                .actor_chain
                .as_slice()
                .iter()
                .any(|actor| self.revoked_actor_ids.contains(&actor.id))
            || self
                .all_authority_revoked_at_unix_seconds
                .is_some_and(|revoked_at| grant.issued_at_unix_seconds <= revoked_at)
    }

    pub fn mandate_is_revoked(&self, mandate: &ProtectedActionMandate) -> bool {
        self.kill_switch_active
            || self.revoked_mandate_ids.contains(&mandate.mandate_id)
            || mandate
                .actor_chain
                .as_slice()
                .iter()
                .any(|actor| self.revoked_actor_ids.contains(&actor.id))
            || self
                .all_authority_revoked_at_unix_seconds
                .is_some_and(|revoked_at| mandate.approved_at_unix_seconds <= revoked_at)
    }

    pub fn validate(&self) -> Result<(), RevocationError> {
        if self.schema_version != REVOCATION_SCHEMA_VERSION {
            return Err(RevocationError::UnsupportedSchemaVersion {
                found: self.schema_version,
                supported: REVOCATION_SCHEMA_VERSION,
            });
        }
        let applied_events = u64::try_from(self.applied_event_ids.len())
            .map_err(|_| RevocationError::GenerationOverflow)?;
        if self.generation != applied_events {
            return Err(RevocationError::GenerationMismatch {
                generation: self.generation,
                applied_events,
            });
        }
        if self
            .all_authority_revoked_at_unix_seconds
            .is_some_and(|timestamp| timestamp < 0)
        {
            return Err(RevocationError::CorruptStateTimestamp);
        }
        if let Some(last_event) = &self.last_kill_switch_event
            && (last_event.created_at_unix_seconds < 0
                || !self.applied_event_ids.contains(&last_event.event_id))
        {
            return Err(RevocationError::CorruptKillSwitchState);
        }
        if self.kill_switch_active
            && (self.last_kill_switch_event.is_none()
                || self.all_authority_revoked_at_unix_seconds.is_none())
        {
            return Err(RevocationError::CorruptKillSwitchState);
        }
        Ok(())
    }
}

impl DispatchFence {
    pub fn schema_version(&self) -> u32 {
        self.schema_version
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn phase(&self) -> DispatchPhase {
        self.phase
    }

    pub fn queued_for_grant(
        run_id: BoundedText,
        expected_generation: u64,
        grant: &BoundedGrant,
        revocations: &RevocationState,
    ) -> Result<Self, RevocationError> {
        grant
            .validate()
            .map_err(|_| RevocationError::InvalidAuthority)?;
        Self::queued(
            run_id,
            expected_generation,
            DispatchAuthorityKind::Grant,
            grant.grant_id.clone(),
            revocations,
            revocations.grant_is_revoked(grant),
        )
    }

    pub fn queued_for_mandate(
        run_id: BoundedText,
        expected_generation: u64,
        mandate: &ProtectedActionMandate,
        revocations: &RevocationState,
    ) -> Result<Self, RevocationError> {
        mandate
            .validate()
            .map_err(|_| RevocationError::InvalidAuthority)?;
        Self::queued(
            run_id,
            expected_generation,
            DispatchAuthorityKind::Mandate,
            mandate.mandate_id.clone(),
            revocations,
            revocations.mandate_is_revoked(mandate),
        )
    }

    fn queued(
        run_id: BoundedText,
        expected_generation: u64,
        authority_kind: DispatchAuthorityKind,
        authority_id: BoundedText,
        revocations: &RevocationState,
        revoked: bool,
    ) -> Result<Self, RevocationError> {
        revocations.validate()?;
        if expected_generation != revocations.generation {
            return Err(RevocationError::StaleDispatchGeneration {
                expected: expected_generation,
                current: revocations.generation,
            });
        }
        if revoked {
            return Err(RevocationError::AuthorityRevoked);
        }
        Ok(Self {
            schema_version: DISPATCH_FENCE_SCHEMA_VERSION,
            run_id,
            authority_kind,
            authority_id,
            generation: expected_generation,
            phase: DispatchPhase::Queued,
        })
    }

    pub fn authorize_grant(
        &mut self,
        run_id: &BoundedText,
        grant: &BoundedGrant,
        revocations: &RevocationState,
        step: ProtectedDispatchStep,
    ) -> Result<(), RevocationError> {
        grant
            .validate()
            .map_err(|_| RevocationError::InvalidAuthority)?;
        self.authorize(
            run_id,
            DispatchAuthorityKind::Grant,
            &grant.grant_id,
            revocations,
            revocations.grant_is_revoked(grant),
            step,
        )
    }

    pub fn authorize_mandate(
        &mut self,
        run_id: &BoundedText,
        mandate: &ProtectedActionMandate,
        revocations: &RevocationState,
        step: ProtectedDispatchStep,
    ) -> Result<(), RevocationError> {
        mandate
            .validate()
            .map_err(|_| RevocationError::InvalidAuthority)?;
        self.authorize(
            run_id,
            DispatchAuthorityKind::Mandate,
            &mandate.mandate_id,
            revocations,
            revocations.mandate_is_revoked(mandate),
            step,
        )
    }

    pub fn refresh_grant(
        &mut self,
        run_id: &BoundedText,
        grant: &BoundedGrant,
        revocations: &RevocationState,
    ) -> Result<(), RevocationError> {
        grant
            .validate()
            .map_err(|_| RevocationError::InvalidAuthority)?;
        self.refresh(
            run_id,
            DispatchAuthorityKind::Grant,
            &grant.grant_id,
            revocations,
            revocations.grant_is_revoked(grant),
        )
    }

    pub fn refresh_mandate(
        &mut self,
        run_id: &BoundedText,
        mandate: &ProtectedActionMandate,
        revocations: &RevocationState,
    ) -> Result<(), RevocationError> {
        mandate
            .validate()
            .map_err(|_| RevocationError::InvalidAuthority)?;
        self.refresh(
            run_id,
            DispatchAuthorityKind::Mandate,
            &mandate.mandate_id,
            revocations,
            revocations.mandate_is_revoked(mandate),
        )
    }

    fn authorize(
        &mut self,
        run_id: &BoundedText,
        authority_kind: DispatchAuthorityKind,
        authority_id: &BoundedText,
        revocations: &RevocationState,
        revoked: bool,
        step: ProtectedDispatchStep,
    ) -> Result<(), RevocationError> {
        self.check_binding(run_id, authority_kind, authority_id)?;
        revocations.validate()?;
        if revoked {
            self.phase = DispatchPhase::Fenced;
            return Err(RevocationError::AuthorityRevoked);
        }
        if self.generation != revocations.generation {
            return Err(RevocationError::StaleDispatchGeneration {
                expected: self.generation,
                current: revocations.generation,
            });
        }
        self.phase = match (self.phase, step) {
            (DispatchPhase::Queued, ProtectedDispatchStep::Admit) => DispatchPhase::Admitted,
            (DispatchPhase::Admitted, ProtectedDispatchStep::EstablishChannel) => {
                DispatchPhase::EstablishedChannel
            }
            (
                DispatchPhase::Admitted | DispatchPhase::EstablishedChannel,
                ProtectedDispatchStep::BeginUpload,
            )
            | (DispatchPhase::Uploading, ProtectedDispatchStep::UploadWrite) => {
                DispatchPhase::Uploading
            }
            (DispatchPhase::Uploading, ProtectedDispatchStep::FinishUpload)
            | (DispatchPhase::EstablishedChannel, ProtectedDispatchStep::ChannelWrite) => {
                DispatchPhase::EstablishedChannel
            }
            _ => return Err(RevocationError::InvalidDispatchTransition),
        };
        Ok(())
    }

    fn refresh(
        &mut self,
        run_id: &BoundedText,
        authority_kind: DispatchAuthorityKind,
        authority_id: &BoundedText,
        revocations: &RevocationState,
        revoked: bool,
    ) -> Result<(), RevocationError> {
        self.check_binding(run_id, authority_kind, authority_id)?;
        revocations.validate()?;
        if revoked {
            self.phase = DispatchPhase::Fenced;
            return Err(RevocationError::AuthorityRevoked);
        }
        if matches!(
            self.phase,
            DispatchPhase::Fenced
                | DispatchPhase::Completed
                | DispatchPhase::UnknownFinancialOutcome
        ) {
            return Err(RevocationError::InvalidDispatchTransition);
        }
        self.generation = revocations.generation;
        Ok(())
    }

    fn check_binding(
        &self,
        run_id: &BoundedText,
        authority_kind: DispatchAuthorityKind,
        authority_id: &BoundedText,
    ) -> Result<(), RevocationError> {
        if self.schema_version != DISPATCH_FENCE_SCHEMA_VERSION {
            return Err(RevocationError::UnsupportedDispatchFenceVersion {
                found: self.schema_version,
                supported: DISPATCH_FENCE_SCHEMA_VERSION,
            });
        }
        if &self.run_id != run_id
            || self.authority_kind != authority_kind
            || &self.authority_id != authority_id
        {
            return Err(RevocationError::DispatchBindingMismatch);
        }
        Ok(())
    }

    pub fn record_completed(&mut self) -> Result<(), RevocationError> {
        self.record_terminal(DispatchPhase::Completed)
    }

    pub fn record_unknown_financial_outcome(&mut self) -> Result<(), RevocationError> {
        self.record_terminal(DispatchPhase::UnknownFinancialOutcome)
    }

    fn record_terminal(&mut self, outcome: DispatchPhase) -> Result<(), RevocationError> {
        match self.phase {
            DispatchPhase::Admitted
            | DispatchPhase::EstablishedChannel
            | DispatchPhase::Uploading
            | DispatchPhase::Fenced => self.phase = outcome,
            phase if phase == outcome => {}
            _ => return Err(RevocationError::InvalidDispatchTransition),
        }
        Ok(())
    }
}

impl Default for RevocationState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Error)]
pub enum RevocationError {
    #[error("revocation issuer must be a human")]
    IssuerMustBeHuman,
    #[error("revocation timestamp must be non-negative")]
    NegativeTimestamp,
    #[error("revocation integrity digest does not match its bound fields")]
    IntegrityMismatch,
    #[error("revocation generation overflowed")]
    GenerationOverflow,
    #[error("revocation generation {generation} does not match {applied_events} applied events")]
    GenerationMismatch {
        generation: u64,
        applied_events: u64,
    },
    #[error("revocation state contains an invalid timestamp")]
    CorruptStateTimestamp,
    #[error("revocation state contains an invalid kill-switch history")]
    CorruptKillSwitchState,
    #[error("kill-switch deactivation is not an emergency restriction")]
    NotARestriction,
    #[error("dispatch authority is malformed")]
    InvalidAuthority,
    #[error("dispatch authority has been revoked")]
    AuthorityRevoked,
    #[error("dispatch generation {expected} is stale; current generation is {current}")]
    StaleDispatchGeneration { expected: u64, current: u64 },
    #[error("dispatch run or authority binding does not match")]
    DispatchBindingMismatch,
    #[error("dispatch phase transition is invalid")]
    InvalidDispatchTransition,
    #[error("unsupported dispatch-fence schema version {found}; supported version is {supported}")]
    UnsupportedDispatchFenceVersion { found: u32, supported: u32 },
    #[error("unsupported revocation schema version {found}; supported version is {supported}")]
    UnsupportedSchemaVersion { found: u32, supported: u32 },
    #[error(transparent)]
    BoundedText(#[from] crate::BoundedTextError),
    #[error("failed to serialize canonical revocation object: {0}")]
    Serialization(serde_json::Error),
}
