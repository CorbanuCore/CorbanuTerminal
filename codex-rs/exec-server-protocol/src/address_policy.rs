//! Outbound address policy for executor-side HTTP requests (#1947).
//!
//! A request carrying an [`HttpAddressPolicy`] may connect only to the
//! addresses the policy allows. The executor applies it to every address it
//! resolves (and connects only to those, so DNS can't rebind between check and
//! connect), to every IP-literal URL, and again on every redirect hop.
//!
//! Public addresses are always allowed. Loopback and private addresses
//! (RFC 1918, CGNAT and IPv6 unique-local) are allowed only when the policy
//! says so. Link-local, cloud-metadata, unspecified, multicast, broadcast and
//! reserved addresses are never allowed by class; a literal URL may still use
//! the one address in [`HttpAddressPolicy::allowed_address`].

use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::net::Ipv6Addr;

use serde::Deserialize;
use serde::Serialize;

/// Prefix of the error message returned when the address policy blocks a request.
pub const HTTP_ADDRESS_POLICY_DENIED_PREFIX: &str = "http/request blocked by address policy";
/// Error message when a policy-bearing request is not sent because the
/// executor would not enforce the policy.
pub const HTTP_ADDRESS_POLICY_UNSUPPORTED: &str =
    "the remote executor does not enforce HTTP address policies; update it to a version that does";

/// Cloud instance-metadata and similar host-local service addresses. These are
/// blocked even where their class (private, unique-local) would be allowed.
const METADATA_IPV4: [Ipv4Addr; 5] = [
    Ipv4Addr::new(169, 254, 169, 254),
    Ipv4Addr::new(169, 254, 170, 2),
    Ipv4Addr::new(169, 254, 170, 23),
    Ipv4Addr::new(100, 100, 100, 200),
    Ipv4Addr::new(168, 63, 129, 16),
];
const METADATA_IPV6: [Ipv6Addr; 3] = [
    Ipv6Addr::new(0xfd00, 0xec2, 0, 0, 0, 0, 0, 0x254),
    Ipv6Addr::new(0xfd00, 0xec2, 0, 0, 0, 0, 0, 0x23),
    Ipv6Addr::new(0xfd20, 0xce, 0, 0, 0, 0, 0, 0x254),
];
/// Host names of cloud instance-metadata services, refused before resolving
/// (a proxy would resolve them on its own host).
const METADATA_HOST_NAMES: [&str; 5] = [
    "metadata",
    "metadata.google.internal",
    "metadata.azure.internal",
    "instance-data",
    "instance-data.ec2.internal",
];

/// Which non-public destinations an executor-side HTTP request may reach.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpAddressPolicy {
    /// Allow loopback addresses (`127.0.0.0/8`, `::1`).
    #[serde(default)]
    pub allow_loopback: bool,
    /// Allow private addresses (RFC 1918, `100.64.0.0/10`, `fc00::/7`).
    #[serde(default)]
    pub allow_private: bool,
    /// One address an IP-literal URL may always use, whatever its class.
    /// Resolved host names never match it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allowed_address: Option<IpAddr>,
}

/// Address class used by [`HttpAddressPolicy`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpAddressClass {
    Public,
    Loopback,
    Private,
    LinkLocal,
    Blocked,
}

impl HttpAddressClass {
    pub fn of(addr: IpAddr) -> Self {
        match canonical(addr) {
            IpAddr::V4(v4) => classify_v4(v4),
            IpAddr::V6(v6) => classify_v6(v6),
        }
    }
}

impl HttpAddressPolicy {
    /// Whether a host name may connect to `addr`, one of its resolved addresses.
    pub fn allows_resolved(&self, addr: IpAddr) -> bool {
        match HttpAddressClass::of(addr) {
            HttpAddressClass::Public => true,
            HttpAddressClass::Loopback => self.allow_loopback,
            HttpAddressClass::Private => self.allow_private,
            HttpAddressClass::LinkLocal | HttpAddressClass::Blocked => false,
        }
    }

