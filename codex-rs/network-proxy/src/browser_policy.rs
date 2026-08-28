//! Public-web-only connection policy for the networkless PF-30 worker.
//! Resolution returns the exact addresses a broker must connect to; validating
//! a hostname and then resolving it again at connect time is not sufficient.

use crate::NetworkDecision;
use crate::NetworkPolicyRequest;
use crate::NetworkPolicyRequestArgs;
use crate::NetworkProtocol;
use crate::NetworkProxyState;
use crate::network_policy::evaluate_host_policy;
use crate::policy::is_non_public_ip;
use std::net::IpAddr;
use std::net::SocketAddr;
use std::time::Duration;
use thiserror::Error;
use url::Host;
use url::Url;

const MAX_URL_BYTES: usize = 4096;
const MAX_ADDRESSES: usize = 16;

/// Host-validated destination. Fields cannot be supplied by worker JSON.
pub struct BrowserDestination {
    url: Url,
    host: String,
    addresses: Vec<SocketAddr>,
}

impl BrowserDestination {
    pub fn url(&self) -> &Url {
        &self.url
    }

    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn addresses(&self) -> &[SocketAddr] {
        &self.addresses
    }
}

/// Fixed categories deliberately omit URLs, response bodies and backend errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum BrowserPolicyError {
    #[error("invalid public-web destination")]
    InvalidDestination,
    #[error("public-web destination resolution failed")]
    ResolutionFailed,
    #[error("non-public browser destination denied")]
    NonPublicDestination,
    #[error("existing network policy denied browser acquisition")]
    PolicyDenied,
}

/// Resolve each request (including every redirect) and compose with the existing
/// network policy without invoking an approval override. The caller must disable
/// automatic redirects/proxies and use only these addresses with normal TLS checks.
pub async fn resolve_browser_destination(
    input: &str,
    state: &NetworkProxyState,
) -> Result<BrowserDestination, BrowserPolicyError> {
    resolve_browser_destination_with_lookup(input, state, |host, port| async move {
        tokio::net::lookup_host((host, port))
            .await
            .map(|addresses| addresses.take(MAX_ADDRESSES + 1).collect())
    })
    .await
}

// Keep the resolver behind the policy gate, including when testing ordering:
// denied names must never be passed to the host resolver by this adapter.
async fn resolve_browser_destination_with_lookup<F>(
    input: &str,
    state: &NetworkProxyState,
    lookup: impl FnOnce(String, u16) -> F,
) -> Result<BrowserDestination, BrowserPolicyError>
where
    F: std::future::Future<Output = std::io::Result<Vec<SocketAddr>>>,
{
    let url = parse_destination(input)?;
    let host = match url.host().ok_or(BrowserPolicyError::InvalidDestination)? {
        Host::Domain(name) => name.to_owned(),
        Host::Ipv4(ip) => ip.to_string(),
        Host::Ipv6(ip) => ip.to_string(),
    };
    let port = url
        .port_or_known_default()
        .ok_or(BrowserPolicyError::InvalidDestination)?;
    let request = NetworkPolicyRequest::new(NetworkPolicyRequestArgs {
        protocol: if url.scheme() == "https" {
            NetworkProtocol::HttpsConnect
        } else {
            NetworkProtocol::Http
        },
        host: host.clone(),
        port,
        environment_id: None,
        client_addr: None,
        method: Some("GET".to_owned()),
        command: None,
        exec_policy_hint: None,
    });
    if !state
        .method_allowed("GET")
        .await
        .map_err(|_| BrowserPolicyError::PolicyDenied)?
        || evaluate_host_policy(state, /*decider*/ None, &request)
            .await
            .map_err(|_| BrowserPolicyError::PolicyDenied)?
            != NetworkDecision::Allow
    {
        return Err(BrowserPolicyError::PolicyDenied);
    }
    // Native host policy denies explicitly blocked names before its own DNS
    // checks. Its not-allowlisted DNS ordering remains owned by that policy.
    let addresses = tokio::time::timeout(Duration::from_secs(3), lookup(host.clone(), port))
        .await
        .map_err(|_| BrowserPolicyError::ResolutionFailed)?
        .map_err(|_| BrowserPolicyError::ResolutionFailed)?;
    validate_addresses(&addresses)?;
    Ok(BrowserDestination {
        url,
        host,
        addresses,
    })
}

fn parse_destination(input: &str) -> Result<Url, BrowserPolicyError> {
    if input.len() > MAX_URL_BYTES || input.chars().any(char::is_control) || input.trim() != input {
        return Err(BrowserPolicyError::InvalidDestination);
    }
    let mut url = Url::parse(input).map_err(|_| BrowserPolicyError::InvalidDestination)?;
    if !matches!(
        (url.scheme(), url.port_or_known_default()),
        ("http", Some(80)) | ("https", Some(443))
    ) || !url.username().is_empty()
        || url.password().is_some()
    {
        return Err(BrowserPolicyError::InvalidDestination);
    }
    match url.host().ok_or(BrowserPolicyError::InvalidDestination)? {
        Host::Domain(name) => {
            let name = name.trim_end_matches('.');
            if !name.contains('.')
                || [
                    "localhost",
                    "local",
                    "internal",
                    "invalid",
                    "test",
                    "onion",
                    "arpa",
                ]
                .iter()
                .any(|suffix| name == *suffix || name.ends_with(&format!(".{suffix}")))
            {
                return Err(BrowserPolicyError::NonPublicDestination);
            }
        }
        Host::Ipv4(ip) => validate_addresses(&[SocketAddr::new(ip.into(), 80)])?,
        Host::Ipv6(ip) => validate_addresses(&[SocketAddr::new(ip.into(), 80)])?,
    }
    url.set_fragment(None);
    Ok(url)
}

fn validate_addresses(addresses: &[SocketAddr]) -> Result<(), BrowserPolicyError> {
    if addresses.is_empty() || addresses.len() > MAX_ADDRESSES {
        return Err(BrowserPolicyError::ResolutionFailed);
    }
    if addresses
        .iter()
        .any(|address| !is_public_browser_ip(address.ip()))
    {
        return Err(BrowserPolicyError::NonPublicDestination);
    }
    Ok(())
}

fn is_public_browser_ip(ip: IpAddr) -> bool {
    if is_non_public_ip(ip) {
        return false;
    }
    match ip {
        IpAddr::V4(ip) => !ip.octets().starts_with(&[192, 88, 99]),
        IpAddr::V6(ip) => {
            let s = ip.segments();
            // Only ordinary global unicast. Exclude translation/tunnel and
            // documentation prefixes, even if a host's routing table handles them.
            s[0] & 0xe000 == 0x2000
                && !(s[0] == 0x2001 && (s[1] < 0x200 || s[1] == 0xdb8))
                && s[0] != 0x2002
                && !(s[0] == 0x3fff && s[1] & 0xf000 == 0)
        }
    }
}

#[cfg(test)]
#[path = "browser_policy_tests.rs"]
mod tests;
