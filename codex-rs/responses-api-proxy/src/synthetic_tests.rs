use super::*;
use crate::synthetic_policy::PACKETS;
use bytes::Bytes;
use http_body_util::BodyExt;
use http_body_util::Full;
use pretty_assertions::assert_eq;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::task::JoinHandle;
use tokio::time::sleep;
use tokio::time::timeout;

struct Case {
    address: SocketAddr,
    upstream: TcpListener,
    metrics: Arc<Metrics>,
    task: JoinHandle<Result<RunSummary>>,
}

impl Case {
    async fn start(limits: Limits) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let upstream = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let metrics = Arc::new(Metrics::default());
        let task = tokio::spawn(serve(
            listener,
            upstream.local_addr().unwrap(),
            limits,
            metrics.clone(),
        ));
        Self {
            address,
            upstream,
            metrics,
            task,
        }
    }
    async fn finish(self) -> RunSummary {
        let result = timeout(Duration::from_secs(3), self.task)
            .await
            .expect("controller deadline: FAIL")
            .unwrap()
            .unwrap();
        assert_eq!(result.remaining_sockets, 0);
        assert_eq!(result.outcomes.len(), result.accepted);
        assert!(result.peak <= 2);
        assert!(!result.outcomes.contains(&Outcome::Panicked));
        result
    }
}

fn quick() -> Limits {
    Limits {
        head: Duration::from_millis(120),
        upload: Duration::from_millis(240),
        exchange: Duration::from_millis(500),
        run: Duration::from_millis(700),
    }
}

fn request(packet: &str) -> String {
    format!(
        "POST /v1/responses HTTP/1.1\r\nHost: fixture\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{packet}",
        packet.len()
    )
}

async fn closed(stream: &mut TcpStream) -> Vec<u8> {
    let mut bytes = Vec::new();
    let result = timeout(Duration::from_secs(2), stream.read_to_end(&mut bytes))
        .await
        .expect("socket survived deadline");
    if let Err(error) = result {
        // An abortive close reads as WSAECONNABORTED on Windows.
        assert!(
            matches!(
                error.kind(),
                std::io::ErrorKind::ConnectionReset | std::io::ErrorKind::ConnectionAborted
            ),
            "unexpected close error: {error}"
        );
    }
    bytes
}

async fn received(stream: &mut TcpStream) -> Vec<u8> {
    let mut bytes = Vec::new();
    loop {
        let mut byte = [0];
        stream.read_exact(&mut byte).await.unwrap();
        bytes.push(byte[0]);
        if bytes.ends_with(b"\r\n\r\n") {
            break;
        }
    }
    let head = String::from_utf8(bytes).unwrap();
    assert!(head.starts_with("POST /v1/responses HTTP/1.1\r\n"));
    assert!(!head.to_ascii_lowercase().contains("authorization:"));
    let length: usize = head
        .lines()
        .find_map(|line| {
            line.to_ascii_lowercase()
                .strip_prefix("content-length: ")
                .map(str::to_owned)
        })
        .unwrap()
        .trim()
        .parse()
        .unwrap();
    let mut body = vec![0; length];
    stream.read_exact(&mut body).await.unwrap();
    body
}

async fn post(
    address: SocketAddr,
    packet: &'static str,
) -> (
    JoinHandle<hyper::Response<hyper::body::Incoming>>,
    JoinHandle<std::result::Result<(), hyper::Error>>,
) {
    let stream = TcpStream::connect(address).await.unwrap();
    let (mut sender, driver) = hyper::client::conn::http1::handshake(TokioIo::new(stream))
        .await
        .unwrap();
    let driver = tokio::spawn(driver);
    let response = tokio::spawn(async move {
        sender
            .send_request(
                hyper::Request::builder()
                    .method("POST")
                    .uri("/v1/responses")
                    .header("host", "fixture")
                    .header("content-type", "application/json")
                    .header("content-length", packet.len())
                    .header("connection", "close")
                    .body(Full::new(Bytes::from_static(packet.as_bytes())))
                    .unwrap(),
            )
            .await
            .unwrap()
    });
    (response, driver)
}

