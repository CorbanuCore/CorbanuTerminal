# CI Coverage

Corbanu Terminal CI currently runs only checks backed by available GitHub-hosted Linux
x64 runners. This is intentional: prior Mac, Windows, ARM, and self-hosted
runner jobs were failing before checkout with no executed steps, which created
red status noise without testing the code.

## Active Push/PR Coverage

- Formatting, spelling, manifest, and dependency checks.
- Rust cargo checks on Linux x64.
- Linux clippy (`rust-ci / Lint/Build — ubuntu-24.04 - x86_64-unknown-linux-gnu`):
  `cargo clippy --tests -D warnings` for the gnu dev target, so Linux-only
  lint errors fail the PR instead of landing on `main`. It runs when
  `codex-rs/` or `.github/` changes, is part of `CI results (required)`, uses
  the shared sccache/cargo-home caches (typically 6-10 minutes) and has a
  20-minute timeout. The job is `rust-ci-lint-build.yml`, shared with
  postmerge `rust-ci-full`, which also runs the musl dev and musl release legs.
  The full Linux test suite stays postmerge only.
- Windows clippy (`rust-ci / Lint/Build — windows-2022 - x86_64-pc-windows-msvc`):
  the same `cargo clippy --tests -D warnings` for the Windows msvc dev target
  (#717), so `cfg(unix)`-gated imports, helpers and `mut`s that are unused on
  Windows fail the PR instead of breaking postmerge. Same triggers, required
  status and shared job as the Linux leg. It restores the caches postmerge
  `rust-ci-full` saves for this leg (about 12 minutes warm, 15 cold; 60-minute
  timeout).
- Argument comment lint on changed crates
  (`rust-ci / Argument comment lint - changed crates`, #636): runs the
  `/*param*/` lint, test code included, on every `codex-rs` crate whose `.rs`
  files the PR changes (`.github/scripts/argument_comment_lint_packages.py`;
  a change to the lint job itself lints the small `codex-utils-string` crate).
  It uses `tools/argument-comment-lint/run.py` (Cargo, `--all-targets`) on
  Linux and is part of `CI results (required)`. Run it locally with
  `tools/argument-comment-lint/run.py -p <crate> -- --ignore-rust-version`.
  The Windows Bazel lint below skips test targets, and the full lints run
  nightly.
- Rust nextest on Linux x64.
- SDK checks on hosted Linux x64.
- V8 canary coverage on hosted Linux x64 only.

## Actions Cache

The repository's Actions cache is limited to 10 GB and GitHub evicts the least
recently used entries beyond that. Only `main` saves caches (#636): every save
step is guarded by `github.ref == 'refs/heads/main'`, as are Swatinem
`save-if`, setup-uv `save-cache` and setup-bazel `cache-save`. Pull requests,
and workflows dispatched on other branches, restore the caches `main` saved and
save none, so PR churn can't evict them. setup-node and setup-zig can't restore
without saving, so their caches are on only for `main` (both are used only by
postmerge or nightly jobs). Where blocking-ci and postmerge run the same
clippy leg on `main`, only postmerge `rust-ci-full` saves it.
`.github/scripts/test_ci_timeout_budgets.py` enforces this. Check usage with
`gh api repos/CorbanuCore/CorbanuTerminal/actions/cache/usage`.

## Nightly Coverage

`nightly-ci.yml` runs daily at 09:17 UTC and on manual dispatch. It does not
block merges. Without a BuildBuddy remote cache these cold Bazel jobs take two
to five hours, so they no longer run on every PR and get the 360-minute
hosted-runner maximum:

- Bazel test (Linux gnu and musl, and native Windows), clippy, and
  release-build verification.
- Argument comment lint on Linux and macOS (Windows still runs on PRs;
  `rust-ci-full.yml` does not repeat it).
- The Bazel-built SDK job (`sdk / sdks`; the Python SDK test still runs on PRs).

CodeQL (`codeql.yml`) also runs only on pushes to `main`, daily at 08:41 UTC
and on manual dispatch, not on pull requests: its Rust analysis takes over an
hour. Alerts appear on `main`. GitHub's default code-scanning setup must stay
disabled, because it conflicts with this workflow.

Check the latest nightly run before cutting a release. To run these jobs on
every PR again, pass `cold_bazel: true` to the `bazel`, `rust-ci` and `sdk` calls in
`blocking-ci.yml` (and add a `BUILDBUDDY_API_KEY` secret so they finish in time).

## Disabled Until Runners Exist

- macOS Rust and Bazel checks outside the manual release builder.
- Windows Rust and Bazel checks.
- Linux ARM64 Rust, Bazel, and V8 push/PR checks outside the manual release
  builder.
- Self-hosted runner jobs for unavailable Linux, macOS, Windows, and ARM64
  platform pools.

These disabled jobs were not real failing tests. They failed before checkout
because GitHub could not assign a runner.

## Restoration Requirements

Before re-enabling platform checks:

1. Confirm the runner label exists and is attached to this repository or org.
2. Confirm a trivial workflow using that exact `runs-on` value reaches checkout.
3. Re-enable one platform leg at a time.
4. Make the platform check required only after it has passed on at least one
   fresh push.

Until then, CI should stay green and honest rather than displaying checks that
never run.

Manual release builds are separate from push/PR CI. The Corbanu Terminal release
workflow uses GitHub-hosted macOS runners and the hosted `ubuntu-24.04-arm`
runner for the Linux ARM64 release artifact, so it does not depend on the
disabled self-hosted runner labels above.
