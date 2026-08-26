use super::NetworkProxySpec;
use crate::security::credential_capability::AuthorizedCredentialCapability;
use crate::security::credential_capability::CredentialClock;
use crate::security::credential_capability::SystemCredentialClock;
use codex_network_proxy::NetworkProxyAuditMetadata;
use codex_network_proxy::NetworkProxyState;
use codex_network_proxy::ScopedCredentialCallbackError as ProxyCredentialCallbackError;
use codex_network_proxy::ScopedCredentialResolver;
use codex_network_proxy::ScopedCredentialResolverError;
use codex_network_proxy::ScopedCredentialRoute;
use codex_network_proxy::ScopedCredentialUse;
use codex_security_policy::ActionReceipt;
use codex_security_policy::BoundedText;
use codex_security_policy::CredentialCapabilityRequest;
use codex_security_policy::CredentialTransport;
use codex_security_policy::DecisionReason;
use codex_security_policy::MandateOutcome;
use codex_security_policy::RevocationState;
use codex_vault::ScopedCredentialCallbackError as VaultCredentialCallbackError;
use codex_vault::ScopedCredentialError as VaultCredentialError;
use codex_vault::Vault;
use codex_vault::VaultCredentialRef;
use std::sync::Arc;
use std::sync::RwLock;

struct VaultNetworkCredentialResolver<C> {
    vault: Arc<Vault>,
    credential: VaultCredentialRef,
    authority: CredentialCapabilityRequest,
    revocations: Arc<RwLock<RevocationState>>,
    clock: C,
}

impl<C> ScopedCredentialResolver for VaultNetworkCredentialResolver<C>
where
    C: CredentialClock + 'static,
{
    fn resolve(
        &self,
        request: &ScopedCredentialUse<'_>,
        callback: &mut dyn FnMut(&str) -> Result<(), ProxyCredentialCallbackError>,
    ) -> Result<(), ScopedCredentialResolverError> {
        let now = self
            .clock
            .now_unix_seconds()
            .map_err(|_| ScopedCredentialResolverError::Unavailable)?;
        if request.capability_id != self.credential.capability_id()
            || request.authority != &self.authority
            || request.scheme != "https"
            || self.authority.destination.transport != CredentialTransport::Https
            || request.host != self.authority.destination.host.as_str()
            || request.port != self.authority.destination.port
            || request.method != self.authority.method.as_str()
            || request.path != self.authority.path.as_str()
        {
            self.emit_receipt(now, MandateOutcome::Denied, DecisionReason::ScopeMismatch)?;
            return Err(ScopedCredentialResolverError::Denied);
        }

        // Hold the current-revocation read guard through the trusted callback.
        // A concurrent revocation therefore linearizes either before resolution
        // (and denies it) or after this already-started one-shot use completes.
        let result = with_current_revocations(&self.revocations, |revocations| {
            self.vault
                .with_scoped_credential(&self.credential, now, revocations, |secret| {
                    callback(secret).map_err(|ProxyCredentialCallbackError::Failed| {
                        VaultCredentialCallbackError::Failed
                    })
                })
        })?;
        let (outcome, reason) = receipt_outcome(result);
        self.emit_receipt(now, outcome, reason)?;
        result.map_err(map_vault_credential_error)
    }
}

impl<C> VaultNetworkCredentialResolver<C> {
    fn emit_receipt(
        &self,
        completed_at_unix_seconds: i64,
        outcome: MandateOutcome,
        policy_reason: DecisionReason,
    ) -> Result<(), ScopedCredentialResolverError> {
        let destination = self
            .authority
            .destination
            .authority()
            .map_err(|_| ScopedCredentialResolverError::Unavailable)?;
        let authority_digest = BoundedText::new(
            self.authority
                .digest()
                .map_err(|_| ScopedCredentialResolverError::Unavailable)?,
        )
        .map_err(|_| ScopedCredentialResolverError::Unavailable)?;
        let receipt = ActionReceipt::complete_credential_use(
            self.credential.capability_id().clone(),
            policy_reason,
            self.authority.authorization.context.operation.clone(),
            destination,
            authority_digest,
            outcome,
            completed_at_unix_seconds,
        )
        .map_err(|_| ScopedCredentialResolverError::Unavailable)?;
        let metadata = receipt
            .credential_use
            .as_ref()
            .ok_or(ScopedCredentialResolverError::Unavailable)?;
        tracing::info!(
            receipt_id = %receipt.receipt_id,
            capability_id = %metadata.capability_id.as_str(),
            policy_reason = ?metadata.policy_reason,
            operation = %metadata.operation,
            destination = %metadata.destination,
            outcome = ?receipt.outcome,
            "scoped credential action receipt"
        );
        Ok(())
    }
}

