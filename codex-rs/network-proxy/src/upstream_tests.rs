use super::*;
use crate::config::NetworkProxyConfig;
use crate::connect_policy::PinnedPeers;
use crate::state::network_proxy_state_for_policy;
use pretty_assertions::assert_eq;
use rama_http::StatusCode;
use rama_http::Version;
use rama_net::address::Host;
use rama_net::address::HostWithPort;
use rama_tls_rustls::dep::pki_types::CertificateDer;
use rama_tls_rustls::dep::pki_types::PrivateKeyDer;
use rama_tls_rustls::dep::pki_types::pem::PemObject;
use rama_tls_rustls::dep::rcgen::BasicConstraints;
use rama_tls_rustls::dep::rcgen::CertificateParams;
use rama_tls_rustls::dep::rcgen::DistinguishedName;
use rama_tls_rustls::dep::rcgen::DnType;
use rama_tls_rustls::dep::rcgen::ExtendedKeyUsagePurpose;
use rama_tls_rustls::dep::rcgen::IsCa;
use rama_tls_rustls::dep::rcgen::Issuer;
use rama_tls_rustls::dep::rcgen::KeyPair;
use rama_tls_rustls::dep::rcgen::KeyUsagePurpose;
use rama_tls_rustls::dep::rcgen::PKCS_ECDSA_P256_SHA256;
use rama_tls_rustls::dep::tokio_rustls::TlsAcceptor;
use std::collections::HashMap;
use std::fs;
use std::sync::Arc;
use tempfile::tempdir;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;

fn generate_ca(common_name: &str) -> (String, KeyPair) {
    let mut params = CertificateParams::default();
    params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    params.key_usages = vec![
        KeyUsagePurpose::KeyCertSign,
        KeyUsagePurpose::DigitalSignature,
        KeyUsagePurpose::KeyEncipherment,
    ];
    let mut distinguished_name = DistinguishedName::new();
    distinguished_name.push(DnType::CommonName, common_name);
    params.distinguished_name = distinguished_name;
    let key_pair = KeyPair::generate_for(&PKCS_ECDSA_P256_SHA256).unwrap();
    let cert = params.self_signed(&key_pair).unwrap();
    (cert.pem(), key_pair)
}

#[tokio::test]
async fn mitm_upstream_client_trusts_startup_custom_ca() {
    ensure_rustls_crypto_provider();
    let temp_dir = tempdir().unwrap();
    let startup_ca_path = temp_dir.path().join("startup-ca.pem");
    let managed_ca_path = temp_dir.path().join("managed-ca.pem");
    let (startup_ca_pem, startup_ca_key) = generate_ca("startup CA");
    let (managed_ca_pem, _) = generate_ca("managed MITM CA");
    fs::write(&startup_ca_path, &startup_ca_pem).unwrap();
    fs::write(&managed_ca_path, managed_ca_pem).unwrap();

    let issuer = Issuer::from_ca_cert_pem(&startup_ca_pem, startup_ca_key).unwrap();
    let mut server_params = CertificateParams::new(vec!["localhost".to_string()]).unwrap();
    server_params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ServerAuth];
    server_params.key_usages = vec![
        KeyUsagePurpose::DigitalSignature,
        KeyUsagePurpose::KeyEncipherment,
    ];
    let server_key = KeyPair::generate_for(&PKCS_ECDSA_P256_SHA256).unwrap();
    let server_cert = server_params.signed_by(&server_key, &issuer).unwrap();
    let server_cert = CertificateDer::from_pem_slice(server_cert.pem().as_bytes()).unwrap();
    let server_key = PrivateKeyDer::from_pem_slice(server_key.serialize_pem().as_bytes()).unwrap();
    let mut server_config =
        rustls::ServerConfig::builder_with_protocol_versions(rustls::ALL_VERSIONS)
            .with_no_client_auth()
            .with_single_cert(vec![server_cert], server_key)
            .unwrap();
    server_config.alpn_protocols = vec![b"http/1.1".to_vec()];

    let env = HashMap::from([(
        "SSL_CERT_FILE",
        startup_ca_path.to_string_lossy().into_owned(),
    )]);
    let roots =
        crate::certs::upstream_tls_root_store_for_cert_path(&managed_ca_path, &env).unwrap();
    let baseline_roots =
        crate::certs::upstream_tls_root_store_for_cert_path(&managed_ca_path, &HashMap::new())
            .unwrap();
    assert_eq!(roots.len(), baseline_roots.len() + 1);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let acceptor = TlsAcceptor::from(Arc::new(server_config));
    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let mut stream = acceptor.accept(stream).await.unwrap();
        let mut request = [0; 4096];
        let bytes_read = stream.read(&mut request).await.unwrap();
        assert!(bytes_read > 0);
        stream
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
            .await
            .unwrap();
    });

    let mut config = NetworkProxyConfig::default();
    config.set_allowed_domains(vec!["localhost".to_string()]);
    let state = Arc::new(network_proxy_state_for_policy(config));
    let client = UpstreamClient::direct_with_tls_root_store(state, roots);
    let response = client
        .serve(
            Request::builder()
                .version(Version::HTTP_2)
                .uri(format!("https://localhost:{}/", address.port()))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    server.await.unwrap();
}

