//! PF-28-S01: the TUI log file is a diagnostic sink; managed secrets, key
//! shapes and credential headers (#380) are removed before a formatted line
//! reaches it.

use codex_secret_broker::output_gate;
use codex_secret_broker::output_gate::OutputSink;
use std::borrow::Cow;
use std::io;
use std::io::Write;

pub(crate) struct GatedLogWriter<W>(pub(crate) W);

impl<W: Write> Write for GatedLogWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        // #380: credential header shapes, whatever library logged them.
        let redacted = codex_log_guard::redact_credentials_bytes(buf);
        let gated = output_gate::active()
            .and_then(|gate| gate.scrub_bytes(OutputSink::Diagnostic, &redacted));
        match (gated, redacted) {
            (Some((gated, _)), _) => self.0.write_all(&gated)?,
            (None, Cow::Owned(redacted)) => self.0.write_all(&redacted)?,
            (None, Cow::Borrowed(_)) => return self.0.write(buf),
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }
}
