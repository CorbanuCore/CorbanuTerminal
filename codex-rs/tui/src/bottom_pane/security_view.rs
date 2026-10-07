//! Observation-only profile exploration: no sender, persistence or authority API.
//! With the `security_levels` flag (or a stored non-Permissive level) the view
//! delegates to the human-only [`SecurityLevelPicker`].

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
use crate::security::current::CurrentValues;
use crate::security::preflight::PreflightInput;
use crate::security::view::PROFILES;
use crate::security::view::profile_name;
use crate::security::view::profile_summary;
use crate::security::view::requested_summary;

use super::CancellationEvent;
use super::bottom_pane_view::BottomPaneView;
use super::bottom_pane_view::ViewCompletion;
use super::security_level_picker::SecurityLevelPicker;

pub(crate) struct SecurityView {
    requested: Option<SecurityLevel>,
    selected: usize,
    keymap: ListKeymap,
    cancelled: bool,
    inspected: bool,
    picker: Option<SecurityLevelPicker>,
}

impl SecurityView {
    /// `current` computes this session's value for each Aggressive row; it is
    /// called only when the level picker is enabled.
    pub(crate) fn new(
        requested: Option<SecurityLevel>,
        current: impl FnOnce() -> CurrentValues,
        preflight: impl FnOnce() -> Option<PreflightInput>,
        keymap: ListKeymap,
    ) -> Self {
        Self {
            requested,
            selected: PROFILES
                .iter()
                .position(|level| Some(*level) == requested)
                .unwrap_or(0),
            picker: crate::security::level::context()
                .filter(|context| context.picker_enabled)
                .map(|context| {
                    SecurityLevelPicker::new(context, current(), preflight(), keymap.clone())
                }),
            keymap,
            cancelled: false,
            inspected: false,
        }
    }

    /// PF-24-S02: Core's configured levels, and where "restart now" goes.
    pub(crate) fn with_confirmation(
        mut self,
        core: super::security_level_picker::CoreLevels,
        app_event_tx: crate::app_event_sender::AppEventSender,
    ) -> Self {
        if let Some(picker) = self.picker.as_mut() {
            picker.set_core_levels(core);
            picker.set_app_event_tx(app_event_tx);
        }
        self
    }

    fn lines(&self, width: u16) -> Vec<Line<'static>> {
        let mut lines = vec!["Security profiles — read only".bold().into()];
        let paragraphs = [
            format!("Requested: {}", requested_summary(self.requested)),
            "Effective protection: unverified. Live security status is unavailable.".to_string(),
            "Protected modes are blocked; required controls are not qualified.".to_string(),
        ];
        for paragraph in paragraphs {
            lines.extend(
                textwrap::wrap(&paragraph, usize::from(width.max(1)))
                    .into_iter()
                    .map(|line| Line::from(line.into_owned())),
            );
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
        lines.extend(
            textwrap::wrap(
                profile_summary(PROFILES[self.selected]),
                usize::from(width.max(1)),
            )
            .into_iter()
            .map(|line| Line::from(line.into_owned())),
        );
        if self.inspected {
            lines.extend(
                textwrap::wrap(
                    "Nothing changed. Applying profiles is not available in this build.",
                    usize::from(width.max(1)),
                )
                .into_iter()
                .map(|line| Line::from(line.into_owned()).cyan()),
            );
        }
        lines
    }

    fn footer(&self) -> String {
        let label = |bindings: &[key_hint::KeyBinding]| {
            bindings
                .first()
                .map(super::super::key_hint::KeyBinding::display_label)
                .unwrap_or_else(|| "unbound".into())
        };
        format!(
            "{}/{} explore · {} inspect · esc close",
            label(&self.keymap.move_up),
            label(&self.keymap.move_down),
            label(&self.keymap.accept)
        )
    }
}

impl SecurityView {
    fn body(&self, width: u16) -> Vec<Line<'static>> {
        match &self.picker {
            Some(picker) => picker.lines(width),
            None => self.lines(width),
        }
    }

    fn footer_text(&self) -> String {
        match &self.picker {
            Some(picker) => picker.footer(),
            None => self.footer(),
        }
    }
}

impl BottomPaneView for SecurityView {
    fn handle_key_event(&mut self, key: KeyEvent) {
        if let Some(picker) = self.picker.as_mut() {
            picker.handle_key_event(key);
            self.cancelled = picker.closed;
            return;
        }
        if key_hint::plain(KeyCode::Esc).is_press(key) || self.keymap.cancel.is_pressed(key) {
            self.cancelled = true;
        } else if self.keymap.move_up.is_pressed(key) {
            self.selected = (self.selected + PROFILES.len() - 1) % PROFILES.len();
            self.inspected = false;
        } else if self.keymap.move_down.is_pressed(key) {
            self.selected = (self.selected + 1) % PROFILES.len();
            self.inspected = false;
        } else if self.keymap.accept.is_pressed(key) {
            self.inspected = true;
        }
    }

    fn is_complete(&self) -> bool {
        self.cancelled
    }

    fn completion(&self) -> Option<ViewCompletion> {
        self.cancelled.then_some(ViewCompletion::Cancelled)
    }

    fn on_ctrl_c(&mut self) -> CancellationEvent {
        // A confirmation being saved finishes first; its result is shown.
        if !self.picker.as_ref().is_some_and(SecurityLevelPicker::saving) {
            self.cancelled = true;
        }
        CancellationEvent::Handled
    }

    fn prefer_esc_to_handle_key_event(&self) -> bool {
        true
    }

    fn pre_draw_tick(&mut self, _now: std::time::Instant) -> bool {
        self.picker.as_mut().is_some_and(SecurityLevelPicker::poll)
    }

    fn next_frame_delay(&self) -> Option<std::time::Duration> {
        self.picker
            .as_ref()
            .filter(|picker| picker.saving())
            .map(|_| std::time::Duration::from_millis(50))
    }
}

impl Renderable for SecurityView {
    fn desired_height(&self, width: u16) -> u16 {
        self.body(width).len() as u16
            + textwrap::wrap(&self.footer_text(), usize::from(width.max(1))).len() as u16
            + 1
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        Clear.render(area, buf);
        let footer_lines = |text: String| -> Vec<Line<'static>> {
            textwrap::wrap(&text, usize::from(area.width.max(1)))
                .into_iter()
                .map(|line| Line::from(line.into_owned()).dim())
                .collect()
        };
        let lines = self.body(area.width);
        let mut footer = footer_lines(self.footer_text());
        if let Some(picker) = self.picker.as_ref() {
            // A review taller than the pane scrolls, and its footer says so.
            let body_height = area.height.saturating_sub(footer.len() as u16);
            picker.scroll_for(lines.len(), body_height);
            footer = footer_lines(self.footer_text());
        }
        let footer_height = (footer.len() as u16).min(area.height);
        let body = Rect {
            height: area.height.saturating_sub(footer_height),
            ..area
        };
        let scroll = self
            .picker
            .as_ref()
            .map_or(0, |picker| picker.scroll_for(lines.len(), body.height));
        Paragraph::new(lines).scroll((scroll, 0)).render(body, buf);
        Paragraph::new(footer).render(
            Rect {
                y: area.bottom().saturating_sub(footer_height),
                height: footer_height,
                ..area
            },
            buf,
        );
    }
}

#[cfg(test)]
#[path = "security_view_tests.rs"]
mod tests;
