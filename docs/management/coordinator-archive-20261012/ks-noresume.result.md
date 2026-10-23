PR #602 is merged as `32deb10a9a`, with all 34 checks passing and none pending. Turning the kill switch off now starts nothing in any pane. A Claude pane's waiting task goes only with that pane's next turn: when someone types in it, or when new work is sent to it.

**What changed**
- **Claude panes:** a task sent while the switch is on waits. Each pane keeps the newest task, and the sender of an older one is still told it was superseded. When the pane's next turn starts, the waiting task is added to that turn, and the pane shows "Sent with this turn: …".
- **Texts:** the `/security` on/off review, the "Kill switch: on" line, the stop message and the docs (`config.md`, `orchestrate.md`, `claude-headless-panes.md`) now say nothing resumes until someone acts.

**Other paths checked (each has a test)**
- **Already safe:** Codex-pane agent messages, a child agent's messages, and a timer the agent set (woken by any incoming message) already started nothing after switch-off. I added tests that keep them that way.
- **Goal continuation:** it is now refused while the switch is on. Before, it ran a refused turn, which blocked the goal.
- **`/orchestrate` assignments and whips:** they pause as soon as the switch goes on and stay paused until `/orchestrate resume`.
- **Child reports:** a report sent while the switch is on, or from a Claude turn the switch stopped, starts no turn in the parent. The parent sees it on its next turn.
- **Codex panes opened in `/panes`:** a task sent to one while the switch is on is now refused, with a message. These panes have nowhere to hold a message, so the task can't wait. Before, it was refused but recorded as delivered, and a retry could send it after switch-off.
- **Removed or missing Claude panes:** a waiting task for a removed pane is dropped and its sender is told. A task for a pane that doesn't exist is refused.
- **Codex `/spawn` members:** a task sent while the switch is on now says it waits, and the pane is no longer shown as running.

**Checks**
- **Regression tests:** eight tests fail on the old base `c4999b1265` and pass now:
  - goal continuation;
  - whips;
  - three Claude-pane tests;
  - the switch-on/off events;
  - stopped-turn reports;
  - tasks for panes opened in `/panes`.

  The other tests are guards for paths that already started nothing.
- **Focused `just test` on the Mac:** 875/875 pass, plain and developer-accounting.
- **Clippy `-D warnings`:** clean on the Mac and on glitch (`-j10`), plain and developer-accounting.
- **Opus 5.5 High review:**
  - Round 1 requested changes: whips paused only at the next 45 s sweep, so one could fire right after switch-off. I fixed this.
  - Round 2 approved.
  - Round 3 re-approved after I applied its non-blocking notes.
- **tmux check:**
  - **Setup:** fresh debug binary in a disposable home. A `/spawn` crew with a GLM 5.3 Flash Troll and a Claude Code 2.1.296 Orc on Opus 5.5 Claude Plan.
  - **Before:** the Claude task ran as soon as the switch went off.
  - **After:**
    - For 45 s after switch-off nothing ran: no Claude turn file and no `codex-task.txt`.
    - Typing in the Troll pane ran its waiting task, which created `codex-task.txt`.
    - Typing in the Claude pane delivered "pf507-claude-task".
  - **Evidence:** in `qa/security-levels/fixes/issue-507-switch-off-20261012/`.
- **Video (80 s, leak-scanned):** [issue507-switch-off-restarts-nothing](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/issue-507-issue507-switch-off-restarts-nothing-4f41d8b859bc-2026-10-12.mp4)
- **Leaks:** the evidence folders contain neither the Z.AI key nor the Claude token.

**Overlap with agg-dec2 (#593, still open)**
#593 changes the same switch-on/off handling and still calls `send_held_claude_tasks`, which #602 removed. So it will fail to build until it merges main and makes these changes:
- call `on_kill_switch_turned_on()` / `on_kill_switch_turned_off()` from `on_kill_switch_changed`;
- make its "turned on elsewhere" text say turning the switch off restarts nothing;
- keep both edits in the `/security` "on" paragraph and its snapshot.

I posted this on #593 and in `agg-dec-off.coord.md`.

**Still open**
- **Small review notes left (non-blocking):**
  - Some test gaps: a report to a Codex parent while the switch is on, the same task re-sent while it waits, and the missing-pane sender check.
  - The stop message says agent tasks "wait" without excepting panes opened in `/panes`; `config.md` does mention it.
  - A whip a person resumes while the switch is on stays armed until the next sweep.
  - A Claude task already queued at the moment of switch-off can still go (a millisecond window).
- **Existing problems I hit while testing:**
  - A Claude Plan Orc can't join a `/spawn` crew unless `[agents] provider_allowlist` includes `external-claude:claudeplan`. I set that in the test config.
  - The crew then showed "Crew recovery is read-only until repaired".
- **Slip:** I ran `pkill -f "seq 1 40"` once on the Mac to stop my own polling loop. It ended only that loop.

**Cleanup:** the worktree, local branch, build target, scratch binaries, run folders and the scratch Claude Code install are removed on the Mac. The worktree, target, logs and script are removed on glitch.