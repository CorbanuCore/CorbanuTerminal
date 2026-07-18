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

/// One-time host-owned recovery display. Its contents never enter transcript or composer state.
pub(crate) struct WalletRecoveryView {
    address: String,
    recovery: String,
    completion: Option<ViewCompletion>,
}

impl WalletRecoveryView {
    pub(crate) fn new(address: String, recovery: String) -> Self {
        Self {
            address,
            recovery,
            completion: None,
        }
    }

    fn close(&mut self) {
        self.recovery.zeroize();
        self.completion = Some(ViewCompletion::Accepted);
    }
}

impl Drop for WalletRecoveryView {
    fn drop(&mut self) {
        self.recovery.zeroize();
    }
}

impl BottomPaneView for WalletRecoveryView {
    fn handle_key_event(&mut self, event: KeyEvent) {
        if matches!(event.code, KeyCode::Enter | KeyCode::Esc) {
            self.close();
        }
    }
    fn on_ctrl_c(&mut self) -> CancellationEvent {
        self.close();
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

impl Renderable for WalletRecoveryView {
    fn desired_height(&self, _width: u16) -> u16 {
        10
    }

    fn render(&self, area: Rect, buffer: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Wallet recovery — shown once ".bold());
        let inner = block.inner(area);
        block.render(area, buffer);
        let lines = vec![
            Line::from("Store this recovery material offline. Anyone with it controls the wallet.")
                .red(),
            Line::from(""),
            Line::from(format!("Address: {}", self.address)).dim(),
            Line::from(""),
            Line::from(self.recovery.as_str().cyan().bold()),
            Line::from(""),
            Line::from("Press Enter after you have stored it. It will not be shown again.").dim(),
        ];
        Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .render(inner, buffer);
    }
}
