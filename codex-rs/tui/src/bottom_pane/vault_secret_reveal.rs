//! Host-owned secure display for an explicitly revealed vault credential.

use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Widget;
use ratatui::widgets::Wrap;
use zeroize::Zeroize;

use crate::render::renderable::Renderable;

use super::BottomPaneView;
use super::CancellationEvent;
use super::ViewCompletion;

/// Secure, transient secret display. Its contents never enter transcript or composer state.
pub(crate) struct VaultSecretRevealView {
    label: String,
    secret: String,
    completion: Option<ViewCompletion>,
}

impl VaultSecretRevealView {
    pub(crate) fn new(label: String, secret: String) -> Self {
        Self {
            label,
            secret,
            completion: None,
        }
    }

    fn close(&mut self, completion: ViewCompletion) {
        self.secret.zeroize();
        self.completion = Some(completion);
    }
}

impl Drop for VaultSecretRevealView {
    fn drop(&mut self) {
        self.secret.zeroize();
    }
}

impl BottomPaneView for VaultSecretRevealView {
    fn handle_key_event(&mut self, event: KeyEvent) {
        match event.code {
            KeyCode::Enter => self.close(ViewCompletion::Accepted),
            KeyCode::Esc => self.close(ViewCompletion::Cancelled),
            _ => {}
        }
    }

    fn on_ctrl_c(&mut self) -> CancellationEvent {
        self.close(ViewCompletion::Cancelled);
        CancellationEvent::Handled
    }

    fn is_complete(&self) -> bool {
        self.completion.is_some()
    }

    fn completion(&self) -> Option<ViewCompletion> {
        self.completion
    }

    fn terminal_title_requires_action(&self) -> bool {
        true
    }
}

impl Renderable for VaultSecretRevealView {
    fn desired_height(&self, _width: u16) -> u16 {
        10
    }

    fn render(&self, area: Rect, buffer: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Vault credential — secure view ".bold());
        let inner = block.inner(area);
        block.render(area, buffer);
        let lines = vec![
            Line::from("Anyone with this value may control the associated account.").red(),
            Line::from(""),
            Line::from(format!("Label: {}", self.label)).dim(),
            Line::from(""),
            Line::from(self.secret.as_str().cyan().bold()),
            Line::from(""),
            Line::from("Press Enter or Esc to clear this secure view.").dim(),
        ];
        Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .render(inner, buffer);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closing_zeroizes_revealed_secret() {
        let mut view = VaultSecretRevealView::new(
            "qa/disposable".to_string(),
            "sensitive-vault-marker".to_string(),
        );

        view.handle_key_event(KeyEvent::from(KeyCode::Enter));

        assert!(view.secret.chars().all(|character| character == '\0'));
        assert_eq!(view.completion(), Some(ViewCompletion::Accepted));
    }
}
