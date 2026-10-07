//! The popup asking a person about one contained Claude pane's tool request
//! (#218).
//!
//! A stray key must never allow a tool:
//! - it opens on Deny, and Allow is chosen only with ←/→ or Tab, then Enter;
//! - for [`INPUT_GUARD`] after it first shows, every key but Esc and Ctrl-C
//!   (both deny) is ignored, so an Enter meant for the composer, a held Enter
//!   or a double Enter on the next stacked request does nothing;
//! - repeated (held) keys never confirm.
//!
//! It shows everything the request would do: every input field, wrapped and
//! scrollable, with hidden characters escaped. A request too long to show in
//! full can only be denied. When the turn stops waiting (interrupt, timeout,
//! cancel request, end of turn) the popup is removed.

use std::cell::Cell;
use std::time::Duration;
use std::time::Instant;

use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyEventKind;
use crossterm::event::KeyModifiers;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Widget;
use ratatui::widgets::Wrap;

use super::BottomPaneView;
use super::CancellationEvent;
use super::ViewCompletion;
use super::selection_popup_common::menu_surface_padding_height;
use super::selection_popup_common::render_menu_surface;
use crate::claude_panes::approval::ClaudeApprovalRequest;
use crate::claude_panes::approval::escape_plain;
use crate::render::renderable::Renderable;

/// Keys other than Esc/Ctrl-C are ignored this long after the popup shows.
pub(crate) const INPUT_GUARD: Duration = Duration::from_millis(600);
/// Most rows of request details shown at once; the rest scroll.
const MAX_DETAIL_ROWS: u16 = 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Choice {
    Deny,
    Allow,
}

pub(crate) struct ClaudeApprovalView {
    request: ClaudeApprovalRequest,
    choice: Choice,
    scroll: u16,
    completion: Option<ViewCompletion>,
    /// When the popup was first drawn: the input guard starts then, so a
    /// request stacked under another is guarded when it comes up.
    shown_at: Cell<Option<Instant>>,
    /// Detail rows that fit at the last draw, for paging, and how far the
    /// details can scroll.
    detail_rows: Cell<u16>,
    max_scroll: Cell<u16>,
    /// The hint was redrawn after the guard ended.
    guard_over_drawn: bool,
}

impl ClaudeApprovalView {
    pub(crate) fn new(request: ClaudeApprovalRequest) -> Self {
        Self {
            request,
            choice: Choice::Deny,
            scroll: 0,
            completion: None,
            shown_at: Cell::new(None),
            detail_rows: Cell::new(MAX_DETAIL_ROWS),
            max_scroll: Cell::new(0),
            guard_over_drawn: false,
        }
    }

    /// A request too long to show in full can only be denied.
    fn can_allow(&self) -> bool {
        self.request.details.hidden_chars == 0
    }

    fn guarded(&self, now: Instant) -> bool {
        self.shown_at
            .get()
            .is_none_or(|shown_at| now.saturating_duration_since(shown_at) < INPUT_GUARD)
    }

    fn finish(&mut self, allow: bool) {
        self.request.responder.respond(allow && self.can_allow());
        self.completion = Some(if allow {
            ViewCompletion::Accepted
        } else {
            ViewCompletion::Cancelled
        });
    }

    fn handle_key_at(&mut self, key: KeyEvent, now: Instant) {
        if self.completion.is_some() || key.kind == KeyEventKind::Release {
            return;
        }
        if key.code == KeyCode::Esc
            || (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL))
        {
            self.finish(/*allow*/ false);
            return;
        }
        if self.request.responder.is_settled() || self.guarded(now) {
            return;
        }
        let page = self.detail_rows.get().max(1);
        match key.code {
            KeyCode::Left | KeyCode::Right | KeyCode::Tab | KeyCode::BackTab
                if self.can_allow() =>
            {
                self.choice = match self.choice {
                    Choice::Deny => Choice::Allow,
                    Choice::Allow => Choice::Deny,
                };
            }
            KeyCode::Up => self.scroll = self.scroll.saturating_sub(1),
            KeyCode::Down => self.scroll = self.scroll.saturating_add(1),
            KeyCode::PageUp => self.scroll = self.scroll.saturating_sub(page),
            KeyCode::PageDown => self.scroll = self.scroll.saturating_add(page),
            KeyCode::Home => self.scroll = 0,
            KeyCode::End => self.scroll = u16::MAX,
            KeyCode::Enter if key.kind == KeyEventKind::Press => {
                self.finish(self.choice == Choice::Allow);
            }
            _ => {}
        }
        self.scroll = self.scroll.min(self.max_scroll.get());
    }

    fn header_lines(&self) -> Vec<Line<'static>> {
        let request = &self.request;
        let mut lines = vec![
            Line::from(vec![
                "Claude pane ".bold(),
                request.short_pane_id().to_string().cyan().bold(),
                format!(" ({})", escape_plain(&request.pane_title)).bold(),
                " wants to use ".bold(),
                escape_plain(&request.tool_name).magenta().bold(),
            ]),
            Line::from(format!(
                "folder {}",
                escape_plain(&request.cwd.to_string_lossy())
            ))
            .dim(),
        ];
        if let Some(tool_use_id) = request.tool_use_id.as_deref() {
            lines.push(Line::from(format!("tool use {}", escape_plain(tool_use_id))).dim());
        }
        lines
    }

    fn details(&self) -> Paragraph<'static> {
        Paragraph::new(self.request.details.lines.clone()).wrap(Wrap { trim: false })
    }

    fn footer_lines(&self, total_rows: u16, shown_rows: u16, scroll: u16) -> Vec<Line<'static>> {
        let mut lines = Vec::new();
        if total_rows > shown_rows {
            lines.push(
                Line::from(format!(
                    "lines {}-{} of {} · ↑/↓ PgUp/PgDn scroll",
                    scroll + 1,
                    scroll + shown_rows,
                    total_rows
                ))
                .dim(),
            );
        }
        let hidden = self.request.details.hidden_chars;
        if hidden > 0 {
            lines.push(
                Line::from(format!(
                    "{hidden} more characters are not shown, so this request can only be denied"
                ))
                .red(),
            );
        }
        lines.push(Line::from(""));
        let option = |label: &'static str, choice: Choice| -> Span<'static> {
            if self.choice == choice {
                format!("› {label} ").reversed().bold()
            } else {
                format!("  {label} ").into()
            }
        };
        let mut options = vec![option("Deny", Choice::Deny)];
        if self.can_allow() {
            options.extend(["   ".into(), option("Allow once", Choice::Allow)]);
        }
        lines.push(Line::from(options));
        let hint = if self.request.responder.is_settled() {
            "This request is no longer waiting · Esc closes"
        } else if self.guarded(Instant::now()) {
            "Esc denies · keys work in a moment"
        } else if self.can_allow() {
            "←/→ choose · Enter confirms · Esc denies"
        } else {
            "Enter or Esc denies"
        };
        lines.push(Line::from(hint).dim());
        lines
    }

    /// `(header, detail rows shown, detail rows in total)` at `width`.
    fn layout(&self, width: u16) -> (Vec<Line<'static>>, u16, u16) {
        let header = self.header_lines();
        let total = u16::try_from(self.details().line_count(width)).unwrap_or(u16::MAX);
        (header, total.min(MAX_DETAIL_ROWS), total)
    }
}

