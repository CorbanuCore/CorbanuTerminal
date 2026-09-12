# Case 7 — Restart and confirm provider persistence

Status: **BLOCKED** (the exercised TUI subset passed)

## Summary

The supplied candidate was exercised through raw TMUX key input on macOS arm64. After accepting the one-time visible directory trust prompt, the first launch showed chat. Following exit, the supplied harness relaunches also opened directly to chat; no `/providers` screen appeared automatically.

Before and after relaunch, `/status` visibly showed the same selected model (`Claude Fable 5.1 Plan`), provider (`Claude Plan`), and `Claude Plan account connected`. The exact request visibly returned `RESTART_OK` after relaunch.

The full acceptance case is blocked because its required relaunch from the native app shortcut was not accessible through this interface. The supplied TMUX harness launch is recorded as a TUI subset only and is not treated as proof of the native-shortcut criterion.

## Observed checks

| Check | Result | Evidence |
|---|---|---|
| Relaunch reaches chat without an automatic `/providers` screen (harness subset) | PASS | `04-relaunch-chat.txt`, `07-relaunch-chat.txt` |
| Selected provider/model visibly match before and after relaunch | PASS | `02-status-before-request.txt`, `05-status-after-relaunch.txt`, `08-status-after-relaunch.txt` |
| Provider account remains visibly connected | PASS | `02-status-before-request.txt`, `05-status-after-relaunch.txt`, `08-status-after-relaunch.txt` |
| Exact request returns `RESTART_OK` after relaunch | PASS | `09-response-after-relaunch.txt` |
| Relaunch from native app shortcut | BLOCKED | Native Applications shortcut is inaccessible; `07-relaunch-chat.txt` documents the harness-launched subset only |

## Actions and cleanup

The complete raw-key/action history is in `actions.log`. Only the target session created for this run was exited. No credentials, login codes, purchases, wallet operations, or native permission changes were accessed.

## Test-instruction ambiguity

“Quit normally” did not specify a visible quit command or key. The visible slash-command list did not include a quit command; Ctrl-C at the idle TUI exited the target session, and the post-exit captures record no running target server. A harmless exploratory `/help` entry was rejected by the UI and did not affect the acceptance observations.

## Candidate

- SHA-256: `4a8eba7b10199ea49aee42a720687b63cb4b1b2194c2ce164d1b9f6f88ee510e`
- Platform: macOS arm64
