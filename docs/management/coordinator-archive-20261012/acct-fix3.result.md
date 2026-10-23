#308 is fixed and merged. In developer-accounting builds, `corbanu delete --force` now works with the clock behind the ledger checkpoint. If the cost records can't be updated, the session is left untouched.

**What shipped:** PR #322, merged at `ccc38bfc03`. #308 closed itself on merge, and I added an evidence comment to it.
- **Cause:** delete removed the rollout files first, then failed in the step that updates the cost records, because the clock was behind the checkpoint. It also read the plain system clock, not the time rule #299 added.
- **Clock:** delete now uses the same helper as #299 (`AsOf::Now.sample_on`), so there's still only one notion of clock skew. While the clock is behind, delete uses the checkpoint time. Test-only explicit clocks still reject backward times.
- **All-or-nothing:** before removing any rollout, `thread/delete` runs the same database delete as a trial and rolls it back. If that fails, nothing is deleted and the error reads: "could not delete conversation X: its local records cannot be updated right now, so it was left as it is. Try again; if it keeps failing, report this message."
- **Why a trial run instead of one atomic step:** keeping the database write lock open while rollouts are removed would block other writers for a long time, because removal scans every rollout and the store may be remote.
- **Remaining gap:** a failure that appears between the trial and the real delete can still leave a half-delete. That error says the conversation was deleted and to "Delete it again to finish"; a retry completes it.
- **Deleted spend:** still counted under "deleted conversations", as #303 does.

**Gate evidence:**
- **New tests:**
  - State: delete with the clock 58 s and 6 days behind succeeds for a conversation plus a subagent. The checkpoint never goes backward and the deleted-conversations count is 2. A second test confirms a failed trial changes nothing.
  - App-server: the same two clock cases succeed and remove the rollout. With an injected ledger failure, the rollout and cost records stay, the error is the one above, and a retry succeeds once the fault clears.
- **Bug reproduced:** with the fix reverted, both app-server tests and the state clock test fail. The clock tests fail with the original "negative or backward checkpoint" error.
- **Focused tests:** all codex-state tests (364 unit tests plus integration) pass. The 7 app-server `thread_delete` tests pass with and without `codex-core/developer-accounting`. The 93 codex-core accounting tests pass with the feature. I used the existing test clock injection (checkpoint set ahead of real time); no libfaketime.
- **Linux clippy:** clean with `-D warnings` on the RTX box for codex-state and codex-app-server, with and without the feature. I re-ran it after the review fixes and after merging main.
- **Review:** Opus 5.5 High approved it (`.codex-work/workers-20261002/acct-fix308-review/review-result.md`). I fixed four of its five non-blocking points: error wording, error reporting when rollback fails, a subagent test, and a check that the test clock is still behind when the delete is sent. The fifth, performance, is #324.
- **CI and merging main:** CI was all green on the final head. I merged origin/main before merging; it included #318's changes to the shared state accounting files, and I re-ran the tests and clippy afterwards. I didn't touch any codex-tui files.

**Still open:**
- **#324 (filed):** in developer-accounting builds, delete now holds the database write lock about twice as long, because of the trial run. Running the cleanup step once per delete instead of once per subagent would more than make up for it.
- **Dev-dependency added:** `sqlx` is now an app-server dev-dependency so the test can inject the ledger failure. `Cargo.lock` gains one line; I didn't check Bazel.

**Cleanup:** my build target, worktree and branch are removed locally. The RTX worktree, both target dirs and my scripts and logs are removed. No real keychain was touched: every run used `CORBANU_TEST_NO_NATIVE_KEYRING=1` and temporary homes.