use crate::error::ApiError;
use codex_client::Request;
use codex_client::RequestTelemetry;
use codex_client::Response;
use codex_client::RetryPolicy;
use codex_client::StreamResponse;
use codex_client::TransportError;
use codex_client::run_with_retry;
use http::HeaderValue;
use http::StatusCode;
use std::future::Future;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use tokio::time::Instant;
use tokio_tungstenite::tungstenite::Error;
use tokio_tungstenite::tungstenite::Message;

const CORBANU_REQUEST_ID_HEADERS: [&str; 2] = ["x-corbanu-request-id", "x-pfterminal-request-id"];

/// Generic telemetry.
pub trait SseTelemetry: Send + Sync {
    fn on_sse_poll(
        &self,
        result: &Result<
            Option<
                Result<
                    eventsource_stream::Event,
                    eventsource_stream::EventStreamError<TransportError>,
                >,
            >,
            tokio::time::error::Elapsed,
        >,
        duration: Duration,
    );
}

/// Telemetry for Responses WebSocket transport.
pub trait WebsocketTelemetry: Send + Sync {
    fn on_ws_request(&self, duration: Duration, error: Option<&ApiError>, connection_reused: bool);

    fn on_ws_event(
        &self,
        result: &Result<Option<Result<Message, Error>>, ApiError>,
        duration: Duration,
    );
}

pub(crate) trait WithStatus {
    fn status(&self) -> StatusCode;
}

fn http_status(err: &TransportError) -> Option<StatusCode> {
    match err {
        TransportError::Http { status, .. } => Some(*status),
        _ => None,
    }
}

impl WithStatus for Response {
    fn status(&self) -> StatusCode {
        self.status
    }
}

impl WithStatus for StreamResponse {
    fn status(&self) -> StatusCode {
        self.status
    }
}

pub(crate) async fn run_with_request_telemetry<T, F, Fut>(
    policy: RetryPolicy,
    telemetry: Option<Arc<dyn RequestTelemetry>>,
    mut make_request: impl FnMut() -> Request,
    send: F,
) -> Result<T, TransportError>
where
    T: WithStatus,
    F: Clone + Fn(Request) -> Fut,
    Fut: Future<Output = Result<T, TransportError>>,
{
    // Wraps `run_with_retry` to attach per-attempt request telemetry for both
    // unary and streaming HTTP calls.
    // A released Corbanu reservation cannot be reused. Rotate only after the
    // gateway confirms rejection of this exact attempt; uncertain outcomes keep
    // the identity so server-side duplicate protection remains effective.
    let next_id = Arc::new(Mutex::new(None::<HeaderValue>));
    let request_id = Arc::clone(&next_id);
    let make_request = move || {
        let mut request = make_request();
        if let Some(value) = request_id
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .as_ref()
        {
            for name in CORBANU_REQUEST_ID_HEADERS {
                if request.headers.contains_key(name) {
                    request.headers.insert(name, value.clone());
                }
            }
        }
        request
    };
    run_with_retry(policy, make_request, move |req, attempt| {
        let telemetry = telemetry.clone();
        let send = send.clone();
        let sent_id = CORBANU_REQUEST_ID_HEADERS
            .iter()
            .find_map(|name| req.headers.get(*name))
            .cloned();
        let next_id = Arc::clone(&next_id);
        async move {
            let start = Instant::now();
            let result = send(req).await;
            if let Err(TransportError::Http {
                headers: Some(headers),
                ..
            }) = &result
                && headers
                    .get("x-corbanu-request-state")
                    .and_then(|v| v.to_str().ok())
                    == Some("released")
                && sent_id.is_some()
                && headers.get("x-corbanu-request-id") == sent_id.as_ref()
            {
                *next_id.lock().unwrap_or_else(|error| error.into_inner()) = Some(
                    HeaderValue::from_str(&uuid::Uuid::new_v4().to_string())
                        .expect("UUID is a valid header"),
                );
            }
            if let Some(t) = telemetry.as_ref() {
                let (status, err) = match &result {
                    Ok(resp) => (Some(resp.status()), None),
                    Err(err) => (http_status(err), Some(err)),
                };
                t.on_request(attempt, status, err, start.elapsed());
            }
            result
        }
    })
    .await
}

#[cfg(test)]
#[path = "telemetry_tests.rs"]
mod tests;
