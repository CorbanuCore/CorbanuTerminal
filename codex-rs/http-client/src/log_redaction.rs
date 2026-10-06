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

/// Header-name fragments whose values are never logged.
const SECRET_HEADER_FRAGMENTS: [&str; 9] = [
    "cookie",
    "auth",
    "token",
    "key",
    "secret",
    "session",
    "signature",
    "credential",
    "password",
];

/// `Debug` view of `headers` for logs: every name is kept, and values of
/// sensitive or credential-like headers (`set-cookie`, `authorization`,
/// `x-api-key`, …) are replaced.
pub fn redact_headers(headers: &HeaderMap) -> RedactedHeaders<'_> {
    RedactedHeaders(headers)
}

pub struct RedactedHeaders<'a>(&'a HeaderMap);

impl fmt::Debug for RedactedHeaders<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map()
            .entries(self.0.iter().map(|(name, value)| {
                let lower = name.as_str();
                let secret = value.is_sensitive()
                    || SECRET_HEADER_FRAGMENTS
                        .iter()
                        .any(|fragment| lower.contains(fragment));
                let shown = if secret {
                    REDACTED
                } else {
                    value.to_str().unwrap_or("<non-utf8>")
                };
                (lower, shown)
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
