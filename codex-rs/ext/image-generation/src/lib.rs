// Corbanu accounting wraps the request transport, which deepens the tool
// future past the default limit when its `Send` bound is checked.
#![recursion_limit = "256"]

mod artifact;
mod backend;
mod extension;
mod tool;

pub use extension::install;

pub(crate) const IMAGE_GEN_NAMESPACE: &str = "image_gen";
pub(crate) const IMAGEGEN_TOOL_NAME: &str = "imagegen";
