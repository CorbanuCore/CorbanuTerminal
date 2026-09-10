use super::*;
use codex_client::RetryOn;
use http::HeaderMap;
use http::Method;
use pretty_assertions::assert_eq;

#[tokio::test]
async fn corbanu_retry_rotates_only_confirmed_released_attempts() {
    for header in ["x-corbanu-request-id", "x-pfterminal-request-id"] {
        for state in [Some("released"), Some("settled"), Some("reserved"), None] {
            for matching in [true, false] {
                let observed = Arc::new(Mutex::new(Vec::new()));
                let seen = Arc::clone(&observed);
                let result: Result<Response, _> = run_with_request_telemetry(
                    RetryPolicy {
                        max_attempts: 1,
                        base_delay: Duration::ZERO,
                        retry_on: RetryOn {
                            retry_429: true,
                            retry_5xx: true,
                            retry_transport: true,
                        },
                    },
                    None,
                    || {
                        let mut request = Request::new(
                            Method::POST,
                            "https://api.corbanu.example/v1/chat/completions".into(),
                        );
                        request
                            .headers
                            .insert(header, HeaderValue::from_static("original"));
                        request
                    },
                    move |req| {
                        let seen = Arc::clone(&seen);
                        async move {
                            let id = req.headers.get(header).unwrap().clone();
                            seen.lock().unwrap().push(id.clone());
                            let mut headers = HeaderMap::new();
                            if let Some(state) = state {
                                headers.insert(
                                    "x-corbanu-request-state",
                                    HeaderValue::from_static(state),
                                );
                            }
                            headers.insert(
                                "x-corbanu-request-id",
                                if matching {
                                    id
                                } else {
                                    HeaderValue::from_static("different-request")
                                },
                            );
                            Err(TransportError::Http {
                                status: StatusCode::SERVICE_UNAVAILABLE,
                                url: None,
                                headers: Some(headers),
                                body: Some("upstream unavailable".into()),
                            })
                        }
                    },
                )
                .await;
                assert!(matches!(
                    result,
                    Err(TransportError::Http {
                        status: StatusCode::SERVICE_UNAVAILABLE,
                        ..
                    })
                ));
                let seen = observed.lock().unwrap();
                assert_eq!(seen.len(), 2);
                assert_eq!(seen[0] != seen[1], state == Some("released") && matching);
            }
        }
    }
}