fn receipt_outcome(result: Result<(), VaultCredentialError>) -> (MandateOutcome, DecisionReason) {
    match result {
        Ok(()) => (MandateOutcome::Executed, DecisionReason::MatchingGrant),
        Err(VaultCredentialError::Expired) => {
            (MandateOutcome::Denied, DecisionReason::ExpiredGrant)
        }
        Err(VaultCredentialError::Revoked) => (MandateOutcome::Denied, DecisionReason::Revoked),
        Err(
            VaultCredentialError::InvalidCapability
            | VaultCredentialError::LabelMismatch
            | VaultCredentialError::ScopeMismatch,
        ) => (MandateOutcome::Denied, DecisionReason::ScopeMismatch),
        Err(VaultCredentialError::CallbackCancelled) => {
            (MandateOutcome::Cancelled, DecisionReason::MatchingGrant)
        }
        Err(
            VaultCredentialError::NotFound
            | VaultCredentialError::CredentialTypeDenied
            | VaultCredentialError::Storage
            | VaultCredentialError::CallbackFailed
            | VaultCredentialError::CallbackPanicked,
        ) => (MandateOutcome::Failed, DecisionReason::MatchingGrant),
    }
}

fn with_current_revocations<T>(
    revocations: &RwLock<RevocationState>,
    operation: impl FnOnce(&RevocationState) -> T,
) -> Result<T, ScopedCredentialResolverError> {
    let revocations = revocations
        .read()
        .map_err(|_| ScopedCredentialResolverError::Unavailable)?;
    Ok(operation(&revocations))
}

fn map_vault_credential_error(error: VaultCredentialError) -> ScopedCredentialResolverError {
    match error {
        VaultCredentialError::InvalidCapability
        | VaultCredentialError::LabelMismatch
        | VaultCredentialError::ScopeMismatch => ScopedCredentialResolverError::Denied,
        VaultCredentialError::Expired => ScopedCredentialResolverError::Expired,
        VaultCredentialError::Revoked => ScopedCredentialResolverError::Revoked,
        VaultCredentialError::NotFound
        | VaultCredentialError::CredentialTypeDenied
        | VaultCredentialError::Storage => ScopedCredentialResolverError::Unavailable,
        VaultCredentialError::CallbackFailed
        | VaultCredentialError::CallbackCancelled
        | VaultCredentialError::CallbackPanicked => ScopedCredentialResolverError::CallbackFailed,
    }
}

impl NetworkProxySpec {
    #[allow(dead_code)]
    pub(crate) fn build_state_with_scoped_openai_credential(
        &self,
        audit_metadata: NetworkProxyAuditMetadata,
        authorized: AuthorizedCredentialCapability,
        vault: Arc<Vault>,
        revocations: Arc<RwLock<RevocationState>>,
    ) -> std::io::Result<NetworkProxyState> {
        self.build_state_with_scoped_openai_credential_and_clock(
            audit_metadata,
            authorized,
            vault,
            revocations,
            SystemCredentialClock,
        )
    }

    pub(crate) fn build_state_with_scoped_openai_credential_and_clock<C>(
        &self,
        audit_metadata: NetworkProxyAuditMetadata,
        authorized: AuthorizedCredentialCapability,
        vault: Arc<Vault>,
        revocations: Arc<RwLock<RevocationState>>,
        clock: C,
    ) -> std::io::Result<NetworkProxyState>
    where
        C: CredentialClock + 'static,
    {
        let authority = authorized.request.clone();
        let credential = authorized.into_vault_ref().map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "scoped credential authority is invalid",
            )
        })?;
        let capability_id = credential.capability_id().clone();
        let resolver = Arc::new(VaultNetworkCredentialResolver {
            vault,
            credential,
            authority: authority.clone(),
            revocations,
            clock,
        });
        let route =
            ScopedCredentialRoute::new(capability_id, authority, resolver).map_err(|_| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "scoped OpenAI credential route is invalid",
                )
            })?;
        let state = self.build_state_with_audit_metadata(audit_metadata)?;
        state.install_scoped_credential_route(route).map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "scoped OpenAI credential route is unavailable",
            )
        })?;
        Ok(state)
    }
}

#[cfg(test)]
#[path = "network_proxy_credential_tests.rs"]
mod tests;
