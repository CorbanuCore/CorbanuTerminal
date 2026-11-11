//! Restricts which IP addresses a [`crate::RouteAwareClientPool`] may connect to.
//!
//! Host names are resolved by a filtering DNS resolver installed in the
//! transport, so the client connects only to addresses that passed the check
//! (no window for DNS rebinding between check and connect). IP-literal URLs,
//! which the transport never resolves, and host names themselves (for names a
//! proxy would resolve) are checked by the pool before each request and
//! redirect hop. Proxy hosts are exempt from the resolved-address check: a
//! proxied request resolves only the proxy's name, and the proxy resolves the
//! target, so with a proxy only literals and host names are checked.

use std::fmt;
use std::net::IpAddr;
use std::net::SocketAddr;
use std::net::ToSocketAddrs;
use std::sync::Arc;

use reqwest::Url;
use reqwest::dns::Addrs;
use reqwest::dns::Name;
use reqwest::dns::Resolve;
use reqwest::dns::Resolving;

type AddressPredicate = Arc<dyn Fn(IpAddr) -> bool + Send + Sync>;
type HostNamePredicate = Arc<dyn Fn(&str) -> bool + Send + Sync>;
type Lookup = Arc<dyn Fn(String) -> Resolving + Send + Sync>;

const PROXY_ENV_VARS: [&str; 6] = [
    "HTTP_PROXY",
    "http_proxy",
    "HTTPS_PROXY",
    "https_proxy",
    "ALL_PROXY",
    "all_proxy",
];

/// Address checks applied to every connection a pool makes.
#[derive(Clone)]
pub struct ResolvedAddressFilter {
    allows_resolved: AddressPredicate,
    allows_literal: AddressPredicate,
    allows_host_name: HostNamePredicate,
    exempt_hosts: Arc<[String]>,
    lookup: Lookup,
}

impl fmt::Debug for ResolvedAddressFilter {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ResolvedAddressFilter")
            .field("exempt_hosts", &self.exempt_hosts)
            .finish_non_exhaustive()
    }
}

/// A request was refused because its destination is not allowed.
#[derive(Debug, Clone, thiserror::Error)]
pub enum DisallowedAddressError {
    #[error("`{0}` is an address this request may not connect to")]
    Literal(IpAddr),
    #[error("the host name `{0}` is not allowed for this request")]
    HostName(String),
    #[error("`{host}` resolves only to addresses this request may not connect to ({addresses})")]
    Resolved { host: String, addresses: String },
    #[error("the URL has no host")]
    MissingHost,
}

impl DisallowedAddressError {
    fn resolved(host: &str, addresses: &[IpAddr]) -> Self {
        let addresses = if addresses.is_empty() {
            "no addresses".to_string()
        } else {
            addresses
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        };
        Self::Resolved {
            host: host.to_string(),
            addresses,
        }
    }
}

impl ResolvedAddressFilter {
    /// `allows_resolved` decides each address a host name resolves to;
    /// `allows_literal` decides IP-literal URL hosts.
    pub fn new(
        allows_resolved: impl Fn(IpAddr) -> bool + Send + Sync + 'static,
        allows_literal: impl Fn(IpAddr) -> bool + Send + Sync + 'static,
    ) -> Self {
        Self {
            allows_resolved: Arc::new(allows_resolved),
            allows_literal: Arc::new(allows_literal),
            allows_host_name: Arc::new(|_| true),
            exempt_hosts: Arc::new([]),
            lookup: Arc::new(system_lookup),
        }
    }

    /// Also checks DNS host names before any request, including proxied ones.
    pub fn with_host_name_check(
        mut self,
        allows_host_name: impl Fn(&str) -> bool + Send + Sync + 'static,
    ) -> Self {
        self.allows_host_name = Arc::new(allows_host_name);
        self
    }

    /// Replaces the system resolver, for tests that need fixed DNS answers.
    #[cfg(test)]
    pub(crate) fn with_lookup(
        mut self,
        lookup: impl Fn(&str) -> Vec<IpAddr> + Send + Sync + 'static,
    ) -> Self {
        let lookup = Arc::new(lookup);
        self.lookup = Arc::new(move |host| {
            let addrs: Vec<SocketAddr> = lookup(&host)
                .into_iter()
                .map(|ip| SocketAddr::new(ip, 0))
                .collect();
            Box::pin(async move { Ok(Box::new(addrs.into_iter()) as Addrs) })
        });
        self
    }

