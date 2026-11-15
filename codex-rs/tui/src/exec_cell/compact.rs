//! Owned-transcript command previews with full command/output retained for local disclosure.

use super::model::CommandOutput;
use super::model::ExecCell;
use crate::exec_command::strip_bash_lc_and_escape;
use crate::history_cell::ActivityDisclosure;
use crate::history_cell::HistoryRenderMode;
use crate::history_cell::activity_preview::DETAIL_PREVIEW_LINES;
use crate::history_cell::activity_preview::clipped_line;
use crate::motion::MotionMode;
use crate::motion::ReducedMotionIndicator;
use crate::motion::activity_indicator;
use crate::render::highlight::highlight_bash_to_lines;
use crate::terminal_hyperlinks::HyperlinkLine;
use codex_ansi_escape::ansi_escape_line;
use ratatui::style::Modifier;
use ratatui::style::Stylize;
use ratatui::text::Line;

impl ExecCell {
    pub(super) fn command_disclosure(&self, width: u16) -> Option<ActivityDisclosure> {
        let [call] = self.group.calls.as_slice() else {
            return None;
        };
        // Interaction output is intentionally absent from the expanded transcript.
        if !call.is_unified_exec_interaction()
            && let Some(output) = &call.output
        {
            let retained = output.line_counts().1;
            let clipped = output
                .lines()
                .rev()
                .take(DETAIL_PREVIEW_LINES)
                .filter(|raw| ansi_escape_line(raw.as_ref()).width() + 4 > usize::from(width))
                .count();
            let hidden = retained.saturating_sub(DETAIL_PREVIEW_LINES) + clipped;
            if hidden > 0 {
                return Some(ActivityDisclosure::OutputLines(hidden));
            }
        }
        if !self
            .group
            .details
            .lines_after(/*after_calls*/ 1, width, HistoryRenderMode::Rich)
            .is_empty()
        {
            return Some(ActivityDisclosure::Generic);
        }
        let script = strip_bash_lc_and_escape(&call.command);
        if script.lines().nth(/*n*/ 1).is_some()
            || self
                .compact_command_lines(u16::MAX)
                .first()
                .is_some_and(|header| header.width() > usize::from(width))
        {
            return Some(ActivityDisclosure::Generic);
        }
        call.output
            .as_ref()
            .is_some_and(|output| {
                output.line_counts().1 > DETAIL_PREVIEW_LINES
                    || output.transcript_lines().any(|line| {
                        ansi_escape_line(line.as_ref()).width() + 4 > usize::from(width)
                    })
            })
            .then_some(ActivityDisclosure::Generic)
    }

    pub(super) fn compact_command_lines(&self, width: u16) -> Vec<HyperlinkLine> {
        let [call] = self.group.calls.as_slice() else {
            return Vec::new();
        };
        if call.is_unified_exec_interaction()
            && !self.is_active()
            && call.output.as_ref().is_some_and(|output| {
                output.exit_code() == Some(0) && output.lines().all(|line| line.trim().is_empty())
            })
        {
            return Vec::new();
        }
        // #873: a declined, refused or unconfirmed command is never shown as "Ran".
        let unfinished = call.is_declined() || call.is_refused() || call.is_unknown();
        let nonzero_exit = call
            .output
            .as_ref()
            .and_then(CommandOutput::exit_code)
            .filter(|code| *code != 0);
        let marker = if unfinished {
            "✗".red().bold()
        } else if nonzero_exit.is_some() {
            "•".bold()
        } else if self.is_active() {
            activity_indicator(
                call.start_time,
                MotionMode::from_animations_enabled(self.animations_enabled()),
                ReducedMotionIndicator::StaticBullet,
            )
            .unwrap_or_else(|| "•".dim())
        } else {
            "•".green().bold()
        };
        let title = if call.is_declined() {
            "Did not run".to_owned()
        } else if call.is_refused() {
            "Refused".to_owned()
        } else if call.is_unknown() {
            "Execution unconfirmed".to_owned()
        } else if let Some(code) = nonzero_exit {
            format!("Exited (status {code})")
        } else if self.is_active() {
            "Running".to_owned()
        } else if call.is_unified_exec_interaction() {
            "Background output".to_owned()
        } else {
            "Ran".to_owned()
        };
        let script = strip_bash_lc_and_escape(&call.command);
        // Highlight before clipping so the compact row keeps the same shell token colors.
        let mut command_lines = highlight_bash_to_lines(&script).into_iter();
        let mut header = command_lines.next().unwrap_or_default();
        header
            .spans
            .splice(0..0, [marker, " ".into(), title.bold(), " ".into()]);
        if command_lines.next().is_some() {
            header.push_span(" …");
        }
        let mut lines = vec![clipped_line(header, width)];
        if let Some(output) = &call.output {
            let tail: Vec<_> = output.lines().rev().take(DETAIL_PREVIEW_LINES).collect();
            for (index, raw) in tail.into_iter().rev().enumerate() {
                let mut line = ansi_escape_line(raw.as_ref());
                line.spans.insert(
                    /*index*/ 0,
                    if index == 0 { "  └ " } else { "    " }.dim(),
                );
                line.spans.iter_mut().for_each(|span| {
                    span.style = span.style.add_modifier(Modifier::DIM);
                });
                lines.push(clipped_line(line, width));
            }
            // #873: as in the full view, these states name themselves instead of "(no output)".
            if lines.len() == 1 && call.is_declined() {
                lines.push(clipped_line(
                    Line::from("  └ Approval declined".dim()),
                    width,
                ));
            } else if lines.len() == 1 && call.is_unknown() {
                lines.push(clipped_line(
                    Line::from("  └ No exit status received".dim()),
                    width,
                ));
            } else if lines.len() == 1 && !call.is_unified_exec_interaction() && !call.is_refused()
            {
                lines.push(clipped_line(Line::from("  └ (no output)".dim()), width));
            }
        }
        lines
    }
}
