The E0463 compile error is fixed in PR #152, but the native-main lane still won't pass. In the debug run it compiled cleanly and then hit the 40-minute timeout while running tests, with real Windows test failures among the ones that finished.

**Root cause**
- rules_rs 0.0.102 only ships a Windows rustc built for the MSVC host. Its toolchain checks only OS and CPU, so it also gets picked for the native gnullvm host (`//:local_windows`).
- On that host, proc macros like `rustversion` are compiled for gnullvm. The MSVC rustc then refuses to load them, so `castaway` fails with E0463. The job's execution log shows the `.dll` was built and passed in; rustc just rejects it.
- Local `bazel aquery` confirms it: a gnullvm host compiles `rustversion` with `--target=x86_64-pc-windows-gnullvm` using the MSVC-host rustc.
- The clean-worktree step in that job actually passed. The compile error is the only failure.
- Other Windows lanes aren't affected: the cross shards build tools on Linux, and the MSVC product job uses Cargo.

**Fix (`MODULE.bazel` only)**
- Added a stable 1.95.0 toolchain whose rustc runs on gnullvm, limited to gnullvm exec and target and to the stable channel. It copies the nightly gnullvm setup that the Windows argument-comment lint already uses.
- Limited the two existing nightly Windows toolchains to the nightly channel; they previously matched any channel.
- Registered `@rust_toolchains` before `@default_rust_toolchains` so the new toolchain wins on the native gnullvm host. Local aquery shows MSVC, Linux and nightly builds still resolve to the same toolchains as before.

**Debug-run evidence** (temporary workflow on `debug/win-native-*`; workflow and branches deleted)
- **Run 37170421744:** switching the build host to MSVC (`--windows-msvc-host-platform`) fails, with either a gnullvm target or the MSVC target upstream uses. MSVC-style link arguments get passed to clang++ (`clang++: error: no such file or directory: '/NOLOGO'`).
- **Run 37171276000** (exact copy of the job, with this fix): no E0463. Everything compiled, including `castaway`, and 82 of 403 tests ran before the 40-minute timeout.

**Still blocking the lane (not fixed here, needs a decision)**
1. **Capacity:** there is no `BUILDBUDDY_API_KEY`, so the 4-core `windows-2022` runner builds everything from scratch (about 25 minutes) and can't finish 403 tests in 40 minutes. Upstream runs this job on large self-hosted runners with BuildBuddy. The options are providing the key, using a larger runner, or narrowing what the lane tests.
2. **Real Windows test failures:**
   - `utils/pty`: ConPTY tests.
   - `uds`: "DACL omitted current user", probably because the runner runs as admin.
   - `wallet` and `wallet-daemon`: "Access is denied (os error 5)".
   - `codex-api`: a timing flake in `comment_only_stream_survives_actionable_silence_timeout`.

   Also, without the key, the CI script doesn't apply `--config=ci-windows`, so its existing skip list (which already covers the ConPTY test) isn't used.

**PR:** https://github.com/CorbanuCore/CorbanuTerminal/pull/152 (not merged). The required checks (`CI required`, `rust-ci / CI results (required)`), the Windows gnullvm aggregate check, all 4 shards, the MSVC product build and the Windows argument-comment lint pass: 24 passed, 0 failed. Six long Cargo release, tmux and CodeQL jobs were still pending; Bazel config changes shouldn't affect them. The native-main job itself only runs on pushes to main, so it was skipped on the PR.