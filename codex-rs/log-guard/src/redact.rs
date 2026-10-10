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

/// Headers whose value is a credential.
const CREDENTIAL_HEADERS: &str = "proxy-authorization|authorization|x-api-key|api-key|x-goog-api-key|x-amz-security-token|x-amz-sso_bearer_token|ocp-apim-subscription-key|x-auth-token|private-token|x-access-token";
/// Start of a header name: a word boundary or an escaped line break.
const NAME_START: &str = r"(?:\b|\\[rnt])";
/// A name/value separator, the name's closing quote and the value's
/// opening quote (quotes escaped any number of times).
const SEPARATOR: &str = r#"\\*["']?[ \t]*(?::|=>|=)[ \t]*"#;

/// The pattern whose value may be a type name such as `Some` or
/// `HeaderValue`; the next pattern finds the quoted value inside it.
const SCHEME_AND_TOKEN: usize = 0;

/// Each pattern's group `v` is the value to hide.
static PATTERNS: LazyLock<Vec<String>> = LazyLock::new(|| {
    let headers = CREDENTIAL_HEADERS;
    vec![
        // `authorization: Bearer x`, `"x-api-key": "x"`, `api-key=x`: an
        // optional scheme and one token, or a whole SigV4 authorization.
        format!(
            r#"(?i-u){NAME_START}(?:{headers}){SEPARATOR}\\*["']?(?P<v>aws4-hmac-sha256[^\r\n"'\\]*|(?:(?:bearer|basic|token|digest|negotiate)[ \t]+)?[^\s"'\\,;(){{}}\[\]]+)"#
        ),
        // `x-api-key: Some("x")`, `"authorization": HeaderValue {{ _private: H0("x") }}`.
        format!(
            r#"(?i-u){NAME_START}(?:{headers}){SEPARATOR}[a-z_][a-z0-9_]*[ \t]*[({{][^"'\r\n]{{0,48}}?\\*["'](?P<v>[^"'\\\r\n]+)"#
        ),
        // Tuples: `("api-key", "x")`.
        format!(
            r#"(?i-u)\\*["'](?:{headers}|set-cookie|cookie)\\*["'][ \t]*,[ \t]*\\*["'](?P<v>[^"'\\\r\n]+)"#
        ),
        // Cookies: every pair up to a line break, quote or escape.
        format!(
            r#"(?i-u){NAME_START}(?:set-cookie|cookie){SEPARATOR}\\*["']?(?P<v>[^\s"'\\][^\r\n"'\\]*)"#
        ),
        // Bearer tokens anywhere (API keys, ChatGPT access tokens).
        r"(?i-u)\bbearer[ \t]+(?P<v>[a-z0-9._~+/=-]{16,})".to_string(),
        // OpenAI and Anthropic key formats.
        r"(?-u)\b(?P<v>sk-[A-Za-z0-9_-]{20,})".to_string(),
        // JWTs (ChatGPT ID and access tokens).
        r"(?-u)\b(?P<v>eyJ[A-Za-z0-9_-]{10,}\.eyJ[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,})"
            .to_string(),
        // Credential query parameters in URLs.
        r#"(?i-u)[?&](?:api[-_]?key|key|access[-_]?token|token|sig|signature|code|client[-_]secret|password|x-amz-security-token|x-amz-signature)=(?P<v>[^&#\s"'<>\\]+)"#
            .to_string(),
    ]
});

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
    RegexSet::new(PATTERNS.iter()).expect("credential patterns compile")
});

/// Values other redaction already replaced; kept so redacted forms stay
/// readable (`"set-cookie": "REDACTED"`, `Sensitive` header values). Only
/// an exact marker (after an optional scheme) counts, so a marker can't hide
/// a credential later in the value.
fn is_redacted(value: &[u8]) -> bool {
    let value = value.trim_ascii();
    let value = value.strip_suffix(b",").unwrap_or(value).trim_ascii();
    let words: Vec<&[u8]> = value
        .split(u8::is_ascii_whitespace)
        .filter(|word| !word.is_empty())
        .collect();
    match words.as_slice() {
        [] => true,
        [marker] | [_, marker] => {
            [&b"REDACTED"[..], b"<redacted>", b"[redacted]", b"Sensitive"].contains(marker)
                || marker.iter().all(|byte| *byte == b'*')
        }
        _ => false,
    }
}

/// Whether `rest` (the text after a value) shows the value was a type name.
fn followed_by_wrapper(rest: &[u8]) -> bool {
    matches!(rest.trim_ascii_start().first(), Some(b'(' | b'{'))
}

fn credential_spans(text: &[u8]) -> Vec<Range<usize>> {
    let mut spans = Vec::new();
    for index in REGEX_SET.matches(text).iter() {
        for captures in REGEXES[index].captures_iter(text) {
            if let Some(value) = captures.name("v")
                && !is_redacted(value.as_bytes())
                && !(index == SCHEME_AND_TOKEN && followed_by_wrapper(&text[value.end()..]))
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
