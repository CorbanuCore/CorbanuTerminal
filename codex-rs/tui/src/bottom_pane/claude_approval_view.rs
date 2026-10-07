//! The popup asking a person about contained Claude pane tool requests
//! (#218).
//!
//! A stray key must never allow a tool:
//! - each request opens on Deny, and Allow is chosen only with ←/→, then
//!   Enter;
//! - every key but Esc and Ctrl-C (both deny) is ignored until [`INPUT_GUARD`]
//!   has passed since the request first showed, since the last ignored key,
//!   and since the person last typed in the composer; so an Enter meant for
//!   the composer, continued typing, a held Enter or a double Enter on the
//!   next request does nothing;
//! - repeated (held) keys never confirm;
//! - requests queue in one popup (first in, first out) instead of covering
//!   each other, and each one starts on Deny with a fresh guard; the popup
//!   does the same when it comes back from under another view.
//!
//! It shows everything the request would do: every input field, wrapped and
//! scrollable, with hidden characters escaped. Allow works only once the last
//! line has been on screen; a request too long to show in full can only be
//! denied. When the turn stops waiting (interrupt, timeout, cancel request,
//! end of turn) the request leaves the queue.

use std::cell::Cell;
use std::collections::VecDeque;
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
use super::selection_popup_common::menu_surface_padding_height;
use super::selection_popup_common::render_menu_surface;
use crate::claude_panes::approval::ClaudeApprovalRequest;
use crate::claude_panes::approval::escape_plain;
use crate::render::renderable::Renderable;

/// Keys other than Esc/Ctrl-C are ignored this long after the request
/// shows, after the last ignored key and after the last composer keystroke.
pub(crate) const INPUT_GUARD: Duration = Duration::from_millis(600);
/// Most rows of request details shown at once; the rest scroll.
const MAX_DETAIL_ROWS: u16 = 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Choice {
    Deny,
    Allow,
}

pub(crate) struct ClaudeApprovalView {
    /// The front request is shown; the rest wait their turn.
    queue: VecDeque<ClaudeApprovalRequest>,
    choice: Choice,
    scroll: u16,
    /// Every request was answered. Reported as no completion, so closing
    /// never acts on the views below.
    done: bool,
    /// When the front request was first drawn. The guard runs from then.
    shown_at: Cell<Option<Instant>>,
    /// Keys stay ignored until then (the last ignored key, composer typing).
    quiet_until: Option<Instant>,
    /// The front request's last detail line has been on screen.
    seen_end: Cell<bool>,
    /// Detail rows that fit at the last draw, for paging, and how far the
    /// details can scroll.
    detail_rows: Cell<u16>,
    max_scroll: Cell<u16>,
    /// The hint was redrawn after the guard ended.
    guard_over_drawn: bool,
}

impl ClaudeApprovalView {
    /// `typed_at`: the person's last composer keystroke, if any.
    pub(crate) fn new(request: ClaudeApprovalRequest, typed_at: Option<Instant>) -> Self {
        Self {
            queue: VecDeque::from([request]),
            choice: Choice::Deny,
            scroll: 0,
            done: false,
            shown_at: Cell::new(None),
            quiet_until: typed_at.map(|typed_at| typed_at + INPUT_GUARD),
            seen_end: Cell::new(false),
            detail_rows: Cell::new(MAX_DETAIL_ROWS),
            max_scroll: Cell::new(0),
            guard_over_drawn: false,
        }
    }

    /// Queues another request behind the ones already here.
    pub(crate) fn enqueue(&mut self, request: ClaudeApprovalRequest) {
        self.queue.push_back(request);
    }

    fn current(&self) -> Option<&ClaudeApprovalRequest> {
        self.queue.front()
    }

    /// Start the front request afresh: on Deny, at the top, guarded.
    fn restart(&mut self) {
        self.choice = Choice::Deny;
        self.scroll = 0;
        self.shown_at.set(None);
        self.seen_end.set(false);
        self.guard_over_drawn = false;
    }

