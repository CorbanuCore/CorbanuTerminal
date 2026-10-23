# Worker brief: make the nightly cold-Bazel lane (nightly-ci.yml) green

You are an Opus 5.5 High worker. Work autonomously; do not ask questions.

## Context
- Repo CorbanuCore/CorbanuTerminal. `nightly-ci.yml` (daily 09:17 UTC + manual dispatch) runs the cold Bazel jobs
  that no longer block PRs: Linux Bazel test (gnu, musl), Bazel clippy, release-build verify, the SDK Bazel build,
  Linux+macOS argument-comment lint, and the native Windows gnullvm test lane. 360-minute budgets; no BuildBuddy cache.
- Today's scheduled run `37210663414` on main `3ecc3c4` (just after the integration merge #138) already shows:
  - `rust-ci / Argument comment lint - Linux` — failure (likely positional literal arguments without
    `/*name*/` comments in code that #138 brought in; #148 annotated main's backlog before that merge);
  - `Bazel / Bazel test on windows-latest for x86_64-pc-windows-gnullvm (native main)` — failure. Known Windows
    failures from #152's investigation: `utils/pty` ConPTY tests; `uds` "DACL omitted current user" (runner is
    admin); `wallet` and `wallet-daemon` "Access is denied (os error 5)"; a timing flake in codex-api
    `comment_only_stream_survives_actionable_silence_timeout`. Also note the CI script doesn't apply
    `--config=ci-windows` without the BuildBuddy key, so its existing skip list isn't used.
  - Other jobs (Linux Bazel test gnu/musl, clippy, release verify, sdk, macOS lint) were still running — check them
    when they finish (`gh run view 37210663414 -R CorbanuCore/CorbanuTerminal`).
- Previous nightly-Bazel work: `.codex-work/workers-20261002/ci-bazel-test2.result.md`, `win-native-main.result.md`.
- PR #155 (rust-ci-full fixes) and a skills-dedupe PR are in flight; avoid their files where possible.

## Do
1. Fix the Linux argument-comment lint failures properly (add the `/*param*/` comments; follow the repo's
   argument-comment-lint conventions). Verify with the lint locally/Linux if feasible.
2. Fix the Windows native-lane failures at the root where they're real bugs; where a test is genuinely incompatible
   with the hosted runner (e.g. admin DACL), follow the repo's existing pattern (the `ci-windows` skip list) and make
   sure that config is actually applied in the keyless path. Don't weaken test intent.
3. When the run finishes, fix any other failing nightly job the same way.
4. Branch `codex/nightly-green-20261004` off `origin/main` in a new worktree; open a PR to main. Validate the nightly
   jobs by dispatching on your branch: `gh workflow run nightly-ci.yml -R CorbanuCore/CorbanuTerminal --ref <branch>`
   (it exists on main, so dispatch works). Cap: 3 dispatched rounds. Report when the PR's checks pass; don't merge.
5. Use the RTX box (`ssh ambient@100.99.88.49`) for Linux verification if needed (fresh clone under
   `~/nightly-green-20261004/`, clean up afterwards).

## Never
Never print secrets. `rm -f` is blocked by policy. No force-push. Don't change CI timeouts.

## Final report (your last message)
Table: job/test → cause → fix (commit) → verified how; nightly dispatch results; PR URL and check status.
