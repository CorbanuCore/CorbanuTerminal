// Aggregates all former standalone integration tests as modules.
#[cfg(unix)]
mod focus_palette;
#[cfg(unix)]
mod large_image;
#[cfg(unix)]
mod output_text_stream;
mod resize_reflow;
#[cfg(unix)]
mod slash_dispatch;
mod status_indicator;
mod vt100_history;
mod vt100_live_commit;
