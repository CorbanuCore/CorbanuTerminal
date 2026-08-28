use super::*;
use codex_network_proxy::NetworkProxyConfig;
use codex_network_proxy::RemoteNetworkProxyConfig;
use codex_network_proxy::RemoteNetworkProxyLaunchConfig;
use pretty_assertions::assert_eq;

fn policy() -> NetworkProxyState {
    NetworkProxyState::from_remote_launch_config(RemoteNetworkProxyLaunchConfig::new(
        RemoteNetworkProxyConfig::from_effective_config(&NetworkProxyConfig {
            enabled: true,
            ..NetworkProxyConfig::default()
        })
        .unwrap(),
    ))
    .unwrap()
}

#[tokio::test]
async fn private_urls_abort_without_http_or_authority_dispatch() {
    let policy = policy();
    let mut broker = Broker::new(&policy, AuthorityEpoch::new([1; 16], 0, 0).unwrap());
    let reply = broker
        .request(1, "http://169.254.169.254/", || {
            panic!("no dispatch on private URL")
        })
        .await
        .unwrap();
    assert_eq!(
        serde_json::to_value(reply).unwrap(),
        serde_json::json!({"id":1,"denied":true,"status":403,"headers":{},"body":""})
    );
    assert!(broker.visited.is_empty());
    assert!(broker.artifacts.is_empty());
    assert!(matches!(
        broker.request(3, "https://example.com/", || Ok(())).await,
        Err(BrowserError::ResourceLimit)
    ));
}

#[tokio::test]
async fn wire_parser_rejects_overlong_unterminated_and_extended_messages() {
    for input in [
        b"{\"type\":\"failed\",\"approved\":true}\n".to_vec(),
        b"{\"type\":\"failed\"}".to_vec(),
        vec![b'x'; MAX_LINE + 1],
    ] {
        assert!(read_message(&mut input.as_slice()).await.is_err());
    }
    assert!(matches!(
        read_message(&mut b"{\"type\":\"failed\"}\n".as_slice()).await,
        Ok(WorkerMessage::Failed {})
    ));
}

#[test]
fn encoded_results_are_bounded_and_not_implicitly_trusted() {
    assert_eq!(decode_body("aHRtbA=="), Ok(b"html".to_vec()));
    assert_eq!(decode_body("%%%"), Err(BrowserError::InvalidWorkerResponse));
    assert_eq!(
        decode_body(&STANDARD.encode(vec![0; MAX_BODY + 1])),
        Err(BrowserError::ResourceLimit)
    );
}

#[test]
fn downloads_do_not_depend_on_server_filename_or_attachment_header() {
    for media in [
        "application/octet-stream",
        "application/pdf",
        "application/zip",
        "unknown/type",
    ] {
        assert!(is_download(media, None));
    }
    assert!(is_download(
        "text/html",
        Some("ATTACHMENT; filename=../../secret")
    ));
    for media in [
        "text/html; charset=utf-8",
        "application/javascript",
        "image/png",
        "font/woff2",
    ] {
        assert!(!is_download(media, None));
    }
}

#[tokio::test]
async fn connection_resolver_has_no_hostname_or_system_dns_fallback() {
    use reqwest::dns::Resolve;
    let addresses = vec!["1.1.1.1:443".parse::<SocketAddr>().unwrap()];
    let resolver = PinnedAddresses(addresses.clone());
    for name in ["example.com", "example.com.", "unexpected.invalid"] {
        let result = resolver
            .resolve(name.parse().unwrap())
            .await
            .unwrap()
            .collect::<Vec<_>>();
        assert_eq!(result, addresses);
    }
}