    /// Exempts the hosts of the given proxy URLs from resolved-address checks.
    pub(crate) fn exempting_proxies(mut self, proxies: impl IntoIterator<Item = String>) -> Self {
        let proxy_hosts = proxies.into_iter().filter_map(|proxy| {
            let proxy = if proxy.contains("://") {
                proxy
            } else {
                format!("http://{proxy}")
            };
            Url::parse(&proxy).ok()?.host_str().map(normalize_host)
        });
        self.exempt_hosts = self
            .exempt_hosts
            .iter()
            .cloned()
            .chain(proxy_hosts)
            .collect();
        self
    }

    /// Exempts proxies the transport may pick up from the environment.
    pub(crate) fn exempting_env_proxies(self) -> Self {
        self.exempting_proxies(
            PROXY_ENV_VARS
                .iter()
                .filter_map(|var| std::env::var(var).ok()),
        )
    }

    /// Checks a URL's host: IP literals by address, host names by name.
    /// Host names' addresses are checked when resolved.
    pub(crate) fn check_url(&self, url: &Url) -> Result<(), DisallowedAddressError> {
        let Some(host) = url.host_str() else {
            return Err(DisallowedAddressError::MissingHost);
        };
        // `Url` normalizes IPv4 literals (`0x7f.1` is `127.0.0.1`) and
        // brackets IPv6 ones.
        match host
            .trim_start_matches('[')
            .trim_end_matches(']')
            .parse::<IpAddr>()
        {
            Ok(ip) if (self.allows_literal)(ip) => Ok(()),
            Ok(ip) => Err(DisallowedAddressError::Literal(ip)),
            Err(_) if (self.allows_host_name)(host) => Ok(()),
            Err(_) => Err(DisallowedAddressError::HostName(host.to_string())),
        }
    }

    pub(crate) fn resolver(&self) -> Arc<FilteringResolver> {
        Arc::new(FilteringResolver(self.clone()))
    }
}

/// DNS resolver that drops every address the filter rejects.
pub(crate) struct FilteringResolver(ResolvedAddressFilter);

impl Resolve for FilteringResolver {
    fn resolve(&self, name: Name) -> Resolving {
        let filter = self.0.clone();
        Box::pin(async move {
            let host = normalize_host(name.as_str());
            let resolved: Vec<SocketAddr> =
                (filter.lookup)(name.as_str().to_string()).await?.collect();
            if filter.exempt_hosts.contains(&host) {
                return Ok(Box::new(resolved.into_iter()) as Addrs);
            }
            let (allowed, rejected): (Vec<SocketAddr>, Vec<SocketAddr>) = resolved
                .into_iter()
                .partition(|addr| (filter.allows_resolved)(addr.ip()));
            if allowed.is_empty() {
                let rejected: Vec<IpAddr> = rejected.iter().map(SocketAddr::ip).collect();
                let error = DisallowedAddressError::resolved(&host, &rejected);
                tracing::warn!("{error}");
                return Err(Box::new(error) as Box<dyn std::error::Error + Send + Sync>);
            }
            Ok(Box::new(allowed.into_iter()) as Addrs)
        })
    }
}

/// Finds a [`DisallowedAddressError`] in an error's source chain.
pub(crate) fn find_disallowed_address(
    error: &(dyn std::error::Error + 'static),
) -> Option<DisallowedAddressError> {
    let mut current = Some(error);
    while let Some(error) = current {
        if let Some(found) = error.downcast_ref::<DisallowedAddressError>() {
            return Some(found.clone());
        }
        current = error.source();
    }
    None
}

fn system_lookup(host: String) -> Resolving {
    Box::pin(async move {
        let addrs = tokio::task::spawn_blocking(move || (host.as_str(), 0).to_socket_addrs())
            .await
            .map_err(|error| Box::new(error) as Box<dyn std::error::Error + Send + Sync>)??;
        Ok(Box::new(addrs) as Addrs)
    })
}

fn normalize_host(host: &str) -> String {
    host.trim_matches(|c| c == '[' || c == ']')
        .trim_end_matches('.')
        .to_ascii_lowercase()
}

#[cfg(test)]
#[path = "address_filter_tests.rs"]
mod tests;
