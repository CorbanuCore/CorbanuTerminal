# #507: turning the kill switch off restarts nothing (2026-10-12)

Travis's decision (2026-10-12): after the kill switch is turned off, nothing restarts automatically. Claude panes
behave like Codex panes: no turn starts on switch-off, and a waiting task goes with that pane's next turn (a person
types in it, or it gets new work). Senders are still told when a waiting task is superseded.

## Live check (tmux, GLM 5.3 Flash; Opus 5.5 Claude Plan for the Claude pane)

Setup: a private tmux socket, a disposable home with `CORBANU_TEST_NO_NATIVE_KEYRING=1`, the "Aggressive ships"
feature flags (contained Claude panes on), the real Z.AI key and the `claude-plan-test-token` from the vault helper on
the launch command only, a disposable `CLAUDE_CONFIG_DIR`, Claude Code 2.1.296 in a scratch folder, and
`[agents] provider_allowlist` naming the Claude Plan route so a Claude Orc can join the crew. Scripts:
[launch.sh](scripts/launch.sh), [lib.sh](scripts/lib.sh), [case.sh](scripts/case.sh).

A `/spawn` crew: Nazgul Angmar and Troll Burzum (native, GLM 5.3 Flash) and Orc Snaga (Claude Code, Opus 5.5 Claude
Plan). With the switch on, `/spawn status` → "Send task to …" sends Burzum "create codex-task.txt containing
pf507-codex-task" and Snaga "reply with exactly pf507-claude-task". Then the switch goes off and the run waits 45 s.

Builds (macOS arm64, debug): **before** `c4999b1265` (`origin/main` at the branch point); **after** this branch at
`4f41d8b859` (the after captures are from that final build).

Video (SOP, leak-scanned): [issue507-switch-off-restarts-nothing](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/issue-507-issue507-switch-off-restarts-nothing-4f41d8b859bc-2026-10-12.mp4)
([spec](../../../demos/specs/issue507-switch-off-restarts-nothing.toml)).

| Step | Before | After |
| --- | --- | --- |
| Tasks sent while the switch is on | the Claude task waits; nothing said for Burzum ([capture](captures/before-02-tasks-wait.txt)) | both wait, each with a "Turning it off starts nothing" line ([capture](captures/after-02-tasks-wait.txt)) |
| Switch off | Snaga's task is sent at once ([capture](captures/before-04-switch-off.txt)) | "Nothing restarted. A task waits for … Snaga" ([capture](captures/after-04-switch-off.txt)) |
| 45 s later | Snaga ran a turn (1 turn file, result "pf507-claude-task"); Burzum shown "running" though idle ([status](captures/before-05-status-45s-after-off.txt), [files](captures/before-05-files-45s-after-off.txt)) | nothing ran: 0 Claude turn files, no codex-task.txt, both idle ([status](captures/after-05-status-45s-after-off.txt), [files](captures/after-05-files-45s-after-off.txt)) |
| A person types "Go ahead." in Burzum | | the waiting task goes with that turn; codex-task.txt is created ([capture](captures/after-06-troll-typed.txt), [files](captures/after-06-files-after-troll.txt)) |
| A person types "Hello." in Snaga | | "Sent with this turn: the task that waited …"; Claude replies pf507-claude-task ([capture](captures/after-07-claude-typed.txt)) |
| Kill-switch texts | | on: "Turning it off later restarts nothing: work resumes only when someone acts." ([capture](captures/after-01-kill-on-review.txt)); off: "nothing restarts on its own …" ([capture](captures/after-03-kill-off-review.txt)) |

## Paths audited (each has a test)

| Path | What happens at switch-off | Test |
| --- | --- | --- |
| Claude pane, task from another agent | waits; goes with the pane's next turn (typed or new work); a refused turn keeps it | `app::tests::issue_507_switch_off_starts_no_claude_pane_turn`, `…_waiting_claude_task_goes_with_the_next_turn`, `…_survives_a_refused_turn` |
| Claude pane, a turn ending after switch-off | does not send the waiting task | `issue_507_switch_off_starts_no_claude_pane_turn` |
| Claude pane removed while its task waits | the task is dropped and its sender told | `issue_507_removed_pane_drops_its_waiting_task` |
| Report of a Claude turn the switch stopped, finishing after switch-off | reaches the parent, starts no parent turn (no auto-processing; mailbox mail without `trigger_turn`) | `issue_507_report_of_a_stopped_claude_turn_starts_no_parent_turn` |
| Codex pane, agent message (mailbox, `trigger_turn`) | no turn; goes with the next turn | `suite::issue_507_switch_off::issue_507_switch_off_does_not_start_a_turn_for_waiting_agent_messages` |
| Spawned child agent's mailbox | no turn in the child | `…_does_not_start_a_child_agent` |
| Durable sleep (timer) woken by mail | not woken | `…_does_not_wake_a_durable_sleep` |
| Automatic idle turn (goal continuation) | refused while on (was: a refused turn, which blocks the goal); nothing after | `…_no_automatic_idle_turn_while_the_switch_is_on` |
| `/orchestrate` assignments and whips (45 s sweep, idle edges) | paused when the switch goes on (and at the next sweep or idle edge if it came from elsewhere); stay paused until `/orchestrate resume` | `app::tests::issue_507_whips_pause_and_stay_paused_after_switch_off`, `…_switch_events_pause_whips_at_once_and_start_nothing` |
| The switch-off event itself | sends no op, turn, task or agent message | `issue_507_switch_events_pause_whips_at_once_and_start_nothing` |
| Task for a Codex pane the person opened (`/panes`), while on | reported as not sent before the session lookup and its retry (was: a user turn, refused and not kept, recorded as sent; a store-lock retry could send it after switch-off) | `issue_507_operator_codex_pane_task_is_reported_not_sent` |

Not separately tested: the capacity-wait wake re-enters the same kill-switch check as the mailbox wake; the
agent-control inbox submits only when a work order arrives (a new trigger), and while the switch is on core refuses it.

Fail before / pass after (on `c4999b1265`, with the test-only `prepared_prompts` seam backported where a test needs
it): the goal-continuation, whip, three Claude-pane, switch-event, stopped-report and operator-pane tests fail before
and pass after; the mailbox, child-agent, durable-sleep, refused-turn and removed-pane tests are guards (those paths
already started nothing, or the helper is new).