#[tokio::test]
async fn all_packets_and_buffered_final_frames_forward_exactly() {
    for packet in PACKETS {
        let case = Case::start(quick()).await;
        let (response, driver) = post(case.address, packet).await;
        let (mut upstream, _) = case.upstream.accept().await.unwrap();
        assert_eq!(received(&mut upstream).await, packet.as_bytes());
        upstream
            .write_all(
                b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 2\r\n\r\n{}",
            )
            .await
            .unwrap();
        drop(upstream);
        let response = response.await.unwrap();
        assert_eq!(response.status(), hyper::StatusCode::OK);
        assert_eq!(
            response.into_body().collect().await.unwrap().to_bytes(),
            "{}"
        );
        driver.await.unwrap().unwrap();
        assert!(
            timeout(Duration::from_millis(20), case.upstream.accept())
                .await
                .is_err()
        );
        assert_eq!(case.finish().await.outcomes, vec![Outcome::Complete]);
    }
}

#[tokio::test]
async fn sse_is_incremental_and_exact_before_upstream_completes() {
    let case = Case::start(quick()).await;
    let (response, driver) = post(case.address, PACKETS[0]).await;
    let (mut upstream, _) = case.upstream.accept().await.unwrap();
    received(&mut upstream).await;
    upstream.write_all(b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nTransfer-Encoding: chunked\r\n\r\n9\r\ndata: a\n\n\r\n").await.unwrap();
    let mut body = response.await.unwrap().into_body();
    let first = timeout(Duration::from_millis(100), body.frame())
        .await
        .unwrap()
        .unwrap()
        .unwrap()
        .into_data()
        .unwrap();
    assert_eq!(first, "data: a\n\n");
    upstream
        .write_all(b"9\r\ndata: b\n\n\r\n0\r\n\r\n")
        .await
        .unwrap();
    drop(upstream);
    assert_eq!(body.collect().await.unwrap().to_bytes(), "data: b\n\n");
    driver.await.unwrap().unwrap();
    assert_eq!(case.finish().await.outcomes, vec![Outcome::Complete]);
}

#[tokio::test]
async fn rejected_ingress_never_opens_upstream_or_drains_body() {
    let bad = [
        "POST /v1/responses HTTP/1.1\r\nHost: x\r\nContent-Length: 999999999999\r\n\r\n".to_owned(),
        "POST /v1/responses HTTP/1.1\r\nHost: x\r\nContent-Length: 1\r\nContent-Length: 2\r\n\r\n"
            .to_owned(),
        "POST /v1/responses HTTP/1.1\r\nHost: x\r\nTransfer-Encoding: chunked\r\n\r\n".to_owned(),
        request(PACKETS[0]).replace("Content-Length:", "Expect: 100-continue\r\nContent-Length:"),
        request(PACKETS[0]).replace(
            "Content-Length:",
            "Content-Encoding: gzip\r\nContent-Length:",
        ),
        request(PACKETS[0]).replace(
            "Content-Length:",
            "Transfer-Encoding: chunked\r\nContent-Length:",
        ),
        request(PACKETS[0]).replace("/v1/responses", "/v1/responses?remote=x"),
        request("{\"tools\":[{\"type\":\"web_search\"}]}"),
        format!(
            "POST /v1/responses HTTP/1.1\r\n{}\r\n",
            "X: x\r\n".repeat(65)
        ),
        format!("POST /v1/responses HTTP/1.1\r\nX: {}", "x".repeat(HEAD + 1)),
    ];
    for raw in bad {
        let case = Case::start(quick()).await;
        let mut client = TcpStream::connect(case.address).await.unwrap();
        let _ = client.write_all(raw.as_bytes()).await;
        closed(&mut client).await;
        assert!(
            timeout(Duration::from_millis(20), case.upstream.accept())
                .await
                .is_err()
        );
        let summary = case.finish().await;
        assert_eq!(summary.accepted, 1);
    }
}

#[tokio::test]
async fn stalled_and_trickling_ingress_does_not_renew_deadline() {
    for prefix in [
        "P",
        "POST /v1/responses HTTP/1.1\r\nHost: x\r\nContent-Type: application/json\r\nContent-Length: 1\r\n\r\n",
    ] {
        let case = Case::start(quick()).await;
        let mut client = TcpStream::connect(case.address).await.unwrap();
        client.write_all(prefix.as_bytes()).await.unwrap();
        let started = Instant::now();
        closed(&mut client).await;
        assert!(started.elapsed() < Duration::from_millis(500));
        assert_eq!(case.finish().await.accepted, 1);
    }
    let case = Case::start(quick()).await;
    let mut client = TcpStream::connect(case.address).await.unwrap();
    let prefix = request(PACKETS[0]);
    let head = prefix.split("\r\n\r\n").next().unwrap();
    client
        .write_all(format!("{head}\r\n\r\n").as_bytes())
        .await
        .unwrap();
    for _ in 0..8 {
        let _ = client.write_all(b" ").await;
        sleep(Duration::from_millis(40)).await;
    }
    closed(&mut client).await;
    assert!(matches!(
        case.finish().await.outcomes[0],
        Outcome::Expired(Stage::Upload)
    ));
}

#[tokio::test]
async fn pipelines_cannot_forward_twice_and_attempts_are_not_refunded() {
    let case = Case::start(quick()).await;
    for _ in 0..8 {
        let mut client = TcpStream::connect(case.address).await.unwrap();
        client
            .write_all(b"GET / HTTP/1.1\r\nHost: x\r\n\r\n")
            .await
            .unwrap();
        closed(&mut client).await;
    }
    let summary = case.finish().await;
    assert_eq!((summary.accepted, summary.outcomes.len()), (8, 8));
    let case = Case::start(quick()).await;
    let mut client = TcpStream::connect(case.address).await.unwrap();
    client
        .write_all(request(PACKETS[0]).repeat(2).as_bytes())
        .await
        .unwrap();
    let (mut upstream, _) = case.upstream.accept().await.unwrap();
    received(&mut upstream).await;
    upstream
        .write_all(
            b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 2\r\n\r\n{}",
        )
        .await
        .unwrap();
    drop(upstream);
    closed(&mut client).await;
    assert!(
        timeout(Duration::from_millis(30), case.upstream.accept())
            .await
            .is_err()
    );
    assert_eq!(case.finish().await.accepted, 1);
}

#[tokio::test]
async fn two_admissions_then_run_expiry_joins_both_sockets() {
    let case = Case::start(Limits {
        run: Duration::from_millis(100),
        ..quick()
    })
    .await;
    let mut clients = Vec::new();
    for _ in 0..3 {
        clients.push(TcpStream::connect(case.address).await.unwrap());
    }
    sleep(Duration::from_millis(30)).await;
    assert_eq!(case.metrics.accepted.load(Ordering::SeqCst), 2);
    let summary = case.finish().await;
    assert_eq!((summary.accepted, summary.peak), (2, 2));
    // The connection's clipped deadline equals run_end. Either timer or root
    // abort can win; both must terminate, join, and close the owned sockets.
    assert_eq!(summary.outcomes.len(), 2);
    assert!(
        summary
            .outcomes
            .iter()
            .all(|outcome| matches!(outcome, Outcome::Cancelled | Outcome::Expired(Stage::Head)))
    );
    for client in &mut clients {
        closed(client).await;
    }
}

#[tokio::test]
async fn upstream_stalls_floods_truncation_and_trailers_close_both_sides() {
    let cases = [
        "".to_owned(), "HTTP/1.1 200".to_owned(),
        format!("HTTP/1.1 200 OK\r\nX: {}", "x".repeat(HEAD + 1)),
        format!("HTTP/1.1 200 OK\r\n{}\r\n", "X: x\r\n".repeat(65)),
        "HTTP/1.1 100 Continue\r\n\r\n".repeat(1000),
        "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nTransfer-Encoding: chunked\r\n\r\n1\r\nx\r\n0\r\nX: trailer\r\n\r\n".to_owned(),
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 100\r\n\r\nshort".to_owned(),
        "HTTP/1.1 302 Found\r\nLocation: https://example.invalid\r\nContent-Length: 0\r\n\r\n".to_owned(),
    ];
    for raw in cases {
        let case = Case::start(quick()).await;
        let mut client = TcpStream::connect(case.address).await.unwrap();
        client
            .write_all(request(PACKETS[0]).as_bytes())
            .await
            .unwrap();
        let (mut upstream, _) = case.upstream.accept().await.unwrap();
        received(&mut upstream).await;
        let _ = upstream.write_all(raw.as_bytes()).await;
        if raw.ends_with("short") {
            upstream.shutdown().await.unwrap();
        }
        closed(&mut client).await;
        closed(&mut upstream).await;
        let summary = case.finish().await;
        assert!(!summary.outcomes.contains(&Outcome::Complete), "{raw}");
    }
}

#[tokio::test]
async fn trickling_upstream_has_absolute_exchange_deadline() {
    let case = Case::start(quick()).await;
    let mut client = TcpStream::connect(case.address).await.unwrap();
    client
        .write_all(request(PACKETS[0]).as_bytes())
        .await
        .unwrap();
    let (mut upstream, _) = case.upstream.accept().await.unwrap();
    received(&mut upstream).await;
    upstream.write_all(b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nTransfer-Encoding: chunked\r\n\r\n").await.unwrap();
    for _ in 0..15 {
        let _ = upstream.write_all(b"1\r\nx\r\n").await;
        sleep(Duration::from_millis(40)).await;
    }
    closed(&mut client).await;
    closed(&mut upstream).await;
    assert_eq!(
        case.finish().await.outcomes,
        vec![Outcome::Expired(Stage::Stream)]
    );
}

#[tokio::test]
async fn cap_survives_allowance_increase_and_join_records_panic_and_abort() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let mut writer = TcpStream::connect(listener.local_addr().unwrap())
        .await
        .unwrap();
    let (reader, _) = listener.accept().await.unwrap();
    let metrics = Arc::new(Metrics::default());
    let allowance = Arc::new(AtomicUsize::new(3));
    let mut reader = CappedIo::new(reader, allowance.clone(), metrics.clone());
    writer.write_all(b"abcdefg").await.unwrap();
    let mut bytes = [0; 3];
    reader.read_exact(&mut bytes).await.unwrap();
    assert_eq!(&bytes, b"abc");
    allowance.store(6, Ordering::SeqCst);
    reader.read_exact(&mut bytes).await.unwrap();
    assert_eq!(&bytes, b"def");
    assert!(reader.read_u8().await.is_err());
    let task = tokio::spawn(async move {
        let _reader = reader;
        panic!("controlled fixture panic");
    });
    assert_eq!(joined(task.await), Outcome::Panicked);
    assert_eq!(metrics.sockets.load(Ordering::SeqCst), 0);
    let task = tokio::spawn(std::future::pending::<Outcome>());
    task.abort();
    assert_eq!(joined(task.await), Outcome::Cancelled);
}

#[tokio::test]
async fn nonreading_client_really_blocks_writer_then_scope_expires() {
    let case = Case::start(quick()).await;
    let socket = TcpSocket::new_v4().unwrap();
    socket.set_recv_buffer_size(1024).unwrap();
    let mut client = socket.connect(case.address).await.unwrap();
    client
        .write_all(request(PACKETS[0]).as_bytes())
        .await
        .unwrap();
    let (mut upstream, _) = case.upstream.accept().await.unwrap();
    received(&mut upstream).await;
    let writer = tokio::spawn(async move {
        upstream.write_all(format!("HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {OUTPUT}\r\n\r\n").as_bytes()).await.unwrap();
        let _ = upstream.write_all(&vec![b'x'; OUTPUT]).await;
        closed(&mut upstream).await;
    });
    let metrics = case.metrics.clone();
    let summary = case.finish().await;
    // Zero reads until the proxy has terminated: a sleeping client alone is not proof.
    assert!(metrics.blocked_writes.load(Ordering::SeqCst) > 0);
    assert!(summary.outcomes.contains(&Outcome::Expired(Stage::Stream)));
    writer.await.unwrap();
    drop(client);
}

#[tokio::test]
async fn data_and_raw_chunk_overflow_are_bounded_without_complete_response() {
    for chunk_extension in [false, true] {
        let case = Case::start(quick()).await;
        let mut client = TcpStream::connect(case.address).await.unwrap();
        client
            .write_all(request(PACKETS[0]).as_bytes())
            .await
            .unwrap();
        let (mut upstream, _) = case.upstream.accept().await.unwrap();
        received(&mut upstream).await;
        let writer = tokio::spawn(async move {
            let raw = if chunk_extension {
                format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nTransfer-Encoding: chunked\r\n\r\n1;{}\r\nx\r\n0\r\n\r\n",
                    "x".repeat(OUTPUT + 2 * HEAD)
                )
            } else {
                format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                    OUTPUT + 1,
                    "x".repeat(OUTPUT + 1)
                )
            };
            let _ = upstream.write_all(raw.as_bytes()).await;
            closed(&mut upstream).await;
        });
        let output = closed(&mut client).await;
        assert!(output.iter().filter(|byte| **byte == b'x').count() <= OUTPUT);
        writer.await.unwrap();
        assert!(!case.finish().await.outcomes.contains(&Outcome::Complete));
    }
}

