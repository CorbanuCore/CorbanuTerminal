Both fixes are merged with all checks green and none pending, and #473 and #477 are closed with evidence comments.

**What shipped**
- **#491 → #473 (Linux sandbox), merged `9cd9fc78c2`.**
  - Sandboxed commands no longer all fail at Moderate and Aggressive when `~/.claude` has no `.credentials.json`.
  - That folder is now replaced, for each command, by a read-only copy of what it held at launch. It stays readable, and a credentials file written while a command runs stays hidden from it (the TOCTOU requirement), shown with a real bubblewrap test.
  - A second failure is fixed too: several protected paths under a missing `~/.config` collided.
  - Paths that already exist are still protected as before.
  - macOS Seatbelt needs no change. On Windows this failure can't happen, so nothing changed there.
- **#492 → #477 (exec), merged `1b2a81a85a`.**
  - Shutdown now cancels the background plugin `git` clone and waits up to 3 s, and nothing it started survives exec.
  - This covers exec, the TUI and the app-server, forced shutdowns included.
  - A new exec test is deterministic: it uses a fake `git` and only answers the model once the fetch is running.

**Gate evidence (glitch)**
- **Before/after tests:**
  - #473's new bubblewrap test fails on the old code with the exact `Read-only file system` error and passes with the fix.
  - #477's new exec test fails on the old code ("plugin sync git outlived codex-exec") and passes with the fix: 10/10, and 15/15 together with `trace_log_credentials`.
- **Focused suites:**
  - Linux sandbox: 100 unit and 30 integration tests pass.
  - Plugins: 363/363.
  - App-server shutdown tests: 8/8.
  - `pf_30_s01` passes with every test home layout.
  - Full `codex-core` gives the same results before and after. 120 tests fail in every run, because I ran the test binary directly and some helper programs weren't built.
- **Linux clippy `-D warnings`:** clean for every changed crate.
- **Functional check:** fresh debug binary, disposable home, a `~/.claude` without `.credentials.json`, GLM 5.3 Flash.
  - Before: every command failed with the bubblewrap error at both levels.
  - After: the command ran and printed its output at both levels, re-checked on the final commit, and nothing was left behind in `.claude`.
  - The Z.AI key was deleted from glitch afterwards.
- **Reviews (Opus 5.5 High):**
  - #491: round 1 asked for changes. One fix there could have left a protected file writable in nested writable folders; I fixed it and added tests. Round 2 approved.
  - #492: approved in both rounds. I also addressed its suggested fixes.
- **Not done:** no tmux run or demo videos for either fix (the brief's gate didn't list them). The #492 reviewer also asked for a real TUI run, because quitting can now take up to 3 s longer while a sync is stopping; I didn't do that one.

**Opened**
- **#493:** in the same Moderate/Aggressive setup, writing to `/dev/null` fails ("Permission denied") in every sandboxed command, which pollutes shell output. It already happens on main and isn't fixed here.
- **#494:** some error exits in exec skip shutdown, so a plugin `git` can still survive on those paths. This is older code.

**Cleanup**
- Glitch: my clones, build folders, homes and binaries are removed; 17 MB of logs remain in `~/corbanu-rtx/fix473-logs`.
- Mac: both worktrees are removed and no build folders were created. Git won't delete the two local branches with a normal delete because local main hasn't fetched the merges, and I didn't force it.

This report is also saved as `.codex-work/workers-20261002/fix-473-477.result.md`.