# #660: the kill switch's event-queue window for Codex panes (#613 follow-up)

Owner decisions: #507 (Travis, 2026-10-12): nothing restarts after the kill switch is turned off; Claude panes behave
like Codex panes. #657 closed this window for Claude panes; this closes it for Codex panes.

| Item | Before | After |
| --- | --- | --- |
| A task another agent sent to a managed Codex pane (`/spawn`) while the switch was on, handled after switch-off | admitted with `trigger_turn: true`: started a turn | asks for no turn; it goes with that pane's next turn ("A task for … waits: it was sent while the security kill switch was on …") |
| The same for a Codex pane the person opened in `/panes` | submitted as a user turn | reported "Not sent to …: it was sent while the security kill switch was on …" |
| The sender of such a refused task, or of a held Claude-pane task that fails (missing pane, superseded) in that window | its report started a turn in the sender | the sender is told, with no turn |
| A held task for a managed Codex pane while the switch is on | admitted with `trigger_turn: true` (Core held it) | admitted with `trigger_turn: false`, so a later wake or restart doesn't start it either |
| The switch-off in another process not yet handled here (about 1 s) | saved state only | the state this process last learned also holds the task (`kill_switch_on_now()`), as for Claude panes |
| Permissive, kill switch never used | — | unchanged |

## Evidence

- Regression tests (`codex-tui`): `app::tests::dispatch_integration::issue_660_*` (real embedded app server and mock
  provider) and `app::tests::issue_660::*`. All five fail with the fix reverted and pass with it.
- tmux (GLM 5.3 Flash on zai, debug build, disposable home): Aggressive, a Codex Troll gets a task while the switch is on
  ("waits"); switch-off starts nothing (no file after 20 s, Troll idle); a person types in the Troll and the task goes
  with that turn. The millisecond event-queue window itself can't be hit by hand; the tests cover it.
