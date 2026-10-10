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

/// Headers and fields whose value is a credential.
const CREDENTIAL_NAMES: &str = "proxy-authorization|authorization|x-api-key|api-key|x-goog-api-key|x-amz-security-token|x-amz-sso_bearer_token|ocp-apim-subscription-key|x-auth-token|private-token|x-access-token|access_token|refresh_token|id_token";
/// Authorization schemes kept in front of a redacted value.
const SCHEMES: [&[u8]; 5] = [b"bearer", b"basic", b"token", b"digest", b"negotiate"];
/// Start of a name: a word boundary or an escaped line break.
const NAME_START: &str = r"(?:\b|\\[rnt])";
/// A name/value separator after the name's optional closing quote (quotes
/// escaped any number of times).
const SEPARATOR: &str = r#"\\*["']?[ \t]*(?::|=>|=)[ \t]*"#;

/// The unquoted-value pattern. Its value may be a type name such as `Some`
/// or `HeaderValue`, whose quoted contents [`WRAPPED`] redacts instead.
const UNQUOTED: usize = 0;
const WRAPPED: usize = 1;

/// Each pattern's group `v` is the value to hide.
static PATTERNS: LazyLock<Vec<String>> = LazyLock::new(|| {
    let names = CREDENTIAL_NAMES;
    let token = r#"[^\s"'\\,;&(){}\[\]]"#;
    vec![
        // UNQUOTED: `authorization: Bearer x`, `api-key=x`,
        // `Authorization: Token token="x"`: an optional scheme and one
        // token, or a whole SigV4 authorization.
        format!(
            r#"(?i-u){NAME_START}(?:{names}){SEPARATOR}(?P<v>aws4-hmac-sha256[^\r\n"'\\]*|(?:(?:bearer|basic|token|digest|negotiate)[ \t]+)?{token}(?:{token}|\\*"[^"\\\r\n]*\\*")*)"#
        ),
        // WRAPPED: `x-api-key: Some("x")`, `"authorization": HeaderValue {{ _private: H0("x") }}`.
        format!(
            r#"(?i-u){NAME_START}(?:{names}){SEPARATOR}[a-z_][a-z0-9_]*[ \t]*[({{][^"'\r\n)}}]{{0,48}}?\\*["'](?P<v>[^"'\\\r\n]+)"#
        ),
        // Quoted values, to the closing quote: `"x-api-key": "x y"`.
        format!(r#"(?i-u){NAME_START}(?:{names}){SEPARATOR}\\*["'](?P<v>[^"'\\\r\n]+)"#),
        // Tuples: `("api-key", "x")`.
        format!(
            r#"(?i-u)\\*["'](?:{names}|set-cookie|cookie)\\*["'][ \t]*,[ \t]*\\*["'](?P<v>[^"'\\\r\n]+)"#
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
        // Credential parameters in URL queries and form bodies.
        r#"(?i-u)[?&](?:api[-_]?key|key|access[-_]?token|refresh[-_]?token|id[-_]?token|token|sig|signature|code|code[-_]verifier|client[-_]secret|client[-_]assertion|password|x-amz-security-token|x-amz-signature)=(?P<v>[^&#\s"'<>\\]+)"#
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
/// an exact marker, alone or after a scheme, counts, so a marker can't hide
/// a credential next to it.
fn is_redacted(value: &[u8]) -> bool {
    let value = value.trim_ascii();
    let value = value.strip_suffix(b",").unwrap_or(value).trim_ascii();
    let words: Vec<&[u8]> = value
        .split(u8::is_ascii_whitespace)
        .filter(|word| !word.is_empty())
        .collect();
    let is_marker = |word: &[u8]| {
        [
            &b"REDACTED"[..],
            b"<redacted>",
            b"[redacted]",
            b"Sensitive",
            b"None",
        ]
        .contains(&word)
            || word.iter().all(|byte| *byte == b'*')
    };
    match words.as_slice() {
        [] => true,
        [marker] => is_marker(marker),
        [scheme, marker] => {
            SCHEMES
                .iter()
                .any(|known| scheme.eq_ignore_ascii_case(known))
                && is_marker(marker)
        }
        _ => false,
    }
}

/// When an [`UNQUOTED`] value is a `Debug` type name (`Some(`,
/// `HeaderValue {`), the byte range of its bracketed contents.
fn type_name_contents(text: &[u8], value: Range<usize>) -> Option<Range<usize>> {
    let name = &text[value.clone()];
    let rest = &text[value.end..];
    let is_name = name
        .first()
        .is_some_and(|byte| byte.is_ascii_alphabetic() || *byte == b'_')
        && name
            .iter()
            .all(|byte| byte.is_ascii_alphanumeric() || *byte == b'_');
    let open = match rest {
        [b'(', ..] => 1,
        [b' ', b'{', ..] => 2,
        _ => return None,
    };
    if !is_name {
        return None;
    }
    let start = value.end + open;
    let len = text[start..]
        .iter()
        .position(|byte| matches!(byte, b')' | b'}' | b'\r' | b'\n'))
        .unwrap_or(text.len() - start);
    Some(start..start + len)
}

fn credential_spans(text: &[u8]) -> Vec<Range<usize>> {
    let matched = REGEX_SET.matches(text);
    // Where WRAPPED matched, it redacts a type name's quoted contents.
    let wrapped_starts: Vec<usize> = if matched.matched(UNQUOTED) && matched.matched(WRAPPED) {
        REGEXES[WRAPPED]
            .find_iter(text)
            .map(|found| found.start())
            .collect()
    } else {
        Vec::new()
    };
    let mut spans = Vec::new();
    for index in matched.iter() {
        for captures in REGEXES[index].captures_iter(text) {
            let Some(value) = captures.name("v") else {
                continue;
            };
            let mut span = value.range();
            if index == UNQUOTED
                && let Some(contents) = type_name_contents(text, value.range())
            {
                // WRAPPED redacts quoted contents; otherwise hide them here.
                let wrapped = captures
                    .get(0)
                    .is_some_and(|found| wrapped_starts.contains(&found.start()));
                if wrapped {
                    continue;
                }
                span = contents;
            }
            if !is_redacted(&text[span.clone()]) {
                spans.push(span);
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