#[tokio::test]
async fn disconnect_and_run_expiry_cancel_an_active_upstream() {
    for disconnect in [false, true] {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let upstream = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let upstream_address = upstream.local_addr().unwrap();
        let mut client = TcpStream::connect(listener.local_addr().unwrap())
            .await
            .unwrap();
        client
            .write_all(request(PACKETS[0]).as_bytes())
            .await
            .unwrap();
        let (stream, _) = listener.accept().await.unwrap();
        let metrics = Arc::new(Metrics::default());
        // Anchor the deadlines to a test-controlled instant far enough out to
        // cover a scheduler stall. No deadline can fire before this synthetic
        // accept instant; the assertion below proves the upstream was active
        // before the window even started. The run-expiry branch keeps the
        // original 90 ms duration.
        let setup_window = Duration::from_secs(1);
        let accepted = Instant::now() + setup_window;
        let (run_end, limits) = if disconnect {
            // Keep every deadline beyond the 2 s close bound so only the
            // client disconnect can cancel the active upstream.
            let far = Duration::from_secs(10);
            (
                accepted + far,
                Limits {
                    head: far,
                    upload: far,
                    exchange: far,
                    run: far,
                },
            )
        } else {
            (
                accepted + Duration::from_millis(90),
                Limits {
                    // Non-binding ceiling: it keeps the Connect/ResponseHead
                    // deadlines from winning before the run deadline.
                    head: setup_window + Duration::from_millis(120),
                    ..quick()
                },
            )
        };
        let task = tokio::spawn(serve_one(
            stream,
            accepted,
            run_end,
            upstream_address,
            limits,
            metrics.clone(),
        ));
        let (mut upstream, _) = timeout(Duration::from_secs(2), upstream.accept())
            .await
            .expect("proxy expired before it connected upstream")
            .unwrap();
        let _ = timeout(Duration::from_secs(2), received(&mut upstream))
            .await
            .expect("proxy connected but never forwarded the request");
        assert!(
            Instant::now() <= accepted,
            "upstream became active after the synthetic accept instant"
        );
        if disconnect {
            drop(client);
        }
        closed(&mut upstream).await;
        let outcome = timeout(Duration::from_secs(2), task)
            .await
            .expect("serve_one did not stop")
            .unwrap();
        assert_eq!(metrics.sockets.load(Ordering::SeqCst), 0);
        if disconnect {
            assert!(
                !matches!(outcome, Outcome::Expired(_)),
                "disconnect path used a deadline: {outcome:?}"
            );
        } else {
            assert_eq!(outcome, Outcome::Expired(Stage::ResponseHead));
        }
    }
}

