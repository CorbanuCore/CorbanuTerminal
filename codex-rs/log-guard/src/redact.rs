use std::borrow::Cow;
use std::io;
use std::io::Write;
use std::ops::Range;
use std::sync::LazyLock;

use regex::bytes::Regex;
use regex::bytes::RegexSet;
use tracing::Metadata;
use tracing_subscriber::fmt::MakeWriter;

const REDACTED: &[u8] = b"REDACTED";

/// Each pattern's group `v` is the value to hide.
const PATTERNS: [&str; 4] = [
    // A credential header and its value, as an HTTP header line, a `Debug`
    // or JSON map entry (quotes escaped any number of times) or `NAME=value`.
    // The name may follow an escaped line break; the value ends at a line
    // break, a quote or an escape.
    r#"(?i-u)(?:\b|\\[rnt])(?:proxy-authorization|authorization|set-cookie|cookie|x-api-key|api-key|x-goog-api-key)\\*["']?[ \t]*(?::|=>|=)[ \t]*\\*["']?(?P<v>[^\s"'\\][^\r\n"'\\]*)"#,
    // Bearer tokens anywhere (API keys, ChatGPT access tokens).
    r"(?i-u)\bbearer[ \t]+(?P<v>[a-z0-9._~+/=-]{16,})",
    // OpenAI and Anthropic key formats.
    r"(?-u)\b(?P<v>sk-[A-Za-z0-9_-]{20,})",
    // JWTs (ChatGPT ID and access tokens).
    r"(?-u)\b(?P<v>eyJ[A-Za-z0-9_-]{10,}\.eyJ[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,})",
];

static REGEXES: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    PATTERNS
        .iter()
        .map(|pattern| {
            #[expect(clippy::expect_used)]
            Regex::new(pattern).expect("credential pattern compiles")
        })
        .collect()
});

static REGEX_SET: LazyLock<RegexSet> = LazyLock::new(|| {
    #[expect(clippy::expect_used)]
    RegexSet::new(PATTERNS).expect("credential patterns compile")
});

/// Values other redaction already replaced; kept so redacted forms stay
/// readable (`"set-cookie": "REDACTED"`, `Sensitive` header values).
fn is_redacted(value: &[u8]) -> bool {
    let value = value.trim_ascii();
    value.is_empty()
        || value.starts_with(REDACTED)
        || value.starts_with(b"<redacted>")
        || value.starts_with(b"[redacted]")
        || value.starts_with(b"Sensitive")
        || value.iter().all(|byte| *byte == b'*')
}

fn credential_spans(text: &[u8]) -> Vec<Range<usize>> {
    let mut spans = Vec::new();
    for index in REGEX_SET.matches(text).iter() {
        for captures in REGEXES[index].captures_iter(text) {
            if let Some(value) = captures.name("v")
                && !is_redacted(value.as_bytes())
            {
                spans.push(value.range());
            }
        }
    }
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

/// `text` with credential header values, bearer tokens and well-known key
/// formats replaced by `REDACTED`. Detection over-matches on purpose: a
/// false positive only hides part of a log line.
pub fn redact_credentials_bytes(text: &[u8]) -> Cow<'_, [u8]> {
    let spans = credential_spans(text);
    if spans.is_empty() {
        return Cow::Borrowed(text);
    }
    let mut redacted = Vec::with_capacity(text.len());
    let mut position = 0;
    for span in spans {
        redacted.extend_from_slice(&text[position..span.start]);
        redacted.extend_from_slice(REDACTED);
        position = span.end;
    }
    redacted.extend_from_slice(&text[position..]);
    Cow::Owned(redacted)
}

/// [`redact_credentials_bytes`] for text.
pub fn redact_credentials(text: &str) -> Cow<'_, str> {
    match redact_credentials_bytes(text.as_bytes()) {
        Cow::Borrowed(_) => Cow::Borrowed(text),
        // Spans start and end next to ASCII bytes, so this stays UTF-8.
        Cow::Owned(bytes) => Cow::Owned(String::from_utf8_lossy(&bytes).into_owned()),
    }
}

/// A [`MakeWriter`] whose writers pass every write through
/// [`redact_credentials_bytes`]. `tracing_subscriber::fmt` writes each
/// formatted event in one call, so values are never split.
#[derive(Clone, Copy, Debug, Default)]
pub struct RedactingMakeWriter<M> {
    inner: M,
}

impl<M> RedactingMakeWriter<M> {
    pub fn new(inner: M) -> Self {
        Self { inner }
    }
}

impl<'a, M: MakeWriter<'a>> MakeWriter<'a> for RedactingMakeWriter<M> {
    type Writer = RedactingWriter<M::Writer>;

    fn make_writer(&'a self) -> Self::Writer {
        RedactingWriter::new(self.inner.make_writer())
    }

    fn make_writer_for(&'a self, meta: &Metadata<'_>) -> Self::Writer {
        RedactingWriter::new(self.inner.make_writer_for(meta))
    }
}

/// A writer that redacts each write with [`redact_credentials_bytes`].
#[derive(Debug)]
pub struct RedactingWriter<W> {
    inner: W,
}

impl<W> RedactingWriter<W> {
    pub fn new(inner: W) -> Self {
        Self { inner }
    }
}

impl<W: Write> Write for RedactingWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match redact_credentials_bytes(buf) {
            Cow::Borrowed(_) => self.inner.write(buf),
            Cow::Owned(redacted) => {
                self.inner.write_all(&redacted)?;
                Ok(buf.len())
            }
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

#[cfg(test)]
#[path = "redact_tests.rs"]
mod tests;
