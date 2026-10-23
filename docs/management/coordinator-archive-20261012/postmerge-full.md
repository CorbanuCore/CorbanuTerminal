# Worker brief: make main's post-merge full suite (rust-ci-full) pass

You are an Opus 5.5 High worker. Work autonomously; do not ask questions.

## Context
- Repo CorbanuCore/CorbanuTerminal. `postmerge-ci.yml` had never started on main (startup_failure since
  2026-09-12) until #151 fixed it today. Its first real run, 37178057395, fails in `rust-ci-full`:
  - `Lint/Build — ubuntu-24.04 - x86_64-unknown-linux-gnu` and `... -musl` (failure)
  - `Tests — ubuntu-24.04 - x86_64-unknown-linux-gnu / Tests shard 1-4/4` (all failure)
  - `Argument comment lint - Linux` (cancelled)
  So this is a backlog of real failures that nobody has seen since September.
- Main also now has: #152 (native Windows lane → nightly), #153 (read-only workflow tokens, 360-min nightly budget),
  nightly-ci.yml (cold Bazel jobs). A separate worker is fixing the nightly Bazel test failures on branch
  `codex/ci-main-baseline-20261002`; another is deflaking the tmux smoke suite. Avoid overlapping their files where
  possible and coordinate via small, separate commits.

## Do
1. Pull every failing job's log (`gh api repos/CorbanuCore/CorbanuTerminal/actions/jobs/<id>/logs`), list each
   failing lint/build error and failing test with its cause, and classify: real bug, stale test/snapshot, environment
   (runner/network/sandbox), or flaky.
2. Fix on branch `codex/fix-rust-ci-full-20261004` in a new worktree off `origin/main`. Never weaken a test's intent;
   follow the repo's existing patterns for environment-dependent tests. Verify on Linux using the RTX box
   (`ssh ambient@100.99.88.49`, 32 cores; fresh clone under `~/rust-ci-full-20261004/`, cleaned up afterwards) with
   the same commands the workflow runs (read `rust-ci-full.yml` and `rust-ci-full-nextest-platform.yml`).
3. Open a PR to main. To exercise the full suite on the PR, push the branch name containing `full-ci` too if useful
   (`rust-ci-full.yml` has a `"**full-ci**"` push trigger), e.g. a mirror branch `codex/full-ci-20261004`.
4. Iterate until rust-ci-full passes or every remaining failure is explained. Cap: 4 CI rounds. Report; don't merge.

## Never
Never print secrets. `rm -f` is blocked by policy. No force-push to shared branches. Don't change CI timeouts.

## Final report (your last message)
Table: failure → cause class → fix (commit) → verified how; PR URL; final rust-ci-full status; what's left.
