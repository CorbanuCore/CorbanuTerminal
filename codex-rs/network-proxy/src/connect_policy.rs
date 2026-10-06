use crate::policy::is_non_public_ip;
use crate::runtime::HostBlockDecision;
use crate::state::NetworkProxyState;
use rama_core::Service;
use rama_core::error::BoxError;
use rama_core::error::ErrorExt as _;
use rama_core::error::OpaqueError;
use rama_core::extensions::ExtensionsMut;
use rama_net::address::Host;
use rama_net::address::HostWithPort;
use rama_net::address::ProxyAddress;
use rama_net::client::EstablishedClientConnection;
use rama_net::stream::ClientSocketInfo;
use rama_net::stream::Socket;
use rama_net::stream::SocketInfo;
use rama_net::transport::TryRefIntoTransportContext;
use rama_tcp::TcpStream;
use rama_tcp::client::TcpStreamConnector;
use rama_tcp::client::service::TcpConnector;
use std::io;
use std::net::IpAddr;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tracing::info;

/// Bound on one dial to one pinned address before the next one is tried.
const PINNED_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
/// Bound on all pinned dials of one request (up to 16 checked answers).
const PINNED_DIAL_BUDGET: Duration = Duration::from_secs(30);
const PINNING_REFUSED: &str = "network target rejected by connection pinning";

/// PF-33-S02: the answers the destination guard checked for one request. Under
/// the guard the connector dials only these addresses, for this exact host and
/// port, so the connection uses the peer that was authorized instead of a
/// second, possibly rebound, DNS answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PinnedPeers {
    host: String,
    port: u16,
    addrs: Vec<IpAddr>,
}

impl PinnedPeers {
    pub(crate) fn new(host: &str, port: u16, addrs: impl IntoIterator<Item = IpAddr>) -> Self {
        Self {
            host: pin_host_key(host),
            port,
            addrs: addrs.into_iter().collect(),
        }
    }

    /// Whether this pin was issued for exactly `target`.
    fn covers(&self, target: &HostWithPort) -> bool {
        self.covers_authority(&target.host.to_string(), target.port)
    }

    /// Whether this pin was issued for exactly `host` and `port`.
    pub(crate) fn covers_authority(&self, host: &str, port: u16) -> bool {
        self.port == port && self.host == pin_host_key(host)
    }

    pub(crate) fn addrs(&self) -> &[IpAddr] {
        &self.addrs
    }
}

/// Canonical form of a host for pin comparison: IP literals in their standard
/// form, names lowercased without brackets or a trailing dot.
fn pin_host_key(host: &str) -> String {
    let host = host.trim_start_matches('[').trim_end_matches(']');
    match host.parse::<IpAddr>() {
        Ok(ip) => ip.to_string(),
        Err(_) => host.trim_end_matches('.').to_ascii_lowercase(),
    }
}

fn pinning_refused(detail: &str) -> BoxError {
    io::Error::new(
        io::ErrorKind::PermissionDenied,
        format!("{PINNING_REFUSED} ({detail})"),
    )
    .into()
}

#[derive(Clone)]
pub(crate) struct TargetCheckedTcpConnector {
    state: Arc<NetworkProxyState>,
    /// Test seam: let pinned dials reach loopback fixtures. Never set outside tests.
    #[cfg(test)]
    allow_loopback_peers: bool,
}

impl TargetCheckedTcpConnector {
    pub(crate) fn new(state: Arc<NetworkProxyState>) -> Self {
        Self {
            state,
            #[cfg(test)]
            allow_loopback_peers: false,
        }
    }

    #[cfg(test)]
    pub(crate) fn allowing_loopback_peers_for_tests(state: Arc<NetworkProxyState>) -> Self {
        Self {
            state,
            allow_loopback_peers: true,
        }
    }

    fn stream_connector(&self, target: HostWithPort) -> TargetCheckedStreamConnector {
        TargetCheckedStreamConnector {
            state: self.state.clone(),
            target,
            #[cfg(test)]
            allow_loopback_peers: self.allow_loopback_peers,
        }
    }
}

