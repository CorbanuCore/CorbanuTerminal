//! Inspect the completed frame without adding test helpers to the terminal implementation.

use std::io;
use std::io::Write;

use ratatui::backend::Backend;
use ratatui::buffer::Buffer;
use ratatui::layout::Size;

use super::Terminal;

/// Control the sampled backend size while an event loop borrows the terminal.
pub(crate) fn screen_size_controller<B>(terminal: &Terminal<B>) -> impl Fn(Size) + use<B>
where
    B: Backend<Error = io::Error> + Write,
{
    let size = terminal
        .screen_size_override
        .as_ref()
        .expect("test screen size override")
        .clone();
    move |next_size| *size.lock().expect("test screen size lock") = next_size
}

pub(crate) fn last_rendered_buffer<B>(terminal: &Terminal<B>) -> &Buffer
where
    B: Backend<Error = io::Error> + Write,
{
    terminal.previous_buffer()
}
