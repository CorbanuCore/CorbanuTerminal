//! Incremental SSE decoding keeps incomplete UTF-8 bytes across network chunks.
use serde_json::Value;

#[derive(Default)]
pub struct StreamDecoder {
    bytes: Vec<u8>,
    done: bool,
    failed: bool,
}

impl StreamDecoder {
    pub fn push(&mut self, chunk: &[u8]) -> Result<Vec<(String, Value)>, String> {
        if self.bytes.len() + chunk.len() > 16 * 1024 * 1024 {
            return Err("Task Node stream frame exceeded 16 MiB.".to_string());
        }
        self.bytes.extend_from_slice(chunk);
        let mut events = Vec::new();
        loop {
            let lf = self
                .bytes
                .windows(2)
                .position(|bytes| bytes == b"\n\n")
                .map(|index| (index, 2));
            let crlf = self
                .bytes
                .windows(4)
                .position(|bytes| bytes == b"\r\n\r\n")
                .map(|index| (index, 4));
            let Some((index, width)) = [lf, crlf].into_iter().flatten().min() else {
                break;
            };
            let block = self.bytes.drain(..index + width).collect::<Vec<_>>();
            let block = std::str::from_utf8(&block).map_err(|_| "Task Node sent invalid UTF-8.")?;
            let mut event = "message";
            let mut data = Vec::new();
            for line in block.lines() {
                if let Some(value) = line.strip_prefix("event:") {
                    event = value.trim();
                }
                if let Some(value) = line.strip_prefix("data:") {
                    data.push(value.strip_prefix(' ').unwrap_or(value));
                }
            }
            if data.is_empty() {
                continue;
            }
            let value: Value = serde_json::from_str(&data.join("\n"))
                .map_err(|_| "Task Node sent an invalid stream event.")?;
            if !value.is_object() {
                return Err("Task Node sent an invalid stream event.".to_string());
            }
            self.done |= event == "done";
            self.failed |= event == "error";
            events.push((event.to_string(), value));
        }
        Ok(events)
    }

    pub fn finish(&self) -> Result<(), String> {
        if self.bytes.iter().any(|byte| !byte.is_ascii_whitespace()) || (!self.done && !self.failed)
        {
            return Err("Task Node connection ended before completion. Reopen the conversation to recover saved messages.".to_string());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_unicode_at_every_boundary_and_requires_completion() {
        let bytes = "event: delta\r\ndata: {\"delta\":\"Hello 🌍 中文\"}\r\n\r\nevent: done\ndata: {\"ok\":true}\n\n".as_bytes();
        for split in 0..bytes.len() {
            let mut decoder = StreamDecoder::default();
            let mut events = decoder.push(&bytes[..split]).unwrap();
            events.extend(decoder.push(&bytes[split..]).unwrap());
            assert_eq!(events[0].1["delta"], "Hello 🌍 中文");
            assert_eq!(events[1].0, "done");
            assert!(decoder.finish().is_ok());
        }
        let mut decoder = StreamDecoder::default();
        decoder
            .push(b"event: delta\ndata: {\"delta\":\"partial\"}\n\n")
            .unwrap();
        assert!(decoder.finish().is_err());
        assert!(decoder.push(b"event: done\ndata: invalid\n\n").is_err());
    }
}
