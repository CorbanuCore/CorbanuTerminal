use std::io::Read;
use std::io::Write;
use std::net::IpAddr;
use std::net::SocketAddr;
use std::net::TcpListener;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::time::Duration;

use pretty_assertions::assert_eq;

use super::*;
use crate::ClientRouteClass;
use crate::HttpClientFactory;
use crate::OutboundProxyPolicy;
use crate::RouteAwareClientPool;
use crate::RouteAwareRequestError;

const OK: &str = "HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok";

/// Serves `response` to every connection on 127.0.0.1 and counts connections.
fn spawn_server(response: String) -> (SocketAddr, Arc<AtomicUsize>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind test server");
    let addr = listener.local_addr().expect("test server address");
    let connections = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&connections);
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else {
                return;
            };
            counter.fetch_add(1, Ordering::SeqCst);
            let mut request = Vec::new();
            let mut buffer = [0; 1024];
            while !request.windows(4).any(|window| window == b"\r\n\r\n") {
                match stream.read(&mut buffer) {
                    Ok(0) | Err(_) => break,
                    Ok(read) => request.extend_from_slice(&buffer[..read]),
                }
            }
            let _ = stream.write_all(response.as_bytes());
        }
    });
    (addr, connections)
}

fn redirect_to(location: &str) -> String {
    format!(
        "HTTP/1.1 302 Found\r\nLocation: {location}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
    )
}

fn loopback_literal_only() -> ResolvedAddressFilter {
    ResolvedAddressFilter::new(|_| false, |ip: IpAddr| ip == IpAddr::from([127, 0, 0, 1]))
}

fn pool(filter: ResolvedAddressFilter) -> RouteAwareClientPool {
    RouteAwareClientPool::new(
        HttpClientFactory::new(OutboundProxyPolicy::ReqwestDefault),
        ClientRouteClass::Other,
    )
    .with_resolved_address_filter(filter)
}

async fn get(pool: &RouteAwareClientPool, url: &str) -> Result<String, RouteAwareRequestError> {
    let response = tokio::time::timeout(Duration::from_secs(10), pool.get(url).send())
        .await
        .expect("request should finish without waiting on a blocked address")?;
    Ok(response.text().await.expect("response body"))
}

fn assert_disallowed(result: Result<String, RouteAwareRequestError>) {
    match result {
        Err(RouteAwareRequestError::DisallowedAddress(_)) => {}
        other => panic!("expected a disallowed-address error, got {other:?}"),
    }
}

