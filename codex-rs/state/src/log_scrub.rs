//! One-time scrub of credential-shaped values that older builds wrote to the
//! logs database and to log files (#179, #183, #196): provider bearer tokens,
//! header and query values, URL userinfo, response `set-cookie` headers and
//! environment values of spawned commands.
//!
//! Detection deliberately over-matches: a false positive only hides part of
//! a log line. Known limits: values split across a 1 MiB line piece, and
//! lines an older build still running writes after the pass.

use std::fs::File;
use std::fs::OpenOptions;
use std::io;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;
use std::ops::Range;
use std::path::Path;
use std::path::PathBuf;
use std::sync::LazyLock;

use regex::bytes::Regex;
use regex::bytes::RegexSet;

/// Replacement for scrubbed values in the logs database.
const REDACTED: &str = "REDACTED";

/// Lines longer than this are scrubbed in pieces; a value split across two
/// pieces can be missed.
const MAX_LINE_BYTES: u64 = 1024 * 1024;

const SECRET_NAME: &str = "(?:cookie|auth|token|key|secret|session|signature|credential|pass|pwd|mnemonic|seed|private|jwt|dsn)";
/// Credential-named keys whose values are diagnostics, not secrets.
const NOT_SECRET_SUFFIXES: [&str; 7] =
    ["_id", "_ids", "_count", "tokens", "_mode", "_kind", "_type"];
/// A quoted Debug/JSON string (quotes escaped at most once), and the same
/// capturing its contents as `v`.
const QUOTED: &str = r#"\\?"(?:[^"\\]|\\[^"])*\\?""#;
const QUOTED_VALUE: &str = r#"\\?"(?P<v>(?:[^"\\]|\\[^"])*)\\?""#;

/// Each pattern's group `v` is the value to hide; group `n`, when present,
/// is the key name, checked against [`NOT_SECRET_SUFFIXES`].
static PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    [
        // Header maps, environment maps and JSON: `"set-cookie": "v"`.
        format!(
            r#"(?i-u)\\?"(?P<n>[a-z0-9_.-]*{SECRET_NAME}[a-z0-9_.-]*)\\?"\s*(?::|=>|=)\s*{QUOTED_VALUE}"#
        ),
        // Rust `Debug` fields: `experimental_bearer_token: Some("v")`.
        format!(
            r#"(?i-u)\b(?P<n>[a-z0-9_]*{SECRET_NAME}[a-z0-9_]*): (?:Some\()?{QUOTED_VALUE}"#
        ),
        // `NAME=value`, `NAME="value"` for secret-named variables.
        format!(
            r#"(?i-u)\b(?P<n>[a-z0-9_]*{SECRET_NAME}[a-z0-9_]*)=(?:\\?["'])?(?P<v>[^\s"'\\&;,]+)"#
        ),
        // Bearer tokens, and Basic credentials after `authorization`.
        r"(?i-u)\bbearer\s+(?P<v>[a-z0-9._~+/=-]{8,})".to_string(),
        r#"(?i-u)authorization\\?["']?\s*[:=]\s*\\?["']?basic\s+(?P<v>[a-z0-9+/=]{8,})"#.to_string(),
        // URL query values and userinfo.
        r#"(?-u)[?&][A-Za-z0-9_.%-]+=(?P<v>[^&#\s"'<>)\]\\]+)"#.to_string(),
        r#"(?i-u)\b[a-z][a-z0-9+.-]*://(?P<v>[^/\s:@"']+:[^/\s@"']+)@"#.to_string(),
        // Well-known key formats anywhere.
        r"(?-u)\b(?P<v>sk-(?:ant-|proj-)?[A-Za-z0-9_-]{20,}|gh[pousr]_[A-Za-z0-9]{30,}|github_pat_[A-Za-z0-9_]{30,}|xox[abprs]-[A-Za-z0-9-]{10,}|AKIA[0-9A-Z]{16}|AIza[0-9A-Za-z_-]{35}|eyJ[A-Za-z0-9_-]{10,}\.eyJ[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,})"
            .to_string(),
    ]
    .iter()
    .map(|pattern| compile(pattern))
    .collect()
});

static PATTERN_SET: LazyLock<RegexSet> = LazyLock::new(|| {
    #[expect(clippy::expect_used)]
    RegexSet::new(PATTERNS.iter().map(Regex::as_str)).expect("log scrub patterns compile")
});

/// Maps whose values are secret whatever the key: a provider's
/// `http_headers` and `query_params` in its `Debug` output (#196).
static SECRET_MAP: LazyLock<Regex> =
    LazyLock::new(|| compile(r"(?-u)\b(?:http_headers|query_params): Some\(\{(?P<v>[^}]*)\}\)"));
/// Lines that printed a spawned command's whole environment (#179).
static ENV_DUMP_LINE: LazyLock<Regex> =
    LazyLock::new(|| compile(r"(?-u)spawn_child_async|ExecOneOffCommand"));
