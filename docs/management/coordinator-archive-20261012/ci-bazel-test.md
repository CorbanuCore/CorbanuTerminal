# Worker brief: fix the Bazel test failures on the combined CI PR (#147)

You are an Opus 5.5 High worker. Work autonomously; do not ask questions.

## Context
- Repo: CorbanuCore/CorbanuTerminal (a Codex fork). PR #147 (`ci/combined-20261002`, worktree
  `/Volumes/CorbanuDrive/Corbanu/worktrees/ci-fixes-20261002`) is a trial merge of: #146 baseline CI fixes
  (`codex/ci-main-baseline-20261002`, worktree `/Volumes/CorbanuDrive/Corbanu/worktrees/ci-main-baseline-20261002`),
  fork PRs #131/#132/#133/#142/#143/#144/#145, #148 (arg-lint, worktree `worktrees/arglint-20261002`) and #149
  (broker fixture crate, worktree `worktrees/broker-fixture-20261002`).
- For the first time, `Bazel test on ubuntu-24.04 for x86_64-unknown-linux-gnu` ran to completion (163 min)
  instead of timing out: 298 of 403 tests executed, 11 targets fail. The full job log is at
  `/Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/bazel-test-gnu-37124901527.log`
  (run 37124901527, job 111208201952). The musl Bazel test and Bazel clippy jobs are still running; check
  them too with `gh pr checks 147 -R CorbanuCore/CorbanuTerminal` and include their failures when they finish.
- Failing targets: wine-test-support-unit-tests, app-server-transport-unit-tests, linux-sandbox-all-test,
  tui-all-test, tui-manager_dependency_regression-test, core-unit-tests, app-server-all-wine-exec-test,
  app-server-all-test, tui-unit-tests, core-all-test, core-all-wine-exec-test.

## Do
1. From the log, list every failing test with its panic and classify: (a) real product/test bug, (b) Bazel
   environment/data-dependency issue (missing runfiles, compile_data, sandbox/user-namespace, network), (c)
   flaky/timing. Decide for each whether it also fails on plain `main` (check whether the code involved came from
   main or from one of the merged PRs: `git log`/`git blame` in the combined worktree).
2. Fix at the source: problems that exist on main go on the #146 baseline branch; problems introduced by a
   specific PR go on that PR's branch if it is ours to push (fork PR branches may not be pushable — then put the
   fix on #146 and say so). Never weaken a test's intent to make it pass; if a test is genuinely incompatible with
   Bazel's sandbox, follow the repo's existing pattern for that (look for how other targets handle it in
   BUILD.bazel / `codex_rust_crate` attrs) and justify it.
3. Reproduce and verify on Linux. The RTX box `ssh ambient@100.99.88.49` (32 cores) has Rust; use a fresh clone
   under `~/ci-bazel-20261003/` and `cargo test`/`cargo nextest` for the affected crates, and Bazel there if it is
   installed (check `bazel --version` / `bazelisk`). Clean up what you create there.
4. Commit with clear messages, push the owning branches (normal push, never force), then merge them into
   `ci/combined-20261002` (fetch from https://github.com/CorbanuCore/CorbanuTerminal.git) and push it.
5. Watch the new CI round on #147 (poll every ~10 min with `gh pr checks 147 -R CorbanuCore/CorbanuTerminal`) and
   iterate on any remaining Bazel test/clippy/sdk failures until those jobs are green or only blocked by the
   180-minute timeout. Cap: stop after 4 CI rounds and report.

## Never
Never print secrets. `rm -f` is blocked by policy. No force-push. Do not merge any PR. Do not change CI
timeouts (that decision is pending with Travis). Do not touch `worktrees/management-workstreams-20260911`.

## Final report (your last message)
Table: test → cause class → fix (commit, branch) → verified how. Then the latest #147 check summary and anything
still failing with the reason.
