//! Inspect completed frames and control the fake backend used by terminal tests.

use std::hash::Hash;
use std::hash::Hasher;
use std::io;
use std::io::Write;
use std::sync::Arc;
use std::sync::Mutex;

use ratatui::backend::Backend;
use ratatui::buffer::Buffer;
use ratatui::layout::Size;

use super::Terminal;

/// External control retains the original override's value-based clone/equality/hash.
#[derive(Debug)]
pub(super) struct ScreenSizeOverride(Arc<Mutex<Size>>);

impl ScreenSizeOverride {
    pub(super) fn new(size: Size) -> Self {
        Self(Arc::new(Mutex::new(size)))
    }

    pub(super) fn get(&self) -> Size {
        *self.0.lock().expect("test screen size lock")
    }

    fn controller(&self) -> impl Fn(Size) + use<> {
        let size = Arc::clone(&self.0);
        move |next_size| *size.lock().expect("test screen size lock") = next_size
    }
}

impl Clone for ScreenSizeOverride {
    fn clone(&self) -> Self {
        Self::new(self.get())
    }
}

impl PartialEq for ScreenSizeOverride {
    fn eq(&self, other: &Self) -> bool {
        self.get() == other.get()
    }
}

impl Eq for ScreenSizeOverride {}

impl Hash for ScreenSizeOverride {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.get().hash(state);
    }
}

/// Control the sampled backend size while an event loop borrows the terminal.
pub(crate) fn screen_size_controller<B>(terminal: &Terminal<B>) -> impl Fn(Size) + use<B>
where
    B: Backend<Error = io::Error> + Write,
{
    terminal
        .screen_size_override
        .as_ref()
        .expect("test screen size override")
        .controller()
}

pub(crate) fn last_rendered_buffer<B>(terminal: &Terminal<B>) -> &Buffer
where
    B: Backend<Error = io::Error> + Write,
{
    terminal.previous_buffer()
}
