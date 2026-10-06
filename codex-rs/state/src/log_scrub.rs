//! One-time scrub of credential-shaped values that older builds wrote to the
//! logs database and `codex-tui.log` (#179, #183, #196): provider bearer
//! tokens and header values, URL query values and userinfo, response
//! `set-cookie` headers and secret-named environment values.
//!
//! Detection deliberately over-matches: a false positive only hides part of
//! a log line.

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

/// Replacement for scrubbed values in the logs database.
pub const REDACTED: &str = "REDACTED";

/// Lines longer than this are scrubbed in pieces.
const MAX_LINE_BYTES: u64 = 1024 * 1024;

const SECRET_NAME: &str =
    "(?:cookie|auth|token|key|secret|session|signature|credential|password|passphrase)";

/// Each pattern's first capture group is the value to hide.
static PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    [
        // Header maps, environment maps and JSON: `"set-cookie": "v"`.
        // Quotes may be escaped once (`\"`) when the map sits inside a string.
        format!(
            r#"(?i)\\?"[a-z0-9_.-]*{SECRET_NAME}[a-z0-9_.-]*\\?"\s*(?::|=>|=)\s*\\?"((?:[^"\\]|\\[^"])*)\\?""#
        ),
        // Rust `Debug` fields: `experimental_bearer_token: Some("v")`.
        format!(
            r#"(?i)\b[a-z0-9_]*{SECRET_NAME}[a-z0-9_]*: (?:Some\()?\\?"((?:[^"\\]|\\[^"])*)\\?""#
        ),
        // `NAME=value` for secret-named environment variables.
        r#"\b[A-Z0-9_]*(?:KEY|TOKEN|SECRET|PASSWORD|PASSPHRASE|CREDENTIAL)[A-Z0-9_]*=([^\s"'&;,]+)"#
            .to_string(),
        // `Authorization` schemes.
        r"(?i)\b(?:bearer|basic)\s+([A-Za-z0-9._~+/=-]{8,})".to_string(),
        // URL query values and userinfo.
        r#"[?&][A-Za-z0-9_.%-]+=([^&#\s"'<>)\]]+)"#.to_string(),
        r#"(?i)\b[a-z][a-z0-9+.-]*://([^/\s:@"']+:[^/\s@"']+)@"#.to_string(),
        // Well-known key formats anywhere.
        r"\b(sk-(?:ant-|proj-)?[A-Za-z0-9_-]{20,}|gh[pousr]_[A-Za-z0-9]{30,}|github_pat_[A-Za-z0-9_]{30,}|xox[abprs]-[A-Za-z0-9-]{10,}|AKIA[0-9A-Z]{16}|AIza[0-9A-Za-z_-]{35}|eyJ[A-Za-z0-9_-]{10,}\.eyJ[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,})"
            .to_string(),
    ]
    .iter()
    .map(|pattern| {
        #[expect(clippy::expect_used)]
        Regex::new(pattern).expect("log scrub pattern compiles")
    })
    .collect()
});

/// Sorted, non-overlapping byte ranges of credential-shaped values in
/// `text`. Values that are already redacted are skipped, so scrubbing is
/// idempotent.
pub fn secret_spans(text: &[u8]) -> Vec<Range<usize>> {
    let mut spans = PATTERNS
        .iter()
        .flat_map(|pattern| {
            pattern
                .captures_iter(text)
                .filter_map(|captures| captures.get(1))
                .map(|value| value.range())
        })
        .filter(|span| !is_redacted(&text[span.clone()]))
        .collect::<Vec<_>>();
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

/// `text` with every secret value replaced by [`REDACTED`], or `None` when
/// nothing needs scrubbing.
pub fn scrub_text(text: &str) -> Option<String> {
    let spans = secret_spans(text.as_bytes());
    if spans.is_empty() {
        return None;
    }
    let mut scrubbed = String::with_capacity(text.len());
    let mut position = 0;
    for span in spans {
        scrubbed.push_str(&String::from_utf8_lossy(
            &text.as_bytes()[position..span.start],
        ));
        scrubbed.push_str(REDACTED);
        position = span.end;
    }
    scrubbed.push_str(&String::from_utf8_lossy(&text.as_bytes()[position..]));
    Some(scrubbed)
}

/// Marker written next to a log file once it has been scrubbed.
pub fn log_file_scrub_marker(path: &Path) -> PathBuf {
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