#[test]
fn inherited_upstream_proxy_is_bypassed_for_non_public_targets() {
    let proxy = ProxyAddress::try_from("http://127.0.0.1:43128").unwrap();
    let config = ProxyConfig {
        http: Some(proxy.clone()),
        https: Some(proxy),
        all: None,
    };

    for target in [
        HostWithPort::new(Host::LOCALHOST_NAME, 8080),
        HostWithPort::new(Host::LOCALHOST_IPV4, 8080),
        HostWithPort::new(Host::Address("10.0.0.1".parse().unwrap()), 8080),
    ] {
        assert_eq!(config.proxy_for_target(&target, /*is_secure*/ false), None);
        assert_eq!(config.proxy_for_target(&target, /*is_secure*/ true), None);
    }

    let public = HostWithPort::new(Host::EXAMPLE_NAME, 443);
    assert!(
        config
            .proxy_for_target(&public, /*is_secure*/ true)
            .is_some()
    );
}

/// PF-33-S02 fixture: a loopback TLS server presenting a leaf for `name`,
/// signed by a test CA that the returned root store trusts.
struct PinnedTlsServer {
    address: std::net::SocketAddr,
    accepted: Arc<std::sync::atomic::AtomicUsize>,
    roots: Arc<rustls::RootCertStore>,
}

impl PinnedTlsServer {
    /// Requests whose path is `/hang` never get an answer.
    async fn start(name: &str) -> Self {
        ensure_rustls_crypto_provider();
        let (ca_pem, ca_key) = generate_ca("PF-33-S02 test CA");
        let issuer = Issuer::from_ca_cert_pem(&ca_pem, ca_key).unwrap();
        let mut params = CertificateParams::new(vec![name.to_string()]).unwrap();
        params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ServerAuth];
        let key = KeyPair::generate_for(&PKCS_ECDSA_P256_SHA256).unwrap();
        let cert = params.signed_by(&key, &issuer).unwrap();
        let cert = CertificateDer::from_pem_slice(cert.pem().as_bytes()).unwrap();
        let key = PrivateKeyDer::from_pem_slice(key.serialize_pem().as_bytes()).unwrap();
        let mut config = rustls::ServerConfig::builder_with_protocol_versions(rustls::ALL_VERSIONS)
            .with_no_client_auth()
            .with_single_cert(vec![cert], key)
            .unwrap();
        config.alpn_protocols = vec![b"http/1.1".to_vec()];
        let mut roots = rustls::RootCertStore::empty();
        roots
            .add(CertificateDer::from_pem_slice(ca_pem.as_bytes()).unwrap())
            .unwrap();

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let accepted = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let acceptor = TlsAcceptor::from(Arc::new(config));
        let counter = accepted.clone();
        tokio::spawn(async move {
            while let Ok((stream, _)) = listener.accept().await {
                counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                let acceptor = acceptor.clone();
                tokio::spawn(async move {
                    let Ok(mut stream) = acceptor.accept(stream).await else {
                        return;
                    };
                    let mut request = [0; 4096];
                    let Ok(read) = stream.read(&mut request).await else {
                        return;
                    };
                    if request[..read].starts_with(b"GET /hang ") {
                        std::future::pending::<()>().await;
                    }
                    let _ = stream
                        .write_all(
                            b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                        )
                        .await;
                });
            }
        });
        Self {
            address,
            accepted,
            roots: Arc::new(roots),
        }
    }

    fn accepted(&self) -> usize {
        self.accepted.load(std::sync::atomic::Ordering::SeqCst)
    }
}

