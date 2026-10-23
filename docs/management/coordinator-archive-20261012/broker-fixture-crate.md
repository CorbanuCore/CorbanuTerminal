You are a Rust worker for CorbanuCore/CorbanuTerminal. Prepare a DRAFT PR for an open decision; do not merge.

## Problem
`codex-rs/secret-broker-service` has a Cargo feature, `synthetic-fixture`, that gates:
- two test-only binaries: `codex-secret-broker-service-fixture` (src/fixture/) and `codex-protected-root-probe`
  (src/probe/), about 830 lines, both `required-features = ["synthetic-fixture"]`;
- the library module `launch` (src/launch/), which re-exports SyntheticChildIdentity, SyntheticChildRole,
  SyntheticLaunchRecipe, SyntheticManifestInspection and SyntheticSealedImage under
  `cfg(all(target_os = "linux", feature = "synthetic-fixture"))`;
- seven optional dependencies.
Workspace policy forbids crate features (.github/scripts/verify_cargo_workspace_manifests.py), so `repo-checks`
fails. Bazel ignores features but still compiles the probe binary, so Bazel test, clippy and "Verify release
build" fail with `unresolved import codex_secret_broker_service::SyntheticChildIdentity`. defs.bzl's own
guidance is to "prefer to refactor optional functionality to a separate crate".

## Goal: option B
Move the fixture and probe binaries, and the `launch` module if it is only fixture scaffolding, into a new
test-only workspace crate `codex-rs/secret-broker-fixture` (package `codex-secret-broker-fixture`). Then:
- no `[features]` and no optional deps remain in secret-broker-service;
- the production broker crate no longer contains fixture code at all;
- Bazel builds and tests the fixture crate normally, with a BUILD.bazel like the other crates'.

## Rules
- Work in a new worktree /Volumes/CorbanuDrive/Corbanu/worktrees/broker-fixture-20261002 on branch
  `codex/secret-broker-fixture-crate`, based on `codex/ci-main-baseline-20261002` (open PR #146; it carries
  main's other CI fixes). Read AGENTS.md and codex-rs/AGENTS.md first.
- Behavior must be identical. Keep linux-only cfgs as they are. If `launch` or the fixture needs crate-private
  broker internals, expose the narrowest pub API needed and justify each item in the PR body. If that would mean
  widening a security-sensitive surface, stop and report instead.
- Update every caller of `--features synthetic-fixture`. That includes the PF-27 scripts under
  qa/security-levels/sprints/PF-27-S04/ and any test or justfile references (`rg -n synthetic-fixture`). Point
  them at the new package and binaries.
- Cargo.lock: run the normal cargo build, then `just bazel-lock-update` (MODULE.bazel.lock). Keep the dependency
  set the same; only the owning crate changes.
- Verify: `python3 .github/scripts/verify_cargo_workspace_manifests.py` passes; `just fmt`; `cargo check
  --tests` for both crates (on macOS, plus `cargo check --target x86_64-unknown-linux-gnu` if that toolchain is
  available, otherwise say so); `just test -p codex-secret-broker-service` and `-p codex-secret-broker-fixture`.
  The linux-only fixture tests can't run on macOS; say so. You may also run them on the RTX Linux box
  (ssh ambient@100.99.88.49, in a fresh clone under ~/corbanu-rtx/, never touching ~/corbanu-rtx/integration).
- Open a DRAFT PR to main titled "secret-broker: move synthetic fixture into its own test crate (no crate
  features)". In the body: the why (the policy and the Bazel failures), what moved, any newly-pub items with
  justification, how it was verified, and that it needs the security owner's (Jim Ricketts) review. Do not merge.
  Never force-push other branches.

## Report (your final message)
The PR URL, files moved, newly-public items (if any), the verification results, Linux run results, and risks.
