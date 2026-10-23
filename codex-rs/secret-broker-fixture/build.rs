// Cargo builds enable the synthetic-fixture features of codex-protected-state and
// codex-linux-pidfd-spawn through this crate's dependency entries. Bazel compiles
// internal crates without Cargo features and does not run this script (see
// BUILD.bazel), so the descriptor-owner modules that need those APIs stay out of
// Bazel builds until the features are removed.
fn main() {
    println!("cargo:rustc-check-cfg=cfg(synthetic_pidfd_owner)");
    println!("cargo:rustc-cfg=synthetic_pidfd_owner");
}