fn pf_33_s02_guarded_client(
    roots: Arc<rustls::RootCertStore>,
    proxy: ProxyConfig,
) -> UpstreamClient {
    let mut config = NetworkProxyConfig::default();
    config.set_url_destination_policy(/*enabled*/ true);
    let state = Arc::new(network_proxy_state_for_policy(config));
    UpstreamClient::new(
        proxy,
        TargetCheckedTcpConnector::allowing_loopback_peers_for_tests(state),
        roots,
    )
}

fn pf_33_s02_request(host: &str, port: u16, path: &str, pin: PinnedPeers) -> Request<Body> {
    let mut request = Request::builder()
        .uri(format!("https://{host}:{port}{path}"))
        .body(Body::empty())
        .unwrap();
    request.extensions_mut().insert(pin);
    request
}

/// RFC 6761 `.invalid` names never resolve, so a response proves the request
/// reached the pinned answer rather than a DNS result.
const PINNED_NAME: &str = "pinned.invalid";

#[tokio::test]
async fn pf_33_s02_https_request_uses_pinned_peer_and_dials_fresh_each_time() {
    let server = PinnedTlsServer::start(PINNED_NAME).await;
    let port = server.address.port();
    let client = pf_33_s02_guarded_client(server.roots.clone(), ProxyConfig::default());
    let pin = PinnedPeers::new(PINNED_NAME, port, [server.address.ip()]);

    for _ in 0..2 {
        let response = client
            .serve(pf_33_s02_request(PINNED_NAME, port, "/", pin.clone()))
            .await
            .expect("pinned request");
        assert_eq!(response.status(), StatusCode::OK);
    }

    // No pooled connection carries a later request: each one dialled again.
    assert_eq!(server.accepted(), 2);
}

#[tokio::test]
async fn pf_33_s02_pinned_peer_must_prove_the_requested_tls_identity() {
    // The checked address answers with a certificate for another name.
    let server = PinnedTlsServer::start("other.invalid").await;
    let port = server.address.port();
    let client = pf_33_s02_guarded_client(server.roots.clone(), ProxyConfig::default());
    let pin = PinnedPeers::new(PINNED_NAME, port, [server.address.ip()]);

    let result = client
        .serve(pf_33_s02_request(PINNED_NAME, port, "/", pin))
        .await;

    assert!(
        result.is_err(),
        "TLS identity mismatch must fail: {result:?}"
    );
    assert_eq!(server.accepted(), 1, "the pinned peer was dialled");
}

#[tokio::test]
async fn pf_33_s02_aborting_one_request_leaves_its_sibling_working() {
    let server = PinnedTlsServer::start(PINNED_NAME).await;
    let port = server.address.port();
    let client = pf_33_s02_guarded_client(server.roots.clone(), ProxyConfig::default());
    let pin = PinnedPeers::new(PINNED_NAME, port, [server.address.ip()]);

    let hanging = tokio::spawn({
        let client = client.clone();
        let request = pf_33_s02_request(PINNED_NAME, port, "/hang", pin.clone());
        async move { client.serve(request).await }
    });
    while server.accepted() == 0 {
        tokio::task::yield_now().await;
    }
    let sibling = client
        .serve(pf_33_s02_request(PINNED_NAME, port, "/", pin))
        .await
        .expect("sibling request");
    hanging.abort();

    assert_eq!(sibling.status(), StatusCode::OK);
    assert!(hanging.await.unwrap_err().is_cancelled());
}

#[tokio::test]
async fn pf_33_s02_guard_refuses_inherited_upstream_proxy() {
    let server = PinnedTlsServer::start(PINNED_NAME).await;
    let port = server.address.port();
    let proxy = ProxyAddress::try_from("http://proxy.invalid:3128").unwrap();
    let proxy = ProxyConfig {
        http: None,
        https: Some(proxy),
        all: None,
    };
    let client = pf_33_s02_guarded_client(server.roots.clone(), proxy);
    assert!(client.has_upstream_proxy());
    let pin = PinnedPeers::new(PINNED_NAME, port, [server.address.ip()]);

    let result = client
        .serve(pf_33_s02_request(PINNED_NAME, port, "/", pin))
        .await;

    let message = format!("{:?}", result.expect_err("proxy route must be refused"));
    assert!(
        message.contains("upstream proxy"),
        "unexpected error: {message}"
    );
    assert_eq!(server.accepted(), 0);
    assert!(
        !pf_33_s02_guarded_client(server.roots.clone(), ProxyConfig::default())
            .has_upstream_proxy()
    );
}
