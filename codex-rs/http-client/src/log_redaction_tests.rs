use pretty_assertions::assert_eq;

use super::*;

#[test]
fn redact_url_hides_query_values_and_userinfo() {
    assert_eq!(
        [
            redact_url("https://api.example.com/v1/chat/completions"),
            redact_url("https://api.example.com/v1/models?key=fake-query-secret&api-version=2025"),
            redact_url("https://user:fake-password@proxy.example.com/v1?flag"),
            redact_url("not a url fake-secret"),
        ],
        [
            "https://api.example.com/v1/chat/completions".to_string(),
            "https://api.example.com/v1/models?key=REDACTED&api-version=REDACTED".to_string(),
            "https://REDACTED:REDACTED@proxy.example.com/v1?flag=REDACTED".to_string(),
            "<unparsable url>".to_string(),
        ]
    );
}

#[test]
fn redact_headers_shows_only_diagnostic_values_and_keeps_names() {
    let mut headers = HeaderMap::new();
    headers.append("content-type", "application/json".parse().unwrap());
    headers.append("x-request-id", "req-123".parse().unwrap());
    headers.append("x-ratelimit-remaining-tokens", "99".parse().unwrap());
    headers.append("set-cookie", "session=fake-cookie-secret".parse().unwrap());
    headers.append("set-cookie", "__cf_bm=fake-cf-secret".parse().unwrap());
    headers.append("x-api-key", "fake-api-key".parse().unwrap());
    headers.append(
        "location",
        "https://x.test/cb?code=fake-code".parse().unwrap(),
    );
    headers.append("cf-access-jwt-assertion", "fake-jwt".parse().unwrap());
    let mut marked = http::HeaderValue::from_static("fake-marked-secret");
    marked.set_sensitive(true);
    headers.append("x-request-id", marked);

    assert_eq!(
        format!("{:?}", redact_headers(&headers)),
        "{\"content-type\": \"application/json\", \"x-request-id\": \"req-123\", \
         \"x-request-id\": \"REDACTED\", \"x-ratelimit-remaining-tokens\": \"99\", \
         \"set-cookie\": \"REDACTED\", \"set-cookie\": \"REDACTED\", \
         \"x-api-key\": \"REDACTED\", \"location\": \"REDACTED\", \
         \"cf-access-jwt-assertion\": \"REDACTED\"}"
    );
}

#[test]
fn transport_error_debug_redacts_url_and_headers() {
    let mut headers = HeaderMap::new();
    headers.append("set-cookie", "fake-debug-cookie".parse().unwrap());
    let error = crate::TransportError::Http {
        status: http::StatusCode::UNAUTHORIZED,
        url: Some("https://x.test/v1?key=fake-debug-key".to_string()),
        headers: Some(headers),
        body: Some("denied".to_string()),
    };
    assert_eq!(
        format!("{error:?}"),
        "Http { status: 401, url: Some(\"https://x.test/v1?key=REDACTED\"), headers: Some({\"set-cookie\": \"REDACTED\"}), body: Some(\"denied\") }"
    );
}
