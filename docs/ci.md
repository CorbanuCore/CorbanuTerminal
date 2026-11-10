# CI Coverage

Corbanu Terminal CI runs only on GitHub-hosted runners: Linux x64 for most
jobs, plus the hosted Windows and macOS jobs listed below. This is intentional:
prior self-hosted (and some Mac, Windows and ARM) runner jobs were failing
before checkout with no executed steps, which created red status noise without
testing the code.

Which Rust tests a PR runs: the tests of the crates it changes, on Linux
(below, #1972), and for five crates also on macOS (non-blocking, #1989);
codex-core's security suites on Linux and macOS; native Windows tests for some
crates. The whole Linux suite runs only postmerge
(`rust-ci-full`) and in the nightly Bazel job, so a PR that breaks a crate it
doesn't change (through a dependency) shows up on `main`.

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
- Native Windows tests on PRs that change one of these groups of crates
  (`rust-ci / Windows native tests / Bazel test (changed crates)`): `tui`,
  `core`, `state` and `login` (#636); `windows-sandbox-rs`, `network-proxy`,
  `process-hardening` and `sandboxing`, whose Windows tests use one another
  (#1813); `exec-server` (#1813); `utils/pty` and `app-server` (#1839). It
  runs the nightly native Windows Bazel tests (`test-windows-native-main`),
  limited to the changed groups' test targets (`--build_tests_only`). As in
  nightly, a crate's integration tests (`tests/*.rs`) run through their
  `<name>-test-windows-cross` targets; the plain `<name>-test` twins show as
  `SKIPPED` (incompatible with the gnullvm platform) but test the same files.
  The exceptions are codex-core, codex-app-server, code-mode-host and
  code-mode-runtime: their targets depend on codex-code-mode-host / V8, which needs a
  target-platform Python that Bazel lacks there, so no Bazel job runs them on
  Windows (#1856). The `app-server` group therefore runs app-server's unit-test
  targets only; core's and app-server's integration tests run in the
  non-blocking Cargo job below. code-mode-host and code-mode-runtime run no
  tests on Windows at all, unit tests included.
  Other PRs skip it; a change to its workflow
  (`rust-ci-windows-native-tests.yml`) runs every group, but changes to
  `.bazelrc`, other Bazel files or the crates these depend on don't (nightly
  covers those). So a `windows-sandbox-rs` change doesn't run core's Windows
  tests, which would add core's cold build. When it runs it is part of `CI results (required)`. It restores the
  Bazel repository cache that nightly saves on `main` and saves nothing. With
  no build cache it compiles cold: about 43 minutes for the #636 group, about
  as long as the Windows argument comment lint that already runs on PRs
  (90-minute timeout); every group together took about 51 minutes on #1852
  (37 minutes before `utils/pty` and `app-server` were added).
  Bazel gives the sharded core, tui and app-server unit tests its usual three
  attempts.
  This job and the nightly one also give `//codex-rs/login:login-unit-tests`
  one re-run (`--flaky_test_attempts`) for the intermittent #691 test. If that
  test fails both attempts again, add it to the skip list in `.bazelrc`'s
  `windows-native-tests` config.
- Windows integration tests for codex-core and codex-app-server
  (`windows-integration-tests / codex-core` and `... / codex-app-server`,
  #1856), on PRs that change one of those crates; a change to the workflow
  (`windows-integration-tests.yml`), its known-failure list or script, or
  `codex-rs/.config/nextest.toml` runs both. They run
  `cargo nextest run -p <crate> --test '*'` on windows-2022 (msvc). The first
  native runs found 13 codex-core and 19 codex-app-server failures (#1967,
  #1968, #1969); they are listed in
  `.github/windows-integration-known-failures.txt`, excluded, and named in the
  job summary, so any other failure turns the job red. It is a separate
  workflow, not part of `CI required` (which neither waits for it nor fails
  with it), until those are fixed. It has no compile cache: about 22 minutes to
  build, then about 5 (core) or 13 (app-server) minutes of tests (60-minute
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
- Tests of the changed crates on Linux
  (`rust-ci / Changed crate tests / cargo nextest (changed crates)`, #1972):
  `cargo nextest run -p <crate>...` (unit and integration tests) for every
  `codex-rs` crate under which the PR changes any file
  (`.github/scripts/changed_crate_test_packages.py`), including crates Cargo
  makes members only because a member depends on them by path, such as
  `codex-windows-sandbox` (#2027). It does not add the
  crates that depend on them, and a `Cargo.lock`-only change runs nothing. It
  runs codex-core's remote-aware tests locally, without postmerge's Docker
  remote environment. It first builds the helper binaries some suites
  start from other crates (`codex`, the code-mode host, the Linux sandbox, the
  rmcp-client test servers, the wallet daemon) and installs the packaged
  ripgrep. A change to the job itself, `rust-ci.yml`, the workspace
  `Cargo.toml`, the toolchain or nextest config runs `codex-rmcp-client`'s
  tests. It uses postmerge's
  `ci-test` profile and restores its sccache (saves nothing). Before this no
  Linux or macOS PR job ran most crates' tests: #1963 went green while
  breaking two rmcp-client tests. Part of `CI results (required)` whenever it
  runs; 85-minute timeout. Measured on the #1972 trials: about 15 minutes for
  rmcp-client, 24 for codex-core and 46 for every crate at once (an upstream
  sync), against the roughly 35 minutes `CI required` already takes.
- Tests of the changed crates on macOS
  (`rust-ci changed crate tests (macOS) / <crate>`, #1989, owner decision
  D503): the same `cargo nextest run -p <crate>` on macos-15, one crate per
  leg, for the crates with macOS-only code paths that no other job tests on
  macOS: `codex-sandboxing` (Seatbelt), `codex-exec-server` (macOS process and
  file-system sandboxes), `codex-keyring-store` (Keychain),
  `codex-process-hardening` and `codex-network-proxy` (native certificates,
  the credential broker's memory scan). It selects them as the Linux job does
  (`.github/scripts/macos_changed_crate_tests.py`, using the shared
  `cargo_packages` list); a change to the job, its scripts or actions, the
  workspace `Cargo.toml`, the toolchain or nextest config runs all five.
  Known macOS failures are listed with their issues in
  `.github/macos-changed-crate-known-failures.txt`, excluded, and named in the
  job summary (#2069). It is a separate workflow, not part of
  `CI required`, until it has a cost and flake record. It restores the macOS
  core security sccache (saves nothing). Trial (#2068): 1.6 to 7 minutes per
  crate; `codex-login` (14 minutes, it builds codex-core) and `codex-vault` (no
  macOS-only code) passed but are left out.
- Core security tests (`rust-ci / Core security tests — ubuntu-24.04` and
  `— macos-15`, #512, #553, #902): codex-core's `security::` unit tests and
  selected integration suites on Linux and macOS, on every `codex-rs` change.
- Full Rust nextest suite on Linux x64: postmerge only (`postmerge-ci` →
  `rust-ci-full`), not on PRs.
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
clippy leg or the macOS core security tests on `main`, only postmerge
`rust-ci-full` saves the cache.

Build caches such as sccache are keyed by run ID, so every `main` run saves a
fresh copy (up to 2.1 GB each). The `cache-prune` workflow keeps only the newest
copy of each such cache: it runs after blocking-ci, postmerge-ci, nightly-ci,
tmux smoke and CodeQL finish on `main`, and hourly, and deletes older `main`
copies with `.github/scripts/prune_actions_caches.py` (`--dry-run` lists them).
It is the only job here that holds `actions: write`. Caches without a run ID in
the key (cargo-home, apt) are saved only on a key miss and aren't pruned.
`.github/scripts/test_ci_timeout_budgets.py` enforces this; a new workflow that
saves a run-ID-keyed cache on `main` must be added to `cache-prune`'s trigger
list. Check usage with
`gh api repos/CorbanuCore/CorbanuTerminal/actions/cache/usage`.

## Nightly Coverage

`nightly-ci.yml` runs daily at 09:17 UTC and on manual dispatch. It does not
block merges. Without a BuildBuddy remote cache these cold Bazel jobs take two
to five hours, so they no longer run on every PR and get the 360-minute
hosted-runner maximum:

- Bazel test (Linux gnu and musl, and native Windows for every crate), clippy,
  and release-build verification.
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

## Rust toolchain bumps

See [Rust toolchain bump](rust-toolchain-bump.md) for what to update and when.

## Disabled Until Runners Exist

- macOS Bazel checks, and macOS Rust checks other than the core security tests
  and the five crates above, outside the manual release builder.
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
