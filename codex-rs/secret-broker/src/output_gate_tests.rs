use super::*;
use base64::engine::general_purpose::STANDARD;
use base64::engine::general_purpose::URL_SAFE;
use pretty_assertions::assert_eq;
use std::sync::Barrier;

// Synthetic canaries only.
const CANARY: &str = "pf28-canary-Zq7/x+Y=k&p 9";
const LABEL: &str = "env:PF28_CANARY";

fn gate_with(value: &str) -> OutputGate {
    let gate = OutputGate::new();
    gate.register(LABEL, SecretClass::Operational, value)
        .expect("register");
    gate
}

fn scrubbed(gate: &OutputGate, sink: OutputSink, input: &str) -> String {
    gate.scrub(sink, input)
        .map(|(text, _)| text)
        .unwrap_or_else(|| input.to_string())
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

/// No run of `min` or more bytes of the value survives.
fn assert_no_fragment(output: &[u8], value: &str, min: usize) {
    let value = value.as_bytes();
    for start in 0..value.len() {
        for end in (start + min)..=value.len() {
            let fragment = &value[start..end];
            assert!(
                !output.windows(fragment.len()).any(|w| w == fragment),
                "fragment {start}..{end} leaked"
            );
        }
    }
}

#[test]
fn pf_28_s01_exact_value_and_encodings_are_removed_whole() {
    let gate = gate_with(CANARY);
    let json = serde_json::to_string(CANARY).expect("json");
    let mut encodings = vec![
        CANARY.to_string(),
        json[1..json.len() - 1].to_string(),
        percent_encode(CANARY.as_bytes(), true, false)
            .into_iter()
            .map(char::from)
            .collect(),
        percent_encode(CANARY.as_bytes(), false, false)
            .into_iter()
            .map(char::from)
            .collect(),
        hex(CANARY.as_bytes(), false)
            .into_iter()
            .map(char::from)
            .collect(),
        hex(CANARY.as_bytes(), true)
            .into_iter()
            .map(char::from)
            .collect(),
    ];
    // Base64 at every alignment inside a larger blob (`user:` prefixes).
    for prefix in ["", "u", "us", "use:"] {
        let joined = format!("{prefix}{CANARY}");
        encodings.push(STANDARD.encode(&joined));
        encodings.push(URL_SAFE.encode(&joined));
    }
    for encoded in encodings {
        let input = format!("before {encoded} after");
        let (output, provenance) = gate
            .scrub(OutputSink::ToolResult, &input)
            .unwrap_or_else(|| panic!("not found: {encoded}"));
        // Base64 partial characters and padding go with the value; only
        // characters encoding a non-secret prefix (`use:`) may remain.
        assert!(output.starts_with("before "), "{output}");
        assert!(
            output.ends_with("[REDACTED:env:PF28_CANARY] after"),
            "{output}"
        );
        assert!(
            output.len() <= "before [REDACTED:env:PF28_CANARY] after".len() + 5,
            "{output}"
        );
        assert!(!output.contains(CANARY));
        assert_eq!(provenance.labels, vec![LABEL.to_string()]);
        assert!(provenance.changed());
    }
}

#[test]
fn pf_28_s01_repeated_values_are_all_removed() {
    let gate = gate_with(CANARY);
    let input = format!("{CANARY}{CANARY} x {CANARY}");
    let (output, provenance) = gate.scrub(OutputSink::Transcript, &input).expect("found");
    assert_eq!(
        output,
        "[REDACTED:env:PF28_CANARY] x [REDACTED:env:PF28_CANARY]"
    );
    assert_eq!(provenance.redacted, 2);
}

#[test]
fn pf_28_s01_short_values_fail_safe() {
    let gate = OutputGate::new();
    assert_eq!(
        gate.register("short", SecretClass::Operational, "ab"),
        Err(RegisterError::TooShort)
    );
    gate.register("pin", SecretClass::Operational, "x7Q2")
        .expect("4-byte value");
    // Whole words only: ordinary text containing the letters is untouched.
    assert_eq!(
        scrubbed(&gate, OutputSink::ToolResult, "abx7Q2cd"),
        "abx7Q2cd"
    );
    assert_eq!(
        scrubbed(&gate, OutputSink::ToolResult, "pin=x7Q2; again x7Q2"),
        "pin=[REDACTED:pin]; again [REDACTED:pin]"
    );
    // Split across chunks it is still held back and removed.
    let out = stream(&gate, &[b"pin=x7", b"Q2 done"]);
    assert_eq!(
        String::from_utf8(out).expect("utf8"),
        "pin=[REDACTED:pin] done"
    );
}

#[test]
fn pf_28_s01_more_than_512_representations_all_protected() {
    let gate = OutputGate::new();
    let values: Vec<String> = (0..100).map(|i| format!("pf28-value-{i:04}-Qz9")).collect();
    let labels: Vec<String> = (0..values.len()).map(|i| format!("v{i}")).collect();
    let batch: Vec<(&str, SecretClass, &str)> = labels
        .iter()
        .zip(&values)
        .map(|(label, value)| (label.as_str(), SecretClass::Operational, value.as_str()))
        .collect();
    gate.register_all(&batch).expect("register");
    assert!(
        gate.representation_count() > 512,
        "{}",
        gate.representation_count()
    );
    for value in &values {
        let b64 = STANDARD.encode(value);
        let input = format!("a {value} b {b64} c");
        let output = scrubbed(&gate, OutputSink::ModelRequest, &input);
        assert!(!output.contains(value.as_str()), "{output}");
        assert!(!output.contains(b64.trim_end_matches('=')), "{output}");
    }
}

#[test]
fn pf_28_s01_capacity_exhaustion_denies_without_evicting() {
    let gate = OutputGate::new();
    let mut admitted: Vec<String> = Vec::new();
    let mut denied = None;
    for batch in 0..200 {
        let values: Vec<String> = (0..50)
            .map(|i| format!("pf28-capacity-{batch:03}-{i:02}-Kx"))
            .collect();
        let entries: Vec<(&str, SecretClass, &str)> = values
            .iter()
            .map(|value| ("cap", SecretClass::Operational, value.as_str()))
            .collect();
        match gate.register_all(&entries) {
            Ok(_) => admitted.extend(values),
            Err(err) => {
                denied = Some((values, err));
                break;
            }
        }
    }
    let (rejected, err) = denied.expect("capacity is bounded");
    assert_eq!(
        err,
        RegisterError::CapacityExhausted {
            limit: MAX_REPRESENTATIONS
        }
    );
    assert!(gate.representation_count() <= MAX_REPRESENTATIONS);
    assert_eq!(gate.value_count(), admitted.len());
    // Every earlier value is still protected; the rejected batch is not
    // admitted at all (the caller must not use those values).
    for value in [
        &admitted[0],
        &admitted[admitted.len() / 2],
        admitted.last().expect("some"),
    ] {
        assert!(!scrubbed(&gate, OutputSink::ToolResult, value).contains(value.as_str()));
    }
    for value in &rejected {
        assert_eq!(&scrubbed(&gate, OutputSink::ToolResult, value), value);
    }
    // Single registrations fill the rest, then fail the same way.
    let mut i = 0;
    let err = loop {
        match gate.register(
            "one",
            SecretClass::Operational,
            &format!("pf28-single-{i:04}-Kx"),
        ) {
            Ok(_) => i += 1,
            Err(err) => break err,
        }
    };
    assert!(matches!(err, RegisterError::CapacityExhausted { .. }));
    assert!(!scrubbed(&gate, OutputSink::ToolResult, &admitted[0]).contains(admitted[0].as_str()));
}

#[test]
fn pf_28_s01_overlapping_values_leave_no_fragment() {
    let gate = OutputGate::new();
    gate.register("a", SecretClass::Operational, "alpha-1234-beta")
        .expect("a");
    gate.register("b", SecretClass::Operational, "1234-beta-gamma-99")
        .expect("b");
    gate.register("c", SecretClass::Operational, "beta")
        .expect("c");
    let (output, provenance) = gate
        .scrub(OutputSink::ToolResult, "x alpha-1234-beta-gamma-99 y")
        .expect("found");
    assert_eq!(output, "x [REDACTED:a,b,c] y");
    assert_eq!(provenance.redacted, 1);
    assert_eq!(provenance.labels, vec!["a", "b", "c"]);
}

#[test]
fn pf_28_s01_chunk_boundaries_never_split_a_value() {
    let gate = gate_with(CANARY);
    let input = format!("start {CANARY} middle {} end", STANDARD.encode(CANARY));
    let whole = scrubbed(&gate, OutputSink::ToolResult, &input);
    let bytes = input.as_bytes();
    for cut in 0..=bytes.len() {
        let out = stream(&gate, &[&bytes[..cut], &bytes[cut..]]);
        assert_eq!(
            String::from_utf8(out.clone()).expect("utf8"),
            whole,
            "cut {cut}"
        );
        assert_no_fragment(&out, CANARY, 6);
    }
    let one_byte_chunks: Vec<&[u8]> = bytes.chunks(1).collect();
    let out = stream(&gate, &one_byte_chunks);
    assert_eq!(String::from_utf8(out).expect("utf8"), whole);
}

#[test]
fn pf_28_s01_stream_holds_back_at_most_the_longest_value() {
    let gate = gate_with(CANARY);
    let mut scrubber = StreamScrubber::new();
    let out = scrubber.push(&gate, OutputSink::ToolResult, &[b'z'; 4096], true);
    assert!(scrubber.pending() <= gate.snapshot().max_len + BASE64_SLACK + 2);
    assert_eq!(out.len() + scrubber.pending(), 4096);
}

#[test]
fn pf_28_s01_stream_holds_back_only_a_possible_value_start() {
    let gate = gate_with(CANARY);
    let mut scrubber = StreamScrubber::new();
    // Ordinary text streams through with nothing held back.
    let out = scrubber.push(
        &gate,
        OutputSink::ToolResult,
        b"ordinary output, no value here ",
        true,
    );
    assert_eq!(out, b"ordinary output, no value here ");
    assert_eq!(scrubber.pending(), 0);
    // A tail that could start the value is held with one boundary byte.
    let start = &CANARY.as_bytes()[..5];
    let mut chunk = b"next ".to_vec();
    chunk.extend_from_slice(start);
    let out = scrubber.push(&gate, OutputSink::ToolResult, &chunk, true);
    assert_eq!(out, b"next");
    assert_eq!(scrubber.pending(), start.len() + 1);
}

#[test]
fn pf_28_s01_retire_keeps_a_value_another_owner_registered() {
    let gate = OutputGate::new();
    let first = gate
        .register("first", SecretClass::Operational, "pf28-shared-value-1")
        .expect("first");
    let second = gate
        .register("second", SecretClass::Operational, "pf28-shared-value-1")
        .expect("second");
    assert_eq!(first, second);
    gate.retire(first);
    assert!(
        gate.scrub(OutputSink::ToolResult, "pf28-shared-value-1")
            .is_some()
    );
    gate.retire(second);
    assert_eq!(
        gate.scrub(OutputSink::ToolResult, "pf28-shared-value-1"),
        None
    );
}

#[test]
fn pf_28_s01_multibyte_text_is_cut_on_char_boundaries() {
    let gate = gate_with("geheim-schlüssel-ä9");
    let input = "ä ö ü geheim-schlüssel-ä9 ß";
    let bytes = input.as_bytes();
    for cut in 0..=bytes.len() {
        let mut scrubber = StreamScrubber::new();
        let first = scrubber.push(&gate, OutputSink::ModelResponse, &bytes[..cut], true);
        assert!(String::from_utf8(first.clone()).is_ok(), "cut {cut}");
        let second = scrubber.push(&gate, OutputSink::ModelResponse, &bytes[cut..], true);
        let rest = scrubber.finish(&gate, OutputSink::ModelResponse);
        let out = [first, second, rest].concat();
        assert_eq!(
            String::from_utf8(out).expect("utf8"),
            "ä ö ü [REDACTED:env:PF28_CANARY] ß"
        );
    }
}

#[test]
fn pf_28_s01_rotation_keeps_in_flight_values_protected() {
    let gate = OutputGate::new();
    let old = gate
        .register("token", SecretClass::Operational, "pf28-old-token-AAAA")
        .expect("old");
    let lease = gate.lease(old).expect("lease");
    let new = gate
        .rotate(
            old,
            "token",
            SecretClass::Operational,
            "pf28-new-token-BBBB",
        )
        .expect("rotate");
    assert_ne!(old, new);
    // Retired but leased: still protected while the response is in flight.
    let text = "pf28-old-token-AAAA pf28-new-token-BBBB";
    assert_eq!(
        scrubbed(&gate, OutputSink::ModelResponse, text),
        "[REDACTED:token] [REDACTED:token]"
    );
    drop(lease);
    assert_eq!(
        scrubbed(&gate, OutputSink::ModelResponse, text),
        "pf28-old-token-AAAA [REDACTED:token]"
    );
}

#[test]
fn pf_28_s01_failed_rotation_keeps_the_old_value() {
    let gate = OutputGate::new();
    let old = gate
        .register("token", SecretClass::Operational, "pf28-old-token-CCCC")
        .expect("old");
    assert_eq!(
        gate.rotate(old, "token", SecretClass::Operational, "x"),
        Err(RegisterError::TooShort)
    );
    assert_eq!(
        scrubbed(&gate, OutputSink::ToolResult, "pf28-old-token-CCCC"),
        "[REDACTED:token]"
    );
}

#[test]
fn pf_28_s01_concurrent_rotation_never_unprotects_a_leased_value() {
    let gate = OutputGate::new();
    let held = gate
        .register("held", SecretClass::Operational, "pf28-held-value-ZZZZ")
        .expect("held");
    let lease = gate.lease(held).expect("lease");
    let threads = 4;
    let barrier = Arc::new(Barrier::new(threads + 1));
    let mut workers = Vec::new();
    for _ in 0..threads {
        let gate = gate.clone();
        let barrier = Arc::clone(&barrier);
        workers.push(std::thread::spawn(move || {
            barrier.wait();
            for _ in 0..200 {
                let out = scrubbed(&gate, OutputSink::ToolResult, "x pf28-held-value-ZZZZ y");
                assert_eq!(out, "x [REDACTED:held] y");
            }
        }));
    }
    barrier.wait();
    let mut current = gate
        .register("rot", SecretClass::Operational, "pf28-rotating-00000")
        .expect("rot");
    for i in 1..100 {
        current = gate
            .rotate(
                current,
                "rot",
                SecretClass::Operational,
                &format!("pf28-rotating-{i:05}"),
            )
            .expect("rotate");
        // The held value is retired concurrently but stays protected.
        gate.retire(held);
    }
    for worker in workers {
        worker.join().expect("worker");
    }
    drop(lease);
    assert_eq!(
        scrubbed(&gate, OutputSink::ToolResult, "pf28-held-value-ZZZZ"),
        "pf28-held-value-ZZZZ"
    );
}

#[test]
fn pf_28_s01_seed_and_private_key_are_withheld_not_redacted() {
    let gate = OutputGate::new();
    let seed = "abandon ability able about above absent absorb abstract";
    gate.register("vault:wallet-seed", SecretClass::SeedPhrase, seed)
        .expect("seed");
    let (output, provenance) = gate
        .scrub(OutputSink::ToolResult, &format!("words: {seed}\nother"))
        .expect("withheld");
    assert!(provenance.withheld);
    assert!(output.starts_with("[WITHHELD: output contained a protected seed phrase"));
    assert!(!output.contains("abandon"));
    assert!(!output.contains("other"));
    // In a stream, the rest of the stream is withheld too.
    let input = format!("ok {seed} tail");
    let out = stream(&gate, &[&input.as_bytes()[..10], &input.as_bytes()[10..]]);
    let out = String::from_utf8(out).expect("utf8");
    assert!(!out.contains("abandon ability"), "{out}");
    assert!(!out.contains("tail"), "{out}");
}

#[test]
fn pf_28_s01_oversized_payload_is_withheld() {
    let gate = OutputGate::new();
    let big = "a".repeat(MAX_SCAN_BYTES + 1);
    let (output, provenance) = gate.scrub(OutputSink::Export, &big).expect("withheld");
    assert!(provenance.withheld);
    assert!(output.contains("too large to check for secrets"));
}

#[test]
fn pf_28_s01_key_patterns_apply_to_diagnostic_sinks_only() {
    let gate = OutputGate::new();
    let text =
        "log: Authorization: Bearer abcdefghijklmnopqrstuvwxyz012345 sk-ABCDEFGHIJKLMNOPQRSTUV";
    assert_eq!(gate.scrub(OutputSink::ToolResult, text), None);
    let (output, provenance) = gate.scrub(OutputSink::Diagnostic, text).expect("patterns");
    assert!(!output.contains("abcdefghijklmnop"), "{output}");
    assert!(!output.contains("ABCDEFGHIJKLMNOP"), "{output}");
    assert!(provenance.pattern_redacted > 0);
    let pem =
        "-----BEGIN OPENSSH PRIVATE KEY-----\nAAAAB3NzaC1yc2E\n-----END OPENSSH PRIVATE KEY-----";
    assert_eq!(
        scrubbed(&gate, OutputSink::Trace, pem),
        "[REDACTED:key-pattern]"
    );
}

#[test]
fn pf_28_s01_disarmed_gate_passes_through_and_keeps_registrations() {
    let gate = OutputGate::new_disarmed();
    gate.register(LABEL, SecretClass::Operational, CANARY)
        .expect("register before arming");
    assert_eq!(gate.scrub(OutputSink::ToolResult, CANARY), None);
    let mut scrubber = StreamScrubber::new();
    assert_eq!(
        scrubber.push(&gate, OutputSink::ToolResult, CANARY.as_bytes(), true),
        CANARY.as_bytes()
    );
    gate.arm().expect("arm");
    assert_eq!(
        scrubbed(&gate, OutputSink::ToolResult, CANARY),
        "[REDACTED:env:PF28_CANARY]"
    );
}

#[test]
fn pf_28_s01_debug_and_markers_never_show_values() {
    let gate = gate_with(CANARY);
    assert!(!format!("{gate:?}").contains("canary"));
    gate.register(
        "bad label <script>",
        SecretClass::Operational,
        "pf28-label-test-1",
    )
    .expect("label");
    assert_eq!(
        scrubbed(&gate, OutputSink::ToolResult, "pf28-label-test-1"),
        "[REDACTED:badlabelscript]"
    );
}

#[test]
fn pf_28_s01_short_value_is_not_redacted_inside_a_longer_word_at_a_cut() {
    let gate = OutputGate::new();
    gate.register("short", SecretClass::Operational, "abc")
        .expect("register");
    let mut scrubber = StreamScrubber::new();
    let mut out = scrubber.push(&gate, OutputSink::ToolResult, b"abcdzzq", true);
    out.extend(scrubber.push(&gate, OutputSink::ToolResult, b"x end abc.", true));
    out.extend(scrubber.finish(&gate, OutputSink::ToolResult));
    assert_eq!(
        String::from_utf8(out).expect("utf8"),
        "abcdzzqx end [REDACTED:short]."
    );
}
