//! PF-28-S02 rescans: managed values hidden by line wrapping, by a second
//! encoding, or (seed phrases) by other separators.
//!
//! - Wrapped blocks: `base64` (76 columns), `openssl base64` (64) and
//!   `xxd -p` (60) break an encoded value across lines. A line break (raw or
//!   JSON-escaped, plus indentation) after a run of at least
//!   [`MIN_WRAPPED_LINE`] encoding characters, or after a run that starts
//!   the input (a stream chunk may begin mid-line), is removed in a joined
//!   view, which is scanned like the input.
//! - Decode and rescan: runs of at least [`MIN_DECODE_RUN`] base64 or hex
//!   characters are decoded, up to [`MAX_DECODE_DEPTH`] levels, and the
//!   result is scanned for substring values and seed phrases. A hit redacts
//!   the whole run.
//! - Seed phrases: three consecutive words of a registered phrase, in order,
//!   whatever separates them (commas, numbering, quotes, escapes, lines).

use super::MAX_BLOCK_CARRY;
use super::Match;
use super::SecretClass;
use super::Snapshot;
use super::find_in;
use super::is_base64_byte;
use super::longest_partial_suffix;
use base64::Engine as _;
use base64::engine::DecodePaddingMode;
use base64::engine::GeneralPurpose;
use base64::engine::GeneralPurposeConfig;
use std::sync::Arc;
use zeroize::Zeroizing;

/// A line break is joined only after this many encoding characters.
const MIN_WRAPPED_LINE: usize = 32;
/// Shortest run of encoding characters that is decoded and rescanned.
pub(super) const MIN_DECODE_RUN: usize = 24;
/// Decoding levels (base64 of hex, base64 of base64, ...).
const MAX_DECODE_DEPTH: usize = 2;
/// Longest separator between two words of a seed phrase.
const MAX_SEED_GAP: usize = 16;
/// Words in a seed window.
const SEED_WINDOW: usize = 3;

const LENIENT: GeneralPurposeConfig = GeneralPurposeConfig::new()
    .with_decode_allow_trailing_bits(true)
    .with_decode_padding_mode(DecodePaddingMode::Indifferent);
const STANDARD_LENIENT: GeneralPurpose = GeneralPurpose::new(&base64::alphabet::STANDARD, LENIENT);
const URL_SAFE_LENIENT: GeneralPurpose = GeneralPurpose::new(&base64::alphabet::URL_SAFE, LENIENT);

/// Seed-phrase windows and words, built from every seed-phrase value.
#[derive(Default)]
pub(super) struct SeedIndex {
    /// Lowercase `"w1 w2 w3"`, sorted.
    windows: Vec<(Zeroizing<Vec<u8>>, Arc<str>, SecretClass)>,
    /// Every word, sorted and deduplicated.
    vocab: Vec<Zeroizing<Vec<u8>>>,
}

impl SeedIndex {
    pub(super) fn build<'a>(
        phrases: impl Iterator<Item = (&'a [u8], Arc<str>, SecretClass)>,
    ) -> Self {
        let mut index = Self::default();
        for (phrase, label, class) in phrases {
            let words: Vec<Zeroizing<Vec<u8>>> = words(phrase)
                .into_iter()
                .map(|(start, end)| lowercase(&phrase[start..end]))
                .collect();
            for window in words.windows(SEED_WINDOW) {
                index
                    .windows
                    .push((join_words(window), Arc::clone(&label), class));
            }
            index.vocab.extend(words);
        }
        index
            .windows
            .sort_by(|a, b| a.0.as_slice().cmp(b.0.as_slice()));
        index.vocab.sort_by(|a, b| a.as_slice().cmp(b.as_slice()));
        index.vocab.dedup_by(|a, b| a.as_slice() == b.as_slice());
        index
    }

    fn is_empty(&self) -> bool {
        self.windows.is_empty()
    }
}

