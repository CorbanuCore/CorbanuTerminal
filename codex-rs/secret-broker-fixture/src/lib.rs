#![forbid(unsafe_code)]

//! Test-only synthetic launch scaffolding for the credential broker service.
//! Nothing here is validated executable or OS authority, and no production
//! crate depends on this package.

#[cfg(target_os = "linux")]
mod launch;
#[cfg(target_os = "linux")]
pub use launch::SyntheticChildIdentity;
#[cfg(target_os = "linux")]
pub use launch::SyntheticChildRole;
#[cfg(target_os = "linux")]
pub use launch::SyntheticLaunchRecipe;
#[cfg(target_os = "linux")]
pub use launch::SyntheticManifestInspection;
#[cfg(target_os = "linux")]
pub use launch::SyntheticSealedImage;
