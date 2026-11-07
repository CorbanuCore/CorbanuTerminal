//! Shared HTTP transport for fixed clients and route-aware pools, with optional response limits.
//!
//! Each request's limit applies to observed body bytes, including unsuccessful responses;
//! declared content lengths are only an additional early-rejection check.

use crate::HttpResponse;
use crate::RouteAwareClientPool;
use crate::RouteAwareRequestError;
use crate::client::HttpClient;
use crate::client::HttpClientBackend;
use crate::error::TransportError;
use crate::model_broker_route::ModelBrokerRequest;
use crate::model_broker_route::ModelBrokerResponse;
use crate::model_broker_route::ModelBrokerRoute;
use crate::model_broker_route::ModelBrokerSender;
use crate::request::Request;
use crate::request::RequestBody;
use crate::request::Response;
use crate::request_draft::RequestDraft;
use crate::retry_after::RetryAfter;
use bytes::Bytes;
use bytes::BytesMut;
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
    client: TransportClient,
}

#[derive(Clone, Debug)]
enum TransportClient {
    Fixed(HttpClient),
    RouteAware(Box<RouteAwareClientPool>),
}

impl ReqwestTransport {
    pub fn new(client: reqwest::Client) -> Self {
        Self::from_http_client(HttpClient::new(client))
    }

    pub fn from_http_client(client: HttpClient) -> Self {
        Self {
            client: TransportClient::Fixed(client),
        }
    }

    /// Uses the pool to resolve the route for every request and redirect URL.
    pub fn from_route_aware_client_pool(client: RouteAwareClientPool) -> Self {
        Self {
            client: TransportClient::RouteAware(Box::new(client)),
        }
    }

    async fn send(&self, req: Request) -> Result<Sent, TransportError> {
        let prepared = req.prepare_body_for_send().map_err(TransportError::Build)?;

        let Request {
            method,
            url,
            headers: _,
            body: _,
            compression: _,
            timeout,
            response_body_limit_bytes: _,
        } = req;
        let method = Method::from_bytes(method.as_str().as_bytes()).unwrap_or(Method::GET);

        // PF-27-S05: a broker-authorized request goes to the broker and
        // nowhere else, whichever client or route pool the caller built; with
        // no broker it is not sent.
        // A managed application network policy approves the brokered request's
        // real destination, and can revoke it, as for any other request.
        let permit = match self.managed_network_policy() {
            Some(policy)
                if prepared
                    .headers
                    .contains_key(crate::MODEL_BROKER_FRAME_HEADER)
                    && !self.is_broker_socket() =>
            {
                let target = brokered_target_url(&url).ok_or_else(|| {
                    TransportError::Build(
                        "a brokered request needs a plain-HTTP broker URL".to_string(),
                    )
                })?;
                Some(policy.acquire(&target).map_err(TransportError::Policy)?)
            }
            _ => None,
        };
        let broker = if prepared
            .headers
            .contains_key(crate::MODEL_BROKER_FRAME_HEADER)
            && !self.is_broker_socket()
        {
            match crate::model_broker_route::model_broker_route().ok_or_else(|| {
                if let Some(frame) = crate::model_broker_route::broker_frame(&prepared.headers) {
                    crate::model_broker_route::report(
                        &frame,
                        crate::ModelBrokerOutcome::NotRunning,
                    );
                }
                TransportError::Build("the credential broker is not running".to_string())
            })? {
                ModelBrokerRoute::Client(client) => Some(client),
                // PF-27-S09: the Windows broker's checked data pipe.
                ModelBrokerRoute::Sender(sender) => {
                    return Ok(Sent::Broker {
                        sender,
                        permit,
                        request: ModelBrokerRequest {
                            method,
                            url,
                            headers: prepared.headers,
                            body: prepared.body.unwrap_or_default(),
                        },
                        timeout,
                    });
                }
            }
        } else {
            None
        };

        let mut request =
            RequestDraft::new(method, url).map_err(|error| Self::map_error(error.into()))?;
        request.extend_headers(prepared.headers);
        *request.request.timeout_mut() = timeout;
        *request.request.body_mut() = prepared.body.map(Into::into);

        let response = match (&broker, &self.client) {
            (Some(client), _) => {
                let send = Self::send_with_client(client, request);
                match &permit {
                    Some(permit) => permit
                        .run(send)
                        .await
                        .map_err(TransportError::Policy)??
                        .with_permit(permit.clone()),
                    None => send.await?,
                }
            }
            (None, TransportClient::Fixed(client)) => {
                Self::send_with_client(client, request).await?
            }
            (None, TransportClient::RouteAware(client)) => client
                .send(request)
                .await
                .map_err(Self::map_route_aware_error)?,
        };
        Ok(Sent::Response(response))
    }

