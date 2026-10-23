# Worker brief: fix "Bazel test on windows-latest … (native main)" on main

You are an Opus 5.5 High worker. Work autonomously; do not ask questions.

## Context
- Repo CorbanuCore/CorbanuTerminal. After today's CI repair merges, main's `blocking-ci` (run 37165973285 on
  `91ed4ab8a1`) fails only one job: `Bazel / Bazel test on windows-latest for x86_64-pc-windows-gnullvm (native
  main)` (job 111334572714), which runs only on pushes to main (`test-windows-native-main` in
  `.github/workflows/bazel.yml`). It has failed on every main push since at least 2026-09-22.
- Failure: `ERROR: .../rules_rs++crate+crates__castaway-0.2.4/BUILD.bazel:5:11: Compiling Rust rlib castaway v0.2.4
  failed … error[E0463]: can't find crate for 'rustversion'` (proc-macro dependency not visible on the native
  Windows host build), then `CI job left tracked changes or untracked files in the repository worktree`.
- Full log: `gh api repos/CorbanuCore/CorbanuTerminal/actions/jobs/111334572714/logs`.

## Do
1. Root-cause it: how rules_rs / the crate universe wires proc-macro deps for exec vs target configuration on
   native Windows MSVC (compare with the passing `test-windows-shard` gnullvm jobs and `test-windows-native-msvc`
   product build in the same workflow), and what the clean-worktree check flagged.
2. Fix minimally (MODULE.bazel / crate annotations / platform config / workflow), on branch
   `codex/fix-windows-native-main-20261004` in a new worktree off `origin/main`. Verify with a debug run: a
   temporary workflow copy with a `push` trigger limited to a `debug/win-native-*` branch (delete it afterwards).
3. Open a PR to main with root cause and evidence; report when its checks pass. Do not merge.

## Never
Never print secrets. `rm -f` is blocked by policy. No force-push to shared branches. Don't change CI timeouts.

## Final report (your last message)
Root cause, fix, debug-run evidence, PR URL and check status.
