Both fixes are merged with every check green and none pending, and #493 and #494 are closed with evidence comments.

**What shipped**
- **#496 → #493 (Linux sandbox), merged `3f4537b566`.**
  - Under Moderate or Aggressive with `danger-full-access`, writing to `/dev/null` works again.
  - After the writable `/` bind, the sandbox now mounts a fresh minimal `/dev`. This happens before that step's protections are applied and before any writable folders under `/dev`.
  - The sandbox now sees less of the host's `/dev` than before: a short standard set instead of the full host device list. No protected path became readable, and a test confirms a protected `/dev` entry stays protected.
  - macOS Seatbelt doesn't have this bug: its base policy allows writing to `/dev/null` explicitly, and a new test covers that. Windows doesn't either, but that's from reading the code, not from a test.
- **#497 → #494 (exec), merged `b7320791df`.**
  - Once exec has started the app-server client, it now always shuts it down, including when it exits early on an error. So the plugin sync's `git` no longer outlives exec on those paths.

**Gate evidence**
- **Before/after tests:**
  - #493: the new test (real bubblewrap on glitch) fails on the old code with `/dev/null: Permission denied` and passes with the fix. The protected file stays unreadable in both.
  - #494: in the new exec test, `turn/start` fails while the fake `git` is running. It fails on the old code with "plugin sync git outlived codex-exec". With the fix it passed 5/5 on macOS and 3/3 on Linux.
- **Focused tests:**
  - Linux sandbox: 101 unit and 31 integration tests pass.
  - Exec: 61 unit and 85 integration tests pass on both macOS and Linux.
- **Clippy `-D warnings`:** clean on Linux (glitch) and on macOS for every changed crate.
- **#493 functional check:** fresh debug binary on glitch, disposable home, GLM 5.3 Flash.
  - Before, at both Moderate and Aggressive: `/dev/null: Permission denied`, rc=1.
  - After, at both levels: rc=0.
  - The protected file was unreadable in all four runs, and its dummy content never appeared in any output.
  - The Z.AI key file was deleted from glitch afterwards.
- **TUI quit check (gap from #492's review):** tmux on the Mac, fresh debug binary, disposable home, `CORBANU_TEST_NO_NATIVE_KEYRING=1`.
  - In 3/3 runs, `/quit` during an in-flight plugin sync exited in 0.04–0.14 s and left no `git` behind.
  - An exec run that failed early with the sync in flight exited in 0.65–1.12 s, also leaving no `git`.
  - The timings are recorded in #497.
- **Reviews (Opus 5.5 High, one per PR):** both returned APPROVE in round 1. I then made their small suggested changes to the tests, one code comment and the PR descriptions, and didn't run a second round.

**Opened**
- **#498 needs your decision:** the no-protection path of the Linux sandbox (full write without protected paths, network not fully open) has the same `/dev/null` bug. I left it alone because Permissive may use it. Fixing it would change what Permissive sees in `/dev`; I'd apply the same fix.

**Not done**
- No demo videos; your gate for this task didn't list them.
- The #493 reviewer noted an older edge case I didn't file: a writable root that is a symlink to `/` is ordered by its link path.

**Cleanup**
- Both worktrees, all build folders (Mac and glitch), the glitch clones, the disposable homes and the binaries are removed.
- 212 KB of logs remain in `~/corbanu-rtx/fix493-logs` on glitch.
- I kept the two TUI/exec check scripts as `fix-494-*-check.sh` next to the report.

Files are in `.codex-work/workers-20261002`:
- `fix-493-494.result.md`
- `fix-494-tui-quit-check.sh`
- `fix-494-exec-error-check.sh`