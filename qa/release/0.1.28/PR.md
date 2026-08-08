## release: prepare PfTerminal 0.1.28

Patch release. Scope confirmed by the operator: **PR #93 only** (the
non-destructive tasknode auth-state machine, expiry-aware session
resolution, and the tasknode-usage skill entrypoint fix). PR #94
(upstream Codex 0.147.0 sync) is deliberately excluded and deferred to
0.1.29 for its own review cycle including the full test suite.

### This PR contains

- Version bump `0.1.27` → `0.1.28` in `codex-rs/Cargo.toml` with a
  version-only lockfile refresh.
- `qa/release/0.1.28/`: feature-manifest baseline, candidate,
  comparison (zero differences, empty allowlist), and the acceptance
  ledger `ACCEPTANCE-20260808.md`.

### Gate status

- Feature manifest: 0 differences vs `rust-v0.1.27`, 0 unresolved, 0
  invalid allowlist entries.
- Release-facing local suites: all pass (worktree, manifest tooling,
  release contract, codex_package, install).
- Focused Rust suites: tasknode-session 10/10, TUI tasknode 2/2, CLI
  tasknode 6/6; fmt clean; clippy clean on touched crates.
- CI: Format and Codespell green on the release content; remaining red
  jobs fail identically on `main` (pre-existing debt, itemized in the
  ledger).

### Open cells for operator acceptance

1. macOS/Windows hands-on matrix not run (Linux-only preparer host).
2. Full workspace `cargo test` not run (focused suites + production
   field evidence of the auth-state machine stand in).
3. Pre-existing red CI jobs on main ride along unchanged.

Merging this PR records operator acceptance of those residual risks per
`docs/RELEASE.md`. After merge: unpublished qualification run, hash
reconciliation, then tag and publish only on explicit operator
authorization.
