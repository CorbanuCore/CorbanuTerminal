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
use codex_security_policy::CredentialCapabilityRequest;
use codex_security_policy::CredentialTransport;
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
        if request.capability_id != self.credential.capability_id()
            || request.authority != &self.authority
            || request.scheme != "https"
            || self.authority.destination.transport != CredentialTransport::Https
            || request.host != self.authority.destination.host.as_str()
            || request.port != self.authority.destination.port
            || request.method != self.authority.method.as_str()
            || request.path != self.authority.path.as_str()
        {
            return Err(ScopedCredentialResolverError::Denied);
        }

        let now = self
            .clock
            .now_unix_seconds()
            .map_err(|_| ScopedCredentialResolverError::Unavailable)?;
        let revocations = self
            .revocations
            .read()
            .map_err(|_| ScopedCredentialResolverError::Unavailable)?
            .clone();
        self.vault
            .with_scoped_credential(&self.credential, now, &revocations, |secret| {
                callback(secret).map_err(|ProxyCredentialCallbackError::Failed| {
                    VaultCredentialCallbackError::Failed
                })
            })
            .map_err(map_vault_credential_error)
    }
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
