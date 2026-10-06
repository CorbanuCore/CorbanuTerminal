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
use rama_http::header::TRANSFER_ENCODING;
use std::pin::Pin;
use std::task::Context;
use std::task::Poll;

/// Header naming why a response was refused.
const ERROR_HEADER: &str = "x-proxy-error";
/// Shortest part of a hook header value registered on its own (shorter
/// parts are schemes and names such as `Bearer` or `key`, not secrets).
const MIN_HOOK_PART_BYTES: usize = 12;

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
        // A value that is not text cannot be matched reliably: refuse.
        let value = header.value.to_str().map_err(|_| RegisterError::NotText)?;
        let label = format!("hook:{}", header.name);
        let parts = value
            .split(|ch: char| ch.is_whitespace() || matches!(ch, '=' | ':' | ',' | ';'))
            .filter(|part| part.len() >= MIN_HOOK_PART_BYTES && *part != value);
        for part in std::iter::once(value).chain(parts) {
            match gate.add(&label, part) {
                Ok(()) | Err(RegisterError::TooShort) => {}
                Err(err) => return Err(err),
            }
        }
    }
    Ok(Some(gate))
}

/// Scrubs `response` with `gate`. Refuses it when its body is encoded (any
/// `Content-Encoding` occurrence other than identity, or a transfer coding
/// other than chunked) or when it switches protocols, since the scrubber
/// would not see those bytes.
pub(crate) fn scrub_response(gate: &ResponseGate, response: Response) -> Response {
    let (mut parts, body) = response.into_parts();
    let encodings = parts
        .headers
        .get_all(CONTENT_ENCODING)
        .iter()
        .map(HeaderValue::as_bytes);
    // Defence in depth: the HTTP client decodes chunked bodies itself, so a
    // remaining transfer coding means bytes the scrubber cannot read.
    let transfer_coded = parts
        .headers
        .get_all(TRANSFER_ENCODING)
        .iter()
        .any(|value| {
            value.as_bytes().split(|byte| *byte == b',').any(|coding| {
                let coding = coding.trim_ascii();
                !coding.eq_ignore_ascii_case(b"chunked")
                    && !coding.eq_ignore_ascii_case(b"identity")
            })
        });
    let refusal = if parts.status == StatusCode::SWITCHING_PROTOCOLS {
        Some(("credential-response-upgrade", "it switched protocols"))
    } else if transfer_coded || !ResponseGate::checks_content_encoding(encodings) {
        Some(("credential-response-encoding", "it was compressed"))
    } else {
        None
    };
    if let Some((code, why)) = refusal {
        tracing::warn!("PF-28-S02: refused a response to a credentialed request: {code}");
        return Response::builder()
            .status(StatusCode::BAD_GATEWAY)
            .header("content-type", "text/plain")
            .header(ERROR_HEADER, code)
            .body(Body::from(format!(
                "Corbanu refused this response: {why}, so it could not be checked for the credential it was sent with.\n"
            )))
            .unwrap_or_else(|_| Response::new(Body::from("refused\n")));
    }
    scrub_headers(gate, &mut parts.headers);
    // The scrubbed body may differ in length; responses without a body keep
    // theirs.
    if !matches!(
        parts.status,
        StatusCode::NO_CONTENT | StatusCode::NOT_MODIFIED
    ) {
        parts.headers.remove(CONTENT_LENGTH);
    }
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
