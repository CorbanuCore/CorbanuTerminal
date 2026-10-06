//! Credential-safe request URLs and response headers for logs. Provider
//! query parameters (for example `?key=…`), URL userinfo and headers such as
//! `set-cookie` can hold credentials, and request logs reach the logs
//! database and `/feedback` at any `RUST_LOG`.

use std::fmt;

use http::HeaderMap;
use reqwest::Url;

const REDACTED: &str = "REDACTED";

/// `url` with userinfo and every query value replaced, keeping the scheme,
/// host, path and query names. Text that is not a URL is not echoed.
pub fn redact_url(url: &str) -> String {
    match Url::parse(url) {
        Ok(mut parsed) => {
            redact_parsed_url(&mut parsed);
            parsed.into()
        }
        Err(_) => "<unparsable url>".to_string(),
    }
}

/// `error`'s message with its URL redacted as by [`redact_url`].
pub fn redact_reqwest_error(error: &reqwest::Error) -> String {
    let message = error.to_string();
    match error.url() {
        Some(url) => message.replace(url.as_str(), &redact_url(url.as_str())),
        None => message,
    }
}

/// Response headers whose values are logged. Every other value, and any
/// value marked sensitive, is replaced: providers and MCP servers can put
/// credentials in arbitrary headers (`set-cookie`, `location`, `*-jwt-*`).
const LOGGED_HEADER_NAMES: [&str; 16] = [
    "age",
    "cache-control",
    "cf-ray",
    "connection",
    "content-encoding",
    "content-length",
    "content-type",
    "date",
    "etag",
    "request-id",
    "retry-after",
    "server",
    "transfer-encoding",
    "vary",
    "x-models-etag",
    "x-request-id",
];
const LOGGED_HEADER_PREFIXES: [&str; 4] = ["openai-", "x-codex-", "x-oai-", "x-ratelimit-"];

/// `Debug` view of `headers` for logs: names are kept and only the values of
/// known diagnostic headers are shown.
pub fn redact_headers(headers: &HeaderMap) -> RedactedHeaders<'_> {
    RedactedHeaders(headers)
}

/// See [`redact_headers`].
pub struct RedactedHeaders<'a>(&'a HeaderMap);

impl fmt::Debug for RedactedHeaders<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map()
            .entries(self.0.iter().map(|(name, value)| {
                let name = name.as_str();
                let logged = !value.is_sensitive()
                    && (LOGGED_HEADER_NAMES.contains(&name)
                        || LOGGED_HEADER_PREFIXES
                            .iter()
                            .any(|prefix| name.starts_with(prefix)));
                let shown = if logged {
                    value.to_str().unwrap_or("<non-utf8>")
                } else {
                    REDACTED
                };
                (name, shown)
            }))
            .finish()
    }
}

fn redact_parsed_url(url: &mut Url) {
    if !url.username().is_empty() {
        let _ = url.set_username(REDACTED);
    }
    if url.password().is_some() {
        let _ = url.set_password(Some(REDACTED));
    }
    if url.query().is_some() {
        let names = url
            .query_pairs()
            .map(|(name, _)| name.into_owned())
            .collect::<Vec<_>>();
        url.query_pairs_mut()
            .clear()
            .extend_pairs(names.iter().map(|name| (name.as_str(), REDACTED)));
    }
}

#[cfg(test)]
#[path = "log_redaction_tests.rs"]
mod tests;
