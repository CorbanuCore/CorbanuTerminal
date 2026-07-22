use std::time::Duration;
use std::time::Instant;

pub const TELEGRAM_TEXT_LIMIT: usize = 4096;
const STREAM_EDIT_INTERVAL: Duration = Duration::from_millis(1_300);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HtmlChunk {
    pub raw: String,
    pub html: String,
}

#[derive(Debug, Clone)]
pub struct StreamingText {
    raw: String,
    last_edit_at: Option<Instant>,
}

impl Default for StreamingText {
    fn default() -> Self {
        Self::new()
    }
}

impl StreamingText {
    pub fn new() -> Self {
        Self {
            raw: String::new(),
            last_edit_at: None,
        }
    }

    pub fn push_delta(&mut self, delta: &str, now: Instant) -> bool {
        self.raw.push_str(delta);
        if self
            .last_edit_at
            .is_some_and(|last| now.duration_since(last) < STREAM_EDIT_INTERVAL)
        {
            return false;
        }
        self.last_edit_at = Some(now);
        true
    }

    pub fn raw(&self) -> &str {
        &self.raw
    }

    pub fn preview_html(&self) -> String {
        render_html_chunks(&self.raw)
            .into_iter()
            .next()
            .map(|chunk| chunk.html)
            .unwrap_or_default()
    }
}

pub fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

pub fn split_raw_text(text: &str) -> Vec<String> {
    if text.is_empty() {
        return vec![String::new()];
    }

    let mut chunks = Vec::new();
    let mut remaining = text;
    while utf16_len(remaining) > TELEGRAM_TEXT_LIMIT {
        let split_at = preferred_split_byte(remaining, TELEGRAM_TEXT_LIMIT);
        let (head, tail) = remaining.split_at(split_at);
        chunks.push(head.to_string());
        remaining = tail;
    }
    chunks.push(remaining.to_string());
    chunks
}

pub fn render_html_chunks(text: &str) -> Vec<HtmlChunk> {
    let mut in_code = false;
    split_raw_text(text)
        .into_iter()
        .map(|raw| {
            let html = render_html_chunk(&raw, &mut in_code);
            HtmlChunk { raw, html }
        })
        .collect()
}

fn utf16_len(text: &str) -> usize {
    text.encode_utf16().count()
}

fn preferred_split_byte(text: &str, max_utf16_units: usize) -> usize {
    let mut units = 0;
    let mut hard_limit = text.len();
    for (idx, ch) in text.char_indices() {
        units += ch.len_utf16();
        if units > max_utf16_units {
            hard_limit = idx;
            break;
        }
    }
    text[..hard_limit]
        .rfind('\n')
        .filter(|idx| *idx > 0)
        .map(|idx| idx + 1)
        .unwrap_or(hard_limit)
}

fn render_html_chunk(raw: &str, in_code: &mut bool) -> String {
    let mut html = String::new();
    if *in_code {
        html.push_str("<pre><code>");
    }

    for segment in raw.split_inclusive('\n') {
        let line = segment.strip_suffix('\n').unwrap_or(segment);
        let has_newline = segment.ends_with('\n');
        if line.trim_start().starts_with("```") {
            if *in_code {
                html.push_str("</code></pre>");
                *in_code = false;
            } else {
                html.push_str("<pre><code>");
                *in_code = true;
            }
            if has_newline {
                html.push('\n');
            }
            continue;
        }
        html.push_str(&escape_html(line));
        if has_newline {
            html.push('\n');
        }
    }

    if *in_code {
        html.push_str("</code></pre>");
    }
    html
}
