use crate::synthetic::{HEAD, INPUT, OUTPUT, Scope, Stage};
use crate::synthetic_io::CappedIo;
use crate::synthetic_policy;
use anyhow::{Result, ensure};
use bytes::Bytes;
use http_body_util::{BodyExt, Full, Limited};
use hyper::body::{Body, Frame, Incoming};
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::task::{Context, Poll};
use tokio::net::TcpStream;
use tokio::time::Instant;

pub(crate) type Driver =
    hyper::client::conn::http1::Connection<TokioIo<CappedIo<TcpStream>>, Full<Bytes>>;
type BoxError = Box<dyn std::error::Error + Send + Sync>;

pub(crate) struct ResponseBody {
    inner: Pin<Box<Limited<Incoming>>>,
    scope: Scope,
}

impl Body for ResponseBody {
    type Data = Bytes;
    type Error = BoxError;
    fn poll_frame(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Bytes>, BoxError>>> {
        let this = self.get_mut();
        if let Err(error) = this.scope.check() {
            return Poll::Ready(Some(Err(error.into())));
        }
        match this.inner.as_mut().poll_frame(cx) {
            Poll::Ready(Some(Ok(frame))) if frame.is_trailers() => {
                Poll::Ready(Some(Err("upstream trailers forbidden".into())))
            }
            result => result,
        }
    }
}

pub(crate) async fn forward(
    request: Request<Incoming>,
    scope: Scope,
) -> Result<Response<ResponseBody>> {
    scope.advance(Stage::Upload, scope.accepted + scope.limits.upload)?;
    ensure!(
        !scope.used.swap(true, Ordering::SeqCst),
        "one request per connection"
    );
    ensure!(
        request.method() == hyper::Method::POST && *request.uri() == "/v1/responses",
        "forbidden target"
    );
    ensure!(
        request.version() == hyper::Version::HTTP_11,
        "HTTP/1.1 required"
    );
    let headers = request.headers();
    for name in [
        "transfer-encoding",
        "expect",
        "content-encoding",
        "upgrade",
        "trailer",
        "authorization",
        "cookie",
    ] {
        ensure!(!headers.contains_key(name), "forbidden header");
    }
    ensure!(
        headers.get("content-type").and_then(|v| v.to_str().ok()) == Some("application/json"),
        "JSON required"
    );
    let length = headers
        .get("content-length")
        .ok_or_else(|| anyhow::anyhow!("length required"))?
        .to_str()?
        .parse::<usize>()?;
    ensure!(length > 0 && length <= INPUT, "input size");
    scope.ingress.store(INPUT + HEAD, Ordering::SeqCst);
    let body = Limited::new(request.into_body(), INPUT)
        .collect()
        .await
        .map_err(anyhow::Error::msg)?;
    ensure!(body.trailers().is_none(), "trailers forbidden");
    let bytes = body.to_bytes();
    ensure!(bytes.len() == length, "length mismatch");
    synthetic_policy::admit(&bytes)?;
    scope.advance(Stage::Connect, Instant::now() + scope.limits.head)?;
    let stream = TcpStream::connect(scope.upstream).await?;
    scope.check()?;
    let allowance = Arc::new(AtomicUsize::new(HEAD));
    let io = TokioIo::new(CappedIo::new(
        stream,
        allowance.clone(),
        scope.metrics.clone(),
    ));
    let (mut sender, driver) = hyper::client::conn::http1::Builder::new()
        .max_headers(64)
        .max_buf_size(HEAD)
        .handshake(io)
        .await?;
    scope
        .drivers
        .try_send(driver)
        .map_err(|_| anyhow::anyhow!("driver already registered"))?;
    scope.advance(Stage::ResponseHead, Instant::now() + scope.limits.head)?;
    let outgoing = Request::builder()
        .method("POST")
        .uri("/v1/responses")
        .header("host", scope.upstream.to_string())
        .header("content-type", "application/json")
        .header("content-length", bytes.len())
        .header("connection", "close")
        .body(Full::new(bytes))?;
    let response = sender.send_request(outgoing).await?;
    scope.advance(Stage::Stream, scope.end)?;
    ensure!(
        response.status() == hyper::StatusCode::OK,
        "non-success upstream"
    );
    for name in ["content-encoding", "upgrade", "trailer"] {
        ensure!(
            !response.headers().contains_key(name),
            "unsupported upstream header"
        );
    }
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    ensure!(
        matches!(content_type, "text/event-stream" | "application/json"),
        "unsupported response type"
    );
    let content_type = content_type.to_owned();
    allowance.store(OUTPUT + 2 * HEAD, Ordering::SeqCst);
    // Preserve DATA/SSE bytes, not HTTP chunk boundaries. Never forward remote
    // length/cookies/location or claim that a flushed byte was consumed by a peer.
    Ok(Response::builder()
        .header("content-type", content_type)
        .header("connection", "close")
        .body(ResponseBody {
            inner: Box::pin(Limited::new(response.into_body(), OUTPUT)),
            scope,
        })?)
}
