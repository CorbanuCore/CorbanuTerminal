use super::*;
use crate::render::renderable::Renderable;
use pretty_assertions::assert_eq;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

fn render_wallet_lines(width: u16) -> String {
    let mut header = ColumnRenderable::new();
    push_wallet_text(
        &mut header,
        "Weekly: 4,795,298 used + 0 in flight of 50,000,000 tokens (9.6%)",
        WalletTextStyle::Normal,
    );
    push_wallet_text(
        &mut header,
        "45,204,702 remaining · resets in 6d 10h · 2026-07-26T00:35:20.051Z",
        WalletTextStyle::Dimmed,
    );
    push_wallet_text(
        &mut header,
        "Next: Basic plan · 20 USDC prepaid · 2026-08-19T00:35:20.051Z to 2026-09-19T00:35:20.051Z",
        WalletTextStyle::Dimmed,
    );
    let height = header.desired_height(width);
    let area = Rect::new(0, 0, width, height);
    let mut buffer = Buffer::empty(area);
    header.render(area, &mut buffer);
    (0..height)
        .map(|row| {
            (0..width)
                .map(|column| buffer[(column, row)].symbol())
                .collect::<String>()
                .trim_end()
                .to_string()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn normalized(rendered: &str) -> String {
    rendered.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[test]
fn wallet_lines_measure_and_render_without_clipping_at_adjacent_widths() {
    let expected = "Weekly: 4,795,298 used + 0 in flight of 50,000,000 tokens (9.6%) \
        45,204,702 remaining · resets in 6d 10h · 2026-07-26T00:35:20.051Z \
        Next: Basic plan · 20 USDC prepaid · 2026-08-19T00:35:20.051Z to 2026-09-19T00:35:20.051Z";
    for width in [36, 63, 64, 90] {
        let rendered = render_wallet_lines(width);
        assert_eq!(normalized(&rendered), normalized(expected));
        assert!(
            rendered
                .lines()
                .all(|line| line.chars().count() <= width.into())
        );
    }
}

#[test]
fn wallet_lines_narrow_and_wide_snapshots() {
    insta::assert_snapshot!("wallet_lines_content_width_63", render_wallet_lines(/*width*/ 63));
    insta::assert_snapshot!("wallet_lines_content_width_90", render_wallet_lines(/*width*/ 90));
}
