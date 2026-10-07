//! PF-27-S05: Core's own model-provider keys held by the isolated broker.
//!
//! Core registers a key once and keeps only a [`ModelCredential`]: an opaque
//! reference plus the channel to sign requests for it. Each model request is
//! sent over the broker's private Unix socket with a signed frame naming its
//! exact origin, method and path; the broker attaches the key and performs the
//! HTTPS request. There is no API that returns the key.

use super::isolated::IsolatedBrokerClient;
use super::isolated::IsolatedBrokerError;
use super::isolated::IsolatedBrokerLauncher;
use super::isolated::IsolatedBrokerOptions;
use super::isolated::protocol::FRAME_HEADER;
pub use super::isolated::protocol::ModelAuthHeader;
use super::isolated::protocol::ModelBindingWire;
use codex_secret_broker::CredentialReference;
use codex_secret_broker::ProviderRequestOperation;
use std::fmt;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use thiserror::Error;

/// Request header carrying the signed broker frame.
pub const MODEL_BROKER_FRAME_HEADER: &str = FRAME_HEADER;

#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum ModelCredentialBrokerError {
    #[error("credential broker is unavailable")]
    Unavailable,
    #[error("credential broker rejected the credential or request")]
    Rejected,
}

impl From<IsolatedBrokerError> for ModelCredentialBrokerError {
    fn from(error: IsolatedBrokerError) -> Self {
        match error {
            IsolatedBrokerError::Rejected | IsolatedBrokerError::Unpinned => Self::Rejected,
            IsolatedBrokerError::Spawn
            | IsolatedBrokerError::Control
            | IsolatedBrokerError::Unavailable => Self::Unavailable,
        }
    }
}

/// Where a model-provider key may be sent: one HTTPS origin and a path prefix.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelCredentialBinding {
    pub host: String,
    pub port: u16,
    /// `/` or a plain path without a trailing slash, e.g. `/v1`.
    pub path_prefix: String,
    pub header: ModelAuthHeader,
}

impl ModelCredentialBinding {
    fn wire(&self) -> ModelBindingWire {
        ModelBindingWire {
            host: self.host.clone(),
            port: self.port,
            path_prefix: self.path_prefix.clone(),
            header: self.header,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct ModelCredentialBrokerOptions {
    /// Parent of the broker's private socket directory (`CODEX_HOME/run`).
    pub runtime_dir: Option<PathBuf>,
    /// Scrub the key from responses (PF-28-S02).
    pub scrub_responses: bool,
}

/// A broker process holding Core's model-provider keys.
#[derive(Clone)]
pub struct ModelCredentialBroker {
    client: Arc<IsolatedBrokerClient>,
}

impl fmt::Debug for ModelCredentialBroker {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ModelCredentialBroker(<isolated>)")
    }
}

impl ModelCredentialBroker {
    /// Starts a contained broker by re-executing the current binary. A broker
    /// that cannot confine itself is refused.
    pub fn spawn(
        options: ModelCredentialBrokerOptions,
    ) -> Result<Self, ModelCredentialBrokerError> {
        Self::spawn_with_launcher(options, &IsolatedBrokerLauncher::current_exe())
    }

    pub(crate) fn spawn_with_launcher(
        options: ModelCredentialBrokerOptions,
        launcher: &IsolatedBrokerLauncher,
    ) -> Result<Self, ModelCredentialBrokerError> {
        let client = IsolatedBrokerClient::spawn(
            launcher,
            IsolatedBrokerOptions {
                // Core itself may use private-network or proxied providers.
                allow_local_binding: true,
                allow_upstream_proxy: true,
                runtime_dir: options.runtime_dir,
                require_containment: true,
                scrub_responses: options.scrub_responses,
                pin_connections: false,
            },
        )?;
        Ok(Self {
            client: Arc::new(client),
        })
    }

    /// Hands `value` to the broker. The returned credential holds no copy.
    pub fn register(
        &self,
        binding: ModelCredentialBinding,
        value: &str,
    ) -> Result<ModelCredential, ModelCredentialBrokerError> {
        if !self.client.is_alive() {
            return Err(ModelCredentialBrokerError::Unavailable);
        }
        let reference = self.client.register_model(binding.wire(), value)?;
        Ok(ModelCredential {
            client: self.client.clone(),
            reference,
            binding,
        })
    }

    pub fn is_alive(&self) -> bool {
        self.client.is_alive()
    }

    #[cfg(test)]
    pub(crate) fn kill_for_test(&self) {
        self.client.kill_for_test();
    }
}

/// An opaque reference to a key held by the broker.
#[derive(Clone)]
pub struct ModelCredential {
    client: Arc<IsolatedBrokerClient>,
    reference: CredentialReference,
    binding: ModelCredentialBinding,
}

impl fmt::Debug for ModelCredential {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ModelCredential")
            .field("binding", &self.binding)
            .finish_non_exhaustive()
    }
}

impl ModelCredential {
    pub fn binding(&self) -> &ModelCredentialBinding {
        &self.binding
    }

    /// The broker's private socket. Requests go there as plain HTTP/1.1 with
    /// the origin-form path and a signed frame.
    pub fn socket_path(&self) -> &Path {
        self.client.socket_path()
    }

    pub fn is_alive(&self) -> bool {
        self.client.is_alive()
    }

    /// Signs one request (`path_and_query` in origin form) and returns the
    /// value of [`MODEL_BROKER_FRAME_HEADER`]. Each frame is single-use.
    pub fn sign(
        &self,
        method: &str,
        host: &str,
        port: u16,
        path_and_query: &str,
    ) -> Result<String, ModelCredentialBrokerError> {
        if !self.client.is_alive() {
            return Err(ModelCredentialBrokerError::Unavailable);
        }
        if !self.binding.wire().allows(host, port, path_and_query) {
            return Err(ModelCredentialBrokerError::Rejected);
        }
        let operation = ProviderRequestOperation::new(host, port, method, path_and_query)
            .map_err(|_| ModelCredentialBrokerError::Rejected)?;
        Ok(self.client.sign_frame(&self.reference, &operation)?)
    }

    /// Signs without the local binding check, to qualify the broker's own.
    #[cfg(test)]
    pub(crate) fn sign_unchecked_for_test(
        &self,
        method: &str,
        host: &str,
        port: u16,
        path_and_query: &str,
    ) -> String {
        let operation = ProviderRequestOperation::new(host, port, method, path_and_query)
            .expect("test operation");
        self.client
            .sign_frame(&self.reference, &operation)
            .expect("test frame")
    }
}
