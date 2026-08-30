use codex_security_policy::ActionReceipt;
use codex_security_policy::ActorChain;
use codex_security_policy::AuthorizationDecision;
use codex_security_policy::AuthorizationRequest;
use codex_security_policy::BoundedGrant;
use codex_security_policy::BoundedText;
use codex_security_policy::MandateOutcome;
use codex_security_policy::PolicyAction;
use codex_security_policy::PolicyPrincipal;
use codex_security_policy::ProtectedActionMandate;
use codex_security_policy::ProtectedResource;
use codex_security_policy::RevocationEvent;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;

use crate::ActionId;
use crate::DecisionId;
use crate::ReservationId;
use crate::SECURITY_AUDIT_SCHEMA_VERSION;
use crate::SecurityEventError;
use crate::SecurityEventId;
use crate::event_identity::hash_value;
use crate::event_identity::is_lower_hex_sha256;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EventContext {
    pub producer: PolicyPrincipal,
    pub policy_generation: u64,
    pub run_generation: u64,
}

impl EventContext {
    pub fn new(
        producer: PolicyPrincipal,
        policy_generation: u64,
        run_generation: u64,
    ) -> Result<Self, SecurityEventError> {
        if run_generation == 0 {
            return Err(SecurityEventError::InvalidRunGeneration);
        }
        Ok(Self {
            producer,
            policy_generation,
            run_generation,
        })
    }
}

/// Minimal identity retained from a PF-16 authorization request.
///
/// Purpose, operation, destination, quantities and other request values are not
/// copied into the audit journal. Consumers retain the validated request and use
/// this digest-bound identity for correlation only.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RequestIdentity {
    pub request_digest: BoundedText,
    /// Digest of effect semantics that intentionally excludes wall-clock,
    /// session, task and grant-correlation fields so idempotent retries remain
    /// stable without persisting purpose/destination/quantity values.
    pub action_digest: BoundedText,
    pub actor_chain: ActorChain,
    pub resource: ProtectedResource,
    pub action: PolicyAction,
    pub session_id: BoundedText,
    pub task_id: BoundedText,
}

impl RequestIdentity {
    pub fn from_request(request: &AuthorizationRequest) -> Result<Self, SecurityEventError> {
        request
            .validate()
            .map_err(|_| SecurityEventError::InvalidRequest)?;
        Ok(Self {
            request_digest: BoundedText::new(
                request
                    .digest()
                    .map_err(|_| SecurityEventError::InvalidRequest)?,
            )?,
            action_digest: BoundedText::new(hash_value(&(
                "action-semantics-v1",
                &request.subject,
                &request.resource,
                request.action,
                &request.context.purpose,
                &request.context.operation,
                &request.context.destination,
                &request.context.quantity,
            ))?)?,
            actor_chain: request.subject.clone(),
            resource: request.resource.clone(),
            action: request.action,
            session_id: request.context.session_id.clone(),
            task_id: request.context.task_id.clone(),
        })
    }

