**Verdict: APPROVE.** All five round-2 fixes hold, and I found no security regression. There are three small findings below (two low, one nit). None of them blocks.

**Round-2 findings, checked:**
- **#1 "Nothing changed" with no live tree:** fixed. `propagate` now returns how many trees took the change. `reached_other_trees` is set for Restrictive, Unchanged and release commits. `NotSaved` is returned only when there is no live tree, no other tree took the change and the save failed. Otherwise `report.live` is true. The new test `..._unsaved_but_taken_by_another_session_is_reported` covers this.
- **#2 Release reaching a switch the person never saw:** fixed. `only_with_switch` is this tree's own kill-switch event (`memory`, read before the commit). Trees holding a different switch are skipped before `adopt`, so their epoch and grants are untouched. `kill_switch_event_id` returns the last kill-switch event, which a release requires to be an active "on", so the filter is correct. The test also checks that the other tree's epoch did not move.
- **#3 Kill switch that lost a race:** fixed. `revoke_everything()` now runs before any early return. `outcome_line` returns an error saying authority was revoked but another session changed the switch first, and restart is not offered.
- **#4 Restart offered next to an error:** fixed. The rule is now `restartable = message.is_ok() && not_saved.is_none()`.
- **#5 `i` without a keymap check:** fixed. It is checked against move-up, move-down and accept, and `security_view_tests.rs:99-103` covers it. The order is still inspector, then grants view, then `i`, then picker.

**Findings**

1. **Low (regression): "nothing changed" can be false again.** `core/src/security/revocation_change.rs:128` and `:137`.
   - **Problem:** `revoke_everything()` now runs before the `NotSaved` check. With no live tree, no other tree and a failed save, revoke-all or kill-switch-on still clears every grant in the process, including grants the view listed for threads outside any tree. The error still says "nothing changed".
   - **Fix:** in that branch, say that grants were ended but the change was not saved, or return `Ok` with `not_saved` set. Moving the check back above `revoke_everything()` would also work, but it would undo the round-2 #3 fix for that path.

2. **Low: no test for the lost-race path.** `revocation_change_tests.rs` and `revocation_view_tests.rs`.
   - **Problem:** nothing tests the case where kill-switch-on is superseded and the report comes back `Ok` with `kill_switch_active == false`. Nothing checks the new error text, `restartable == false`, or that the grants were cleared.
   - **Fix:** add a unit test of `outcome_line` for that report, plus a Core test that writes a newer "off" event to the saved file before the commit.

3. **Nit: wrong wording when only other sessions took the change.** `tui/src/security/revocation_view.rs:507`.
   - **Problem:** when `live` is true only because of `reached_other_trees`, there is no tree for this session, yet the result reads "in this session and the others of this process".
   - **Fix:** add a separate field to `RevocationReport` for "other sessions only" and say "in the other running sessions of this process".

Still open for product, as before: with `security_levels` off, a saved kill switch can't be turned off from the TUI. The round-2 #6 limit is also unchanged: a tall review starts at its end with no "↑ more" hint.