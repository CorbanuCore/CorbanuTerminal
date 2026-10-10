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