/// Every value of a `"key": "value"` map.
static MAP_VALUE: LazyLock<Regex> =
    LazyLock::new(|| compile(&format!(r#"(?-u){QUOTED}\s*:\s*{QUOTED_VALUE}"#)));

fn compile(pattern: &str) -> Regex {
    #[expect(clippy::expect_used)]
    Regex::new(pattern).expect("log scrub pattern compiles")
}

/// Sorted, non-overlapping byte ranges of credential-shaped values in
/// `text`. Values that are already redacted are skipped, so scrubbing is
/// idempotent.
fn secret_spans(text: &[u8]) -> Vec<Range<usize>> {
    let mut spans = Vec::new();
    for index in PATTERN_SET.matches(text).iter() {
        for captures in PATTERNS[index].captures_iter(text) {
            let is_diagnostic = captures.name("n").is_some_and(|name| {
                let name = name.as_bytes().to_ascii_lowercase();
                NOT_SECRET_SUFFIXES
                    .iter()
                    .any(|suffix| name.ends_with(suffix.as_bytes()))
            });
            if let Some(value) = captures.name("v").filter(|_| !is_diagnostic) {
                spans.push(value.range());
            }
        }
    }
    let mut map_values = |region: Range<usize>| {
        for captures in MAP_VALUE.captures_iter(&text[region.clone()]) {
            if let Some(value) = captures.name("v") {
                spans.push(region.start + value.start()..region.start + value.end());
            }
        }
    };
    for captures in SECRET_MAP.captures_iter(text) {
        if let Some(map) = captures.name("v") {
            map_values(map.range());
        }
    }
    if ENV_DUMP_LINE.is_match(text) {
        map_values(0..text.len());
    }
    spans.retain(|span| !is_redacted(&text[span.clone()]));
    spans.sort_by_key(|span| (span.start, span.end));
    let mut merged: Vec<Range<usize>> = Vec::with_capacity(spans.len());
    for span in spans {
        match merged.last_mut() {
            Some(last) if span.start <= last.end => last.end = last.end.max(span.end),
            _ => merged.push(span),
        }
    }
    merged
}

fn is_redacted(value: &[u8]) -> bool {
    value.is_empty()
        || value == REDACTED.as_bytes()
        || value == b"<redacted>"
        || value.iter().all(|byte| *byte == b'*')
}

/// `text` with every secret value replaced by `REDACTED`, or `None` when
/// nothing needs scrubbing. Invalid UTF-8 is replaced lossily.
pub(crate) fn scrub_bytes(text: &[u8]) -> Option<String> {
    let spans = secret_spans(text);
    if spans.is_empty() {
        return None;
    }
    let mut scrubbed = Vec::with_capacity(text.len());
    let mut position = 0;
    for span in spans {
        scrubbed.extend_from_slice(&text[position..span.start]);
        scrubbed.extend_from_slice(REDACTED.as_bytes());
        position = span.end;
    }
    scrubbed.extend_from_slice(&text[position..]);
    Some(String::from_utf8_lossy(&scrubbed).into_owned())
}

/// Marker written next to a log file once it has been scrubbed.
fn log_file_scrub_marker(path: &Path) -> PathBuf {
    let name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    path.with_file_name(format!(".{name}.scrub-v1"))
}

/// Mask secret values in `path` once, in place, with `*` of the same length.
///
/// Lengths never change, so lines appended concurrently (by this process or
/// another one) are kept, and only bytes that existed when the scrub started
/// are rewritten. A missing file counts as scrubbed. Returns the number of
/// values masked, or `None` when the marker shows the file was done before.
pub fn scrub_log_file_once(path: &Path) -> io::Result<Option<usize>> {
    let marker = log_file_scrub_marker(path);
    if marker.exists() {
        return Ok(None);
    }
    let masked = match scrub_log_file_in_place(path) {
        Ok(masked) => masked,
        Err(err) if err.kind() == io::ErrorKind::NotFound => 0,
        Err(err) => return Err(err),
    };
    File::create(marker)?;
    Ok(Some(masked))
}

fn scrub_log_file_in_place(path: &Path) -> io::Result<usize> {
    let file = File::open(path)?;
    let scrub_len = file.metadata()?.len();
    let mut reader = BufReader::new(file).take(scrub_len);
    let mut writer = OpenOptions::new().write(true).open(path)?;
    let mut offset = 0_u64;
    let mut masked = 0;
    let mut line = Vec::new();
    loop {
        line.clear();
        let read = reader
            .by_ref()
            .take(MAX_LINE_BYTES)
            .read_until(b'\n', &mut line)?;
        if read == 0 {
            break;
        }
        let spans = secret_spans(&line);
        if !spans.is_empty() {
            // Another process truncated or replaced the file: writing past
            // its new end would bring old (masked) bytes back.
            if writer.metadata()?.len() < offset + read as u64 {
                break;
            }
            for span in &spans {
                line[span.clone()].fill(b'*');
            }
            writer.seek(SeekFrom::Start(offset))?;
            writer.write_all(&line)?;
            masked += spans.len();
        }
        offset += read as u64;
    }
    writer.sync_all()?;
    Ok(masked)
}

#[cfg(test)]
#[path = "log_scrub_tests.rs"]
mod tests;
