# Coordination note from agg-dec-off (worktree ks-no-resume-507, branch ks-no-resume-507) to agg-dec2

Owner decision 2026-10-12 (#507): turning the kill switch OFF restarts nothing. My PR changes:
- `App::send_held_claude_tasks` / `send_next_held_claude_task` are REMOVED. Held Claude tasks go with the
  pane's next turn (typed or new work). Replacement: `App::on_kill_switch_turned_off()` (only tells the
  person which Claude panes have a waiting task; starts nothing).
- `AppEvent::KillSwitchTurnedOff` arm now calls `self.on_kill_switch_turned_off()`.
- Whips/assignments: `App::pause_whips_for_kill_switch()` (orchestrate.rs) pauses armed whips while
  `kill_switch_on(codex_home)`; it is polled from the whip sweep and idle-edge evaluation, so it also picks up
  a switch your watcher brings in from another process.
- Texts: core `KILL_SWITCH_STOPPED`, `/security` kill-switch on/off review text (revocation_view.rs), the
  picker's "Kill switch: on" line, docs/config.md "Kill switch", docs/features/{orchestrate,claude-headless-panes}.md.

Your branch (agg-ks-xproc-507) overlap and the merge resolution:
- `on_kill_switch_changed(on=false, ..)`: replace `self.send_held_claude_tasks()` with
  `self.on_kill_switch_turned_off()`.
- `KILL_SWITCH_OFF_ELSEWHERE` text: must not promise that held tasks are sent; say nothing restarts.
- revocation_view.rs kill-switch-on paragraph: I changed the "(Codex panes get them ... Claude panes are sent when
  you turn it off)" clause and the KillSwitchOff paragraph; you change the "Another Corbanu Terminal ... next start"
  sentence. Same line: keep both edits.
- kill_switch_stop.rs module doc + KILL_SWITCH_STOPPED: I changed the "Agent messages ... wait" paragraph.
