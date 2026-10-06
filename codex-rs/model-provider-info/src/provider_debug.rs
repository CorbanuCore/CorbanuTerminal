//! `Debug` for [`ModelProviderInfo`] without credentials. Provider
//! definitions are logged (for example "Configuring session"), and the logs
//! database and `/feedback` buffer keep DEBUG and TRACE at any `RUST_LOG`.
//! The bearer token, header and query values, and URL userinfo are redacted;
//! field and header names are kept for diagnosis. `auth.args` are kept: they
//! name a token command, not a token.

use std::collections::HashMap;
use std::fmt;

use super::ModelProviderInfo;

const REDACTED: &str = "<redacted>";

impl fmt::Debug for ModelProviderInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        /// Keeps the map's keys and hides its values.
        struct RedactedValues<'a>(&'a HashMap<String, String>);
        impl fmt::Debug for RedactedValues<'_> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let mut keys = self.0.keys().collect::<Vec<_>>();
                keys.sort();
                f.debug_map()
                    .entries(keys.into_iter().map(|key| (key, REDACTED)))
                    .finish()
            }
        }

        // Destructured so a new field cannot be added without deciding
        // whether it must be redacted.
        let Self {
            name,
            base_url,
            env_key,
            env_key_instructions,
            experimental_bearer_token,
            auth,
            aws,
            wire_api,
            query_params,
            http_headers,
            env_http_headers,
            chat_completions_provider,
            request_max_retries,
            stream_max_retries,
            stream_idle_timeout_ms,
            stream_actionable_timeout_ms,
            stream_long_failure_retry_threshold_ms,
            stream_long_failure_max_retries,
            runtime_policy,
            websocket_connect_timeout_ms,
            requires_openai_auth,
            supports_websockets,
            supports_standalone_web_search,
        } = self;
        f.debug_struct("ModelProviderInfo")
            .field("name", name)
            .field("base_url", &base_url.as_deref().map(redact_url))
            .field("env_key", env_key)
            .field("env_key_instructions", env_key_instructions)
            .field(
                "experimental_bearer_token",
                &experimental_bearer_token.as_ref().map(|_| REDACTED),
            )
            .field("auth", auth)
            .field("aws", aws)
            .field("wire_api", wire_api)
            .field("query_params", &query_params.as_ref().map(RedactedValues))
            .field("http_headers", &http_headers.as_ref().map(RedactedValues))
            .field("env_http_headers", env_http_headers)
            .field("chat_completions_provider", chat_completions_provider)
            .field("request_max_retries", request_max_retries)
            .field("stream_max_retries", stream_max_retries)
            .field("stream_idle_timeout_ms", stream_idle_timeout_ms)
            .field("stream_actionable_timeout_ms", stream_actionable_timeout_ms)
            .field(
                "stream_long_failure_retry_threshold_ms",
                stream_long_failure_retry_threshold_ms,
            )
            .field(
                "stream_long_failure_max_retries",
                stream_long_failure_max_retries,
            )
            .field("runtime_policy", runtime_policy)
            .field("websocket_connect_timeout_ms", websocket_connect_timeout_ms)
            .field("requires_openai_auth", requires_openai_auth)
            .field("supports_websockets", supports_websockets)
            .field(
                "supports_standalone_web_search",
                supports_standalone_web_search,
            )
            .finish()
    }
}

/// `url` with userinfo and the query replaced.
fn redact_url(url: &str) -> String {
    let Some((scheme, rest)) = url.split_once("://") else {
        return url.to_string();
    };
    let (authority, tail) = rest.split_at(rest.find(['/', '?', '#']).unwrap_or(rest.len()));
    let authority = match authority.rsplit_once('@') {
        Some((_, host)) => format!("{REDACTED}@{host}"),
        None => authority.to_string(),
    };
    let tail = match tail.split_once('?') {
        Some((path, _)) => format!("{path}?{REDACTED}"),
        None => tail.to_string(),
    };
    format!("{scheme}://{authority}{tail}")
}
