use crate::client::HttpClient;
use crate::client::RequestBuilder;
use crate::error::TransportError;
use crate::model_broker_route::ModelBrokerRequest;
use crate::model_broker_route::ModelBrokerResponse;
use crate::model_broker_route::ModelBrokerRoute;
use crate::model_broker_route::ModelBrokerSender;
use crate::request::Request;
use crate::request::RequestBody;
use crate::request::Response;
use bytes::Bytes;
use futures::StreamExt;
use futures::stream::BoxStream;
use http::HeaderMap;
use http::Method;
use http::StatusCode;
use std::sync::Arc;
use std::time::Duration;
use tracing::Level;
use tracing::enabled;
use tracing::trace;

pub type ByteStream = BoxStream<'static, Result<Bytes, TransportError>>;

pub struct StreamResponse {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub bytes: ByteStream,
}

pub trait HttpTransport: Send + Sync {
    fn execute(
        &self,
        req: Request,
    ) -> impl std::future::Future<Output = Result<Response, TransportError>> + Send;
    fn stream(
        &self,
        req: Request,
    ) -> impl std::future::Future<Output = Result<StreamResponse, TransportError>> + Send;
}

#[derive(Clone, Debug)]
pub struct ReqwestTransport {
    client: HttpClient,
}

impl ReqwestTransport {
    pub fn new(client: reqwest::Client) -> Self {
        Self {
            client: HttpClient::new(client),
        }
    }

    pub fn from_http_client(client: HttpClient) -> Self {
        Self { client }
    }

    fn build(&self, req: Request) -> Result<Prepared, TransportError> {
        let prepared = req.prepare_body_for_send().map_err(TransportError::Build)?;

        let Request {
            method,
            url,
            headers: _,
            body: _,
            compression: _,
            timeout,
        } = req;

        // PF-27-S05: a broker-authorized request goes to the broker and
        // nowhere else; with no broker it is not sent.
        let broker;
        let client = if prepared
            .headers
            .contains_key(crate::MODEL_BROKER_FRAME_HEADER)
            && !self.client.is_broker_socket()
        {
            match crate::model_broker_route::model_broker_route().ok_or_else(|| {
                TransportError::Build("the credential broker is not running".to_string())
            })? {
                ModelBrokerRoute::Client(client) => {
                    broker = client;
                    &broker
                }
                // PF-27-S09: the Windows broker's checked data pipe.
                ModelBrokerRoute::Sender(sender) => {
                    return Ok(Prepared::Broker {
                        sender,
                        request: ModelBrokerRequest {
                            method: Method::from_bytes(method.as_str().as_bytes())
                                .unwrap_or(Method::GET),
                            url,
                            headers: prepared.headers,
                            body: prepared.body.unwrap_or_default(),
                        },
                        timeout,
                    });
                }
            }
        } else {
            &self.client
        };
        let mut builder = client.request(
            Method::from_bytes(method.as_str().as_bytes()).unwrap_or(Method::GET),
            &url,
        );

        if let Some(timeout) = timeout {
            builder = builder.timeout(timeout);
        }

        builder = builder.headers(prepared.headers);
        if let Some(body) = prepared.body {
            builder = builder.body(body);
        }
        Ok(Prepared::Reqwest(builder))
    }

    fn map_error(err: reqwest::Error) -> TransportError {
        if err.is_timeout() {
            TransportError::Timeout
        } else {
            TransportError::Network(crate::redact_reqwest_error(&err))
        }
    }

    fn trace_request(&self, req: &Request) {
        if self.client.request_logging_enabled() && enabled!(Level::TRACE) {
            trace!(
                "{} to {}: {}",
                req.method,
                crate::redact_url(&req.url),
                request_body_for_trace(req)
            );
        }
    }
}

fn request_body_for_trace(req: &Request) -> String {
    match req.body.as_ref() {
        Some(RequestBody::Json(body)) => format!("<json body: {} bytes>", body.to_string().len()),
        Some(RequestBody::EncodedJson(body)) => {
            format!("<encoded JSON body: {} bytes>", body.as_bytes().len())
        }
        Some(RequestBody::Raw(body)) => format!("<raw body: {} bytes>", body.len()),
        None => String::new(),
    }
}

impl HttpTransport for ReqwestTransport {
    async fn execute(&self, req: Request) -> Result<Response, TransportError> {
        self.trace_request(&req);

        let url = req.url.clone();
        let builder = match self.build(req)? {
            Prepared::Reqwest(builder) => builder,
            Prepared::Broker {
                sender,
                request,
                timeout,
            } => return execute_via_broker(sender.as_ref(), request, timeout, &url).await,
        };
        let resp = builder.send().await.map_err(Self::map_error)?;
        let status = resp.status();
        let headers = resp.headers().clone();
        let bytes = resp.bytes().await.map_err(Self::map_error)?;
        if !status.is_success() {
            let body = String::from_utf8(bytes.to_vec()).ok();
            return Err(TransportError::Http {
                status,
                // Shown to users in error messages; query values and
                // userinfo can hold the provider's credentials.
                url: Some(crate::redact_url(&url)),
                headers: Some(headers),
                body,
            });
        }
        Ok(Response {
            status,
            headers,
            body: bytes,
        })
    }