/// Matches the direct scan cannot see: values in wrapped blocks, values
/// inside decoded runs, and seed phrases with other separators.
pub(super) fn rescan(snapshot: &Snapshot, input: &[u8]) -> Vec<Match> {
    if snapshot.reps.is_empty() {
        return Vec::new();
    }
    let mut found = seed_matches(snapshot, input);
    match join_wrapped(input) {
        Some(joined) => {
            let mut inner = find_in(snapshot, &joined.bytes, /*whole_words*/ true);
            decode_runs(snapshot, &joined.bytes, MAX_DECODE_DEPTH, &mut inner);
            found.extend(inner.into_iter().map(|m| joined.map(m)));
        }
        None => decode_runs(snapshot, input, MAX_DECODE_DEPTH, &mut found),
    }
    found
}

/// Where a stream must hold back so a value is not emitted in part: the
/// longest tail of a wrapped block (its last line break may already have
/// arrived) that could still grow into a value, an encoded run that may
/// continue (up to [`MAX_BLOCK_CARRY`], so it is decoded whole), or seed
/// words that may start a window.
pub(super) fn stream_hold(snapshot: &Snapshot, buffer: &[u8]) -> Option<usize> {
    if snapshot.reps.is_empty() {
        return None;
    }
    let joined = join_wrapped(buffer);
    let view: &[u8] = joined.as_ref().map_or(buffer, |joined| &joined.bytes);
    let at = |index: usize| joined.as_ref().map_or(index, |joined| joined.origin(index));
    let mut hold = None::<usize>;
    let mut keep = |start: usize| hold = Some(hold.map_or(start, |hold| hold.min(start)));
    let end = wrapped_line_end(view).unwrap_or(view.len());
    if joined.is_some() || end < view.len() {
        let partial = longest_partial_suffix(snapshot, &view[..end]);
        if partial > 0 {
            keep(at(end - partial).saturating_sub(1));
        }
    }
    let run = view[..end]
        .iter()
        .rev()
        .take_while(|byte| is_base64_byte(**byte))
        .count();
    // Any length: a run whose first characters were already emitted would
    // be decoded out of alignment.
    if run > 0 && view.len() - (end - run) <= MAX_BLOCK_CARRY {
        keep(at(end - run));
    }
    if let Some(start) = seed_hold(snapshot, buffer) {
        keep(start);
    }
    hold
}

/// A view of the input with wrapped-block line breaks removed. `breaks`
/// holds, for each removed break, the view index after it and the input
/// index that view byte came from.
struct Joined {
    bytes: Zeroizing<Vec<u8>>,
    breaks: Vec<(usize, usize)>,
}

impl Joined {
    /// The input index of view byte `index` (or of the end, for `len`).
    fn origin(&self, index: usize) -> usize {
        match self.breaks.partition_point(|(view, _)| *view <= index) {
            0 => index,
            after => {
                let (view, input) = self.breaks[after - 1];
                input + (index - view)
            }
        }
    }

    fn map(&self, m: Match) -> Match {
        Match {
            start: self.origin(m.start),
            end: self.origin(m.end - 1) + 1,
            ..m
        }
    }
}

fn join_wrapped(input: &[u8]) -> Option<Joined> {
    let mut bytes = Zeroizing::new(Vec::new());
    let mut breaks = Vec::new();
    let mut joined = false;
    let mut run = 0;
    let mut leading = true;
    let mut at = 0;
    while at < input.len() {
        if (run >= MIN_WRAPPED_LINE || (leading && run > 0))
            && let Some(next) = line_break_end(input, at)
            && input.get(next).is_some_and(|byte| is_base64_byte(*byte))
        {
            if !joined {
                bytes.reserve(input.len());
                bytes.extend_from_slice(&input[..at]);
                joined = true;
            }
            breaks.push((bytes.len(), next));
            at = next;
            continue;
        }
        if joined {
            bytes.push(input[at]);
        }
        if is_base64_byte(input[at]) {
            run += 1;
        } else {
            run = 0;
            leading = false;
        }
        at += 1;
    }
    joined.then_some(Joined { bytes, breaks })
}