#[tokio::test]
#[cfg_attr(
    windows,
    ignore = "a nonblocking Windows send accepts the whole 4 MiB body at once, so the write never blocks"
)]
async fn owned_driver_cancels_a_proven_blocked_large_fixture_write() {
    // Supporting transport proof only: production admission still allows four
    // small packets, which cannot reliably fill a TCP send window by themselves.
    let socket = TcpSocket::new_v4().unwrap();
    socket.set_recv_buffer_size(1024).unwrap();
    socket.bind("127.0.0.1:0".parse().unwrap()).unwrap();
    let listener = socket.listen(1).unwrap();
    let sender = TcpSocket::new_v4().unwrap();
    sender.set_send_buffer_size(1024).unwrap();
    let stream = sender
        .connect(listener.local_addr().unwrap())
        .await
        .unwrap();
    let (mut peer, _) = listener.accept().await.unwrap();
    let metrics = Arc::new(Metrics::default());
    let io = CappedIo::new(stream, Arc::new(AtomicUsize::new(HEAD)), metrics.clone());
    let (mut sender, driver) = hyper::client::conn::http1::handshake(TokioIo::new(io))
        .await
        .unwrap();
    let result = timeout(Duration::from_millis(120), async move {
        let request = sender.send_request(hyper::Request::builder().method("POST").uri("/v1/responses")
            .header("host", "fixture").body(Full::new(Bytes::from(vec![b'x'; OUTPUT]))).unwrap());
        tokio::pin!(request, driver);
        tokio::select! { result = &mut request => { result.unwrap(); }, result = &mut driver => { result.unwrap(); } }
    }).await;
    assert!(result.is_err());
    assert!(metrics.blocked_writes.load(Ordering::SeqCst) > 0);
    assert_eq!(metrics.sockets.load(Ordering::SeqCst), 0);
    closed(&mut peer).await;
}

