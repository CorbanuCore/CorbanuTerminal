# Case 5 — Select Fable 5.1 Max and make a live request

Status: **PASSED**

The supplied macOS arm64 candidate launched to the visible Corbanu Terminal trust prerequisite. After accepting it, `/model` offered `Claude Fable 5.1 Plan`. The visible effort flow led from `More reasoning…` to `Advanced Reasoning` → `Max`. The header and footer then both showed `claude-fable-5-1-plan max`, and the UI confirmed the model change.

I entered the complete literal prompt `Reply with exactly: FABLE_51_OK` and pressed Return separately. The assistant visibly returned exactly `FABLE_51_OK`. No missing-environment or provider-auth-command error was visible in the captured UI.

## Observed checks

- Passed — Fable 5.1 was offered in `/model`: [02-model-picker.txt](/Volumes/CorbanuDrive/Corbanu/.codex-work/luna-0911/case-05-replay/evidence/02-model-picker.txt)
- Passed — Max selection changed both visible model labels and produced a model-change confirmation: [06-fable-max-selected.txt](/Volumes/CorbanuDrive/Corbanu/.codex-work/luna-0911/case-05-replay/evidence/06-fable-max-selected.txt)
- Passed — Complete prompt was visibly typed before submission: [07-prompt-typed.txt](/Volumes/CorbanuDrive/Corbanu/.codex-work/luna-0911/case-05-replay/evidence/07-prompt-typed.txt)
- Passed — Live assistant response was exactly `FABLE_51_OK` under the Max model, with no visible auth/environment error: [09-request-complete.txt](/Volumes/CorbanuDrive/Corbanu/.codex-work/luna-0911/case-05-replay/evidence/09-request-complete.txt)

## Blockers

None observed for this case.

## Actions

- Started only the supplied `target` tmux session with the provided launch command.
- Accepted the visible startup trust prompt.
- Typed `/model`, pressed Enter, selected the visible Claude Fable 5.1 entry, selected `More reasoning…`, selected `Max`, and confirmed each visible menu.
- Typed the complete required prompt and pressed Enter as a separate action.
- Captured plaintext UI checkpoints and did not inspect credentials or non-UI internals.
- Sent Ctrl+D at the empty composer after the test; the dedicated `target` session exited normally.

## Test-instruction ambiguity

The human instruction says “Claude Fable 5.1,” while the visible catalog labels the offered entry “Claude Fable 5.1 Plan” and the resulting labels include `-plan`. This was treated as the matching Fable 5.1 model because it was the only visible Fable 5.1 entry and the UI consistently reported that selection.

## Limitations

- Result is bounded to the supplied candidate SHA-256 `4a8eba7b10199ea49aee42a720687b63cb4b1b2194c2ce164d1b9f6f88ee510e` on macOS arm64.
- Evidence is terminal UI capture from the dedicated tmux session, not native desktop UI evidence.
- No credential validity is inferred beyond the observed successful request.
