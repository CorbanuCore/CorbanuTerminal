//! PF-27-S05: a request authorized by the isolated credential broker (it
//! carries the broker's signed frame) is sent only to the broker's private
//! Unix socket, whatever client the caller built. It never reaches the
//! network as plain HTTP.

use crate::client::HttpClient;
use std::sync::RwLock;

/// Header carrying the broker's signed frame. Must equal
/// `codex_network_proxy::model_auth::MODEL_BROKER_FRAME_HEADER`.
pub const MODEL_BROKER_FRAME_HEADER: &str = "x-corbanu-broker-frame";

static BROKER_CLIENT: RwLock<Option<HttpClient>> = RwLock::new(None);

/// Routes every frame-bearing request to `client` (a client for the broker's
/// socket, see [`HttpClient::unix_socket`]).
pub fn install_model_broker_client(client: HttpClient) {
    let mut installed = BROKER_CLIENT
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    *installed = Some(client);
}

/// The client a frame-bearing request must use, or `None` when no broker is
/// installed (the request must then fail).
pub(crate) fn model_broker_client() -> Option<HttpClient> {
    BROKER_CLIENT
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone()
}
