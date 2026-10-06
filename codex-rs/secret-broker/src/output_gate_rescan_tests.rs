use crate::output_gate::OutputGate;
use crate::output_gate::OutputSink;
use crate::output_gate::SecretClass;
use crate::output_gate::StreamScrubber;
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use pretty_assertions::assert_eq;

// Synthetic canaries only.
const CANARY: &str = "pf28s02-canary-Lk3vQ9xR2mT7wY5nB8cJ4hF6dG1sA0eZ";
const SEED: &str =
    "abandon ability able about above absent absorb abstract absurd abuse access accident";

fn gate(label: &str, class: SecretClass, value: &str) -> OutputGate {
    let gate = OutputGate::new();
    gate.register(label, class, value).expect("register");
    gate
}

fn scrub(gate: &OutputGate, input: &str) -> Option<String> {
    gate.scrub(OutputSink::ToolResult, input)
        .map(|(text, _)| text)
}

fn wrap(text: &str, width: usize, separator: &str) -> String {
    text.as_bytes()
        .chunks(width)
        .map(|line| String::from_utf8_lossy(line).into_owned())
        .collect::<Vec<_>>()
        .join(separator)
}

/// No 12-character window of `encoded` survives once line breaks are removed.
fn assert_no_encoded_fragment(output: &str, encoded: &str) {
    let joined: String = output
        .replace("\\r\\n", "")
        .replace("\\n", "")
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect();
    let encoded = encoded.as_bytes();
    for window in encoded.windows(12) {
        let window = std::str::from_utf8(window).expect("ascii");
        assert!(!joined.contains(window), "fragment {window} leaked");
    }
}

fn stream(gate: &OutputGate, chunks: &[&[u8]]) -> Vec<u8> {
    let mut scrubber = StreamScrubber::new();
    let mut out = Vec::new();
    for chunk in chunks {
        out.extend(scrubber.push(gate, OutputSink::ToolResult, chunk, /*utf8*/ true));
    }
    out.extend(scrubber.finish(gate, OutputSink::ToolResult));
    out
}

#[test]
fn pf_28_s02_wrapped_base64_and_hex_blocks_are_found() {
    let gate = gate("env:PF28S02", SecretClass::Operational, CANARY);
    // A credentials file run through `base64` (76 columns) and `openssl
    // base64` (64), and `xxd -p` (60 hex columns).
    let file = format!("user=ci\ntoken={CANARY}\nregion=us-east-1\n");
    let base64 = STANDARD.encode(&file);
    let hex: String = file.bytes().map(|byte| format!("{byte:02x}")).collect();
    for (block, encoded) in [
        (wrap(&base64, 76, "\n"), &base64),
        (wrap(&base64, 64, "\r\n"), &base64),
        (wrap(&base64, 76, "\n    "), &base64),
        (wrap(&base64, 76, "\\n"), &base64),
        (wrap(&hex, 60, "\n"), &hex),
    ] {
        let input = format!("$ cat creds | encode\n{block}\n$ ");
        let output = scrub(&gate, &input).expect("wrapped value found");
        assert!(output.contains("[REDACTED:env:PF28S02]"), "{output}");
        assert!(output.starts_with("$ cat creds | encode\n"), "{output}");
        assert_no_encoded_fragment(&output, &encoded[24..encoded.len() - 24]);
    }
}

#[test]
fn pf_28_s02_wrapped_block_split_across_chunks_never_leaks() {
    let gate = gate("env:PF28S02", SecretClass::Operational, CANARY);
    for (width, prefix) in [(40, ""), (76, "header line one\nheader line two\n")] {
        let encoded = STANDARD.encode(format!("{prefix}token={CANARY}"));
        let input = format!("out:\n{}\ndone\n", wrap(&encoded, width, "\n"));
        let bytes = input.as_bytes();
        for cut in 0..=bytes.len() {
            let out = stream(&gate, &[&bytes[..cut], &bytes[cut..]]);
            let out = String::from_utf8(out).expect("utf8");
            // A cut inside the block can move where the redaction starts
            // (the prefix is not secret), never what it removes.
            assert!(out.contains("[REDACTED:env:PF28S02]"), "cut {cut}: {out}");
            assert!(out.ends_with("\ndone\n"), "cut {cut}: {out}");
            let core = encoded.len() - STANDARD.encode(CANARY).len() + 4;
            assert_no_encoded_fragment(&out, &encoded[core..encoded.len() - 4]);
        }
    }
}

#[test]
fn pf_28_s02_nested_encodings_are_decoded_and_rescanned() {
    let gate = gate("env:PF28S02", SecretClass::Operational, CANARY);
    let hex: String = CANARY.bytes().map(|byte| format!("{byte:02x}")).collect();
    let json = serde_json::json!({ "auth": { "token": CANARY } }).to_string();
    for nested in [
        STANDARD.encode(STANDARD.encode(CANARY)),
        STANDARD.encode(&hex),
        STANDARD.encode(format!("Basic {}", STANDARD.encode(format!("x:{CANARY}")))),
        STANDARD
            .encode(&json)
            .bytes()
            .map(|byte| format!("{byte:02x}"))
            .collect(),
    ] {
        let input = format!("blob {nested} end");
        assert_eq!(
            scrub(&gate, &input),
            Some("blob [REDACTED:env:PF28S02] end".to_string()),
            "{nested}"
        );
    }
}

