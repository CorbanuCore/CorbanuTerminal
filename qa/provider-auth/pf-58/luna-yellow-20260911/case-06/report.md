# Case 6 — Switch models and compare both labels

Status: **PASSED**

The supplied macOS arm64 candidate was exercised through the visible TUI using raw TMUX key events. `/model` was used to select the visible OpenAI `GPT-5.6-Sol` model at `low`, then `/model` was used to return to `Claude Fable 5.1 Plan` with the visible `Max` reasoning level.

## Observed checks

- **PASS** — The `/model` UI exposed a different provider and model, and the OpenAI selection completed visibly. Evidence: [04-openai-model-menu.txt](04-openai-model-menu.txt), [06-openai-selected.txt](06-openai-selected.txt).
- **PASS** — Immediately after the OpenAI selection, the boxed indicator showed `gpt-5.6-sol low` and the footer showed `GPT-5.6-Sol low`; no restart or extra prompt was observed. Evidence: [06-openai-selected.txt](06-openai-selected.txt).
- **PASS** — The return flow exposed `Claude Fable 5.1 Plan`, then the visible `Max` reasoning choice, and completed without a restart or extra prompt. Evidence: [08-fable-selection-reasoning-menu.txt](08-fable-selection-reasoning-menu.txt), [10-more-reasoning-menu.txt](10-more-reasoning-menu.txt), [11-fable-max-selected.txt](11-fable-max-selected.txt).
- **PASS** — Immediately after returning, the boxed indicator showed `claude-fable-5-1-plan max` and the footer showed `Claude Fable 5.1 Plan via claude-plan max`. The two labels agreed; no premature label change or mismatch was observed in the captured selection flow. Evidence: [05-after-openai-selection.txt](05-after-openai-selection.txt), [08-fable-selection-reasoning-menu.txt](08-fable-selection-reasoning-menu.txt), [11-fable-max-selected.txt](11-fable-max-selected.txt), [12-final-scrollback.txt](12-final-scrollback.txt).

## Actions

The executed key/action history is recorded in [action-history.txt](action-history.txt).

## Blockers

None observed for this TUI case.

## Test-instruction ambiguity

The instruction names the target as “Fable 5.1 Max,” while the UI presents the model as `Claude Fable 5.1 Plan` and exposes `Max` as a separate reasoning-level choice. This execution treated that visible combination as the requested target.

## Limitations

- Observation was limited to the supplied compiled candidate on macOS arm64; no Linux inference was made.
- Evidence consists of visible TMUX pane checkpoints, not source or implementation inspection and not frame-by-frame instrumentation.
- Native Applications/Desktop criteria were not part of this TUI case.

Candidate SHA-256: `4a8eba7b10199ea49aee42a720687b63cb4b1b2194c2ce164d1b9f6f88ee510e`
