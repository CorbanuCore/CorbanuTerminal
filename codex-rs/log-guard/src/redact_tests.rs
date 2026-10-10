use std::borrow::Cow;
use std::io::Write;

use pretty_assertions::assert_eq;

use super::*;

#[test]
fn redacts_credential_headers_in_every_log_shape() {
    let cases = [
        // tungstenite's `Debug` of the upgrade request.
        (
            r#"Request: "GET /v1 HTTP/1.1\r\nhost: api\r\nauthorization: Bearer fake-ws-key-0001\r\nupgrade: websocket\r\n\r\n""#,
            r#"Request: "GET /v1 HTTP/1.1\r\nhost: api\r\nauthorization: REDACTED\r\nupgrade: websocket\r\n\r\n""#,
        ),
        // Raw header lines.
        (
            "Authorization: Basic ZmFrZTpmYWtl\r\nCookie: a=fake-cookie; b=c\r\nX-Api-Key: fake-key\r\nAccept: */*\r\n",
            "Authorization: REDACTED\r\nCookie: REDACTED\r\nX-Api-Key: REDACTED\r\nAccept: */*\r\n",
        ),
        // `Debug` and JSON maps, including escaped quotes.
        (
            r#"headers: {"api-key": "fake-azure-key", "content-type": "application/json"}"#,
            r#"headers: {"api-key": "REDACTED", "content-type": "application/json"}"#,
        ),
        (
            r#"{\"proxy-authorization\": \"Basic ZmFrZQ==\"}"#,
            r#"{\"proxy-authorization\": \"REDACTED\"}"#,
        ),
        // Values inside wrapper types and tuples (aws-smithy, Option, Vec).
        (
            r#"{"authorization": HeaderValue { _private: H0("AWS4-HMAC-SHA256 Credential=AKID/x, Signature=fake") }}"#,
            r#"{"authorization": HeaderValue { _private: H0("REDACTED") }}"#,
        ),
        (
            r#"x-api-key: Some("fake-key-0004")"#,
            r#"x-api-key: Some("REDACTED")"#,
        ),
        (
            "authorization: AWS4-HMAC-SHA256 Credential=AKID/x, SignedHeaders=host, Signature=fake\n",
            "authorization: REDACTED\n",
        ),
        (
            r#"[("api-key", "fake-azure-0005"), ("accept", "*/*")]"#,
            r#"[("api-key", "REDACTED"), ("accept", "*/*")]"#,
        ),
        // A SigV4 canonical request.
        (
            "host:bedrock\nx-amz-security-token:fake-session-0006\n",
            "host:bedrock\nx-amz-security-token:REDACTED\n",
        ),
        // A marker hides nothing after it; unquoted values end at one token.
        (
            "authorization: <redacted> x-api-key: fake-real-0007",
            "authorization: <redacted> x-api-key: REDACTED",
        ),
        (
            "authorization=required model=gpt-5",
            "authorization=REDACTED model=gpt-5",
        ),
        // URL query credentials.
        (
            "Trying to contact wss://host/v1/realtime?model=gpt&api_key=fake-q-0008 now",
            "Trying to contact wss://host/v1/realtime?model=gpt&api_key=REDACTED now",
        ),
        // Bearer tokens, key formats and JWTs anywhere.
        (
            "retrying with Bearer fake-bearer-token-0002 now",
            "retrying with Bearer REDACTED now",
        ),
        (
            "key sk-proj-AAAAAAAAAAAAAAAAAAAAAAAA end",
            "key REDACTED end",
        ),
        (
            "token=eyJhbGciOiJIUzI1.eyJzdWIiOiIxMjM0.c2lnbmF0dXJlMTIz",
            "token=REDACTED",
        ),
    ];
    for (input, expected) in cases {
        assert_eq!(redact_credentials(input), expected);
    }
}

#[test]
fn keeps_redacted_values_and_ordinary_text() {
    for text in [
        r#""set-cookie": "REDACTED""#,
        r#"{"authorization": Sensitive, "x-request-id": "req-1"}"#,
        r#""X-Sentinel": "<redacted>""#,
        "experimental_bearer_token: Some(\"<redacted>\")",
        "uses bearer auth for the provider",
        "authorization server metadata discovered",
        "cookies: 3, max_tokens=100",
        r#"auth.header_name="authorization" auth_mode="ApiKey""#,
        "url=https://api.example.com/v1/models?key=REDACTED&api-version=REDACTED",
        "task-runner-AAAAAAAAAAAAAAAAAAAAAAAAA",
    ] {
        assert!(
            matches!(redact_credentials(text), Cow::Borrowed(_)),
            "{text}"
        );
    }
}

#[test]
fn writer_redacts_each_write() {
    let mut out = Vec::new();
    {
        let mut writer = RedactingWriter::new(&mut out);
        writer
            .write_all(b"a authorization: Bearer fake-writer-key-0003\n")
            .unwrap();
        writer.write_all(b"plain line\n").unwrap();
    }
    assert_eq!(
        String::from_utf8(out).unwrap(),
        "a authorization: REDACTED\nplain line\n"
    );
}