impl BottomPaneView for ClaudeApprovalView {
    fn handle_key_event(&mut self, key: KeyEvent) {
        self.handle_key_at(key, Instant::now());
    }

    fn on_ctrl_c(&mut self) -> CancellationEvent {
        self.finish(/*allow*/ false);
        CancellationEvent::Handled
    }

    fn prefer_esc_to_handle_key_event(&self) -> bool {
        true
    }

    fn is_complete(&self) -> bool {
        self.completion.is_some()
    }

    fn completion(&self) -> Option<ViewCompletion> {
        self.completion
    }

    fn is_settled_elsewhere(&self) -> bool {
        self.completion.is_none() && self.request.responder.is_settled()
    }

    fn handle_paste(&mut self, _pasted: String) -> bool {
        // Pasted text never answers a request.
        false
    }

    fn pre_draw_tick(&mut self, now: Instant) -> bool {
        // Redraw once when the guard ends, to update the hint.
        if self.guard_over_drawn || self.guarded(now) {
            return false;
        }
        self.guard_over_drawn = true;
        true
    }

    fn next_frame_delay(&self) -> Option<Duration> {
        let shown_at = self.shown_at.get()?;
        INPUT_GUARD
            .checked_sub(Instant::now().saturating_duration_since(shown_at))
            .filter(|delay| !delay.is_zero())
    }

    fn terminal_title_requires_action(&self) -> bool {
        true
    }
}

impl Renderable for ClaudeApprovalView {
    fn desired_height(&self, width: u16) -> u16 {
        let inner = width.saturating_sub(4).max(1);
        let (header, shown, total) = self.layout(inner);
        let header_rows = Paragraph::new(header)
            .wrap(Wrap { trim: false })
            .line_count(inner);
        let footer = self.footer_lines(total, shown, 0);
        let footer_rows = Paragraph::new(footer)
            .wrap(Wrap { trim: false })
            .line_count(inner);
        u16::try_from(header_rows + footer_rows + 1)
            .unwrap_or(u16::MAX)
            .saturating_add(shown)
            .saturating_add(menu_surface_padding_height())
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        if self.shown_at.get().is_none() {
            self.shown_at.set(Some(Instant::now()));
        }
        let inner = render_menu_surface(area, buf);
        if inner.is_empty() {
            return;
        }
        let (header, shown, total) = self.layout(inner.width);
        let header = Paragraph::new(header).wrap(Wrap { trim: false });
        let header_rows = u16::try_from(header.line_count(inner.width)).unwrap_or(u16::MAX);
        let footer_rows = Paragraph::new(self.footer_lines(total, shown, 0))
            .wrap(Wrap { trim: false })
            .line_count(inner.width);
        let footer_rows = u16::try_from(footer_rows).unwrap_or(u16::MAX);
        // When space is short, the options and hint stay; details shrink.
        let detail_rows = inner
            .height
            .saturating_sub(header_rows + footer_rows + 1)
            .min(shown);
        self.detail_rows.set(detail_rows);
        self.max_scroll.set(total.saturating_sub(detail_rows));
        let scroll = self.scroll.min(total.saturating_sub(detail_rows));
        let footer = Paragraph::new(self.footer_lines(total, detail_rows, scroll))
            .wrap(Wrap { trim: false });
        let mut y = inner.y;
        let mut place = |rows: u16| {
            let rows = rows.min(inner.bottom().saturating_sub(y));
            let rect = Rect::new(inner.x, y, inner.width, rows);
            y += rows;
            rect
        };
        header.render(place(header_rows), buf);
        place(1);
        self.details()
            .scroll((scroll, 0))
            .render(place(detail_rows), buf);
        footer.render(place(footer_rows), buf);
    }
}

#[cfg(test)]
#[path = "claude_approval_view_tests.rs"]
mod tests;
