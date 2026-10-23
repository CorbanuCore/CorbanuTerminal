# Frozen independent design

Designer: `01a0a120-e9c7-73e1-b6b1-5fdb995ac102`. Original response follows.

Frozen original cases — v1. Instruction-only design; no repository, source, history, tests, or results inspected. These cases specify observable behavior without assuming an implementation.

For baseline cases, “successful interactive use” means the selected provider/model accepted an interactive request and returned a response.

| ID | Priority | Starting state | Action | Observable result |
|---|---|---|---|---|
| F01 | P0 | Default profile; last successful interactive use was Claude / Fable 5.1 Plan. | Exit normally; launch the app shortcut without an explicit override. | Claude / Fable 5.1 Plan is active before the first request and handles that request; GPT5.4 is not substituted. |
| F02 | P0 | Last successful interactive use was GPT5.4. | Select Claude / Fable 5.1 Plan, complete a request, exit, and launch the shortcut. | The newly used Claude provider and Fable model are remembered together. |
| F03 | P0 | Startup currently selects GPT5.4; a saved session used Claude / Fable 5.1 Plan with its own settings. | Resume that saved session normally and send a request. | Its Claude provider, Fable model, and saved applicable settings are restored; the request succeeds. |
| F04 | P0 | A GPT configuration uses `priority`; the saved Fable session has no such tier. | Resume Fable and send a request. | GPT’s tier does not appear as an active Fable setting or affect its request. The quoted unsupported-`priority` warning does not appear. |
| F05 | P0 | Last remembered choice is Claude / Fable 5.1 Plan. | Launch with an explicit, valid provider/model override using a supported user control. | The explicit choice takes effect and handles the request despite the remembered choice. |
| F06 | P0 | Named profile A last successfully used Fable; named profile B last successfully used GPT5.4. | Exit and launch A, then B, then A, without model overrides. | Each profile restores its own provider/model; using B does not replace A’s remembered choice. |
| F07 | P1 | Fable is the last successful choice. | Open model/provider selection, consider another choice, cancel, exit, and relaunch. | Fable remains active and remembered; cancellation does not commit the candidate. |
| F08 | P1 | Fable is the last successful choice; another provider cannot authenticate or initialize. | Attempt to switch to that provider; observe failure, exit, and relaunch without an override. | The failed attempt does not replace Fable as the last successful choice. Failure is visible rather than presented as a successful switch. |
| F09 | P1 | A working Fable session is active; another saved session is available. | Cancel a resume attempt; separately attempt a resume that fails before restoration completes. | Neither attempt changes the active session’s provider/model/settings or commits partially restored settings for the next launch. |
| F10 | P1 | A provider/model supports a tier, and that tier was intentionally selected for it. | Use it successfully, exit, and relaunch; separately resume its saved session. | The applicable tier is preserved in each relevant context. Preventing leakage does not discard valid tier choices. |
| F11 | P1 | A user intentionally configures `priority` for Fable itself, which does not advertise support for it. | Start or resume that configuration and send a request. | Unsupported-tier handling remains visible and consistent with the product’s policy. An intentional incompatible setting is distinguishable from an unrelated provider’s leaked setting. |
| F12 | P1 | Profile A has provider/model/tier settings different from profile B. | In B, apply an explicit valid override and complete a request; exit and launch A. | B honors the override; A retains its own provider/model/tier settings. |
| F13 | P1 | The remembered Fable provider is unavailable at the next launch. | Launch the shortcut without an override. | The unavailable remembered choice is clearly identified. Any fallback is visible, and startup failure alone does not silently replace the remembered successful choice. |

Ambiguities to resolve before extending or revising these frozen cases:

- **When remembrance commits:** selection, successful initialization, or first successful response? What happens after a first-request cancellation/failure, cancellation after earlier success, or an abrupt exit?
- **Resume’s effect on future startup:** after successfully resuming Fable, should the next fresh launch remember Fable or retain the last choice made outside resume?
- **Override scope and precedence:** which controls count as intentional overrides, do they persist after successful use, and what wins when an explicit override conflicts with saved-session settings?
- **Profile boundaries:** which profile does the shortcut select, is the default profile isolated from named profiles, and can a saved session be resumed across profiles?
- **Tier and fallback policy:** for intentionally unsupported tiers, is warning-and-omission the intended contract? For unavailable remembered providers, should startup stop, prompt, or visibly fall back?

## Addendum from designer after user-intent clarification

Original cases F01–F13 remain frozen and unchanged. The following disposition is additive.

| ID | Priority | Starting state / action | Observable result |
|---|---|---|---|
| A01 | P0 | Fresh-launch default is GPT5.4. Ordinarily resume saved Fable; receive server acknowledgement, send no model request, exit, and launch fresh. | Fable’s restored provider/model becomes the next default within the same profile. |
| A02 | P0 | An ordinary resume is cancelled or fails before server acknowledgement; then launch fresh. | The previously remembered default remains. |
| A03 | P0 | A remembered default exists. Launch with explicit CLI model, provider, or effort overrides; exercise each separately and valid combinations. Exit and launch without overrides. | Overrides govern that invocation only. The next launch retains the prior remembered defaults, including effort where applicable. |
| A04 | P0 | Profiles A and B have different defaults. Successfully acknowledge an ordinary Fable resume in A, then launch B and A fresh. | B retains its settings; A now defaults to Fable. No provider, model, effort, or tier crosses profiles. |

Disposition of existing cases:

- F03 gains A01’s persistence expectation for ordinary resume. Server acknowledgement is sufficient; no model response is required.
- F05 and F12 gain explicit one-off semantics for CLI overrides.
- F08 and F13 retain the existing authentication/fallback policy as their oracle. Their original wording is preserved, but any stronger fallback or persistence expectation requires confirmation against that policy.
- F04, F10, and F11 continue to distinguish unrelated tier leakage from intentional tier configuration.

Campaign boundary: no live model calls or credential use. Offline synthetic PTY and request/settings observations provide supporting evidence only; they do not establish live provider success. Original cases requiring model responses remain unexecuted under this authorization.

Remaining ambiguities: when a fresh interactive selection becomes remembered, behavior after abrupt exit, and precedence when explicit CLI overrides accompany resume.
