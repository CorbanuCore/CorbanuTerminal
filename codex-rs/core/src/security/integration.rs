//! Thin adapters over PF-22's policy state; enforcement/ingress consumers follow
//! in their own sprints. No backend availability is inferred from a level.
#![allow(dead_code)]

use codex_protocol::security::SecurityInspectorEvent;
use codex_security_policy::ActionContext;
use codex_security_policy::ActionContextError;
use codex_security_policy::AuthorityEpoch;
use codex_security_policy::AuthorizationRequest;
use codex_security_policy::ProvenanceError;
use codex_security_policy::SecurityControlHealthSnapshot;
use codex_security_policy::SecurityInspectorSnapshot;
use codex_security_policy::SourceEnvelope;
use codex_security_policy::SourceId;
use codex_security_policy::SourceKind;
use codex_security_policy::TaintContext;
use uuid::Uuid;

use super::effective_policy::EffectivePolicySnapshot;
use super::effective_policy::SecurityPolicyError;

/// Source classification comes from the native adapter. Embedded labels, marker
/// strings, JSON, or a classifier verdict in `content` cannot assign identity.
pub(crate) fn capture_source(
    kind: SourceKind,
    content: &[u8],
) -> Result<SourceEnvelope, ProvenanceError> {
    let source_id = SourceId::try_from(*Uuid::new_v4().as_bytes())?;
    Ok(SourceEnvelope::host_assigned(source_id, kind, content))
}

impl EffectivePolicySnapshot {
    pub(crate) fn authority_epoch(&self) -> Result<AuthorityEpoch, ActionContextError> {
        AuthorityEpoch::new(self.runtime_nonce, self.epoch, self.revocation_generation)
    }

    pub(crate) fn unavailable_inspector(
        &self,
    ) -> Result<SecurityInspectorEvent, SecurityPolicyError> {
        Ok(SecurityInspectorEvent {
            snapshot: SecurityInspectorSnapshot::new(
                self.requested_level,
                self.level,
                SecurityControlHealthSnapshot::default(),
            )
            .map_err(|_| SecurityPolicyError::AuthorityMismatch)?,
            epoch: self
                .authority_epoch()
                .map_err(|_| SecurityPolicyError::AuthorityMismatch)?,
        })
    }

    pub(crate) fn bind_action(
        &self,
        request: AuthorizationRequest,
        taint: TaintContext,
    ) -> Result<ActionContext, ActionContextError> {
        if request.subject != self.actor_chain
            || request.context.session_id != self.session_id
            || request.context.task_id != self.task_id
        {
            return Err(ActionContextError::InvalidRequest);
        }
        if self.kill_switch_active {
            return Err(ActionContextError::Revoked);
        }
        ActionContext::new(request, taint, self.authority_epoch()?)
    }
}
