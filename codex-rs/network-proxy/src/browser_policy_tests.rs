use super::*;
use pretty_assertions::assert_eq;

#[tokio::test]
async fn explicit_denial_never_calls_the_destination_resolver() {
    use crate::NetworkProxyConfig;
    use crate::RemoteNetworkProxyConfig;
    use crate::RemoteNetworkProxyLaunchConfig;
    let config = NetworkProxyConfig {
        enabled: true,
        domains: Some(
            serde_json::from_value(serde_json::json!({"blocked.example":"deny"})).unwrap(),
        ),
        ..NetworkProxyConfig::default()
    };
    let policy = NetworkProxyState::from_remote_launch_config(RemoteNetworkProxyLaunchConfig::new(
        RemoteNetworkProxyConfig::from_effective_config(&config).unwrap(),
    ))
    .unwrap();
    for url in ["https://blocked.example/", "https://blocked.example./"] {
        let lookup_called = std::cell::Cell::new(false);
        let result = resolve_browser_destination_with_lookup(url, &policy, |_, _| {
            lookup_called.set(true);
            std::future::ready(Ok(vec!["1.1.1.1:443".parse().unwrap()]))
        })
        .await;
        assert_eq!(result.err(), Some(BrowserPolicyError::PolicyDenied));
        assert!(
            !lookup_called.get(),
            "explicitly denied destinations must not reach DNS"
        );
    }
}

#[tokio::test]
async fn allowed_host_still_validates_the_exact_resolver_answers() {
    use crate::NetworkProxyConfig;
    use crate::RemoteNetworkProxyConfig;
    use crate::RemoteNetworkProxyLaunchConfig;
    let config = NetworkProxyConfig {
        enabled: true,
        // Disable the native policy's DNS preflight in this offline fixture.
        // The browser's own public-address requirement must still hold.
        allow_local_binding: true,
        domains: Some(
            serde_json::from_value(serde_json::json!({"allowed.example":"allow"})).unwrap(),
        ),
        ..NetworkProxyConfig::default()
    };
    let policy = NetworkProxyState::from_remote_launch_config(RemoteNetworkProxyLaunchConfig::new(
        RemoteNetworkProxyConfig::from_effective_config(&config).unwrap(),
    ))
    .unwrap();
    for (address, expected) in [
        (
            "1.1.1.1:443",
            Ok(vec!["1.1.1.1:443".parse::<SocketAddr>().unwrap()]),
        ),
        (
            "10.0.0.1:443",
            Err(BrowserPolicyError::NonPublicDestination),
        ),
    ] {
        let address: SocketAddr = address.parse().unwrap();
        let result = resolve_browser_destination_with_lookup(
            "https://allowed.example/",
            &policy,
            |host, port| {
                assert_eq!((host.as_str(), port), ("allowed.example", 443));
                std::future::ready(Ok(vec![address]))
            },
        )
        .await;
        assert_eq!(
            result.map(|destination| destination.addresses().to_vec()),
            expected
        );
    }
}

#[test]
fn canonical_public_urls_preserve_queries_but_not_fragments() {
    assert_eq!(
        parse_destination("https://Example.COM/a?q=1#x")
            .unwrap()
            .as_str(),
        "https://example.com/a?q=1"
    );
}

#[test]
fn rejects_non_web_credentials_ports_and_ambiguous_input() {
    for url in [
        "file:///etc/passwd",
        "ftp://example.com/",
        "https://u:p@example.com/",
        "https://example.com:8443/",
        " https://example.com/",
        "https://example.com/\n",
    ] {
        assert_eq!(
            parse_destination(url),
            Err(BrowserPolicyError::InvalidDestination),
            "{url}"
        );
    }
}

#[test]
fn rejects_private_obfuscated_and_special_destinations() {
    for url in [
        "http://localhost/",
        "http://a.localhost./",
        "http://2130706433/",
        "http://0x7f000001/",
        "http://0177.0.0.1/",
        "http://169.254.169.254/",
        "http://100.99.88.49/",
        "http://[::ffff:127.0.0.1]/",
        "http://[64:ff9b::7f00:1]/",
        "http://[2002:7f00:1::]/",
        "http://[2001:db8::1]/",
    ] {
        assert_eq!(
            parse_destination(url),
            Err(BrowserPolicyError::NonPublicDestination),
            "{url}"
        );
    }
}

#[test]
fn one_private_dns_answer_denies_the_entire_destination() {
    let public: SocketAddr = "93.184.216.34:443".parse().unwrap();
    let private: SocketAddr = "10.0.0.1:443".parse().unwrap();
    assert_eq!(validate_addresses(&[public]), Ok(()));
    assert_eq!(
        validate_addresses(&[public, private]),
        Err(BrowserPolicyError::NonPublicDestination)
    );
    assert_eq!(
        validate_addresses(&[]),
        Err(BrowserPolicyError::ResolutionFailed)
    );
    assert_eq!(
        validate_addresses(&[public; 17]),
        Err(BrowserPolicyError::ResolutionFailed)
    );
}

#[test]
fn accepts_public_ipv4_and_ordinary_ipv6_only() {
    for ip in [
        "8.8.8.8",
        "1.1.1.1",
        "2606:4700:4700::1111",
        "2001:4860:4860::8888",
    ] {
        assert!(is_public_browser_ip(ip.parse().unwrap()), "{ip}");
    }
    for ip in [
        "192.88.99.1",
        "198.18.0.1",
        "224.0.0.1",
        "::ffff:8.8.8.8",
        "2001::1",
        "3fff::1",
        "fc00::1",
        "fe80::1",
    ] {
        assert!(!is_public_browser_ip(ip.parse().unwrap()), "{ip}");
    }
}

#[tokio::test]
async fn existing_denial_wins_without_approval_override() {
    use crate::NetworkProxyConfig;
    use crate::RemoteNetworkProxyConfig;
    use crate::RemoteNetworkProxyLaunchConfig;
    for domains in [
        serde_json::json!({"1.1.1.1":"deny"}),
        serde_json::json!({"example.com":"allow"}),
    ] {
        let config = NetworkProxyConfig {
            enabled: true,
            domains: Some(serde_json::from_value(domains).unwrap()),
            ..NetworkProxyConfig::default()
        };
        let policy =
            NetworkProxyState::from_remote_launch_config(RemoteNetworkProxyLaunchConfig::new(
                RemoteNetworkProxyConfig::from_effective_config(&config).unwrap(),
            ))
            .unwrap();
        assert!(matches!(
            resolve_browser_destination("https://1.1.1.1/", &policy).await,
            Err(BrowserPolicyError::PolicyDenied)
        ));
    }
}

#[tokio::test]
async fn allow_returns_exact_connection_addresses_without_network_io() {
    use crate::NetworkProxyConfig;
    use crate::RemoteNetworkProxyConfig;
    use crate::RemoteNetworkProxyLaunchConfig;
    let config = NetworkProxyConfig {
        enabled: true,
        domains: Some(serde_json::from_value(serde_json::json!({"1.1.1.1":"allow"})).unwrap()),
        ..NetworkProxyConfig::default()
    };
    let policy = NetworkProxyState::from_remote_launch_config(RemoteNetworkProxyLaunchConfig::new(
        RemoteNetworkProxyConfig::from_effective_config(&config).unwrap(),
    ))
    .unwrap();
    let destination = resolve_browser_destination("https://1.1.1.1/", &policy)
        .await
        .unwrap();
    assert_eq!(
        destination.addresses(),
        &["1.1.1.1:443".parse::<SocketAddr>().unwrap()]
    );
    assert_eq!(destination.host(), "1.1.1.1");
}
