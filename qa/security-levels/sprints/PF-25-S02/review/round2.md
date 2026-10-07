**Verdict: APPROVE WITH FIXES.** Round-1 findings 1–10 are fixed. The new problems below are smaller and all come from one path: committing with no live tree for this session's thread (thread not started, or not registered on the home). That path still updates the other sessions in this process, and the fixes didn't account for it.

**Checked and fixed:**
- **#3 and PF-24-S02:** `LevelBasis` now includes the kill-switch event id. It is deterministic in both paths, including an unreadable saved state (`recover` adds no event), so PF-24-S02 level confirmations are only refused after a real change.
- **#4:** the release check against the saved file (`StoredStateChanged`) still holds.
- **#2:** restart is now gated on a saved Core change.
- **#6:** the selection is kept by grant id.
- **#7:** the scroll offset is computed from the same lines that are drawn.
- **#5:** `g` is checked against the move-up, move-down and accept keys.
- **Saving guards:** Esc and Ctrl+C are still ignored while a change is saving.
- **Merge resolution:** inspector first, then grants view, then `i`, then picker. `i` can't open over the grants view, and the picker's background save is still polled while the inspector is open.

**Findings**

1. **Low–Medium — "Nothing changed" can be false when there is no live tree.** `core/src/security/revocation_change.rs:122-127`, `transition.rs:359-369`.
   - The path that commits from the saved file (`stored_controller`) uses `HomeTransitionStore`, whose `home()` is `Some`. So a revoke-all or kill-switch-on that can't be saved is still applied to every live tree on the home before `NotSaved("…nothing changed")` is returned.
   - In that case the other sessions really are under the kill switch until exit, while the screen says nothing changed.
   - **Fix:** return `NotSaved` only if no other tree took the change (have `propagate` return a count). Otherwise report "applied to running sessions, not saved". This is the "unless another tree took the change" part of round-1 #1b.

2. **Low — the release can turn off a kill switch the person never saw.** `transition.rs:381-392`.
   - `propagate(..., only_if_stricter=false)` merges the release into every tree on the home, and the newest event wins.
   - With no live tree, the person sees only the saved switch. Another tree may hold a newer switch that was never saved (its save failed). The release is stamped at least one second after the newest event this commit can see. The other tree's unsaved event isn't one of them, so the release is usually newer and turns that switch off. In the same second, the winner depends on the event id.
   - Separately, every tree that receives the release has its epoch bumped and its grants cleared (`adopt`), even trees that never had this switch on.
   - **Fix:** apply the release only to trees whose `kill_switch_event_id()` equals the released event id. That is round-1 #4's suggested filter.

3. **Low — kill-switch-on that loses a race is reported as "Not changed", and the grants aren't cleared.** `revocation_change.rs:128-135`.
   - When the "on" event is superseded, `apply` still sets the all-authority revocation time, and the transition has already been adopted and propagated with channels closed. Authority was revoked, yet the view says "Not changed: … changed since you reviewed".
   - The early return also skips `revoke_everything()`, so grants on threads outside any tree survive.
   - **Fix:** call `revoke_everything()` once the commit succeeds (before the "on" check). Report "authority revoked; the kill switch was changed elsewhere, review again".

4. **Nit — restart can be offered next to an error.** `tui/src/security/revocation_view.rs:303-311`. `restartable` is `report.not_saved.is_none()` even when `outcome_line` returns `Err` (release reported, switch still on). That case is effectively unreachable now, but the rule should be `message.is_ok() && not_saved.is_none()`.

5. **Nit — `i` opens the inspector without the keymap check `g` has.** `tui/src/bottom_pane/security_view.rs` (`i` handling, from PF-41-S01). It is checked before the picker without the move-up/move-down/accept guard, so a user keymap that binds `i` loses that binding in `/security`. Apply the same guard as `g`.

6. **Nit — a tall review starts at its end.** `revocation_view.rs` `scroll_for`. In a short pane the review's explanation can be scrolled off while its choices stay visible, and nothing says it scrolled. Consider a "↑ more" footer hint, as the picker has.

Still open for product, unchanged from round-1 #11: with `security_levels` off, a saved kill switch can't be turned off from the TUI.