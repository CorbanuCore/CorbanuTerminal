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
- Rust nextest on Linux x64.
- SDK checks on hosted Linux x64.
- V8 canary coverage on hosted Linux x64 only.

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
