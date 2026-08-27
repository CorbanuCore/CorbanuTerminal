//! Confirmation capability stays with the existing trusted controller. Neither a
//! deserialized request nor agent-readable policy view can mint this token.
#![allow(dead_code)]

use codex_protocol::security::SecurityControlAction;
use codex_protocol::security::SecurityControlRequest;
use codex_security_policy::AuthorityEpoch;
use codex_security_policy::PolicyPrincipal;

use super::EffectivePolicyState;
use super::SecurityPolicyError;
use super::TrustedSecurityController;

/// Non-serializable, single-consumption confirmation, not a bearer wire message.
pub(crate) struct ConfirmedSecurityRequest {
    request: SecurityControlRequest,
    authority: PolicyPrincipal,
}

impl TrustedSecurityController {
    /// Called only by the future trusted human-confirmation adapter, never by a
    /// model tool or by receipt of a wire request. Does not apply a policy change.
    pub(crate) fn confirm_security_request(
        &self,
        request: SecurityControlRequest,
        now_unix_seconds: i64,
    ) -> Result<ConfirmedSecurityRequest, SecurityPolicyError> {
        let guard = self.read_state()?;
        let state = guard
            .as_ref()
            .ok_or(SecurityPolicyError::RuntimeNotInitialized)?;
        check_epoch(state, &request)?;
        if now_unix_seconds < 0 {
            return Err(SecurityPolicyError::AuthorityMismatch);
        }
        if let SecurityControlAction::CreateGrant {
            actor_chain,
            scope,
            expires_at_unix_seconds,
        } = request.action()
        {
            let bound_actor = state.agents.values().any(|binding| {
                &binding.actor_chain == actor_chain
                    && binding.session_id == scope.context.session_id
                    && binding.task_id == scope.context.task_id
                    && !binding.force_deny
            });
            if !bound_actor
                || *expires_at_unix_seconds <= now_unix_seconds
                || state.persisted.revocations.kill_switch_active
            {
                return Err(SecurityPolicyError::AuthorityMismatch);
            }
        }
        Ok(ConfirmedSecurityRequest {
            request,
            authority: state.persisted.human_authority.clone(),
        })
    }

    /// Recheck when consuming; callers must still apply mutation atomically with
    /// the same expected epoch in PF-23/25. This seam is not an execution permit.
    pub(crate) fn consume_security_confirmation(
        &self,
        confirmed: ConfirmedSecurityRequest,
    ) -> Result<SecurityControlRequest, SecurityPolicyError> {
        let guard = self.read_state()?;
        let state = guard
            .as_ref()
            .ok_or(SecurityPolicyError::RuntimeNotInitialized)?;
        check_epoch(state, &confirmed.request)?;
        if state.persisted.human_authority != confirmed.authority {
            return Err(SecurityPolicyError::AuthorityMismatch);
        }
        Ok(confirmed.request)
    }
}

fn check_epoch(
    state: &EffectivePolicyState,
    request: &SecurityControlRequest,
) -> Result<(), SecurityPolicyError> {
    let epoch = AuthorityEpoch::new(
        state.runtime_nonce,
        state.epoch,
        state.persisted.revocations.generation,
    )
    .map_err(|_| SecurityPolicyError::AuthorityMismatch)?;
    if request.expected_epoch() != epoch {
        return Err(SecurityPolicyError::AuthorityMismatch);
    }
    Ok(())
}
