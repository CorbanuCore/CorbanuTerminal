use std::io;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::Once;

use pretty_assertions::assert_eq;
use tracing::Level;
use tracing::Subscriber;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::Layer;
use tracing_subscriber::filter::Targets;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::layer::SubscriberExt;

use super::*;

const SENTINEL: &str = "fake-guard-sentinel-380-5c1e9a";

#[derive(Clone, Default)]
struct Buffer(Arc<Mutex<Vec<u8>>>);

impl Buffer {
    fn text(&self) -> String {
        String::from_utf8(self.0.lock().unwrap().clone()).unwrap()
    }
}

impl io::Write for Buffer {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<'a> MakeWriter<'a> for Buffer {
    type Writer = Buffer;

    fn make_writer(&'a self) -> Self::Writer {
        self.clone()
    }
}

/// Forwards `log` records to the current `tracing` dispatcher, as the
/// binaries' `try_init` does.
fn bridge_log_records() {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        tracing_log::LogTracer::init().unwrap();
    });
}

/// A trace-level file sink as the user's `RUST_LOG` asks for it, plus a
/// capture-everything sink like the logs database and `/feedback`.
fn sinks(rust_log: &str) -> (impl Subscriber + Send + Sync, Buffer, Buffer) {
    let file = Buffer::default();
    let feedback = Buffer::default();
    let subscriber = tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(file.clone())
                .with_ansi(false)
                .with_filter(EnvFilter::new(rust_log)),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(feedback.clone())
                .with_ansi(false)
                .with_filter(Targets::new().with_default(Level::TRACE)),
        );
    (subscriber, file, feedback)
}

fn emit_tungstenite_handshake() {
    // tungstenite's `trace!("Request: {:?}", ...)` with the upgrade request.
    log::trace!(
        target: "tungstenite::handshake::client",
        "Request: {:?}",
        format!("GET /v1/responses HTTP/1.1\r\nauthorization: Bearer {SENTINEL}\r\n\r\n")
    );
    log::debug!(target: "tungstenite::handshake::client", "Client handshake done.");
}

#[test]
fn bridged_tungstenite_handshake_trace_is_dropped_under_any_rust_log() {
    bridge_log_records();
    for rust_log in [
        "trace",
        "tungstenite=trace",
        "tungstenite::handshake::client=trace",
        "trace,tungstenite=trace,tokio_tungstenite=trace",
    ] {
        let (subscriber, file, feedback) = sinks(rust_log);
        tracing::subscriber::with_default(guard(subscriber), emit_tungstenite_handshake);
        for (sink, text) in [("file", file.text()), ("feedback", feedback.text())] {
            assert!(!text.contains(SENTINEL), "{rust_log}: leaked into {sink}");
            assert!(
                text.contains("Client handshake done."),
                "{rust_log}: {sink} should keep DEBUG: {text}"
            );
        }
    }
}

#[test]
fn unguarded_sinks_record_the_handshake() {
    // Control: the test bridge really delivers the record without the guard.
    bridge_log_records();
    let (subscriber, file, feedback) = sinks("trace");
    tracing::subscriber::with_default(subscriber, emit_tungstenite_handshake);
    assert!(file.text().contains(SENTINEL));
    assert!(feedback.text().contains(SENTINEL));
}

#[test]
fn tracing_events_from_capped_targets_are_dropped() {
    let (subscriber, file, feedback) = sinks("trace");
    tracing::subscriber::with_default(guard(subscriber), || {
        tracing::trace!(target: "h2::codec::framed_write", "h2 trace {SENTINEL}");
        tracing::debug!(target: "h2::codec::framed_write", "h2 debug kept");
        tracing::trace!(target: "codex_core::client", "codex trace kept");
    });
    for text in [file.text(), feedback.text()] {
        assert!(!text.contains(SENTINEL), "{text}");
        assert!(text.contains("h2 debug kept"), "{text}");
        assert!(text.contains("codex trace kept"), "{text}");
    }
}

#[test]
fn caps_match_crates_and_their_modules_only() {
    let capped = |target: &str, level: Level| {
        TARGET_CAPS
            .iter()
            .any(|(pattern, cap)| level > *cap && target_matches(target, pattern))
    };
    let cases = [
        ("tungstenite::handshake::client", Level::TRACE, true),
        ("tungstenite::handshake::client", Level::DEBUG, false),
        ("tokio_tungstenite", Level::TRACE, true),
        ("hyper::proto::h1::role", Level::TRACE, true),
        ("hyper_util::client::legacy::pool", Level::TRACE, true),
        ("h2", Level::TRACE, true),
        ("h2o", Level::TRACE, false),
        ("reqwest::connect::verbose", Level::TRACE, true),
        ("rustls::client", Level::TRACE, true),
        ("ureq_proto::util", Level::TRACE, true),
        ("rama_http_core::proto::h1", Level::TRACE, true),
        ("opentelemetry-otlp", Level::DEBUG, true),
        ("opentelemetry-otlp", Level::INFO, false),
        ("rmcp::transport::auth", Level::DEBUG, true),
        ("rmcp::service", Level::DEBUG, false),
        (
            "codex_api::endpoint::responses_websocket",
            Level::TRACE,
            false,
        ),
    ];
    let actual: Vec<_> = cases
        .iter()
        .map(|(target, level, _)| (*target, *level, capped(target, *level)))
        .collect();
    assert_eq!(actual, cases.to_vec());
}

#[test]
fn max_level_hint_is_the_inner_subscribers() {
    let subscriber = tracing_subscriber::registry().with(
        tracing_subscriber::fmt::layer()
            .with_writer(Buffer::default())
            .with_filter(LevelFilter::WARN),
    );
    let expected = subscriber.max_level_hint();
    assert_eq!(guard(subscriber).max_level_hint(), expected);
    assert_eq!(expected, Some(LevelFilter::WARN));
}
