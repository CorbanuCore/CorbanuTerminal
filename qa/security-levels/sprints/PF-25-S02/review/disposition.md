# PF-25-S02 review disposition

Round 1 (Opus 5.5 High, APPROVE WITH FIXES):

1. "Kill switch on" reported when it is not: fixed. Core returns "changed" when the switch did not end up on (a newer
   event elsewhere won), and "not saved, nothing changed" when there is no live tree and the save failed.
2. "Restart now" after an unsaved change: fixed. `r` is offered only after a saved Core change (not after a single
   grant or a failure).
3. Release not tied to the switch seen without a live tree: fixed. `LevelBasis` now includes the kill-switch event in
   force, so a switch turned off and on again since the review is refused. Test added.
4. Release did not reach other sessions: fixed. A release propagates to the other trees of the process on the home
   (a newer switch there stays, as the merge keeps the newest event). Test added.
5. `k` clashed with the default "move up" key: the view now opens with `g`, and only when no list binding uses it.
6. Selection by row number: fixed. The selected row is kept by what it is (a grant by id); a grant that ended is
   reported instead of opening another row. Test added.
7. No scrolling: fixed. The list keeps the selected row visible; a review shows its end (its choices).
8. Revoke all and the kill switch now also clear every grant of the process (`aggressive::revoke_everything`).
9. Revoking one grant is logged (grant id, thread).
10. Tests added: grant rows and single-grant review, "already ended", selection follows the grant, the background
    commit and `poll`, `g` from the picker and the kill-switch status line, the view inside `/security` (opens and
    Esc returns), release propagation and the reviewed switch, unsaved without a session, revoke all clears every
    grant.
11. For Travis (not changed): with `security_levels` off there is no `/security` picker, so a saved kill switch cannot
    be turned off from the TUI (it can by editing nothing else: only a later build with the flag). Under Permissive with
    no grants and the switch off, `g` is not offered.

Round 2 (Opus 5.5 High, APPROVE WITH FIXES):

1. "Nothing changed" when another session took an unsaved change: fixed. `propagate` reports whether another tree
   took the change; then the result says it applied to running sessions and was not saved. Test added.
2. A release reaching a different switch: fixed. A release reaches only trees whose kill switch in force is the
   released event; others keep their switch and epoch. Test added.
3. A kill switch that lost a race: grants of the process are cleared first, and the view says authority was revoked
   but the switch was changed elsewhere (review it again).
4. Restart offered next to an error: fixed (`restartable` needs an Ok message and a save).
5. `i` (PF-41-S01) now has the same keymap guard as `g`.
6. A tall review starts at its end: known limit (the choices stay visible; panes are normally tall enough).