    fn validate(&self) -> Result<(), SecurityEventError> {
        if !is_lower_hex_sha256(self.request_digest.as_str())
            || !is_lower_hex_sha256(self.action_digest.as_str())
        {
            return Err(SecurityEventError::InvalidRequest);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
/// A correlation identity for the authority checked by the caller.
///
/// This serialized value is never authority itself. [`Self::from_grant`] and
/// [`Self::from_mandate`] validate the PF-17/PF-18 source shape. The public
/// variants also support fixture and recovered correlation IDs, but constructing
/// one does not validate authority. An effectful consumer must revalidate the
/// live grant or mandate and PF-19/PF-20 state immediately before dispatch.
pub enum AuthorityIdentity {
    Grant { grant_id: BoundedText },
    Mandate { mandate_id: BoundedText },
}

impl AuthorityIdentity {
    pub fn from_grant(grant: &BoundedGrant) -> Result<Self, SecurityEventError> {
        grant
            .validate()
            .map_err(|_| SecurityEventError::InvalidAuthority)?;
        Ok(Self::Grant {
            grant_id: grant.grant_id.clone(),
        })
    }

    pub fn from_mandate(mandate: &ProtectedActionMandate) -> Result<Self, SecurityEventError> {
        mandate
            .validate()
            .map_err(|_| SecurityEventError::InvalidAuthority)?;
        Ok(Self::Mandate {
            mandate_id: mandate.mandate_id.clone(),
        })
    }

    pub fn id(&self) -> &BoundedText {
        match self {
            Self::Grant { grant_id } => grant_id,
            Self::Mandate { mandate_id } => mandate_id,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnknownOutcomeReason {
    DispatchTimeout,
    TransportLost,
    SettlementUncertain,
    PersistenceUncertain,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case", deny_unknown_fields)]
pub enum DispatchResolution {
    Completed {
        outcome: MandateOutcome,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        mandate_receipt: Option<ActionReceipt>,
    },
    Unknown {
        reason: UnknownOutcomeReason,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum SecurityEventKind {
    Decision {
        decision_id: DecisionId,
        request: RequestIdentity,
        decision: AuthorizationDecision,
    },
    DispatchIntent {
        action_id: ActionId,
        reservation_id: ReservationId,
        request: RequestIdentity,
        authority: AuthorityIdentity,
        deduplication_digest: BoundedText,
    },
    DispatchResolution {
        action_id: ActionId,
        reservation_id: ReservationId,
        resolution: DispatchResolution,
    },
    Restriction {
        event: RevocationEvent,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SecurityEvent {
    pub schema_version: u32,
    pub event_id: SecurityEventId,
    pub context: EventContext,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub causal_parent: Option<SecurityEventId>,
    pub occurred_at_unix_seconds: i64,
    pub kind: SecurityEventKind,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SecurityEventWire {
    schema_version: u32,
    event_id: SecurityEventId,
    context: EventContext,
    #[serde(default)]
    causal_parent: Option<SecurityEventId>,
    occurred_at_unix_seconds: i64,
    kind: SecurityEventKind,
}

impl SecurityEvent {
    pub fn decision(
        context: EventContext,
        causal_parent: Option<SecurityEventId>,
        request: &AuthorizationRequest,
        decision: AuthorizationDecision,
        occurred_at_unix_seconds: i64,
    ) -> Result<Self, SecurityEventError> {
        let request = RequestIdentity::from_request(request)?;
        if decision.request_digest != request.request_digest {
            return Err(SecurityEventError::DecisionRequestMismatch);
        }
        let decision_id = DecisionId::from_digest(hash_value(&(
            "decision-v1",
            &request.request_digest,
            &decision,
        ))?);
        Self::build(
            context,
            causal_parent,
            occurred_at_unix_seconds,
            SecurityEventKind::Decision {
                decision_id,
                request,
                decision,
            },
        )
    }

    pub fn dispatch_intent(
        context: EventContext,
        causal_parent: Option<SecurityEventId>,
        request: &AuthorizationRequest,
        authority: AuthorityIdentity,
        deduplication_key: BoundedText,
        occurred_at_unix_seconds: i64,
    ) -> Result<Self, SecurityEventError> {
        let request = RequestIdentity::from_request(request)?;
        // Authority correlation is deliberately not part of effect identity.
        // A retry must revalidate current authority, but a re-issued grant or
        // mandate must not escape the durable replay fence for the same effect.
        let action_id = ActionId::from_digest(hash_value(&("action-v3", &request.action_digest))?);
        let deduplication_digest =
            BoundedText::new(hash_value(&("deduplication-v1", &deduplication_key))?)?;
        let reservation_id = ReservationId::from_digest(hash_value(&(
            "reservation-v1",
            &action_id,
            &authority,
            &deduplication_digest,
            context.policy_generation,
            context.run_generation,
        ))?);
        Self::build(
            context,
            causal_parent,
            occurred_at_unix_seconds,
            SecurityEventKind::DispatchIntent {
                action_id,
                reservation_id,
                request,
                authority,
                deduplication_digest,
            },
        )
    }

    pub fn restriction(
        context: EventContext,
        causal_parent: Option<SecurityEventId>,
        event: RevocationEvent,
    ) -> Result<Self, SecurityEventError> {
        event
            .validate()
            .map_err(|_| SecurityEventError::InvalidRestriction)?;
        let occurred_at_unix_seconds = event.created_at_unix_seconds;
        Self::build(
            context,
            causal_parent,
            occurred_at_unix_seconds,
            SecurityEventKind::Restriction { event },
        )
    }

    pub(crate) fn dispatch_resolution(
        context: EventContext,
        intent_event_id: SecurityEventId,
        action_id: ActionId,
        reservation_id: ReservationId,
        resolution: DispatchResolution,
        occurred_at_unix_seconds: i64,
    ) -> Result<Self, SecurityEventError> {
        Self::build(
            context,
            Some(intent_event_id),
            occurred_at_unix_seconds,
            SecurityEventKind::DispatchResolution {
                action_id,
                reservation_id,
                resolution,
            },
        )
    }

    fn build(
        context: EventContext,
        causal_parent: Option<SecurityEventId>,
        occurred_at_unix_seconds: i64,
        kind: SecurityEventKind,
    ) -> Result<Self, SecurityEventError> {
        let mut event = Self {
            schema_version: SECURITY_AUDIT_SCHEMA_VERSION,
            event_id: SecurityEventId::from_digest("0".repeat(64)),
            context,
            causal_parent,
            occurred_at_unix_seconds,
            kind,
        };
        event.validate_fields()?;
        event.event_id = SecurityEventId::from_digest(event.expected_id()?);
        Ok(event)
    }

    pub fn validate(&self) -> Result<(), SecurityEventError> {
        self.validate_fields()?;
        if self.event_id.as_str() != self.expected_id()? {
            return Err(SecurityEventError::IntegrityMismatch);
        }
        Ok(())
    }

    fn validate_fields(&self) -> Result<(), SecurityEventError> {
        if self.schema_version != SECURITY_AUDIT_SCHEMA_VERSION {
            return Err(SecurityEventError::UnsupportedVersion {
                found: self.schema_version,
                supported: SECURITY_AUDIT_SCHEMA_VERSION,
            });
        }
        if self.context.run_generation == 0 {
            return Err(SecurityEventError::InvalidRunGeneration);
        }
        if self.occurred_at_unix_seconds < 0 {
            return Err(SecurityEventError::NegativeTimestamp);
        }
        match &self.kind {
            SecurityEventKind::Decision {
                decision_id,
                request,
                decision,
            } => {
                request.validate()?;
                if decision.request_digest != request.request_digest {
                    return Err(SecurityEventError::DecisionRequestMismatch);
                }
                let expected = hash_value(&("decision-v1", &request.request_digest, decision))?;
                if decision_id.as_str() != expected {
                    return Err(SecurityEventError::DecisionIntegrityMismatch);
                }
            }
            SecurityEventKind::DispatchIntent {
                action_id,
                reservation_id,
                request,
                authority,
                deduplication_digest,
            } => {
                request.validate()?;
                let expected_action = hash_value(&("action-v3", &request.action_digest))?;
                if action_id.as_str() != expected_action {
                    return Err(SecurityEventError::ActionIntegrityMismatch);
                }
                if !is_lower_hex_sha256(deduplication_digest.as_str()) {
                    return Err(SecurityEventError::DeduplicationIntegrityMismatch);
                }
                let expected_reservation = hash_value(&(
                    "reservation-v1",
                    action_id,
                    authority,
                    deduplication_digest,
                    self.context.policy_generation,
                    self.context.run_generation,
                ))?;
                if reservation_id.as_str() != expected_reservation {
                    return Err(SecurityEventError::ReservationIntegrityMismatch);
                }
            }
            SecurityEventKind::DispatchResolution { resolution, .. } => {
                if let DispatchResolution::Completed {
                    mandate_receipt: Some(receipt),
                    ..
                } = resolution
                    && receipt.completed_at_unix_seconds != self.occurred_at_unix_seconds
                {
                    return Err(SecurityEventError::ResolutionTimestampMismatch);
                }
            }
            SecurityEventKind::Restriction { event } => event
                .validate()
                .map_err(|_| SecurityEventError::InvalidRestriction)?,
        }
        Ok(())
    }

    fn expected_id(&self) -> Result<String, SecurityEventError> {
        hash_value(&(
            self.schema_version,
            &self.context,
            &self.causal_parent,
            self.occurred_at_unix_seconds,
            &self.kind,
        ))
    }
}

impl TryFrom<SecurityEventWire> for SecurityEvent {
    type Error = SecurityEventError;

    fn try_from(value: SecurityEventWire) -> Result<Self, Self::Error> {
        let event = Self {
            schema_version: value.schema_version,
            event_id: value.event_id,
            context: value.context,
            causal_parent: value.causal_parent,
            occurred_at_unix_seconds: value.occurred_at_unix_seconds,
            kind: value.kind,
        };
        event.validate()?;
        Ok(event)
    }
}

impl<'de> Deserialize<'de> for SecurityEvent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        SecurityEventWire::deserialize(deserializer)?
            .try_into()
            .map_err(serde::de::Error::custom)
    }
}