#[test]
fn pf_28_s02_rescans_leave_ordinary_text_alone() {
    let gate = gate("env:PF28S02", SecretClass::Operational, CANARY);
    gate.register("vault:wallet", SecretClass::SeedPhrase, SEED)
        .expect("seed");
    let other = STANDARD.encode("an unrelated document that holds no managed value at all");
    let shas = "5b107095a68f0c1d2e3f405162738495a6b7c8d9\n\
                24a57e38c4d3e2f1a0b9c8d7e6f5a4b3c2d1e0f9\n\
                39c1f06213a4b5c6d7e8f90a1b2c3d4e5f6a7b8c\n";
    let json = serde_json::json!({ "image": STANDARD.encode([7u8; 3000]), "ok": true }).to_string();
    for input in [
        shas.to_string(),
        json,
        "We are able to see above it, and about the ability to abandon it.".to_string(),
        "CODEX_SANDBOX_NETWORK_DISABLED_ENV_VAR is read by tests".to_string(),
        format!("data:{}\n{}\n", wrap(&other, 20, "\n"), other),
        "0123456789abcdef0123456789abcdef0123456789abcdef".to_string(),
        "use std::collections::HashMap;\nfn main() {}\n".to_string(),
    ] {
        assert_eq!(scrub(&gate, &input), None, "{input}");
    }
}

#[test]
fn pf_28_s02_seed_phrases_with_any_separator_are_withheld() {
    let gate = gate("vault:wallet", SecretClass::SeedPhrase, SEED);
    let words: Vec<&str> = SEED.split(' ').collect();
    let numbered: String = words
        .iter()
        .enumerate()
        .map(|(index, word)| format!("{}. {word}\n", index + 1))
        .collect();
    for input in [
        numbered,
        words.join(", "),
        serde_json::to_string(&words).expect("json"),
        words.join("\\n"),
        "Words: Absent | Absorb | Abstract".to_string(),
        format!("blob {}", STANDARD.encode(words.join(","))),
    ] {
        let (output, provenance) = gate
            .scrub(OutputSink::ToolResult, &input)
            .expect("seed phrase found");
        assert!(provenance.withheld, "{input}");
        assert!(output.starts_with("[WITHHELD:"), "{output}");
    }
    for input in [
        "abandon the ability to be able",
        "able ability abandon",
        "abandon 1234567890123456789012 ability able",
    ] {
        assert_eq!(scrub(&gate, input), None, "{input}");
    }
}

#[test]
fn pf_28_s02_streamed_numbered_seed_phrase_never_shows_three_words() {
    let gate = gate("vault:wallet", SecretClass::SeedPhrase, SEED);
    let input = "your backup:\n1. abandon\n2. ability\n3. able\n4. about\nkeep it safe\n";
    let bytes = input.as_bytes();
    let one_byte: Vec<&[u8]> = bytes.chunks(1).collect();
    for chunks in [one_byte, bytes.chunks(7).collect()] {
        let out = String::from_utf8(stream(&gate, &chunks)).expect("utf8");
        assert!(out.starts_with("your backup:\n"), "{out}");
        assert!(out.contains("[WITHHELD:"), "{out}");
        assert!(!out.contains("ability"), "{out}");
    }
}

#[test]
fn pf_28_s02_display_streams_do_not_hold_ordinary_words() {
    let gate = gate("env:PF28S02", SecretClass::Operational, CANARY);
    let mut scrubber = StreamScrubber::new();
    let out = scrubber.push(
        &gate,
        OutputSink::ToolResult,
        b"hello wor",
        /*utf8*/ true,
    );
    assert_eq!(out.as_slice(), b"hello wor".as_slice());
    // A long encoded tail is held so it can be decoded whole.
    let out = scrubber.push(
        &gate,
        OutputSink::ToolResult,
        b"ld\nblob QUJDREVGR0hJSktMTU5P",
        /*utf8*/ true,
    );
    assert_eq!(out.as_slice(), b"ld\nblob ".as_slice());
    assert_eq!(scrubber.pending(), "QUJDREVGR0hJSktMTU5P".len());
}

#[test]
fn pf_28_s02_display_stream_decodes_a_run_whose_start_was_emitted() {
    let gate = gate("env:PF28S02", SecretClass::Operational, CANARY);
    let hex: String = CANARY.bytes().map(|byte| format!("{byte:02x}")).collect();
    let marker = "[REDACTED:env:PF28S02]";
    for nested in [
        STANDARD.encode(STANDARD.encode(format!("padding-{CANARY}"))),
        STANDARD.encode(format!("padding-{hex}")),
        // Up to 7 characters can go out before the hold: the padding covers
        // them, as any prefix before the value does.
        format!("padding-{}", STANDARD.encode(CANARY))
            .bytes()
            .map(|byte| format!("{byte:02x}"))
            .collect(),
    ] {
        let input = format!("out {nested} end");
        for lead in 1..8 {
            let (first, rest) = input.split_at(4 + lead);
            let out = String::from_utf8(stream(&gate, &[first.as_bytes(), rest.as_bytes()]))
                .expect("utf8");
            assert!(out.contains(marker), "lead {lead}: {out}");
            assert!(out.ends_with(" end"), "lead {lead}: {out}");
        }
    }
}
