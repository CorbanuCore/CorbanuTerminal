//! Keeps credential headers out of every log sink (#380).
//!
//! HTTP and websocket client libraries can log whole requests at TRACE:
//! tungstenite logs the websocket upgrade request with its `Authorization`
//! header, and ureq-proto hex-dumps every request byte. The TUI log file
//! and stderr record TRACE whenever `RUST_LOG` asks for it, and the logs
//! database and `/feedback` buffer always do, so no `RUST_LOG` filter can be
//! trusted to keep these events out.
//!
//! - [`guard`] wraps a subscriber so those libraries' targets stay capped
//!   whatever its own filters, including `RUST_LOG`, enable. Wrap every
//!   subscriber a binary installs.
//! - [`redact_credentials`] and [`RedactingMakeWriter`] are a second,
//!   pattern-based pass for sink writers: credential header values, bearer
//!   tokens, credential command-line options and environment assignments,
//!   URL passwords and well-known key formats become `REDACTED`. Command
//!   logs that are not `tracing` sinks (the Windows sandbox log) call
//!   [`redact_credentials`] directly.

mod guard;
mod redact;

pub use guard::Guarded;
pub use guard::guard;
pub use guard::is_capped;
pub use redact::RedactingMakeWriter;
pub use redact::RedactingWriter;
pub use redact::contains_credentials;
pub use redact::credential_spans;
pub use redact::redact_command;
pub use redact::redact_credentials;
pub use redact::redact_credentials_bytes;
