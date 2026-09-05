use codex_secret_broker::BrokerBinding;
use codex_secret_broker::BrokerChannelMac;
use codex_secret_broker::BrokerCredentialGrant;
use codex_secret_broker::BrokerDispatchError;
use codex_secret_broker::BrokerRuntime;
use codex_secret_broker::BrokerRuntimeConfig;
use codex_secret_broker::ObservedPeer;
use codex_secret_broker::journal_adapter::BrokerJournalClock;
use codex_secret_broker::journal_adapter::JournalBrokerAudit;
use codex_secret_broker::linux_transport::LinuxBrokerSession;
use codex_secret_broker::linux_transport::observed_peer;
use codex_secret_broker::linux_transport::serve_connection;
use codex_secret_broker::platform_contract::ProtectedModeAuthorization;
use codex_vault::VaultBrokerBackend;
use codex_vault::VaultBrokerClock;
use codex_vault::VaultBrokerTransport;
use std::os::unix::net::UnixStream;
use std::sync::Arc;

/// Process-owned session inputs, supplied only by trusted bootstrap. MAC key
/// material and native expected identity never come from a dispatch frame.
pub struct TrustedSession {
    pub socket: UnixStream,
    pub expected_peer: ObservedPeer,
    pub binding: BrokerBinding,
    pub channel_mac: BrokerChannelMac,
    pub credential_grants: Vec<BrokerCredentialGrant>,
}

/// Composes the actual typed Vault backend and durable PF-41 audit into the
/// broker runtime without letting the broker library depend back on Vault.
pub struct BrokerService<T, V, J> {
    runtime: Arc<BrokerRuntime<VaultBrokerBackend<T, V>, JournalBrokerAudit<J>>>,
}

impl<T: VaultBrokerTransport, V: VaultBrokerClock, J: BrokerJournalClock> BrokerService<T, V, J> {
    pub fn new(
        broker_instance: String,
        authorization: ProtectedModeAuthorization,
        backend: VaultBrokerBackend<T, V>,
        audit: JournalBrokerAudit<J>,
    ) -> Result<Self, BrokerDispatchError> {
        Ok(Self {
            runtime: Arc::new(BrokerRuntime::new(
                broker_instance,
                BrokerRuntimeConfig::default(),
                authorization,
                backend,
                audit,
            )?),
        })
    }

    /// Consumes a trusted socket handle. Peer mismatch fails before registration
    /// or Vault access. Channel exit always cancels that registered generation.
    pub fn serve(&self, session: TrustedSession) -> Result<(), BrokerDispatchError> {
        if observed_peer(&session.socket)? != session.expected_peer {
            return Err(BrokerDispatchError::SessionUnavailable);
        }
        let handle = self.runtime.register_session(
            session.binding,
            session.expected_peer.clone(),
            session.channel_mac,
            session.credential_grants,
        )?;
        let handler = LinuxBrokerSession::new(self.runtime.clone(), handle);
        serve_connection(session.socket, &session.expected_peer, &handler)
    }

    /// Trusted control plane only. No wire command can acquire this operation.
    pub fn revoke_run(&self, controller: &str, run: &str) -> Result<(), BrokerDispatchError> {
        self.runtime.revoke_run(controller, run)
    }
}
