# Worker brief: continue fixing the Bazel test failures (round 2)

You are an Opus 5.5 High worker continuing a previous worker that died mid-task from an unrelated harness error
("Native Anthropic accounting failed"). Work autonomously; do not ask questions.

Read first:
- The original brief: `/Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/ci-bazel-test.md` (its Do/Never rules
  apply, except where changed below).
- The previous worker's log, `/Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/ci-bazel-test.log` (67k
  lines; read the last ~1500 lines and grep its `codex` messages to recover its findings/plan), and the commits it
  made: `git log --oneline origin/main..origin/codex/ci-main-baseline-20261002` in
  `/Volumes/CorbanuDrive/Corbanu/worktrees/ci-main-baseline-20261002`.

## What changed
- Travis decided the cold Bazel jobs run **nightly** instead of on every PR: Linux Bazel test (gnu, musl), clippy,
  release verify, Linux+macOS argument comment lint and the `sdk / sdks` Bazel build are gated by a `cold_bazel`
  input and run from `.github/workflows/nightly-ci.yml` (commits `4d0351a6be`, `0d48f7ea7a` on the #146 baseline
  branch, merged into #147). They no longer appear in PR #147's checks.
- To verify Bazel fixes, use the manually dispatched Bazel workflow on the combined branch:
  `gh workflow run bazel.yml -R CorbanuCore/CorbanuTerminal --ref ci/combined-20261002 -f cold_bazel=true`.
  Run `37145287607` (dispatched by the previous worker on `b218a434f6`) may still be running — check it with
  `gh run view 37145287607 -R CorbanuCore/CorbanuTerminal` before dispatching another; a new dispatch on the same ref
  cancels the previous one (shared concurrency group). `nightly-ci.yml` can't be dispatched until it's on main.
- The CI timeout ban from the original brief still applies.

## Goal
The nightly cold Bazel jobs (test gnu/musl, clippy, release verify, SDK build, Linux/macOS arg lint) pass on the
combined branch, or every remaining failure is explained with a concrete reason. Keep PR #147's own checks green.
Cap: 3 more dispatched Bazel rounds, then report.

## Final report (your last message)
Table: test → cause class → fix (commit, branch) → verified how; the dispatched-run results; #147 check summary;
anything still failing with the reason.
