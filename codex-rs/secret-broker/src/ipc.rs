//! Bounded authenticated messages for the controller-to-broker channel.
//!
//! The wire format deliberately exposes no generic secret-resolution operation.
//! A transport must obtain [`ObservedPeer`] from the operating system rather
//! than trusting identity fields sent in the frame.

use hmac::Hmac;
use hmac::Mac;
use serde::Deserialize;
use serde::Serialize;
use sha2::Sha256;
use std::fmt;
use thiserror::Error;
use zeroize::Zeroize;

pub const IPC_PROTOCOL_VERSION: u32 = 1;
pub const MAX_FRAME_BYTES: usize = 16 * 1024;
const MAC_BYTES: usize = 32;
const MAX_ID_BYTES: usize = 128;
const MAX_PATH_BYTES: usize = 1_024;

type HmacSha256 = Hmac<Sha256>;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BrokerBinding {
    pub controller_instance: String,
    pub worker_instance: String,
    pub session_id: String,
    pub task_id: String,
    pub run_id: String,
    pub run_generation: u64,
}

impl BrokerBinding {
    pub fn validate(&self) -> Result<(), BrokerFrameError> {
        for value in [
            &self.controller_instance,
            &self.worker_instance,
            &self.session_id,
            &self.task_id,
            &self.run_id,
        ] {
            validate_id(value)?;
        }
        if self.run_generation == 0 {
            return Err(BrokerFrameError::InvalidRunGeneration);
        }
        Ok(())
    }
}

/// Peer identity observed by the trusted transport.
///
/// Callers must derive this from `SO_PEERCRED`, an authenticated XPC audit
/// token/code requirement, or a named-pipe client token. Frame payloads can
/// never construct or override it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObservedPeer {
    principal: String,
    process_id: u32,
}

impl ObservedPeer {
    pub fn from_os(
        principal: impl Into<String>,
        process_id: u32,
    ) -> Result<Self, BrokerFrameError> {
        let principal = principal.into();
        validate_id(&principal)?;
        if process_id == 0 {
            return Err(BrokerFrameError::InvalidPeer);
        }
        Ok(Self {
            principal,
            process_id,
        })
    }

    pub fn principal(&self) -> &str {
        &self.principal
    }