    async fn send_with_client(
        client: &HttpClient,
        request: RequestDraft,
    ) -> Result<HttpResponse, TransportError> {
        match &client.backend {
            HttpClientBackend::Routed(pool) => pool
                .send(request)
                .await
                .map_err(Self::map_route_aware_error),
            HttpClientBackend::Direct(client) => {
                let request = request
                    .build(client)
                    .map_err(|error| Self::map_error(error.into()))?;
                client
                    .execute(request)
                    .await
                    .map(HttpResponse::from)
                    .map_err(|error| Self::map_error(error.into()))
            }
        }
    }

    /// The caller's managed application network policy, if its routes have one.
    fn managed_network_policy(&self) -> Option<&crate::NetworkPolicy> {
        match &self.client {
            TransportClient::Fixed(client) => match &client.backend {
                HttpClientBackend::Routed(pool) => pool.managed_network_policy(),
                HttpClientBackend::Direct(_) => None,
            },
            TransportClient::RouteAware(pool) => pool.managed_network_policy(),
        }
    }

    /// Whether this transport connects only to the credential broker's socket.
    fn is_broker_socket(&self) -> bool {
        match &self.client {
            TransportClient::Fixed(client) => client.is_broker_socket(),
            TransportClient::RouteAware(_) => false,
        }
    }

    fn map_error(err: crate::HttpError) -> TransportError {
        let err = err.without_url();
        if let crate::HttpError::Policy(denied) = err {
            TransportError::Policy(denied)
        } else if err.is_connect() {
            TransportError::Connection(err.without_url())
        } else if err.is_timeout() {
            TransportError::Timeout
        } else {
            TransportError::Network(err.redacted_message())
        }
    }

    fn map_route_aware_error(error: RouteAwareRequestError) -> TransportError {
        match error.without_url() {
            RouteAwareRequestError::Policy(denied) => TransportError::Policy(denied),
            RouteAwareRequestError::Request(error) => Self::map_error(error.into()),
            RouteAwareRequestError::Timeout => TransportError::Timeout,
            RouteAwareRequestError::Route(error) => TransportError::Build(error.to_string()),
            RouteAwareRequestError::Build(error) => TransportError::Build(error),
            error @ (RouteAwareRequestError::UnsupportedRedirectScheme(_)
            | RouteAwareRequestError::TooManyRedirects) => {
                TransportError::Network(error.to_string())
            }
        }
    }

    fn trace_request(&self, req: &Request) {
        let request_logging_enabled = match &self.client {
            TransportClient::Fixed(client) => client.request_logging_enabled(),
            TransportClient::RouteAware(client) => client.request_logging_enabled(),
        };
        if request_logging_enabled && enabled!(Level::TRACE) {
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
        // #878: whoever signed a frame-bearing request learns the broker's answer.
        let frame = crate::model_broker_route::broker_frame(&req.headers);
        let result = self.execute_unreported(req).await;
        crate::model_broker_route::report_result(frame.as_ref(), &result);
        result
    }

    async fn stream(&self, req: Request) -> Result<StreamResponse, TransportError> {
        let frame = crate::model_broker_route::broker_frame(&req.headers);
        let result = self.stream_unreported(req).await;
        crate::model_broker_route::report_result(frame.as_ref(), &result);
        result
    }
}

impl ReqwestTransport {
    async fn execute_unreported(&self, req: Request) -> Result<Response, TransportError> {
        self.trace_request(&req);

        let url = req.url.clone();
        let response_body_limit_bytes = req.response_body_limit_bytes;
        let resp = match self.send(req).await? {
            Sent::Response(resp) => resp,
            Sent::Broker {
                sender,
                permit,
                request,
                timeout,
            } => {
                return execute_via_broker(
                    sender.as_ref(),
                    permit,
                    request,
                    timeout,
                    response_body_limit_bytes,
                    &url,
                )
                .await;
            }
        };
        let status = resp.status();
        let headers = resp.headers().clone();
        let retry_after = RetryAfter::from_headers(&headers);
        let bytes = match response_body_limit_bytes {
            Some(max_bytes) => bounded_response_bytes(resp, max_bytes).await,
            None => resp.bytes().await.map_err(Self::map_error),
        };
        if !status.is_success() {
            let body = match bytes {
                Ok(bytes) => String::from_utf8(bytes.to_vec()).ok(),
                Err(
                    error @ (TransportError::ResponseTooLarge { .. } | TransportError::Policy(_)),
                ) => return Err(error),
                // Keep bounded diagnostic-body failures from hiding HTTP auth/retry status.
                Err(_) if response_body_limit_bytes.is_some() => None,
                // TODO(anp): Preserve retry advice on body failures without changing their
                // network/timeout classification to HTTP status handling.
                Err(error) => return Err(error),
            };
            return Err(TransportError::Http {
                status,
                // Shown to users in error messages; query values and
                // userinfo can hold the provider's credentials.
                url: Some(crate::redact_url(&url)),
                headers: Some(headers),
                body,
                retry_after,
            });
        }
        Ok(Response {
            status,
            headers,
            body: bytes?,
        })
    }

