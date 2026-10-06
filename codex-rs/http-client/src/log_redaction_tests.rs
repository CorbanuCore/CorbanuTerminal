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
fn redact_headers_hides_credential_values_and_keeps_names() {
    let mut headers = HeaderMap::new();
    headers.append("content-type", "application/json".parse().unwrap());
    headers.append("x-request-id", "req-123".parse().unwrap());
    headers.append("set-cookie", "session=fake-cookie-secret".parse().unwrap());
    headers.append("set-cookie", "__cf_bm=fake-cf-secret".parse().unwrap());
    headers.append("x-api-key", "fake-api-key".parse().unwrap());
    headers.append(
        "www-authenticate",
        "Bearer realm=fake-realm".parse().unwrap(),
    );
    headers.append("x-amz-security-token", "fake-amz-token".parse().unwrap());
    let mut marked = http::HeaderValue::from_static("fake-marked-secret");
    marked.set_sensitive(true);
    headers.append("x-custom", marked);

    assert_eq!(
        format!("{:?}", redact_headers(&headers)),
        "{\"content-type\": \"application/json\", \"x-request-id\": \"req-123\", \
         \"set-cookie\": \"REDACTED\", \"set-cookie\": \"REDACTED\", \
         \"x-api-key\": \"REDACTED\", \"www-authenticate\": \"REDACTED\", \
         \"x-amz-security-token\": \"REDACTED\", \"x-custom\": \"REDACTED\"}"
    );
}
