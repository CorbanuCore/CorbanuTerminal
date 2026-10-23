//! PF-28-S02 reflected-credential gate for one proxied response.
//!
//! When the proxy (or the isolated broker) injects a credential into an
//! agent's request, an allowed origin can echo it back: in a header, an
//! error page, a JSON body or a server-sent event stream. The agent must not
//! receive it. A [`ResponseGate`] holds the values injected into one request
//! (in a private [`OutputGate`], never the process-wide one) and removes them,
//! with every encoding the output gate knows, from response headers,
//! trailers and the streamed body before any byte is returned.
//!
//! Bounds: the body is scanned as it streams; at most the longest
//! representation (plus a line break) is held back between chunks. Only
//! identity-encoded bodies can be checked: callers ask for `identity` and
//! refuse a response that arrives compressed.

use crate::output_gate::OutputGate;
use crate::output_gate::OutputSink;
use crate::output_gate::RegisterError;
use crate::output_gate::SecretClass;
use crate::output_gate::StreamScrubber;

/// The values injected into one request.
#[derive(Clone)]
pub struct ResponseGate {
    gate: OutputGate,
}

impl std::fmt::Debug for ResponseGate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResponseGate")
            .field("values", &self.gate.value_count())
            .finish()
    }
}

impl ResponseGate {
    /// A gate for `(label, value)` pairs. Fails closed: the caller must not
    /// inject a value the gate cannot hold.
    pub fn new<'a>(
        values: impl IntoIterator<Item = (&'a str, &'a str)>,
    ) -> Result<Self, RegisterError> {
        let values: Vec<(&str, SecretClass, &str)> = values
            .into_iter()
            .map(|(label, value)| (label, SecretClass::Operational, value))
            .collect();
        let gate = OutputGate::new();
        gate.register_all(&values)?;
        Ok(Self { gate })
    }

    /// Adds a value (a request can carry several injected values).
    pub fn add(&self, label: &str, value: &str) -> Result<(), RegisterError> {
        self.gate
            .register(label, SecretClass::Operational, value)
            .map(|_| ())
    }

    /// Scrubs one header name or value; `None` when unchanged.
    pub fn scrub_header(&self, bytes: &[u8]) -> Option<Vec<u8>> {
        self.gate
            .scrub_bytes(OutputSink::ProviderResponse, bytes)
            .map(|(bytes, _)| bytes)
    }

    /// A scrubber for the response body.
    pub fn body(&self) -> ResponseBodyScrubber {
        ResponseBodyScrubber {
            gate: self.gate.clone(),
            stream: StreamScrubber::for_responses(),
        }
    }

    /// Whether a body sent with these `Content-Encoding` header values
    /// (every occurrence, each possibly a list) can be checked: only
    /// `identity`.
    pub fn checks_content_encoding<'a>(values: impl IntoIterator<Item = &'a [u8]>) -> bool {
        values.into_iter().all(|value| {
            value
                .split(|byte| *byte == b',')
                .all(|coding| coding.trim_ascii().eq_ignore_ascii_case(b"identity"))
        })
    }
}

/// Streams one response body through its [`ResponseGate`].
pub struct ResponseBodyScrubber {
    gate: OutputGate,
    stream: StreamScrubber,
}

impl ResponseBodyScrubber {
    /// Returns the bytes of `chunk` (and any held-back bytes) safe to emit.
    pub fn push(&mut self, chunk: &[u8]) -> Vec<u8> {
        self.stream.push(
            &self.gate,
            OutputSink::ProviderResponse,
            chunk,
            /*utf8*/ false,
        )
    }

    /// Ends the body; returns whatever was held back, scrubbed.
    pub fn finish(&mut self) -> Vec<u8> {
        self.stream.finish(&self.gate, OutputSink::ProviderResponse)
    }

    /// Bytes currently held back.
    pub fn pending(&self) -> usize {
        self.stream.pending()
    }
}

#[cfg(test)]
#[path = "response_gate_tests.rs"]
mod tests;
