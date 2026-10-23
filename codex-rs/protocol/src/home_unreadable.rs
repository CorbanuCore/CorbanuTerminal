//! #616: marks failures caused by a Corbanu home that cannot be read.
//!
//! A Corbanu helper (such as the Claude Plan token helper) that finds its home
//! unreadable exits with [`HOME_UNREADABLE_EXIT_CODE`]. Its caller turns that
//! into an [`std::io::Error`] carrying [`HomeUnreadableError`], so retry logic
//! can tell "the home is temporarily unreadable" apart from other failures.

use std::fmt;
use std::io;

/// Exit status of a Corbanu process that could not read its home
/// (`EX_TEMPFAIL`: the condition is expected to be temporary).
pub const HOME_UNREADABLE_EXIT_CODE: i32 = 75;

/// The payload of an [`io::Error`] caused by an unreadable Corbanu home.
#[derive(Debug)]
pub struct HomeUnreadableError(String);

impl fmt::Display for HomeUnreadableError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for HomeUnreadableError {}

/// An [`io::Error`] (`PermissionDenied`) marked as caused by an unreadable home.
pub fn home_unreadable_io_error(message: String) -> io::Error {
    io::Error::new(
        io::ErrorKind::PermissionDenied,
        HomeUnreadableError(message),
    )
}

/// Whether `error` was made by [`home_unreadable_io_error`].
pub fn is_home_unreadable(error: &io::Error) -> bool {
    let Some(inner) = error.get_ref() else {
        return false;
    };
    inner.is::<HomeUnreadableError>()
}