    /// Whether a URL may name `host` (a DNS name, not an IP literal) at all.
    /// Applies even when a proxy resolves the name: metadata service names are
    /// always refused, and `localhost` names unless loopback is allowed.
    pub fn allows_host_name(&self, host: &str) -> bool {
        let host = host.trim_end_matches('.').to_ascii_lowercase();
        if METADATA_HOST_NAMES.contains(&host.as_str()) {
            false
        } else if host == "localhost" || host.ends_with(".localhost") {
            self.allow_loopback
        } else {
            true
        }
    }

    /// Whether an IP-literal URL may connect to `addr`.
    pub fn allows_literal(&self, addr: IpAddr) -> bool {
        self.allowed_address
            .is_some_and(|allowed| canonical(allowed) == canonical(addr))
            || self.allows_resolved(addr)
    }
}

/// Unwraps IPv4 addresses embedded in IPv4-mapped, IPv4-translated
/// (`::ffff:0:0/96`), IPv4-compatible, NAT64 (`64:ff9b::/96`) and 6to4
/// (`2002::/16`) IPv6 addresses.
fn canonical(addr: IpAddr) -> IpAddr {
    let IpAddr::V6(v6) = addr else {
        return addr;
    };
    let [a, b, c, d, e, f, g, h] = v6.segments();
    let low = Ipv4Addr::from((u32::from(g) << 16) | u32::from(h));
    match [a, b, c, d, e, f] {
        [0, 0, 0, 0, 0, 0xffff] | [0, 0, 0, 0, 0xffff, 0] | [0x64, 0xff9b, 0, 0, 0, 0] => {
            IpAddr::V4(low)
        }
        [0, 0, 0, 0, 0, 0] if !v6.is_loopback() && !v6.is_unspecified() => IpAddr::V4(low),
        [0x2002, ..] => IpAddr::V4(Ipv4Addr::from((u32::from(b) << 16) | u32::from(c))),
        _ => addr,
    }
}

fn classify_v4(addr: Ipv4Addr) -> HttpAddressClass {
    let [a, b, c, _] = addr.octets();
    if METADATA_IPV4.contains(&addr) {
        HttpAddressClass::Blocked
    } else if addr.is_loopback() {
        HttpAddressClass::Loopback
    } else if addr.is_private() || (a == 100 && (64..=127).contains(&b)) {
        HttpAddressClass::Private
    } else if addr.is_link_local() {
        HttpAddressClass::LinkLocal
    } else if a == 0
        || a >= 224
        || (a == 192 && b == 0 && c == 0)
        || (a == 198 && matches!(b, 18 | 19))
    {
        // This network, multicast, reserved and broadcast, IETF protocol
        // assignments, benchmarking.
        HttpAddressClass::Blocked
    } else {
        HttpAddressClass::Public
    }
}

fn classify_v6(addr: Ipv6Addr) -> HttpAddressClass {
    let segments = addr.segments();
    if METADATA_IPV6.contains(&addr) {
        HttpAddressClass::Blocked
    } else if addr.is_loopback() {
        HttpAddressClass::Loopback
    } else if (segments[0] & 0xffc0) == 0xfe80 {
        HttpAddressClass::LinkLocal
    } else if (segments[0] & 0xfe00) == 0xfc00 {
        HttpAddressClass::Private
    } else if addr.is_unspecified()
        || addr.is_multicast()
        // Deprecated site-local, Teredo, local-use NAT64, discard-only.
        || (segments[0] & 0xffc0) == 0xfec0
        || segments[..2] == [0x2001, 0]
        || segments[..3] == [0x64, 0xff9b, 1]
        || segments[..4] == [0x100, 0, 0, 0]
    {
        HttpAddressClass::Blocked
    } else {
        HttpAddressClass::Public
    }
}

#[cfg(test)]
#[path = "address_policy_tests.rs"]
mod tests;