    pub fn process_id(&self) -> u32 {
        self.process_id
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CredentialReference(String);

impl CredentialReference {
    pub fn from_sha256_hex(value: impl Into<String>) -> Result<Self, BrokerFrameError> {
        let value = value.into();
        if value.len() != 64
            || !value
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(BrokerFrameError::InvalidCredentialReference);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    fn validate(&self) -> Result<(), BrokerFrameError> {
        if self.0.len() != 64
            || !self
                .0
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(BrokerFrameError::InvalidCredentialReference);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OpenAiResponsesOperation {
    path: String,
}

impl OpenAiResponsesOperation {
    /// Creates the only credential-bearing operation accepted by this slice.
    /// Scheme, host, port and method are fixed to PF-13's exact adapter.
    pub fn new(path: impl Into<String>) -> Result<Self, BrokerFrameError> {
        let operation = Self { path: path.into() };
        operation.validate()?;
        Ok(operation)
    }

    pub const fn scheme(&self) -> &'static str {
        "https"
    }

    pub const fn host(&self) -> &'static str {
        "api.openai.com"
    }

    pub const fn port(&self) -> u16 {
        443
    }

    pub const fn method(&self) -> &'static str {
        "POST"
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    fn validate(&self) -> Result<(), BrokerFrameError> {
        if self.path.len() > MAX_PATH_BYTES
            || !self.path.starts_with("/v1/")
            || !self.path.bytes().all(|byte| byte.is_ascii_graphic())
            || self.path.contains('\\')
            || self.path.contains("..")
            || self.path.contains('?')
            || self.path.contains('#')
        {
            return Err(BrokerFrameError::UnsupportedOperation);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case", deny_unknown_fields)]
pub enum BrokerOperation {
    OpenAiResponses {
        credential: CredentialReference,
        request: OpenAiResponsesOperation,
    },
}

impl BrokerOperation {
    pub(crate) fn credential(&self) -> &CredentialReference {
        match self {
            Self::OpenAiResponses { credential, .. } => credential,
        }
    }

    fn validate(&self) -> Result<(), BrokerFrameError> {
        match self {
            Self::OpenAiResponses {
                credential,
                request,
            } => {
                credential.validate()?;
                request.validate()
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct BrokerFramePayload {
    protocol_version: u32,
    sequence: u64,
    binding: BrokerBinding,
    operation: BrokerOperation,
}

/// Length-prefixed payload plus HMAC-SHA256 tag.
pub struct SignedBrokerFrame(Vec<u8>);

impl SignedBrokerFrame {
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, BrokerFrameError> {
        validate_wire_length(&bytes)?;
        Ok(Self(bytes))
    }
}

impl fmt::Debug for SignedBrokerFrame {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("SignedBrokerFrame(<authenticated>)")
    }
}

/// Channel authenticator installed independently in controller and broker.
///
/// The key is non-serializable, redacted in `Debug`, and zeroized on drop.
pub struct BrokerChannelMac([u8; MAC_BYTES]);

impl BrokerChannelMac {
    pub fn from_secret(secret: [u8; MAC_BYTES]) -> Self {
        Self(secret)
    }

    pub fn sign(
        &self,
        binding: BrokerBinding,
        sequence: u64,
        operation: BrokerOperation,
    ) -> Result<SignedBrokerFrame, BrokerFrameError> {
        binding.validate()?;
        operation.validate()?;
        if sequence == 0 {
            return Err(BrokerFrameError::InvalidSequence);
        }
        let payload = BrokerFramePayload {
            protocol_version: IPC_PROTOCOL_VERSION,
            sequence,
            binding,
            operation,
        };
        let payload = serde_json::to_vec(&payload).map_err(|_| BrokerFrameError::MalformedFrame)?;
        if payload.len() > MAX_FRAME_BYTES - MAC_BYTES - size_of::<u32>() {
            return Err(BrokerFrameError::FrameTooLarge);
        }
        let payload_len =
            u32::try_from(payload.len()).map_err(|_| BrokerFrameError::FrameTooLarge)?;
        let mut mac = HmacSha256::new_from_slice(&self.0)
            .map_err(|_| BrokerFrameError::AuthenticationFailed)?;
        mac.update(&payload_len.to_be_bytes());
        mac.update(&payload);
        let tag = mac.finalize().into_bytes();
        let mut wire = Vec::with_capacity(size_of::<u32>() + payload.len() + MAC_BYTES);
        wire.extend_from_slice(&payload_len.to_be_bytes());
        wire.extend_from_slice(&payload);
        wire.extend_from_slice(&tag);
        Ok(SignedBrokerFrame(wire))
    }

    pub(crate) fn verify(
        &self,
        frame: &SignedBrokerFrame,
    ) -> Result<VerifiedBrokerRequest, BrokerFrameError> {
        validate_wire_length(frame.as_bytes())?;
        let (length, rest) = frame.as_bytes().split_at(size_of::<u32>());
        let payload_len = u32::from_be_bytes(
            length
                .try_into()
                .map_err(|_| BrokerFrameError::MalformedFrame)?,
        ) as usize;
        let (payload, tag) = rest.split_at(payload_len);
        let mut mac = HmacSha256::new_from_slice(&self.0)
            .map_err(|_| BrokerFrameError::AuthenticationFailed)?;
        mac.update(length);
        mac.update(payload);
        mac.verify_slice(tag)
            .map_err(|_| BrokerFrameError::AuthenticationFailed)?;
        let payload: BrokerFramePayload =
            serde_json::from_slice(payload).map_err(|_| BrokerFrameError::MalformedFrame)?;
        if payload.protocol_version != IPC_PROTOCOL_VERSION {
            return Err(BrokerFrameError::UnsupportedProtocol);
        }
        payload.binding.validate()?;
        payload.operation.validate()?;
        if payload.sequence == 0 {
            return Err(BrokerFrameError::InvalidSequence);
        }
        Ok(VerifiedBrokerRequest {
            sequence: payload.sequence,
            binding: payload.binding,
            operation: payload.operation,
        })
    }
}

impl fmt::Debug for BrokerChannelMac {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("BrokerChannelMac(<redacted>)")
    }
}

impl Drop for BrokerChannelMac {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

pub(crate) struct VerifiedBrokerRequest {
    pub(crate) sequence: u64,
    pub(crate) binding: BrokerBinding,
    pub(crate) operation: BrokerOperation,
}

fn validate_wire_length(bytes: &[u8]) -> Result<(), BrokerFrameError> {
    if bytes.len() > MAX_FRAME_BYTES {
        return Err(BrokerFrameError::FrameTooLarge);
    }
    if bytes.len() < size_of::<u32>() + MAC_BYTES {
        return Err(BrokerFrameError::MalformedFrame);
    }
    let payload_len = u32::from_be_bytes(
        bytes[..size_of::<u32>()]
            .try_into()
            .map_err(|_| BrokerFrameError::MalformedFrame)?,
    ) as usize;
    if payload_len != bytes.len() - size_of::<u32>() - MAC_BYTES {
        return Err(BrokerFrameError::MalformedFrame);
    }
    Ok(())
}

fn validate_id(value: &str) -> Result<(), BrokerFrameError> {
    if value.is_empty()
        || value.len() > MAX_ID_BYTES
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b':' | b'-'))
    {
        return Err(BrokerFrameError::InvalidIdentity);
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum BrokerFrameError {
    #[error("broker frame exceeds the bounded size")]
    FrameTooLarge,
    #[error("broker frame is malformed")]
    MalformedFrame,
    #[error("broker frame authentication failed")]
    AuthenticationFailed,
    #[error("broker IPC protocol is unsupported")]
    UnsupportedProtocol,
    #[error("broker identity is invalid")]
    InvalidIdentity,
    #[error("broker run generation must be nonzero")]
    InvalidRunGeneration,
    #[error("broker peer identity is invalid")]
    InvalidPeer,
    #[error("broker sequence must be nonzero")]
    InvalidSequence,
    #[error("credential reference must be a canonical SHA-256 identifier")]
    InvalidCredentialReference,
    #[error("credential operation is unsupported")]
    UnsupportedOperation,
}
