use super::*;
use pretty_assertions::assert_eq;

fn record(value: &[u8], prefix: &[u8]) -> Fingerprint {
    Fingerprint {
        length: value.len(),
        prefix: prefix.to_vec(),
        digest: sha2::Sha256::new()
            .chain_update(prefix)
            .chain_update(value)
            .finalize()
            .into(),
    }
}

// Original per-record SHA-256 matcher, without its obsolete 2^21 limit.
// Keep this independent of the new grouping, rolling index and cache.
fn old_match(text: &str, records: &[Fingerprint]) -> bool {
    let text = text.as_bytes();
    for record in records {
        if record.length > text.len() {
            continue;
        }
        for start in 0..=text.len() - record.length {
            let word = |byte: u8| byte.is_ascii_alphanumeric() || byte == b'_';
            if record.length < 8
                && ((start > 0 && word(text[start - 1]))
                    || text.get(start + record.length).copied().is_some_and(word))
            {
                continue;
            }
            let digest: [u8; 32] = sha2::Sha256::new()
                .chain_update(&record.prefix)
                .chain_update(&text[start..start + record.length])
                .finalize()
                .into();
            if digest == record.digest {
                return true;
            }
        }
    }
    false
}

#[test]
fn randomized_equivalence_to_original_matcher() {
    use rand::Rng;
    use rand::SeedableRng;
    let mut rng = rand::rngs::StdRng::seed_from_u64(/*state*/ 2502);
    for case in 0..600 {
        let length = rng.random_range(0..300);
        let mut text: Vec<u8> = (0..length).map(|_| rng.random_range(0..128)).collect();
        // Repeated text exercises exact cache hits, including punctuation/NUL.
        if case % 3 == 0 {
            text = text.repeat(/*n*/ 3);
        }
        let mut records = Vec::new();
        for index in 0..8 {
            let value_len = rng.random_range(3..40);
            let value: Vec<u8> = if case % 2 == 0 && text.len() >= value_len {
                let start = rng.random_range(0..=text.len() - value_len);
                text[start..start + value_len].to_vec()
            } else {
                (0..value_len).map(|_| rng.random_range(0..128)).collect()
            };
            let prefix = if index % 2 == 0 {
                b"model".as_slice()
            } else {
                b"provider\0".as_slice()
            };
            records.push(record(&value, prefix));
        }
        let text = String::from_utf8(text).expect("ASCII");
        let expected = old_match(&text, &records);
        assert_eq!(
            scan(&text, records, MAX_WINDOWS),
            Ok(expected),
            "case {case}"
        );
    }
}

#[test]
fn full_rolling_hash_collision_requires_exact_bytes() {
    // Base-257 representation of 30 * 2^64: distinct ASCII windows with
    // identical rolling hashes, exercising the exact comparison on a hit.
    let collision = [29u8, 20, 62, 127, 37, 122, 68, 17, 30];
    let hash = collision.iter().fold(0u64, |h, b| {
        h.wrapping_mul(BASE).wrapping_add(u64::from(*b))
    });
    assert_eq!(hash, 0);
    let mut text = vec![0u8; collision.len()];
    text.extend_from_slice(&collision);
    assert_eq!(
        scan(
            &String::from_utf8(text).expect("ASCII"),
            vec![record(&collision, b"synthetic")],
            MAX_WINDOWS
        ),
        Ok(true)
    );
}

#[test]
fn long_repeated_windows_still_obey_the_byte_work_bound() {
    let records = vec![record(&vec![b'y'; 8192], b"synthetic")];
    assert_eq!(
        scan(&"x".repeat(/*n*/ 2 * 1024 * 1024), records, MAX_WINDOWS),
        Err(())
    );
}

#[test]
fn cache_collision_and_short_word_boundaries_do_not_hide_matches() {
    // Force many cache-slot collisions before a match at the end. Exact byte
    // comparison, rather than the rolling index, decides cache hits.
    let text = (0..40000)
        .map(|n| format!("function_{n:05} () {{ echo ordinary; }}\n"))
        .collect::<String>();
    for value in ["abc", "1234567", "12345678", "é-synthetic-key"] {
        let text = format!("{text}x{value}x '{value}'");
        let records = vec![record(value.as_bytes(), b"synthetic")];
        assert_eq!(scan(&text, records, MAX_WINDOWS), Ok(true));
    }
}

