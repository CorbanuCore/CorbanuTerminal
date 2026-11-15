// Corbanu accounting wraps the request transport, which deepens the tool
// future past the default limit when its `Send` bound is checked.
#![recursion_limit = "256"]

mod extension;
mod history;
mod output;
mod schema;
mod tool;

pub use extension::install;
