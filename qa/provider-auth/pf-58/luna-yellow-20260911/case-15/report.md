# Case 15 — Check API-key and browser guidance

Status: BLOCKED

Candidate SHA-256: `4a8eba7b10199ea49aee42a720687b63cb4b1b2194c2ce164d1b9f6f88ee510e`

Platform: macOS arm64

## Summary

The candidate launched in the dedicated TMUX session. The visible slash-command list had no provider or account-setup command. The `/model` picker exposed only `OpenAI Codex plan` and `Claude Plan` tabs; no API-key provider entry was available. Selecting the account-gated `GPT-6 Astra` row opened only a reasoning-level screen, with no login link or device-code guidance.

Because the required API-key provider entry screen was not exposed, the API-key guidance and conditional OpenAI setup guidance could not be inspected. Per the executor instructions, no hidden route or prerequisite was guessed. The picker was cancelled, the composer was restored, and `Ctrl+D` terminated the dedicated target session. No human acceptance is reported.

## Observed checks

- BLOCKED — API-key provider entry screen: no API-key provider was visible in the slash-command list or model-picker provider tabs. Evidence: [03_command_suggestions.txt](/Volumes/CorbanuDrive/Corbanu/.codex-work/luna-0911/case-15/evidence/03_command_suggestions.txt), [09_openai_provider_tab.txt](/Volumes/CorbanuDrive/Corbanu/.codex-work/luna-0911/case-15/evidence/09_openai_provider_tab.txt).
- BLOCKED — API-key guidance naming the environment variable, credential storage, and chat exclusion: the corresponding entry screen was not reachable. Evidence: [09_openai_provider_tab.txt](/Volumes/CorbanuDrive/Corbanu/.codex-work/luna-0911/case-15/evidence/09_openai_provider_tab.txt).
- BLOCKED — OpenAI account setup/login-link/device-code guidance: no setup action was visible; GPT-6 Astra opened the ordinary reasoning-level screen only. Evidence: [13_gpt6_selected.txt](/Volumes/CorbanuDrive/Corbanu/.codex-work/luna-0911/case-15/evidence/13_gpt6_selected.txt), [14_gpt6_entry.txt](/Volumes/CorbanuDrive/Corbanu/.codex-work/luna-0911/case-15/evidence/14_gpt6_entry.txt).
- BLOCKED/UNQUALIFIED — repeated keychain prompts: none appeared in the observed TUI captures, but the supplied native-keyring fallback does not qualify a Keychain-prompt pass. Evidence: [01_initial.txt](/Volumes/CorbanuDrive/Corbanu/.codex-work/luna-0911/case-15/evidence/01_initial.txt), [16_returned_composer.txt](/Volumes/CorbanuDrive/Corbanu/.codex-work/luna-0911/case-15/evidence/16_returned_composer.txt).
- PASSED — cancellation and normal TUI exit: Escape returned to the composer and `Ctrl+D` ended the target session without a forced kill. Evidence: [15_after_gpt6_cancel.txt](/Volumes/CorbanuDrive/Corbanu/.codex-work/luna-0911/case-15/evidence/15_after_gpt6_cancel.txt), [17_exit_status.txt](/Volumes/CorbanuDrive/Corbanu/.codex-work/luna-0911/case-15/evidence/17_exit_status.txt).

## Blockers

- No API-key provider entry screen was exposed by the visible UI.
- No OpenAI account setup action was exposed; therefore no browser link or device-code guidance was available to inspect.
- Native browser/desktop access is outside this TMUX interface.

## Actions

The complete key/action history is preserved in [actions.log](/Volumes/CorbanuDrive/Corbanu/.codex-work/luna-0911/case-15/evidence/actions.log). No credentials were entered, read, printed, or logged. The target was exited with `Ctrl+D`; no forced session termination was used.

## Limitations

- This was a black-box macOS arm64 TUI run only; no source, binary internals, prior reports, test implementations, credential files, project policies, or external guidance were inspected.
- Native Applications/Desktop placement and browser/native interfaces were not accessible through the supplied interface.
- Managed-token validity was observed only through the visible UI and was not assumed; no browser login was completed.
- The cloned profile's native-keyring fallback cannot establish a Keychain-prompt pass.

## Test-instruction ambiguity

The human instruction does not name the API-key provider or specify a visible route to its entry screen. The available UI exposed only the two subscription-plan tabs above, and the conditional OpenAI account-setup action was not present. The executor therefore treated the requested inspection as blocked rather than guessing an undisclosed shortcut.
