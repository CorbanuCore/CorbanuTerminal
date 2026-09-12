# Case 12 — Open three concurrent launcher windows

## Result

**Blocked.** The supplied interface exposed one compiled application in a TMUX TUI session. The required native Applications shortcut and macOS Desktop 3 placement were inaccessible under the harness limits. I did not substitute TMUX windows or guess an undocumented shortcut, so the three-window/card-slot behavior was not observed.

## Observed checks

- **Passed — accessible TUI subset:** the target launched to the Corbanu Terminal TUI and remained responsive. The visible model selector changed from Claude Fable 5.1 Plan to Claude Fable 5 Plan, and a short request returned `READY`. Evidence: `02_tui_model_switch_request.txt`.
- **Blocked — full native criterion:** three concurrent launcher windows on Desktop 3, distinct card slots, independent usability, and no replacement/overlap/movement could not be exercised. Evidence checkpoint: `01_tui_start.txt` and `03_final_state.txt`.

## Blockers and limitations

- Native Applications shortcut access and macOS Desktop 3 placement are unavailable through this interface.
- Only the supplied target TMUX session was used; TMUX windows were not treated as launcher windows or card-slot evidence.
- No Linux inference was made; the result is limited to macOS arm64 observations.
- No credentials or browser login data were recorded.

## Actions

The complete executed key/action history is in `actions.log`. Named pane captures are `01_tui_start.txt`, `02_tui_model_switch_request.txt`, and `03_final_state.txt`.

## Test-instruction ambiguity

The supplied instruction does not define which “app shortcut” should be pressed. Because native shortcut access was unavailable, no shortcut was guessed.
