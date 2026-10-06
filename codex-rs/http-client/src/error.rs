//! Errors returned by the shared Codex HTTP transport.

use http::HeaderMap;
use http::StatusCode;
use thiserror::Error;

#[derive(Error)]
pub enum TransportError {
    #[error("http {status}: {body:?}")]
    Http {
        status: StatusCode,
        url: Option<String>,
        headers: Option<HeaderMap>,
        body: Option<String>,
    },
    #[error("retry limit reached")]
    RetryLimit,
    #[error("timeout")]
    Timeout,
    #[error("network error: {0}")]
    Network(String),
    #[error("request build error: {0}")]
    Build(String),
}

/// Redacts the URL and header values, which can hold credentials.
impl std::fmt::Debug for TransportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Http {
                status,
                url,
                headers,
                body,
            } => f
                .debug_struct("Http")
                .field("status", status)
                .field("url", &url.as_deref().map(crate::redact_url))
                .field("headers", &headers.as_ref().map(crate::redact_headers))
                .field("body", body)
                .finish(),
            Self::RetryLimit => f.write_str("RetryLimit"),
            Self::Timeout => f.write_str("Timeout"),
            Self::Network(message) => f.debug_tuple("Network").field(message).finish(),
            Self::Build(message) => f.debug_tuple("Build").field(message).finish(),
        }
    }
}

#[derive(Debug, Error)]
pub enum StreamError {
    #[error("stream failed: {0}")]
    Stream(String),
    #[error("timeout")]
    Timeout,
}
