# #507: the kill switch reaches other running Corbanu Terminals on the home (2026-10-12)

Travis's decision (2026-10-12): when the kill switch is turned on, other Corbanu Terminal processes already running
on the same home pick it up while they run (they stop running work as #522 does and refuse new turns), not only at
their next start; turning it off reaches them too.

## Live check (two processes, one disposable home, GLM 5.3 Flash)

Setup: two private tmux sockets, one disposable home with `CORBANU_TEST_NO_NATIVE_KEYRING=1` and the "Aggressive
ships" feature flags, the Z.AI key from the vault helper on the launch command only. Two separate `corbanu`
processes (pids in [processes](captures/after-processes.txt)). Process B runs
`touch cmd-started.txt && sleep 45 && touch ran-after-kill-switch.txt`; once its first file exists, process A
confirms Aggressive and turns the kill switch on in `/security`. Script: [case507x.sh](scripts/case507x.sh).

Builds (macOS arm64, debug): **before** `origin/main` `b784f4d79e`; **after** this branch at `3287745cee` (after merging main with #602).

| Step | Before | After |
| --- | --- | --- |
| B's command is running when A turns the switch on | yes ([started](captures/before-started.txt)) | yes ([started](captures/after-started.txt)) |
| B after A's confirm | keeps running; the command finished and wrote its file ([marker](captures/before-marker.txt), [scrollback](captures/before-b-scrollback.txt)) | "turned on in another Corbanu Terminal", "Stopped: …", "Turn stopped by the security kill switch." 1.07 s after A's Enter, incl. A's save ([latency](captures/after-latency.txt), [screen](captures/after-b-after-switch.txt)); no file ([marker](captures/after-marker.txt)) |
| "What is 2+2?" in B | answered | "Not sent: the security kill switch is on …" ([capture](captures/after-b-turn-kill-on.txt)) |
| A turns it off; "What is 3+3?" in B | answered (it was never on in B) | B says it was turned off in another Corbanu Terminal (1.07 s), starts nothing by itself, and answers the next question ([capture](captures/after-b-turn-kill-off.txt)) |
| A's review text | "Another Corbanu Terminal on this home takes the switch when it next starts." | "Other Corbanu Terminals running on this home take it too, within about a second …" ([capture](captures/after-a-kill-on-review.txt)) |

Latency: the watch reads `security_state.json` on a file notification (at once) or the 1 s poll, so a change is
taken within about a second of the save; the tests require under 2 s.

## Not covered live

- Claude panes in the other process (covered by `app::tests::issue_507_kill_switch_from_another_process_stops_claude_panes_once`).
- Windows: not run live. The watch is the same code (`notify` uses ReadDirectoryChangesW there, with the same 1 s poll).
