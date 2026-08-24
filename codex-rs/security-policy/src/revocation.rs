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
    #[error("unsupported revocation schema version {found}; supported version is {supported}")]
    UnsupportedSchemaVersion { found: u32, supported: u32 },
    #[error(transparent)]
    BoundedText(#[from] crate::BoundedTextError),
    #[error("failed to serialize canonical revocation object: {0}")]
    Serialization(serde_json::Error),
}
