use std::any::TypeId;

use tracing::Dispatch;
use tracing::Event;
use tracing::Level;
use tracing::Metadata;
use tracing::Subscriber;
use tracing::level_filters::LevelFilter;
use tracing::span;
use tracing::subscriber::Interest;

/// Most verbose level kept for each target (the crate and its modules).
/// A trailing `*` matches every crate with that prefix.
const TARGET_CAPS: &[(&str, Level)] = &[
    // The client handshake logs the whole upgrade request, `Authorization`
    // included, at TRACE (#380).
    ("tungstenite", Level::DEBUG),
    ("tokio_tungstenite", Level::DEBUG),
    // Hex-dumps every request byte at TRACE (sentry's transport).
    ("ureq_proto", Level::DEBUG),
    ("ureq", Level::DEBUG),
    // Dumps raw connection bytes at TRACE when `connection_verbose` is on.
    ("reqwest", Level::DEBUG),
    // HTTP/1, HTTP/2, TLS and SSE internals. Their TRACE output carries no
    // header values today; capped so an upgrade can't add one unnoticed.
    ("hyper", Level::DEBUG),
    ("hyper_util", Level::DEBUG),
    ("h2", Level::DEBUG),
    ("rustls", Level::DEBUG),
    ("tokio_rustls", Level::DEBUG),
    ("eventsource_stream", Level::DEBUG),
    // The network proxy's HTTP stack (a hyper/h2 fork) carries injected
    // credentials.
    ("rama_*", Level::DEBUG),
    // aws-sigv4 logs the canonical request, `x-amz-security-token` value
    // included, and aws-smithy-runtime every request, at TRACE.
    ("aws_*", Level::DEBUG),
    // Logs the ECS container authorization token at WARN when it is
    // malformed (a trailing newline is enough).
    ("aws_config::ecs", Level::ERROR),
    // Logs `OTEL_EXPORTER_OTLP_HEADERS` values at DEBUG.
    ("opentelemetry-otlp", Level::INFO),
    ("opentelemetry_otlp", Level::INFO),
    // Logs OAuth token exchange results at DEBUG.
    ("rmcp::transport::auth", Level::INFO),
];

/// Whether `metadata` is more verbose than its target's cap.
///
/// Records bridged from the `log` crate (tungstenite's) are checked with
/// their original target before they are dispatched, so they are covered.
pub fn is_capped(metadata: &Metadata<'_>) -> bool {
    let target = metadata.target();
    TARGET_CAPS
        .iter()
        .any(|(pattern, cap)| metadata.level() > cap && target_matches(target, pattern))
}

fn target_matches(target: &str, pattern: &str) -> bool {
    match pattern.strip_suffix('*') {
        Some(prefix) => target.starts_with(prefix),
        None => target
            .strip_prefix(pattern)
            .is_some_and(|rest| rest.is_empty() || rest.starts_with("::")),
    }
}

/// `subscriber` with [`is_capped`] events and spans disabled before any of
/// its layers or filters see them.
pub fn guard<S>(subscriber: S) -> Guarded<S> {
    Guarded { inner: subscriber }
}

/// A subscriber wrapped by [`guard`]. Everything else, including the
/// inner subscriber's max level hint, passes through unchanged.
#[derive(Debug)]
pub struct Guarded<S> {
    inner: S,
}

impl<S: Subscriber> Subscriber for Guarded<S> {
    fn on_register_dispatch(&self, subscriber: &Dispatch) {
        self.inner.on_register_dispatch(subscriber);
    }

    fn register_callsite(&self, metadata: &'static Metadata<'static>) -> Interest {
        if is_capped(metadata) {
            Interest::never()
        } else {
            self.inner.register_callsite(metadata)
        }
    }

    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        !is_capped(metadata) && self.inner.enabled(metadata)
    }

    fn max_level_hint(&self) -> Option<LevelFilter> {
        self.inner.max_level_hint()
    }

    fn new_span(&self, span: &span::Attributes<'_>) -> span::Id {
        self.inner.new_span(span)
    }

    fn record(&self, span: &span::Id, values: &span::Record<'_>) {
        self.inner.record(span, values);
    }

    fn record_follows_from(&self, span: &span::Id, follows: &span::Id) {
        self.inner.record_follows_from(span, follows);
    }

    fn event_enabled(&self, event: &Event<'_>) -> bool {
        self.inner.event_enabled(event)
    }

    fn event(&self, event: &Event<'_>) {
        self.inner.event(event);
    }

    fn enter(&self, span: &span::Id) {
        self.inner.enter(span);
    }

    fn exit(&self, span: &span::Id) {
        self.inner.exit(span);
    }

    fn clone_span(&self, id: &span::Id) -> span::Id {
        self.inner.clone_span(id)
    }

    fn try_close(&self, id: span::Id) -> bool {
        self.inner.try_close(id)
    }

    fn current_span(&self) -> tracing_core::span::Current {
        self.inner.current_span()
    }

    unsafe fn downcast_raw(&self, id: TypeId) -> Option<*const ()> {
        if id == TypeId::of::<Self>() {
            return Some(self as *const Self as *const ());
        }
        // SAFETY: forwards the same contract to the wrapped subscriber.
        unsafe { self.inner.downcast_raw(id) }
    }
}

#[cfg(test)]
#[path = "guard_tests.rs"]
mod tests;
