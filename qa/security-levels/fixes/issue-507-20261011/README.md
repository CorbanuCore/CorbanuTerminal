# #507: the kill switch stops running work, Claude panes included (2026-10-11)

Travis's decisions (2026-10-11): the kill switch stops Claude panes too; turning it on interrupts the current turn in
every pane (a streaming reply is cancelled, its pending/running commands do not run or are stopped) with a clear
message; agent messages keep queuing until the next turn after it is turned off, and the text says so.

## Live check (tmux, GLM 5.3 Flash; Claude Plan for the Claude pane)

Setup: a private tmux socket, a disposable home with `CORBANU_TEST_NO_NATIVE_KEYRING=1`, the "Aggressive ships"
feature flags (contained Claude panes on), the real Z.AI key and the `claude-plan-test-token` from the vault helper on
the launch command only, a disposable `CLAUDE_CONFIG_DIR`, and Claude Code 2.1.296 installed in a scratch folder
(contained panes need 2.1.292 or later). Scripts: [launch.sh](scripts/launch.sh), [case507.sh](scripts/case507.sh),
[case507claude.sh](scripts/case507claude.sh). The kill switch is in the Aggressive grants view, so each run confirms
Aggressive first (no restart).

Builds (macOS arm64, debug): **before** `origin/main` `2e2e2f8d82`; **after** this branch at `031dbad0fc` (later
commits: one held task per pane, comments).

**Codex pane.** A `sleep 45 && touch ran-after-kill-switch.txt` command is running when the switch goes on.

| Step | Before | After |
| --- | --- | --- |
| Switch on while the command runs | the turn keeps running ([capture](captures/before-after-switch.txt)) | "Stopped: the security kill switch was turned on …" and "Turn stopped by the security kill switch." ([capture](captures/after-after-switch.txt)) |
| The command | finished and wrote its file ([marker](captures/before-marker.txt)) | stopped: no file ([marker](captures/after-marker.txt)) |
| "What is 2+2?" | refused (#445) ([capture](captures/before-turn-kill-on.txt)) | refused (#445) ([capture](captures/after-turn-kill-on.txt)) |
| Switch off, "What is 3+3?" | answered ([capture](captures/before-turn-kill-off.txt)) | answered ([capture](captures/after-turn-kill-off.txt)) |
| Kill-switch review text | "a reply already in progress finishes, and Claude panes are not covered" ([capture](captures/before-kill-on-review.txt)) | stops running work in every pane, Claude panes included; messages from other agents wait ([capture](captures/after-kill-on-review.txt)) |

**Claude pane** (Opus 5.5 Claude Plan, contained). A long essay is streaming when the switch goes on.

| Step | Before | After |
| --- | --- | --- |
| Switch on while Claude writes | Claude keeps running ([capture](captures/claude-before-after-switch.txt)) | the stop message, and the Claude turn is interrupted ([capture](captures/claude-after-after-switch.txt)) |
| A new Claude turn | not refused (the pane was still busy) ([capture](captures/claude-before-turn-kill-on.txt)) | "Not sent: the security kill switch is on …" ([capture](captures/claude-after-turn-kill-on.txt)) |
| Switch off, new Claude turn | answered ([capture](captures/claude-before-turn-kill-off.txt)) | answered ([capture](captures/claude-after-turn-kill-off.txt)) |

## Not covered live

- Held tasks from other agents to Claude panes (covered by `app::tests::issue_507_*`).
- Another Corbanu Terminal on the same home: it takes the switch when it next starts; its running work is not stopped.
