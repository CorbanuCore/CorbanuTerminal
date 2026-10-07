use super::*;
use serde_json::json;
use std::io::Write;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use tracing_subscriber::Layer;
use tracing_subscriber::layer::SubscriberExt;

#[test]
fn trace_summary_never_contains_request_content() {
    let secret = "prompt-and-credential-must-not-appear";
    let json_request = Request::new(Method::POST, "https://example.test".to_string())
        .with_json(&json!({ "prompt": secret }));
    let encoded_request = json_request
        .clone()
        .into_prepared()
        .expect("encode request");
    let raw_request = Request::new(Method::POST, "https://example.test".to_string())
        .with_raw_body(secret.as_bytes().to_vec());

    for summary in [
        request_body_for_trace(&json_request),
        request_body_for_trace(&encoded_request),
        request_body_for_trace(&raw_request),
    ] {
        assert!(!summary.contains(secret));
        assert!(summary.contains("body:"));
        assert!(summary.contains("bytes"));
    }
}

#[tokio::test]
async fn enabled_request_logging_emits_redacted_transport_url_but_not_body() {
    let logs = capture_transport_logs(HttpClient::new(test_reqwest_client())).await;

    assert!(logs.contains("log capture sentinel"));
    assert!(logs.contains("/request?token=REDACTED"));
    assert!(!logs.contains("url-secret"));
    assert!(!logs.contains("body-secret"));
    assert!(logs.contains("json body:"));
    assert!(logs.contains("bytes"));
}

#[tokio::test]
async fn disabled_request_logging_suppresses_transport_url_and_body() {
    let logs = capture_transport_logs(HttpClient::new_without_request_logging(
        test_reqwest_client(),
    ))
    .await;

    assert!(logs.contains("log capture sentinel"));
    assert!(!logs.contains("url-secret"));
    assert!(!logs.contains("body-secret"));
}

/// The URL in an HTTP error is shown to users; credentials in it must not be.
#[tokio::test]
async fn http_error_url_redacts_query_values_and_userinfo() {
    for streaming in [false, true] {
        let (server_addr, server) = serve_one_401();
        let transport = ReqwestTransport::from_http_client(HttpClient::new(test_reqwest_client()));
        let request = Request::new(
            Method::GET,
            format!("http://user:fake-pass@{server_addr}/v1/models?key=fake-query-key"),
        );
        let result = if streaming {
            transport.stream(request).await.map(|_| ())
        } else {
            transport.execute(request).await.map(|_| ())
        };
        let Err(TransportError::Http { status, url, .. }) = result else {
            panic!("expected an HTTP error (streaming: {streaming})");
        };
        server.join().expect("server thread");

        assert_eq!(
            (status.as_u16(), url),
            (
                401,
                Some(format!(
                    "http://REDACTED:REDACTED@{server_addr}/v1/models?key=REDACTED"
                ))
            ),
            "streaming: {streaming}"
        );
    }
}

/// Answers one request with a 401 after reading its headers.
fn serve_one_401() -> (std::net::SocketAddr, std::thread::JoinHandle<()>) {
    let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).expect("server port should bind");
    let server_addr = listener.local_addr().expect("listener address");
    let server = std::thread::spawn(move || {
        use std::io::BufRead;
        let (socket, _) = listener.accept().expect("accept");
        let mut reader = std::io::BufReader::new(socket);
        let mut line = String::new();
        while reader.read_line(&mut line).expect("read request") > 0 && line != "\r\n" {
            line.clear();
        }
        reader
            .into_inner()
            .write_all(b"HTTP/1.1 401 Unauthorized\r\ncontent-length: 7\r\nconnection: close\r\n\r\nno auth")
            .expect("write response");
    });
    (server_addr, server)
}

fn test_reqwest_client() -> reqwest::Client {
    reqwest::Client::builder()
        .no_proxy()
        .build()
        .expect("HTTP client should build")
}