    /// Too long to show in full: only Deny.
    fn too_long(&self) -> bool {
        self.current()
            .is_some_and(|request| request.details.hidden_chars > 0)
    }

    fn can_allow(&self) -> bool {
        !self.too_long() && self.seen_end.get()
    }

    fn guarded(&self, now: Instant) -> bool {
        self.shown_at
            .get()
            .is_none_or(|shown_at| now.saturating_duration_since(shown_at) < INPUT_GUARD)
            || self.quiet_until.is_some_and(|until| now < until)
    }

    /// Answer the front request and move on to the next one.
    fn finish(&mut self, allow: bool) {
        let allow = allow && self.can_allow();
        if let Some(request) = self.queue.pop_front() {
            request.responder.respond(allow);
        }
        if self.queue.is_empty() {
            self.done = true;
        } else {
            self.restart();
        }
    }

    fn handle_key_at(&mut self, key: KeyEvent, now: Instant) {
        if self.done || key.kind == KeyEventKind::Release {
            return;
        }
        if key.code == KeyCode::Esc
            || (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL))
        {
            self.finish(/*allow*/ false);
            return;
        }
        if self
            .current()
            .is_none_or(|request| request.responder.is_settled())
        {
            return;
        }
        if self.guarded(now) {
            // Still typing, or a key held down: wait for quiet again.
            if self.shown_at.get().is_some() {
                self.quiet_until = Some(now + INPUT_GUARD);
                self.guard_over_drawn = false;
            }
            return;
        }
        let page = self.detail_rows.get().max(1);
        match key.code {
            KeyCode::Left | KeyCode::Right if self.can_allow() => {
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

    fn header_lines(&self, request: &ClaudeApprovalRequest) -> Vec<Line<'static>> {
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
        if self.queue.len() > 1 {
            lines.push(Line::from(format!("{} more requests waiting", self.queue.len() - 1)).dim());
        }
        lines
    }

    fn details(request: &ClaudeApprovalRequest) -> Paragraph<'static> {
        Paragraph::new(request.details.lines.clone()).wrap(Wrap { trim: false })
    }

    fn footer_lines(&self, total_rows: usize, shown_rows: u16, scroll: u16) -> Vec<Line<'static>> {
        let mut lines = Vec::new();
        if total_rows > usize::from(shown_rows) {
            lines.push(
                Line::from(format!(
                    "lines {}-{} of {} · ↑/↓ PgUp/PgDn scroll",
                    usize::from(scroll) + 1,
                    usize::from(scroll) + usize::from(shown_rows),
                    total_rows
                ))
                .dim(),
            );
        }
        let hidden = self
            .current()
            .map_or(0, |request| request.details.hidden_chars);
        if hidden > 0 {
            lines.push(
                Line::from(format!(
                    "{hidden} more characters are not shown, so this request can only be denied"
                ))
                .red(),
            );
        } else if !self.seen_end.get() {
            lines.push(Line::from("Scroll to the end to allow it").red());
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
        if !self.too_long() {
            options.push("   ".into());
            options.push(if self.can_allow() {
                option("Allow once", Choice::Allow)
            } else {
                "  Allow once ".dim()
            });
        }
        lines.push(Line::from(options));
        let hint = if self
            .current()
            .is_none_or(|request| request.responder.is_settled())
        {
            "This request is no longer waiting · Esc closes"
        } else if self.guarded(Instant::now()) {
            "Esc denies · keys work once you stop typing"
        } else if self.can_allow() {
            "←/→ choose · Enter confirms · Esc denies"
        } else {
            "Enter or Esc denies"
        };
        lines.push(Line::from(hint).dim());
        lines
    }

    /// `(detail rows shown, detail rows in total)` at `width`.
    fn detail_layout(request: &ClaudeApprovalRequest, width: u16) -> (u16, usize) {
        let total = Self::details(request).line_count(width);
        let shown = u16::try_from(total)
            .unwrap_or(u16::MAX)
            .min(MAX_DETAIL_ROWS);
        (shown, total)
    }