/// End of a line break at `at` (`\n`, `\r\n`, or their JSON escapes) and
/// the indentation after it.
fn line_break_end(input: &[u8], at: usize) -> Option<usize> {
    let rest = &input[at..];
    let width = [b"\r\n".as_slice(), b"\n", b"\\r\\n", b"\\n"]
        .into_iter()
        .find(|brk| rest.starts_with(brk))?
        .len();
    let mut end = at + width;
    while input
        .get(end)
        .is_some_and(|byte| matches!(byte, b' ' | b'\t'))
    {
        end += 1;
    }
    Some(end)
}

/// Where a wrapped block's last line ends when the input stops right after
/// its line break (and indentation): the next line may continue it.
fn wrapped_line_end(view: &[u8]) -> Option<usize> {
    let mut end = view.len();
    while end > 0 && matches!(view[end - 1], b' ' | b'\t') {
        end -= 1;
    }
    let rest = &view[..end];
    let width = [b"\r\n".as_slice(), b"\n", b"\\r\\n", b"\\n"]
        .into_iter()
        .find(|brk| rest.ends_with(brk))?
        .len();
    let end = end - width;
    let run = view[..end]
        .iter()
        .rev()
        .take_while(|byte| is_base64_byte(**byte))
        .count();
    (run >= MIN_WRAPPED_LINE || (run > 0 && run == end)).then_some(end)
}

/// Decodes each long run in `view` and reports the run when its decoded
/// form holds a managed value.
fn decode_runs(snapshot: &Snapshot, view: &[u8], depth: usize, out: &mut Vec<Match>) {
    let mut start = 0;
    while start < view.len() {
        if !is_base64_byte(view[start]) {
            start += 1;
            continue;
        }
        let mut end = start;
        while end < view.len() && is_base64_byte(view[end]) {
            end += 1;
        }
        if end - start >= MIN_DECODE_RUN
            && let Some(hit) = decodings(&view[start..end])
                .iter()
                .filter_map(|decoded| strictest_hit(snapshot, decoded, depth))
                .max_by_key(|hit| hit.class)
        {
            let mut padded = end;
            while padded < view.len() && padded - end < 2 && view[padded] == b'=' {
                padded += 1;
            }
            out.push(Match {
                start,
                end: padded,
                ..hit
            });
        }
        start = end;
    }
}

/// The strictest managed value in decoded bytes. Short whole-word values
/// are not searched: decoded noise would match them by chance.
fn strictest_hit(snapshot: &Snapshot, decoded: &[u8], depth: usize) -> Option<Match> {
    let mut hits = find_in(snapshot, decoded, /*whole_words*/ false);
    hits.extend(seed_matches(snapshot, decoded));
    if depth > 1 {
        decode_runs(snapshot, decoded, depth - 1, &mut hits);
    }
    hits.into_iter().max_by_key(|hit| hit.class)
}

fn decodings(run: &[u8]) -> Vec<Zeroizing<Vec<u8>>> {
    let mut out = Vec::new();
    if run.len().is_multiple_of(2) && run.iter().all(u8::is_ascii_hexdigit) {
        out.push(Zeroizing::new(
            run.chunks(2)
                .map(|pair| (hex_digit(pair[0]) << 4) | hex_digit(pair[1]))
                .collect(),
        ));
    }
    let url_safe = run.iter().any(|byte| matches!(byte, b'-' | b'_'));
    let standard = run.iter().any(|byte| matches!(byte, b'+' | b'/'));
    if !(url_safe && standard) {
        let engine = if url_safe {
            &URL_SAFE_LENIENT
        } else {
            &STANDARD_LENIENT
        };
        // A length of 1 mod 4 cannot be base64; drop the stray character.
        let usable = &run[..run.len() - usize::from(run.len() % 4 == 1)];
        if let Ok(bytes) = engine.decode(usable) {
            out.push(Zeroizing::new(bytes));
        }
    }
    out
}

fn hex_digit(byte: u8) -> u8 {
    match byte {
        b'0'..=b'9' => byte - b'0',
        b'a'..=b'f' => byte - b'a' + 10,
        _ => byte - b'A' + 10,
    }
}

