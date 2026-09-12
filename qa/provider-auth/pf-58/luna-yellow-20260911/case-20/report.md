# Case 20 — Continue your ordinary session

Status: **PASSED** for the observable acceptance criteria.

The supplied candidate launched in the ordinary TUI. `/permissions` opened with `Full Access (current)` selected; Escape returned to chat and made no change. `/model` displayed both the Claude Plan and OpenAI provider tabs. The visible OpenAI default, GPT-5.6-Sol, was selected with its default Low reasoning level, and the session visibly reported `gpt-5.6-sol low`. The harmless request `Reply with exactly: OK` received the assistant response `OK`.

Ctrl+D at an empty composer terminated the target normally. A fresh launch restored the selected `gpt-5.6-sol low` session. `/security` then displayed a read-only profile view with `Requested: Permissive (configuration only)`, reported that existing policies remained unchanged, and the selected Permissive profile inspection displayed `Nothing changed. Applying profiles is not available in this build.` Escape returned to chat. The restarted session retained the same YOLO permission header and selected model.

## Observed checks

- `/permissions` can be inspected and cancelled without a visible permission change — passed. Evidence: `03-permissions-open.txt`, `04-permissions-cancelled.txt`.
- Already-visible provider/model switching works through `/model` — passed. Evidence: `05-model-picker.txt`, `06-model-openai-tab.txt`, `07-openai-effort-or-active.txt`, `08-openai-selected.txt`.
- A harmless request reaches the selected provider and produces an assistant reply — passed. Evidence: `09-openai-request.txt`.
- Normal exit and restart preserve the observed ordinary session state — passed. Evidence: `11-restarted.txt`, `15-normal-exit.txt`.
- `/security` can be read-only inspected and closed without a policy change — passed. Evidence: `12-security-open.txt`, `13-security-inspected.txt`, `14-security-closed.txt`.

## Blockers

None for the stated observable checks.

## Instruction ambiguity

- “Repeat `/security`” did not specify whether Enter should inspect the highlighted profile, so the visible read-only flow was followed: open, Enter to inspect Permissive, then Escape.
- “Already-configured providers” did not name a target model/provider. The visible picker was used; the OpenAI default was selected, while the initially active Claude Plan model and both provider tabs were recorded.
- The phrase “normal Permissive test session” is distinct from the visible `/permissions` label `YOLO mode`/`Full Access (current)`; both labels were recorded without treating them as equivalent.

## Limitations

- The UI reported `Effective protection: unverified. Live security status is unavailable` and `Protected modes are blocked; required controls are not qualified`. Protected modes were not manufactured or tested, per the acceptance instruction.
- Evidence is limited to macOS arm64 tmux TUI observations. No native Applications shortcut/Desktop placement or other native-only behavior was claimed.
- Provider validity is claimed only for the selected OpenAI model because an assistant reply was observed. No credential values or login codes were accessed or recorded.

## Run metadata

- Candidate SHA-256: `4a8eba7b10199ea49aee42a720687b63cb4b1b2194c2ce164d1b9f6f88ee510e`
- Platform: macOS arm64
- Action history: `actions.log`