#[tokio::test]
async fn literal_address_is_checked_before_connecting() {
    let (addr, connections) = spawn_server(OK.to_string());
    let pool = pool(ResolvedAddressFilter::new(|_| false, |_| false));

    let port = addr.port();
    for url in [
        format!("http://{addr}/"),
        format!("http://[::ffff:127.0.0.1]:{port}/"),
        format!("http://2130706433:{port}/"),
        format!("http://0x7f.1:{port}/"),
        format!("http://0177.0.0.1:{port}/"),
    ] {
        assert_disallowed(get(&pool, &url).await);
    }
    assert_eq!(connections.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn allowed_literal_address_connects() {
    let (addr, connections) = spawn_server(OK.to_string());
    let pool = pool(loopback_literal_only());

    assert_eq!(get(&pool, &format!("http://{addr}/")).await.unwrap(), "ok");
    assert_eq!(connections.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn host_name_resolving_to_disallowed_address_is_refused() {
    let (addr, connections) = spawn_server(OK.to_string());
    let filter = loopback_literal_only().with_lookup(|_| vec![IpAddr::from([127, 0, 0, 1])]);

    let error = get(
        &pool(filter),
        &format!("http://rebind.test:{}/", addr.port()),
    )
    .await
    .expect_err("loopback answer should be refused");
    assert!(
        matches!(
            &error,
            RouteAwareRequestError::DisallowedAddress(DisallowedAddressError::Resolved { host, .. })
                if host == "rebind.test"
        ),
        "{error:?}"
    );
    assert_eq!(connections.load(Ordering::SeqCst), 0);
}

/// The transport connects to the addresses the filter checked: `pinned.test`
/// exists only in the test lookup, and the unroutable private answer is
/// dropped before connecting.
#[tokio::test]
async fn connects_only_to_checked_addresses() {
    let (addr, connections) = spawn_server(OK.to_string());
    let filter =
        ResolvedAddressFilter::new(|ip: IpAddr| ip.is_loopback(), |_| false).with_lookup(|_| {
            vec![
                IpAddr::from([10, 255, 255, 1]),
                IpAddr::from([127, 0, 0, 1]),
            ]
        });

    let body = get(
        &pool(filter),
        &format!("http://pinned.test:{}/", addr.port()),
    )
    .await
    .unwrap();
    assert_eq!(body, "ok");
    assert_eq!(connections.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn redirect_to_disallowed_literal_is_refused() {
    let (addr, connections) = spawn_server(redirect_to("http://169.254.169.254/latest/meta-data/"));

    assert_disallowed(get(&pool(loopback_literal_only()), &format!("http://{addr}/")).await);
    assert_eq!(connections.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn redirect_to_host_resolving_to_disallowed_address_is_refused() {
    let (addr, connections) = spawn_server(String::new());
    let (redirect_addr, _) = spawn_server(redirect_to(&format!(
        "http://internal.test:{}/",
        addr.port()
    )));
    let filter = loopback_literal_only().with_lookup(|_| vec![IpAddr::from([127, 0, 0, 1])]);

    assert_disallowed(get(&pool(filter), &format!("http://{redirect_addr}/")).await);
    assert_eq!(connections.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn proxy_hosts_are_exempt_from_resolved_address_checks() {
    let filter = ResolvedAddressFilter::new(|_| false, |_| false)
        .with_lookup(|_| vec![IpAddr::from([127, 0, 0, 1])])
        .exempting_proxies(["proxy.test:3128".to_string()]);
    let resolve = |host: &str| {
        let name: Name = host.parse().expect("dns name");
        filter.resolver().resolve(name)
    };

    let addrs: Vec<SocketAddr> = resolve("proxy.test").await.unwrap().collect();
    assert_eq!(addrs, vec![SocketAddr::from(([127, 0, 0, 1], 0))]);
    assert!(resolve("target.test").await.is_err());
}

#[tokio::test]
async fn host_names_are_checked_before_resolving() {
    let (addr, connections) = spawn_server(OK.to_string());
    let filter = ResolvedAddressFilter::new(|_| true, |_| true)
        .with_lookup(|_| vec![IpAddr::from([127, 0, 0, 1])])
        .with_host_name_check(|host| !host.trim_end_matches('.').eq_ignore_ascii_case("metadata"));
    let pool = pool(filter);

    for host in ["metadata", "METADATA."] {
        let error = get(&pool, &format!("http://{host}:{}/", addr.port()))
            .await
            .expect_err("metadata host name should be refused");
        assert!(
            matches!(
                error,
                RouteAwareRequestError::DisallowedAddress(DisallowedAddressError::HostName(_))
            ),
            "{error:?}"
        );
    }
    assert_eq!(connections.load(Ordering::SeqCst), 0);
}

/// Through a proxy, the proxy's own name is exempt from the resolved-address
/// check while the target's host name is still checked.
#[tokio::test]
async fn proxied_requests_check_target_names_and_exempt_the_proxy() {
    let (proxy_addr, proxy_connections) = spawn_server(OK.to_string());
    let target = "http://target.test/via-proxy";
    let blocked_target = "http://metadata/via-proxy";
    for url in [target, blocked_target] {
        crate::cache_system_proxy_route_for_test(
            url,
            format!("http://proxy.test:{}", proxy_addr.port()),
        );
    }
    let filter = ResolvedAddressFilter::new(|_| false, |_| false)
        .with_lookup(|_| vec![IpAddr::from([127, 0, 0, 1])])
        .with_host_name_check(|host| host != "metadata");
    let pool = RouteAwareClientPool::new(
        HttpClientFactory::new(OutboundProxyPolicy::RespectSystemProxy),
        ClientRouteClass::Other,
    )
    .with_resolved_address_filter(filter);

    assert_eq!(get(&pool, target).await.unwrap(), "ok");
    assert_disallowed(get(&pool, blocked_target).await);
    assert_eq!(proxy_connections.load(Ordering::SeqCst), 1);
}
