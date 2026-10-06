//! PF-28-S01: the TUI log file is a diagnostic sink; managed secrets and key
//! shapes are removed before a formatted line reaches it.

use codex_secret_broker::output_gate;
use codex_secret_broker::output_gate::OutputSink;
use std::io;
use std::io::Write;

pub(crate) struct GatedLogWriter<W>(pub(crate) W);

impl<W: Write> Write for GatedLogWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match output_gate::active().and_then(|gate| gate.scrub_bytes(OutputSink::Diagnostic, buf)) {
            Some((gated, _)) => {
                self.0.write_all(&gated)?;
                Ok(buf.len())
            }
            None => self.0.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }
}
