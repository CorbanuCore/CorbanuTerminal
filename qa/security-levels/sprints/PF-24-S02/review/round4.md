**Verdict: APPROVE.** All four round-3 fixes are correct, and I found no regressions. The two notes below are informational and don't block merge.

This was a read-only review of `git show HEAD` and the parts of `origin/main...HEAD` around it. I didn't build or run anything.

**Round-3 fixes checked**

1. **Next-start level after a downgrade to the same level (fixed):** `transition.rs:421-427` now updates `next_start_level` for any `Downgrade`.
   - **No regression:** only `SetLevel` can produce a `Downgrade`. `Revoke` produces `Restrictive` or `KillSwitchRelease`, so revocation commits still leave the next-start level alone.
   - **What gets saved:** the merge always saves `prepared.to` for a `Downgrade` and rejects it with `StoredLevelChanged` first if the stored level is above the floor, so the in-memory value matches the file.
   - **Test:** `security_transition_downgrade_of_the_record_sets_the_next_start` reproduces the exact case: an `Unchanged` merge leaves Permissive in force with Aggressive for the next start, then a `Downgrade` reports Permissive. It would fail on the old condition.

2. **The `Option` floor (fixed):** I checked both callers.
   - **`prepare_transition` (`None`):** nothing changes. `unwrap_or_default()` gives Permissive (`#[default]` in `security-policy/src/level.rs:30`), so the `Downgrade` test is `level < from` and the floor is `from`, same as before.
   - **`commit_human_level_change` (`Some(reviewed)`):** the floor is now the stored level the person reviewed. A stored level saved after the review, even one at or below `from`, is now refused instead of silently lowered. That's the round-3 finding 4 fix.
   - **Scope:** `floor` is only read in the `Downgrade` arm of `merge`, so `Unchanged`, `Restrictive` and `KillSwitchRelease` are unaffected.

3. **Restart marker (fixed):** `take_restart_marker()` is the first statement of `main()` in both `cli/src/main.rs:1097` and `tui/src/main.rs:51`. That's before `configure_for_current_process`, `arg0_dispatch_or_else` and the Tokio runtime start, so no other thread exists yet and the `unsafe remove_var` is sound.
   - **Callers:** `codex_tui::run_main` has only one caller (`cli/src/main.rs:2871`). The `codex-tui` binary also runs `take_restart_marker` first. So `restarted()` reading from the `OnceLock` can't miss the marker on a real restart.
   - **Tests:** none rely on the old read from the environment.

4. **Permissive review wording (fixed):** "Grants end" and "saved by another session" now appear only when the person is in a live session at Permissive and the stored level is above Permissive. That is exactly the case where `prepare_reviewed_transition` makes a `Downgrade` with a revocation.
   - **Absent, unreadable, or stored Permissive** while the next start is stricter: these commit as `Unchanged`, and the new wording promises no revocation, which is correct.
   - **Non-live session at Permissive:** this only reaches the commit path when the record is unreadable, and the "repaired" wording fits that case.

**Findings**

1. **Info: the mismatch message overstates what was restored.** `tui/src/security/confirm.rs:308`
   - **Problem:** the message says the level file "was put back to match it". In fact `snapshot.restore()` puts back the previous files, which may not match what Core saved. Example: the previous file was Permissive, but a race left Core at Aggressive.
   - **Reachability:** practically unreachable. `ConfirmLock` serializes `/security`, and revocation commits don't change the stored level. If it did happen, it would err on the strict side.
   - **Fix:** say "the previous level file was put back. Review it again".

2. **Info: the new review branch has no test.** `tui/src/bottom_pane/security_level_picker.rs:1066-1067`
   - **Problem:** round 3 asked for a snapshot test of the Permissive review when the record is absent or unreadable. None was added, and no existing test reaches either new branch. The corrupt-JSON test goes down the tamper path instead.
   - **Fix:** add two text or snapshot assertions:
     - live session at Permissive, stored Aggressive: expect "saved by another session".
     - live session at Permissive with an unreadable record (but not tamper) and Aggressive as the next start: expect "repaired if unreadable" and no "Grants".

The two partly-fixed round-1 items (#3 Windows Ctrl-C and #10 profile-v2 config path) are still recorded as follow-ups in `disposition.md`. Neither changed in this commit.