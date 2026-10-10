//! PF-27-S05: a request authorized by the isolated credential broker (it
//! carries the broker's signed frame) is sent only to the broker, whatever
//! client the caller built: over its private Unix socket, or on Windows
//! (PF-27-S09) through a [`ModelBrokerSender`] that talks to the broker's
//! data pipe. It never reaches the network as plain HTTP.

use crate::client::HttpClient;
use crate::error::TransportError;
use crate::transport::ByteStream;
use bytes::Bytes;
use http::HeaderMap;
use http::Method;
use http::StatusCode;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::RwLock;

/// Header carrying the broker's signed frame. Must equal
/// `codex_network_proxy::model_auth::MODEL_BROKER_FRAME_HEADER`.
pub const MODEL_BROKER_FRAME_HEADER: &str = "x-corbanu-broker-frame";

/// One frame-bearing request for a [`ModelBrokerSender`].
#[derive(Debug)]
pub struct ModelBrokerRequest {
    pub method: Method,
    /// The plain-HTTP URL the request was rewritten to (`http://host:port/path`).
    pub url: String,
    pub headers: HeaderMap,
    pub body: Bytes,
}

/// The broker's response; the body streams.
pub struct ModelBrokerResponse {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub bytes: ByteStream,
}

pub type ModelBrokerFuture =
    Pin<Box<dyn Future<Output = Result<ModelBrokerResponse, TransportError>> + Send>>;

/// Sends frame-bearing requests to the credential broker (PF-27-S09: the
/// Windows broker's data pipe, every connection checked to be served by the
/// broker process).
pub trait ModelBrokerSender: Send + Sync {
    fn send(&self, request: ModelBrokerRequest) -> ModelBrokerFuture;
}

/// Where frame-bearing requests go.
#[derive(Clone)]
pub(crate) enum ModelBrokerRoute {
    /// A client for the broker's Unix socket.
    Client(HttpClient),
    Sender(Arc<dyn ModelBrokerSender>),
}

static BROKER_ROUTE: RwLock<Option<ModelBrokerRoute>> = RwLock::new(None);

fn install(route: ModelBrokerRoute) {
    let mut installed = BROKER_ROUTE
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    *installed = Some(route);
}

/// Routes every frame-bearing request to `client` (a client for the broker's
/// socket, see [`HttpClient::unix_socket`]).
pub fn install_model_broker_client(client: HttpClient) {
    install(ModelBrokerRoute::Client(client));
}

/// Routes every frame-bearing request to `sender` (PF-27-S09).
pub fn install_model_broker_sender(sender: Arc<dyn ModelBrokerSender>) {
    install(ModelBrokerRoute::Sender(sender));
}

/// Where a frame-bearing request must go, or `None` when no broker is
/// installed (the request must then fail).
pub(crate) fn model_broker_route() -> Option<ModelBrokerRoute> {
    BROKER_ROUTE
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone()
}

/// Serializes tests that install a route (it is process-wide).
#[cfg(test)]
pub(crate) static ROUTE_TEST_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

/// Removes the installed broker route (tests only).
#[cfg(test)]
pub(crate) fn uninstall_model_broker_route() {
    *BROKER_ROUTE
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
}