#[tokio::test]
async fn full_owned_accept_queue_stalls_connect_and_scope_closes_it() {
    let socket = TcpSocket::new_v4().unwrap();
    socket.bind("127.0.0.1:0".parse().unwrap()).unwrap();
    let upstream = socket.listen(1).unwrap();
    let address = upstream.local_addr().unwrap();
    let mut occupied = Vec::new();
    let mut full = false;
    for _ in 0..32 {
        match timeout(Duration::from_millis(30), TcpStream::connect(address)).await {
            Ok(Ok(stream)) => occupied.push(stream),
            Ok(Err(error)) => {
                // Linux and Windows drop the SYN once the queue is full, so
                // the connect times out. macOS resets it immediately. Both
                // prove the fixture has saturated the accept queue.
                assert!(
                    matches!(
                        error.kind(),
                        std::io::ErrorKind::ConnectionReset | std::io::ErrorKind::ConnectionAborted
                    ),
                    "unexpected connect error while filling the accept queue: {error}"
                );
                full = true;
                break;
            }
            Err(_) => {
                full = true;
                break;
            }
        }
    }
    assert!(full, "fixture failed to saturate the upstream accept queue");

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let mut client = TcpStream::connect(listener.local_addr().unwrap())
        .await
        .unwrap();
    client
        .write_all(request(PACKETS[0]).as_bytes())
        .await
        .unwrap();
    let (stream, _) = listener.accept().await.unwrap();
    let metrics = Arc::new(Metrics::default());
    // Buffer the request before the connection deadline starts, and use a
    // synthetic accept instant so the incidental Head deadline cannot win the
    // race. The Connect deadline still starts from the real clock.
    let accepted = Instant::now() + Duration::from_secs(5);
    let run_end = accepted + Duration::from_secs(10);
    let outcome = timeout(
        Duration::from_secs(2),
        serve_one(stream, accepted, run_end, address, quick(), metrics.clone()),
    )
    .await
    .expect("controller deadline: FAIL");
    assert_eq!(metrics.sockets.load(Ordering::SeqCst), 0);
    closed(&mut client).await;
    #[cfg(target_os = "macos")]
    // macOS resets a connect into a full accept queue instead of leaving the
    // SYN pending, so the connect fails before its deadline. Either a reset
    // or a pending SYN is correct product behaviour for a saturated queue.
    assert!(
        matches!(
            outcome,
            Outcome::Failed(_) | Outcome::Expired(Stage::Connect)
        ),
        "expected the saturated upstream connect to fail or expire, got {outcome:?}"
    );
    #[cfg(not(target_os = "macos"))]
    assert_eq!(outcome, Outcome::Expired(Stage::Connect));

    // serve's run clock starts before serve_one can reach Connect, so this
    // pass deliberately does not pin the expiry stage. It restores the
    // serve-scope coverage: accept, run-end abort, join, downstream close,
    // and the summary invariants previously checked through case.finish().
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let downstream = listener.local_addr().unwrap();
    let metrics = Arc::new(Metrics::default());
    let mut client = TcpStream::connect(downstream).await.unwrap();
    client
        .write_all(request(PACKETS[0]).as_bytes())
        .await
        .unwrap();
    let case = Case {
        address: downstream,
        upstream,
        metrics: metrics.clone(),
        task: tokio::spawn(serve(
            listener,
            address,
            Limits {
                run: Duration::from_millis(150),
                ..quick()
            },
            metrics,
        )),
    };
    closed(&mut client).await;
    let summary = case.finish().await;
    assert_eq!(summary.accepted, 1);
    assert!(
        summary.outcomes.iter().all(|outcome| matches!(
            outcome,
            Outcome::Expired(_) | Outcome::Failed(_) | Outcome::Cancelled
        )),
        "unexpected scope outcomes: {:?}",
        summary.outcomes
    );
    drop(occupied);
}