impl<Input> Service<Input> for TargetCheckedTcpConnector
where
    Input: TryRefIntoTransportContext + Send + ExtensionsMut + 'static,
    Input::Error: Into<BoxError> + Send + Sync + 'static,
{
    type Output = EstablishedClientConnection<TcpStream, Input>;
    type Error = BoxError;

    async fn serve(&self, input: Input) -> Result<Self::Output, Self::Error> {
        // A pin is honoured even where the guard is off: the isolated broker
        // has no guard of its own and dials the answers Core checked.
        let pin = input.extensions().get::<PinnedPeers>().cloned();
        let guarded = pin.is_some() || guard_enabled(&self.state).await?;
        if input.extensions().get::<ProxyAddress>().is_some() {
            // PF-33-S02: an upstream proxy resolves and connects on its own, so
            // the checked answers could not be pinned. The guard refuses it.
            if guarded {
                return Err(pinning_refused("upstream proxy"));
            }
            return TcpConnector::new().serve(input).await;
        }

        let target = input
            .try_ref_into_transport_ctx()
            .map_err(|err| OpaqueError::from_boxed(err.into()).context("read network target"))?
            .host_with_port()
            .ok_or_else(|| OpaqueError::from_display("network target is missing a port"))?;

        if guarded {
            // Every guarded dial carries the guard's checked answers for its
            // exact authority; anything else is refused, never re-resolved.
            let pin = pin.ok_or_else(|| pinning_refused("no checked answers"))?;
            if !pin.covers(&target) {
                return Err(pinning_refused("authority changed"));
            }
            let connector = self.stream_connector(target);
            return dial_pinned(input, &pin, &connector).await;
        }

        TcpConnector::new()
            .with_connector(self.stream_connector(target))
            .serve(input)
            .await
    }
}

async fn guard_enabled(state: &NetworkProxyState) -> Result<bool, BoxError> {
    crate::destination::guard_enabled(state)
        .await
        .map_err(|err| {
            OpaqueError::from_display(err.to_string())
                .context("read destination policy")
                .into_boxed()
        })
}

/// Connect to the pinned addresses in order, without any DNS lookup. Each
/// address still passes the peer check of `connector`.
async fn dial_pinned<Input>(
    input: Input,
    pin: &PinnedPeers,
    connector: &TargetCheckedStreamConnector,
) -> Result<EstablishedClientConnection<TcpStream, Input>, BoxError> {
    tokio::time::timeout(
        PINNED_DIAL_BUDGET,
        dial_pinned_in_order(input, pin, connector),
    )
    .await
    .unwrap_or_else(|_| {
        Err(io::Error::new(io::ErrorKind::TimedOut, "pinned dials timed out").into())
    })
}

async fn dial_pinned_in_order<Input>(
    input: Input,
    pin: &PinnedPeers,
    connector: &TargetCheckedStreamConnector,
) -> Result<EstablishedClientConnection<TcpStream, Input>, BoxError> {
    let mut last_error = pinning_refused("no checked answers");
    for ip in &pin.addrs {
        let addr = SocketAddr::new(*ip, pin.port);
        match tokio::time::timeout(PINNED_CONNECT_TIMEOUT, connector.connect(addr)).await {
            Ok(Ok(mut conn)) => {
                let local = conn.local_addr().ok();
                conn.extensions_mut()
                    .insert(ClientSocketInfo(SocketInfo::new(local, addr)));
                info!(
                    "pinned dial established (host={}, port={}, answers={})",
                    pin.host,
                    pin.port,
                    pin.addrs.len()
                );
                return Ok(EstablishedClientConnection { input, conn });
            }
            Ok(Err(err)) => last_error = err,
            Err(_) => {
                last_error = io::Error::new(io::ErrorKind::TimedOut, "pinned dial timed out").into()
            }
        }
    }
    Err(last_error)
}

#[derive(Clone)]
struct TargetCheckedStreamConnector {
    state: Arc<NetworkProxyState>,
    target: HostWithPort,
    #[cfg(test)]
    allow_loopback_peers: bool,
}

impl TcpStreamConnector for TargetCheckedStreamConnector {
    type Error = BoxError;

    async fn connect(&self, addr: SocketAddr) -> Result<TcpStream, Self::Error> {
        #[cfg(test)]
        let test_loopback = self.allow_loopback_peers && addr.ip().is_loopback();
        #[cfg(not(test))]
        let test_loopback = false;
        if is_non_public_ip(addr.ip())
            && !test_loopback
            && !self.allows_non_public_target(addr).await?
        {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "network target rejected by policy",
            )
            .into());
        }

        tokio::net::TcpStream::connect(addr)
            .await
            .map(TcpStream::from)
            .map_err(Into::into)
    }
}

