# #445: with the kill switch on, model turns are refused (2026-10-11)

Travis's decision (2026-10-11): with the kill switch on, model turns are refused and not sent. The refusal message
names the kill switch and says how to turn it off. The kill-switch review text must match.

**Change.**

- A turn is refused before its input is recorded. The error says the kill switch is on and gives the `/security` steps
  to turn it off. The refused prompt is not kept, so it is not sent later either.
- Only the switch a person turned on counts. A session started from unreadable state, or a cold-resumed child agent,
  still has protected actions denied, but its turns run as before.
- A refused turn does not loop on pending input. No turn is started for agent messages while the switch is on; they wait
  and reach the model with the first turn after it is turned off.
- Every model request path also refuses at the client: turns, compaction, memory summaries and realtime.
- The kill-switch on/off reviews and the `/security` status line say that Corbanu's turns are refused while the switch
  is on. A reply already in progress finishes, and Claude panes are not covered.

## Live check (tmux, GLM 5.3 Flash)

Setup:

- A private tmux socket.
- A disposable home with `CORBANU_TEST_NO_NATIVE_KEYRING=1` and the "Aggressive ships" feature flags.
- The real Z.AI key from the vault helper.
- Scripts: [launch.sh](scripts/launch.sh), [case445.sh](scripts/case445.sh).

Steps:

1. Confirm Aggressive. The **restart** variant restarts at this point, as in AGG-22.
2. In `/security`, press `g` and turn the kill switch on.
3. Ask "What is 2+2?".
4. Turn the kill switch off and ask "What is 3+3?".

Builds (macOS arm64, debug):

- **before:** `origin/main` plus only the unrelated #443 change.
- **after:** this branch at `405b34fed7`.

| Step | Before | After |
| --- | --- | --- |
| Kill switch on: review | "… and the credential broker stays closed" ([capture](captures/before-kill-on-review.txt)) | adds "Corbanu sends nothing new to the model (each turn is refused …)" ([capture](captures/after-kill-on-review.txt)) |
| "What is 2+2?" | answered "4" ([capture](captures/before-turn-kill-on.txt)) | "■ Not sent: the security kill switch is on … this message was not kept. To turn it off, open /security, press g, choose "Turn the kill switch off" and confirm." ([capture](captures/after-turn-kill-on.txt)) |
| `/security` status | "New grants and protected actions … are refused" ([capture](captures/before-list-kill-on.txt)) | "Corbanu's model turns, new grants and … are refused; press g to turn it off." ([capture](captures/after-list-kill-on.txt)) |
| Kill switch off, then "What is 3+3?" | answered ([capture](captures/before-turn-kill-off.txt)) | answered "6"; the refused prompt was not sent ([capture](captures/after-turn-kill-off.txt)) |
| After a restart into Aggressive | (milestone AGG-22: answered) | refused, then answered after turning it off ([on](captures/after-restart-turn-kill-on.txt), [off](captures/after-restart-turn-kill-off.txt)) |

The kill-switch-off review after the change: [capture](captures/after-kill-off-review.txt).
