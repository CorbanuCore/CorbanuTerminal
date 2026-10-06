//! Credential-safe request URLs for logs. Provider query parameters (for
//! example `?key=…`) and URL userinfo can hold API keys, and request logs
//! reach the logs database and `/feedback` at any `RUST_LOG`.

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
