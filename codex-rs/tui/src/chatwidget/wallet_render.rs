use ratatui::style::Stylize as _;
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Wrap;

use crate::render::renderable::ColumnRenderable;

pub(super) enum WalletTextStyle {
    Normal,
    Dimmed,
    Danger,
}

/// Adds one semantic wallet line that measures and wraps against its rendered area.
pub(super) fn push_wallet_text(
    header: &mut ColumnRenderable<'_>,
    text: &str,
    style: WalletTextStyle,
) {
    let span = match style {
        WalletTextStyle::Normal => text.to_string().into(),
        WalletTextStyle::Dimmed => text.to_string().dim(),
        WalletTextStyle::Danger => text.to_string().red(),
    };
    header.push(Paragraph::new(Line::from(span)).wrap(Wrap { trim: false }));
}

#[cfg(test)]
#[path = "wallet_render_tests.rs"]
mod tests;
