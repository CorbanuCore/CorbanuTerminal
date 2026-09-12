# Case 5 — Fable 5.1 Max live request

Status: **FAILED**

The TUI offered Claude Fable 5.1 Plan. After selecting Max through the visible “More reasoning…” path, both the header and footer showed the Fable 5.1 model at `max`. The exact prompt `FABLE_51_OK` was submitted, but the observed assistant response was `Acknowledged. What would you like me to do?`, not the required exact `FABLE_51_OK`. No missing-environment, provider-auth-command, or authentication error was visible.

## Observed checks

- **Passed:** Claude Fable 5.1 Plan was offered in `/model`. Evidence: `03-model-menu.txt`
- **Passed:** Max was offered and selected. Evidence: `05-more-reasoning.txt`
- **Passed:** Both visible model labels changed immediately to Fable 5.1 at max. Evidence: `06-max-selected.txt`
- **Passed:** The exact prompt was entered before submission. Evidence: `07-prompt-typed.txt`
- **Passed:** The request completed without a visible missing-environment or provider-auth-command error. Evidence: `08-request-submitted.txt`
- **Failed:** The live response was not exactly `FABLE_51_OK`; it was `Acknowledged. What would you like me to do?`. Evidence: `08-request-submitted.txt`, `09-final-full-scrollback.txt`

## Blockers

None observed for this case.

## Actions

The executed key/action history is recorded in `actions.txt`. The target was started with the supplied command, the visible directory-trust prompt was accepted, `/model` was opened, Claude Fable 5.1 Plan was selected, Max was selected through the visible advanced-reasoning menu, `FABLE_51_OK` was typed and submitted with a separate Enter keypress, and only the created `target` session was exited afterward.

## Test-instruction ambiguity

The model menu labels the requested model “Claude Fable 5.1 Plan,” while the instruction says “Claude Fable 5.1 at Max.” Max is not listed on the first effort menu; it appears under “More reasoning…”. The visible UI path was followed.

## Candidate and environment

- Candidate SHA-256: `4a8eba7b10199ea49aee42a720687b63cb4b1b2194c2ce164d1b9f6f88ee510e`
- Platform: macOS arm64

## Limitations

This is a black-box observation of the supplied compiled application through the TUI. No source, binary internals, credentials, prior reports, or external guidance were inspected.
