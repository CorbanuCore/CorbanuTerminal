#![allow(dead_code)]

//! Authenticated, secret-free client for the isolated credential broker.

use codex_secret_broker::BrokerBinding;
use codex_secret_broker::BrokerChannelMac;
use codex_secret_broker::BrokerDispatchError;
use codex_secret_broker::BrokerOperation;
use codex_secret_broker::CredentialReference;
use codex_secret_broker::OpenAiResponsesOperation;
use codex_secret_broker::SignedBrokerFrame;
use codex_secret_broker::TypedOperationReceipt;
use std::fmt;
use std::sync::Arc;
use std::sync::Mutex;
use thiserror::Error;

/// Qualified OS-service transport. The server side derives peer identity from
/// the connection; no caller-supplied peer reaches this boundary.
pub(crate) trait BrokerClientTransport: Send + Sync + 'static {
    fn dispatch(
        &self,
        frame: &SignedBrokerFrame,
    ) -> Result<TypedOperationReceipt, BrokerDispatchError>;

    fn close(&self, binding: &BrokerBinding) -> Result<(), BrokerDispatchError>;
}

struct ClientState {
    next_sequence: u64,
    closed: bool,
    dispatch_in_flight: bool,
}

/// One generation-bound broker channel. It contains no raw credential value.
pub(crate) struct IsolatedBrokerClient {
    binding: BrokerBinding,
    credential: CredentialReference,
    channel_mac: BrokerChannelMac,
    transport: Arc<dyn BrokerClientTransport>,
    state: Mutex<ClientState>,
}

impl IsolatedBrokerClient {
    pub(crate) fn new(
        binding: BrokerBinding,
        credential: CredentialReference,
        channel_mac: BrokerChannelMac,
        transport: Arc<dyn BrokerClientTransport>,
    ) -> Result<Self, BrokerClientError> {
        binding
            .validate()
            .map_err(|_| BrokerClientError::InvalidBinding)?;
        Ok(Self {
            binding,
            credential,
            channel_mac,
            transport,
            state: Mutex::new(ClientState {
                next_sequence: 1,
                closed: false,
                dispatch_in_flight: false,
            }),
        })
    }

    pub(crate) fn dispatch_openai_responses(
        &self,
        path: &str,
    ) -> Result<TypedOperationReceipt, BrokerClientError> {
        let request = OpenAiResponsesOperation::new(path)
            .map_err(|_| BrokerClientError::UnsupportedOperation)?;
        let (sequence, next_sequence) = {
            let mut state = self.state.lock().map_err(|_| BrokerClientError::State)?;
            if state.closed {
                return Err(BrokerClientError::Closed);
            }
            if state.dispatch_in_flight {
                return Err(BrokerClientError::Unavailable);
            }
            let sequence = state.next_sequence;
            let next_sequence = sequence
                .checked_add(1)
                .ok_or(BrokerClientError::SequenceExhausted)?;
            state.dispatch_in_flight = true;
            (sequence, next_sequence)
        };
        let frame = match self.channel_mac.sign(
            self.binding.clone(),
            sequence,
            BrokerOperation::OpenAiResponses {
                credential: self.credential.clone(),
                request,
            },
        ) {
            Ok(frame) => frame,
            Err(_) => {
                if let Ok(mut state) = self.state.lock() {
                    state.dispatch_in_flight = false;
                    state.closed = true;
                }
                let _ = self.transport.close(&self.binding);
                return Err(BrokerClientError::Authentication);
            }
        };
        let dispatch_result = self.transport.dispatch(&frame);
        let (result, close_transport) = {
            let mut state = match self.state.lock() {
                Ok(state) => state,
                Err(_) => {
                    let _ = self.transport.close(&self.binding);
                    return Err(BrokerClientError::State);
                }
            };
            state.dispatch_in_flight = false;
            match dispatch_result {
                Ok(receipt) => {
                    state.next_sequence = next_sequence;
                    (Ok(receipt), false)
                }
                Err(error) => {
                    // A transport error cannot prove whether the broker consumed
                    // this sequence. Close the generation-bound channel instead
                    // of advancing or retrying an ambiguous frame.
                    let close_transport = !state.closed;
                    state.closed = true;
                    (Err(map_broker_error(error)), close_transport)
                }
            }
        };
        if close_transport {
            let _ = self.transport.close(&self.binding);
        }
        result
    }

    pub(crate) fn close(&self) -> Result<(), BrokerClientError> {
        {
            let mut state = self.state.lock().map_err(|_| BrokerClientError::State)?;
            if state.closed {
                return Ok(());
            }
            state.closed = true;
        }
        self.transport
            .close(&self.binding)
            .map_err(map_broker_error)
    }
}

impl fmt::Debug for IsolatedBrokerClient {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("IsolatedBrokerClient(<authenticated>)")
    }
}

impl Drop for IsolatedBrokerClient {
    fn drop(&mut self) {
        let _ = self.close();
    }
}

#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub(crate) enum BrokerClientError {
    #[error("broker client binding is invalid")]
    InvalidBinding,
    #[error("broker client operation is unsupported")]
    UnsupportedOperation,
    #[error("broker client frame authentication failed")]
    Authentication,
    #[error("broker client sequence is exhausted")]
    SequenceExhausted,
    #[error("broker client is closed")]
    Closed,
    #[error("broker client state is unavailable")]
    State,
    #[error("isolated broker is unavailable")]
    Unavailable,
    #[error("isolated broker denied the request")]
    Denied,
    #[error("isolated broker cancelled the request")]
    Cancelled,
    #[error("isolated broker outcome is unknown")]
    OutcomeUnknown,
}

fn map_broker_error(error: BrokerDispatchError) -> BrokerClientError {
    match error {
        BrokerDispatchError::Cancelled => BrokerClientError::Cancelled,
        BrokerDispatchError::OutcomeUnknown | BrokerDispatchError::AuditCommitUnknown => {
            BrokerClientError::OutcomeUnknown
        }
        BrokerDispatchError::WrongPeer
        | BrokerDispatchError::BindingMismatch
        | BrokerDispatchError::ReplayOrSequenceGap
        | BrokerDispatchError::CredentialUnavailable
        | BrokerDispatchError::CredentialExpired
        | BrokerDispatchError::StaleRunGeneration => BrokerClientError::Denied,
        _ => BrokerClientError::Unavailable,
    }
}

#[cfg(test)]
#[path = "broker_client_tests.rs"]
mod tests;
