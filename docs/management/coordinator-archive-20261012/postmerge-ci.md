# Worker brief: fix postmerge-ci startup_failure on main

You are an Opus 5.5 High worker. Work autonomously; do not ask questions.

## Context
- Repo CorbanuCore/CorbanuTerminal. Every push to `main` since 2026-09-12 has produced a `postmerge-ci` run with
  conclusion `startup_failure` ("This run likely failed because of a workflow file issue"), e.g. run 37165973398 on
  `91ed4ab8a1`. So main's post-merge suite (`rust-ci-full.yml`, `v8-canary.yml` via `workflow_call`) never runs.
- `postmerge-ci.yml` calls `rust-ci-full.yml` (which itself calls `rust-ci-full-nextest-platform.yml`) and
  `v8-canary.yml`, then a results job using `.github/scripts/check_ci_results.py`. Repo default workflow token
  permissions are `read`.
- The new `nightly-ci.yml` (calls `bazel.yml`, `rust-ci.yml`, `sdk.yml` with `cold_bazel: true`) does start fine
  (run 37166293445), so compare against it.

## Do
1. Find the exact startup error. Try: the GitHub web UI's annotation text via the API (`gh api graphql` on the
   workflow run / check suite), `actionlint` (installed via brew) on all involved workflows, and bisecting with a
   throwaway branch + `workflow_dispatch`-able copy of postmerge-ci (e.g. a temporary `postmerge-ci-debug.yml` on a
   branch named `debug/postmerge-ci-*`; dispatch with `gh workflow run --ref` only works for workflows on the default
   branch, so use a `push` trigger restricted to your debug branch instead). Known suspects: nested reusable
   workflow limits, `permissions` requested by called jobs exceeding the caller's, `secrets: inherit` across nesting,
   undefined inputs, `environment` use in called workflows, concurrency groups that reference `github.workflow`.
2. Fix it minimally on a branch `codex/fix-postmerge-ci-20261004` in a new worktree
   `/Volumes/CorbanuDrive/Corbanu/worktrees/fix-postmerge-ci-20261004` off `origin/main`, prove a debug run starts
   and its jobs begin executing, then delete the debug workflow/branch.
3. Open a PR to main with the root cause and evidence. Run `python3 -m unittest discover -s .github/scripts -p
   'test_*.py'` (4 setup-telegram failures are a known macOS bash issue). When the PR's checks pass, report — do not
   merge.

## Never
Never print secrets. `rm -f` is blocked by policy. No force-push to shared branches. Don't change CI timeouts.

## Final report (your last message)
Root cause (with the exact error), the fix, evidence that the workflow now starts, PR URL and check status.
