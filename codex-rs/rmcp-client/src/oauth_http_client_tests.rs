use std::io::Read;
use std::io::Write;
use std::net::SocketAddr;
use std::net::TcpListener;
use std::sync::atomic::AtomicUsize;

use codex_exec_server::RouteAwareHttpClient;
use codex_http_client::HttpClientFactory;
use codex_http_client::OutboundProxyPolicy;
use pretty_assertions::assert_eq;

use super::*;

const OK: &str = "HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok";
const PUBLIC_SERVER: &str = "https://mcp.example.com/mcp";

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
            let mut buffer = [0; 4096];
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

fn adapter(server_url: &str) -> OAuthHttpClientAdapter {
    OAuthHttpClientAdapter::new(
        Arc::new(RouteAwareHttpClient::new(HttpClientFactory::new(
            OutboundProxyPolicy::ReqwestDefault,
        ))),
        HeaderMap::new(),
        server_url,
    )
}

async fn send(
    adapter: &OAuthHttpClientAdapter,
    method: &str,
    url: &str,
    redirect_policy: OAuthHttpRedirectPolicy,
) -> Result<HttpResponse, OAuthHttpClientError> {
    let request = oauth2::http::Request::builder()
        .method(method)
        .uri(url)
        .body(Vec::new())
        .expect("test request");
    adapter
        .execute_request(request, redirect_policy, Some(Duration::from_secs(5)))
        .await
}

fn assert_blocked(result: Result<HttpResponse, OAuthHttpClientError>) {
    let error = match result {
        Ok(response) => panic!("expected a blocked request, got HTTP {}", response.status()),
        Err(error) => error.to_string(),
    };
    assert!(
        error.starts_with("MCP OAuth request blocked:"),
        "unexpected error: {error}"
    );
}

#[test]
fn policy_follows_the_server_url() {
    let loopback = |addr: &str| HttpAddressPolicy {
        allow_loopback: true,
        allow_private: false,
        allowed_address: Some(addr.parse().unwrap()),
    };
    for (url, expected) in [
        ("https://mcp.example.com/mcp", HttpAddressPolicy::default()),
        ("not a url", HttpAddressPolicy::default()),
        ("http://127.0.0.1:8080/mcp", loopback("127.0.0.1")),
        ("http://[::1]:8080/mcp", loopback("::1")),
        (
            "http://localhost.:8080/mcp",
            HttpAddressPolicy {
                allow_loopback: true,
                ..HttpAddressPolicy::default()
            },
        ),
        (
            "http://dev.localhost/mcp",
            HttpAddressPolicy {
                allow_loopback: true,
                ..HttpAddressPolicy::default()
            },
        ),
        (
            "http://10.0.0.5/mcp",
            HttpAddressPolicy {
                allow_loopback: false,
                allow_private: true,
                allowed_address: Some("10.0.0.5".parse().unwrap()),
            },
        ),
        (
            "http://[fd12::5]/mcp",
            HttpAddressPolicy {
                allow_loopback: false,
                allow_private: true,
                allowed_address: Some("fd12::5".parse().unwrap()),
            },
        ),
        (
            "http://169.254.10.10/mcp",
            HttpAddressPolicy {
                allowed_address: Some("169.254.10.10".parse().unwrap()),
                ..HttpAddressPolicy::default()
            },
        ),
        // A host name's DNS answers never widen the policy.
        ("http://mcp.corp.internal/mcp", HttpAddressPolicy::default()),
    ] {
        assert_eq!(oauth_address_policy(url), expected, "{url}");
    }
}

#[tokio::test]
async fn local_servers_keep_working() {
    let (addr, connections) = spawn_server(OK.to_string());
    let port = addr.port();
    for (server_url, request_url) in [
        (format!("http://{addr}/mcp"), format!("http://{addr}/token")),
        (
            format!("http://localhost:{port}/mcp"),
            format!("http://localhost:{port}/token"),
        ),
        (
            format!("http://localhost:{port}/mcp"),
            format!("http://{addr}/token"),
        ),
    ] {
        let response = send(
            &adapter(&server_url),
            "POST",
            &request_url,
            OAuthHttpRedirectPolicy::Follow,
        )
        .await
        .unwrap_or_else(|error| panic!("{server_url} -> {request_url}: {error}"));
        assert_eq!(response.body(), b"ok");
    }
    assert_eq!(connections.load(Ordering::SeqCst), 3);
}

#[tokio::test]
async fn public_server_cannot_reach_loopback_literal() {
    let (addr, connections) = spawn_server(OK.to_string());
    let adapter = adapter(PUBLIC_SERVER);
    for url in [
        format!("http://{addr}/.well-known/oauth-authorization-server"),
        format!("http://[::ffff:127.0.0.1]:{}/register", addr.port()),
        format!("http://2130706433:{}/token", addr.port()),
    ] {
        assert_blocked(send(&adapter, "GET", &url, OAuthHttpRedirectPolicy::Stop).await);
    }
    assert_eq!(connections.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn public_server_cannot_reach_host_name_resolving_to_loopback() {
    let (addr, connections) = spawn_server(OK.to_string());
    let adapter = adapter(PUBLIC_SERVER);
    for host in ["localhost", "localhost.", "dev.localhost"] {
        let url = format!("http://{host}:{}/token", addr.port());
        assert_blocked(send(&adapter, "POST", &url, OAuthHttpRedirectPolicy::Follow).await);
    }
    assert_eq!(connections.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn metadata_addresses_are_blocked_even_for_private_servers() {
    let adapter = adapter("http://10.0.0.5/mcp");
    for url in [
        "http://169.254.169.254/latest/meta-data/",
        "http://[::ffff:169.254.169.254]/latest/meta-data/",
        "http://[fd00:ec2::254]/latest/meta-data/",
        "http://169.254.1.1/",
        "http://0.0.0.0/",
        "http://[::]/",
        "http://metadata.google.internal/computeMetadata/v1/",
        "http://metadata/",
    ] {
        assert_blocked(send(&adapter, "GET", url, OAuthHttpRedirectPolicy::Stop).await);
    }
}

#[tokio::test]
async fn redirect_to_private_address_is_blocked() {
    let (addr, connections) = spawn_server(
        "HTTP/1.1 307 Temporary Redirect\r\nLocation: http://10.255.255.1/token\r\n\
         Content-Length: 0\r\nConnection: close\r\n\r\n"
            .to_string(),
    );
    let url = format!("http://{addr}/token");
    assert_blocked(
        send(
            &adapter(&format!("http://{addr}/mcp")),
            "POST",
            &url,
            OAuthHttpRedirectPolicy::Follow,
        )
        .await,
    );
    assert_eq!(connections.load(Ordering::SeqCst), 1);
}
