//! Versioned, bounded control messages between Core and the credential broker.
//!
//! The broker prints one bootstrap line (its control socket path) on stdout;
//! the control channel is that Unix socket, accepted only from the spawning
//! controller's pid, and the controller checks the socket's peer is the
//! broker it spawned (PF-27-S02). Raw values move one way (controller to
//! broker) and are never echoed back.

use super::super::providers::CredentialHostBinding;
use serde::Deserialize;
use serde::Serialize;
use std::fmt;
use zeroize::Zeroize;

pub(crate) const CONTROL_PROTOCOL_VERSION: u32 = 2;
/// Environment variable naming the broker's runtime parent directory.
pub(crate) const BROKER_RUNTIME_DIR_ENV: &str = "CODEX_CREDENTIAL_BROKER_RUNTIME_DIR";
pub(crate) const MAX_CONTROL_LINE_BYTES: usize = 16 * 1024;
pub(crate) const MAX_CREDENTIAL_VALUE_BYTES: usize = 8 * 1024;
pub(crate) const MAX_BINDING_ENTRIES: usize = 16;
pub(crate) const MAX_HOST_BYTES: usize = 253;
pub(crate) const MAX_ID_BYTES: usize = 128;
pub(crate) const FRAME_HEADER: &str = "x-corbanu-broker-frame";
pub(crate) const BROKER_ERROR_HEADER: &str = "x-corbanu-broker-error";
pub(crate) const BROKER_SESSION_ID: &str = "network-proxy";
pub(crate) const BROKER_TASK_ID: &str = "provider-credential";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ProviderId {
    Github,
    Openai,
}

/// Owned host binding enforced by the broker before every substitution.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct HostBindingWire {
    pub(crate) exact_hosts: Vec<String>,
    pub(crate) suffixes: Vec<String>,
}

impl HostBindingWire {
    pub(in crate::credential_broker) fn from_binding(binding: &CredentialHostBinding) -> Self {
        match binding {
            CredentialHostBinding::ExactHost(host) => Self {
                exact_hosts: vec![host.clone()],
                suffixes: Vec::new(),
            },
            CredentialHostBinding::HostPattern {
                exact_hosts,
                suffixes,
            } => Self {
                exact_hosts: exact_hosts.iter().map(ToString::to_string).collect(),
                suffixes: suffixes.iter().map(ToString::to_string).collect(),
            },
        }
    }

    pub(crate) fn validate(&self) -> bool {
        let entries = self.exact_hosts.len() + self.suffixes.len();
        entries > 0
            && entries <= MAX_BINDING_ENTRIES
            && self.exact_hosts.iter().all(|host| valid_host(host))
            && self
                .suffixes
                .iter()
                .all(|suffix| suffix.starts_with('.') && valid_host(&suffix[1..]))
    }

    pub(crate) fn matches_host(&self, host: &str) -> bool {
        self.exact_hosts.iter().any(|exact| exact == host)
            || self.suffixes.iter().any(|suffix| host.ends_with(suffix))
    }
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum ControlRequest {
    Hello {
        protocol_version: u32,
        controller_pid: u32,
        controller_instance: String,
        channel_key: String,
        allow_local_binding: bool,
        allow_upstream_proxy: bool,
        /// PF-28-S02: scrub registered values from returned responses.
        #[serde(default)]
        scrub_responses: bool,
        /// PF-33-S02: refuse provider requests that carry no checked DNS
        /// answers; pinned requests never resolve their host in the broker.
        #[serde(default)]
        pin_connections: bool,
    },
    Register {
        provider: ProviderId,
        binding: HostBindingWire,
        value: String,
    },
    Revoke,
}

impl fmt::Debug for ControlRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Hello { .. } => formatter.write_str("ControlRequest::Hello(<redacted>)"),
            Self::Register { provider, .. } => {
                write!(
                    formatter,
                    "ControlRequest::Register({provider:?}, <redacted>)"
                )
            }
            Self::Revoke => formatter.write_str("ControlRequest::Revoke"),
        }
    }
}

impl Drop for ControlRequest {
    fn drop(&mut self) {
        match self {
            Self::Hello { channel_key, .. } => channel_key.zeroize(),
            Self::Register { value, .. } => value.zeroize(),
            Self::Revoke => {}
        }
    }
}

/// The broker's only stdout message: where to connect for control.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct BrokerBootstrap {
    pub(crate) protocol_version: u32,
    pub(crate) control_socket: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum ControlResponse {
    Ready {
        protocol_version: u32,
        broker_instance: String,
        socket_path: String,
        run_generation: u64,
        /// OS containment the broker applied to itself, e.g. `seatbelt`.
        containment: String,
    },
    Registered {
        reference: String,
        run_generation: u64,
    },
    Revoked {
        run_generation: u64,
    },
    Error {
        code: ControlErrorCode,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ControlErrorCode {
    Malformed,
    UnsupportedProtocol,
    InvalidCredential,
    CapacityReached,
    Unavailable,
}

pub(crate) fn valid_host(host: &str) -> bool {
    !host.is_empty()
        && host.len() <= MAX_HOST_BYTES
        && host.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'-')
        })
}

pub(crate) fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_ID_BYTES
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b':' | b'-'))
}

pub(crate) fn encode_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(char::from(HEX[usize::from(byte >> 4)]));
        encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    encoded
}

pub(crate) fn decode_key(value: &str) -> Option<[u8; 32]> {
    if value.len() != 64 {
        return None;
    }
    let mut key = [0_u8; 32];
    for (index, chunk) in value.as_bytes().chunks_exact(2).enumerate() {
        let high = hex_value(chunk[0])?;
        let low = hex_value(chunk[1])?;
        key[index] = (high << 4) | low;
    }
    Some(key)
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        _ => None,
    }
}
