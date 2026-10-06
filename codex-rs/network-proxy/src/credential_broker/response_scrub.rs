//! PF-28-S02: removes an injected credential from the response returned to
//! the agent (headers, body, trailers). Used by the proxy for credentials it
//! injects itself and by the isolated broker for the ones it holds.
//!
//! The request asks for an identity body; a response that still arrives
//! compressed cannot be checked and is refused without returning its bytes.
//! A body error drops the held-back bytes.

use crate::mitm_hook::MitmHookActions;
use codex_secret_broker::output_gate::RegisterError;
use codex_secret_broker::response_gate::ResponseBodyScrubber;
use codex_secret_broker::response_gate::ResponseGate;
use rama_core::bytes::Bytes;
use rama_core::error::BoxError;
use rama_http::Body;
use rama_http::HeaderMap;
use rama_http::HeaderValue;
use rama_http::Response;
use rama_http::StatusCode;
use rama_http::StreamingBody;
use rama_http::body::Frame;
use rama_http::header::ACCEPT_ENCODING;
use rama_http::header::CONTENT_ENCODING;
use rama_http::header::CONTENT_LENGTH;
use rama_http::header::HeaderName;
use std::pin::Pin;
use std::task::Context;
use std::task::Poll;

/// Header naming why a response was refused.
const ERROR_HEADER: &str = "x-proxy-error";

/// Asks the origin for a body the gate can read.
pub(crate) fn request_identity_body(headers: &mut HeaderMap) {
    headers.insert(ACCEPT_ENCODING, HeaderValue::from_static("identity"));
}

/// Adds the values MITM hooks inject (for `Bearer <token>`, the token too).
/// Values too short to be secrets are skipped; a full gate fails closed.
pub(crate) fn with_hook_values(
    gate: Option<ResponseGate>,
    actions: Option<&MitmHookActions>,
) -> Result<Option<ResponseGate>, RegisterError> {
    let Some(actions) = actions.filter(|actions| !actions.inject_request_headers.is_empty()) else {
        return Ok(gate);
    };
    let gate = match gate {
        Some(gate) => gate,
        None => ResponseGate::new(std::iter::empty())?,
    };
    for header in &actions.inject_request_headers {
        let Ok(value) = header.value.to_str() else {
            continue;
        };
        let label = format!("hook:{}", header.name);
        let token = value.rsplit_once(' ').map(|(_, token)| token);
        for part in std::iter::once(value).chain(token) {
            match gate.add(&label, part) {
                Ok(()) | Err(RegisterError::TooShort) => {}
                Err(err) => return Err(err),
            }
        }
    }
    Ok(Some(gate))
}

/// Scrubs `response` with `gate`, or refuses it when its body is encoded.
pub(crate) fn scrub_response(gate: &ResponseGate, response: Response) -> Response {
    let (mut parts, body) = response.into_parts();
    let encoding = parts
        .headers
        .get(CONTENT_ENCODING)
        .map(HeaderValue::as_bytes);
    if !ResponseGate::checks_content_encoding(encoding) {
        tracing::warn!("PF-28-S02: refused an encoded response to a credentialed request");
        return Response::builder()
            .status(StatusCode::BAD_GATEWAY)
            .header("content-type", "text/plain")
            .header(ERROR_HEADER, "credential-response-encoding")
            .body(Body::from(
                "Corbanu refused this response: it was compressed, so it could not be checked for the credential it was sent with.\n",
            ))
            .unwrap_or_else(|_| Response::new(Body::from("refused\n")));
    }
    scrub_headers(gate, &mut parts.headers);
    // The scrubbed body may differ in length.
    parts.headers.remove(CONTENT_LENGTH);
    let body = ScrubbedBody {
        inner: body,
        scrubber: gate.body(),
        gate: gate.clone(),
        trailers: None,
        done: false,
    };
    Response::from_parts(parts, Body::new(body))
}

/// Drops headers whose name holds the value; rewrites values that do.
fn scrub_headers(gate: &ResponseGate, headers: &mut HeaderMap) {
    let mut changed = false;
    let mut scrubbed = HeaderMap::with_capacity(headers.len());
    for (name, value) in headers.iter() {
        if gate.scrub_header(name.as_str().as_bytes()).is_some() {
            changed = true;
            continue;
        }
        let value = match gate.scrub_header(value.as_bytes()) {
            Some(bytes) => {
                changed = true;
                // Markers are ASCII; an unrepresentable value is dropped.
                match HeaderValue::from_bytes(&bytes) {
                    Ok(value) => value,
                    Err(_) => continue,
                }
            }
            None => value.clone(),
        };
        scrubbed.append(HeaderName::clone(name), value);
    }
    if changed {
        *headers = scrubbed;
    }
}

struct ScrubbedBody {
    inner: Body,
    scrubber: ResponseBodyScrubber,
    gate: ResponseGate,
    /// Trailers waiting behind the body's held-back tail.
    trailers: Option<HeaderMap>,
    done: bool,
}

impl StreamingBody for ScrubbedBody {
    type Data = Bytes;
    type Error = BoxError;

    fn poll_frame(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        let this = &mut *self;
        if let Some(trailers) = this.trailers.take() {
            this.done = true;
            return Poll::Ready(Some(Ok(Frame::trailers(trailers))));
        }
        while !this.done {
            let frame = match Pin::new(&mut this.inner).poll_frame(cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(frame) => frame,
            };
            match frame {
                Some(Ok(frame)) => match frame.into_data() {
                    Ok(data) => {
                        let out = this.scrubber.push(&data);
                        if !out.is_empty() {
                            return Poll::Ready(Some(Ok(Frame::data(Bytes::from(out)))));
                        }
                    }
                    Err(frame) => {
                        let Ok(mut trailers) = frame.into_trailers() else {
                            continue;
                        };
                        scrub_headers(&this.gate, &mut trailers);
                        let tail = this.scrubber.finish();
                        if tail.is_empty() {
                            this.done = true;
                            return Poll::Ready(Some(Ok(Frame::trailers(trailers))));
                        }
                        this.trailers = Some(trailers);
                        return Poll::Ready(Some(Ok(Frame::data(Bytes::from(tail)))));
                    }
                },
                Some(Err(err)) => {
                    // Held-back bytes are dropped, never returned raw.
                    this.done = true;
                    let _ = this.scrubber.finish();
                    return Poll::Ready(Some(Err(BoxError::from(err))));
                }
                None => {
                    this.done = true;
                    let tail = this.scrubber.finish();
                    if !tail.is_empty() {
                        return Poll::Ready(Some(Ok(Frame::data(Bytes::from(tail)))));
                    }
                }
            }
        }
        Poll::Ready(None)
    }
}

#[cfg(test)]
#[path = "response_scrub_tests.rs"]
mod tests;
