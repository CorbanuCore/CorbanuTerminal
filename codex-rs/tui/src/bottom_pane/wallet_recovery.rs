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

/// Host-owned recovery display. Its contents never enter transcript or composer state.
pub(crate) struct WalletRecoveryView {
    address: String,
    recovery: String,
    completion: Option<ViewCompletion>,
    on_confirm: Option<Box<dyn FnOnce()>>,
    on_cancel: Option<Box<dyn FnOnce()>>,
}

impl WalletRecoveryView {
    pub(crate) fn new(address: String, recovery: String) -> Self {
        Self {
            address,
            recovery,
            completion: None,
            on_confirm: None,
            on_cancel: None,
        }
    }

    pub(crate) fn with_confirmation(mut self, on_confirm: Box<dyn FnOnce()>) -> Self {
        self.on_confirm = Some(on_confirm);
        self
    }

    pub(crate) fn with_cancellation(mut self, on_cancel: Box<dyn FnOnce()>) -> Self {
        self.on_cancel = Some(on_cancel);
        self
    }

    fn close(&mut self, accepted: bool) {
        self.recovery.zeroize();
        if accepted {
            self.on_cancel = None;
            if let Some(on_confirm) = self.on_confirm.take() {
                on_confirm();
            }
            self.completion = Some(ViewCompletion::Accepted);
        } else {
            self.on_confirm = None;
            if let Some(on_cancel) = self.on_cancel.take() {
                on_cancel();
            }
            self.completion = Some(ViewCompletion::Cancelled);
        }
    }
}

impl Drop for WalletRecoveryView {
    fn drop(&mut self) {
        self.recovery.zeroize();
    }
}

impl BottomPaneView for WalletRecoveryView {
    fn handle_key_event(&mut self, event: KeyEvent) {
        match event.code {
            KeyCode::Enter => self.close(/*accepted*/ true),
            KeyCode::Esc => self.close(/*accepted*/ false),
            _ => {}
        }
    }
    fn on_ctrl_c(&mut self) -> CancellationEvent {
        self.close(/*accepted*/ false);
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
            .title(" Wallet recovery — secure view ".bold());
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
            Line::from("Press Enter after you have stored it. This secure view will clear.").dim(),
        ];
        Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .render(inner, buffer);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::rc::Rc;

    #[test]
    fn enter_confirms_and_zeroizes_recovery_material() {
        let confirmed = Rc::new(Cell::new(false));
        let callback_state = Rc::clone(&confirmed);
        let mut view = WalletRecoveryView::new(
            "address".to_string(),
            "sensitive-recovery-marker".to_string(),
        )
        .with_confirmation(Box::new(move || callback_state.set(true)));

        view.handle_key_event(KeyEvent::from(KeyCode::Enter));

        assert!(confirmed.get());
        assert!(view.recovery.chars().all(|character| character == '\0'));
        assert_eq!(view.completion(), Some(ViewCompletion::Accepted));
    }

    #[test]
    fn escape_clears_without_claiming_backup_confirmation() {
        let confirmed = Rc::new(Cell::new(false));
        let callback_state = Rc::clone(&confirmed);
        let mut view = WalletRecoveryView::new(
            "address".to_string(),
            "sensitive-recovery-marker".to_string(),
        )
        .with_confirmation(Box::new(move || callback_state.set(true)));

        view.handle_key_event(KeyEvent::from(KeyCode::Esc));

        assert!(!confirmed.get());
        assert!(view.recovery.chars().all(|character| character == '\0'));
        assert_eq!(view.completion(), Some(ViewCompletion::Cancelled));
    }

    #[test]
    fn escape_reports_cancellation_to_owning_flow() {
        let cancelled = Rc::new(Cell::new(false));
        let callback_state = Rc::clone(&cancelled);
        let mut view = WalletRecoveryView::new("address".into(), "recovery".into())
            .with_cancellation(Box::new(move || callback_state.set(true)));

        view.handle_key_event(KeyEvent::from(KeyCode::Esc));

        assert!(cancelled.get());
        assert_eq!(view.completion(), Some(ViewCompletion::Cancelled));
    }
}
