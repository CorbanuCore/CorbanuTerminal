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