fn zsh_snapshot(bytes: usize) -> String {
    let mut text = String::new();
    for index in 0.. {
        use std::fmt::Write;
        writeln!(text, "_plugin_{index:05} () {{ local context state line; _arguments '-h[help]' '*:file:_files'; }}\nalias plugin_{index:05}='git status --short'").expect("text");
        if text.len() >= bytes {
            text.truncate(bytes);
            return text;
        }
    }
    unreachable!()
}

#[test]
fn three_mib_eight_isolated_lengths_pass_and_copies_at_end_are_found() {
    let text = zsh_snapshot(/*bytes*/ 3 * 1024 * 1024);
    let values: Vec<_> = (0..8)
        .map(|n| format!("synthetic-2502-{}", "k".repeat(16 + n)))
        .collect();
    let records = || {
        values
            .iter()
            .map(|value| record(value.as_bytes(), b"synthetic"))
            .collect()
    };
    assert_eq!(scan(&text, records(), MAX_WINDOWS), Ok(false));
    assert_eq!(
        scan(&format!("{text}{}", values[7]), records(), MAX_WINDOWS),
        Ok(true)
    );
}

#[test]
fn shared_budget_counts_distinct_lengths_and_refuses_past_bound() {
    let text = "x".repeat(/*n*/ 64);
    let records = || vec![record(b"abcdefgh", b"one"), record(b"ijklmnop", b"two")];
    assert_eq!(scan(&text, records(), /*max_windows*/ 57), Ok(false));
    assert_eq!(scan(&text, records(), /*max_windows*/ 56), Err(()));
    let records = vec![record(b"abcdefgh", b"one"), record(b"abcdefghi", b"two")];
    assert_eq!(scan(&text, records, /*max_windows*/ 112), Err(()));
    let records = (0..11)
        .map(|n| record(&vec![b'k'; 16 + n], b"synthetic"))
        .collect();
    assert_eq!(
        scan(&"x".repeat(/*n*/ 3 * 1024 * 1024), records, MAX_WINDOWS),
        Err(())
    );
}

#[test]
fn malformed_metadata_and_oversize_text_refuse() {
    for length in [0, 1, 2, 8193] {
        assert_eq!(
            scan(
                "safe",
                vec![record(&vec![b'x'; length], b"key")],
                MAX_WINDOWS
            ),
            Err(())
        );
    }
    assert_eq!(
        scan(
            &"x".repeat(codex_secret_broker::output_gate::MAX_SCAN_BYTES + 1),
            vec![],
            MAX_WINDOWS
        ),
        Err(())
    );
    assert_eq!(
        scan(
            "safe",
            (0..129).map(|_| record(b"synthetic", b"key")).collect(),
            MAX_WINDOWS
        ),
        Err(())
    );
}

#[test]
#[ignore = "manual release snapshot benchmark, synthetic inputs only"]
#[allow(clippy::print_stderr)]
fn snapshot_scan_release_benchmark() {
    for bytes in [512 * 1024, 1024 * 1024, 3 * 1024 * 1024] {
        let text = zsh_snapshot(bytes);
        for keys in [1, 2, 4, 8] {
            for distinct in [false, true] {
                let values: Vec<_> = (0..keys)
                    .map(|n| {
                        format!(
                            "synthetic-key-{n:02}-{}",
                            "k".repeat(if distinct { 16 + n } else { 16 })
                        )
                    })
                    .collect();
                for sample in 0..3 {
                    let records = values
                        .iter()
                        .map(|v| record(v.as_bytes(), b"synthetic"))
                        .collect();
                    let start = std::time::Instant::now();
                    let result = scan(&text, records, MAX_WINDOWS);
                    eprintln!(
                        "SNAPSHOT_SCAN bytes={bytes} keys={keys} distinct={distinct} sample={sample} us={} result={result:?}",
                        start.elapsed().as_micros()
                    );
                    assert_eq!(result, Ok(false));
                }
            }
        }
    }
}
