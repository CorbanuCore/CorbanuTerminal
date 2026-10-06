use super::*;
use crate::mitm_hook::ResolvedInjectedHeader;
use crate::mitm_hook::SecretSource;
use pretty_assertions::assert_eq;
use std::collections::VecDeque;

// Synthetic credentials only.
const TOKEN: &str = "ghp_pf28s02ProxyReflectionCanary000000000";
const MARKER: &str = "[REDACTED:broker:GH_TOKEN]";

/// A body that yields the given frames, then ends.
struct Frames(VecDeque<Frame<Bytes>>);

impl StreamingBody for Frames {
    type Data = Bytes;
    type Error = BoxError;

    fn poll_frame(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        Poll::Ready(self.0.pop_front().map(Ok))
    }
}

fn gate() -> ResponseGate {
    ResponseGate::new([("broker:GH_TOKEN", TOKEN)]).expect("gate")
}

fn response(headers: &[(&str, &str)], frames: Vec<Frame<Bytes>>) -> Response {
    let mut builder = Response::builder();
    for (name, value) in headers {
        builder = builder.header(*name, *value);
    }
    builder
        .body(Body::new(Frames(frames.into())))
        .expect("response")
}

/// Every frame of the body: data concatenated, trailers in order.
async fn frames(response: Response) -> (String, Vec<HeaderMap>) {
    let mut body = response.into_body();
    let mut data = Vec::new();
    let mut trailers = Vec::new();
    while let Some(frame) = std::future::poll_fn(|cx| Pin::new(&mut body).poll_frame(cx)).await {
        let frame = frame.expect("frame");
        match frame.into_data() {
            Ok(bytes) => {
                assert!(trailers.is_empty(), "data after trailers");
                data.extend_from_slice(&bytes);
            }
            Err(frame) => trailers.push(frame.into_trailers().expect("trailers")),
        }
    }
    (String::from_utf8(data).expect("utf8"), trailers)
}

#[tokio::test]
async fn pf_28_s02_reflected_headers_body_and_trailers_are_scrubbed() {
    let body = format!("{{\"echo\":\"Bearer {TOKEN}\"}}\ndata: {TOKEN}\n\n");
    let (first, second) = body.split_at(20);
    let mut trailers = HeaderMap::new();
    trailers.insert("x-debug", HeaderValue::from_str(TOKEN).expect("value"));
    let upstream = response(
        &[
            ("x-echo-authorization", &format!("Bearer {TOKEN}")),
            ("content-length", &body.len().to_string()),
            ("content-type", "application/json"),
        ],
        vec![
            Frame::data(Bytes::from(first.to_string())),
            Frame::data(Bytes::from(second.to_string())),
            Frame::trailers(trailers),
        ],
    );

    let scrubbed = scrub_response(&gate(), upstream);

    let headers = scrubbed.headers().clone();
    assert_eq!(
        headers
            .get("x-echo-authorization")
            .map(HeaderValue::as_bytes),
        Some(format!("Bearer {MARKER}").as_bytes())
    );
    assert_eq!(headers.get(CONTENT_LENGTH), None);
    assert_eq!(
        headers.get("content-type").map(HeaderValue::as_bytes),
        Some(b"application/json".as_slice())
    );
    let (data, trailers) = frames(scrubbed).await;
    assert_eq!(data, body.replace(TOKEN, MARKER));
    assert_eq!(trailers.len(), 1);
    assert_eq!(
        trailers[0].get("x-debug").map(HeaderValue::as_bytes),
        Some(MARKER.as_bytes())
    );
}

#[tokio::test]
async fn pf_28_s02_compressed_response_is_refused_without_its_bytes() {
    let upstream = response(
        &[("content-encoding", "gzip")],
        vec![Frame::data(Bytes::from(format!("raw {TOKEN}")))],
    );

    let refused = scrub_response(&gate(), upstream);

    assert_eq!(refused.status(), StatusCode::BAD_GATEWAY);
    assert_eq!(
        refused
            .headers()
            .get(ERROR_HEADER)
            .map(HeaderValue::as_bytes),
        Some(b"credential-response-encoding".as_slice())
    );
    let (data, _) = frames(refused).await;
    assert!(!data.contains(TOKEN));
    assert!(data.starts_with("Corbanu refused this response"));
}

#[test]
fn pf_28_s02_hook_injected_values_join_the_gate() {
    let actions = MitmHookActions {
        strip_request_headers: Vec::new(),
        inject_request_headers: vec![ResolvedInjectedHeader {
            name: HeaderName::from_static("x-api-key"),
            value: HeaderValue::from_static("Token pf28s02-hook-secret-0000"),
            source: SecretSource::EnvVar("PF28_HOOK".to_string()),
        }],
    };

    let gate = with_hook_values(/*gate*/ None, Some(&actions))
        .expect("gate")
        .expect("hook values gated");

    assert_eq!(
        gate.scrub_header(b"echo pf28s02-hook-secret-0000"),
        Some(b"echo [REDACTED:hook:x-api-key]".to_vec())
    );
    assert!(
        with_hook_values(/*gate*/ None, /*actions*/ None)
            .expect("no hooks")
            .is_none()
    );
}