async fn capture_transport_logs(client: HttpClient) -> String {
    let unavailable_server =
        std::net::TcpListener::bind(("127.0.0.1", 0)).expect("server port should bind");
    let server_addr = unavailable_server
        .local_addr()
        .expect("server listener should have an address");
    drop(unavailable_server);
    let transport = ReqwestTransport::from_http_client(client);
    let log_buffer = Arc::new(Mutex::new(Vec::new()));
    let writer_buffer = Arc::clone(&log_buffer);
    let subscriber = tracing_subscriber::registry().with(
        tracing_subscriber::fmt::layer()
            .with_ansi(false)
            .with_writer(move || TestLogWriter(Arc::clone(&writer_buffer)))
            .with_filter(
                tracing_subscriber::filter::Targets::new()
                    .with_target("codex_http_client::transport", tracing::Level::TRACE),
            ),
    );
    let _guard = tracing::subscriber::set_default(subscriber);
    tracing::trace!(target: "codex_http_client::transport", "log capture sentinel");
    let mut request = Request::new(
        Method::POST,
        format!("http://{server_addr}/request?token=url-secret"),
    )
    .with_json(&json!({"token": "body-secret"}));
    request.timeout = Some(Duration::from_secs(1));

    let _ = transport.execute(request).await;

    String::from_utf8(
        log_buffer
            .lock()
            .expect("log buffer should not be poisoned")
            .clone(),
    )
    .expect("captured logs should be UTF-8")
}

#[derive(Clone)]
struct TestLogWriter(Arc<Mutex<Vec<u8>>>);

impl Write for TestLogWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0
            .lock()
            .map_err(|_| std::io::Error::other("log buffer should not be poisoned"))?
            .extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// PF-27-S05: a request carrying the broker's frame never leaves over the
/// caller's network client: with no broker it is refused, and with one it
/// goes to the broker's socket.
#[cfg(unix)]
#[tokio::test]
async fn pf_27_s05_broker_frame_requests_go_only_to_the_broker_socket() {
    use tokio::io::AsyncReadExt as _;
    use tokio::io::AsyncWriteExt as _;

    let network = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
    network.set_nonblocking(true).expect("nonblocking");
    let url = format!(
        "http://127.0.0.1:{}/v1/responses",
        network.local_addr().expect("addr").port()
    );
    let transport = ReqwestTransport::new(test_reqwest_client());
    let mut framed = Request::new(Method::POST, url.clone());
    framed.headers.insert(
        crate::MODEL_BROKER_FRAME_HEADER,
        http::HeaderValue::from_static("signed-frame"),
    );

    let refused = transport.execute(framed.clone()).await;
    assert!(
        matches!(&refused, Err(TransportError::Build(message)) if message.contains("credential broker")),
        "{refused:?}"
    );
    assert!(network.accept().is_err(), "nothing may reach the network");

    // Socket paths are short; the test temp directory may not be.
    let dir = tempfile::Builder::new()
        .prefix("pf27")
        .tempdir_in("/tmp")
        .expect("dir");
    let socket = dir.path().join("b.sock");
    let listener = tokio::net::UnixListener::bind(&socket).expect("bind socket");
    crate::install_model_broker_client(
        HttpClient::unix_socket(&socket, http::HeaderMap::new()).expect("broker client"),
    );
    let server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.expect("accept");
        let mut request = vec![0_u8; 4096];
        let read = stream.read(&mut request).await.expect("read");
        stream
            .write_all(b"HTTP/1.1 200 OK\r\ncontent-length: 2\r\n\r\nok")
            .await
            .expect("write");
        String::from_utf8_lossy(&request[..read]).into_owned()
    });
    let response = transport.execute(framed).await.expect("brokered response");
    assert_eq!(response.body.as_ref(), b"ok");
    let seen = server.await.expect("server");
    assert!(seen.starts_with("POST /v1/responses"), "{seen}");
    assert!(
        seen.contains("x-corbanu-broker-frame: signed-frame"),
        "{seen}"
    );
    assert!(network.accept().is_err(), "nothing may reach the network");
}
