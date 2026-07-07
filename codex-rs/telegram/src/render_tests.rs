use std::time::Duration;
use std::time::Instant;

use pretty_assertions::assert_eq;

use codex_telegram::render::StreamingText;
use codex_telegram::render::TELEGRAM_TEXT_LIMIT;
use codex_telegram::render::escape_html;
use codex_telegram::render::render_html_chunks;
use codex_telegram::render::split_raw_text;

#[test]
fn html_escape_only_escapes_telegram_html_entities() {
    assert_eq!(
        escape_html("<tag attr=\"x\">& text"),
        "&lt;tag attr=\"x\"&gt;&amp; text"
    );
}

#[test]
fn splitter_limits_raw_text_not_escaped_text() {
    let raw = "<".repeat(TELEGRAM_TEXT_LIMIT + 1);
    let chunks = render_html_chunks(&raw);

    assert_eq!(chunks.len(), 2);
    assert_eq!(chunks[0].raw.chars().count(), TELEGRAM_TEXT_LIMIT);
    assert!(chunks[0].html.len() > TELEGRAM_TEXT_LIMIT);
}

#[test]
fn splitter_prefers_newline_boundaries() {
    let raw = format!("{}\n{}", "a".repeat(100), "b".repeat(TELEGRAM_TEXT_LIMIT));
    let chunks = split_raw_text(&raw);

    assert_eq!(chunks.len(), 2);
    assert_eq!(chunks[0], format!("{}\n", "a".repeat(100)));
    assert_eq!(chunks[1], "b".repeat(TELEGRAM_TEXT_LIMIT));
}

#[test]
fn code_blocks_are_balanced_across_chunks() {
    let raw = format!("```\n{}\n```", "x".repeat(TELEGRAM_TEXT_LIMIT + 10));
    let chunks = render_html_chunks(&raw);

    assert!(chunks.len() > 1);
    for chunk in chunks {
        assert_eq!(
            chunk.html.matches("<pre><code>").count(),
            chunk.html.matches("</code></pre>").count()
        );
    }
}

#[test]
fn streaming_text_coalesces_fast_deltas() {
    let mut stream = StreamingText::new();
    let start = Instant::now();

    assert_eq!(stream.push_delta("a", start), true);
    assert_eq!(
        stream.push_delta("b", start + Duration::from_millis(10)),
        false
    );
    assert_eq!(
        stream.push_delta("c", start + Duration::from_millis(1_300)),
        true
    );
    assert_eq!(stream.raw(), "abc");
}