    async fn stream(&self, req: Request) -> Result<StreamResponse, TransportError> {
        self.trace_request(&req);

        let url = req.url.clone();
        let builder = match self.build(req)? {
            Prepared::Reqwest(builder) => builder,
            Prepared::Broker {
                sender,
                request,
                timeout,
            } => return stream_via_broker(sender.as_ref(), request, timeout, &url).await,
        };
        let resp = builder.send().await.map_err(Self::map_error)?;
        let status = resp.status();
        let headers = resp.headers().clone();
        if !status.is_success() {
            let body = resp.text().await.ok();
            return Err(TransportError::Http {
                status,
                // Shown to users in error messages; query values and
                // userinfo can hold the provider's credentials.
                url: Some(crate::redact_url(&url)),
                headers: Some(headers),
                body,
            });
        }
        let stream = resp
            .bytes_stream()
            .map(|result| result.map_err(Self::map_error));
        Ok(StreamResponse {
            status,
            headers,
            bytes: Box::pin(stream),
        })
    }
}

/// A request ready to send: through reqwest, or to the credential broker's
/// sender (PF-27-S09).
enum Prepared {
    Reqwest(RequestBuilder),
    Broker {
        sender: Arc<dyn ModelBrokerSender>,
        request: ModelBrokerRequest,
        timeout: Option<Duration>,
    },
}

/// Like reqwest's request timeout: one deadline for the whole exchange,
/// response body included.
fn deadline(timeout: Option<Duration>) -> Option<tokio::time::Instant> {
    timeout.map(|timeout| tokio::time::Instant::now() + timeout)
}

async fn before<T>(
    deadline: Option<tokio::time::Instant>,
    future: impl std::future::Future<Output = Result<T, TransportError>>,
) -> Result<T, TransportError> {
    match deadline {
        Some(deadline) => tokio::time::timeout_at(deadline, future)
            .await
            .map_err(|_| TransportError::Timeout)?,
        None => future.await,
    }
}

/// Sends a brokered request and fails on a non-success status, with the same
/// error shape as the reqwest path.
async fn send_via_broker(
    sender: &dyn ModelBrokerSender,
    request: ModelBrokerRequest,
    deadline: Option<tokio::time::Instant>,
    url: &str,
) -> Result<ModelBrokerResponse, TransportError> {
    let response = before(deadline, sender.send(request)).await?;
    if response.status.is_success() {
        return Ok(response);
    }
    let ModelBrokerResponse {
        status,
        headers,
        bytes,
    } = response;
    let body = before(deadline, collect(bytes)).await.ok();
    Err(TransportError::Http {
        status,
        url: Some(crate::redact_url(url)),
        headers: Some(headers),
        body: body.and_then(|body| String::from_utf8(body.to_vec()).ok()),
    })
}

async fn collect(mut bytes: ByteStream) -> Result<Bytes, TransportError> {
    let mut body = Vec::new();
    while let Some(chunk) = bytes.next().await {
        body.extend_from_slice(&chunk?);
    }
    Ok(Bytes::from(body))
}

async fn execute_via_broker(
    sender: &dyn ModelBrokerSender,
    request: ModelBrokerRequest,
    timeout: Option<Duration>,
    url: &str,
) -> Result<Response, TransportError> {
    let deadline = deadline(timeout);
    let response = send_via_broker(sender, request, deadline, url).await?;
    let body = before(deadline, collect(response.bytes)).await?;
    Ok(Response {
        status: response.status,
        headers: response.headers,
        body,
    })
}

async fn stream_via_broker(
    sender: &dyn ModelBrokerSender,
    request: ModelBrokerRequest,
    timeout: Option<Duration>,
    url: &str,
) -> Result<StreamResponse, TransportError> {
    let deadline = deadline(timeout);
    let response = send_via_broker(sender, request, deadline, url).await?;
    let bytes = match deadline {
        // The stream ends with a timeout error once the deadline passes.
        Some(deadline) => futures::stream::unfold(Some(response.bytes), move |bytes| async move {
            let mut bytes = bytes?;
            match tokio::time::timeout_at(deadline, bytes.next()).await {
                Ok(Some(chunk)) => Some((chunk, Some(bytes))),
                Ok(None) => None,
                Err(_) => Some((Err(TransportError::Timeout), None)),
            }
        })
        .boxed(),
        None => response.bytes,
    };
    Ok(StreamResponse {
        status: response.status,
        headers: response.headers,
        bytes,
    })
}

#[cfg(test)]
#[path = "transport_tests.rs"]
mod tests;