    async fn stream_unreported(&self, req: Request) -> Result<StreamResponse, TransportError> {
        self.trace_request(&req);

        let url = req.url.clone();
        let response_body_limit_bytes = req.response_body_limit_bytes;
        let resp = match self.send(req).await? {
            Sent::Response(resp) => resp,
            Sent::Broker {
                sender,
                permit,
                request,
                timeout,
            } => {
                return stream_via_broker(
                    sender.as_ref(),
                    permit,
                    request,
                    timeout,
                    response_body_limit_bytes,
                    &url,
                )
                .await;
            }
        };
        let status = resp.status();
        let headers = resp.headers().clone();
        if !status.is_success() {
            let retry_after = RetryAfter::from_headers(&headers);
            let body = match response_body_limit_bytes {
                Some(max_bytes) => match bounded_response_bytes(resp, max_bytes).await {
                    Ok(bytes) => {
                        // Reuse the unbounded path's charset, BOM and replacement decoding,
                        // but only after the body has passed the byte limit.
                        let mut buffered = http::Response::new(bytes);
                        *buffered.headers_mut() = headers.clone();
                        reqwest::Response::from(buffered).text().await.ok()
                    }
                    Err(
                        error @ (TransportError::ResponseTooLarge { .. }
                        | TransportError::Policy(_)),
                    ) => return Err(error),
                    // A failed diagnostic body must not hide HTTP auth or retry semantics.
                    Err(_) => None,
                },
                None => match resp.text().await {
                    Err(crate::HttpError::Policy(denied)) => return Err(denied.into()),
                    body => body.ok(),
                },
            };
            return Err(TransportError::Http {
                status,
                // Shown to users in error messages; query values and
                // userinfo can hold the provider's credentials.
                url: Some(crate::redact_url(&url)),
                headers: Some(headers),
                body,
                retry_after,
            });
        }
        let bytes = match response_body_limit_bytes {
            Some(max_bytes) => bounded_response_stream(resp, max_bytes)?,
            None => Box::pin(
                resp.bytes_stream()
                    .map(|result| result.map_err(Self::map_error)),
            ),
        };
        Ok(StreamResponse {
            status,
            headers,
            bytes,
        })
    }
}

/// A request ready to send: through reqwest, or to the credential broker's
/// sender (PF-27-S09).
enum Sent {
    Response(HttpResponse),
    Broker {
        sender: Arc<dyn ModelBrokerSender>,
        /// The managed policy's permit for the real destination, if any.
        permit: Option<crate::NetworkPermit>,
        request: ModelBrokerRequest,
        timeout: Option<Duration>,
    },
}

/// The HTTPS destination a broker URL stands for: core rewrites a brokered
/// request to plain HTTP on the same host and port (`core::model_broker_auth`).
fn brokered_target_url(url: &str) -> Option<reqwest::Url> {
    let mut url = reqwest::Url::parse(url).ok()?;
    if url.scheme() != "http"
        || !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
        || matches!(url.host()?, url::Host::Ipv6(_))
    {
        return None;
    }
    let port = url.port_or_known_default()?;
    url.set_scheme("https").ok()?;
    url.set_port(Some(port)).ok()?;
    Some(url)
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
    response_body_limit_bytes: Option<usize>,
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
    let body = match before(deadline, collect(bytes, response_body_limit_bytes)).await {
        Ok(body) => String::from_utf8(body.to_vec()).ok(),
        Err(error @ TransportError::ResponseTooLarge { .. }) => return Err(error),
        Err(_) => None,
    };
    let retry_after = RetryAfter::from_headers(&headers);
    Err(TransportError::Http {
        status,
        url: Some(crate::redact_url(url)),
        headers: Some(headers),
        body,
        retry_after,
    })
}

/// The brokered body, held to the request's response limit like the reqwest path.
async fn collect(bytes: ByteStream, max_bytes: Option<usize>) -> Result<Bytes, TransportError> {
    let mut bytes = limit_stream(bytes, max_bytes);
    let mut body = BytesMut::new();
    while let Some(chunk) = bytes.next().await {
        body.extend_from_slice(&chunk?);
    }
    Ok(body.freeze())
}

fn limit_stream(bytes: ByteStream, max_bytes: Option<usize>) -> ByteStream {
    let Some(max_bytes) = max_bytes else {
        return bytes;
    };
    futures::stream::try_unfold(
        (bytes, max_bytes),
        move |(mut bytes, remaining)| async move {
            match bytes.next().await.transpose()? {
                Some(chunk) if chunk.len() <= remaining => {
                    let remaining = remaining - chunk.len();
                    Ok(Some((chunk, (bytes, remaining))))
                }
                Some(_) => Err(TransportError::ResponseTooLarge { max_bytes }),
                None => Ok(None),
            }
        },
    )
    .boxed()
}

async fn execute_via_broker(
    sender: &dyn ModelBrokerSender,
    permit: Option<crate::NetworkPermit>,
    request: ModelBrokerRequest,
    timeout: Option<Duration>,
    response_body_limit_bytes: Option<usize>,
    url: &str,
) -> Result<Response, TransportError> {
    let deadline = deadline(timeout);
    let exchange = async {
        let response =
            send_via_broker(sender, request, deadline, response_body_limit_bytes, url).await?;
        let body = before(deadline, collect(response.bytes, response_body_limit_bytes)).await?;
        Ok::<_, TransportError>((response.status, response.headers, body))
    };
    let (status, headers, body) = match permit {
        Some(permit) => permit
            .run(exchange)
            .await
            .map_err(TransportError::Policy)??,
        None => exchange.await?,
    };
    Ok(Response {
        status,
        headers,
        body,
    })
}

async fn stream_via_broker(
    sender: &dyn ModelBrokerSender,
    permit: Option<crate::NetworkPermit>,
    request: ModelBrokerRequest,
    timeout: Option<Duration>,
    response_body_limit_bytes: Option<usize>,
    url: &str,
) -> Result<StreamResponse, TransportError> {
    let deadline = deadline(timeout);
    let send = send_via_broker(sender, request, deadline, response_body_limit_bytes, url);
    let response = match &permit {
        Some(permit) => permit.run(send).await.map_err(TransportError::Policy)??,
        None => send.await?,
    };
    let response_bytes = limit_stream(response.bytes, response_body_limit_bytes);
    // The permit governs the whole body: a revocation ends the stream.
    let response_bytes = match permit {
        Some(permit) => {
            futures::stream::unfold(Some((response_bytes, permit)), |state| async move {
                let (mut bytes, permit) = state?;
                match permit.run(bytes.next()).await {
                    Ok(Some(chunk)) => Some((chunk, Some((bytes, permit)))),
                    Ok(None) => None,
                    Err(denied) => Some((Err(TransportError::Policy(denied)), None)),
                }
            })
            .boxed()
        }
        None => response_bytes,
    };
    let bytes = match deadline {
        // The stream ends with a timeout error once the deadline passes.
        Some(deadline) => futures::stream::unfold(Some(response_bytes), move |bytes| async move {
            let mut bytes = bytes?;
            match tokio::time::timeout_at(deadline, bytes.next()).await {
                Ok(Some(chunk)) => Some((chunk, Some(bytes))),
                Ok(None) => None,
                Err(_) => Some((Err(TransportError::Timeout), None)),
            }
        })
        .boxed(),
        None => response_bytes,
    };
    Ok(StreamResponse {
        status: response.status,
        headers: response.headers,
        bytes,
    })
}

fn bounded_response_stream(
    response: HttpResponse,
    max_bytes: usize,
) -> Result<ByteStream, TransportError> {
    if response
        .content_length()
        .is_some_and(|length| length > max_bytes as u64)
    {
        return Err(TransportError::ResponseTooLarge { max_bytes });
    }

    let stream = futures::stream::try_unfold(
        (response, max_bytes),
        move |(mut response, remaining)| async move {
            match response
                .chunk()
                .await
                .map_err(|error| ReqwestTransport::map_error(error.without_url()))?
            {
                Some(chunk) if chunk.len() <= remaining => {
                    let remaining = remaining - chunk.len();
                    Ok(Some((chunk, (response, remaining))))
                }
                Some(_) => Err(TransportError::ResponseTooLarge { max_bytes }),
                None => Ok(None),
            }
        },
    );
    Ok(Box::pin(stream))
}

async fn bounded_response_bytes(
    response: HttpResponse,
    max_bytes: usize,
) -> Result<Bytes, TransportError> {
    let mut stream = bounded_response_stream(response, max_bytes)?;
    let mut body = BytesMut::new();
    while let Some(chunk) = stream.next().await {
        body.extend_from_slice(&chunk?);
    }
    Ok(body.freeze())
}

#[cfg(test)]
#[path = "transport_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "transport_limit_tests.rs"]
mod limit_tests;
