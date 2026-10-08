**Verdict: APPROVE WITH FIXES.** I found no way for a model, tool, agent or app-server request to trigger, block, undo or release a revocation or the kill switch. The fixes needed are in how results are reported and in tying a release to the switch the person actually reviewed.

**What checked out**
- **Who can call it:** the only path in is a human key press in `/security`. The source-scan test confirms the TUI view is the only caller outside `core/src/security`.
- **Timing guards:** Esc and Ctrl+C are ignored while saving, so the commit finishes and its result is shown. Pressing Enter twice cannot turn the kill switch off.
- **Level:** turning the kill switch off never lowers the level. A revocation does not save a level of its own.
- **Same-second fix:** the new timestamp is one second after the later of the live and saved kill-switch events.
- **Live races:** in the live path, the epoch from the review catches any change made after the person reviewed it.

**Findings**

1. **Medium — "Kill switch on" can be reported when it is not on.** `revocation_change.rs:97-123`, `revocation_view.rs:402-425`.
   - (a) The save time is read before the store lock, so another process can write a newer kill-switch event first. The policy crate then quietly ignores this "on" event (`apply` returns `Ok(false)` in `revocation.rs`), and the switch stays off.
   - (b) With no live session tree, a failed save means nothing persisted it, yet the message says it is on "for the next start".
   - `outcome_line` only double-checks the "off" case.
   - **Fix:** in `commit_human_revocation`, return an error if the choice was "on" but `!committed.kill_switch_active`. When `!live && not_saved.is_some()`, report "not applied" unless another tree took the change. Add tests for both.

2. **Medium — `r` (restart) is offered after a change that could not be saved.** `revocation_view.rs:201,358-361`. A kill switch that applied but wasn't saved lasts only until exit, so restarting throws it away. The restart footer also appears after errors and single-grant revokes.
   - **Fix:** keep the `RevocationReport` in `Screen::Done` and offer `r` only for `Ok` results with `not_saved == None`.

3. **Medium — with no live session, turning the switch off isn't tied to the switch the person saw.**
   - `LevelBasis` only records `kill_switch_active`, with no event id, and the epoch is read fresh at commit (`revocation_change.rs:88-92`).
   - If another process turns the switch off and back on (a new event) between the review and Enter, the bases still match. The merge check compares against state recovered after the review, so this release turns off a switch the person never saw.
   - **Fix:** add the kill-switch event id and revocation generation to `LevelBasis` (from the live tree or the saved state) so the equality check catches it. Add a test.

4. **Low–Medium — turning the switch off doesn't reach other sessions, but the message says it does.**
   - `transition.rs:381` doesn't propagate the release to other trees. Yet `outcome_line` says "in this session and the others of this process".
   - Those other sessions keep the switch on. Their own attempt to turn it off fails every time with "changed since you reviewed" (merge id mismatch) until a restart.
   - **Fix:** either propagate the release to trees whose kill-switch event id matches the released one (a product decision), or correct the wording. Add a test either way.

5. **Low — `k` clashes with the default "move up" key.** `security_level_picker.rs:274`.
   - The default `ListKeymap.move_up` includes plain `k` (`keymap.rs:1141`). So in the picker, `k` stops moving up whenever the grants view is offered, and the behaviour flips with state.
   - Inside the view, `k` also selects "Turn it off" on the off review.
   - **Fix:** use a key that isn't bound in `ListKeymap`, or check for conflicts against the user's keymap.

6. **Low — the selection is by row number across a refresh.** `revocation_view.rs:163-174`. If a grant expires or is used up between drawing and Enter, the rows shift. "Revoke all" can then open the kill-switch review, or grant A can open grant B. Every choice still has a review, so nothing commits silently.
   - **Fix:** remember the selected row's identity (grant id or row type) before `refresh()`, find it again afterwards, and show a note if it's gone.

7. **Low — the grants view can't scroll.** `security_view.rs:270,286` skips scrolling while this view is open. With many grants (up to 64 per session) or a short pane, the selected row or the Back / "Turn it off" markers can be cut off.
   - **Fix:** add a scroll offset that keeps the selection and those options visible, with a footer hint.

8. **Low — "Revoke all" and the kill switch only clear grants of session trees registered on the home.** `adopt` calls `revoke_all` per agent, while `held_grants()` and the review text cover the whole process.
   - **Fix:** after a successful revoke-all or kill-switch-on, also clear the process ledger and check that `held_grants()` is empty. Test with a grant on an unregistered thread.

9. **Nit — revoking one grant leaves no audit trace.** `aggressive.rs:317`. Add a `tracing::info!` line (grant id and thread), matching how transitions are logged.

10. **Tests — gaps:**
    - No TUI test for a list with grants or for revoking a single grant.
    - No test of how the view sits inside `SecurityView`: `k` opens it, Esc returns to the picker with the basis re-read, Ctrl+C is ignored while saving.
    - The background-thread and `poll` path is never run, because `commit_inline = cfg!(test)`.
    - Missing tests for #1, #3 and #4.

11. **For product to confirm:**
    - With `security_levels` off, there is no picker, so no `k` and no kill-switch status line. A saved kill switch then can't be turned off from the TUI.
    - Under Permissive with no grants and the switch off, the kill switch isn't offered, even though it is the emergency stop that also closes broker channels.