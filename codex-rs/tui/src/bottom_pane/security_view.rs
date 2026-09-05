//! Observation-only profile exploration: no sender, persistence or authority API.

use codex_protocol::security::SecurityLevel;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::Clear;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Widget;

use crate::key_hint;
use crate::key_hint::KeyBindingListExt;
use crate::keymap::ListKeymap;
use crate::render::renderable::Renderable;
use crate::security::view::PROFILES;
use crate::security::view::profile_name;
use crate::security::view::profile_summary;
use crate::security::view::requested_summary;

use super::CancellationEvent;
use super::bottom_pane_view::BottomPaneView;
use super::bottom_pane_view::ViewCompletion;

pub(crate) struct SecurityView {
    requested: Option<SecurityLevel>,
    selected: usize,
    keymap: ListKeymap,
    cancelled: bool,
    inspected: bool,
}

impl SecurityView {
    pub(crate) fn new(requested: Option<SecurityLevel>, keymap: ListKeymap) -> Self {
        Self {
            requested,
            selected: PROFILES.iter().position(|level| Some(*level) == requested).unwrap_or(0),
            keymap,
            cancelled: false,
            inspected: false,
        }
    }

    fn lines(&self, width: u16) -> Vec<Line<'static>> {
        let mut lines = vec!["Security profiles — read only".bold().into()];
        let paragraphs = [
            format!("Requested: {}", requested_summary(self.requested)),
            "Effective protection: unverified. Live security status is unavailable.".to_string(),
            "Protected modes are blocked; required controls are not qualified.".to_string(),
        ];
        for paragraph in paragraphs {
            lines.extend(textwrap::wrap(&paragraph, usize::from(width.max(1)))
                .into_iter().map(|line| Line::from(line.into_owned())));
        }
        lines.push(Line::default());
        for (index, level) in PROFILES.iter().enumerate() {
            let name = profile_name(*level);
            lines.push(if index == self.selected {
                format!("> {name}").cyan().bold().into()
            } else {
                format!("  {name}").into()
            });
        }
        lines.push(Line::default());
        lines.extend(textwrap::wrap(profile_summary(PROFILES[self.selected]), usize::from(width.max(1)))
            .into_iter().map(|line| Line::from(line.into_owned())));
        if self.inspected {
            lines.extend(textwrap::wrap("Nothing changed. Applying profiles is not available in this build.", usize::from(width.max(1)))
                .into_iter().map(|line| Line::from(line.into_owned()).cyan()));
        }
        lines
    }

    fn footer(&self) -> String {
        let label = |bindings: &[key_hint::KeyBinding]| {
            bindings.first().map(|key| key.display_label()).unwrap_or_else(|| "unbound".into())
        };
        format!("{}/{} explore · {} inspect · esc close", label(&self.keymap.move_up), label(&self.keymap.move_down), label(&self.keymap.accept))
    }
}

impl BottomPaneView for SecurityView {
    fn handle_key_event(&mut self, key: KeyEvent) {
        if key_hint::plain(KeyCode::Esc).is_press(key) || self.keymap.cancel.is_press(key) {
            self.cancelled = true;
        } else if self.keymap.move_up.is_press(key) {
            self.selected = (self.selected + PROFILES.len() - 1) % PROFILES.len();
            self.inspected = false;
        } else if self.keymap.move_down.is_press(key) {
            self.selected = (self.selected + 1) % PROFILES.len();
            self.inspected = false;
        } else if self.keymap.accept.is_press(key) {
            self.inspected = true;
        }
    }

    fn is_complete(&self) -> bool { self.cancelled }

    fn completion(&self) -> Option<ViewCompletion> {
        self.cancelled.then_some(ViewCompletion::Cancelled)
    }

    fn on_ctrl_c(&mut self) -> CancellationEvent {
        self.cancelled = true;
        CancellationEvent::Handled
    }

    fn prefer_esc_to_handle_key_event(&self) -> bool { true }
}

impl Renderable for SecurityView {
    fn desired_height(&self, width: u16) -> u16 {
        self.lines(width).len() as u16 + textwrap::wrap(&self.footer(), usize::from(width.max(1))).len() as u16 + 1
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        Clear.render(area, buf);
        let footer = self.footer();
        let footer_lines: Vec<Line> = textwrap::wrap(&footer, usize::from(area.width.max(1)))
            .into_iter().map(|line| Line::from(line.into_owned()).dim()).collect();
        let footer_height = (footer_lines.len() as u16).min(area.height);
        let body = Rect { height: area.height.saturating_sub(footer_height), ..area };
        Paragraph::new(self.lines(area.width)).render(body, buf);
        Paragraph::new(footer_lines).render(Rect {
            y: area.bottom().saturating_sub(footer_height), height: footer_height, ..area
        }, buf);
    }
}

#[cfg(test)]
#[path = "security_view_tests.rs"]
mod tests;
