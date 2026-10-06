use super::*;
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use pretty_assertions::assert_eq;

// Synthetic credential only.
const TOKEN: &str = "ghp_pf28s02SyntheticReflectedToken0123456789";

fn gate() -> ResponseGate {
    ResponseGate::new([("broker:GITHUB_TOKEN", TOKEN)]).expect("gate")
}

fn body(gate: &ResponseGate, chunks: &[&[u8]]) -> String {
    let mut scrubber = gate.body();
    let mut out = Vec::new();
    for chunk in chunks {
        out.extend(scrubber.push(chunk));
    }
    out.extend(scrubber.finish());
    assert_eq!(scrubber.pending(), 0);
    String::from_utf8(out).expect("utf8")
}

#[test]
fn pf_28_s02_reflected_header_values_are_scrubbed() {
    let gate = gate();
    let basic = STANDARD.encode(format!("x-access-token:{TOKEN}"));
    for (input, expected) in [
        (
            format!("Bearer {TOKEN}"),
            "Bearer [REDACTED:broker:GITHUB_TOKEN]".to_string(),
        ),
        (
            format!("Basic {basic}"),
            "Basic [REDACTED:broker:GITHUB_TOKEN]".to_string(),
        ),
        (
            format!("https://example.test/cb?token={TOKEN}&x=1"),
            "https://example.test/cb?token=[REDACTED:broker:GITHUB_TOKEN]&x=1".to_string(),
        ),
    ] {
        let scrubbed = gate.scrub_header(input.as_bytes()).expect("scrubbed");
        assert_eq!(String::from_utf8(scrubbed).expect("utf8"), expected);
    }
    assert_eq!(gate.scrub_header(b"application/json"), None);
}

#[test]
fn pf_28_s02_reflected_body_is_scrubbed_at_every_split() {
    let gate = gate();
    // An echo service, an error page and a server-sent event stream.
    let input = format!(
        "{{\"headers\":{{\"Authorization\":\"Bearer {TOKEN}\"}}}}\n\
         <p>invalid credential {TOKEN}</p>\n\
         data: {{\"echo\":\"{}\"}}\n\n",
        STANDARD.encode(TOKEN)
    );
    let expected = input
        .replace(TOKEN, "[REDACTED:broker:GITHUB_TOKEN]")
        .replace(&STANDARD.encode(TOKEN), "[REDACTED:broker:GITHUB_TOKEN]");
    let bytes = input.as_bytes();
    for cut in 0..=bytes.len() {
        assert_eq!(
            body(&gate, &[&bytes[..cut], &bytes[cut..]]),
            expected,
            "cut {cut}"
        );
    }
    let one_byte: Vec<&[u8]> = bytes.chunks(1).collect();
    assert_eq!(body(&gate, &one_byte), expected);
}

#[test]
fn pf_28_s02_clean_body_streams_with_a_bounded_hold() {
    let gate = gate();
    let mut scrubber = gate.body();
    let chunk = b"{\"items\":[1,2,3]} ordinary response text ";
    let out = scrubber.push(chunk);
    assert_eq!(out.as_slice(), chunk.as_slice());
    // A possible start of the value (and the byte before it) is held, never
    // more than the value.
    let out = scrubber.push(b"tail ghp_pf28");
    assert_eq!(out.as_slice(), b"tail".as_slice());
    assert!(scrubber.pending() < 64);
    assert_eq!(scrubber.finish().as_slice(), b" ghp_pf28".as_slice());
}

#[test]
fn pf_28_s02_only_identity_bodies_are_checked() {
    for (value, checked) in [
        (None, true),
        (Some(b"identity".as_slice()), true),
        (Some(b" Identity ".as_slice()), true),
        (Some(b"gzip".as_slice()), false),
        (Some(b"identity, br".as_slice()), false),
        (Some(b"zstd".as_slice()), false),
    ] {
        assert_eq!(
            ResponseGate::checks_content_encoding(value),
            checked,
            "{value:?}"
        );
    }
}

#[test]
fn pf_28_s02_response_gate_refuses_a_value_it_cannot_hold() {
    assert_eq!(
        ResponseGate::new([("broker:SHORT", "ab")]).err(),
        Some(RegisterError::TooShort)
    );
}