impl TargetCheckedStreamConnector {
    async fn allows_non_public_target(&self, addr: SocketAddr) -> Result<bool, BoxError> {
        // PF-33-S01: under the destination guard, local binding and literal
        // local allowlist entries are not private-network trust grants.
        if guard_enabled(&self.state).await? {
            return Ok(false);
        }
        if self.state.allow_local_binding().await.map_err(|err| {
            let err: BoxError = err.into();
            OpaqueError::from_boxed(err)
                .context("read network proxy config")
                .into_boxed()
        })? {
            return Ok(true);
        }

        if !target_matches_non_public_addr(&self.target.host, addr.ip()) {
            return Ok(false);
        }

        self.state
            .host_blocked(&self.target.host.to_string(), self.target.port)
            .await
            .map(|decision| decision == HostBlockDecision::Allowed)
            .map_err(|err| {
                let err: BoxError = err.into();
                OpaqueError::from_boxed(err)
                    .context("evaluate network proxy target")
                    .into_boxed()
            })
    }
}

pub(crate) fn is_non_public_target(host: &Host) -> bool {
    match host {
        Host::Address(ip) => is_non_public_ip(*ip),
        Host::Name(name) => name
            .as_str()
            .trim_end_matches('.')
            .eq_ignore_ascii_case("localhost"),
    }
}

