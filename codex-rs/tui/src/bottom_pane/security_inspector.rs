//! PF-41-S01: the read-only security inspector, opened with `i` from
//! `/security`. It shows configured, resolved and observed protection
//! separately (see [`crate::security::inspector`]), refreshes with `r`, and
//! returns to `/security` with Esc. It has no key that changes anything.

use std::cell::Cell;

use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Widget;

use crate::key_hint;
use crate::key_hint::KeyBindingListExt;
use crate::keymap::ListKeymap;
use crate::legacy_core::security_inspection::PolicyFacts;
use crate::legacy_core::security_inspection::RuntimeFacts;
use crate::security::inspector;
use crate::security::inspector::Badge;
use crate::security::inspector::InspectorInput;
use crate::security::inspector::STALE_AFTER_SECONDS;
use crate::security::inspector::Saved;
use crate::security::inspector::State;
use crate::wrapping::RtOptions;
use crate::wrapping::word_wrap_lines;

/// Section rows shown at once; the rest scrolls.
const MAX_BODY_ROWS: u16 = 20;

pub(crate) struct SecurityInspector {
    input: InspectorInput,
    saved: Saved,
    facts: RuntimeFacts,
    keymap: ListKeymap,
    /// The clock; tests pin it.
    now: fn() -> i64,
    scroll: u16,
    max_scroll: Cell<u16>,
    pub(crate) closed: bool,
}

fn unix_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs() as i64)
}

impl SecurityInspector {
    pub(crate) fn new(input: InspectorInput, keymap: ListKeymap) -> Self {
        Self::with_clock(input, keymap, unix_now)
    }

    pub(crate) fn with_clock(input: InspectorInput, keymap: ListKeymap, now: fn() -> i64) -> Self {
        let facts = input.observe(now());
        Self {
            saved: Saved::load(&input),
            input,
            facts,
            keymap,
            now,
            scroll: 0,
            max_scroll: Cell::new(0),
            closed: false,
        }
    }

    /// Read every fact again. Reading never changes authority.
    fn refresh(&mut self) {
        self.saved = Saved::load(&self.input);
        self.facts = self.input.observe((self.now)());
    }

    pub(crate) fn handle_key_event(&mut self, key: KeyEvent) {
        if key_hint::plain(KeyCode::Esc).is_press(key) || self.keymap.cancel.is_pressed(key) {
            self.closed = true;
        } else if key_hint::plain(KeyCode::Char('r')).is_press(key) {
            self.refresh();
        } else if self.keymap.move_up.is_pressed(key) {
            self.scroll = self.scroll.min(self.max_scroll.get()).saturating_sub(1);
        } else if self.keymap.move_down.is_pressed(key) {
            self.scroll = (self.scroll + 1).min(self.max_scroll.get());
        }
    }

