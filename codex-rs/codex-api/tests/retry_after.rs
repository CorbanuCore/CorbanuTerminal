#![allow(clippy::expect_used)]
//! #510: the transport retry of an HTTP 429 (Z.AI's `retry_429` policy)
//! honours `Retry-After`. A short one is waited out; a long one ends the
//! retries at once, so the provider-request throttle owns that wait.
use std::time::Duration;
use std::time::Instant;

use codex_api::RetryConfig;
use codex_client::HttpTransport;
use codex_client::Request;
use codex_client::ReqwestTransport;
use codex_client::TransportError;
use codex_client::run_with_retry;
use codex_http_client::HttpClientBuilder;
use http::HeaderMap;
use http::HeaderValue;
use http::Method;
use http::StatusCode;
use wiremock::Mock;
use wiremock::MockServer;
use wiremock::ResponseTemplate;
use wiremock::matchers::method;

/// Z.AI's transport retry policy (`ModelProviderInfo::to_api_provider`).
fn zai_retry() -> RetryConfig {
    RetryConfig {
        max_attempts: 4,
        base_delay: Duration::from_millis(10),
        retry_429: true,
        retry_5xx: true,
        retry_transport: true,
    }
}

/// Serves `first` once, then 200 for every later request.
async fn server_answering(first: ResponseTemplate) -> MockServer {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(first)
        .up_to_n_times(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    server
}

/// Sends one request under the Z.AI policy: (result status, requests the
/// server saw, time taken).
async fn send(server: &MockServer) -> (Result<u16, TransportError>, usize, Duration) {
    let transport = ReqwestTransport::from_http_client(
        HttpClientBuilder::new()
            .build_direct()
            .expect("test HTTP client should build"),
    );
    let url = format!("{}/chat/completions", server.uri());
    let started = Instant::now();
    let result = run_with_retry(
        zai_retry().to_policy(),
        || Request::new(Method::POST, url.clone()),
        |request, _attempt| transport.execute(request),
    )
    .await
    .map(|response| response.status.as_u16());
    let seen = server.received_requests().await.unwrap_or_default().len();
    (result, seen, started.elapsed())
}

fn rate_limited() -> ResponseTemplate {
    ResponseTemplate::new(429).set_body_string(r#"{"error":{"code":"1302"}}"#)
}

#[tokio::test]
async fn a_long_retry_after_ends_transport_retries_at_once() {
    let server = server_answering(rate_limited().insert_header("retry-after", "120")).await;
    let (result, seen, elapsed) = send(&server).await;
    let Err(TransportError::Http {
        status, headers, ..
    }) = result
    else {
        panic!("expected the 429 back, got {result:?}");
    };
    assert_eq!(status.as_u16(), 429);
    // The caller's throttle reads the provider's wait from the 429 it gets.
    assert_eq!(
        headers
            .as_ref()
            .and_then(|headers| headers.get("retry-after"))
            .and_then(|value| value.to_str().ok()),
        Some("120")
    );
    assert_eq!(seen, 1, "a 429 asking for 120 s was retried");
    assert!(elapsed < Duration::from_secs(10), "{elapsed:?}");
}

#[tokio::test]
async fn an_http_date_retry_after_ends_transport_retries_at_once() {
    let server = server_answering(
        rate_limited().insert_header("retry-after", "Wed, 21 Oct 2099 07:28:00 GMT"),
    )
    .await;
    let (result, seen, _) = send(&server).await;
    assert!(
        matches!(result, Err(TransportError::Http { .. })),
        "{result:?}"
    );
    assert_eq!(seen, 1);
}

#[tokio::test]
async fn a_short_retry_after_is_waited_out_before_the_retry() {
    let server = server_answering(rate_limited().insert_header("retry-after", "1")).await;
    let (result, seen, elapsed) = send(&server).await;
    assert_eq!(result.ok(), Some(200));
    assert_eq!(seen, 2);
    assert!(
        elapsed >= Duration::from_secs(1),
        "retried after {elapsed:?}"
    );
}

#[tokio::test]
async fn retry_after_ms_is_honoured_too() {
    let server = server_answering(
        rate_limited()
            .insert_header("retry-after-ms", "60000")
            .insert_header("retry-after", "1"),
    )
    .await;
    let (result, seen, _) = send(&server).await;
    assert!(
        matches!(result, Err(TransportError::Http { .. })),
        "{result:?}"
    );
    assert_eq!(seen, 1);
}

/// A transient overload without `Retry-After` keeps the bounded backoff retry
/// it was enabled for (Z.AI code 1305).
#[tokio::test]
async fn a_429_without_retry_after_keeps_the_backoff_retry() {
    let server = server_answering(rate_limited()).await;
    let (result, seen, _) = send(&server).await;
    assert_eq!(result.ok(), Some(200));
    assert_eq!(seen, 2);
}

/// How one failed attempt with `status` and `headers` is retried under Z.AI's
/// policy, on a paused clock: (attempts made, virtual time before the retry).
async fn retry_of(status: u16, headers: &[(&'static str, HeaderValue)]) -> (u64, Duration) {
    let mut map = HeaderMap::new();
    for (name, value) in headers {
        map.insert(*name, value.clone());
    }
    let attempts = std::sync::atomic::AtomicU64::new(0);
    let started = tokio::time::Instant::now();
    let _ = run_with_retry(
        zai_retry().to_policy(),
        || Request::new(Method::POST, "http://unused.invalid".to_string()),
        |_request, attempt| {
            attempts.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            let map = map.clone();
            async move {
                if attempt == 0 {
                    Err(TransportError::Http {
                        status: StatusCode::from_u16(status).expect("status"),
                        url: None,
                        headers: Some(map),
                        body: None,
                    })
                } else {
                    Ok(())
                }
            }
        },
    )
    .await;
    (attempts.into_inner(), started.elapsed())
}

fn text(value: &str) -> HeaderValue {
    HeaderValue::from_str(value).expect("header value")
}

fn unix_ms_in(delta: Duration) -> i64 {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock");
    i64::try_from((now + delta).as_millis()).expect("ms")
}

/// The headers are read the way core reads them for the throttle's cooldown.
#[tokio::test(start_paused = true)]
async fn the_requested_wait_is_read_like_the_throttle_reads_it() {
    let quick = Duration::from_millis(500);
    // (case, status, headers, retried, minimum wait before the retry)
    #[allow(clippy::type_complexity)]
    let cases: Vec<(&str, u16, Vec<(&'static str, HeaderValue)>, bool, Duration)> = vec![
        ("no header: backoff", 429, vec![], true, Duration::ZERO),
        (
            "2 s",
            429,
            vec![("retry-after", text("2"))],
            true,
            Duration::from_secs(2),
        ),
        (
            "5 s",
            429,
            vec![("retry-after", text("5"))],
            true,
            Duration::from_secs(5),
        ),
        (
            "6 s",
            429,
            vec![("retry-after", text("6"))],
            false,
            Duration::ZERO,
        ),
        (
            "0",
            429,
            vec![("retry-after", text("0"))],
            true,
            Duration::ZERO,
        ),
        (
            "-1",
            429,
            vec![("retry-after", text("-1"))],
            true,
            Duration::ZERO,
        ),
        (
            "fraction",
            429,
            vec![("retry-after", text("10.5"))],
            false,
            Duration::ZERO,
        ),
        (
            "huge",
            429,
            vec![("retry-after", text("99999999999999999999"))],
            false,
            Duration::ZERO,
        ),
        (
            "non-ASCII is no header",
            429,
            vec![(
                "retry-after",
                HeaderValue::from_bytes(b"\xff1").expect("bytes"),
            )],
            true,
            Duration::ZERO,
        ),
        (
            "ms short",
            429,
            vec![("retry-after-ms", text("300"))],
            true,
            Duration::from_millis(300),
        ),
        (
            "ms long",
            429,
            vec![("retry-after-ms", text("60000"))],
            false,
            Duration::ZERO,
        ),
        (
            "ms 0 falls through to Retry-After",
            429,
            vec![("retry-after-ms", text("0")), ("retry-after", text("120"))],
            false,
            Duration::ZERO,
        ),
        (
            "unreadable ms falls through",
            429,
            vec![("retry-after-ms", text("soon")), ("retry-after", text("1"))],
            true,
            Duration::from_secs(1),
        ),
        (
            "reset ms short",
            429,
            vec![(
                "x-ratelimit-reset-ms",
                text(&unix_ms_in(Duration::from_secs(3)).to_string()),
            )],
            true,
            Duration::from_secs(1),
        ),
        (
            "reset s long",
            429,
            vec![(
                "x-ratelimit-reset",
                text(&(unix_ms_in(Duration::from_secs(600)) / 1000).to_string()),
            )],
            false,
            Duration::ZERO,
        ),
        (
            "Retry-After 0 falls through to a long reset",
            429,
            vec![
                ("retry-after", text("0")),
                (
                    "x-ratelimit-reset-ms",
                    text(&unix_ms_in(Duration::from_secs(60)).to_string()),
                ),
            ],
            false,
            Duration::ZERO,
        ),
        (
            "ms short wins over a date",
            429,
            vec![
                ("retry-after-ms", text("300")),
                ("retry-after", text("Wed, 21 Oct 2099 07:28:00 GMT")),
            ],
            true,
            Duration::from_millis(300),
        ),
        (
            "5xx unchanged",
            503,
            vec![("retry-after", text("120"))],
            true,
            Duration::ZERO,
        ),
    ];
    for (case, status, headers, retried, min_wait) in cases {
        let (attempts, waited) = retry_of(status, &headers).await;
        assert_eq!(attempts, if retried { 2 } else { 1 }, "{case}");
        if retried {
            assert!(waited >= min_wait, "{case}: waited {waited:?}");
            if min_wait.is_zero() {
                assert!(waited < quick, "{case}: waited {waited:?}");
            }
        }
    }
}