fn target_matches_non_public_addr(host: &Host, addr: std::net::IpAddr) -> bool {
    match host {
        Host::Address(ip) => *ip == addr,
        Host::Name(name) => {
            name.as_str()
                .trim_end_matches('.')
                .eq_ignore_ascii_case("localhost")
                && addr.is_loopback()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::NetworkProxyConfig;
    use crate::state::network_proxy_state_for_policy;
    use rama_net::address::HostWithPort;
    use std::net::Ipv4Addr;
    use tokio::net::TcpListener;

    #[tokio::test(flavor = "current_thread")]
    async fn direct_connector_rejects_non_public_target_when_local_binding_disabled() {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
            .await
            .expect("bind local listener");
        let target = listener.local_addr().expect("local addr");
        let connector = TargetCheckedTcpConnector::new(Arc::new(network_proxy_state_for_policy(
            NetworkProxyConfig::default(),
        )));

        let request: rama_tcp::client::Request =
            rama_tcp::client::Request::new(HostWithPort::from(target));
        let err = Service::serve(&connector, request)
            .await
            .expect_err("local target should be rejected");

        assert!(
            format!("{err:?}").contains("network target rejected by policy"),
            "unexpected error: {err:?}"
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn direct_connector_allows_non_public_target_when_local_binding_enabled() {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
            .await
            .expect("bind local listener");
        let target = listener.local_addr().expect("local addr");
        let connector = TargetCheckedTcpConnector::new(Arc::new(network_proxy_state_for_policy(
            NetworkProxyConfig {
                allow_local_binding: true,
                ..NetworkProxyConfig::default()
            },
        )));

        let request: rama_tcp::client::Request =
            rama_tcp::client::Request::new(HostWithPort::from(target));
        let result = Service::serve(&connector, request).await;

        assert!(result.is_ok(), "local target should be allowed: {result:?}");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn pf_33_s01_guard_rejects_local_peer_despite_local_binding() {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
            .await
            .expect("bind local listener");
        let target = listener.local_addr().expect("local addr");
        let mut config = NetworkProxyConfig {
            allow_local_binding: true,
            ..NetworkProxyConfig::default()
        };
        config.set_allowed_domains(vec![target.ip().to_string()]);
        config.set_url_destination_policy(/*enabled*/ true);
        let connector =
            TargetCheckedTcpConnector::new(Arc::new(network_proxy_state_for_policy(config)));

        let mut request: rama_tcp::client::Request =
            rama_tcp::client::Request::new(HostWithPort::from(target));
        // PF-33-S02: carry a pin so the dial reaches the peer check.
        request.extensions_mut().insert(PinnedPeers::new(
            &target.ip().to_string(),
            target.port(),
            [target.ip()],
        ));
        let err = Service::serve(&connector, request)
            .await
            .expect_err("guarded local peer should be rejected");

        assert!(
            format!("{err:?}").contains("network target rejected by policy"),
            "unexpected error: {err:?}"
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn direct_connector_allows_explicitly_allowlisted_non_public_target() {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
            .await
            .expect("bind local listener");
        let target = listener.local_addr().expect("local addr");
        let mut config = NetworkProxyConfig::default();
        config.set_allowed_domains(vec![target.ip().to_string()]);
        let connector =
            TargetCheckedTcpConnector::new(Arc::new(network_proxy_state_for_policy(config)));

        let request: rama_tcp::client::Request =
            rama_tcp::client::Request::new(HostWithPort::from(target));
        let result = Service::serve(&connector, request).await;

        assert!(
            result.is_ok(),
            "explicitly allowlisted local target should be allowed: {result:?}"
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn direct_connector_allows_explicitly_allowlisted_localhost_target() {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
            .await
            .expect("bind local listener");
        let target = listener.local_addr().expect("local addr");
        let mut config = NetworkProxyConfig::default();
        config.set_allowed_domains(vec!["localhost".to_string()]);
        let connector =
            TargetCheckedTcpConnector::new(Arc::new(network_proxy_state_for_policy(config)));

        let request: rama_tcp::client::Request =
            rama_tcp::client::Request::new(HostWithPort::new(Host::LOCALHOST_NAME, target.port()));
        let result = Service::serve(&connector, request).await;

        assert!(
            result.is_ok(),
            "explicitly allowlisted localhost target should be allowed: {result:?}"
        );
    }

    #[test]
    fn resolved_private_address_does_not_match_allowlisted_hostname() {
        let host = Host::Name("example.com".parse().expect("valid domain"));

        assert!(!target_matches_non_public_addr(
            &host,
            Ipv4Addr::LOCALHOST.into()
        ));
    }

    fn guarded_state() -> Arc<NetworkProxyState> {
        let mut config = NetworkProxyConfig::default();
        config.set_url_destination_policy(/*enabled*/ true);
        Arc::new(network_proxy_state_for_policy(config))
    }

    /// A target that no resolver can answer (RFC 6761 `.invalid`): reaching
    /// the listener proves the dial used the pinned answer, not DNS.
    const UNRESOLVABLE: &str = "pinned.invalid";

    fn pinned_request(
        host: &str,
        port: u16,
        pin: Option<PinnedPeers>,
    ) -> rama_tcp::client::Request {
        let target = HostWithPort::new(Host::Name(host.parse().expect("valid name")), port);
        let mut request = rama_tcp::client::Request::new(target);
        if let Some(pin) = pin {
            request.extensions_mut().insert(pin);
        }
        request
    }

    fn assert_refused<T>(result: Result<T, BoxError>, detail: &str) {
        let Err(err) = result else {
            panic!("expected a refusal ({detail})");
        };
        let message = format!("{err:?}");
        assert!(message.contains(detail), "unexpected error: {message}");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn pf_33_s02_guarded_dial_uses_checked_answer_without_dns() {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
            .await
            .expect("bind local listener");
        let target = listener.local_addr().expect("local addr");
        let connector =
            TargetCheckedTcpConnector::allowing_loopback_peers_for_tests(guarded_state());
        let pin = PinnedPeers::new(UNRESOLVABLE, target.port(), [target.ip()]);

        let established = Service::serve(
            &connector,
            pinned_request(UNRESOLVABLE, target.port(), Some(pin)),
        )
        .await
        .expect("pinned dial should reach the checked answer");
        let (_, peer) = listener.accept().await.expect("accept pinned dial");

        assert_eq!(
            established.conn.peer_addr().expect("peer addr"),
            target,
            "the connection must use the checked address"
        );
        assert_eq!(peer.ip(), target.ip());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn pf_33_s02_guarded_dial_falls_back_only_within_checked_answers() {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
            .await
            .expect("bind local listener");
        let target = listener.local_addr().expect("local addr");
        let connector =
            TargetCheckedTcpConnector::allowing_loopback_peers_for_tests(guarded_state());
        // Nothing listens on [::1] at this port: the dial moves to the next
        // checked answer instead of asking DNS again.
        let pin = PinnedPeers::new(
            UNRESOLVABLE,
            target.port(),
            [std::net::Ipv6Addr::LOCALHOST.into(), target.ip()],
        );

        let established = Service::serve(
            &connector,
            pinned_request(UNRESOLVABLE, target.port(), Some(pin)),
        )
        .await
        .expect("second checked answer should be used");

        assert_eq!(established.conn.peer_addr().expect("peer addr"), target);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn pf_33_s02_guard_refuses_unpinned_and_mismatched_dials() {
        let connector = TargetCheckedTcpConnector::new(guarded_state());

        assert_refused(
            Service::serve(&connector, pinned_request("example.com", 443, None)).await,
            "no checked answers",
        );
        let public: IpAddr = "93.184.216.34".parse().expect("ip");
        for (host, port) in [("other.example", 443), ("example.com", 8443)] {
            let pin = PinnedPeers::new(host, port, [public]);
            assert_refused(
                Service::serve(&connector, pinned_request("example.com", 443, Some(pin))).await,
                "authority changed",
            );
        }
        let empty = PinnedPeers::new("example.com", 443, []);
        assert_refused(
            Service::serve(&connector, pinned_request("example.com", 443, Some(empty))).await,
            "no checked answers",
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn pf_33_s02_guard_refuses_upstream_proxy_route() {
        let connector = TargetCheckedTcpConnector::new(guarded_state());
        let public: IpAddr = "93.184.216.34".parse().expect("ip");
        let mut request = pinned_request(
            "example.com",
            443,
            Some(PinnedPeers::new("example.com", 443, [public])),
        );
        request
            .extensions_mut()
            .insert(ProxyAddress::try_from("http://proxy.example:3128").expect("proxy address"));

        assert_refused(Service::serve(&connector, request).await, "upstream proxy");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn pf_33_s02_pin_cannot_authorize_a_private_peer() {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
            .await
            .expect("bind local listener");
        let target = listener.local_addr().expect("local addr");
        // No test seam: a pin naming a loopback answer is still refused by the
        // peer check, even with local binding configured.
        let mut config = NetworkProxyConfig {
            allow_local_binding: true,
            ..NetworkProxyConfig::default()
        };
        config.set_url_destination_policy(/*enabled*/ true);
        let connector =
            TargetCheckedTcpConnector::new(Arc::new(network_proxy_state_for_policy(config)));
        let pin = PinnedPeers::new(UNRESOLVABLE, target.port(), [target.ip()]);

        assert_refused(
            Service::serve(
                &connector,
                pinned_request(UNRESOLVABLE, target.port(), Some(pin)),
            )
            .await,
            "network target rejected by policy",
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn pf_33_s02_pins_bind_even_with_the_flag_off() {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
            .await
            .expect("bind local listener");
        let target = listener.local_addr().expect("local addr");
        let connector = TargetCheckedTcpConnector::new(Arc::new(network_proxy_state_for_policy(
            NetworkProxyConfig {
                allow_local_binding: true,
                ..NetworkProxyConfig::default()
            },
        )));
        // Without a pin the flag-off dial is today's dial.
        let request = rama_tcp::client::Request::new(HostWithPort::from(target));
        let result = Service::serve(&connector, request).await;
        assert!(result.is_ok(), "flag off keeps today's dial: {result:?}");

        // A pin only narrows a dial, so it binds without the guard too: the
        // isolated broker has no guard and dials the answers Core checked.
        let stale = PinnedPeers::new("other.example", 1, ["93.184.216.34".parse().expect("ip")]);
        let mut request = rama_tcp::client::Request::new(HostWithPort::from(target));
        request.extensions_mut().insert(stale);
        assert_refused(
            Service::serve(&connector, request).await,
            "network target rejected by connection pinning (authority changed)",
        );
        let pin = PinnedPeers::new(&target.ip().to_string(), target.port(), [target.ip()]);
        let mut request = rama_tcp::client::Request::new(HostWithPort::from(target));
        request.extensions_mut().insert(pin);
        let result = Service::serve(&connector, request).await;
        assert!(result.is_ok(), "a matching pin dials: {result:?}");
    }

    #[test]
    fn pf_33_s02_pin_host_comparison_is_canonical() {
        let pin = PinnedPeers::new("Example.COM.", 443, []);
        assert!(pin.covers(&HostWithPort::new(
            Host::Name("example.com".parse().expect("name")),
            443
        )));
        // The guard pins the A-label form; clients send the same form.
        let idn = PinnedPeers::new("xn--bcher-kva.example", 443, []);
        assert!(idn.covers(&HostWithPort::new(
            Host::Name("XN--BCHER-KVA.example".parse().expect("name")),
            443
        )));
        assert!(!idn.covers(&HostWithPort::new(
            Host::Name("bcher.example".parse().expect("name")),
            443
        )));
        let v6 = PinnedPeers::new("[2606:2800:220:1:248:1893:25c8:1946]", 443, []);
        assert!(v6.covers(&HostWithPort::new(
            Host::Address("2606:2800:220:1:248:1893:25c8:1946".parse().expect("ip")),
            443
        )));
    }
}