    /// The pinned header (title, badge, observation age) and the sections.
    fn content(&self, width: u16) -> (Vec<Line<'static>>, Vec<Line<'static>>) {
        let width = usize::from(width.max(1));
        let now = (self.now)();
        let sections = inspector::sections(&self.input, &self.saved, &self.facts, now);
        let badge = inspector::badge(&self.input, &self.saved, &self.facts, &sections, now);
        let mut lines = vec![Line::from("Security inspector (read only)".bold())];
        let badge_line: Line<'static> = match badge {
            Badge::Permissive => "○ Permissive: no protection added by /security"
                .dim()
                .into(),
            Badge::Protected(level) => format!(
                "● Protected: {level}; every required control observed or checked at launch"
            )
            .green()
            .bold()
            .into(),
            Badge::Partial(level, off) => {
                format!("◐ {level} set; off or not observed: {}", off.join(", "))
                    .cyan()
                    .into()
            }
            Badge::Degraded(level, reasons) => {
                format!("▲ {level} degraded: {}", reasons.join(", "))
                    .red()
                    .bold()
                    .into()
            }
            Badge::Blocked(reason) => format!("■ Blocked: {reason}").red().bold().into(),
        };
        lines.extend(word_wrap_lines([badge_line], width));
        let age = now - self.facts.observed_at;
        let generation = match &self.facts.policy {
            PolicyFacts::Live(tree) => {
                format!(
                    ", policy generation {}.{}",
                    tree.epoch, tree.revocation_generation
                )
            }
            PolicyFacts::Stored { .. } | PolicyFacts::Unreadable => String::new(),
        };
        let observed = if age > STALE_AFTER_SECONDS {
            Line::from(format!("Observed {age}s ago{generation}: stale, press r").red())
        } else {
            Line::from(format!("Observed {age}s ago{generation}").dim())
        };
        lines.extend(word_wrap_lines([observed], width));
        let mut body = Vec::new();
        for section in sections {
            body.push(Line::default());
            body.push(Line::from(section.title.bold()));
            for row in section.rows {
                let mut spans: Vec<Span<'static>> = vec!["  ".into()];
                spans.extend(tag(row.state));
                spans.push(format!("{}: ", row.label).bold());
                spans.push(row.value.into());
                spans.push(format!(" [{}]", row.source).dim());
                body.extend(word_wrap_lines(
                    [Line::from(spans)],
                    RtOptions::new(width).subsequent_indent("      ".into()),
                ));
            }
        }
        (lines, body)
    }

    fn footer(&self, width: u16) -> Vec<Line<'static>> {
        let label = |bindings: &[key_hint::KeyBinding]| {
            bindings
                .first()
                .map(key_hint::KeyBinding::display_label)
                .unwrap_or_else(|| "unbound".into())
        };
        let text = format!(
            "{}/{} scroll · r refresh · esc back to /security",
            label(&self.keymap.move_up),
            label(&self.keymap.move_down)
        );
        textwrap::wrap(&text, usize::from(width.max(1)))
            .into_iter()
            .map(|line| Line::from(line.into_owned()).dim())
            .collect()
    }

    /// The sections scroll inside a bounded pane, under a pinned header.
    pub(crate) fn desired_height(&self, width: u16) -> u16 {
        let (header, body) = self.content(width);
        header.len() as u16
            + (body.len() as u16).min(MAX_BODY_ROWS)
            + self.footer(width).len() as u16
    }

    pub(crate) fn render(&self, area: Rect, buf: &mut Buffer) {
        let (header, body) = self.content(area.width);
        let footer = self.footer(area.width);
        let header_height = (header.len() as u16).min(area.height);
        let footer_height = (footer.len() as u16).min(area.height - header_height);
        let body_height = area.height - header_height - footer_height;
        let max = u16::try_from(body.len())
            .unwrap_or(u16::MAX)
            .saturating_sub(body_height);
        self.max_scroll.set(max);
        Paragraph::new(header).render(
            Rect {
                height: header_height,
                ..area
            },
            buf,
        );
        Paragraph::new(body)
            .scroll((self.scroll.min(max), 0))
            .render(
                Rect {
                    y: area.y + header_height,
                    height: body_height,
                    ..area
                },
                buf,
            );
        Paragraph::new(footer).render(
            Rect {
                y: area.bottom() - footer_height,
                height: footer_height,
                ..area
            },
            buf,
        );
    }
}

fn tag(state: State) -> Option<Span<'static>> {
    Some(match state {
        State::Enforcing => "[on] ".green(),
        State::Resolved => "[set] ".cyan(),
        State::Unobserved => "[unobserved] ".dim().italic(),
        State::Off => "[off] ".dim(),
        State::Degraded => "[degraded] ".red().bold(),
        State::NotContained => "[not contained] ".magenta(),
        State::NotAvailable => "[not available] ".dim(),
        State::Info => return None,
    })
}

#[cfg(test)]
#[path = "security_inspector_tests.rs"]
mod tests;
