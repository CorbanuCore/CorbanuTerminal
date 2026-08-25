use anyhow::Context;
use anyhow::Result;

pub(super) const MAX_CONTROL_LINE_BYTES: usize = 256 * 1024;
pub(super) const MAX_CONTROL_EVENTS: usize = 1_024;
pub(super) const MAX_CONTROL_BACKLOG_BYTES: usize = 4 * 1024 * 1024;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CommandStatus {
    Success,
    Error,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CommandResult {
    pub(super) time: u64,
    pub(super) number: u64,
    pub(super) flags: u64,
    pub(super) status: CommandStatus,
    pub(super) output: Vec<Vec<u8>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ControlEvent {
    Output {
        pane_id: String,
        data: Vec<u8>,
    },
    ExtendedOutput {
        pane_id: String,
        age_millis: u64,
        future: Vec<String>,
        data: Vec<u8>,
    },
    LayoutChange {
        window_id: String,
        layout: String,
        visible_layout: String,
        flags: String,
    },
    WindowPaneChanged {
        window_id: String,
        pane_id: String,
    },
    PaneModeChanged {
        pane_id: String,
    },
    Pause {
        pane_id: String,
    },
    Continue {
        pane_id: String,
    },
    Exit {
        reason: Vec<u8>,
    },
    Unknown {
        name: String,
        arguments: Vec<u8>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum ControlItem {
    Command(CommandResult),
    Event(ControlEvent),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct ParsedItem {
    pub(super) value: ControlItem,
    pub(super) retained_bytes: usize,
}

#[derive(Debug, Default)]
pub(super) struct ControlParser {
    pending: Option<PendingCommand>,
}

#[derive(Debug)]
struct PendingCommand {
    time: u64,
    number: u64,
    flags: u64,
    output: Vec<Vec<u8>>,
    retained_bytes: usize,
}

impl ControlParser {
    pub(super) fn parse_line(&mut self, line: &[u8]) -> Result<Option<ParsedItem>> {
        anyhow::ensure!(
            line.len() <= MAX_CONTROL_LINE_BYTES,
            "tmux control line exceeded {MAX_CONTROL_LINE_BYTES} bytes"
        );

        if self.pending.is_some() {
            return self.parse_command_line(line);
        }
        if line.starts_with(b"%begin ") {
            let (time, number, flags) = parse_command_marker(line, b"%begin ")?;
            self.pending = Some(PendingCommand {
                time,
                number,
                flags,
                output: Vec::new(),
                retained_bytes: line.len(),
            });
            return Ok(None);
        }
        anyhow::ensure!(
            !line.starts_with(b"%end ") && !line.starts_with(b"%error "),
            "tmux command boundary had no matching %begin marker"
        );
        anyhow::ensure!(
            line.starts_with(b"%"),
            "unexpected tmux control output outside a command block"
        );
        let event = parse_notification(line)?;
        Ok(Some(ParsedItem {
            value: ControlItem::Event(event),
            retained_bytes: line.len(),
        }))
    }

    fn parse_command_line(&mut self, line: &[u8]) -> Result<Option<ParsedItem>> {
        anyhow::ensure!(
            !line.starts_with(b"%begin "),
            "tmux command block contained a nested %begin marker"
        );
        let status = if line.starts_with(b"%end ") {
            Some(CommandStatus::Success)
        } else if line.starts_with(b"%error ") {
            Some(CommandStatus::Error)
        } else {
            None
        };

        let Some(status) = status else {
            let pending = self.pending.as_mut().context("missing pending command")?;
            pending.retained_bytes = pending
                .retained_bytes
                .checked_add(line.len())
                .context("tmux command output size overflow")?;
            anyhow::ensure!(
                pending.retained_bytes <= MAX_CONTROL_BACKLOG_BYTES,
                "tmux command output exceeded {MAX_CONTROL_BACKLOG_BYTES} bytes"
            );
            anyhow::ensure!(
                pending.output.len() < MAX_CONTROL_EVENTS,
                "tmux command output exceeded {MAX_CONTROL_EVENTS} lines"
            );
            pending.output.push(line.to_vec());
            return Ok(None);
        };

        let prefix = match status {
            CommandStatus::Success => b"%end ".as_slice(),
            CommandStatus::Error => b"%error ".as_slice(),
        };
        let marker = parse_command_marker(line, prefix)?;
        let pending = self.pending.take().context("missing pending command")?;
        anyhow::ensure!(
            marker == (pending.time, pending.number, pending.flags),
            "tmux command boundary did not match its %begin marker"
        );
        let retained_bytes = pending
            .retained_bytes
            .checked_add(line.len())
            .context("tmux command output size overflow")?;
        anyhow::ensure!(
            retained_bytes <= MAX_CONTROL_BACKLOG_BYTES,
            "tmux command output exceeded {MAX_CONTROL_BACKLOG_BYTES} bytes"
        );
        Ok(Some(ParsedItem {
            retained_bytes,
            value: ControlItem::Command(CommandResult {
                time: pending.time,
                number: pending.number,
                flags: pending.flags,
                status,
                output: pending.output,
            }),
        }))
    }

    pub(super) fn finish(self) -> Result<()> {
        anyhow::ensure!(
            self.pending.is_none(),
            "tmux control stream ended inside a command block"
        );
        Ok(())
    }
}

fn parse_command_marker(line: &[u8], prefix: &[u8]) -> Result<(u64, u64, u64)> {
    let marker = std::str::from_utf8(
        line.strip_prefix(prefix)
            .context("invalid tmux command marker")?,
    )
    .context("tmux command marker was not UTF-8")?;
    let mut fields = marker.split_ascii_whitespace();
    let time = parse_number(fields.next(), "time")?;
    let number = parse_number(fields.next(), "number")?;
    let flags = parse_number(fields.next(), "flags")?;
    anyhow::ensure!(
        fields.next().is_none(),
        "tmux command marker had extra fields"
    );
    Ok((time, number, flags))
}

fn parse_number(value: Option<&str>, name: &str) -> Result<u64> {
    value
        .with_context(|| format!("tmux control message omitted {name}"))?
        .parse::<u64>()
        .with_context(|| format!("tmux control message had invalid {name}"))
}

fn parse_notification(line: &[u8]) -> Result<ControlEvent> {
    let (name, arguments) = split_once(line, /*separator*/ b' ');
    let name = std::str::from_utf8(name).context("tmux notification name was not UTF-8")?;
    match name {
        "%output" => parse_output(arguments),
        "%extended-output" => parse_extended_output(arguments),
        "%layout-change" => parse_layout_change(arguments),
        "%window-pane-changed" => {
            let fields = ascii_fields(arguments, /*expected*/ 2, name)?;
            Ok(ControlEvent::WindowPaneChanged {
                window_id: fields[0].clone(),
                pane_id: fields[1].clone(),
            })
        }
        "%pane-mode-changed" => Ok(ControlEvent::PaneModeChanged {
            pane_id: one_ascii_field(arguments, name)?,
        }),
        "%pause" => Ok(ControlEvent::Pause {
            pane_id: one_ascii_field(arguments, name)?,
        }),
        "%continue" => Ok(ControlEvent::Continue {
            pane_id: one_ascii_field(arguments, name)?,
        }),
        "%exit" => Ok(ControlEvent::Exit {
            reason: arguments.to_vec(),
        }),
        _ => Ok(ControlEvent::Unknown {
            name: name.to_string(),
            arguments: arguments.to_vec(),
        }),
    }
}

fn parse_output(arguments: &[u8]) -> Result<ControlEvent> {
    let (pane_id, data) = split_required(arguments, /*separator*/ b' ', "%output")?;
    Ok(ControlEvent::Output {
        pane_id: ascii(pane_id, "%output pane id")?,
        data: decode_escaped(data)?,
    })
}

fn parse_extended_output(arguments: &[u8]) -> Result<ControlEvent> {
    let separator = arguments
        .windows(3)
        .position(|window| window == b" : ")
        .context("%extended-output omitted its value separator")?;
    let fields = ascii(&arguments[..separator], "%extended-output fields")?
        .split_ascii_whitespace()
        .map(str::to_string)
        .collect::<Vec<_>>();
    anyhow::ensure!(
        fields.len() >= 2,
        "%extended-output omitted required fields"
    );
    Ok(ControlEvent::ExtendedOutput {
        pane_id: fields[0].clone(),
        age_millis: fields[1]
            .parse::<u64>()
            .context("%extended-output had invalid age")?,
        future: fields[2..].to_vec(),
        data: decode_escaped(&arguments[separator + 3..])?,
    })
}

fn parse_layout_change(arguments: &[u8]) -> Result<ControlEvent> {
    let fields = ascii_fields(arguments, /*expected*/ 4, "%layout-change")?;
    Ok(ControlEvent::LayoutChange {
        window_id: fields[0].clone(),
        layout: fields[1].clone(),
        visible_layout: fields[2].clone(),
        flags: fields[3].clone(),
    })
}

fn decode_escaped(value: &[u8]) -> Result<Vec<u8>> {
    let mut decoded = Vec::with_capacity(value.len());
    let mut index = 0;
    while index < value.len() {
        if value[index] != b'\\' {
            decoded.push(value[index]);
            index += 1;
            continue;
        }
        anyhow::ensure!(index + 3 < value.len(), "truncated tmux octal escape");
        let digits = &value[index + 1..index + 4];
        anyhow::ensure!(
            digits.iter().all(|byte| matches!(byte, b'0'..=b'7')),
            "invalid tmux octal escape"
        );
        decoded.push((digits[0] - b'0') * 64 + (digits[1] - b'0') * 8 + digits[2] - b'0');
        index += 4;
    }
    Ok(decoded)
}

fn ascii_fields(arguments: &[u8], expected: usize, name: &str) -> Result<Vec<String>> {
    let fields = ascii(arguments, name)?
        .split_ascii_whitespace()
        .map(str::to_string)
        .collect::<Vec<_>>();
    anyhow::ensure!(fields.len() == expected, "{name} had invalid field count");
    Ok(fields)
}

fn one_ascii_field(arguments: &[u8], name: &str) -> Result<String> {
    Ok(ascii_fields(arguments, /*expected*/ 1, name)?.remove(0))
}

fn ascii(value: &[u8], description: &str) -> Result<String> {
    std::str::from_utf8(value)
        .with_context(|| format!("{description} was not UTF-8"))
        .map(str::to_string)
}

fn split_required<'a>(value: &'a [u8], separator: u8, name: &str) -> Result<(&'a [u8], &'a [u8])> {
    let position = value
        .iter()
        .position(|byte| *byte == separator)
        .with_context(|| format!("{name} omitted required fields"))?;
    Ok((&value[..position], &value[position + 1..]))
}

fn split_once(value: &[u8], separator: u8) -> (&[u8], &[u8]) {
    value
        .iter()
        .position(|byte| *byte == separator)
        .map_or((value, &[]), |position| {
            (&value[..position], &value[position + 1..])
        })
}

#[cfg(test)]
#[path = "tmux_control_protocol_tests.rs"]
mod tests;