    fn rows(lines: Vec<Line<'static>>, width: u16) -> u16 {
        u16::try_from(
            Paragraph::new(lines)
                .wrap(Wrap { trim: false })
                .line_count(width),
        )
        .unwrap_or(u16::MAX)
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
        self.done
    }

    fn try_consume_claude_approval(
        &mut self,
        request: ClaudeApprovalRequest,
    ) -> Option<ClaudeApprovalRequest> {
        if self.done {
            return Some(request);
        }
        self.enqueue(request);
        None
    }

    fn remove_settled_requests(&mut self) -> bool {
        if self.done {
            return false;
        }
        let front_settled = self
            .current()
            .is_some_and(|request| request.responder.is_settled());
        self.queue.retain(|request| !request.responder.is_settled());
        if front_settled {
            self.restart();
        }
        self.queue.is_empty()
    }

    fn on_uncovered(&mut self) {
        // Back from under another view: a fresh look before any answer.
        self.restart();
    }

    fn handle_paste(&mut self, _pasted: String) -> bool {
        // Pasted text never answers a request, and counts as typing.
        if self.shown_at.get().is_some() {
            self.quiet_until = Some(Instant::now() + INPUT_GUARD);
        }
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
        let now = Instant::now();
        let shown_until = self.shown_at.get()? + INPUT_GUARD;
        let until = self
            .quiet_until
            .map_or(shown_until, |quiet| quiet.max(shown_until));
        until
            .checked_duration_since(now)
            .filter(|delay| !delay.is_zero())
    }

    fn terminal_title_requires_action(&self) -> bool {
        true
    }
}

impl Renderable for ClaudeApprovalView {
    fn desired_height(&self, width: u16) -> u16 {
        let Some(request) = self.current() else {
            return 0;
        };
        let inner = width.saturating_sub(4).max(1);
        let (shown, total) = Self::detail_layout(request, inner);
        let header_rows = Self::rows(self.header_lines(request), inner);
        let footer_rows = Self::rows(self.footer_lines(total, shown, 0), inner);
        header_rows
            .saturating_add(footer_rows)
            .saturating_add(1)
            .saturating_add(shown)
            .saturating_add(menu_surface_padding_height())
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        let Some(request) = self.current() else {
            return;
        };
        if self.shown_at.get().is_none() {
            self.shown_at.set(Some(Instant::now()));
        }
        let inner = render_menu_surface(area, buf);
        if inner.is_empty() {
            return;
        }
        let (shown, total) = Self::detail_layout(request, inner.width);
        let header = Paragraph::new(self.header_lines(request)).wrap(Wrap { trim: false });
        let header_rows = u16::try_from(header.line_count(inner.width)).unwrap_or(u16::MAX);
        let footer_rows = Self::rows(self.footer_lines(total, shown, 0), inner.width);
        // When space is short, the options and hint stay; details shrink.
        let detail_rows = inner
            .height
            .saturating_sub(header_rows + footer_rows + 1)
            .min(shown);
        let total_rows = u16::try_from(total).unwrap_or(u16::MAX);
        let max_scroll = total_rows.saturating_sub(detail_rows);
        let scroll = self.scroll.min(max_scroll);
        self.detail_rows.set(detail_rows);
        self.max_scroll.set(max_scroll);
        // Allow needs the last line seen; details past what a scroll offset
        // can reach can never be.
        if detail_rows > 0
            && u16::try_from(total).is_ok()
            && usize::from(scroll) + usize::from(detail_rows) >= total
        {
            self.seen_end.set(true);
        }
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
        Self::details(request)
            .scroll((scroll, 0))
            .render(place(detail_rows), buf);
        footer.render(place(footer_rows), buf);
    }
}

#[cfg(test)]
#[path = "claude_approval_view_tests.rs"]
mod tests;
