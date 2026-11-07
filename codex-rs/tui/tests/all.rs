#![allow(clippy::expect_used)]

// Single integration test binary that aggregates all test modules.
// The submodules live in `tests/suite/`.
mod test_backend;

<<<<<<< HEAD
#[cfg(unix)]
mod support;

#[allow(unused_imports)]
use codex_cli as _; // Keep dev-dep for cargo-shear; tests spawn the codex binary.

||||||| 413492cd6c
#[allow(unused_imports)]
use codex_cli as _; // Keep dev-dep for cargo-shear; tests spawn the codex binary.

=======
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
mod suite;
