# Case 24 — Recover the affected account, not the current model

Status: **blocked**

## Summary

The raw-tmux TUI flow was exercised against the supplied candidate. Claude Fable 5.1 Plan/high remained the active model. `/providers` showed OpenAI as `Enabled · configured` and Claude Account as `Enabled · configured · current`; no live OpenAI authentication failure or “needs attention” state was observed. A harmless OpenAI request returned `OK`.

The OpenAI recovery flow was opened with `r`, sign-in was selected, the first device-login attempt was canceled, and the provider screen was visibly unchanged. Recovery was reopened and sign-in was launched again. Completion required an external browser/device at the displayed OpenAI device-login page, which is unavailable and not authorized for this run. The full acceptance case therefore remains blocked; no successful sign-in refresh could be assessed.

## Observed checks

- **Passed — working Claude state visible:** Claude Account remained `Enabled · configured · current`; the active model was Claude Fable 5.1 Plan/high. Evidence: [02-providers-open.txt](/Volumes/CorbanuDrive/Corbanu/.codex-work/luna-0911/case-24/evidence/02-providers-open.txt), [21-final-providers.txt](/Volumes/CorbanuDrive/Corbanu/.codex-work/luna-0911/case-24/evidence/21-final-providers.txt).
- **Passed — OpenAI recovery route:** OpenAI was selected, `r` opened “Recover OpenAI,” and Enter selected “Sign in to OpenAI again.” Evidence: [04-provider-list-before-recover.txt](/Volumes/CorbanuDrive/Corbanu/.codex-work/luna-0911/case-24/evidence/04-provider-list-before-recover.txt), [05-recovery-menu.txt](/Volumes/CorbanuDrive/Corbanu/.codex-work/luna-0911/case-24/evidence/05-recovery-menu.txt), [06-signin-first-open.txt](/Volumes/CorbanuDrive/Corbanu/.codex-work/luna-0911/case-24/evidence/06-signin-first-open.txt).
- **Passed — first cancellation changed no visible provider state:** After canceling the first login, the provider list again showed OpenAI configured and Claude configured/current. Evidence: [07-after-first-cancel.txt](/Volumes/CorbanuDrive/Corbanu/.codex-work/luna-0911/case-24/evidence/07-after-first-cancel.txt).
- **Passed — recovery reopened:** The same OpenAI recovery menu and sign-in choice were available again. Evidence: [08-recovery-reopened.txt](/Volumes/CorbanuDrive/Corbanu/.codex-work/luna-0911/case-24/evidence/08-recovery-reopened.txt), [09-signin-second-open.txt](/Volumes/CorbanuDrive/Corbanu/.codex-work/luna-0911/case-24/evidence/09-signin-second-open.txt).
- **Blocked — stated OpenAI failure precondition:** The setup-only live OpenAI request returned `OK`; no authentication failure or “needs attention” state was observed. Evidence: [14-openai-active.txt](/Volumes/CorbanuDrive/Corbanu/.codex-work/luna-0911/case-24/evidence/14-openai-active.txt), [15-openai-live-request.txt](/Volumes/CorbanuDrive/Corbanu/.codex-work/luna-0911/case-24/evidence/15-openai-live-request.txt), [21-final-providers.txt](/Volumes/CorbanuDrive/Corbanu/.codex-work/luna-0911/case-24/evidence/21-final-providers.txt).
- **Blocked — finish sign-in and verify refresh:** The reopened sign-in screen required an external browser/device. No successful sign-in, status refresh, or connection refresh was observed. Evidence: [09-signin-second-open.txt](/Volumes/CorbanuDrive/Corbanu/.codex-work/luna-0911/case-24/evidence/09-signin-second-open.txt).

## Blockers

- The provisioned OpenAI credential successfully answered the harmless live request, so the required authentication-failure scenario was not present.
- Completing the device login requires browser/native access that this executor cannot use or authorize.

## Actions

The executed key/action history is preserved in [actions.txt](/Volumes/CorbanuDrive/Corbanu/.codex-work/luna-0911/case-24/evidence/actions.txt). The temporary setup-only OpenAI model selection was undone; Claude Fable 5.1 Plan/high was restored before cleanup. Only the dedicated `target` tmux session was terminated.

## Test-instruction ambiguity

The instruction assumes an OpenAI authentication failure and says OpenAI should need attention, but the live UI showed only `configured` and the harmless OpenAI request succeeded. I treated that as an unavailable prerequisite and recorded the full case as blocked rather than manufacturing a failure.

## Limitations

- Evidence is plaintext tmux pane capture, not a native desktop screenshot.
- Browser/device login completion and the native Applications/Desktop shortcut were not accessible.
- No credential values or login codes are included; login-code lines in captures are redacted.
- Results are limited to the supplied macOS arm64 candidate.
