# Black-box functional test report

- **Case:** 1 — Launch from the app shortcut
- **Status:** **BLOCKED**
- **Candidate SHA-256:** `4a8eba7b10199ea49aee42a720687b63cb4b1b2194c2ce164d1b9f6f88ee510e`
- **Platform:** macOS arm64

## Summary
The required launch action is the Corbanu Terminal Launcher shortcut in macOS Applications and requires observing Desktop 3 placement. The provided interface exposes no native Applications shortcut or macOS Desktop, so the acceptance criterion could not be exercised. A tmux-launched TUI was started only to follow the harness setup; it is not evidence for the shortcut or desktop-placement requirement.

## Observed checks

| Check | Status | Evidence |
|---|---|---|
| Launching the Applications shortcut opens a new Terminal window on Desktop 3 in the first free 2 × 3 card-grid position | **blocked** | `evidence/initial-pane.txt` (tmux harness pane only; no native shortcut/Desktop observation) |

## Blockers
- Native Applications shortcut and macOS Desktop placement are inaccessible in this test interface.
- The harness explicitly disallows substituting a tmux window for this criterion.

## Actions
See `evidence/actions.txt`. The provided target session was started, its pane was captured, and it was exited with raw tmux `C-c` keys. No credentials, login codes, purchases, wallet actions, or permission changes were performed.

## Limitations
- No conclusion about Desktop 3, grid placement, overlap, build identity, or shortcut behavior.
- No Linux inference; result is limited to the stated macOS arm64 environment.
- The visible pane was from the supplied tmux launch and cannot establish the native-launch requirement.

## Test-instruction ambiguity
None identified; the sole required interaction is native and unavailable through this interface.
