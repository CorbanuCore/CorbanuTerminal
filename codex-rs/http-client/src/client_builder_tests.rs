use super::*;
use http::HeaderValue;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;

#[tokio::test]
async fn fallible_ca_fallback_keeps_no_redirect_headers_and_logging_policy() {
    use pretty_assertions::assert_eq;
    let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let address = listener.local_addr().unwrap();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut request = Vec::new();
        while !request.windows(4).any(|bytes| bytes == b"\r\n\r\n") {
            let mut buffer = [0; 1024];
            let count = stream.read(&mut buffer).unwrap();
            assert!(count > 0);
            request.extend_from_slice(&buffer[..count]);
        }
        stream.write_all(b"HTTP/1.1 308 Permanent Redirect\r\nLocation: /forbidden\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").unwrap();
        String::from_utf8(request).unwrap()
    });
    let builder = HttpClientBuilder::new()
        .without_redirects()
        .without_request_logging()
        .with_chatgpt_cloudflare_cookie_store()
        .default_headers(HeaderMap::from_iter([(
            "x-fallback".parse().unwrap(),
            HeaderValue::from_static("kept"),
        )]));
    let client = builder
        .try_build_with_custom_ca_fallback_using(
            ProxyRouting::Direct,
            |_| {
                Err(BuildCustomCaTransportError::InvalidCaFile {
                    source_env: "TEST_CA_ENV",
                    path: PathBuf::from("synthetic.pem"),
                    detail: "forced fallback".into(),
                })
            },
            reqwest::ClientBuilder::build,
        )
        .unwrap();
    assert!(!client.request_logging_enabled());
    assert_eq!(
        client
            .post(format!("http://{address}/"))
            .send()
            .await
            .unwrap()
            .status()
            .as_u16(),
        308
    );
    let request = server.join().unwrap();
    assert!(
        request
            .lines()
            .any(|line| line.eq_ignore_ascii_case("x-fallback: kept"))
    );
}

#[test]
fn failed_system_root_fallback_returns_error_instead_of_bare_client() {
    let result = HttpClientBuilder::new()
        .without_redirects()
        .try_build_with_custom_ca_fallback_using(
            ProxyRouting::Direct,
            |_| {
                Err(BuildCustomCaTransportError::InvalidCaFile {
                    source_env: "TEST_CA_ENV",
                    path: PathBuf::from("synthetic.pem"),
                    detail: "forced fallback".into(),
                })
            },
            |builder| builder.user_agent("\n").build(),
        );
    assert!(matches!(
        result,
        Err(BuildCustomCaTransportError::BuildClientWithSystemRoots(_))
    ));
}

#[tokio::test]
async fn custom_ca_fallback_preserves_builder_configuration() {
    let listener =
        std::net::TcpListener::bind(("127.0.0.1", 0)).expect("HTTP listener should bind");
    let address = listener
        .local_addr()
        .expect("HTTP listener should have an address");
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("HTTP listener should accept");
        let mut request = Vec::new();
        let mut chunk = [0_u8; 1024];
        while !request.windows(4).any(|window| window == b"\r\n\r\n") {
            let bytes_read = stream.read(&mut chunk).expect("HTTP request should read");
            assert!(bytes_read > 0, "HTTP request should include headers");
            request.extend_from_slice(&chunk[..bytes_read]);
        }
        stream
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
            .expect("HTTP listener should write response");
        String::from_utf8(request).expect("HTTP request should be UTF-8")
    });
    let mut headers = HeaderMap::new();
    headers.insert("x-builder-test", HeaderValue::from_static("preserved"));
    let client = HttpClientBuilder::new()
        .default_headers(headers)
        .build_with_custom_ca_fallback_using(ProxyRouting::Direct, |_| {
            Err(BuildCustomCaTransportError::InvalidCaFile {
                source_env: "TEST_CA_ENV",
                path: PathBuf::from("invalid-test-ca.pem"),
                detail: "synthetic invalid CA".to_string(),
            })
        });

    let response = client
        .get(format!("http://{address}/fallback"))
        .send()
        .await
        .expect("fallback client should send request");
    assert!(response.status().is_success());
    let request = server.join().expect("HTTP listener should finish");
    assert!(
        request
            .lines()
            .any(|line| line.eq_ignore_ascii_case("x-builder-test: preserved"))
    );
}
