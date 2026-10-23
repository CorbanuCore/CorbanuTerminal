use std::any::TypeId;

use tracing::Dispatch;
use tracing::Event;
use tracing::Metadata;
use tracing::Subscriber;
use tracing::level_filters::LevelFilter;
use tracing::span;
use tracing::subscriber::Interest;

/// Most verbose level kept for each target (the crate and its modules).
/// A trailing `*` matches every crate with that prefix.
const TARGET_CAPS: &[(&str, LevelFilter)] = &[
    // The client handshake logs the whole upgrade request, `Authorization`
    // included, at TRACE (#380).
    ("tungstenite", LevelFilter::DEBUG),
    ("tokio_tungstenite", LevelFilter::DEBUG),
    // Hex-dumps every request byte at TRACE (sentry's transport).
    ("ureq_proto", LevelFilter::DEBUG),
    ("ureq", LevelFilter::DEBUG),
    // Dumps raw connection bytes at TRACE when `connection_verbose` is on.
    ("reqwest", LevelFilter::DEBUG),
    // HTTP/1, HTTP/2, TLS and SSE internals. Their TRACE output carries no
    // header values today; capped so an upgrade can't add one unnoticed.
    ("hyper", LevelFilter::DEBUG),
    ("hyper_util", LevelFilter::DEBUG),
    ("h2", LevelFilter::DEBUG),
    ("rustls", LevelFilter::DEBUG),
    ("tokio_rustls", LevelFilter::DEBUG),
    ("eventsource_stream", LevelFilter::DEBUG),
    // The network proxy's HTTP stack (a hyper/h2 fork) carries injected
    // credentials.
    ("rama_*", LevelFilter::DEBUG),
    // The AWS SDK logs credential failures with their whole error chain at
    // WARN and ERROR (#548): a `credential_process` helper's stderr, which
    // may echo anything, the helper's command line (TRACE), the ECS
    // container token, mis-indented config-file lines, and at TRACE the
    // canonical request with `x-amz-security-token` (#380). None of it is
    // kept; Corbanu reports AWS failures itself, with helper output withheld
    // (`AwsAuthError::detailed_message`).
    ("aws_*", LevelFilter::OFF),
    // Logs `OTEL_EXPORTER_OTLP_HEADERS` values at DEBUG.
    ("opentelemetry-otlp", LevelFilter::INFO),
    ("opentelemetry_otlp", LevelFilter::INFO),
    // Logs OAuth token exchange results at DEBUG.
    ("rmcp::transport::auth", LevelFilter::INFO),
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

/// The only sanctioned way to install a global subscriber: `clippy.toml`
/// disallows `set_global_default` and `SubscriberInitExt::{init, try_init}`
/// everywhere else, so a binary can't install one that skips [`guard`].
/// These shadow the `SubscriberInitExt` methods, so `guard(s).try_init()`
/// resolves here.
impl<S> Guarded<S>
where
    S: Subscriber + Send + Sync + 'static,
{
    /// [`SubscriberInitExt::try_init`](tracing_subscriber::util::SubscriberInitExt::try_init).
    pub fn try_init(self) -> Result<(), tracing_subscriber::util::TryInitError> {
        #[expect(clippy::disallowed_methods)]
        tracing_subscriber::util::SubscriberInitExt::try_init(self)
    }

    /// [`SubscriberInitExt::init`](tracing_subscriber::util::SubscriberInitExt::init).
    pub fn init(self) {
        #[expect(clippy::disallowed_methods)]
        tracing_subscriber::util::SubscriberInitExt::init(self);
    }
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
