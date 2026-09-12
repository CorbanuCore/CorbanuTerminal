//! Experimental non-root launch primitive, not production broker activation.
//! Requires a trusted procfs/mount namespace, exclusive child reaping, no external
//! pthread cancellation, and permitted pidfd signal/wait syscalls. A dedicated
//! owner must hold the result: cleanup can block, including in Drop. The separate
//! asynchronous quarantine/supervisor integration is not provided by this crate.
#![deny(unsafe_op_in_unsafe_fn)]
#![cfg(all(target_os = "linux", target_env = "gnu", feature = "synthetic-fixture"))]

mod ffi;
mod peer;
mod spawn;

pub use spawn::OwnedChild;
pub use spawn::SyntheticRole;
pub use spawn::spawn_synthetic_probe;