/// Three consecutive words of a registered seed phrase.
fn seed_matches(snapshot: &Snapshot, input: &[u8]) -> Vec<Match> {
    let seed = &snapshot.seed;
    if seed.is_empty() {
        return Vec::new();
    }
    let tokens = words(input);
    let mut found = Vec::new();
    for window in tokens.windows(SEED_WINDOW) {
        if window
            .windows(2)
            .any(|pair| pair[1].0 - pair[0].1 > MAX_SEED_GAP)
        {
            continue;
        }
        let words: Vec<Zeroizing<Vec<u8>>> = window
            .iter()
            .map(|(start, end)| lowercase(&input[*start..*end]))
            .collect();
        let key = join_words(&words);
        if let Ok(index) = seed
            .windows
            .binary_search_by(|(window, _, _)| window.as_slice().cmp(key.as_slice()))
        {
            let (_, label, class) = &seed.windows[index];
            found.push(Match {
                start: window[0].0,
                end: window[SEED_WINDOW - 1].1,
                label: Arc::clone(label),
                class: *class,
            });
        }
    }
    found
}

/// Start of the trailing words that could begin a seed window: the last two
/// complete words, plus a last word the chunk boundary may have cut.
fn seed_hold(snapshot: &Snapshot, buffer: &[u8]) -> Option<usize> {
    let vocab = &snapshot.seed.vocab;
    if vocab.is_empty() {
        return None;
    }
    let tail = buffer.len().saturating_sub(512);
    let tokens = words(&buffer[tail..]);
    let mut hold = None;
    let mut next = buffer.len();
    let cut_word = tokens
        .last()
        .is_some_and(|(_, end)| end + tail == buffer.len());
    let take = SEED_WINDOW - 1 + usize::from(cut_word);
    for (index, (start, end)) in tokens.iter().rev().take(take).enumerate() {
        let (start, end) = (start + tail, end + tail);
        if next - end > MAX_SEED_GAP {
            break;
        }
        let word = lowercase(&buffer[start..end]);
        let candidate = vocab.partition_point(|known| known.as_slice() < word.as_slice());
        let known = if index == 0 && end == buffer.len() {
            vocab
                .get(candidate)
                .is_some_and(|known| known.starts_with(&word))
        } else {
            vocab
                .get(candidate)
                .is_some_and(|known| known.as_slice() == word.as_slice())
        };
        if !known {
            break;
        }
        hold = Some(start);
        next = start;
    }
    hold
}

/// Word ranges: runs of letters (any non-ASCII byte counts as a letter, so
/// accented word lists tokenize). A backslash escape (`\n`, `\u00e9`)
/// separates words.
fn words(input: &[u8]) -> Vec<(usize, usize)> {
    let mut found = Vec::new();
    let mut start = None;
    let mut at = 0;
    while at < input.len() {
        let byte = input[at];
        if byte == b'\\' {
            if let Some(word_start) = start.take() {
                found.push((word_start, at));
            }
            let skip = if input.get(at + 1) == Some(&b'u') {
                6
            } else {
                2
            };
            at += skip;
            continue;
        }
        let letter = byte.is_ascii_alphabetic() || byte >= 0x80;
        match (letter, start) {
            (true, None) => start = Some(at),
            (false, Some(word_start)) => {
                found.push((word_start, at));
                start = None;
            }
            _ => {}
        }
        at += 1;
    }
    if let Some(word_start) = start {
        found.push((word_start, input.len()));
    }
    found
}

fn lowercase(word: &[u8]) -> Zeroizing<Vec<u8>> {
    Zeroizing::new(word.to_ascii_lowercase())
}

fn join_words(words: &[Zeroizing<Vec<u8>>]) -> Zeroizing<Vec<u8>> {
    let mut joined = Zeroizing::new(Vec::new());
    for (index, word) in words.iter().enumerate() {
        if index > 0 {
            joined.push(b' ');
        }
        joined.extend_from_slice(word);
    }
    joined
}

#[cfg(test)]
#[path = "output_gate_rescan_tests.rs"]
mod tests;
